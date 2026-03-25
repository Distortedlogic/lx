# Goal

Add Schema keyword desugaring. Schema fields declare type AND description so the generated JSON schema tells the model what each field means:

```lx
Schema GradeResult = {
  score: Int = "0-100 weighted score across all rubric categories"
  passed: Bool = "true if score >= threshold and no category failed"
  feedback: Str = "human-readable summary of what passed and failed"
}
```

Desugars to a `Trait GradeResult` with auto-injected `schema()` and `validate()` defaults. `schema()` returns a proper JSON-schema-compatible record:

```json
{
  "type": "object",
  "properties": {
    "score": {"type": "integer", "description": "0-100 weighted score across all rubric categories"},
    "passed": {"type": "boolean", "description": "true if score >= threshold and no category failed"},
    "feedback": {"type": "string", "description": "human-readable summary of what passed and failed"}
  },
  "required": ["score", "passed", "feedback"]
}
```

This is the only keyword that desugars to Trait instead of Class.

# Why

Every `ai.prompt_with { json_schema: ... }` hand-writes a JSON schema string. The model needs field descriptions to know what to put in each field — bare type names are not enough. Schema keyword auto-generates complete JSON schemas from typed, described field declarations. Schema desugars to Trait (not Class) because data contracts are lightweight record constructors, not heap-allocated objects.

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

1. Create `Stmt::Use` for `std/schema {Schema}`.
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

**schema() method:** Build a JSON-schema-compatible record. The trait body syntax `field: Type = "description"` is parsed as `FieldDecl { name, type_name, default: Some(description_expr), constraint }`. The `default` field holds the description string literal. The `type_name` holds the lx type name.

Type mapping for JSON schema: `Int` → `"integer"`, `Float` → `"number"`, `Str` → `"string"`, `Bool` → `"boolean"`, `List` → `"array"`, everything else → `"object"`.

For each `TraitEntry::Field(f)`, generate a property record: `{type: json_type_str, description: desc_str}`. Then wrap in the JSON schema envelope: `{type: "object", properties: {field1: prop1, field2: prop2, ...}, required: ["field1", "field2", ...]}`.

```rust
// For each field, build: {type: "integer", description: "0-100 score"}
let property_fields: Vec<RecordField> = entries.iter().filter_map(|e| {
    if let TraitEntry::Field(f) = e {
        let json_type = match f.type_name.as_str() {
            "Int" => "integer", "Float" => "number", "Str" => "string",
            "Bool" => "boolean", "List" => "array", _ => "object",
        };
        let type_val = gen_literal_str(json_type, span, arena);
        let mut prop_fields = vec![(intern("type"), type_val)];
        // If default is a string literal, use it as description
        if let Some(desc_id) = f.default {
            if let Expr::Literal(Literal::Str(parts)) = arena.expr(desc_id) {
                if let [StrPart::Text(desc_text)] = parts.as_slice() {
                    prop_fields.push((intern("description"), gen_literal_str(desc_text, span, arena)));
                }
            }
        }
        let prop_record = gen_record(prop_fields, span, arena);
        Some(RecordField::Named { name: f.name, value: prop_record })
    } else { None }
}).collect();
let properties = arena.alloc_expr(Expr::Record(property_fields), span);

// Build required array: ["score", "passed", "feedback"]
let required_elems: Vec<ExprId> = entries.iter().filter_map(|e| {
    if let TraitEntry::Field(f) = e { Some(gen_literal_str(f.name.as_str(), span, arena)) } else { None }
}).collect();
let required = gen_list(required_elems, span, arena);

// Envelope: {type: "object", properties: {...}, required: [...]}
let envelope = gen_record(vec![
    (intern("type"), gen_literal_str("object", span, arena)),
    (intern("properties"), properties),
    (intern("required"), required),
], span, arena);

let schema_fn = gen_func(&[], envelope, span, arena);
let schema_method = AgentMethod { name: intern("schema"), handler: schema_fn };
```

**validate(data) method:** Generate a function that checks all required field names exist in `data`:

```lx
(data) {
  missing = ["score", "passed", "feedback"] | filter (k) { not (data | keys | any? (== k)) }
  (missing | len) == 0 ? Ok data : Err {missing: missing}
}
```

Construct this AST: Func with param "data", body is a Block with binding `missing` (list of field name strings piped through filter) and a ternary returning Ok or Err.

Use the gen_ast helpers from Unit 4's `gen_ast.rs` for building expressions. Reference the Desugarer's existing `desugar_ternary` and `desugar_coalesce` in `desugar.rs` for how to build complex expression trees with arena allocation.

**Important:** When the desugaring strips the description from `default`, the generated TraitDecl's `entries` should have `default: None` on each FieldDecl — otherwise the runtime would try to use the description string as the field's default value, which is wrong. Clear `default` on each field entry before emitting the TraitDecl.

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
  name: Str = "the user's full name"
  age: Int = "age in years, must be positive"
  email: Str = "contact email address"
}

-- Schema acts as a record constructor (Trait behavior)
user = UserProfile {name: "Alice", age: 30, email: "a@b.com"}
assert user.name == "Alice"
assert user.age == 30

-- schema() returns JSON-schema-compatible record
s = user.schema ()
assert s.type == "object"
assert (s.properties | keys | len) == 3
assert s.properties.name.type == "string"
assert s.properties.age.type == "integer"
assert s.properties.name.description == "the user's full name"
assert s.properties.age.description == "age in years, must be positive"
assert (s.required | len) == 3

-- validate() checks required fields
good = user.validate {name: "Bob", age: 25, email: "b@c.com"}
assert (good | ok?)

bad = user.validate {name: "Charlie"}
assert (bad | err?)

-- schema() output can be passed directly to ai.prompt_with json_schema
-- (not tested here since it requires LLM, but the structure is correct)
schema_str = json.encode (user.schema ())
assert (schema_str | contains? "object")
assert (schema_str | contains? "integer")
```

Note: Schema methods are accessed via instance (`user.schema()`) not static call (`UserProfile.schema()`). Trait defaults in lx are injected as instance methods when a Class implements the trait. Since Schema desugars to a Trait, and Traits act as record constructors, calling `.schema()` on the constructed record invokes the default.

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
