# Parser Design

Recursive descent with Pratt precedence climbing. Produces an AST from the token stream.

Implements: [syntax.md](../spec/syntax.md), [grammar.md](../spec/grammar.md), [pattern-matching.md](../spec/pattern-matching.md)

## Architecture

The parser consumes `Vec<Token>` and produces `Vec<Stmt>` (a program is a list of statements). It uses a cursor into the token vector with peek/advance primitives.

Two entry points:
- `parse_program()` — top-level, parses until EOF
- `parse_expr(min_bp)` — Pratt core, parses an expression with minimum binding power

## Pratt Precedence

Each operator has a left and right binding power. Higher binding power means tighter binding. The 17 precedence levels from [grammar.md](../spec/grammar.md) map to binding powers:

```
.           → (33, 34)     field/index
(juxtapose) → (31, 32)     application
- !         → (_, 29)      unary prefix
* / % //    → (27, 28)     multiplicative
+ -         → (25, 26)     additive
.. ..=      → (23, 24)     range
++ <>       → (21, 22)     concat/compose
|           → (19, 20)     pipe
== != < > <= >= → (17, 18) comparison
&&          → (15, 16)     logical and
||          → (13, 14)     logical or
??          → (11, 12)     coalesce
^           → (10, _)      postfix error propagation
&           → (7, 8)       pattern guard (inside ? arms)
->          → (6, 5)       arm body / type arrow (right-assoc)
?           → (3, 4)       match/ternary
= := <-     → (1, 2)       binding (non-assoc)
```

`^` is postfix: it has a left binding power but no right binding power. `->` is right-associative: right bp < left bp.

## Expression Parsing

`parse_expr(min_bp)` follows the standard Pratt loop:

1. Parse a **prefix** (literal, ident, unary op, `(`, `[`, `{`, `%{`, `#{`, `$`, `par`, `sel`, `loop`, `break`, `assert`, `use`)
2. Loop: peek at the next token. If it has a left binding power ≥ min_bp, consume it and parse the right side

Prefix parsing dispatches on token kind:
- `Int/Float/Str*/Bool/Unit` → literal AST node
- `Ident` → check for `=`/`:=` after it (binding), else identifier expression
- `TypeName` → type constructor or type definition
- `(` → section, tuple, function def, or grouping (see disambiguation below)
- `[` → list literal
- `{` → record literal or block
- `%{` → map literal
- `#{` → set literal
- `$` variants → shell expression
- `!` → unary not
- `-` → unary negate
- `par` → par block
- `sel` → sel block
- `loop` → loop block
- `break` → break expression
- `assert` → assert expression

**Function body extent in pipe chains** — When `(params) body` appears as an argument to a HOF in a pipe chain, the body is parsed at binding power 0, consuming everything to the right including pipe operators. This means `map (x) x * 2 | sum` gives `map` a function whose body is `x * 2 | sum`. For inline functions with multi-expression bodies in pipe chains, use block delimiters: `map (x) { x * 2 } | sum`. Sections (`(* 2)`, `(> 0)`, `(.field)`) remain the primary mechanism for simple inline functions — they have no body extent ambiguity.

## Disambiguation Challenges

### `(` — Four Meanings

`(` starts one of:
1. **Section**: `(op expr)` or `(expr op)` or `(.field)` — operator with one operand missing
2. **Function definition**: `(params) body` — parameter list followed by body
3. **Tuple**: `(expr expr+)` — two or more sub-expressions
4. **Grouping**: `(expr)` — single expression in parens

Strategy: parse the contents, then decide based on what was found.
- If first token is an operator and there are exactly 2 tokens inside → section (right section)
- If last token is an operator and there are exactly 2 tokens inside → section (left section)
- If first token is `.` followed by `Ident` and nothing else → field section
- If contents look like `ident ident ... -> ...` or `ident:Type ident:Type ...` → function params
- If there's exactly one expression → grouping
- If there are multiple expressions → tuple

The key heuristic for function vs tuple: if the contents start with identifiers that are NOT followed by operators (they look like parameter names, possibly with `:Type` annotations), treat as function params. If any sub-expression is a complex expression (has operators), treat as tuple.

### `?` — Three Modes

