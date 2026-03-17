# Goal

Fix 8 language-level issues discovered during flow satisfaction testing (documented in `agent/FLOW_TESTING_FINDINGS.md`). These fixes make lx actually usable for writing real programs — currently every non-trivial flow hits multiple parse/runtime issues that require ugly workarounds.

# Why

- **Finding #2 (record field values)** is the single most common issue. Every record literal with a function call, pipe, or prefix operator in a field value breaks. This forces agents to extract every expression into a temp binding before every record — 2-3x the code for no reason.
- **Finding #8 (missing field crashes)** means `record.field ?? default` — the most natural lx pattern for optional config — is a trap. It crashes instead of coalescing. Every flow lib that accepts options needs `get "field" record` instead.
- **Finding #10 (invoke_flow closure breakage)** means `test.run` can't test flows that import modules. All test flows must be self-contained, defeating the purpose of testing the actual library code.
- **Finding #6/#7 (spread binding power)** means `[..f x y]` doesn't work — you can't spread a function call result inline.
- **Finding #11 (single `..` parent)** means test files can't import from two directories up.
- **Finding #9 (no empty record)** means optional-config functions need dummy placeholder fields.
- **Finding #12 (CWD-relative flow paths)** means flow paths in test specs must be project-root-relative instead of spec-relative.

Fixing these removes the workarounds from all existing flow lib files, test flows, and spec files — and prevents every future lx program from hitting the same walls.

# What Changes

## Record field value parsing (Findings #2, #4)

In `crates/lx/src/parser/prefix_coll.rs`, line 54: `self.parse_expr(0)?` for record field values currently works at BP 0 — which should consume everything. But `is_application_candidate` (line 72-74 of `helpers.rs`) blocks juxtaposition inside `collection_depth > 0`. This means `{key: f arg1 arg2}` parses as `{key: f}` with `arg1` and `arg2` as errors.

Fix: For record field values (between `:` and the next field or `}`), temporarily reset `collection_depth` to 0 so function application works, then restore it. This allows `{type: get "type" entry ?? "unknown"}` to parse as `{type: (get "type" entry ?? "unknown")}`.

The same approach must NOT be used for list elements or record field names — only for the value expression after `:` in a record field.

## List/record spread binding power (Findings #6, #7)

In `crates/lx/src/parser/prefix_coll.rs`:
- Line 13: `self.parse_expr(32)?` for list spreads — BP 32 is just above pipe. But juxtaposition at BP 32 is blocked by `is_application_candidate` inside collections. Same fix: temporarily reset `collection_depth` to 0 for the spread expression.
- Line 43: `self.parse_expr(32)?` for record spreads — same fix.

After this fix, `[..scan_entries entries pats]` will parse the spread value as `scan_entries entries pats` (full function application).

## Missing field returns None instead of crash (Finding #8)

In the interpreter's field access handler (in `crates/lx/src/interpreter/eval.rs` or `mod.rs`), when `.field` is used on a Record and the field doesn't exist, currently an `LxError` is thrown. Change this to return `Value::None` instead. This makes `record.field ?? default` work naturally — `.field` returns `None`, `??` coalesces to default.

This is a behavior change but is consistent with how `get` works and matches what agents expect. The `get` function already returns `None` for missing fields. Making `.field` consistent removes the split personality.

## Empty record literal (Finding #9)

Currently `{}` is an empty block (`Unit`). Add `{:}` as the empty record literal syntax. In the parser's `parse_block_or_record`, when the next token after `{` is `:` followed by `}`, emit `Expr::Record(vec![])`.

## Multi-level `..` parent in module paths (Finding #11)

In `crates/lx/src/interpreter/modules.rs`, `resolve_module_path` only handles `path[0] == ".."` for one parent step. Change to: consume all leading `".."` segments, calling `.parent()` for each one.

The lexer also needs updating — `use ../../path` tokenizes `..` as `DotDot` which is the range operator. The module path lexer/parser needs to handle consecutive `..` segments separated by `/`.

## `test.run` source-relative flow path resolution (Finding #12)

In `crates/lx/src/stdlib/test_invoke.rs`, `invoke_flow` resolves the flow path as `std::path::Path::new(flow_path)` — relative to CWD. Change to: if the path starts with `./` or `../`, resolve relative to a `source_dir` passed from the calling interpreter. Store the calling interpreter's `source_dir` in the spec record (added during `bi_spec` construction) and pass it through to `invoke_flow`.

## Fix `invoke_flow` closure env chain (Finding #10)

This is the critical fix. When `invoke_flow` creates a fresh `Interpreter::new`, execs the flow, then calls an exported function via `call_value`, the closure env chain breaks for functions from imported modules that use lambdas.

Root cause investigation: `call_value` creates `Interpreter::with_env(&lf.closure, ctx)` which sets `source_dir: None` and `module_cache: fresh`. The closure's env chain is correct (it captured the env from the inner interpreter's module execution). But `eval_expr` on the body creates sub-expressions that may need the interpreter's `source_dir` or `module_cache`.

