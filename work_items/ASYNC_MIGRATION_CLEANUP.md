# Goal

Clean up technical debt from the Session 71d async interpreter migration. Remove the unused rayon dependency, eliminate `reqwest::blocking` in favor of async reqwest, and remove the `rayon_pool` field from `RuntimeCtx`. The `call_value_sync` bridge and sync backend traits are intentionally kept — converting those is a separate, larger effort.

# Why

- **Rayon is dead weight.** The `rayon` crate is declared in `Cargo.toml`, a `rayon_pool` field is created on every `RuntimeCtx`, and a `rayon::ThreadPool` is built on every program startup — but zero code paths use it after the async migration. This wastes compile time, binary size, and thread resources.
- **`reqwest::blocking` fights the async runtime.** Every blocking reqwest call must be wrapped in `tokio::task::block_in_place()` to avoid panicking when called from within a tokio runtime. The lx crate already depends on `reqwest` (which includes the async client) — the blocking feature is a second HTTP client for no reason.
- **The `rayon_pool` field on `RuntimeCtx` misleads future developers.** It looks like concurrency depends on rayon when it doesn't. Removing it makes the architecture honest.

# What Changes

## 1. Remove rayon dependency

- `crates/lx/Cargo.toml`: delete `rayon = "1"` line
- `crates/lx/src/backends/mod.rs`: remove `rayon_pool: Arc<rayon::ThreadPool>` field from `RuntimeCtx`, remove the `rayon::ThreadPoolBuilder` initialization from `Default::default()`, remove `use` of rayon types
- Every file that references `ctx.rayon_pool` or `self.ctx.rayon_pool` — the explore agent confirmed zero such references exist, but verify with `rg rayon crates/lx/` after deletion

## 2. Convert `ReqwestHttpBackend` from blocking to async

- `crates/lx/src/backends/defaults.rs`: change `use reqwest::blocking::Client` to `use reqwest::Client` (async). The `request` method stays sync (the `HttpBackend` trait is sync) but internally uses `tokio::task::block_in_place(|| Handle::current().block_on(async { ... }))` with the async reqwest client. This eliminates the nested-runtime panic risk because async reqwest uses the existing tokio runtime instead of creating its own.
- Response handling: `resp.text().await` inside the block_on, `resp.status()` etc. are sync on the async Response too.
- The `response_to_value` helper takes `reqwest::Response` (async) instead of `reqwest::blocking::Response`.

## 3. Convert MCP HTTP transport from blocking to async

- `crates/lx/src/stdlib/mcp_http.rs`: change `reqwest::blocking::Client` to `reqwest::Client` (async). Store the async client. All methods that do HTTP (`post`, `send`, `send_notify`, `shutdown`) use `block_in_place(|| Handle::current().block_on(async { ... }))`. The `new` constructor no longer needs `block_in_place` for `Client::builder().build()` since async reqwest's builder doesn't create a runtime.
- Remove all `reqwest::blocking::Response` references, replace with `reqwest::Response`.

## 4. Remove `reqwest` blocking feature

- `Cargo.toml` (workspace or crate level): ensure `reqwest` is declared without the `blocking` feature. Check if `features = ["blocking"]` or similar exists and remove it.

# How It Works

After these changes:
- `RuntimeCtx` has `tokio_runtime` but no `rayon_pool`
- HTTP requests use async reqwest inside `block_in_place` + `block_on` (same thread-yielding behavior, but no nested runtime creation)
- MCP HTTP transport uses async reqwest the same way
- No code path creates a `reqwest::blocking::Client` or a `rayon::ThreadPool`

The `block_in_place` pattern remains because the `HttpBackend` trait is sync. Making it async is a separate work item (requires changing the trait, all implementations, and all call sites in the interpreter + stdlib).

# Files Affected

| File | Change |
|------|--------|
| `crates/lx/Cargo.toml` | Remove `rayon = "1"`, verify reqwest has no `blocking` feature |
| `crates/lx/src/backends/mod.rs` | Remove `rayon_pool` field, remove rayon imports |
| `crates/lx/src/backends/defaults.rs` | Convert `ReqwestHttpBackend` from `reqwest::blocking` to async `reqwest` with `block_in_place`+`block_on` |
| `crates/lx/src/stdlib/mcp_http.rs` | Convert `HttpTransport` from `reqwest::blocking` to async `reqwest` with `block_in_place`+`block_on` |

# Task List

### Task 1: Remove rayon from Cargo.toml

In `crates/lx/Cargo.toml`, delete the line `rayon = "1"`. Verify no other Cargo.toml in the workspace depends on rayon.

### Task 2: Remove rayon_pool from RuntimeCtx

