# Structured Agent-to-User Interaction

`std/user` provides typed interaction primitives for agents communicating with humans. Fills the gap between `emit` (fire-and-forget text output) and `yield` (opaque orchestrator protocol).

## Problem

Agent-to-user communication has two extremes:

1. `emit "Processing file 3 of 7..."` — fire-and-forget text. No structure. No response.
2. `yield {kind: "approval" ...}` — full orchestrator round-trip. Heavy, untyped, requires external orchestrator.

Neither covers common interaction patterns:
- Presenting choices and getting a selection
- Asking for confirmation before destructive actions
- Showing structured progress (step N of M, percentage)
- Requesting specific input types (file path, number, text)
- Streaming status updates that a UI can render as a progress bar

Agents are forced to use `emit` for everything (losing structure and interactivity) or `yield` for everything (adding orchestrator complexity to simple interactions).

## `std/user`

### Confirmation

```
use std/user

ok = user.confirm "Delete 15 files from src/old/?" ^
ok ? true -> delete_files old_files
```

Displays a yes/no prompt. Returns `Bool`. Backend renders appropriately (terminal: `[y/N]`, GUI: button pair).

### Choice

```
branch = user.choose "Which branch to deploy?" ["main" "staging" "develop"] ^
```

Presents numbered options. Returns the selected value (not the index). If `options` is a list of records, displays a formatted table.

```
action = user.choose "How to handle conflicts?" [
  {label: "Merge"  desc: "Attempt automatic merge"}
  {label: "Rebase" desc: "Rebase onto target"}
  {label: "Abort"  desc: "Cancel the operation"}
] ^
```

### Text Input

```
name = user.ask "Project name?" ^
name = user.ask_with "Project name?" {default: "my-project"  validate: (s) len s > 0} ^
```

Free-text input with optional default and validation. `validate` is called on input; if it returns `false`, the prompt repeats.

### Progress

```
user.progress 3 7 "Running tests"
user.progress_pct 0.42 "Indexing files"
```

Fire-and-forget (like `emit`). Backends render as progress bar, percentage, or status line. Successive calls update in-place where possible.

```
files | each_with_index (f i) {
  user.progress i (len files) "Processing {f.name}"
  process f ^
}
```

### Status

```
user.status :info "Connected to database"
user.status :warn "Rate limit approaching (80%)"
user.status :error "Failed to reach API endpoint"
user.status :success "All tests passed"
```

Structured status with severity level. Backends render with color/icon. Like `emit` but with semantic level.

### Table Display

```
user.table ["File" "Issues" "Status"] [
  ["auth.rs" "3" "needs review"]
  ["db.rs" "0" "clean"]
  ["api.rs" "1" "minor"]
]
```

Renders a formatted table. Terminal: aligned columns. GUI: native table widget.

## Backend Integration

All `std/user` functions delegate to a new `UserBackend` trait on `RuntimeCtx`:

```rust
pub trait UserBackend: Send + Sync {
    fn confirm(&self, message: &str) -> Result<bool, String>;
    fn choose(&self, message: &str, options: &[String]) -> Result<usize, String>;
    fn ask(&self, message: &str, default: Option<&str>) -> Result<String, String>;
    fn progress(&self, current: usize, total: usize, message: &str);
    fn progress_pct(&self, pct: f64, message: &str);
    fn status(&self, level: &str, message: &str);
    fn table(&self, headers: &[String], rows: &[Vec<String>]);
}
```

### Standard Backends

**`StdinStdoutUserBackend`** (default for CLI):
- `confirm` → prints `message [y/N]: `, reads line, returns `true` on `y`/`yes`
- `choose` → prints numbered list, reads number, returns index
- `ask` → prints prompt, reads line
- `progress` → prints `[3/7] Running tests` (overwrites previous line with `\r`)
- `status` → prints with prefix: `[INFO]`, `[WARN]`, `[ERROR]`, `[OK]`
- `table` → column-aligned text output

