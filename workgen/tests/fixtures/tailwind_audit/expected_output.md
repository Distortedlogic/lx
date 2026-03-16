# Goal

Fix Tailwind CSS violations: v3 @tailwind directives, component classes in @utility, custom colors as utility wrappers, raw palette colors, dark: prefix usage, gray tones in backgrounds, outline-none instead of outline-hidden, ring expecting 3px, bare border without color, focus: instead of focus-visible:, wrong shadow/radius scale, and string interpolation mixed with static classes.

# What changes

- Replace `@tailwind base/components/utilities` with `@import "tailwindcss"`
- Move `btn-accent` from `@utility` to `@layer components` — it sets 6+ properties
- Move `text-info` from `@utility` to `@theme` as `--color-accent-info` variable
- Replace raw palette colors `bg-blue-500`, `text-blue-400`, `bg-slate-800`, `text-gray-300` with semantic tokens
- Remove `dark:` prefix — dark-only codebase
- Replace `bg-slate-800` with semantic background token — no gray/blue tones in backgrounds
- Replace `outline-none` with `outline-hidden`
- Replace bare `ring` with explicit `ring-3` for old 3px width
- Add border color `border-border` after bare `border`
- Replace `focus:ring focus:border-blue-500` with `focus-visible:ring-3 focus-visible:border-ring`
- Replace `rounded-sm` with correct v4 scale name
- Split `"{color} text-white px-2 py-1 rounded"` — separate interpolated and static class attributes

# Why

The CSS uses v3 `@tailwind` directives instead of v4 `@import`. Multi-property component class `btn-accent` is in `@utility` instead of `@layer components`. Color utility `text-info` should be a `@theme` variable for auto-generated utilities. Raw palette colors bypass the semantic token system. `dark:` prefix is wrong in a dark-only codebase. Gray/slate backgrounds produce undesirable tones. `outline-none` removes forced-colors accessibility. `ring` is 1px in v4 not 3px. Bare `border` defaults to currentColor. `focus:` shows ring on mouse click.

# Files affected

- src/styles.css — v3 directives, utility classification, custom colors
- src/components.rs — raw palette colors, dark: prefix, outline-none, bare ring, bare border, focus: prefix, shadow/radius scale, mixed class interpolation

# Task List

## Task 1: Fix CSS structure

Replace @tailwind with @import. Move btn-accent to @layer components. Move text-info to @theme.

```
just fmt
git add src/styles.css
git commit -m "fix: v4 imports, correct utility/component/theme classification"
```

## Task 2: Fix component class attributes

Replace raw palette colors with semantic tokens. Remove dark: prefix. Fix outline-none, ring, border, focus. Fix scale names. Split mixed interpolation.

```
just fmt
git add src/components.rs
git commit -m "fix: semantic tokens, v4 scale, focus-visible, split class attrs"
```

## Task 3: Verification

```
just test
just diagnose
just fmt
git add -A
git commit -m "chore: verify tailwind audit fixes"
```

# CRITICAL REMINDERS

- Run `just fmt` after every file change
- Run `just test` and `just diagnose` before final commit
- No @tailwind — use @import "tailwindcss"
- No raw palette colors — use semantic tokens
- No dark: prefix in dark-only codebase

# Task Loading Instructions

Load these instructions by reading this file, then execute each task in order. After each task, run `just fmt` and commit.