In `crates/lx/src/backends/mod.rs`:
- Remove the `pub rayon_pool: Arc<rayon::ThreadPool>,` field from the `RuntimeCtx` struct
- Remove the `rayon_pool: Arc::new(rayon::ThreadPoolBuilder::new().build().expect("failed to create rayon thread pool")),` line from `Default::default()`
- Remove any `use` statement for rayon types

### Task 3: Format and commit

Run the following command verbatim, exactly as written, with no modifications: `just fmt`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way.

### Task 4: Commit rayon removal

Run the following command verbatim, exactly as written, with no modifications: `git add -A && git commit -m "remove unused rayon dependency and rayon_pool from RuntimeCtx"`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way. CRITICAL: The commit command must be a plain string — do NOT use `$()`, heredocs, `cat <<EOF`, or any subshell substitution in the commit message. Do NOT append `--trailer`, `Co-authored-by`, `Signed-off-by`, or any other trailer/metadata to the commit command.

### Task 5: Convert ReqwestHttpBackend to async reqwest

In `crates/lx/src/backends/defaults.rs`:
- Change `use reqwest::blocking::Client;` to `use reqwest::Client;`
- In `ReqwestHttpBackend::request()`, the body is already inside `tokio::task::block_in_place(|| { ... })`. Change it to `tokio::task::block_in_place(|| { tokio::runtime::Handle::current().block_on(async { ... }) })` where the inner async block uses the async reqwest client: `Client::builder().build()` (returns `Result`, not a nested runtime), `builder.send().await`, etc.
- Update `response_to_value` to accept `reqwest::Response` instead of `reqwest::blocking::Response`. The `.status()`, `.headers()` methods are the same. `.text()` becomes `.text().await` (call inside the async block before passing to the helper, or make the helper async within the block_on).
- Remove all `reqwest::blocking` references from the file.

### Task 6: Format and commit

Run the following command verbatim, exactly as written, with no modifications: `just fmt`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way.

### Task 7: Commit HTTP backend conversion

Run the following command verbatim, exactly as written, with no modifications: `git add -A && git commit -m "convert ReqwestHttpBackend from reqwest::blocking to async reqwest"`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way. CRITICAL: The commit command must be a plain string — do NOT use `$()`, heredocs, `cat <<EOF`, or any subshell substitution in the commit message. Do NOT append `--trailer`, `Co-authored-by`, `Signed-off-by`, or any other trailer/metadata to the commit command.

### Task 8: Convert MCP HTTP transport to async reqwest