**`YieldUserBackend`** (for orchestrator mode):
- Interactive functions (`confirm`, `choose`, `ask`) delegate to `yield` with structured payload:
  ```json
  {"kind": "user.confirm", "message": "Delete 15 files?"}
  ```
  Orchestrator responds with the answer. This bridges `std/user` into the yield protocol automatically.
- Non-interactive functions (`progress`, `status`, `table`) emit structured JSON on stdout.

**`NoopUserBackend`** (for testing/batch):
- `confirm` → returns `true` (auto-approve)
- `choose` → returns `0` (first option)
- `ask` → returns default or empty string
- `progress`/`status`/`table` → no-op

### Signal Check (User-Initiated Interruption)

Absorbs the cooperative interrupt check from `spec/agents-interrupt.md`. `user.check` is a non-blocking check for pending user signals — eliminates the need for a `checkpoint` keyword.

```
signal = user.check ()
signal ? {
  Some {action: "redirect" task: new_task} -> {
    emit "redirecting to: {new_task}"
    current_task <- new_task
  }
  Some {action: "stop"} -> break current_result
  None -> ()
}
```

Returns `Some signal` if a user signal is pending, `None` otherwise. Never blocks.

Signals are delivered via `.lx/signals/{pid}.json` files. The `lx signal` CLI command writes to this file:

```bash
lx signal 12345 '{"action": "redirect", "task": "fix auth bug instead"}'
lx signal 12345 '{"action": "stop"}'
```

The runtime polls the signal file at natural check points (loop iterations, between pipeline stages). `user.check` provides explicit cooperative checking.

**Backend**: `UserBackend` gains a `check_signal(&self) -> Option<Value>` method. `StdinStdoutUserBackend` reads from `.lx/signals/{pid}.json`. `NoopUserBackend` always returns `None`.

For reactive (non-cooperative) signal handling, use `agent.on me :signal handler` via lifecycle hooks, or the `on: {signal: handler}` field in Agent declarations.

## Patterns

### Destructive operation guard

```
use std/user
use std/git

s = git.status () ^
s.untracked | len > 0 ? true -> {
  user.status :warn "{s.untracked | len} untracked files will be lost"
  ok = user.confirm "Proceed with hard reset?" ^
  ok ? false -> Err "aborted by user"
}
git.checkout "main" ^
```

### Interactive workflow selection

```
strategies = [
  {label: "Fast"   desc: "Skip tests, deploy immediately"}
  {label: "Safe"   desc: "Full test suite, staged rollout"}
  {label: "Custom" desc: "Configure each step manually"}
]
choice = user.choose "Deployment strategy?" strategies ^
```

### Progress-tracked batch operation

```
files = glob "src/**/*.rs"
files | each_with_index (f i) {
  user.progress i (len files) "Analyzing {f}"
  analyze f ^
}
user.status :success "Analysis complete: {len files} files"
```

### Compose with agent.gate

`agent.gate` remains the right tool for multi-party approval (human + agent reviewers, escalation). `user.confirm` is for simple single-human yes/no. They compose:

```
user.confirm "Submit for review?" ^
  ? true -> agent.gate review_request {approvers: [lead senior]} ^
```

## Implementation

New stdlib module + new `UserBackend` trait on `RuntimeCtx`.

Functions that return values (`confirm`, `choose`, `ask`) are blocking — they wait for user input. Non-blocking functions (`progress`, `status`, `table`) return `()` immediately.

### Dependencies

- `RuntimeCtx` extension (new `UserBackend` trait)
- `std::io::stdin`/`stdout` (default backend)

## Cross-References

- Fire-and-forget output: `emit` — `std/user` adds structure and interactivity
- Orchestrator protocol: `yield` — `YieldUserBackend` bridges `std/user` to yield
- Approval gates: [agents-handoff.md](agents-handoff.md) (`agent.gate`) — multi-party approval
- Progress/status: `std/trace` — trace is for programmatic observability, user is for human-facing display
