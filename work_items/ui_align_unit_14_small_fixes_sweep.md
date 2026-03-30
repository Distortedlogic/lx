# Unit 14: Small Fixes Sweep (Audit Fixes #15-19)

## Goal

Five small fixes from the UI alignment audit: FilterBar primitive usage, ScrollToBottom wiring, transcript entry animations, GoalTree StatusBadge, and icon sizing convention.

## Preconditions

- Unit 12 (toast animations) should be complete first so that `tailwind.css` has the toast keyframes already appended (this unit adds more keyframes after them)
- Unit 13 (dialog animations) should be complete first so that `tailwind.css` has the dialog keyframes already appended

## Files to Modify

- `crates/lx-desktop/src/components/filter_bar.rs`
- `crates/lx-desktop/src/pages/agents/transcript.rs`
- `crates/lx-desktop/src/pages/goals/tree.rs`
- `crates/lx-desktop/src/tailwind.css`
- Multiple files for icon sizing (listed in Step 5)

## Steps

### Step 1: FilterBar -- replace raw "Clear all" button with Button component

File: `crates/lx-desktop/src/components/filter_bar.rs`

The FilterBar already uses `Badge` with `BadgeVariant::Secondary` for filter chips (line 21-33). The only remaining raw element is the "Clear all" button (lines 36-39).

1a. Add the Button import. Change the existing import block at the top:

Old:
```rust
use super::ui::badge::{Badge, BadgeVariant};
```

New:
```rust
use super::ui::badge::{Badge, BadgeVariant};
use super::ui::button::{Button, ButtonVariant, ButtonSize};
```

1b. Replace the raw "Clear all" button (lines 36-39):

Old:
```rust
      button {
        class: "text-xs text-[var(--on-surface-variant)] hover:text-[var(--on-surface)] px-2 py-1 transition-colors",
        onclick: move |_| on_clear.call(()),
        "Clear all"
      }
```

New:
```rust
      Button {
        variant: ButtonVariant::Ghost,
        size: ButtonSize::Xs,
        onclick: move |_| on_clear.call(()),
        "Clear all"
      }
```

Note: The `Button` component does not have an `onclick` prop by default -- it renders a `<button>` element. Check if Dioxus event spreading works here. If not, wrap the `Button` in an outer element or add the `onclick` on the Button. In Dioxus, event handlers on custom components are spread to the root element if the component supports it. Since `Button` renders a `<button>` with no `onclick` prop, you may need to instead keep it as a raw button but use the button_variant_class helper:

Alternative 1b (if Button component does not accept onclick):
```rust
      button {
        class: button_variant_class(ButtonVariant::Ghost, ButtonSize::Xs),
        onclick: move |_| on_clear.call(()),
        "Clear all"
      }
```

This requires importing `button_variant_class` instead of `Button`:
```rust
use super::ui::button::{button_variant_class, ButtonVariant, ButtonSize};
```

### Step 2: Wire ScrollToBottom into TranscriptView

File: `crates/lx-desktop/src/pages/agents/transcript.rs`

2a. Add the import at the top of the file:

```rust
use crate::components::scroll_to_bottom::ScrollToBottom;
```

2b. Wrap the transcript entries container (lines 43-49) with `ScrollToBottom`. Replace:

Old:
```rust
  rsx! {
    div { class: "space-y-2",
      for entry in entries.iter() {
        TranscriptBlockView { block: entry.clone() }
      }
    }
  }
```

New:
```rust
  rsx! {
    ScrollToBottom { class: "max-h-[60vh]".to_string(),
      div { class: "space-y-2",
        for entry in entries.iter() {
          TranscriptBlockView { block: entry.clone() }
        }
      }
    }
  }
```

The `ScrollToBottom` component accepts an optional `class` prop. `max-h-[60vh]` constrains the scrollable area so `overflow-y-auto` (applied internally by ScrollToBottom) activates.

### Step 3: Add CSS entry animation for transcript blocks

File: `crates/lx-desktop/src/tailwind.css`

3a. Append after the last animation block:

```css
@keyframes transcript-block-enter {
  from {
    opacity: 0;
    transform: translateY(4px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.animate-transcript-enter {
  animation: transcript-block-enter 300ms ease-out both;
}
```

3b. Apply the class in `transcript.rs`. In each branch of the `TranscriptBlockView` match, add `animate-transcript-enter` to the outermost div's class string.

File: `crates/lx-desktop/src/pages/agents/transcript.rs`

For the `Message` branch (line 59), change:
```
"flex gap-3 p-3 rounded-lg {bg}"
```
to:
```
"flex gap-3 p-3 rounded-lg {bg} animate-transcript-enter"
```

