# Goal

Add Schema keyword desugaring. `Schema GradeResult = { score: Int, passed: Bool, feedback: Str }` desugars to a `Trait GradeResult` with auto-injected `schema()` and `validate()` default methods. This is the only keyword that desugars to Trait instead of Class.

# Why

Every `ai.prompt_with { json_schema: ... }` hand-writes a JSON schema string. Schema keyword auto-generates `schema()` and `validate()` from typed field declarations. Schema desugars to Trait (not Class) because data contracts are lightweight record constructors, not heap-allocated objects.

# Critical fact: `requires` does not work

`TraitDeclData.requires` is stored but **never read at runtime**. The `inject_traits()` function in `interpreter/traits.rs` only processes the `traits` list on ClassDecl — it never looks at a trait's `requires` field. This means Schema cannot rely on trait inheritance via `requires: [Schema]`. Instead, the desugaring must directly inject `schema()` and `validate()` as default methods on the generated TraitDecl.

# What Changes

**Parser — `crates/lx/src/parser/stmt_keyword.rs`:**

Schema keyword needs trait-body syntax (`field: Type`) not class-body syntax (`field: defaultExpr`). The trait body parser already exists as `trait_body()` at `stmt.rs:222`. It returns `(Vec<TraitEntry>, Vec<AgentMethod>)`.

Branch the keyword parser: when keyword kind is Schema, parse body with `trait_body()` and store results in `data.trait_entries`. When keyword kind is anything else, parse with `class_body()` as before.

The `trait_body()` function is NOT currently public. It's defined inside `stmt.rs`. Make it `pub(super)` so `stmt_keyword.rs` can call it. The function signature is:

```rust
fn trait_body<'a, I>(
    expr: impl Parser<'a, I, ExprId, extra::Err<Rich<'a, TokenKind, Span>>> + Clone,
) -> impl Parser<'a, I, (Vec<TraitEntry>, Vec<AgentMethod>), extra::Err<Rich<'a, TokenKind, Span>>> + Clone
```

Note: `trait_body()` does NOT parse the enclosing `{ }` braces — the caller (`trait_parser`) handles those. However, `class_body()` (extracted in Unit 2) DOES parse the braces. So for Schema, the keyword parser must parse `{` then call `trait_body()` then parse `}` explicitly.

**Desugar — `crates/lx/src/folder/desugar.rs`:**

In `desugar_keyword`, add the Schema branch. Convert `KeywordDeclData` into `TraitDeclData`:

1. Create `Stmt::Use` for `pkg/core/schema {Schema}`.
2. Convert `data.trait_entries` into the TraitDecl's `entries` field.
3. Generate `schema()` default method: builds a record literal from the trait's field names and type names. For each `TraitEntry::Field(FieldDecl { name, type_name, .. })`, generate a record field `name: type_name_string`. The result is an expression like `{score: "Int", passed: "Bool", feedback: "Str"}`.
4. Generate `validate(data)` default method: checks that `data` is a Record and has all required field names. For each field in entries, check `data | keys | any? (== field_name)`. Return `Ok data` or `Err {missing: [...]}`.
5. Create `Stmt::TraitDecl(TraitDeclData { name, type_params, entries, methods: [], defaults: [schema_method, validate_method], requires: [], description: None, tags: [], exported })`.

**Validate — `crates/lx/src/folder/validate_core.rs`:**

Add Schema to the desugared-keyword assertion list.

# Files Affected

- `crates/lx/src/parser/stmt.rs` — Make trait_body pub(super)
- `crates/lx/src/parser/stmt_keyword.rs` — Branch for Schema body parsing
- `crates/lx/src/folder/desugar.rs` — Add Schema desugaring with generated methods
- `crates/lx/src/folder/validate_core.rs` — Add Schema to assertion
- `tests/keyword_schema.lx` — New test

# Task List

### Task 1: Make trait_body accessible to keyword parser

**Subject:** Change trait_body visibility and branch keyword parser for Schema

**Description:** Edit `crates/lx/src/parser/stmt.rs`. Change `fn trait_body` at line 222 from private to `pub(super) fn trait_body`.

Edit `crates/lx/src/parser/stmt_keyword.rs`. Modify the keyword parser to branch on Schema:

After matching the keyword kind and name, and parsing `=`:

- If keyword is Schema: parse `{` (just LBrace), call `super::stmt::trait_body(expr)`, parse `}` (just RBrace). Store the `(Vec<TraitEntry>, Vec<AgentMethod>)` result in `trait_entries: Some(entries)` and `methods: defaults`. Set `fields` to empty.
- If keyword is anything else: call `class_body(expr)` as before. Set `trait_entries: None`.

This requires restructuring the parser chain. Instead of a single `.then(class_body(expr))` at the end, use a conditional:

