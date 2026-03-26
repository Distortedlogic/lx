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

**Description:** Run `cargo check -p lx-mobile`. If there are errors, fix them. Common issues:
- `reqwest` workspace version is `0.13.2` but lx-mobile was written for `0.12`. The API may have changed. If `reqwest` methods have changed signatures, adapt the `api_client.rs` calls.
- `tokio-tungstenite` workspace version is `0.29.0` but lx-mobile was written for `0.24`. The `connect_async` API may have changed. If `ws_client.rs` fails, adapt to the new API.
- `tokio::spawn` inside `send_response` in `approvals.rs` requires the tokio runtime to be available. On mobile, Dioxus may or may not provide a tokio runtime. If this fails, replace with `dioxus::spawn`.

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
