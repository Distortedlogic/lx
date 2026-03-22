# Goal

Add first-class schema validation for record types so agents can validate structured messages at system boundaries without manual field-by-field checking. Agents exchange records constantly — malformed messages should be caught early with clear error messages, not surface as `field not found` deep in a handler.

# Why

- Every agent dispatch function starts with ad-hoc field checking: `assert (msg.action != None)`, `assert (ok? msg.tool)`. This is repetitive, inconsistent, and produces bad error messages.
- lx traits define behavioral contracts (methods), but schema validation is about data shape contracts (field presence, types, value constraints). These are different concerns.
- The checker's type system operates at compile time. Schema validation operates at runtime on dynamic data — LLM outputs, parsed JSON, agent messages.

# What Changes

## New stdlib module: `std/schema`

New file `crates/lx/src/stdlib/schema.rs` implementing 4 functions:

**`schema.define name spec -> Schema`** — Creates a named schema from a spec record. Each field in the spec describes a field constraint:
```
ToolRequest = schema.define "ToolRequest" {
  tool: true
  args: true
  timeout_ms: {required: true  check: (v) v > 0}
  priority: {default: "normal"  one_of: ["low" "normal" "high"]}
  tags: {default: []}
}
```

Constraint types:
- `true` — shorthand for required field (must be present and non-None)
- `{required: Bool, default: Any, check: Func, one_of: [Any]}` — full constraint record

**`schema.validate schema data -> Result`** — Validates `data` against `schema`. Returns `Ok data_with_defaults` (fills in defaults for missing optional fields) or `Err {field: name, reason: Str}` for the first validation failure.

**`schema.validate_all schema data -> Result`** — Like `validate` but collects all errors: returns `Err [{field, reason}]` with all failures, not just the first.

**`schema.check schema data -> Bool`** — Returns true if data conforms, false otherwise. No error details.

## Usage pattern

```
use std/schema

MessageSchema = schema.define "Message" {
  action: true
  payload: true
  sender: {default: "anonymous"}
  priority: {default: "normal"  one_of: ["low" "normal" "high"]}
}

handle_message = (raw_msg) {
  msg = schema.validate MessageSchema raw_msg ?
  msg.action ? {
    "think" -> think msg.payload
    "plan" -> plan msg.payload
    _ -> error "unknown action: {msg.action}"
  }
}
```

## Implementation

Schemas are stored as `LxVal::Tagged { tag: intern("__schema"), values: Arc::new(vec![name_str, constraint_record]) }`. The `tag` field is `Sym` (interned string via `crate::sym::intern`), not a raw string. The constraint record is a `LxVal::Record` mapping field names to their parsed constraint records. Validation iterates the schema spec fields, checks each against the data record, applies defaults, runs custom `check` functions (via `crate::builtins::call_value_sync`), and validates `one_of` constraints. Records use `IndexMap<Sym, LxVal>` wrapped in `Arc`, constructed via the `record!` macro.

# Files Affected

**New files:**
- `crates/lx/src/stdlib/schema.rs` — schema define/validate/validate_all/check
- `tests/schema.lx` — tests for schema validation (the `tests/` directory does not exist yet — create it first)

**Modified files:**
- `crates/lx/src/stdlib/mod.rs` — register `mod schema;`, add `"schema" => schema::build()` to `get_std_module`, add `"schema"` to `std_module_exists` match

# Task List

### Task 1: Implement schema.define and constraint parsing

**Subject:** Parse schema spec records into internal schema representation

**Description:** Create `crates/lx/src/stdlib/schema.rs`. Implement `bi_define(name, spec)`:

1. Parse the spec record (use `args[1].require_record("schema.define", span)?`). For each field:
   - If value is `LxVal::Bool(true)`, create `FieldConstraint { required: true, default: None, check: None, one_of: None }`.
   - If value is a `LxVal::Record`, extract `required` (Bool, default false), `default` (Any, optional), `check` (Func, optional), `one_of` (List, optional).
   - If value is any other value, treat it as `{ default: value }` (optional field with that default).
2. Store each `FieldConstraint` as a `LxVal::Record` in a constraints `IndexMap<Sym, LxVal>`. Return a `LxVal::Tagged { tag: crate::sym::intern("__schema"), values: Arc::new(vec![name_str, constraints_record]) }`.

Register module in `stdlib/mod.rs`: add `mod schema;`, add `"schema" => schema::build()` to `get_std_module`, add `"schema"` to `std_module_exists`. Add `"define"` to `build()`. Use `crate::builtins::mk` for sync builtins.

Run `just diagnose`.

**ActiveForm:** Implementing schema.define and constraint parsing

---

### Task 2: Implement schema.validate and schema.validate_all

**Subject:** Validate data records against schema constraints

**Description:** In `crates/lx/src/stdlib/schema.rs`:

`bi_validate(schema, data)`:
1. Extract the schema by matching `args[0]` against `LxVal::Tagged { tag, values }` where `tag == crate::sym::intern("__schema")`. The constraint record is `values[1]`.
2. For each field in the constraint record:
   a. If required and missing from data (or None): return `Err {field: name, reason: "required field missing"}`.
   b. If missing but has default: add default to result record.
   c. If present and has `one_of`: check value is in the list. If not: return `Err {field, reason: "must be one of: ..."}`.
   d. If present and has `check` function: call it with the value using `crate::builtins::call_value_sync(f, val, span, ctx)`. If returns false: return `Err {field, reason: "validation check failed"}`.
3. Copy all fields from data (including ones not in schema) to result.
4. Return `Ok result_record`.

`bi_validate_all(schema, data)`: same logic but collects all errors into a list instead of returning on first failure.

Add `"validate"` and `"validate_all"` to `build()`. These are sync builtins (using `mk`) since `call_value_sync` handles async-in-sync via `block_in_place`.

Run `just diagnose`.

**ActiveForm:** Implementing validate and validate_all

---

### Task 3: Implement schema.check and write tests

**Subject:** Boolean check function and schema test suite

**Description:** Implement `bi_check(schema, data)`: run validation logic, return `true` if no errors, `false` otherwise.

Add `"check"` to `build()`.

Create `tests/schema.lx` (create the `tests/` directory first if it does not exist):
1. **Required field present** — define schema with required field, validate record with it, verify Ok.
2. **Required field missing** — validate record without required field, verify Err with field name.
3. **Default filling** — schema with default, validate record without that field, verify default applied.
4. **one_of constraint** — field with `one_of: ["a" "b"]`, validate with "a" (Ok) and "c" (Err).
5. **Custom check** — field with `check: (v) v > 0`, validate with 5 (Ok) and -1 (Err).
6. **validate_all** — multiple failures, verify all collected.
7. **schema.check** — verify returns Bool.
8. **Extra fields preserved** — data has fields not in schema, verify they pass through.

Run `just diagnose` and `just test`.

**ActiveForm:** Implementing check and writing schema tests

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.

---

## Task Loading Instructions

To execute this work item, load it with the workflow MCP:

```
mcp__workflow__load_work_item({ path: "work_items/SCHEMA_VALIDATION.md" })
```

Then call `next_task` to begin.
