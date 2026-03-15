# lx Development Log

Session history + design decisions. For priorities and gap analysis, see `NEXT_PROMPT.md`. For self-critique, see `CURRENT_OPINION.md`.

## Implementation Status

**23/23 PASS.** 12 stdlib modules. 14 flow programs. `just diagnose` clean.

## Key Design Decisions

Non-obvious choices that cause confusion if forgotten:

- **Pipe HIGHER than comparison**. `data | sort | len > 5` = `((data | sort) | len) > 5`.
- **`^` and `??` LOWER than `|`**. `url | fetch ^` = `(url | fetch) ^`.
- **Function body extent** — `map (x) x * 2 | sum` gives map body `x * 2 | sum`. Use blocks or sections.
- **Division by zero is a panic**. Use `math.safe_div` for recoverable.
- **Tuple auto-spread**: N-param function + single N-tuple → auto-destructure.
- **`none?` is 2-arg only** (collection predicate). Use `!some?` for Maybe.
- **`$echo "hello {name}"`** — `"` are shell quotes, `{name}` is lx interpolation.
- **`+` at column 0** is export. Anywhere else is addition.
- **Record equality is order-independent**.
- **`log` is a record** with `.info`, `.warn`, `.err`, `.debug` fields.
- **Application requires callable left-side**. `[1 2 3]` is three elements, not application.
- **Collection-mode**: inside `[]`, only TypeConstructor triggers application.
- **`??` unwraps Ok/Some**. Non-Result/Maybe values pass through.
- **Default params reduce effective arity**. 1 required arg + defaults → executes immediately.
- **Tuple variables need commas**. `(b, a)` tuple. `(b a)` is application.
- **`{x: (f 42)}`** — parens for function calls in record field values.
- **is_func_def ambiguity**: `(a b c) (expr)` with all bare Ident params NOT a func def. Defaults/underscores override.
- **`~>`/`~>?` at concat precedence**. `agent ~>? msg ^ | process` = `((agent ~>? msg) ^) | process`.
- **`yield` is callback-based**. No handler → runtime error.
- **`with` is scoped binding**. Lexical scope, not dynamic. Supports `:=` mutable.
- **Record field update via `<-`**. Requires `:=` binding. Adding new fields allowed.
- **`MCP` declarations are typed tool contracts**. Callable → record of wrapper functions.

## Technical Debt

- `par`/`sel`/`pmap` are sequential; real async needs `tokio`
- Named-arg parser consumes ternary `:` separator (workaround: parens)
- Assert parsing greedy — `assert (expr) "msg"` consumes msg when `(expr)` is callable
- Currying removal deferred — requires parser architecture change

## Known Spec Tensions

- `it` in `sel` blocks — only implicit binding
- Shell line is single-line only — forces `${ }` for complex commands
- Function body extent — inline lambdas consume everything
- Named args + default params + currying interaction

## Session History

| # | Date | Focus | Result |
|---|------|-------|--------|
| 1–5 | 03-13 | Spec authoring, contradiction fixes, test files | Foundation |
| 6–7 | 03-13 | First Rust impl — lexer, parser, interpreter, type defs | 4/13 PASS |
| 8–14 | 03-14 | Parser fixes, iterators, shell, concurrency, modules, agents, Protocol | 15/15 PASS |
| 15–18 | 03-14 | Stdlib (json/ctx/math/fs/env/re/md/agent/mcp), MCP HTTP transport | 17/17 PASS |
| 19 | 03-14 | Removed 7 features (regex, $$, <>, sets, iterators, types, tuple semis) | 17/17 PASS |
| 20–24 | 03-14 | yield, MCP decls, std/http, std/time, file splits, with/field update | 22/22 PASS |
| 25 | 03-14 | Stale spec cleanup — all 22 spec files updated | 22/22 PASS |
| 26 | 03-14 | std/cron, real-flow gap analysis vs mcp-toolbelt arch_diagrams | 23/23 PASS |
| 27 | 03-15 | Repo reorg (asl/ → spec/design/tests/flows), 14 flow programs + specs | 23/23 PASS |
| 28 | 03-15 | Design review: types + regex back, full stdlib roadmap (5 modules, 6 agents, 2 MCPs) | Decision |
