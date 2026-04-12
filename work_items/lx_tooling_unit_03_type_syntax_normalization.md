---
unit: 3
title: Type Syntax Normalization
scope: lx-ast, lx-parser, lx-checker, lx-fmt, lx-desugar, lx-value, lx-eval
depends_on: lx_tooling_unit_02_phase_hardening
optional: false
---

## Goal
Represent every user-written type position with `TypeExprId` so parsing, checking, formatting, and desugaring all operate on one type-syntax path.

## Dependency Contract
Assume Unit 02 is merged and the phase boundary is hardened. Do not redesign traversal or semantics in this unit except for compile-fix updates required by the AST field changes below.

## Verified Preconditions
- `Field<D, C>` in `crates/lx-ast/src/ast/types.rs` currently stores `pub type_name: Sym`.
- `MethodSpec<F>` in the same file currently stores `pub output: Sym`.
- Trait field parsing in `crates/lx-parser/src/parser/stmt_trait.rs` still uses `type_name()` for field types.
- `trait_parser` in `crates/lx-parser/src/parser/stmt_trait.rs` currently sets `methods: vec![]`; there is no parser path today that constructs `TraitMethodDecl` from source text.
- Formatter emission still prints raw names with `.as_str()`:
  - `emit_trait_decl` in `crates/lx-fmt/src/formatter/emit_stmt.rs`
  - `emit_keyword_decl` in `crates/lx-fmt/src/formatter/emit_stmt_keyword.rs`
- Checker trait handling still resolves field types via `named_to_type(f.type_name.as_str())` in `crates/lx-checker/src/visit_stmt.rs`.
- Schema desugaring still matches raw strings in `crates/lx-desugar/src/folder/desugar_schema.rs`:
  - `lx_type_to_json_type`
  - `default_for_type`
  - direct `f.type_name.as_str()` calls
- Runtime values currently reuse the AST generics instead of owning runtime-native containers:
  - `crates/lx-value/src/value/mod.rs` aliases `FieldDef = Field<LxVal, ConstraintExpr>`
  - `crates/lx-value/src/value/mod.rs` aliases `TraitMethodDef = MethodSpec<FieldDef>`
- Runtime construction and use sites currently copy AST `type_name` and `output` fields directly:
  - `crates/lx-eval/src/interpreter/type_apply.rs::eval_field_decl`
  - `crates/lx-eval/src/interpreter/exec_stmt.rs` trait-method construction path
  - `crates/lx-eval/src/interpreter/trait_apply.rs` runtime trait checking
  - `crates/lx-eval/src/stdlib/trait_ops.rs` runtime trait reflection helpers

## Files To Create Or Change
- `crates/lx-ast/src/ast/types.rs`
- `crates/lx-ast/src/ast/walk_impls.rs`
- `crates/lx-ast/src/ast/display.rs`
- `crates/lx-parser/src/parser/stmt_trait.rs`
- `crates/lx-parser/src/parser/stmt_keyword.rs`
- `crates/lx-parser/src/parser/type_ann.rs`
- `crates/lx-parser/tests/surface_parse_regressions.rs`
- `crates/lx-checker/src/visit_stmt.rs`
- `crates/lx-fmt/src/formatter/emit_stmt.rs`
- `crates/lx-fmt/src/formatter/emit_stmt_keyword.rs`
- `crates/lx-desugar/src/folder/desugar_schema.rs`
- `crates/lx-value/src/value/mod.rs`
- `crates/lx-eval/src/interpreter/type_apply.rs`
- `crates/lx-eval/src/interpreter/exec_stmt.rs`
- `crates/lx-eval/src/interpreter/trait_apply.rs`
- `crates/lx-eval/src/stdlib/trait_ops.rs`
- `crates/lx-desugar/tests/surface_to_core_regressions.rs`
- `crates/lx-fmt/tests/format_regressions.rs`
- `crates/lx-checker/tests/checker_regressions.rs`

## Exact Structs And Functions To Inspect Or Change
- `Field<D, C>`
- `FieldDecl`
- `MethodSpec<F>`
- `TraitMethodDecl`
- `trait_body`
- `keyword_parser` support code in `stmt_keyword.rs`
- `type_parser`
- `Checker::resolve_type_ann`
- `Checker::check_stmt`
- `Formatter::emit_trait_decl`
- `Formatter::emit_keyword_decl`
- `lx_type_to_json_type`
- `default_for_type`
- `FieldDef`
- `TraitMethodDef`
- `render_type_expr`
- `Interpreter::eval_field_decl`
- `Interpreter::apply_trait_fields`
- `method_to_record`

## Mechanical Task List
1. In `crates/lx-ast/src/ast/types.rs`, change `Field<D, C>::type_name` from `Sym` to `TypeExprId`.
2. In the same file, change `MethodSpec<F>::output` from `Sym` to `TypeExprId`.
3. Rebuild the affected type aliases so `FieldDecl` and `TraitMethodDecl` continue to compile with the new field types.
4. In `crates/lx-ast/src/ast/walk_impls.rs`, update `recurse_field_decl` so it transforms `field.type_name` through `walk_transform_type_expr`.
5. In the same file, update `TraitDeclData::recurse_children`, `children`, and `walk_children` so:
   - trait field type expressions are visited
   - trait method input type expressions are visited
   - trait method output type expressions are visited