After parsing the left-hand expression, `?` dispatches:
- `? {` → multi-arm match (always, per spec)
- `? expr : expr` → ternary
- `? expr` (no `:` follows) → single-arm conditional

### `{` After `?` — Always Match Block

Per spec: `expr ? {` always starts a multi-arm match. For record literals in ternary position, the user must wrap in parens: `cond ? ({x: 1}) : ({x: 0})`.

### `$` — Shell Mode

After `$`, `$$`, `$^`, or `${`, the parser delegates to the lexer's shell tokens. Shell text tokens and interpolation hole tokens alternate. The parser produces a `ShellExpr` AST node containing literal text segments and interpolated expression nodes.

## Function Application

Function application by juxtaposition has high binding power (31, 32). `f x y` parses as `(f x) y`. The parser detects application when the current token could start an expression AND the previous expression is a callable form.

Callable forms (left side can be applied): `Ident`, `TypeName`, `Apply`, `FieldAccess`, `Section`, `Func`. Non-callable forms (application NOT attempted): `Literal`, `Binary`, `Unary`, `List`, `Record`, `Map`, `Set`, `Tuple`, `Match`, `Shell`, `Par`, `Sel`, `Loop`, `Break`, `Assert`, `Propagate`, `Coalesce`. This ensures `[1 2 3]` parses as three list elements, not `Apply(Apply(1, 2), 3)`.

Tokens that start an argument: `Int`, `Float`, `Str*`, `Ident`, `TypeName`, `(`, `[`, `{`, `true`, `false`, `r/`.

Application stops at tokens that can't start an argument: operators, `)`, `]`, `}`, `;`, newline, `?`, `^`, `|`, `->`.

After positional arguments, the parser checks for named arguments: `IDENT ":"` followed by an expression. Named args are collected in a separate list on the `Apply` node. Named args can only follow positional args — `f name: val` is valid, `f name: val "pos"` is not.

**`dbg` special case** — When the parser sees `Apply(Ident("dbg"), inner)`, it emits `Expr::Dbg(inner)` instead of a normal `Apply` node. This captures the source text of `inner` at parse time for display in the debug output. Other built-in names (`tap`, `map`, etc.) use normal `Apply` nodes.

## Type Definition Parsing

When the parser sees `TYPE IDENT* "="` at statement level, it enters type definition mode:
- If followed by `{` → record type definition (`TypeDef::Record`)
- If followed by `|` → tagged union definition (`TypeDef::Union`)

The result is `Stmt::TypeDef(TypeDefStmt { name, params, def })`. The `params` are the lowercase identifiers between the type name and `=` (generic parameters). `+` before the type name sets `exported: true`.

## Pattern Parsing

Patterns appear in `?` arms, `=` bindings (destructuring), and function parameters. `parse_pattern()` dispatches:

- `Int/Float/Str/true/false` → literal pattern
- `Ident` → binding pattern (captures the matched value into this name)
- `_` → wildcard
- `TypeName` → tagged union variant, followed by zero or more sub-patterns
- `(` → tuple pattern: `(pat pat ...)`
- `[` → list pattern: `[pat ...]` with optional `..rest` spread
- `{` → record pattern: `{field: pat ...}` with optional `..rest`

Guards: after a pattern, `& (expr)` adds a boolean guard condition.

## Error Recovery

On parse error, the parser:
1. Emits a diagnostic with the current span
2. Advances to the next synchronization point: `;`, newline, `}`, `]`, `)`, or EOF
3. Continues parsing from there

Up to 5 errors are collected before aborting. This matches the spec in [diagnostics.md](../spec/diagnostics.md).

## Statement Parsing

```
parse_stmt():
  if peek == Export(+): consume, parse binding, mark as exported
  if peek == Use: parse_use_stmt()
  else: parse_expr(0) — which handles bindings internally
```

`use` is parsed specially: `use path (: alias)? ({ names })?`. The path is a sequence of `Ident/TypeName` tokens separated by `/`. Relative paths start with `./` or `../`.

## Cross-References

- Token types: [impl-lexer.md](impl-lexer.md)
- AST nodes produced: [impl-ast.md](impl-ast.md)
- Operator precedence source: [grammar.md](../spec/grammar.md)
- Pattern matching spec: [pattern-matching.md](../spec/pattern-matching.md)
- Shell expression spec: [shell.md](../spec/shell.md)
