# UI Alignment Unit 05: Component Reuse Fixes

## Goal

Three fixes:
- A) FilterBar: replace inline `span` badges with the `Badge` component
- B) Loading states: replace "Loading..." text with `PageSkeleton` component in shell.rs and app.rs SuspenseBoundary fallbacks
- C) Status dots: add `STATUS_DOT_RUNNING` with `animate-pulse` to `styles.rs`, use it in the agent list

---

## Fix A: FilterBar Badge Component

**File:** `/home/entropybender/repos/lx/crates/lx-desktop/src/components/filter_bar.rs`

Replace the inline `span`-based badge markup with the `Badge` component from `components/ui/badge.rs`. The Badge component uses `BadgeVariant::Secondary` for filter chips.

**old_string:**
```
use dioxus::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct FilterValue {
  pub key: String,
  pub label: String,
  pub value: String,
}

#[component]
pub fn FilterBar(filters: Vec<FilterValue>, on_remove: EventHandler<String>, on_clear: EventHandler<()>) -> Element {
  if filters.is_empty() {
    return rsx! {};
  }

  rsx! {
    div { class: "flex items-center gap-2 flex-wrap",
      for filter in filters.iter() {
        span { class: "inline-flex items-center gap-1 rounded-full bg-gray-700 px-2.5 py-0.5 text-xs pr-1",
          span { class: "text-gray-400", "{filter.label}:" }
          span { "{filter.value}" }
          button {
            class: "ml-1 rounded-full hover:bg-gray-600 p-0.5",
            onclick: {
                let key = filter.key.clone();
                move |_| on_remove.call(key.clone())
            },
            span { class: "material-symbols-outlined text-xs", "close" }
          }
        }
      }
      button {
        class: "text-xs text-gray-400 hover:text-white px-2 py-1 transition-colors",
        onclick: move |_| on_clear.call(()),
        "Clear all"
      }
    }
  }
}
```

**new_string:**
```
use dioxus::prelude::*;

use super::ui::badge::{Badge, BadgeVariant};

#[derive(Clone, Debug, PartialEq)]
pub struct FilterValue {
  pub key: String,
  pub label: String,
  pub value: String,
}

#[component]
pub fn FilterBar(filters: Vec<FilterValue>, on_remove: EventHandler<String>, on_clear: EventHandler<()>) -> Element {
  if filters.is_empty() {
    return rsx! {};
  }

  rsx! {
    div { class: "flex items-center gap-2 flex-wrap",
      for filter in filters.iter() {
        Badge { variant: BadgeVariant::Secondary, class: "pr-1 gap-1".to_string(),
          span { class: "text-[var(--on-surface-variant)]", "{filter.label}:" }
          span { "{filter.value}" }
          button {
            class: "ml-1 rounded-full hover:bg-[var(--surface-bright)] p-0.5",
            onclick: {
                let key = filter.key.clone();
                move |_| on_remove.call(key.clone())
            },
            span { class: "material-symbols-outlined text-xs", "close" }
          }
        }
      }
      button {
        class: "text-xs text-[var(--on-surface-variant)] hover:text-[var(--on-surface)] px-2 py-1 transition-colors",
        onclick: move |_| on_clear.call(()),
        "Clear all"
      }
    }
  }
}
```

---

## Fix B: Loading States with PageSkeleton

### B1: shell.rs SuspenseBoundary

**File:** `/home/entropybender/repos/lx/crates/lx-desktop/src/layout/shell.rs`

**old_string:**
```
                  SuspenseBoundary {
                    fallback: |_| rsx! {
                      div { class: "flex items-center justify-center h-full text-[var(--outline)]", "Loading..." }
                    },
                    LiveUpdatesProvider {}
                  }
```

**new_string:**
```
                  SuspenseBoundary {
                    fallback: |_| rsx! {
                      div { class: "p-6",
                        crate::components::page_skeleton::PageSkeleton {}
                      }
                    },
                    LiveUpdatesProvider {}
                  }
```

### B2: app.rs SuspenseBoundary

**File:** `/home/entropybender/repos/lx/crates/lx-desktop/src/app.rs`

**old_string:**
```
      SuspenseBoundary {
        fallback: |_| rsx! {
          div { class: "flex items-center justify-center h-screen text-[var(--outline)]", "Loading..." }
        },
        Router::<Route> {}
      }
```

**new_string:**
```
      SuspenseBoundary {
        fallback: |_| rsx! {
          div { class: "flex items-center justify-center h-screen p-6",
            crate::components::page_skeleton::PageSkeleton {}
          }
        },
        Router::<Route> {}
      }
```

---

## Fix C: STATUS_DOT_RUNNING with animate-pulse

