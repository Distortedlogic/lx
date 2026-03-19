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

## lx Package Traps

- **Export names shadow builtins inside the module.** `+filter = (criteria s) { ... all | filter pred ... }` — the internal `filter` call recursively calls the export, not the builtin HOF. **Fix:** capture the builtin before the export: `keep = filter` at top of file, then use `keep` internally. Or rename the export to avoid collision (`trace.query` instead of `trace.filter`).
- **Adjacent string interpolation blocks fail.** `"{head}{tail}"` — the first `{head}` evaluates to a Str, then `{tail}` tries to call the Str as a function. **Fix:** use `++` concatenation: `head ++ tail`. Or use a single interpolation with the full expression.
- **Multi-line ternary chains don't parse.** `cond1 ? val1\n: cond2 ? val2\n: default` — the `:` on a new line is parsed as something else. **Fix:** keep the entire ternary chain on one line, or extract conditions into named bindings and nest: `cond1 ? val1 : (cond2 ? val2 : default)`.
- **`{}` is Unit, not empty Record.** `f x {}` passes Unit as the second arg, not an empty record. This breaks functions expecting a Record (e.g., `tasks.list store {}`). **Fix:** use `()` explicitly for Unit, or handle Unit in the function: `filter_rec == () ? defaults : filter_rec.field`.

## Incomplete Wiring

- **`uses` bindings are metadata-only.** `Value::Agent` holds `uses` and `on` fields, but `uses` bindings are not auto-connected to MCP servers at runtime. **Workaround:** Connect MCP manually in method bodies or `init`.