6. In `crates/lx-parser/src/parser/stmt_trait.rs`, replace the field-type parser from `type_name()` to the full `type_parser(arena.clone())`.
7. In `crates/lx-parser/src/parser/stmt_keyword.rs`, keep using `trait_body(...)`, but ensure the keyword parser constructs trait-entry field types as `TypeExprId` through the updated `FieldDecl` shape.
8. Do not add new trait-method grammar in this unit. The verified current parser does not construct `TraitMethodDecl`, so keep parser behavior unchanged there and update only the AST, checker, formatter, and walkers for existing `TraitMethodDecl` values.
9. Do not change class field syntax in `stmt_class.rs` unless the parser already exposes user-written type syntax there. The verified current `ClassField` shape is value-default-only.
10. In `crates/lx-checker/src/visit_stmt.rs`, replace every `named_to_type(f.type_name.as_str())` or direct raw-symbol handling of user-written trait types with `resolve_type_ann(f.type_name)`.
11. In the same file, resolve `method.output` through `resolve_type_ann(method.output)` for any in-memory `TraitMethodDecl` values already constructed by tests or future builders.
12. In `crates/lx-fmt/src/formatter/emit_stmt.rs`, replace direct `.as_str()` writes for trait field types, trait method inputs, and method outputs with `emit_type_expr(...)`.
13. In `crates/lx-fmt/src/formatter/emit_stmt_keyword.rs`, replace direct `.as_str()` writes for keyword trait field types with `emit_type_expr(...)`.
14. In `crates/lx-desugar/src/folder/desugar_schema.rs`, replace raw-string type handling with a small helper that pattern-matches on `TypeExpr`:
    - map `TypeExpr::Named(Int)` to JSON `integer`
    - map `TypeExpr::Named(Float)` to JSON `number`
    - map `TypeExpr::Named(Str)` to JSON `string`
    - map `TypeExpr::Named(Bool)` to JSON `boolean`
    - map `TypeExpr::List(_)` to JSON `array`
    - fall back to `object` for all other type expressions
15. In the same schema file, replace `default_for_type(type_name: &str, ...)` with `default_for_type(type_expr: TypeExprId, arena: &AstArena, ...)` and synthesize defaults from the `TypeExpr` variant instead of a string name.
16. Update any now-dead raw-symbol type helper code that becomes unused after Steps 10-15. Remove it in the same unit.
17. In `crates/lx-value/src/value/mod.rs`, stop aliasing runtime field and method shapes to the AST generics. Define concrete runtime structs instead:
   - `pub struct FieldDef { pub name: Sym, pub type_name: Sym, pub type_display: Arc<str>, pub default: Option<LxVal>, pub constraint: Option<ConstraintExpr> }`
   - `pub struct TraitMethodDef { pub name: Sym, pub input: Vec<FieldDef>, pub output: Sym, pub output_display: Arc<str> }`
   Keep the existing `LxTrait` and `LxClass` APIs otherwise unchanged.
18. In `crates/lx-ast/src/ast/display.rs`, add `pub fn render_type_expr(type_expr: TypeExprId, arena: &AstArena) -> Arc<str>`.
   - The helper owns all rendering for non-`Named` `TypeExpr` values.
   - It must produce the exact display string for every `TypeExpr` variant using AST-owned formatting, not formatter-internal code.
   - In `crates/lx-eval/src/interpreter/type_apply.rs`, add one exact conversion helper for AST syntax to runtime metadata:
   - `fn runtime_type_spec(type_expr: TypeExprId, arena: &AstArena) -> (Sym, Arc<str>)`
   - for `TypeExpr::Named(name)`, return `(name, Arc::from(name.as_str()))`
   - for every other `TypeExpr` variant, return `(intern(ANY_TYPE_NAME), render_type_expr(type_expr, arena))`
   The rendered string must be used only for diagnostics and trait reflection. The runtime matcher stays conservative by using `ANY_TYPE_NAME` for non-`Named` type syntax.
19. Implement the runtime migration at the exact construction sites:
   - `Interpreter::eval_field_decl` in `type_apply.rs` must call `runtime_type_spec(f.type_name, self.arena.as_ref())`
   - the trait-method path in `exec_stmt.rs` must convert each input field and `m.output` through `runtime_type_spec(...)`
   - `trait_apply.rs` must compare `field.type_name` to `ANY_TYPE_NAME` and keep using `field.type_display` in user-facing error text
   - `trait_ops.rs::method_to_record` must expose `type_display` and `output_display` instead of raw `Sym`
20. Do not store bare `TypeExprId` in `lx-value` runtime structs. No runtime value in this unit may require an external `AstArena` to remain valid after construction.
21. Extend Unit 01 tests so they cover non-trivial type syntax in trait fields and in manually constructed `TraitMethodDecl` formatter/checker cases, not only named types.
22. In `crates/lx-parser/tests/surface_parse_regressions.rs`, add the parser-side regression for the non-trivial trait-field type syntax that Unit 01 introduced. Keep it in the same parser regression file so the parser fixture and the later formatter/checker assertions share the same source shape and parse helper.
23. Do not introduce type aliases back to raw `Sym` for user-written type syntax in the AST in this unit.

## Verification
1. Run `cargo test -p lx-parser --test surface_parse_regressions`.
2. Run `cargo test -p lx-fmt --test format_regressions`.
3. Run `cargo test -p lx-checker --test checker_regressions`.
4. Run `cargo test -p lx-desugar --test surface_to_core_regressions`.
5. Run `cargo test -p lx-eval`.
6. Run `cargo test -p lx-value`.
7. Run `just test`.

## Out Of Scope
- New type-system features
- New semantic indexes
- Any CST or lossless syntax work
