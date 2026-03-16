# Gotchas

Permanent language behaviors that are non-obvious and trip up implementation or testing.

## Language

- **List indexing uses `.N` syntax, not `at N`.** `mylist.0`, `mylist.1`, same as tuple indexing. No `at` or `nth` function exists.
- **No `func?` predicate.** Type predicates exist for `ok?`, `err?`, `some?`, `none?`, `int?`, `float?`, `str?`, `list?`, `record?`, `map?`, `tuple?`, `bool?`. No function type predicate. To verify something is callable, call it.
- **Trait conformance errors are hard runtime errors.** They propagate via `LxError`, not `Value::Err`. Not catchable with `??`. A malformed Agent definition halts execution.
- **Protocol failures are also hard runtime errors.** Same as Trait conformance — not catchable with `??`.

## Testing

- **`std/profile` tests create files on disk.** Writes to `.lx/profiles/`. Tests must clean up with `$rm -f` at end.
- **`NoopUserBackend` is the `RuntimeCtx` default.** Confirm returns `true`, choose returns first option, ask returns default/empty, progress/status/table are no-ops, check returns `None`. Tests and batch mode never block on stdin.

## Parser

- **`Agent` body fields must be `Ident` tokens.** The parser expects lowercase identifiers for method names and reserved fields. TypeName tokens (uppercase) in an Agent body will error.
