# Gotchas

Permanent language behaviors that are non-obvious and trip up implementation or testing.

## Language

- **List indexing uses `.N` syntax, not `at N`.** `mylist.0`, `mylist.1`, same as tuple indexing. No `at` or `nth` function exists.
- **No `func?` predicate.** Type predicates exist for `ok?`, `err?`, `some?`, `none?`, `int?`, `float?`, `str?`, `list?`, `record?`, `map?`, `tuple?`, `bool?`. No function type predicate. To verify something is callable, call it.
- **Trait conformance errors are hard runtime errors.** Validated at Agent definition time — if an Agent declares a Trait but is missing a required method, execution halts. Propagates via `LxError`, not `Value::Err`. Not catchable with `??`.
- **Trait methods use MCP tool syntax.** `method: {input fields} -> output_type`. Reserved fields in Trait body: `description`, `requires`, `tags` — everything else is parsed as a method declaration.
- **Protocol failures return `Err` values.** Protocol construction returns `Err "reason"` for missing fields, type mismatches, and constraint violations. Use `^` to propagate or `??` for fallback.
- **Multi-arg function calls before `?` need parentheses.** `(re.is_match pattern input) ? { true -> ... }` — without parens, `?` may bind to the last argument instead of the full call result.

## Testing

- **`std/profile` tests create files on disk.** Writes to `.lx/profiles/`. Tests must clean up with `$rm -f` at end.
- **`NoopUserBackend` is the `RuntimeCtx` default.** Confirm returns `true`, choose returns first option, ask returns default/empty, progress/status/table are no-ops, check returns `None`. Tests and batch mode never block on stdin.

## Parser

- **`Agent` body fields must be `Ident` tokens.** The parser expects lowercase identifiers for method names and reserved fields. TypeName tokens (uppercase) in an Agent body will error.

## Temporary Workarounds

- **`uses` bindings are stored but not auto-connected.** `Value::Agent` now holds `uses` and `on` fields, and `on` is accessible via `Agent.on`. However, `uses` bindings are not yet auto-connected to MCP servers — they store metadata only. **Workaround:** Connect MCP manually in method bodies or `init`.