```rust
.then(
    keyword_kind.clone().then_ignore(just(TokenKind::LBrace))
    .ignore_then(trait_body_or_class_body)
    .then_ignore(just(TokenKind::RBrace))
)
```

The simplest approach: parse the keyword kind first, then decide which body parser to use. Since chumsky is combinator-based, the cleanest way is two branches in a choice: one for Schema keyword + trait body, one for other keywords + class body. Both produce the same output type.

**ActiveForm:** Branching keyword parser for Schema

---

### Task 2: Implement Schema desugaring

**Subject:** Add Schema branch to desugar_keyword

**Description:** Edit `crates/lx/src/folder/desugar.rs`. In `desugar_keyword`, add the Schema case.

The Schema keyword has `trait_entries: Some(entries)` with `TraitEntry::Field(FieldDecl { name, type_name, default, constraint })` items.

Generate two default methods:

**schema() method:** Build a record literal expression where each field in `entries` that is `TraitEntry::Field(f)` produces a record field `f.name: Literal::Str(f.type_name.as_str())`. Example: for `{ score: Int, passed: Bool }`, generate the expression `{score: "Int", passed: "Bool"}`.

```rust
let schema_fields: Vec<RecordField> = entries.iter().filter_map(|e| {
    if let TraitEntry::Field(f) = e {
        let name = f.name;
        let type_str = arena.alloc_expr(Expr::Literal(Literal::Str(vec![StrPart::Text(f.type_name.as_str().to_string())])), span);
        Some(RecordField::Named { name, value: type_str })
    } else { None }
}).collect();
let schema_record = arena.alloc_expr(Expr::Record(schema_fields), span);
let schema_fn = arena.alloc_expr(Expr::Func(ExprFunc {
    params: vec![],
    type_params: vec![],
    ret_type: None,
    guard: None,
    body: schema_record,
}), span);
let schema_method = AgentMethod { name: intern("schema"), handler: schema_fn };
```

**validate(data) method:** Generate a function that checks each field name exists in `data`. Build an expression like:

```lx
(data) {
  missing = ["score", "passed", "feedback"] | filter (k) { not (data | keys | any? (== k)) }
  (missing | len) == 0 ? Ok data : Err {missing: missing}
}
```

Construct this AST: Func with param "data", body is a Block with binding `missing` and a ternary.

This is verbose AST construction. Use the same arena allocation pattern as the Desugarer's existing `desugar_ternary` and `desugar_coalesce` functions for reference on how to build complex expression trees.

Assemble the TraitDecl:

```rust
let trait_decl = TraitDeclData {
    name: data.name,
    type_params: data.type_params,
    entries: data.trait_entries.unwrap_or_default(),
    methods: vec![],
    defaults: vec![schema_method, validate_method],
    requires: vec![],
    description: None,
    tags: vec![],
    exported: data.exported,
};
```

Return `vec![use_stmt, arena.alloc_stmt(Stmt::TraitDecl(trait_decl), span)]`.

**ActiveForm:** Implementing Schema desugaring

---

### Task 3: Update validate_core

**Subject:** Add Schema to desugared assertion

**Description:** Edit `crates/lx/src/folder/validate_core.rs`. Add `KeywordKind::Schema` to the list that must not survive into Core AST.

**ActiveForm:** Updating validate_core for Schema

---

### Task 4: Write Schema keyword test

**Subject:** Test Schema keyword end-to-end

**Description:** Create `tests/keyword_schema.lx`:

```lx
Schema UserProfile = {
  name: Str
  age: Int
  email: Str
}

-- Schema acts as a record constructor (Trait behavior)
user = UserProfile {name: "Alice", age: 30, email: "a@b.com"}
assert user.name == "Alice"
assert user.age == 30

-- schema() returns field type map
s = UserProfile.schema ()
assert (s | keys | len) == 3
assert s.name == "Str"
assert s.age == "Int"

-- validate() checks required fields
good = UserProfile.validate {name: "Bob", age: 25, email: "b@c.com"}
assert (good | ok?)

bad = UserProfile.validate {name: "Charlie"}
assert (bad | err?)
```

Note: `UserProfile.schema()` calls schema as a static method on the trait value itself. If the trait system doesn't support static method calls (only instance method calls), then test via an instance: `user.schema()`. Verify which pattern works by reading how trait defaults are accessed in the interpreter — `pkg/agent.lx`'s Agent trait defaults are called on instances, e.g., `a.think "prompt"` after `a = MyAgent {}`. So test via instance.

Run `just test`.

**ActiveForm:** Writing Schema test

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

1. **Call `complete_task` after each task.**
2. **Call `next_task` to get the next task.**
3. **Do not add, skip, reorder, or combine tasks.**
4. **Tasks are implementation-only.**

---

## Task Loading Instructions

```
mcp__workflow__load_work_item({ path: "work_items/KEYWORD_DESUGAR_3_SCHEMA.md" })
```
