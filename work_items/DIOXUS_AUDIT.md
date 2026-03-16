The file write keeps getting blocked by permissions. The `workgen/tests/fixtures/dioxus_audit/` directory appears to be outside the allowed write scope even though it's within the repo. Could you grant write permission for this path? Alternatively, I can output the full document content here for you to save manually.

Here is the complete work item document:

---

# Goal

Remediate all Dioxus anti-patterns and convention violations in `src/app.rs`: replace `#[server]` with the correct HTTP method macro, eliminate manual `ServerFnError` construction, switch from `use_resource` to `use_loader` for data loading, remove the `use_effect`/`use_action` coupling anti-pattern, strip the redundant `use_memo` around a plain store field read, split mixed static/interpolated RSX class attributes, inline the pure-forwarding `Wrapper` component, replace direct `tracing::info` usage with the built-in Dioxus logger, remove the direct `dioxus_hooks` sub-crate import, convert `AppState` to a `#[derive(Store)]` type managed via `use_store` instead of a `LazyLock<Mutex<>>`, and move `format_item` into an `impl AppState` block.

# Why

- `#[server]` defaults to POST, masking the true HTTP semantics of a data-retrieval endpoint
- Manual `ServerFnError::new()` adds boilerplate that `anyhow::Result` with `?` eliminates
- `use_resource` in a fullstack app bypasses SSR serialization and suspense; `use_loader` is the correct primitive
- `use_effect` reacting to `use_action` state decouples cause from effect; co-locating the post-completion logic inside the triggering event handler's `spawn` block is idiomatic
- `use_memo` wrapping a single store field accessor adds pointless indirection since `Store` fields are already reactive
- Mixing string interpolation with static Tailwind classes in one `class` attribute harms readability and obscures dynamic behavior
- `Wrapper` is a pure forwarding component with no logic — it adds a layer of indirection for no benefit
- Importing `tracing::info` directly bypasses the built-in Dioxus logger, fragmenting logging configuration
- Importing `dioxus_hooks` directly violates the rule to use `dioxus` re-exports
- `LazyLock<Mutex<AppState>>` is a server-side static pattern misapplied to reactive UI state; `#[derive(Store)]` with `use_store` provides granular field-level subscriptions
- `format_item` is a free function whose first parameter is `&AppState` and accesses its fields — it belongs as a method on `AppState`

# What changes

## Server function `get_data`

- Replace `#[server]` attribute with `#[get]`
- Change return type from `Result<Vec<String>, ServerFnError>` to `anyhow::Result<Vec<String>>`
- Replace `Err(ServerFnError::new("not implemented"))` with `anyhow::bail!("not implemented")`

## Imports

- Remove `use dioxus_hooks::use_future;` — `use_future` is re-exported via `dioxus::prelude::*`
- Remove `use tracing::info;` — replace all `info!` calls with `dioxus::logger::tracing::info!` or add `use dioxus::logger::tracing::info;`

## `AppState` struct

- Add `#[derive(Store)]` to `AppState`
- Remove the `static APP_STATE: LazyLock<Mutex<AppState>>` declaration entirely
- Move `format_item` into an `impl AppState` block as a method `fn format_item(&self, key: &str) -> String`

## `Counter` component

- Replace `use_resource(move || async { get_data().await })` with `use_loader(move || get_data())`
- Remove the `use_effect` block that reacts to `action.value()`; move the post-completion log into a `spawn` block inside the event handler that triggers the action
- Replace `let status = use_memo(move || app_store.name());` with `let status = app_store.name();` — use the store field accessor directly
- Split the `class` attribute on the outer `div` into separate attributes: one for the static classes `"bg-card border-r border-border flex flex-col"` and one for the interpolated `"{nav_width}"`

## `Wrapper` component

- Delete the `Wrapper` component entirely
- At every call site of `Wrapper`, inline a `div { class: "p-4", ... }` directly

# Files affected

- `src/app.rs` — All changes are in this single file: import cleanup, `AppState` Store derivation, `LazyLock` removal, server function attribute and error handling changes, `Counter` hook replacements, `Wrapper` removal, `format_item` migration to method

# Task List

## Task 1: Fix server function attribute and error handling

- **File:** `src/app.rs`
- **Changes:**
  - Replace the `#[server]` attribute on `get_data` with `#[get]`
  - Change the return type from `Result<Vec<String>, ServerFnError>` to `anyhow::Result<Vec<String>>`
  - Replace `Err(ServerFnError::new("not implemented"))` with `anyhow::bail!("not implemented")`
- `just fmt` then `git add -A && git commit -m "fix: replace #[server] with #[get] and use anyhow for error handling"`

## Task 2: Fix imports — remove sub-crate and external logging imports

- **File:** `src/app.rs`
- **Changes:**
  - Remove the line `use dioxus_hooks::use_future;`
  - Replace `use tracing::info;` with `use dioxus::logger::tracing::info;`
- `just fmt` then `git add -A && git commit -m "fix: use dioxus re-exports instead of direct sub-crate and tracing imports"`

## Task 3: Convert AppState to Store and remove LazyLock

- **File:** `src/app.rs`
- **Changes:**
  - Add `#[derive(Store)]` to the `AppState` struct (keep existing derives if any, add `Store`)
  - Delete the entire `static APP_STATE: LazyLock<Mutex<AppState>> = ...` line
  - Move the free function `format_item` into an `impl AppState` block as `fn format_item(&self, key: &str) -> String`, changing `store.name` to `self.name`
  - Delete the standalone `fn format_item` after moving it
- `just fmt` then `git add -A && git commit -m "fix: derive Store on AppState, remove LazyLock, move format_item to impl block"`

## Task 4: Fix Counter hooks — use_loader, remove use_effect/use_action coupling, remove redundant use_memo

- **File:** `src/app.rs`
- **Changes:**
  - Replace `let data = use_resource(move || async { get_data().await });` with `let data = use_loader(move || get_data());`
  - Remove the entire `use_effect` block (lines containing `use_effect(move || { if action.value().is_some() { info!("action complete"); } });`)
  - In the event handler that triggers the action, add a `spawn` block that awaits the action and then calls `info!("action complete")` on success
  - Replace `let status = use_memo(move || app_store.name());` with `let status = app_store.name();`
- `just fmt` then `git add -A && git commit -m "fix: use_loader for data loading, co-locate action completion logic, remove redundant memo"`

## Task 5: Split mixed class attribute in RSX

- **File:** `src/app.rs`
- **Changes:**
  - On the outer `div` in `Counter`, replace the single `class: "{nav_width} bg-card border-r border-border flex flex-col"` with two separate class attributes:
    - `class: "bg-card border-r border-border flex flex-col",`
    - `class: "{nav_width}",`
- `just fmt` then `git add -A && git commit -m "fix: split interpolated and static classes into separate class attributes"`

## Task 6: Remove Wrapper component

- **File:** `src/app.rs`
- **Changes:**
  - Delete the entire `Wrapper` component (the `#[component]` attribute and the `fn Wrapper(children: Element) -> Element` function)
  - Search for any call sites of `Wrapper { ... }` in the file and replace with `div { class: "p-4", ... }` inline
- `just fmt` then `git add -A && git commit -m "fix: inline Wrapper component, remove pure-forwarding wrapper"`

## Task 7: Verification

- Run `just test` to confirm all tests pass
- Run `just diagnose` to confirm no compiler errors or clippy warnings
- Run `just fmt` to confirm formatting is clean

---

# CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.

# Task Loading Instructions

To begin executing this work item, run:

```
mcp__workflow__load_work_item({ path: "workgen/tests/fixtures/dioxus_audit/expected_output.md" })
```