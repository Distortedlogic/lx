# Goal

Clean up Cargo.toml dependency management across the lx workspace: hoist crate-local version specs to `[workspace.dependencies]`, convert string shorthand deps to object form, and remove a transient comment.

# Files

- `/home/entropybender/repos/lx/Cargo.toml`
- `/home/entropybender/repos/lx/crates/lx-desktop/Cargo.toml`
- `/home/entropybender/repos/lx/crates/lx-macros/Cargo.toml`

# Steps

## Step 1: Replace entire `[workspace.dependencies]` section in root Cargo.toml

**File:** `/home/entropybender/repos/lx/Cargo.toml`

Find:
```
[workspace.dependencies]
async-recursion = { version = "1.1.1" }
chrono = { version = "0.4.44" }
chumsky = { version = "0.12", features = ["pratt"] }
clap = { version = "4", features = ["derive"] }
cron = { version = "0.15.0" }
dashmap = { version = "6.1.0" }
derive_more = { version = "2.1.1", features = ["deref", "display", "from", "try_from"] }
dioxus = { version = "0.7.3", features = ["fullstack", "router"] }
ena = { version = "0.14" }
extism = { version = "1.20.0" }
fastrand = { version = "2" }
futures = { version = "0.3.32" }
indexmap = { version = "2.13.0" }
itertools = { version = "0.14" }
la-arena = { version = "0.3.1" }
lasso = { version = "0.7.3", features = ["multi-threaded"] }
logos = { version = "0.16.1" }
miette = { version = "7.6.0", features = ["fancy"] }
num-bigint = { version = "0.4.6" }
num-integer = { version = "0.1.46" }
num-traits = { version = "0.2.19" }
parking_lot = { version = "0.12.5" }
pulldown-cmark = { version = "0.13.2" }
regex = { version = "1" }
reqwest = { version = "0.13.2", features = ["json", "query"] }
serde = { version = "1.0.228", features = ["derive"] }
serde_json = { version = "1", features = ["preserve_order"] }
similar = { version = "2.7.0" }
smallvec = { version = "1" }
smart-default = "0.7"
strum = { version = "0.28.0", features = ["derive"] }
thiserror = { version = "2.0.18" }
tokio = { version = "1.50.0", features = ["macros", "rt-multi-thread", "sync", "time"] }
tokio-tungstenite = { version = "0.29.0", features = ["native-tls"] }
toml = { version = "0.9.8" }
```

Replace with:
```
[workspace.dependencies]
anyhow = { version = "1.0.102" }
async-recursion = { version = "1.1.1" }
async-trait = { version = "0.1.89" }
base64 = { version = "0.22.1" }
chrono = { version = "0.4.44" }
chumsky = { version = "0.12", features = ["pratt"] }
clap = { version = "4", features = ["derive"] }
cron = { version = "0.15.0" }
dashmap = { version = "6.1.0" }
derive_more = { version = "2.1.1", features = ["deref", "display", "from", "try_from"] }
dirs = { version = "6.0.0" }
dioxus = { version = "0.7.3", features = ["fullstack", "router"] }
ena = { version = "0.14" }
extism = { version = "1.20.0" }
fastrand = { version = "2" }
futures = { version = "0.3.32" }
indexmap = { version = "2.13.0" }
itertools = { version = "0.14" }
la-arena = { version = "0.3.1" }
lasso = { version = "0.7.3", features = ["multi-threaded"] }
logos = { version = "0.16.1" }
miette = { version = "7.6.0", features = ["fancy"] }
num-bigint = { version = "0.4.6" }
num-integer = { version = "0.1.46" }
num-traits = { version = "0.2.19" }
parking_lot = { version = "0.12.5" }
proc-macro2 = { version = "1" }
pulldown-cmark = { version = "0.13.2" }
quote = { version = "1" }
regex = { version = "1" }
reqwest = { version = "0.13.2", features = ["json", "query"] }
rodio = { version = "0.22.2", default-features = false, features = ["playback", "symphonia-pcm", "symphonia-wav"] }
rustpotter = { version = "3.0.2" }
serde = { version = "1.0.228", features = ["derive"] }
serde_json = { version = "1", features = ["preserve_order"] }
similar = { version = "2.7.0" }
smallvec = { version = "1" }
smart-default = { version = "0.7" }
strum = { version = "0.28.0", features = ["derive"] }
syn = { version = "2", features = ["full"] }
thiserror = { version = "2.0.18" }
tokio = { version = "1.50.0", features = ["macros", "rt-multi-thread", "sync", "time"] }
tokio-tungstenite = { version = "0.29.0", features = ["native-tls"] }
toml = { version = "0.9.8" }
uuid = { version = "1.23.0", features = ["v4"] }
webkit2gtk = { version = "=2.0.1", features = ["v2_38"] }
```

## Step 2: Update lx-desktop Cargo.toml to use workspace dependencies

**File:** `/home/entropybender/repos/lx/crates/lx-desktop/Cargo.toml`

### 2a

Find:
```
anyhow = "1.0.102"
```
Replace with:
```
anyhow = { workspace = true }
```

### 2b

Find:
```
async-trait = "0.1.89"
```
Replace with:
```
async-trait = { workspace = true }
```

### 2c

Find:
```
base64 = "0.22.1"
```
Replace with:
```
base64 = { workspace = true }
```

### 2d

Find:
```
futures = "0.3.32"
```
Replace with:
```
futures = { workspace = true }
```

### 2e

Find:
```
rodio = { version = "0.22.2", default-features = false, features = ["playback", "symphonia-pcm", "symphonia-wav"] }
```
Replace with:
```
rodio = { workspace = true }
```

### 2f

Find:
```
rustpotter = "3.0.2"
```
Replace with:
```
rustpotter = { workspace = true }
```

### 2g

Find:
```
uuid = { version = "1.23.0", features = ["v4"] }
```
Replace with:
```
uuid = { workspace = true }
```

### 2h

Find:
```
dirs = { version = "6.0.0", optional = true }
```
Replace with:
```
dirs = { workspace = true, optional = true }
```

### 2i

Find:
```
webkit2gtk = { version = "=2.0.1", features = ["v2_38"], optional = true }
```
Replace with:
```
webkit2gtk = { workspace = true, optional = true }
```

## Step 3: Delete transient comment in lx-desktop Cargo.toml

**File:** `/home/entropybender/repos/lx/crates/lx-desktop/Cargo.toml`

Find:
```
# @Claude 3/27/26 user note dioxus -> wry -> webkit2gtk
```
Replace with nothing (delete the line).

## Step 4: Update lx-macros Cargo.toml to use workspace dependencies

**File:** `/home/entropybender/repos/lx/crates/lx-macros/Cargo.toml`

### 4a

Find:
```
syn = { version = "2", features = ["full"] }
```
Replace with:
```
syn = { workspace = true }
```

### 4b

Find:
```
quote = "1"
```
Replace with:
```
quote = { workspace = true }
```

### 4c

Find:
```
proc-macro2 = "1"
```
Replace with:
```
proc-macro2 = { workspace = true }
```

## Step 5: Verify

Run `just diagnose` to confirm everything compiles and passes clippy.
