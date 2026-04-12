# Unit 06: Custom Select component

## Goal
Replace the native `<select>` wrapper with a custom dropdown built on the existing `Popover` component, supporting custom-rendered option items, search/filter, and keyboard navigation.

## Preconditions
- No other units required first
- `crates/lx-desktop/src/components/ui/select.rs` exists (35 lines, native `<select>` wrapper)
- `crates/lx-desktop/src/components/ui/popover.rs` exists (44 lines, working Popover/PopoverTrigger/PopoverContent)
- `crates/lx-desktop/src/components/ui/mod.rs` exports `select` module and the `cn` utility

## Files to Modify
- `crates/lx-desktop/src/components/ui/select.rs` (rewrite)
- Any files that use `Select`/`SelectItem` must be updated for the new API (search the codebase)

## Current State

`select.rs` wraps a native `<select>` element with Tailwind classes. API:
- `Select` -- takes `class`, `value`, `disabled`, `onchange: EventHandler<FormEvent>`, `children`
- `SelectItem` -- takes `value`, `disabled`, `children`; renders `<option>`

`popover.rs` provides:
- `Popover` -- takes `open: Signal<bool>`, `children`; renders a `relative inline-block` wrapper
- `PopoverTrigger` -- takes `open: Signal<bool>`, `children`; toggles open on click
- `PopoverContent` -- takes `open: Signal<bool>`, `class`, `children`; renders absolutely positioned dropdown when open

## Steps

### Step 1: Define the SelectOption struct

At the top of `select.rs`, define the data structure for options:

```rust
use dioxus::prelude::*;

use super::cn;

#[derive(Clone, PartialEq)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
    pub disabled: bool,
}

impl SelectOption {
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self { value: value.into(), label: label.into(), disabled: false }
    }
}
```

### Step 2: Define the new Select component API

Replace the existing `Select` component with:

```rust
#[component]
pub fn Select(
    #[props(default)] class: String,
    value: String,
    options: Vec<SelectOption>,
    #[props(default)] placeholder: String,
    #[props(default)] disabled: bool,
    #[props(default)] searchable: bool,
    onchange: EventHandler<String>,
) -> Element {
```

Props:
- `class` -- additional classes on the trigger
- `value` -- currently selected value (controlled)
- `options` -- full list of `SelectOption`
- `placeholder` -- text when no value selected
- `disabled` -- prevents opening
- `searchable` -- shows a search input at top of dropdown
- `onchange` -- fires with the new value string when an option is selected

### Step 3: Internal state signals

Inside `Select`, create:

```rust
let mut open = use_signal(|| false);
let mut search_query = use_signal(String::new);
let mut focused_index = use_signal(|| 0usize);
```

### Step 4: Compute filtered options

```rust
let filtered: Vec<&SelectOption> = options
    .iter()
    .filter(|opt| {
        if !searchable || search_query.read().is_empty() {
            true
        } else {
            opt.label.to_lowercase().contains(&search_query.read().to_lowercase())
        }
    })
    .collect();
```

### Step 5: Find the display label for current value

```rust
let display_label = options.iter().find(|o| o.value == value).map(|o| o.label.as_str());
```

### Step 6: Render the trigger button

The trigger uses the `.select-trigger` utility class defined in `src/tailwind.css` under `@layer components`. The disabled and not-disabled variants are handled by `:disabled` / `:not(:disabled)` pseudo-class selectors on that utility, so the Rust code only needs to set the class name and the `disabled` attribute. Do not introduce a `const TRIGGER_CLASS: &str = "..."` in source — Tailwind class strings belong in CSS via `@apply`, not in Rust constants.

Render:

```rust
rsx! {
    div { "data-slot": "select", class: "relative inline-block",
        button {
            "data-slot": "select-trigger",
            class: cn(&["select-trigger", &class]),
            disabled,
            onclick: move |_| {
                if !disabled {
                    let v = open();
                    open.set(!v);
                    search_query.set(String::new());
                    focused_index.set(0);
                }
            },
            onkeydown: move |evt| { /* Step 8 */ },
            if let Some(label) = display_label {
                span { class: "text-[var(--on-surface)]", "{label}" }
            } else {
                span { class: "text-[var(--outline)]",
                    if placeholder.is_empty() { "Select..." } else { &placeholder }
                }
            }
            span { class: "material-symbols-outlined text-sm text-[var(--outline)]",
                if open() { "expand_less" } else { "expand_more" }
            }
        }
        // dropdown content (Step 7)
    }
}
```

### Step 7: Render the dropdown content

Conditionally render the dropdown when `open` is true. Place it inside the `data-slot="select"` div, after the button:

```rust
if open() {
    div {
        class: "fixed inset-0 z-40",
        onclick: move |_| open.set(false),
    }
    div {
        "data-slot": "select-content",
        class: "absolute top-full left-0 mt-1 z-50 min-w-full max-h-64 overflow-y-auto rounded-md border border-[var(--outline-variant)] bg-[var(--surface-container)] shadow-md py-1",

        if searchable {
            div { class: "px-2 py-1.5 border-b border-[var(--outline-variant)]/30",
                input {
                    class: "w-full bg-transparent text-sm text-[var(--on-surface)] outline-none placeholder:text-[var(--outline)]/40",
                    placeholder: "Search...",
                    value: "{search_query}",
                    oninput: move |e| {
                        search_query.set(e.value());
                        focused_index.set(0);
                    },
                    autofocus: true,
                    onkeydown: move |evt| { /* Step 8 */ },
                }
            }
        }

        for (idx, opt) in filtered.iter().enumerate() {
            {render_select_item(opt, idx, &value, focused_index, open, search_query, &onchange)}
        }

        if filtered.is_empty() {
            div { class: "px-3 py-2 text-sm text-[var(--outline)]", "No results" }
        }
    }
}
```