Fix approach: Instead of using `call_value` from outside the interpreter, invoke the exported function from within the inner interpreter itself. After `exec`, keep the inner interpreter alive and call the function through `interp.eval_expr` by constructing an `Apply` AST node. This keeps the interpreter's `source_dir`, `module_cache`, `source` all intact.

Alternative simpler fix: Make `call_value` propagate `source_dir` and `module_cache` from the closure. Store these in `LxFunc` alongside the closure env.

# Files Affected

**Modified files:**
- `crates/lx/src/parser/prefix_coll.rs` — record field value + spread BP fix
- `crates/lx/src/parser/helpers.rs` — potentially adjust collection_depth logic
- `crates/lx/src/interpreter/eval.rs` or `mod.rs` — missing field returns None
- `crates/lx/src/interpreter/modules.rs` — multi-level `..` parent
- `crates/lx/src/stdlib/test_invoke.rs` — source-relative path + invoke_flow closure fix
- `crates/lx/src/stdlib/test.rs` — store source_dir in spec record
- `crates/lx/src/builtins/call.rs` — potentially propagate interpreter state through closures

**Test files:**
- `tests/16_edge_cases.lx` — add tests for record field values, empty record, spreads
- `tests/06_collections.lx` or new test — record/list construction edge cases

# Task List

### Task 1: Fix record field value parsing to allow function calls

**Subject:** Allow juxtaposition-based function application inside record field values

**Description:** In `crates/lx/src/parser/prefix_coll.rs`, in the `parse_record` method, when parsing the value expression for a field (line 54, after consuming the `:`), save `self.collection_depth`, set it to 0, call `self.parse_expr(0)?`, then restore `collection_depth`.

This allows `{type: get "type" entry ?? "unknown"}`, `{not_empty: !audit.is_empty str}`, and `{gaps: list | join ", "}` to parse correctly — the function application, prefix operators, and pipe chains are all consumed as part of the field value.

Add tests to `tests/16_edge_cases.lx`:
```lx
r = {x: [1 2 3] | sum}
assert (r.x == 6) "record field with pipe"
r2 = {x: to_str 42}
assert (r2.x == "42") "record field with function call"
```

Run `just diagnose` and `just test`.

**ActiveForm:** Fixing record field value parsing

---

### Task 2: Fix list and record spread to consume function application

**Subject:** Allow spreads to contain function calls

**Description:** In `crates/lx/src/parser/prefix_coll.rs`:
- In `parse_list`, line 13: when parsing a spread expression, save `collection_depth`, set to 0, call `self.parse_expr(32)?`, restore. This allows `[..f x y]` to parse as spreading the result of `f x y`.
- In `parse_record`, line 43: same fix for record spreads.

Add tests:
```lx
f = () [1 2 3]
assert ([..f () 4 5] == [1 2 3 4 5]) "list spread with function call"
```

Run `just diagnose` and `just test`.

**ActiveForm:** Fixing spread binding power

---

### Task 3: Make missing record field return None instead of crash

**Subject:** Change `.field` on Record to return None for absent fields

**Description:** Find the field access handler in the interpreter (likely in `crates/lx/src/interpreter/eval.rs` or the match arm for `Expr::FieldAccess` in `mod.rs`). Currently when a Record doesn't have the accessed field, it returns `Err(LxError::runtime("field 'x' not found", span))`. Change this to return `Ok(Value::None)` instead.

This makes `record.missing_field ?? default` work naturally. The `get` builtin already returns `None` for missing fields — this makes `.field` consistent.

Update existing tests if any assert on field-not-found errors. Add tests:
```lx
r = {x: 1}
assert (r.x == 1) "present field works"
assert (r.y == None) "missing field returns None"
assert (r.y ?? 42 == 42) "missing field coalesces"
```

Run `just diagnose` and `just test`.

**ActiveForm:** Making missing field return None

---

### Task 4: Add empty record literal syntax `{:}`

**Subject:** Parse `{:}` as an empty Record

**Description:** In `crates/lx/src/parser/prefix_coll.rs`, in `parse_block_or_record`, after `self.skip_semis()`, add a check: if `*self.peek() == TokenKind::Colon` and the token after that is `TokenKind::RBrace`, consume both and return `SExpr::new(Expr::Record(vec![]), ...)`. This makes `{:}` the empty record literal.

Add tests:
```lx
empty = {:}
assert (empty == {:}) "empty record"
assert (keys empty | len == 0) "empty record has no keys"
```

Run `just diagnose` and `just test`.

**ActiveForm:** Adding empty record literal

---

### Task 5: Support multi-level `..` parent in module paths

**Subject:** Allow `use ../../path` and deeper relative imports

**Description:** This requires changes in both the lexer and the module path resolver.

**Lexer:** In `crates/lx/src/lexer/mod.rs`, when lexing a `use` statement's module path, handle consecutive `..` segments separated by `/`. Currently `../lib/guard` is tokenized as `[DotDot, Slash, Ident("lib"), Slash, Ident("guard")]`. For `../../examples/foo`, we need `[DotDot, Slash, DotDot, Slash, Ident("examples"), Slash, Ident("foo")]`. Check if this already tokenizes correctly — if so, only the parser/resolver needs fixing.

