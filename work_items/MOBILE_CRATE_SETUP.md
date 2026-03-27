# Goal

Move lx-mobile from `future_crates/` to `crates/`, remove dead dependencies, add to workspace, verify compilation.

# Why

lx-mobile sits in `future_crates/lx-mobile/` with path dependencies on `lx-dx` and `lx-ui` that don't exist. It's not part of the workspace. It can't compile.

# Files Affected

| File | Change |
|------|--------|
| `crates/lx-mobile/` | New — copied from future_crates |
| `crates/lx-mobile/Cargo.toml` | Remove dead deps, use workspace deps |
| `Cargo.toml` (workspace) | Add lx-mobile to members |

# Task List

### Task 1: Copy lx-mobile to crates/

**Subject:** Move the crate to the correct location

**Description:** Copy the entire `future_crates/lx-mobile/` directory to `crates/lx-mobile/`. Keep the original in `future_crates/` — don't delete it.

Run: `cp -r future_crates/lx-mobile crates/lx-mobile`

**ActiveForm:** Copying lx-mobile to crates directory

---

### Task 2: Fix Cargo.toml — remove dead deps, use workspace deps

**Subject:** Remove lx-dx and lx-ui, align with workspace dependency versions

**Description:** Edit `crates/lx-mobile/Cargo.toml`.

Remove these two lines from `[dependencies]`:
```toml
lx-dx = { path = "../lx-dx" }
lx-ui = { path = "../lx-ui" }
```

No code in the crate imports from either package. They compile-fail but are unused.

Replace the `[package]` section to use workspace inheritance:
```toml
[package]
name = "lx-mobile"
version = "0.1.0"
authors.workspace = true
edition.workspace = true
license.workspace = true
repository.workspace = true
```

Add workspace lints:
```toml
[lints]
workspace = true
```

Replace pinned dependency versions with workspace versions where they overlap. The workspace defines: `serde`, `serde_json`, `tokio`, `tokio-tungstenite`, `futures`, `reqwest`. Change these:

```toml
reqwest = { version = "0.12", features = ["json"] }
tokio = { version = "1", features = ["rt-multi-thread", "sync"] }
tokio-tungstenite = "0.24"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
futures = "0.3"
```

To:

```toml
reqwest = { workspace = true }
tokio = { workspace = true }
tokio-tungstenite = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
futures = { workspace = true }
```

Keep these as-is (not in workspace):
```toml
dioxus = { version = "0.7", features = ["mobile", "router"] }
uuid = { version = "1", features = ["v4"] }
```

**ActiveForm:** Fixing Cargo.toml dependencies

---

### Task 3: Add to workspace members

**Subject:** Register lx-mobile in the workspace

**Description:** Edit the root `Cargo.toml`. Add `"crates/lx-mobile"` to the `members` array:

```toml
[workspace]
members = [
  "crates/lx",
  "crates/lx-cli",
  "crates/lx-desktop",
  "crates/lx-macros",
  "crates/lx-mobile",
]
```

**ActiveForm:** Adding lx-mobile to workspace members

---

### Task 4: Verify compilation

**Subject:** Confirm the crate compiles

**Description:** Run `cargo check -p lx-mobile`. Fix any errors. Known issues:

**tokio-tungstenite 0.24→0.29:** `Message::Text` changed from `String` to `Utf8Bytes`. In `ws_client.rs`, the pattern `if let Message::Text(text) = msg` still compiles but `text` is now `Utf8Bytes`, not `String`. The `serde_json::from_str::<serde_json::Value>(&text)` call needs `text.as_str()` or `text.to_string()` since `Utf8Bytes` doesn't deref to `&str` the same way. Change:
```rust
if let Message::Text(text) = msg
  && let Ok(val) = serde_json::from_str::<serde_json::Value>(&text)
```
To:
```rust
if let Message::Text(text) = msg
  && let Ok(val) = serde_json::from_str::<serde_json::Value>(text.as_str())
```

If `Utf8Bytes` doesn't have `.as_str()`, use `&text.to_string()`.

**reqwest 0.12→0.13:** The API methods used by `api_client.rs` (`.get()`, `.post()`, `.json()`, `.send()`, `.json()`) are unchanged between 0.12 and 0.13. No fixes needed.

**tokio::spawn in approvals.rs:** `send_response` uses `tokio::spawn` which requires a tokio runtime. Dioxus mobile provides a tokio runtime (same as desktop). If it fails, replace with `dioxus::prelude::spawn`.

**ActiveForm:** Verifying lx-mobile compilation

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

1. **Call `complete_task` after each task.**
2. **Call `next_task` to get the next task.**
3. **Do not add, skip, reorder, or combine tasks.**
4. **Tasks are implementation-only.**

---

## Task Loading Instructions

```
mcp__workflow__load_work_item({ path: "work_items/MOBILE_CRATE_SETUP.md" })
```
