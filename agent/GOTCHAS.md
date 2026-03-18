-- Memory: errata sheet. Non-obvious behaviors that trip up implementation or testing.
-- Only genuine traps belong here — things that look correct but silently misbehave.
-- Design decisions and syntax rules belong in LANGUAGE.md/AGENTS.md, not here.

# Gotchas

## Parser Traps

- **Multi-arg calls before `?` silently misbind.** `re.is_match pattern input ? { ... }` — `?` binds to `input`, not the full call. **Fix:** `(re.is_match pattern input) ? { ... }`.
- **`()` before `(param) { body }` breaks argument parsing.** `f name () (x) { body }` — parser doesn't see 4 args. The `()` followed by `(x)` confuses it. **Fix:** bind either to a variable first: `input = ()` then `f name input (x) { body }`.
- **`f arg () { body }` passes Unit then a separate block, not a closure.** `deadline.scope dl () { body }` — the `()` is consumed as Unit (second arg), and `{ body }` becomes a separate block expression. **Fix:** bind the closure first: `body_fn = () { ... }` then `f arg body_fn`.

## Keyword Field Names

- **`par` is a keyword — can't use as record field via dot access.** `module.par` fails to parse because `par` is consumed as the `par { }` keyword. `std/flow` uses `flow.parallel` instead. Same applies to other keywords: `sel`, `match`, `if`, `use`, `emit`, `yield`, `refine`, `receive`, `Agent`, `Protocol`, `Trait`, `MCP`.

## Uncatchable Errors

- **Trait conformance halts execution.** If an Agent declares a Trait but is missing a required method, it's a hard `LxError` — not `Value::Err`. `??` cannot catch it. This is by design but surprising if you expect defensive coding to work.

## Incomplete Wiring

- **`uses` bindings are metadata-only.** `Value::Agent` holds `uses` and `on` fields, but `uses` bindings are not auto-connected to MCP servers at runtime. **Workaround:** Connect MCP manually in method bodies or `init`.
