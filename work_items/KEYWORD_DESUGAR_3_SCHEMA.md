# Goal

Add Schema keyword desugaring. `Schema GradeResult = { score: Int, passed: Bool, feedback: Str }` desugars to a `Trait GradeResult : [Schema] = { score: Int, passed: Bool, feedback: Str }` with auto-injected `schema()` and `validate()` methods from the Schema trait. This is the only keyword that desugars to Trait instead of Class.

# Why

- Every `ai.prompt_with { json_schema: ... }` call hand-writes a JSON schema string. Every tool needs parameter schemas. Every agent-to-agent message needs a data contract. Schema keyword auto-generates these from typed field declarations.
- Schema desugars to Trait (not Class) because data contracts are lightweight record constructors, not heap-allocated objects with mutable state.
- The parser infrastructure (keyword tokens, KeywordDeclData AST node) already exists from Unit 2. This unit only adds the Schema-specific desugaring codepath.

# What Changes

**Parser — `crates/lx/src/parser/stmt_keyword.rs`:**

Schema keyword needs trait-body parsing (field: Type syntax) instead of class-body parsing (field: defaultExpr syntax). Modify the keyword parser: when the keyword kind is `Schema`, delegate to the existing trait body parser instead of the class body parser. The trait body parser produces `(Vec<TraitEntry>, Vec<AgentMethod>)` which maps to TraitDeclData's `entries` and `defaults` fields.

**Desugar — `crates/lx/src/folder/desugar.rs`:**

In `transform_stmts`, handle `KeywordDecl { keyword: Schema, ... }`:

1. Create `Stmt::Use(UseStmt { path: [intern("pkg"), intern("core"), intern("schema")], kind: Selective([intern("Schema")]) })`.
2. Convert the KeywordDeclData into `Stmt::TraitDecl(TraitDeclData { name: data.name, type_params: data.type_params, entries: <from parsed trait body>, methods: [], defaults: [], requires: [intern("Schema")], description: None, tags: [], exported: data.exported })`.
3. The `requires: [intern("Schema")]` field tells the trait system that this trait extends Schema, inheriting `schema()` and `validate()` defaults.

Note: The KeywordDeclData currently stores `fields: Vec<ClassField>` and `methods: Vec<AgentMethod>` (class-body format). For Schema, the parser produces trait-body format instead. Either: (a) add a `trait_entries: Vec<TraitEntry>` field to KeywordDeclData, or (b) store a union/enum of body formats. Option (a) is simpler — add an optional `trait_entries` field that Schema populates instead of `fields`.

**Validate — `crates/lx/src/folder/validate_core.rs`:**

Remove the temporary pass-through for Schema keyword. It should now be desugared and absent from Core AST.

# Files Affected

- `crates/lx/src/parser/stmt_keyword.rs` — Branch on Schema for trait body parsing
- `crates/lx/src/ast/types.rs` — Add optional trait_entries field to KeywordDeclData
- `crates/lx/src/folder/desugar.rs` — Add Schema desugaring branch
- `crates/lx/src/folder/validate_core.rs` — Remove Schema pass-through
- `tests/keyword_schema.lx` — New test file

# Task List

### Task 1: Add trait_entries field to KeywordDeclData

**Subject:** Extend KeywordDeclData to support trait-body format for Schema

**Description:** Edit `crates/lx/src/ast/types.rs`. Add `pub trait_entries: Option<Vec<TraitEntry>>` field to `KeywordDeclData`. When this field is `Some`, it means the keyword was parsed with trait-body syntax (Schema). When `None`, the keyword uses class-body syntax (all other keywords). Also add `pub trait_methods: Option<Vec<TraitMethodDecl>>` for completeness. Default both to None in all existing parser code from Unit 2.

**ActiveForm:** Extending KeywordDeclData for Schema body format

---

### Task 2: Branch keyword parser for Schema body

**Subject:** Make keyword parser use trait body parsing for Schema keyword

**Description:** Edit `crates/lx/src/parser/stmt_keyword.rs`. After matching the keyword kind and reading the `=` token:

If keyword is Schema: delegate to the trait body parser (the same parser used by `trait_parser()` in `stmt.rs` — extract it into a shared helper if needed). Store results in `trait_entries` and `trait_methods` fields of KeywordDeclData. Set `fields` and `methods` to empty vecs.

If keyword is anything else: use class body parser as before. Set `trait_entries` and `trait_methods` to None.

**ActiveForm:** Branching keyword parser for Schema syntax

---

### Task 3: Implement Schema desugaring

**Subject:** Add Schema desugaring branch to transform_stmts

**Description:** Edit `crates/lx/src/folder/desugar.rs`. In the `transform_stmts` method, add a branch for `KeywordDecl { keyword: Schema, ... }`:

1. Allocate `Stmt::Use` for `pkg/core/schema` importing `Schema`.
2. Build `TraitDeclData` from the KeywordDeclData: `name` from data.name, `type_params` from data.type_params, `entries` from `data.trait_entries.unwrap_or_default()`, `methods` from `data.trait_methods.unwrap_or_default()`, `defaults` empty, `requires: vec![intern("Schema")]`, `description: None`, `tags: vec![]`, `exported` from data.exported.
3. Allocate `Stmt::TraitDecl(trait_data)`.
4. Return both Use and TraitDecl statements.

**ActiveForm:** Implementing Schema desugaring

---

### Task 4: Update validate_core for Schema

**Subject:** Remove Schema pass-through in validate_core

**Description:** Edit `crates/lx/src/folder/validate_core.rs`. Add Schema to the list of keyword kinds that must not survive into Core AST. It should now be desugared alongside the 8 simple keywords.

**ActiveForm:** Updating validate_core for Schema

---

### Task 5: Write Schema keyword test

**Subject:** Create test file validating Schema keyword works end-to-end

**Description:** Create `tests/keyword_schema.lx`:

Define `Schema UserProfile = { name: Str, age: Int, email: Str }`. This should desugar to a Trait with Schema's `schema()` and `validate()` defaults.

Instantiate: `user = UserProfile { name: "Alice", age: 30, email: "alice@example.com" }`. Assert `user.name == "Alice"`.

Call `user_schema = UserProfile.schema()` (if Schema trait provides a static method) or test via `validate`: `result = UserProfile.validate { name: "Bob", age: 25, email: "bob@test.com" }`. Assert result is Ok.

Test validation failure: `bad = UserProfile.validate { name: "Charlie" }`. Assert result is Err with missing fields.

Run `just test`.

**ActiveForm:** Writing Schema keyword test

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
mcp__workflow__load_work_item({ path: "work_items/KEYWORD_DESUGAR_3_SCHEMA.md" })
```

Then call `next_task` to begin.
