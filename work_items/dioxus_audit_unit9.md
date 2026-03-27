# Unit 9: Signal call syntax — .read().clone() → signal()

## Violation

Idiomatic Dioxus pattern: `Signal<T>` implements call syntax via `Readable`, so `signal()` is shorthand for `signal.read().clone()`. `dioxus_storage::use_persistent` returns `Signal<T>`, so call syntax works for persistent signals too.

6 instances across 4 files. (The 7th instance in approvals.rs is handled in Unit 4.)

## Changes

### 1. crates/lx-mobile/src/pages/events.rs, line 26

Current:
```rust
  let current_filter = filter.read().clone();
```

Replace with:
```rust
  let current_filter = filter();
```

### 2. crates/lx-desktop/src/pages/accounts.rs, line 18

Current:
```rust
  let entries = creds.read().clone();
```

Replace with:
```rust
  let entries = creds();
```

### 3. crates/lx-desktop/src/terminal/toolbar.rs, line 24

Current:
```rust
    let val = current_url.read().clone();
```

Replace with:
```rust
    let val = current_url();
```

### 4. crates/lx-desktop/src/pages/settings/state.rs, line 48

Current:
```rust
    let data = use_signal(|| saved.read().clone());
```

Replace with:
```rust
    let data = use_signal(|| saved());
```

### 5. crates/lx-desktop/src/pages/settings/state.rs, line 56

Current:
```rust
    data.set(self.saved.read().clone());
```

Replace with:
```rust
    data.set((self.saved)());
```

Parentheses around `self.saved` are required — without them, `self.saved()` calls a method named `saved` on `self`, which doesn't exist.

### 6. crates/lx-desktop/src/pages/settings/state.rs, line 61

Current:
```rust
    saved.set(self.data.read().clone());
```

Replace with:
```rust
    saved.set((self.data)());
```

Same parenthesization as #5.

## Files Modified

- `crates/lx-mobile/src/pages/events.rs` — line 26
- `crates/lx-desktop/src/pages/accounts.rs` — line 18
- `crates/lx-desktop/src/terminal/toolbar.rs` — line 24
- `crates/lx-desktop/src/pages/settings/state.rs` — lines 48, 56, 61

## Verification

Run `just diagnose` and confirm no errors in the modified files.