### C2: Use STATUS_DOT_RUNNING in agent list

**File:** `/home/entropybender/repos/lx/crates/lx-desktop/src/pages/agents/list.rs`

First, update the import to include the new constant:

**old_string:**
```
use crate::styles::{BTN_OUTLINE_SM, FLEX_BETWEEN, TAB_ACTIVE, TAB_INACTIVE};
```

**new_string:**
```
use crate::styles::{BTN_OUTLINE_SM, FLEX_BETWEEN, STATUS_DOT_RUNNING, TAB_ACTIVE, TAB_INACTIVE};
```

Then find the `StatusBadge` component and update it to use the `STATUS_DOT_RUNNING` style. The agent list already has a `status_dot_class` function imported from `types` module. Verify that module's function. For the `StatusBadge` component in this file, the color classes also need updating:

**old_string:**
```
#[component]
pub fn StatusBadge(status: String) -> Element {
  let (bg, text) = match status.as_str() {
    "active" | "running" | "idle" => ("bg-green-500/10 text-green-600", "Active"),
    "paused" => ("bg-yellow-500/10 text-yellow-600", "Paused"),
    "error" => ("bg-red-500/10 text-red-600", "Error"),
    "terminated" => ("bg-neutral-500/10 text-neutral-500", "Terminated"),
    "pending_approval" => ("bg-amber-500/10 text-amber-600", "Pending"),
    other => ("bg-neutral-500/10 text-neutral-400", other),
  };
  let label = text.to_string();
  rsx! {
    span { class: "inline-flex items-center rounded-full px-2 py-0.5 text-[11px] font-medium {bg}",
      "{label}"
    }
  }
}
```

**new_string:**
```
#[component]
pub fn StatusBadge(status: String) -> Element {
  let (bg, text) = match status.as_str() {
    "running" => ("bg-[var(--tertiary)]/10 text-[var(--tertiary)]", "Running"),
    "active" | "idle" => ("bg-[var(--success)]/10 text-[var(--success)]", "Active"),
    "paused" => ("bg-[var(--warning)]/10 text-[var(--warning)]", "Paused"),
    "error" => ("bg-[var(--error)]/10 text-[var(--error)]", "Error"),
    "terminated" => ("bg-[var(--outline)]/10 text-[var(--outline)]", "Terminated"),
    "pending_approval" => ("bg-[var(--warning)]/10 text-[var(--warning)]", "Pending"),
    other => ("bg-[var(--outline)]/10 text-[var(--outline)]", other),
  };
  let label = text.to_string();
  rsx! {
    span { class: "inline-flex items-center rounded-full px-2 py-0.5 text-[11px] font-medium {bg}",
      "{label}"
    }
  }
}
```

### C3: Update styles.rs dot constants to use CSS vars and add STATUS_DOT_RUNNING

**File:** `/home/entropybender/repos/lx/crates/lx-desktop/src/styles.rs`

**old_string:**
```
pub const STATUS_DOT_ACTIVE: &str = "inline-flex h-2.5 w-2.5 rounded-full bg-green-500";
pub const STATUS_DOT_PAUSED: &str = "inline-flex h-2.5 w-2.5 rounded-full bg-yellow-500";
pub const STATUS_DOT_ERROR: &str = "inline-flex h-2.5 w-2.5 rounded-full bg-red-500";
pub const STATUS_DOT_DEFAULT: &str = "inline-flex h-2.5 w-2.5 rounded-full bg-neutral-400";
```

**new_string:**
```
pub const STATUS_DOT_ACTIVE: &str = "inline-flex h-2.5 w-2.5 rounded-full bg-[var(--success)]";
pub const STATUS_DOT_PAUSED: &str = "inline-flex h-2.5 w-2.5 rounded-full bg-[var(--warning)]";
pub const STATUS_DOT_ERROR: &str = "inline-flex h-2.5 w-2.5 rounded-full bg-[var(--error)]";
pub const STATUS_DOT_RUNNING: &str = "inline-flex h-2.5 w-2.5 rounded-full bg-[var(--tertiary)] animate-pulse";
pub const STATUS_DOT_DEFAULT: &str = "inline-flex h-2.5 w-2.5 rounded-full bg-[var(--outline)]";
```

---

## Verification

1. FilterBar renders `Badge` components with `BadgeVariant::Secondary` instead of raw `span` elements
2. SuspenseBoundary fallbacks in shell.rs and app.rs show skeleton loading animation instead of "Loading..." text
3. `STATUS_DOT_RUNNING` constant exists in `styles.rs` with `animate-pulse` class
4. All status dot constants use CSS variables instead of hardcoded Tailwind colors
5. `StatusBadge` component uses CSS variables for all status colors
