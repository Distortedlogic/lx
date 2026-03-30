# Unit 01: Tailwind Theme Mapping + Border Radius

## Goal

Ensure every Tailwind semantic class used in `components/ui/` resolves to a valid CSS variable in `tailwind.css`, and fix `rounded-full` so circular elements (status dots, toggle knobs, avatars, badges) render as circles instead of squares.

## Preconditions

- No other units need to be complete first.
- The file `crates/lx-desktop/src/tailwind.css` contains the `@theme` block.

## Files to Modify

- `crates/lx-desktop/src/tailwind.css`
- `crates/lx-desktop/src/components/toast_viewport.rs`

## Context: Current @theme Block

The `@theme` block in `crates/lx-desktop/src/tailwind.css` (lines 5-36) defines these mappings:

```
--font-display, --font-body
--radius-sm through --radius-3xl: all 0rem
--radius-full: 9999px
--color-background -> var(--surface)
--color-foreground -> var(--on-surface)
--color-card -> var(--surface-container)
--color-card-foreground -> var(--on-surface)
--color-popover -> var(--surface-container-high)
--color-popover-foreground -> var(--on-surface)
--color-primary -> var(--primary)
--color-primary-foreground -> var(--on-primary)
--color-secondary -> var(--surface-container-high)
--color-secondary-foreground -> var(--on-surface)
--color-muted -> var(--surface-container)
--color-muted-foreground -> var(--on-surface-variant)
--color-accent -> var(--surface-container-high)
--color-accent-foreground -> var(--on-surface)
--color-destructive -> var(--error)
--color-border -> var(--outline-variant)
--color-input -> var(--outline-variant)
--color-ring -> var(--primary)
--color-ring-offset -> var(--surface)
```

## Audit: Semantic Classes Used vs Mappings

### Classes used in `components/ui/*.rs` and their mapping status:

| Semantic class | Used in | CSS variable needed | Status |
|---|---|---|---|
| `bg-primary` | button.rs:33, badge.rs:20, avatar.rs:19 | `--color-primary` | MAPPED |
| `text-primary-foreground` | button.rs:33, badge.rs:20, avatar.rs:19 | `--color-primary-foreground` | MAPPED |
| `text-primary` | button.rs:42, badge.rs:27 | `--color-primary` | MAPPED |
| `bg-destructive` | button.rs:35, badge.rs:23 | `--color-destructive` | MAPPED |
| `ring-destructive` | button.rs:29,35, badge.rs:16,23, input.rs:5, textarea.rs:5, checkbox.rs | `--color-destructive` | MAPPED |
| `bg-background` | button.rs:38, dialog.rs:30, sheet.rs:45 | `--color-background` | MAPPED |
| `bg-accent` | button.rs:38,41, badge.rs:25,26, dropdown_menu.rs:63, command.rs:104, skeleton.rs:7 | `--color-accent` | MAPPED |
| `text-accent-foreground` | button.rs:38,41, badge.rs:25,26, dropdown_menu.rs:63, command.rs:104 | `--color-accent-foreground` | MAPPED |
| `bg-secondary` | button.rs:40, badge.rs:21 | `--color-secondary` | MAPPED |
| `text-secondary-foreground` | button.rs:40, badge.rs:21 | `--color-secondary-foreground` | MAPPED |
| `bg-card` | card.rs:12 | `--color-card` | MAPPED |
| `text-card-foreground` | card.rs:12 | `--color-card-foreground` | MAPPED |
| `text-muted-foreground` | card.rs:53, dialog.rs:105, breadcrumb.rs:24, avatar.rs:17, dropdown_menu.rs:63, command.rs:55,104, select.rs, sheet.rs:121, tabs.rs:25, label.rs | `--color-muted-foreground` | MAPPED |
| `bg-muted` | avatar.rs:17, tabs.rs:25 | `--color-muted` | MAPPED |
| `text-foreground` | badge.rs:25, breadcrumb.rs:50,64, command.rs:90, sheet.rs:110, input.rs:5 | `--color-foreground` | MAPPED |
| `bg-popover` | popover.rs:37, dropdown_menu.rs:38, command.rs:12 | `--color-popover` | MAPPED |
| `text-popover-foreground` | popover.rs:37, dropdown_menu.rs:38, command.rs:12 | `--color-popover-foreground` | MAPPED |
| `border-border` | badge.rs:25 | `--color-border` | MAPPED |
| `border-input` | button.rs:38, input.rs:5, textarea.rs:5 | `--color-input` | MAPPED |
| `bg-input` | button.rs:38, input.rs:5, textarea.rs:5 | `--color-input` | MAPPED |
| `border-ring` / `ring-ring` | button.rs:29, badge.rs:16, input.rs:5, textarea.rs:5, checkbox.rs | `--color-ring` | MAPPED |
| `ring-offset-background` | dialog.rs:37, sheet.rs:53 | `--color-ring-offset` | MAPPED -- but note: Tailwind 4 uses `ring-offset-<color>` differently. The `@theme` has `--color-ring-offset`. Verify this resolves correctly for Tailwind 4. |

### MISSING mappings needed:

| Semantic class | Used in | Required @theme variable |
|---|---|---|
| `ring-background` | avatar.rs:19 (`ring-background`) | Tailwind 4 resolves `ring-<name>` via `--color-<name>`. `--color-background` IS mapped, so `ring-background` resolves to `var(--surface)`. **OK** |

**Conclusion:** All semantic color classes used in `components/ui/` have corresponding `@theme` mappings. No new color variables are needed.

## Problem: Border Radius

