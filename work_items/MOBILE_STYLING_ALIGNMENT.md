# Goal

Replace raw Tailwind color classes in lx-mobile with the CSS variable design system from lx-desktop. Add the shared `tailwind.css` to the mobile app.

# Why

lx-mobile uses `bg-gray-900`, `text-gray-100`, `bg-blue-600`, etc. lx-desktop uses `bg-[var(--surface)]`, `text-[var(--on-surface)]`, `bg-[var(--primary)]`, etc. The two apps look different. The mobile app should use the same design tokens.

# CSS Variable Mapping

From lx-desktop's `src/tailwind.css`:

| Raw Tailwind | CSS Variable | Value |
|---|---|---|
| `bg-gray-900` | `bg-[var(--surface)]` | #0e0e0e |
| `bg-gray-800` | `bg-[var(--surface-container)]` | #191919 |
| `bg-gray-700` | `bg-[var(--surface-container-high)]` | #1f1f1f |
| `text-gray-100` | `text-[var(--on-surface)]` | #ffffff |
| `text-gray-300` | `text-[var(--on-surface-variant)]` | #ababab |
| `text-gray-400` | `text-[var(--outline)]` | #757575 |
| `text-gray-500` | `text-[var(--outline)]` | #757575 |
| `border-gray-700` | `border-[var(--outline-variant)]` | #484848 |
| `border-gray-600` | `border-[var(--outline)]` | #757575 |
| `bg-blue-600` | `bg-[var(--primary)]` | #9cff93 |
| `bg-blue-400` / `text-blue-400` | `text-[var(--primary)]` | #9cff93 |
| `bg-green-600` / `bg-green-500` | `bg-[var(--success)]` | #9cff93 |
| `bg-red-600` / `bg-red-500` | `bg-[var(--error)]` | #ff7351 |
| `text-red-400` / `text-red-500` | `text-[var(--error)]` | #ff7351 |
| `bg-amber-500` | `bg-[var(--warning)]` | #fcaf00 |
| `bg-zinc-500` | `bg-[var(--outline)]` | #757575 |

# Files Affected

| File | Change |
|------|--------|
| `src/tailwind.css` | New file — copy from lx-desktop |
| `src/app.rs` | Add Tailwind CSS asset |
| `src/layout/shell.rs` | Replace color classes |
| `src/layout/bottom_nav.rs` | Replace color classes |
| `src/components/pulse_indicator.rs` | Replace color classes |
| `src/pages/status.rs` | Replace color classes |
| `src/pages/events.rs` | Replace color classes |
| `src/pages/approvals.rs` | Replace color classes |

All paths relative to `crates/lx-mobile/`.

# Task List

### Task 1: Add Tailwind CSS to mobile app

**Subject:** Copy the design system CSS and load it in the app

**Description:** Copy `crates/lx-desktop/src/tailwind.css` to `crates/lx-mobile/src/tailwind.css`. The file is identical — same CSS variables, same theme.

Edit `crates/lx-mobile/src/app.rs`. Add the CSS asset and stylesheet link. The current app.rs has:

```rust
use dioxus::prelude::*;
use crate::routes::Route;

#[component]
pub fn App() -> Element {
  rsx! {
      ErrorBoundary {
```

Change to:

```rust
use dioxus::prelude::*;
use crate::routes::Route;

static TAILWIND_CSS: Asset = asset!("/assets/tailwind.css", AssetOptions::css().with_static_head(true));

#[component]
pub fn App() -> Element {
  rsx! {
      document::Link {
        rel: "stylesheet",
        href: "https://fonts.googleapis.com/css2?family=Space+Grotesk:wght@300;400;500;600;700&family=Inter:wght@300;400;500;600;700&family=JetBrains+Mono:wght@400;500&family=Material+Symbols+Outlined:opsz,wght,FILL,GRAD@24,400,0,0&display=swap",
      }
      document::Stylesheet { href: TAILWIND_CSS }
      ErrorBoundary {
```

