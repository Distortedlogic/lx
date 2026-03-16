# Goal

Fix all 18 Tailwind CSS audit violations across `styles.css` and `components.rs` in the `workgen/tests/fixtures/tailwind_audit/src/` directory. The violations span v3-to-v4 migration issues (deprecated directives, wrong scale names, outline behavior change), incorrect CSS layer placement (component classes in `@utility`, bare classes outside any layer), raw palette colors instead of semantic tokens, dark-mode prefixes in a dark-only codebase, gray/blue background tones, missing border colors, wrong focus variant, string interpolation mixed with static classes, and conditional class bindings hoisted into `let` instead of inline `class: if`.

# Why

- v3 `@tailwind` directives are removed in Tailwind v4 and will fail to compile
- Custom colors defined as `@utility` wrappers instead of `@theme` variables miss auto-generation across all color namespaces (bg-*, border-*, fill-*, stroke-*, ring-*)
- Component classes placed in `@utility` cannot use CSS nesting for hover/focus/disabled states and violate the single-concern-per-utility rule
- Bare classes outside any layer have unpredictable specificity and cannot be overridden by utilities
- v3 shadow/rounded scale names produce different visuals in v4 (shadow-sm is now shadow-xs, rounded-sm is now rounded-xs)
- `outline-none` in v4 removes the forced-colors-mode accessibility outline; `outline-hidden` preserves it
- Bare `ring` is 1px in v4 (was 3px in v3), producing a nearly invisible focus indicator
- Raw palette colors (`bg-slate-800`, `text-gray-300`, `bg-green-500`, `bg-red-500`, `bg-blue-500`, `focus:border-blue-500`) bypass semantic tokens, making theme changes impossible and introducing prohibited gray/blue background tones
- `dark:` variants are wrong in a dark-only codebase — there is no light mode to conditionally override
- Bare `border` defaults to `currentColor` in v4 (was gray-200 in v3), producing unexpected border colors
- `focus:` shows the focus ring on mouse click; `focus-visible:` restricts it to keyboard navigation
- String interpolation mixed with static classes prevents Tailwind's class detection and breaks splitting
- Conditional class `let` bindings add indirection that Dioxus's inline `class: if` syntax eliminates

# What changes

## styles.css

- Replace `@tailwind base; @tailwind components; @tailwind utilities;` with `@import "tailwindcss";`
- Remove `@utility text-info` entirely; add `--color-accent-info: oklch(0.707 0.165 254.624);` to a new `@theme` block (this auto-generates `text-accent-info`, `bg-accent-info`, `border-accent-info`, etc.)
- Remove `@utility btn-accent`; rewrite as `.btn-accent` inside `@layer components` using raw CSS with theme variable references: `background-color: var(--color-primary)`, `color: white`, `border-radius: var(--radius-lg)`, `padding: --spacing(2) --spacing(4)`, `font-weight: var(--font-weight-medium)`, `box-shadow: var(--shadow-xs)` (v4 equivalent of v3 shadow-sm). Add `&:hover` nested inside `@media (hover: hover)` with darkened primary background
- Move `.custom-card` inside `@layer components`; convert hardcoded values to theme variable references: `background: var(--color-card)`, `border-radius: var(--radius-lg)`, `padding: --spacing(4)`

## components.rs

- Card: replace `bg-slate-800` with `bg-card`, replace `rounded-sm` with `rounded-xs`, add `border-border` after `border`, replace `text-gray-300` with `text-muted-foreground`, remove the entire `dark: "dark:bg-slate-900"` line
- StatusBadge: remove the `let color = ...` binding, split the single `class:` into two attributes — `class: "text-white px-2 py-1 rounded"` for static classes and `class: if status == "active" { "bg-success" } else { "bg-destructive" }` for the conditional semantic colors
- Input: replace `outline-none` with `outline-hidden`, replace `focus:ring` with `focus-visible:ring-2`, replace `focus:border-blue-500` with `focus:border-ring` (input border focus exception), add `border-border` after `border`

# Files affected

- `workgen/tests/fixtures/tailwind_audit/src/styles.css` — replace v3 directives, add `@theme` block with accent-info color, rewrite btn-accent as `@layer components` class, move custom-card into `@layer components`, remove text-info utility
- `workgen/tests/fixtures/tailwind_audit/src/components.rs` — fix Card (semantic tokens, remove dark:, fix rounded/border), fix StatusBadge (inline class:if, semantic colors, split interpolation), fix Input (outline-hidden, focus-visible, semantic border color, border-border)