Lines 8-14 of `tailwind.css` set `--radius-sm` through `--radius-3xl` to `0rem`. Line 15 sets `--radius-full: 9999px`.

In Tailwind CSS 4, the `rounded-full` utility maps to `border-radius: var(--radius-full)`, which is correctly `9999px`. So `rounded-full` DOES work correctly already.

However, `rounded-md`, `rounded-lg`, `rounded-sm`, `rounded-xl` all resolve to `0rem`. This is intentional for lx-desktop's industrial aesthetic. But there are components that use intermediate radius values where `0rem` may not be desired:

- `badge.rs:16` uses `rounded-full` -- correctly resolves to `9999px` (pill shape)
- `avatar.rs:14-19` uses `rounded-full` -- correctly resolves to `9999px` (circle)
- Status dots in `styles.rs:3-7` use `rounded-full` -- correctly resolves to `9999px`
- Toggle switch in `config_form.rs:146,148` uses `rounded-full` -- correctly resolves to `9999px`
- `dialog.rs:30` uses `rounded-lg` -- resolves to `0rem` (intentional sharp corners)
- `button.rs:29` uses `rounded-md` -- resolves to `0rem` (intentional)

**Conclusion on radius:** The `--radius-full: 9999px` on line 15 already preserves circles. No radius fix is needed.

## Problem: Toast Viewport Hardcoded Colors

`components/toast_viewport.rs` uses hardcoded Tailwind color classes instead of CSS variables:

```rust
// tone_class (lines 6-11):
ToastTone::Info    => "border-sky-500/25 bg-sky-950/60 text-sky-100"
ToastTone::Success => "border-emerald-500/25 bg-emerald-950/60 text-emerald-100"
ToastTone::Warn    => "border-amber-500/25 bg-amber-950/60 text-amber-100"
ToastTone::Error   => "border-red-500/30 bg-red-950/60 text-red-100"

// dot_class (lines 14-19):
ToastTone::Info    => "bg-sky-400"
ToastTone::Success => "bg-emerald-400"
ToastTone::Warn    => "bg-amber-400"
ToastTone::Error   => "bg-red-400"
```

These should use lx-desktop's CSS variable system for consistency.

## Steps

### Step 1: Verify @theme completeness (no code change needed)

All 22 semantic Tailwind classes used across `components/ui/` have correct `@theme` mappings. The `--radius-full: 9999px` preserves circles. No changes needed to `tailwind.css`.

### Step 2: Replace hardcoded colors in toast_viewport.rs

In `crates/lx-desktop/src/components/toast_viewport.rs`, replace the `tone_class` function (lines 5-11) with CSS variable equivalents:

**Replace:**
```rust
fn tone_class(tone: ToastTone) -> &'static str {
  match tone {
    ToastTone::Info => "border-sky-500/25 bg-sky-950/60 text-sky-100",
    ToastTone::Success => "border-emerald-500/25 bg-emerald-950/60 text-emerald-100",
    ToastTone::Warn => "border-amber-500/25 bg-amber-950/60 text-amber-100",
    ToastTone::Error => "border-red-500/30 bg-red-950/60 text-red-100",
  }
}
```

**With:**
```rust
fn tone_class(tone: ToastTone) -> &'static str {
  match tone {
    ToastTone::Info => "border-[var(--tertiary)]/25 bg-[var(--tertiary)]/10 text-[var(--tertiary)]",
    ToastTone::Success => "border-[var(--success)]/25 bg-[var(--success)]/10 text-[var(--success)]",
    ToastTone::Warn => "border-[var(--warning)]/25 bg-[var(--warning)]/10 text-[var(--warning)]",
    ToastTone::Error => "border-[var(--error)]/30 bg-[var(--error)]/10 text-[var(--error)]",
  }
}
```

Replace the `dot_class` function (lines 14-19):

**Replace:**
```rust
fn dot_class(tone: ToastTone) -> &'static str {
  match tone {
    ToastTone::Info => "bg-sky-400",
    ToastTone::Success => "bg-emerald-400",
    ToastTone::Warn => "bg-amber-400",
    ToastTone::Error => "bg-red-400",
  }
}
```

**With:**
```rust
fn dot_class(tone: ToastTone) -> &'static str {
  match tone {
    ToastTone::Info => "bg-[var(--tertiary)]",
    ToastTone::Success => "bg-[var(--success)]",
    ToastTone::Warn => "bg-[var(--warning)]",
    ToastTone::Error => "bg-[var(--error)]",
  }
}
```

The CSS variable mappings are:
- `--tertiary: #81ecff` (replaces sky-400/sky-500/sky-950)
- `--success: #9cff93` (replaces emerald-400/emerald-500/emerald-950)
- `--warning: #fcaf00` (replaces amber-400/amber-500/amber-950)
- `--error: #ff7351` (replaces red-400/red-500/red-950)

### Step 3: No changes to tailwind.css

The `@theme` block is complete for all semantic classes used in `components/ui/`. The `--radius-full: 9999px` correctly preserves circular elements. No edits needed.

## Verification

1. Run `just diagnose` to confirm no compilation errors.
2. Visual check: launch the app and trigger a toast of each tone (Info, Success, Warn, Error). Confirm:
   - Toast background uses a subtle tinted background
   - Toast border matches the tone color
   - Toast text is readable in the tone color
   - The small dot to the left of each toast is the correct tone color and circular
3. Visual check: confirm status dots on the agents list page are circular (not square).
4. Visual check: confirm toggle switches on config_form are pill-shaped (not rectangular).
5. Visual check: confirm avatar fallbacks are circular.
6. Visual check: confirm badges are pill-shaped.