In `crates/lx/src/stdlib/mcp_http.rs`:
- Change `use reqwest::blocking::Client;` to `use reqwest::Client;`
- Change the `client: Client` field type (it's now the async client)
- In `new()`: remove the `tokio::task::block_in_place` wrapper around `Client::builder().build()` — async reqwest's builder is sync and doesn't create a runtime. Keep the `map_err`.
- In `post()`: change to `tokio::task::block_in_place(|| { tokio::runtime::Handle::current().block_on(async { builder.send().await.map_err(...) }) })`.
- In `send()`: the `resp.text()` call is already in `block_in_place`. Change to `block_in_place(|| Handle::current().block_on(async { resp.text().await.map_err(...) }))`.
- In `send_notify()`: wrap the response status check in `block_in_place` + `block_on` if needed (or just read status synchronously since `reqwest::Response::status()` is sync on async Response too).
- In `shutdown()`: change `self.client.delete(...).send()` to the async equivalent inside `block_in_place` + `block_on`.
- Update `capture_session_id` to accept `&reqwest::Response` instead of `&reqwest::blocking::Response`. The `.headers()` method is the same.
- Update `content_type` helper to accept `&reqwest::Response`.
- Remove all `reqwest::blocking` references from the file.

### Task 9: Format and commit

Run the following command verbatim, exactly as written, with no modifications: `just fmt`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way.

### Task 10: Commit MCP transport conversion

Run the following command verbatim, exactly as written, with no modifications: `git add -A && git commit -m "convert MCP HTTP transport from reqwest::blocking to async reqwest"`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way. CRITICAL: The commit command must be a plain string — do NOT use `$()`, heredocs, `cat <<EOF`, or any subshell substitution in the commit message. Do NOT append `--trailer`, `Co-authored-by`, `Signed-off-by`, or any other trailer/metadata to the commit command.

### Task 11: Remove reqwest blocking feature if present

Check `Cargo.toml` files (both workspace root and `crates/lx/Cargo.toml`) for any `features = ["blocking"]` or `"blocking"` in the reqwest dependency. If present, remove the `blocking` feature. If not present, this task is a no-op — mark complete.

### Task 12: Format and commit (if changes made)

Run the following command verbatim, exactly as written, with no modifications: `just fmt`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way.

### Task 13: Commit feature removal (if changes made)

If Task 11 made changes, run the following command verbatim, exactly as written, with no modifications: `git add -A && git commit -m "remove reqwest blocking feature"`. Do NOT pipe, redirect, append shell operators. If Task 11 was a no-op, skip this task.

### Task 14: Run tests

Run the following command verbatim, exactly as written, with no modifications: `just test`. Do NOT pipe, redirect, or append shell operators. If any tests fail, fix all failures and re-run until all tests pass.

### Task 15: Run diagnostics

Run the following command verbatim, exactly as written, with no modifications: `just diagnose`. Do NOT pipe, redirect, or append shell operators. Fix ALL reported errors AND warnings. Re-run `just diagnose` until the output is completely clean with zero errors and zero warnings.

### Task 16: Final format

Run the following command verbatim, exactly as written, with no modifications: `just fmt`. Do NOT pipe, redirect, or append shell operators.

### Task 17: Final commit

Run the following command verbatim, exactly as written, with no modifications: `git add -A && git commit -m "fix: final verification cleanup"`. Do NOT pipe, redirect, or append shell operators. CRITICAL: Do NOT use `$()`, heredocs, `cat <<EOF`, or any subshell substitution — just a plain `-m "message"` flag. Do NOT append `--trailer`, `Co-authored-by`, `Signed-off-by`, or any other trailer/metadata.

### Task 18: Remove work item file

Run the following command verbatim, exactly as written, with no modifications: `rm work_items/ASYNC_MIGRATION_CLEANUP.md && git add -A && git commit -m "chore: remove completed work item"`. Do NOT pipe, redirect, or append shell operators. Do NOT append `--trailer`, `Co-authored-by`, `Signed-off-by`, or any other trailer/metadata.

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

These are rules you (the executor) are known to violate. Re-read before starting each task:

1. **NEVER run raw cargo commands.** No `cargo check`, `cargo test`, `cargo clippy`, `cargo fmt`. ALWAYS use `just fmt`, `just test`, `just diagnose`, `just fix`. The `just` recipes include additional steps that raw `cargo` skips.
2. **Between implementation tasks, ONLY run `just fmt` + `git commit`.** Do NOT run `just test`, `just diagnose`, or any compilation/verification command between implementation tasks. These are expensive and only belong in the FINAL VERIFICATION section at the end.
3. **Run commands VERBATIM.** Copy-paste the exact command from the task description. Do not modify, "improve," or substitute commands. Do NOT append `--trailer`, `Co-authored-by`, or any metadata to commit commands.
4. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
5. **`just fmt` is not `cargo fmt`.** `just fmt` runs `dx fmt` + `cargo fmt` + `eclint`. Running `cargo fmt` alone is wrong.
6. **Do NOT modify verbatim commands.** No piping (`| tail`, `| head`, `| grep`), no redirects (`2>&1`, `> /dev/null`), no subshells. Run the exact command string as written — nothing appended, nothing prepended.

## Task Loading Instructions

Read each `### Task N:` entry from the `## Task List` section above. For each task, call `TaskCreate` with:
- `subject`: The task heading text (after `### Task N:`) — copied VERBATIM, not paraphrased
- `description`: The full body text under that heading — copied VERBATIM, not paraphrased, summarized, or reworded. Every sentence, every command, every instruction must be transferred exactly as written. Do NOT omit lines, rephrase instructions, drop the "verbatim" language from command instructions, or inject your own wording.
- `activeForm`: A present-continuous form of the subject (e.g., "Removing rayon from Cargo.toml")

After creating all tasks, use `TaskUpdate` to set `addBlockedBy` on each task N (N > 1) pointing to task N-1, enforcing strict sequential execution.

Execution rules:
- Execute tasks strictly in order — mark each `in_progress` before starting and `completed` when done
- Run commands EXACTLY as written in the task description — do not substitute `cargo` for `just` or vice versa
- Do not run any command not specified in the current task
- Do not "pre-check" compilation between implementation tasks — the task list already has verification in the correct places
- If a task says "Run the following command verbatim" then copy-paste that exact command — do not modify it. Do NOT append `--trailer`, `Co-authored-by`, `Signed-off-by`, or any other trailer/metadata to git commit commands.
- Do NOT paraphrase, summarize, reword, combine, split, reorder, skip, or add tasks beyond what is in the Task List section
- When a task description says "Run the following command verbatim, exactly as written, with no modifications" — that phrase and the command after it must appear identically in the loaded task. Do not drop the "verbatim" instruction or rephrase the command.
- Do NOT append shell operators to commands — no pipes (`|`), no redirects (`>`/`2>&1`), no subshells. The command in the task description is the complete command string.