### Step 8: Render individual option items

Create a helper function:

```rust
fn render_select_item(
    opt: &SelectOption,
    idx: usize,
    current_value: &str,
    mut focused_index: Signal<usize>,
    mut open: Signal<bool>,
    mut search_query: Signal<String>,
    onchange: &EventHandler<String>,
) -> Element {
    let is_selected = opt.value == current_value;
    let is_focused = *focused_index.read() == idx;
    let val = opt.value.clone();

    let item_class = cn(&[
        "flex items-center gap-2 px-3 py-1.5 text-sm cursor-pointer transition-colors",
        if is_focused { "bg-[var(--surface-container-highest)]" } else { "" },
        if is_selected { "text-[var(--primary)] font-medium" } else { "text-[var(--on-surface)]" },
        if opt.disabled { "opacity-50 pointer-events-none" } else { "" },
    ]);

    rsx! {
        div {
            "data-slot": "select-item",
            class: "{item_class}",
            onmouseenter: move |_| focused_index.set(idx),
            onclick: move |_| {
                if !opt.disabled {
                    onchange.call(val.clone());
                    open.set(false);
                    search_query.set(String::new());
                }
            },
            if is_selected {
                span { class: "material-symbols-outlined text-sm text-[var(--primary)]", "check" }
            } else {
                span { class: "w-5" }
            }
            span { "{opt.label}" }
        }
    }
}
```

### Step 9: Implement keyboard navigation

The `onkeydown` handler is shared between the trigger button and the search input. Extract it to a closure defined inside `Select`:

```rust
let key_handler = {
    let filtered_len = filtered.len();
    move |evt: KeyboardEvent| {
        let key = evt.key();
        match key {
            Key::ArrowDown => {
                evt.prevent_default();
                if !open() {
                    open.set(true);
                    focused_index.set(0);
                } else {
                    let cur = *focused_index.read();
                    if cur + 1 < filtered_len {
                        focused_index.set(cur + 1);
                    }
                }
            }
            Key::ArrowUp => {
                evt.prevent_default();
                if open() {
                    let cur = *focused_index.read();
                    if cur > 0 {
                        focused_index.set(cur - 1);
                    }
                }
            }
            Key::Enter => {
                evt.prevent_default();
                if open() {
                    let idx = *focused_index.read();
                    if let Some(opt) = filtered.get(idx) {
                        if !opt.disabled {
                            onchange.call(opt.value.clone());
                            open.set(false);
                            search_query.set(String::new());
                        }
                    }
                } else {
                    open.set(true);
                }
            }
            Key::Escape => {
                evt.prevent_default();
                evt.stop_propagation();
                open.set(false);
                search_query.set(String::new());
            }
            _ => {}
        }
    }
};
```

Wire this into both the trigger button's `onkeydown` and the search input's `onkeydown` from Steps 6 and 7. Use two separate closures with the same logic pattern — one for the trigger `onkeydown` and one for the search input `onkeydown`. Both closures capture the same signals and perform identical logic.

### Step 10: Reset focused_index when search query changes

In the `oninput` handler of the search input (Step 7), `focused_index.set(0)` is already included. This ensures arrow-key navigation starts from the top after filtering.

### Step 11: Remove the old SelectItem component

Delete the `SelectItem` component entirely. It is no longer needed since options are passed as `Vec<SelectOption>` data, not as children.

### Step 12: Update all call sites

Search for all usages of `Select` and `SelectItem` in the codebase. Every call site must change from:

```rust
Select { value: "...", onchange: ...,
    SelectItem { value: "a", "Label A" }
    SelectItem { value: "b", "Label B" }
}
```

To:

```rust
Select {
    value: "...",
    options: vec![
        SelectOption::new("a", "Label A"),
        SelectOption::new("b", "Label B"),
    ],
    onchange: move |val: String| { /* ... */ },
}
```

Files that use `Select`/`SelectItem` or raw `<select>` elements (exhaustive list):
- `pages/agents/config_form.rs` -- adapter type raw `<select>` (lines 35-44): convert to `Select` with `ADAPTER_LABELS` as `Vec<SelectOption>`
- `pages/issues/new_issue.rs` -- status/priority/assignee raw `<select>` elements (lines 49-93): convert to `Select` with `Vec<SelectOption>`
- `pages/routines/schedule_editor.rs` -- preset, hour, minute, day selectors: convert raw `<select>` to `Select`
- `components/onboarding/step_agent.rs` -- adapter and role raw `<select>` elements (lines 39-59): convert to `Select`
- `pages/activity.rs` -- type filter raw `<select>` (lines 16-24): convert to `Select`

### Step 13: Update mod.rs if needed

The module export in `crates/lx-desktop/src/components/ui/mod.rs` already has `pub mod select;`. No change needed.

## Verification
1. Run `just diagnose` -- must compile with no errors or warnings
2. Launch the app, find any page with a select dropdown (e.g., agent config, new issue dialog)
3. Click the trigger -- dropdown appears below the trigger, aligned left
4. Click outside the dropdown -- it closes
5. Press Escape -- dropdown closes
6. Press ArrowDown with dropdown closed -- dropdown opens with first item focused
7. Press ArrowDown/ArrowUp -- focus highlight moves between items
8. Press Enter on a focused item -- item is selected, dropdown closes, value updates
9. If `searchable: true`: type in the search box -- options filter in real time; "No results" shows when nothing matches
10. Selected item shows a checkmark icon and primary-colored text
11. Disabled items show 50% opacity and cannot be clicked
12. `select.rs` stays under 300 lines. If it would exceed, split `render_select_item` into a sibling file `select_item.rs` and re-export from `select.rs`