If the `asset!` macro path doesn't resolve for mobile (the CSS may need to be at a different path), check how lx-desktop loads it (`crates/lx-desktop/src/app.rs` line 5: `asset!("/assets/tailwind.css")`). The mobile app may need the CSS in `assets/tailwind.css` or the Dioxus mobile bundler may handle `src/tailwind.css` differently. Adapt the path based on what compiles.

**ActiveForm:** Adding Tailwind CSS to mobile app

---

### Task 2: Replace color classes in shell and bottom nav

**Subject:** Update layout components to use CSS variables

**Description:** Edit `crates/lx-mobile/src/layout/shell.rs`. Replace:
- `bg-gray-900` → `bg-[var(--surface)]`
- `text-gray-100` → `text-[var(--on-surface)]`
- `text-gray-400` → `text-[var(--outline)]`

The main div class becomes: `"min-h-screen bg-[var(--surface)] text-[var(--on-surface)] flex flex-col"`.
The label class becomes: `"text-xs text-[var(--outline)]"`.

Edit `crates/lx-mobile/src/layout/bottom_nav.rs`. Replace:
- `bg-gray-800` → `bg-[var(--surface-container)]`
- `border-gray-700` → `border-[var(--outline-variant)]`
- `text-gray-400` → `text-[var(--outline)]`
- `active:text-blue-400` → `active:text-[var(--primary)]`

**ActiveForm:** Replacing color classes in layout components

---

### Task 3: Replace color classes in pulse indicator

**Subject:** Update execution state colors to use CSS variables

**Description:** Edit `crates/lx-mobile/src/components/pulse_indicator.rs`. Replace the color strings in the match arms:
- `"bg-zinc-500"` → `"bg-[var(--outline)]"`
- `"bg-blue-500"` → `"bg-[var(--primary)]"`
- `"bg-amber-500"` → `"bg-[var(--warning)]"`
- `"bg-green-500"` → `"bg-[var(--success)]"`
- `"bg-red-500"` → `"bg-[var(--error)]"`
- `"text-gray-400"` → `"text-[var(--outline)]"`

**ActiveForm:** Replacing pulse indicator colors

---

### Task 4: Replace color classes in pages

**Subject:** Update all three page components to use CSS variables

**Description:** Edit `crates/lx-mobile/src/pages/status.rs`. Replace:
- `text-gray-300` → `text-[var(--on-surface-variant)]`
- `text-gray-500` → `text-[var(--outline)]`
- `text-red-400` → `text-[var(--error)]`

Edit `crates/lx-mobile/src/pages/events.rs`. Replace:
- `bg-blue-600` → `bg-[var(--primary)]`
- `text-white` (in filter button active state) → `text-[var(--on-primary)]`
- `bg-gray-700` → `bg-[var(--surface-container-high)]`
- `text-gray-300` → `text-[var(--on-surface-variant)]`
- `bg-gray-800` → `bg-[var(--surface-container)]`
- `text-gray-500` → `text-[var(--outline)]`
- `text-gray-400` → `text-[var(--outline)]`
- `text-gray-300` → `text-[var(--on-surface-variant)]`

Edit `crates/lx-mobile/src/pages/approvals.rs`. Replace:
- `bg-gray-800` → `bg-[var(--surface-container)]`
- `bg-green-600` → `bg-[var(--success)]`
- `bg-red-600` → `bg-[var(--error)]`
- `bg-gray-700` → `bg-[var(--surface-container-high)]`
- `bg-gray-600` (hover) → `bg-[var(--surface-bright)]`
- `border-gray-600` → `border-[var(--outline)]`
- `text-gray-100` → `text-[var(--on-surface)]`
- `text-gray-500` → `text-[var(--outline)]`
- `bg-blue-600` → `bg-[var(--primary)]`

Use `replace_all` where a class appears multiple times in the same file.

**ActiveForm:** Replacing page color classes

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

1. **Call `complete_task` after each task.**
2. **Call `next_task` to get the next task.**
3. **Do not add, skip, reorder, or combine tasks.**
4. **Tasks are implementation-only.**

---

## Task Loading Instructions

```
mcp__workflow__load_work_item({ path: "work_items/MOBILE_STYLING_ALIGNMENT.md" })
```