# Task List

## Task 1: Replace v3 directives and restructure styles.css CSS layers

**Files:** `workgen/tests/fixtures/tailwind_audit/src/styles.css`

Replace the three `@tailwind` directives with a single `@import "tailwindcss";` line.

Add a `@theme` block containing `--color-accent-info: oklch(0.707 0.165 254.624);`.

Remove the `@utility text-info` block entirely (the `@theme` variable auto-generates `text-accent-info` and all other color utilities).

Remove the `@utility btn-accent` block. Create a `@layer components` block containing `.btn-accent` with raw CSS properties: `background-color: var(--color-primary)`, `color: white`, `border-radius: var(--radius-lg)`, `padding-inline: --spacing(4)`, `padding-block: --spacing(2)`, `font-weight: var(--font-weight-medium)`, `box-shadow: var(--shadow-xs)`. Nest hover state as `@media (hover: hover) { &:hover { background-color: var(--color-primary-foreground); } }` — or use a slightly darkened primary via opacity: `background-color: --alpha(var(--color-primary) / 80%)`.

Move the existing `.custom-card` rule into the same `@layer components` block. Replace its hardcoded values: `background` becomes `var(--color-card)`, `border-radius` becomes `var(--radius-lg)`, `padding` becomes `--spacing(4)`.

After editing, run: `just fmt` then `git add workgen/tests/fixtures/tailwind_audit/src/styles.css` then `git commit -m "fix(tailwind): replace v3 directives, restructure CSS layers, add theme variable"`.

## Task 2: Fix Card component in components.rs

**Files:** `workgen/tests/fixtures/tailwind_audit/src/components.rs`

In the Card component's outer `div`:
- Replace `bg-slate-800` with `bg-card`
- Replace `rounded-sm` with `rounded-xs`
- Change `border` to `border border-border`
- Remove the entire `dark: "dark:bg-slate-900"` attribute line

In the Card component's `h2`:
- Replace `text-gray-300` with `text-muted-foreground`

After editing, run: `just fmt` then `git add workgen/tests/fixtures/tailwind_audit/src/components.rs` then `git commit -m "fix(tailwind): Card component - semantic tokens, remove dark:, fix scale names"`.

## Task 3: Fix StatusBadge component in components.rs

**Files:** `workgen/tests/fixtures/tailwind_audit/src/components.rs`

Remove the `let color = if status == "active" { "bg-green-500" } else { "bg-red-500" };` binding entirely.

Replace the single `class: "{color} text-white px-2 py-1 rounded"` attribute on the `span` with two separate `class:` attributes:
- `class: "text-white px-2 py-1 rounded"` for the static classes
- `class: if status == "active" { "bg-success" } else { "bg-destructive" }` for the conditional class

After editing, run: `just fmt` then `git add workgen/tests/fixtures/tailwind_audit/src/components.rs` then `git commit -m "fix(tailwind): StatusBadge - semantic tokens, inline class:if, split interpolation"`.

## Task 4: Fix Input component in components.rs

**Files:** `workgen/tests/fixtures/tailwind_audit/src/components.rs`

In the Input component's `input` element, change the class string from `"border outline-none focus:ring focus:border-blue-500"` to `"border border-border outline-hidden focus-visible:ring-2 focus:border-ring"`.

This addresses: bare `border` without color (add `border-border`), `outline-none` to `outline-hidden`, bare `focus:ring` to `focus-visible:ring-2` (keyboard-only, explicit width), raw `focus:border-blue-500` to `focus:border-ring` (input border focus exception).

After editing, run: `just fmt` then `git add workgen/tests/fixtures/tailwind_audit/src/components.rs` then `git commit -m "fix(tailwind): Input - outline-hidden, focus-visible ring, semantic border colors"`.

## Task 5: Verify all changes

Run `just test` and confirm all tests pass. Run `just diagnose` and confirm zero warnings and zero errors. Run `just fmt` and confirm no formatting changes needed.

---

# CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.

# Task Loading Instructions

To begin executing this work item, run:

```
mcp__workflow__load_work_item({ path: "work_items/TAILWIND_AUDIT_FIXES.md" })
```

Then call `next_task` to get the first task and begin implementation.