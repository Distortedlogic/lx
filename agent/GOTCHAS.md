-- Memory: errata sheet. Non-obvious behaviors that trip up implementation or testing.
-- Only genuine traps belong here — things that look correct but silently misbehave.
-- Design decisions and syntax rules belong in LANGUAGE.md/AGENTS.md, not here.

# Gotchas

## Parser Traps

- **Multi-arg calls before `?` silently misbind.** `re.is_match pattern input ? { ... }` — `?` binds to `input`, not the full call. **Fix:** `(re.is_match pattern input) ? { ... }`.
- **`()` before `(param) { body }` breaks argument parsing.** `f name () (x) { body }` — parser doesn't see 4 args. The `()` followed by `(x)` confuses it. **Fix:** bind either to a variable first: `input = ()` then `f name input (x) { body }`.
- **`Agent` body rejects uppercase tokens.** Method names and reserved fields must be lowercase `Ident` tokens. TypeName tokens (uppercase) in an Agent body error.

## Uncatchable Errors

- **Trait conformance halts execution.** If an Agent declares a Trait but is missing a required method, it's a hard `LxError` — not `Value::Err`. `??` cannot catch it. This is by design but surprising if you expect defensive coding to work.

## Incomplete Wiring

- **`uses` bindings are metadata-only.** `Value::Agent` holds `uses` and `on` fields, but `uses` bindings are not auto-connected to MCP servers at runtime. **Workaround:** Connect MCP manually in method bodies or `init`.
