# Unit 12: SettingsState — Signal-per-field → Store

## Problem

`SettingsState` in `crates/lx-desktop/src/pages/settings/state.rs` uses `Signal<SettingsData>` for both `data` and `saved`. `SettingsData` has 7 fields accessed individually by consumer panels (`task_priority.rs`, `env_vars.rs`, `quotas.rs`). Consumers call `settings.data.read().field_name` which subscribes to the entire `SettingsData` on every access.

## Complexity: dioxus_storage::use_persistent

`SettingsState::provide()` calls `dioxus_storage::use_persistent("lx_settings", SettingsData::default)` which returns `Signal<SettingsData>`. The `saved` field is this persistent signal. The store migration must preserve the persistent storage integration — the `saved` signal must remain a `Signal<SettingsData>` because that's what `use_persistent` returns.

Strategy: Convert `SettingsData` to `#[derive(Store)]` and wrap the `data` copy in a Store. Keep `saved` as the raw persistent `Signal<SettingsData>`. The `SettingsState` context becomes a simple struct holding both.

## Current Code

```rust
// crates/lx-desktop/src/pages/settings/state.rs

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SettingsData {
  pub env_vars: Vec<EnvEntry>,
  pub task_priority: f64,
  pub auto_scale: bool,
  pub redundant_verify: bool,
  pub compute_quota: u8,
  pub memory_quota: u8,
  pub storage_quota: u8,
}

#[derive(Clone, Copy)]
pub struct SettingsState {
  pub data: Signal<SettingsData>,
  pub saved: Signal<SettingsData>,
}
```

## Files

| File | Role |
|------|------|
| `crates/lx-desktop/src/pages/settings/state.rs` | Definition — rewrite SettingsData + SettingsState |
| `crates/lx-desktop/src/pages/settings/env_vars.rs` | Consumer — reads/writes `data.env_vars` |
| `crates/lx-desktop/src/pages/settings/task_priority.rs` | Consumer — reads/writes `task_priority`, `auto_scale`, `redundant_verify` |
| `crates/lx-desktop/src/pages/settings/quotas.rs` | Consumer — reads `compute_quota`, `memory_quota`, `storage_quota` |
| `crates/lx-desktop/src/pages/settings/mod.rs` | Provider — calls `SettingsState::provide()`, `discard()`, `execute()` |

## Tasks

### 1. Rewrite `crates/lx-desktop/src/pages/settings/state.rs`

```rust
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EnvEntry {
  pub key: String,
  pub value: String,
}

#[derive(Store, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SettingsData {
  pub env_vars: Vec<EnvEntry>,
  pub task_priority: f64,
  pub auto_scale: bool,
  pub redundant_verify: bool,
  pub compute_quota: u8,
  pub memory_quota: u8,
  pub storage_quota: u8,
}

impl Default for SettingsData {
  fn default() -> Self {
    Self {
      env_vars: vec![
        EnvEntry { key: "API_ENDPOINT_ROOT".into(), value: "https://core.monolith.io/v2".into() },
        EnvEntry { key: "MAX_CONCURRENCY".into(), value: "512".into() },
        EnvEntry { key: "RETRY_POLICY".into(), value: "EXPONENTIAL_BACKOFF".into() },
      ],
      task_priority: 0.84,
      auto_scale: true,
      redundant_verify: false,
      compute_quota: 85,
      memory_quota: 32,
      storage_quota: 95,
    }
  }
}

#[derive(Clone, Copy)]
pub struct SettingsState {
  pub data: Store<SettingsData>,
  pub saved: Signal<SettingsData>,
}

impl SettingsState {
  pub fn provide() -> Self {
    let saved = dioxus_storage::use_persistent("lx_settings", SettingsData::default);
    let data = use_store(|| saved.read().clone());
    let ctx = Self { data, saved };
    use_context_provider(|| ctx);
    ctx
  }

  pub fn discard(&self) {
    let saved_val = (self.saved)();
    self.data.set(saved_val);
  }

  pub fn execute(&self) {
    let data_val = self.data.cloned();
    let mut saved = self.saved;
    saved.set(data_val);
  }
}
```

Key changes:
- `SettingsData` gets `#[derive(Store)]`
- `SettingsState.data` becomes `Store<SettingsData>` (was `Signal<SettingsData>`)
- `SettingsState.saved` stays `Signal<SettingsData>` (from `use_persistent`)
- `provide()` uses `use_store` to create the data store
- `discard()` reads saved signal, sets store
- `execute()` reads store via `.cloned()`, sets saved signal

