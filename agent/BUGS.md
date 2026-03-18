-- Memory: interrupt vector table. Known faults in the lx implementation.
-- Delete entries when fixed. Add entries when discovered. Fix bugs before new features.

# Known Bugs
Each entry has: severity, root cause, affected files, workaround.

When you fix a bug, delete its entry. When you discover a bug, add it here.

## Parser

### Named-arg parser consumes ternary `:` separator
- **Symptom:** `f x key: val ? ...` — `:` parsed as named arg, not ternary else
- **Workaround:** Parenthesize: `(f x key: val) ? ...`

### Assert parsing greedy with callable expressions
- **Symptom:** `assert (expr) "msg"` — when `(expr)` looks callable, `"msg"` is consumed as argument
- **Workaround:** `assert (expr) ; "msg"` or use `assert expr "msg"` without parens around expr

## Runtime

### Tuple destructuring in HOF chains breaks via test.run
- **Symptom:** `| filter (a b) expr | map (a b) expr` fails with "undefined variable" when invoked through `test.run → invoke_flow → call_value`
- **Works via:** `lx run` directly
- **Root cause:** `call_value` tuple-splatting doesn't propagate through deep HOF chains
- **Workaround:** Avoid chained tuple-destructuring lambdas in test flows

### par/sel/pmap are sequential
- **Root cause:** No async runtime. All three execute arms sequentially.
- **Impact:** Programs work but don't get parallelism. `sel` doesn't actually race.
- **Fix requires:** tokio integration — architectural change, not a quick fix