**Parser:** In the `use` statement parser, when building the path segments, handle multiple leading `".."` segments.

**Resolver:** In `crates/lx/src/interpreter/modules.rs`, `resolve_module_path`: change the `path[0] == ".."` check to a loop that consumes all leading `".."` segments, calling `base.parent()` for each.

Add test in `tests/11_modules/` or a new test that imports from two levels up.

Run `just diagnose` and `just test`.

**ActiveForm:** Supporting multi-level parent imports

---

### Task 6: Fix `invoke_flow` to preserve interpreter state for closures

**Subject:** Make module-imported closures work when called from `test.run`

**Description:** The critical fix. In `crates/lx/src/stdlib/test_invoke.rs`, `invoke_flow` currently:
1. Creates `Interpreter::new`
2. Calls `exec` (which loads modules, creates closures)
3. Finds exported function via `collect_flow_exports`
4. Calls it via `call_value` (which creates a NEW `Interpreter::with_env`)

Step 4 is the problem. `Interpreter::with_env` gets `source_dir: None` and `module_cache: fresh`. The closure env chain is correct, but the interpreter context is wrong.

Fix: Keep the inner interpreter from step 2 alive. After `exec`, instead of extracting the function and calling via `call_value`, call the function directly through the inner interpreter:

```rust
let exports = collect_flow_exports(&program, &interp);
let entry = exports.get("run").or_else(|| exports.get("main"))...;
// Instead of call_value(entry, input, span, ctx):
let apply_expr = /* construct Apply AST node: entry(input) */;
interp.eval_expr(&apply_expr)
```

This keeps `source_dir`, `module_cache`, `source` all intact. The inner interpreter can resolve all module references, and closures from imported modules execute in the correct env chain.

Alternative approach if constructing AST nodes is awkward: Modify `call_value` to accept optional `source_dir` and `module_cache` parameters, and propagate them to `with_env`.

Run `just diagnose` and `just test`. Then verify: create a test flow that imports a module using lambdas in higher-order functions, and run it via `test.run`.

**ActiveForm:** Fixing invoke_flow closure resolution

---

### Task 7: Make `test.run` resolve flow paths relative to source file

**Subject:** Resolve flow: path relative to the spec file, not CWD

**Description:** In `crates/lx/src/stdlib/test.rs`, in `bi_spec`, capture the interpreter's `source_dir` and store it in the spec record as `__source_dir`. In `test_invoke.rs`, `invoke_flow`, if the flow path starts with `./` or `../`, resolve it relative to `__source_dir` from the spec record. If the path doesn't start with `.`, treat it as CWD-relative (backward compatible).

Pass the `__source_dir` through from `bi_run` → `run_scenarios` → `invoke_flow`.

The challenge: `bi_spec` is a `BuiltinFn` which receives `&Arc<RuntimeCtx>` but not the interpreter's `source_dir`. Options:
1. Store `source_dir` in `RuntimeCtx` (cleanest — add an `Option<PathBuf>` field)
2. Use a thread-local to pass it from the interpreter
3. Accept CWD-relative paths only (current behavior, least intrusive)

If option 1, add `source_dir: Option<PathBuf>` to `RuntimeCtx`, set it in `Interpreter::new`, and read it in `bi_spec`.

Run `just diagnose` and `just test`.

**ActiveForm:** Making flow paths source-relative

---

### Task 8: Update flow libs and test flows to use fixed syntax

**Subject:** Remove all workarounds from flow libs and test flows now that the parser/runtime is fixed

**Description:** After Tasks 1-7 are complete, go back through all modified files and remove the workarounds:

In `flows/lib/guard.lx`:
- Restore `opts.patterns ?? default_patterns` (instead of `get "patterns" opts ?? ...`)
- Restore `[..scan_entries entries pats ..detect_loops entries threshold ...]` (instead of temp bindings + spread)
- Restore inline `re.is_match` calls if the record field fix covers it

In `flows/lib/transcript.lx`:
- Restore `{type: get "type" entry ?? "unknown" ...}` as a record literal (instead of temp bindings)

In `flows/lib/catalog.lx`, `guidance.lx`:
- Restore complex expressions in record field values

In `flows/tests/security_audit_flow.lx` and `defense_layers_flow.lx`:
- Convert from self-contained to importing `../lib/guard` and `../lib/transcript` directly (now that Finding #10 is fixed)
- Remove inlined scan_text, parse_transcript functions

In `flows/tests/security_audit/main.lx` and `defense_layers/main.lx`:
- Use `{:}` for empty records where needed
- Use `record.field ?? default` instead of `get "field" record ?? default`

Run `just test` and `just test-flows`.

**ActiveForm:** Removing workarounds from flow code

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
mcp__workflow__load_work_item({ path: "work_items/LX_LANGUAGE_FIXES.md" })
```

Then call `next_task` to begin.