### 2. Update `crates/lx-desktop/src/pages/settings/env_vars.rs`

**Line 7**: `settings.data` is now `Store<SettingsData>` not `Signal<SettingsData>`:
```rust
// OLD: let mut data = settings.data;
// This line still works — Store is Copy, so binding it to `data` is fine.
// But write access pattern changes.
```

**Line 11**: Replace `.read().env_vars.clone()`:
```rust
// OLD: let env_vars = settings.data.read().env_vars.clone();
// NEW: let env_vars = settings.data.env_vars().cloned();
```

**Line 41**: Replace `data.write().env_vars.remove(i)`:
```rust
// OLD: data.write().env_vars.remove(i);
// NEW: settings.data.env_vars().write().remove(i);
```
Note: Store's Vec impl provides `.remove()` via the write guard.

**Line 70**: Replace `data.write().env_vars.push(...)`:
```rust
// OLD: data.write().env_vars.push(EnvEntry { key: k, value: v });
// NEW: settings.data.env_vars().push(EnvEntry { key: k, value: v });
```
Note: Store's Vec impl provides `.push()` directly.

Remove `let mut data = settings.data;` line since we access through `settings.data.env_vars()` directly.

### 3. Update `crates/lx-desktop/src/pages/settings/task_priority.rs`

**Lines 8-10**: Replace triple `.read()` calls:
```rust
// OLD:
let priority = settings.data.read().task_priority;
let auto_scale = settings.data.read().auto_scale;
let redundant_verify = settings.data.read().redundant_verify;

// NEW:
let priority = settings.data.task_priority().cloned();
let auto_scale = settings.data.auto_scale().cloned();
let redundant_verify = settings.data.redundant_verify().cloned();
```

**Line 33**: Replace `data.write().task_priority = v`:
```rust
// OLD: data.write().task_priority = v;
// NEW: settings.data.task_priority().set(v);
```

Remove `let mut data = settings.data;` (line 7).

**Lines 48-49**: Replace checkbox toggle for auto_scale:
```rust
// OLD:
let mut d = data.write();
d.auto_scale = !d.auto_scale;

// NEW (read current, set opposite):
let current = settings.data.auto_scale().cloned();
settings.data.auto_scale().set(!current);
```

**Lines 60-61**: Same pattern for redundant_verify:
```rust
// OLD:
let mut d = data.write();
d.redundant_verify = !d.redundant_verify;

// NEW:
let current = settings.data.redundant_verify().cloned();
settings.data.redundant_verify().set(!current);
```

### 4. Update `crates/lx-desktop/src/pages/settings/quotas.rs`

**Lines 14-20**: Replace bulk `.read()` + field extraction:
```rust
// OLD:
let data = settings.data.read();
let quotas = [
  QuotaDisplay { label: "COMPUTE_CORE", percent: data.compute_quota, ... },
  QuotaDisplay { label: "MEMORY_BUFFER", percent: data.memory_quota, ... },
  QuotaDisplay { label: "STORAGE_IO", percent: data.storage_quota, ... },
];
drop(data);

// NEW:
let quotas = [
  QuotaDisplay { label: "COMPUTE_CORE", percent: settings.data.compute_quota().cloned(), ... },
  QuotaDisplay { label: "MEMORY_BUFFER", percent: settings.data.memory_quota().cloned(), ... },
  QuotaDisplay { label: "STORAGE_IO", percent: settings.data.storage_quota().cloned(), ... },
];
```

Remove the `drop(data)` line since there's no read guard to drop.

### 5. Update `crates/lx-desktop/src/pages/settings/mod.rs`

No changes needed. `Settings` component calls `SettingsState::provide()` (unchanged), `settings.discard()` (unchanged API), `settings.execute()` (unchanged API). The child components are the ones that change.

## Preconditions

- `SettingsData` already derives `Clone, Debug, PartialEq, Serialize, Deserialize` — adding `Store` is compatible
- `Store<T>` implements `Copy` (like `Signal`) ✓
- `dioxus_storage::use_persistent` returns `Signal<T>` — confirmed, stays as-is
- `Store<SettingsData>` field accessors: `.env_vars()` returns `Store<Vec<EnvEntry>>`, `.task_priority()` returns `Store<f64>`, etc.
- Store's Vec impl provides `.push()`, `.remove()`, `.len()`, `.iter()` directly

## Verification

`just diagnose` must pass with zero warnings.
