# Goal

Replace custom Dioxus components with dioxus-primitives equivalents: custom modal/dialog, custom tooltip, custom select/dropdown, and custom tabs.

# Why

The codebase reimplements UI components that dioxus-primitives already provides with accessibility, keyboard navigation, and focus management built in. `ConfirmDialog` is a custom modal with overlay + centered content + escape-to-close behavior — replace with AlertDialog primitive. `CustomTooltip` is a custom hover-triggered tooltip — replace with Tooltip primitive. `CustomSelect` is a custom dropdown select with trigger + list + selection — replace with Select primitive. `CustomTabs` is a custom tab component with tab bar + active state — replace with Tabs primitive.

# What changes

- Replace ConfirmDialog with dioxus-primitives AlertDialog (confirmation requiring explicit action)
- Replace CustomTooltip with dioxus-primitives Tooltip
- Replace CustomSelect with dioxus-primitives Select
- Replace CustomTabs with dioxus-primitives Tabs

# Files affected

- src/components.rs — custom modal (AlertDialog), custom tooltip (Tooltip), custom select (Select), custom tabs (Tabs)

# Task List

## Task 1: Replace dialog and tooltip

Replace ConfirmDialog with AlertDialog primitive. Replace CustomTooltip with Tooltip primitive.

```
just fmt
git add src/components.rs
git commit -m "fix: replace custom dialog and tooltip with primitives"
```

## Task 2: Replace select and tabs

Replace CustomSelect with Select primitive. Replace CustomTabs with Tabs primitive.

```
just fmt
git add src/components.rs
git commit -m "fix: replace custom select and tabs with primitives"
```

## Task 3: Verification

```
just test
just diagnose
just fmt
git add -A
git commit -m "chore: verify primitives audit fixes"
```

# CRITICAL REMINDERS

- Run `just fmt` after every file change
- Run `just test` and `just diagnose` before final commit
- No custom components when a dioxus-primitive exists
- Check docs/dioxus-primitives-ref.md for the full catalog

# Task Loading Instructions

Load these instructions by reading this file, then execute each task in order. After each task, run `just fmt` and commit.
