# Goal

Fix the `??` coalesce operator so it passes through plain values instead of dropping them. Currently `data.results ?? []` returns `[]` even when `data.results` is a valid List, because the desugared Match wildcard arm returns the default instead of the matched value.

# Why

Every program that uses `??` on record field access is broken. `json.categories ?? []` drops the categories list. `result.text ?? ""` drops the text. Programs work around this by removing `??` entirely, which loses null-safety.

# What Changes

**`crates/lx/src/folder/desugar.rs` — `desugar_coalesce` function:**

Current desugar produces:
```
match expr {
  Some v -> v
  Ok v -> v
  None -> default
  _ -> default       ← BUG: drops plain values
}
```

Fix — change the wildcard arm to pass through the matched value:
```
match expr {
  Some v -> v
  Ok v -> v
  None -> default
  Err _ -> default
  other -> other      ← plain values pass through
}
```

The change: replace the final wildcard `_ -> default` with two arms: `Err _ -> default` (errors return default) and `other -> other` (everything else passes through). The `other` pattern is a `Pattern::Bind` that captures the value and returns it.

Read `desugar_coalesce` in `desugar.rs` to find the exact Match arm construction. The function builds `MatchArm` structs with patterns and bodies. Add a new `MatchArm` for `Err` with a wildcard sub-pattern (or just `Err _`), then change the final wildcard arm's body from the default expression to a reference to the matched value.

The Err pattern: `Pattern::Constructor(PatternConstructor { name: intern("Err"), args: vec![wildcard_id] })` with body `default_expr`.

The final arm: `Pattern::Bind(intern("__coalesce_val"))` with body `Expr::Ident(intern("__coalesce_val"))`. Use a gensym'd name to avoid collisions.

# Files Affected

- `crates/lx/src/folder/desugar.rs` — modify `desugar_coalesce`
- `tests/coalesce_plain_values.lx` — new test

# Task List

### Task 1: Fix desugar_coalesce wildcard arm

**Subject:** Change wildcard from returning default to returning matched value

**Description:** Read `crates/lx/src/folder/desugar.rs`. Find the `desugar_coalesce` function. It builds a `Match` expression with four arms: Some, Ok, None, wildcard.

Change the wildcard arm:

Before: wildcard pattern `_` → body is `default` expression
After: two new arms replacing the wildcard:
1. `Err _` pattern → body is `default` expression (errors return the default, same as None)
2. Bind pattern `__coalesce_v` → body is `Expr::Ident(intern("__coalesce_v"))` (plain values pass through)

Use the existing `gensym` function (used by other desugar functions) to generate a unique variable name instead of `__coalesce_v`.

The Err pattern construction: allocate `Pattern::Constructor(PatternConstructor { name: intern("Err"), args: vec![wildcard_pattern_id] })` where wildcard is `Pattern::Wildcard`. The body is the default expression.

The bind pattern construction: allocate `Pattern::Bind(gensym("coalesce"))`. The body is `Expr::Ident(same_gensym)`.

Write test `tests/coalesce_plain_values.lx`:
```lx
list = [1; 2; 3]
result = list ?? []
assert (result | len == 3)

text = "hello"
result2 = text ?? "default"
assert (result2 == "hello")

num = 42
result3 = num ?? 0
assert (result3 == 42)

none_val = None ?? "fallback"
assert (none_val == "fallback")

ok_val = Ok 10 ?? 0
assert (ok_val == 10)

err_val = Err "bad" ?? 99
assert (err_val == 99)

some_val = Some "yes" ?? "no"
assert (some_val == "yes")
```

**ActiveForm:** Fixing coalesce wildcard arm

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

1. **Call `complete_task` after each task.**
2. **Call `next_task` to get the next task.**
3. **Do not add, skip, reorder, or combine tasks.**
4. **Tasks are implementation-only.**

---

## Task Loading Instructions

```
mcp__workflow__load_work_item({ path: "work_items/LX_COALESCE_FIX.md" })
```
