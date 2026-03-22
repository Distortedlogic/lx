# Chumsky Parser Rewrite

Replace the hand-rolled Pratt parser (2154 lines, 14 files) with chumsky parser combinators.

## Why

The current parser is imperative, verbose, and manually tracks spans via `SourceSpan::new(SourceOffset::from(start), end - start)` ~90 times. chumsky handles spans automatically, provides declarative combinators, and has built-in Pratt parsing support via `pratt()`.

## Current parser files

- `parser/mod.rs` — Pratt climbing loop, parse_expr, Parser struct
- `parser/prefix.rs` — literals, unary ops, keywords (loop/par/sel/break/assert/emit/yield)
- `parser/infix.rs` — binary ops, field access, match, ternary, pipe, coalesce
- `parser/paren.rs` — tuples, sections, function defs
- `parser/statements.rs` — bindings, type defs, field updates
- `parser/pattern.rs` — destructuring patterns
- `parser/prefix_coll.rs` — lists, records, maps, blocks
- `parser/prefix_with.rs` — with/with-resource/with-context
- `parser/stmt_trait.rs` — trait declarations
- `parser/stmt_class.rs` — class declarations
- `parser/stmt_use.rs` — use/import statements
- `parser/type_ann.rs` — type annotations
- `parser/func.rs` — function definition detection heuristics
- `parser/helpers.rs` — binding power tables, lookahead helpers

## Edge cases to preserve

1. **Juxtaposition application** — `f x y` means `f(x)(y)`. Context-dependent: `is_application_candidate` checks callable expressions and depth state.
2. **Function vs tuple disambiguation** — `(x y)` could be tuple or function def. `is_func_def` does 90-line lookahead with heuristics (underscore params, type annotations, arrow return type).
3. **Context depth tracking** — `collection_depth`, `record_field_depth`, `application_depth` affect parsing behavior.
4. **Sections** — `(+)`, `(+3)`, `(.field)`, `(.0)` — complex paren content disambiguation.
5. **String interpolation** — lexer emits StrStart/StrChunk/StrEnd tokens, parser assembles.
6. **Line-continuation** — semi tokens before operators are consumed to allow multi-line expressions.
7. **Export prefix** — `+` at line start means export, not addition.

## Approach

- Add `chumsky = "0.10"` (or latest) dependency
- Parse from existing logos token stream (Token/TokenKind)
- Use `select!` macro for token matching
- Use `pratt()` for operator precedence (replaces manual binding power tables)
- Custom combinators for complex disambiguation
- Keep AST types unchanged — only the parser changes
- Run `just test` to verify same 27 tests pass

## Target

~800-1000 lines down from 2154.
