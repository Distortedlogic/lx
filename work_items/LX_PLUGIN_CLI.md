# Goal

Add `lx plugin` CLI subcommand for installing, listing, removing, and scaffolding WASM plugins.

# Why

Developers need a way to manage lx extensions without manually copying files into `~/.lx/plugins/`.

# Depends On

- `WASM_PLUGIN_SYSTEM.md` — plugin loading and plugin.toml format must exist first

# Commands

```
lx plugin install <path>      Install plugin from local directory
lx plugin list                 List installed plugins (global + project-local)
lx plugin remove <name>        Remove a global plugin
lx plugin new <name>           Scaffold a new WASM plugin project
```

# What Changes

**`crates/lx-cli/src/main.rs`** — add `Plugin` subcommand to CLI:

```rust
enum Command {
    // ... existing
    Plugin { action: PluginAction },
}

enum PluginAction {
    Install { path: PathBuf },
    List,
    Remove { name: String },
    New { name: String },
}
```

**`crates/lx-cli/src/plugin.rs`** (new file):

### `lx plugin install <path>`
1. Canonicalize source path
2. Read `plugin.toml` from source — error if missing
3. Validate: `[plugin].name`, `[plugin].version`, `[plugin].wasm` must exist
4. Validate: wasm file referenced in manifest must exist at source path
5. Create `~/.lx/plugins/` if it doesn't exist (`std::fs::create_dir_all`)
6. Target dir: `~/.lx/plugins/{name}/`
7. If target exists: print "updating {name} {old_version} → {new_version}", remove old dir
8. Copy entire source directory to target (recursive copy, not symlink — plugin should be self-contained)
9. Print "installed {name} {version} to ~/.lx/plugins/{name}/"

### `lx plugin list`
1. Scan `~/.lx/plugins/` — for each subdirectory, read `plugin.toml`
2. Scan `.lx/plugins/` relative to cwd — same
3. Print table:

```
Name       Version  Location  Description
json       0.1.0    global    JSON parsing and encoding
regex      0.2.1    global    Regular expression matching
my-plugin  0.1.0    local     Custom project plugin
```

4. If no plugins found, print "no plugins installed"
5. Errors reading individual manifests are printed as warnings, don't abort the list

### `lx plugin remove <name>`
1. Check `~/.lx/plugins/{name}/` exists
2. If not, error: "plugin '{name}' not found in ~/.lx/plugins/"
3. Remove directory recursively
4. Print "removed {name}"
5. Does NOT remove project-local plugins — user should delete those manually

### `lx plugin new <name>`
1. Create directory `./{name}/`
2. Error if directory already exists

3. Write `Cargo.toml`:
```toml
[package]
name = "{name}"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]

[dependencies]
extism-pdk = "1.4.1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

4. Write `src/lib.rs`:
```rust
use extism_pdk::*;

#[plugin_fn]
pub fn hello(input: String) -> FnResult<String> {
    Ok(format!("Hello from {name}: {{input}}"))
}
```

5. Write `plugin.toml`:
```toml
[plugin]
name = "{name}"
version = "0.1.0"
description = ""
wasm = "target/wasm32-unknown-unknown/release/{name}.wasm"

[exports]
hello = { arity = 1 }
```

6. Write `.cargo/config.toml`:
```toml
[build]
target = "wasm32-unknown-unknown"
```

7. Print:
```
Created plugin project '{name}'

Build:   cargo build --release
Install: lx plugin install ./{name}
```

# Gotchas

- `~/.lx/` may not exist on first run. `lx plugin install` and `lx plugin new` create it.
- Recursive directory copy needs to handle symlinks correctly (follow them, don't copy as symlinks).
- Plugin names must be valid directory names — reject names with `/`, `\`, `..`, or whitespace.
- `lx plugin new` writes `.cargo/config.toml` with wasm target so `cargo build --release` just works without `--target` flag.
- The scaffold's `Cargo.toml` uses `edition = "2024"` — verify this is correct for the user's Rust toolchain. If not, fall back to `"2021"`.

# Task List

### Task 1: Add plugin subcommand to CLI
Edit `crates/lx-cli/src/main.rs`. Add `Plugin` variant to command enum. Add argument parsing for install/list/remove/new subcommands.

### Task 2: Implement `lx plugin install`
Create `crates/lx-cli/src/plugin.rs`. Implement install with manifest validation, directory creation, recursive copy. Handle update case.

### Task 3: Implement `lx plugin list`
Scan both plugin directories. Read manifests. Print formatted table. Handle missing/malformed manifests gracefully.

### Task 4: Implement `lx plugin remove`
Delete global plugin directory. Error on not-found. Print confirmation.

### Task 5: Implement `lx plugin new`
Create scaffolded project with all four files. Validate name. Print build/install instructions.

### Task 6: Test all subcommands
Create temp directories. Test install → list shows it → remove → list empty. Test new creates valid project structure. Test install with invalid manifest errors cleanly.

---

## Task Loading Instructions

```
mcp__workflow__load_work_item({ path: "work_items/LX_PLUGIN_CLI.md" })
```
