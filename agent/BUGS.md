-- Memory: interrupt vector table. Known faults in the lx implementation.
-- Delete entries when fixed. Add entries when discovered. Fix bugs before new features.

# Known Bugs
Each entry has: severity, root cause, affected files, workaround.

When you fix a bug, delete its entry. When you discover a bug, add it here.

## Parser

### `$^` non-zero exit throws uncatchable LxError
- **Symptom:** `($^rg "pattern" file) ?? ""` does NOT catch when rg exits 1 (no matches). The `??` is bypassed and the error propagates up.
- **Root cause:** `$^` (exec_capture) calls `LxError::propagate` on non-zero exit, which is an `LxError`, not a `Value::Err`. `??` only catches `Value::Err` and `Value::None`.
- **Workaround:** Use `$sh -c "{cmd}"` which returns `Ok({out, err, code})` regardless of exit code. Then check `.out` or `.code` manually.
- **Suggested fix:** `$^` should return `Value::Err` on non-zero exit, not `LxError::propagate`, so `??` can catch it.

### Named-arg parser consumes ternary `:` separator
- **Symptom:** `f x key: val ? ...` — `:` parsed as named arg, not ternary else
- **Workaround:** Parenthesize: `(f x key: val) ? ...`

### Assert parsing greedy with callable expressions
- **Symptom:** `assert (expr) "msg"` — when `(expr)` looks callable, `"msg"` is consumed as argument
- **Workaround:** `assert (expr) ; "msg"` or use `assert expr "msg"` without parens around expr

## Runtime

### Closures inside + functions silently drop non-exported module bindings
- **Symptom:** `helper = (x) x * 2` then `+f = () { list | each (x) { helper x } }` — `each` body silently doesn't execute. No error, no output.
- **Expected:** `helper` should be in scope inside the closure since it's defined at module level before `+f`
- **Root cause:** `+` export exclusion from forward declarations affects closure capture inside `+` function bodies. The closure captures `helper` as `None` and `each` swallows the error.
- **Workaround:** Inline the helper body, pass as parameter, or use two-step export: `helper = ...; +helper = helper`

### Agent Trait defaults not injected into Class : [Agent]
- **Symptom:** `Class Foo : [Agent] = { x: 0 }` — `Foo ().think` is `None`, not the default from `pkg/agent.lx`
- **Expected:** Agent Trait defaults (think, think_with, handle, run, etc.) should be injected like any other Trait default
- **Workaround:** Call `ai.prompt_with` directly instead of `self.think_with`
- **Root cause:** Likely in `interpreter/traits.rs` `inject_traits` — the Agent auto-import may not resolve when using `Class : [Agent]` syntax vs `Agent` keyword

### Tuple destructuring in HOF chains breaks via test.run
- **Symptom:** `| filter (a b) expr | map (a b) expr` fails with "undefined variable" when invoked through `test.run → invoke_flow → call_value`
- **Works via:** `lx run` directly
- **Root cause:** `call_value` tuple-splatting doesn't propagate through deep HOF chains
- **Workaround:** Avoid chained tuple-destructuring lambdas in test flows