For the `Thinking` branch (line 71), change:
```
"flex gap-3 p-3 rounded-lg bg-[var(--warning)]/5 border border-[var(--warning)]/10"
```
to:
```
"flex gap-3 p-3 rounded-lg bg-[var(--warning)]/5 border border-[var(--warning)]/10 animate-transcript-enter"
```

For the `ToolUse` branch (line 84), change:
```
"border {border} rounded-lg p-3 space-y-2"
```
to:
```
"border {border} rounded-lg p-3 space-y-2 animate-transcript-enter"
```

For the `Event` branch (line 114), change:
```
"flex items-center gap-2 py-1"
```
to:
```
"flex items-center gap-2 py-1 animate-transcript-enter"
```

### Step 4: GoalTree -- replace inline status_color() with StatusBadge

File: `crates/lx-desktop/src/pages/goals/tree.rs`

4a. Add the import:

```rust
use crate::components::status_badge::StatusBadge;
```

4b. Remove the `status_color` function entirely (lines 6-14):

Delete:
```rust
fn status_color(status: &str) -> &'static str {
  match status {
    "in_progress" => "text-[var(--primary)]",
    "completed" => "text-[var(--success)]",
    "cancelled" => "text-[var(--error)]",
    "planned" => "text-[var(--warning)]",
    _ => "text-[var(--outline)]",
  }
}
```

4c. In the `GoalNode` component, replace the inline status text span (line 69) that uses `status_color`:

Old:
```rust
      span { class: "text-[10px] uppercase font-semibold tracking-wider shrink-0 {status_color(&goal.status)}",
        "{goal.status}"
      }
```

New:
```rust
      StatusBadge { status: goal.status.clone() }
```

The `StatusBadge` component (from `crates/lx-desktop/src/components/status_badge.rs`) renders an inline-flex span with `rounded-full px-2.5 py-0.5 text-xs font-medium` and calls `status_badge_class()` from `status_colors.rs` which maps statuses like `in_progress`, `completed`, `cancelled`, `planned` to colored bg/text classes. This replaces the hand-rolled `status_color()` with the shared component used elsewhere.

### Step 5: Icon sizing convention sweep

Convention to establish:
- `text-xs` -- tiny inline icons (close buttons inside badges, very small indicators)
- `text-sm` -- default inline icons (toolbar buttons, list item icons, close buttons)
- `text-base` -- standard icons (sidebar nav, metric cards, settings items)
- `text-lg` -- header/prominent icons (command palette, dialog close, page headers)
- `text-xl` -- large featured icons (agent detail hero, icon pickers)
- `text-2xl` and above -- empty state / hero illustrations only

The following files have icon sizes that deviate from this convention and need correction:

| File | Line | Current | Correct | Reason |
|------|------|---------|---------|--------|
| `src/pages/company_import.rs` | 27 | no size class (bare `material-symbols-outlined`) | `text-base` | page section header icon needs explicit size |
| `src/pages/company_import.rs` | 53 | `text-2xl` | `text-lg` | card icon, not an empty state hero |
| `src/pages/company_export.rs` | 20 | no size class | `text-base` | page section header icon needs explicit size |
| `src/pages/agents/detail.rs` | 168 | `text-xl` | `text-lg` | agent icon in header tabs, not a hero |
| `src/pages/agents/icon_picker.rs` | 48 | `text-xl` | `text-lg` | selected icon display, consistent with header size |
| `src/pages/settings/mod.rs` | 58 | no explicit size (just `text-black font-bold`) | `text-base` | settings icon in header |
| `src/pages/settings/mod.rs` | 88 | `text-lg` | `text-base` | settings section icon, not a header |

For each file listed above, find the `material-symbols-outlined` span and update the size class. The change is purely the text size class in the `class` string.

Example for `company_import.rs` line 27:

Old:
```rust
span { class: "material-symbols-outlined text-[var(--outline)]", "upload" }
```

New:
```rust
span { class: "material-symbols-outlined text-base text-[var(--outline)]", "upload" }
```

Do NOT change any icon sizes that already follow the convention. The majority of icons in the codebase already use `text-sm` or `text-base` correctly. Only change the specific lines listed above.

## Verification

1. Run `just diagnose` -- must compile with zero warnings
2. FilterBar: open any page that shows filters and verify the "Clear all" button renders with the ghost button styling (subtle hover background, no visible border in default state)
3. TranscriptView: open an agent detail page with transcript data and verify:
   - The transcript area scrolls and auto-scrolls to bottom on new entries
   - Each transcript block fades in with a slight upward slide (300ms animation)
4. GoalTree: open the goals page and verify status badges show with colored pill styling (rounded background + text) instead of plain colored text
5. Icon sizing: spot-check the changed files -- icons should look proportionally correct in their context
