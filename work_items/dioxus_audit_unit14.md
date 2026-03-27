# Unit 14: RSX class interpolation fix in menu_bar.rs

## Problem

`crates/lx-desktop/src/layout/menu_bar.rs` line 199 mixes string interpolation (`{disabled_class}`) with static Tailwind classes in a single `class:` attribute. Per the audit rule, dynamic and static classes should be in separate `class:` attributes.

## Current Code

```rust
// crates/lx-desktop/src/layout/menu_bar.rs line 195-199
let disabled_class = if has_action { "text-[var(--on-surface)]" } else { "text-[var(--on-surface-variant)] opacity-50" };

rsx! {
  button {
    class: if has_action { "w-full flex items-center justify-between px-3 py-1.5 text-sm hover:bg-[var(--surface-container-highest)] transition-colors duration-100 {disabled_class}" } else { "w-full flex items-center justify-between px-3 py-1.5 text-sm transition-colors duration-100 {disabled_class}" },
```

## Fix

Split into separate `class:` attributes. The static classes shared between both branches go in one attribute. The conditional `hover:` class goes in a second conditional attribute. The dynamic `disabled_class` goes in a third attribute.

## Files

| File | Change |
|------|--------|
| `crates/lx-desktop/src/layout/menu_bar.rs` | Replace line 199 with split class attributes |

## Tasks

### 1. Update `crates/lx-desktop/src/layout/menu_bar.rs`

**Line 195**: Remove `disabled_class` variable definition entirely. Instead, handle the two concerns (text color/opacity, hover state) as separate conditional class attributes.

**Line 199**: Replace the single combined class attribute:

```rust
// OLD (line 199):
class: if has_action { "w-full flex items-center justify-between px-3 py-1.5 text-sm hover:bg-[var(--surface-container-highest)] transition-colors duration-100 {disabled_class}" } else { "w-full flex items-center justify-between px-3 py-1.5 text-sm transition-colors duration-100 {disabled_class}" },

// NEW (multiple class attributes):
class: "w-full flex items-center justify-between px-3 py-1.5 text-sm transition-colors duration-100",
class: if has_action { "text-[var(--on-surface)] hover:bg-[var(--surface-container-highest)]" } else { "text-[var(--on-surface-variant)] opacity-50" },
```

This eliminates both the interpolation-in-static-string issue and the `disabled_class` variable. The static layout/sizing classes are in one attribute. The conditional behavior (text color, opacity, hover) is in a second conditional attribute.

## Verification

`just diagnose` must pass with zero warnings.
