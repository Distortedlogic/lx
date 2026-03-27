# Unit 9: Signal call syntax — .read().clone() → signal()

## Violation

Idiomatic Dioxus pattern: `Signal<T>` implements `Fn() -> T` (via `Readable`), so `signal()` is the idiomatic shorthand for `signal.read().clone()`. This is a mechanical replacement across 4 files (6 instances). The approvals.rs instance is handled in Unit 4.

## Locations

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

Note: `creds` is from `dioxus_storage::use_persistent` which returns a persistent signal. Verify it supports call syntax (implements `Readable` / `Fn`). If not, leave as-is.

### 3. crates/lx-desktop/src/terminal/toolbar.rs, line 24

Current:
```rust
    let val = current_url.read().clone();
```

Replace with:
```rust
    let val = current_url();
```

Note: `current_url` is a `ReadSignal<String>` (prop). `ReadSignal` implements `Readable` and supports call syntax.

### 4. crates/lx-desktop/src/pages/settings/state.rs, line 48

Current:
```rust
    let data = use_signal(|| saved.read().clone());
```

Replace with:
```rust
    let data = use_signal(|| saved());
```

Note: `saved` is from `dioxus_storage::use_persistent`. Same caveat as #2.

### 5. crates/lx-desktop/src/pages/settings/state.rs, line 56

Current:
```rust
    data.set(self.saved.read().clone());
```

Replace with:
```rust
    data.set((self.saved)());
```

Note: Parentheses around `self.saved` are needed because `self.saved()` would try to call a method named `saved` on `self`. If `(self.saved)()` is less readable, keep `.read().clone()` — the audit rule for Store says ".cloned() is acceptable" when call syntax harms readability. Same principle applies here.

### 6. crates/lx-desktop/src/pages/settings/state.rs, line 61

Current:
```rust
    saved.set(self.data.read().clone());
```

Replace with:
```rust
    saved.set((self.data)());
```

Same parenthesization note as #5.

## Judgment Call for #5 and #6

`(self.saved)()` and `(self.data)()` are arguably less readable than `.read().clone()`. If the executing agent judges these harm readability, keep the original form for those two instances only.

## Files Modified

- `crates/lx-mobile/src/pages/events.rs` — line 26
- `crates/lx-desktop/src/pages/accounts.rs` — line 18
- `crates/lx-desktop/src/terminal/toolbar.rs` — line 24
- `crates/lx-desktop/src/pages/settings/state.rs` — lines 48, 56, 61

## Verification

Run `just diagnose` and confirm no errors in the modified files.
