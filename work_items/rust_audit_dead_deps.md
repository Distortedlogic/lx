# Dead Dependencies Cleanup

Remove unused Cargo dependencies flagged by `cargo shear`.

**Supersedes:** cargo_dependency_cleanup.md steps related to removing unused deps (if any).

---

## Task 1: Remove unused crate-level dependencies

### crates/lx/Cargo.toml

Remove these three lines:

- Line 17: `fastrand.workspace = true`
- Line 30: `regex.workspace = true`
- Line 40: `tokio-tungstenite.workspace = true`

### crates/lx-cli/Cargo.toml

Remove this line:

- Line 19: `indexmap.workspace = true`

### crates/lx-desktop/Cargo.toml

Remove this line:

- Line 23: `common-charts = { path = "../../../dioxus-common/crates/common-charts" }`

### crates/lx-mobile/Cargo.toml

Remove this line:

- Line 18: `tokio = { workspace = true }`

---

## Task 2: Remove unused workspace-level dependencies

### Cargo.toml (workspace root)

Remove these three lines from `[workspace.dependencies]`:

- Line 42: `fastrand = { version = "2" }`
- Line 57: `regex = { version = "1" }`
- Line 70: `tokio-tungstenite = { version = "0.29.0", features = ["native-tls"] }`

---

## Task 3: Remove unused `use` imports

### crates/lx-desktop/src/terminal/toolbar.rs

No change needed — `Pane` is a trait required for `.pane_id()` method calls. Both `Pane` and `PaneNode` are used.

---

## Verification

Run `just diagnose` to confirm no compilation errors after removal.
