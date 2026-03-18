-- Memory: interrupt vector table. Known faults in the lx implementation.
-- Delete entries when fixed. Add entries when discovered. Fix bugs before new features.

# Known Bugs
Each entry has: severity, root cause, affected files, workaround.

When you fix a bug, delete its entry. When you discover a bug, add it here.

## Parser

### Single-line multi-field records can't have complex field values
- **Root cause:** `record_field_depth` + `Ident Colon` lookahead in `parser/prefix_coll.rs`
- **Symptom:** `{x: f a  y: z}` misparsed — parser terminates field value at `y:`
- **Multiline works.** Only single-line multi-field records are broken.
- **Workaround:** Put fields on separate lines, or extract to temp bindings

### List spread doesn't consume function application
- **Root cause:** `parse_expr(32)` in list spread (`prefix_coll.rs:13`) — bp too high for application
- **Symptom:** `[..f x y]` spreads `f` (a Func), not `f x y` (the call result)
- **Workaround:** `[..(f x y)]` — wrap in parens
- **Note:** Record spread was fixed (Session 52) using bp=31 + collection_depth=0

### Module path resolver only handles single `..` parent
- **Root cause:** `resolve_module_path` checks `path[0] == ".."` for one level only
- **Symptom:** `use ../../examples/foo` fails
- **Workaround:** Organize files so imports need at most one `..`

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

## Code Quality

### 8 files exceed 300-line limit
- agent_reconcile_strat.rs (326)
- cron.rs (320)
- str.rs (314)
- stmt_agent.rs (313)
- mcp.rs (304)
- diag_walk.rs (304)
- visitor/walk/mod.rs (303)
- tasks.rs (302)
