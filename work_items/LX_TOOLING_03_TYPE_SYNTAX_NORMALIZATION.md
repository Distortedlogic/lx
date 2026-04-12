---
unit: 3
title: Type Syntax Normalization
scope: lx-ast, lx-parser, lx-checker, lx-fmt, lx-desugar
depends_on: LX_TOOLING_02_PHASE_HARDENING
optional: false
---

## Goal
Normalize all user-written type positions onto `TypeExprId` so parsing, checking, formatting, and desugaring all consume the same type syntax model.

## Why this boundary is isolated
This unit is the AST shape change. It should land before traversal or semantic refactors so later agents can rely on one uniform representation for every type-bearing site.

## Primary crates/files touched
- `crates/lx-ast/src/ast/types.rs`
- `crates/lx-ast/src/ast/walk_impls.rs`
- `crates/lx-parser/src/parser/stmt_trait.rs`
- `crates/lx-parser/src/parser/stmt_keyword.rs`
- `crates/lx-parser/src/parser/stmt_class.rs`
- `crates/lx-parser/src/parser/type_ann.rs`
- `crates/lx-checker/src/visit_stmt.rs`
- `crates/lx-fmt/src/formatter/emit_stmt.rs`
- `crates/lx-fmt/src/formatter/emit_stmt_keyword.rs`
- `crates/lx-desugar/src/folder/desugar_schema.rs`

## Mechanical task list
1. Change `FieldDecl.type_name` and `MethodSpec.output` to `TypeExprId` in `crates/lx-ast/src/ast/types.rs`.
2. Update parser construction sites so trait field types and method outputs are parsed through the type annotation parser instead of being stored as raw `Sym`.
3. Update `walk_impls.rs` so recursive AST transforms visit and rebuild the new type-expression fields correctly.
4. Update the checker to resolve these fields through the existing type-expression resolver instead of `named_to_type`.
5. Update formatter emission so trait field types and method outputs print through the type-expression formatter path.
6. Update `desugar_schema.rs` so schema generation reads type syntax from `TypeExprId` rather than from raw symbol strings.
7. Remove any now-dead type-name-only code paths that become unreachable after the normalization.

