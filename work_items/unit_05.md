# Unit 5: Shared Components (Part 2 — Editing & Display)

## Scope

Create the second batch of shared components for lx-desktop: filter_bar, inline_editor, copy_text, page_skeleton, page_tab_bar, markdown_body, toast_viewport, metric_card, comment_thread, inline_entity_selector.

## Preconditions

- **Unit 3 is complete:** `contexts/toast.rs` exists with the canonical `ToastState` (including `ToastTone`, `ToastAction`, `ToastInput`, `ToastItem`). Toast context is already provided in Shell. Do NOT recreate `contexts/toast.rs`.
- Unit 4 is complete: the `src/components/` directory exists with `mod.rs` and the Part 1 components (`status_colors`, `status_icon`, `status_badge`, `priority_icon`, `identity`, `empty_state`, `entity_row`).
- `src/lib.rs` contains `pub mod components;`.
- Dioxus 0.7.3 is the target framework.
- The `status_colors` module is available at `crate::components::status_colors`.

## Paperclip Source References

| lx-desktop target file | Paperclip reference file |
|---|---|
| `components/filter_bar.rs` | `reference/paperclip/ui/src/components/FilterBar.tsx` |
| `components/inline_editor.rs` | `reference/paperclip/ui/src/components/InlineEditor.tsx` |
| `components/copy_text.rs` | `reference/paperclip/ui/src/components/CopyText.tsx` |
| `components/page_skeleton.rs` | `reference/paperclip/ui/src/components/PageSkeleton.tsx` |
| `components/page_tab_bar.rs` | `reference/paperclip/ui/src/components/PageTabBar.tsx` |
| `components/markdown_body.rs` | `reference/paperclip/ui/src/components/MarkdownBody.tsx` |
| `components/toast_viewport.rs` | `reference/paperclip/ui/src/components/ToastViewport.tsx` |
| `components/metric_card.rs` | `reference/paperclip/ui/src/components/MetricCard.tsx` |
| `components/comment_thread.rs` | `reference/paperclip/ui/src/components/CommentThread.tsx` |
| `components/inline_entity_selector.rs` | `reference/paperclip/ui/src/components/InlineEntitySelector.tsx` |

## Steps

### Step 1: Create the FilterBar component

**Create file:** `/home/entropybender/repos/lx/crates/lx-desktop/src/components/filter_bar.rs`

Reference: `reference/paperclip/ui/src/components/FilterBar.tsx`

Define types and component:

```rust
#[derive(Clone, Debug, PartialEq)]
pub struct FilterValue {
    pub key: String,
    pub label: String,
    pub value: String,
}

#[component]
pub fn FilterBar(
    filters: Vec<FilterValue>,
    on_remove: EventHandler<String>,
    on_clear: EventHandler<()>,
) -> Element
```

Behavior:
- If `filters` is empty, return `None` (render nothing).
- Render a `div` with class `"flex items-center gap-2 flex-wrap"`.
- For each filter, render a badge:
  ```
  span (class: "inline-flex items-center gap-1 rounded-full bg-gray-700 px-2.5 py-0.5 text-xs pr-1") {
    span (class: "text-gray-400") { "{filter.label}:" }
    span { "{filter.value}" }
    button (class: "ml-1 rounded-full hover:bg-gray-600 p-0.5",
            onclick: on_remove.call(filter.key.clone())) {
      span (class: "material-symbols-outlined text-xs") { "close" }
    }
  }
  ```
- After all filters, render a "Clear all" button:
  ```
  button (class: "text-xs text-gray-400 hover:text-white px-2 py-1 transition-colors",
          onclick: on_clear.call(())) {
    "Clear all"
  }
  ```

### Step 2: Create the CopyText component

**Create file:** `/home/entropybender/repos/lx/crates/lx-desktop/src/components/copy_text.rs`

Reference: `reference/paperclip/ui/src/components/CopyText.tsx`

```rust
#[component]
pub fn CopyText(
    text: String,
    #[props(optional)] children: Option<Element>,
    #[props(optional)] class: Option<String>,
) -> Element
```

Behavior:
- Use a `use_signal(|| false)` for `copied` state.
- On click, use `document::eval` to copy to clipboard via JS: `navigator.clipboard.writeText("{text}")`. Set `copied` to `true`, spawn a future that sets it back to `false` after 1500ms.
- Render:
  ```
  span (class: "relative inline-flex") {
    button (class: "cursor-copy hover:text-white transition-colors {class}",
            onclick: handle_click) {
      if children.is_some() { {children} } else { "{text}" }
    }
    span (class: "pointer-events-none absolute left-1/2 -translate-x-1/2 bottom-full mb-1.5 rounded-md bg-white text-black px-2 py-1 text-xs whitespace-nowrap transition-opacity duration-300 {opacity_class}") {
      if copied { "Copied!" } else { "" }
    }
  }
  ```
  Where `opacity_class` is `"opacity-100"` when `copied` is true, `"opacity-0"` otherwise.

### Step 3: Create the PageSkeleton component

**Create file:** `/home/entropybender/repos/lx/crates/lx-desktop/src/components/page_skeleton.rs`

Reference: `reference/paperclip/ui/src/components/PageSkeleton.tsx`

```rust
#[component]
pub fn PageSkeleton(
    #[props(default = "list".to_string())] variant: String,
) -> Element
```

Define a helper:
```rust
#[component]
fn Skeleton(class: String) -> Element {
    rsx! { div { class: "animate-pulse bg-gray-700/50 rounded {class}" } }
}
```

Behavior based on `variant`:
- `"dashboard"`: render a space-y-6 div with:
  - One `Skeleton { class: "h-32 w-full" }`
  - A 4-column grid of `Skeleton { class: "h-24 w-full" }` (4 items)
  - A 4-column grid of `Skeleton { class: "h-44 w-full" }` (4 items)
  - A 2-column grid of `Skeleton { class: "h-72 w-full" }` (2 items)
- `"detail"`: render a space-y-6 div with:
  - A `Skeleton { class: "h-3 w-64" }`, row of 3 small skeletons, `Skeleton { class: "h-4 w-40" }`
  - A `Skeleton { class: "h-10 w-full" }` + `Skeleton { class: "h-32 w-full" }`
  - Row of 3 tab-like skeletons + 2 content skeletons
- `"list"` (default): render a space-y-4 div with:
  - A header row: `Skeleton { class: "h-9 w-44" }` + 2 action skeletons
  - 7 rows of `Skeleton { class: "h-11 w-full" }`

### Step 4: Create the PageTabBar component

**Create file:** `/home/entropybender/repos/lx/crates/lx-desktop/src/components/page_tab_bar.rs`

Reference: `reference/paperclip/ui/src/components/PageTabBar.tsx`

```rust
#[derive(Clone, Debug, PartialEq)]
pub struct PageTabItem {
    pub value: String,
    pub label: String,
}

#[component]
pub fn PageTabBar(
    items: Vec<PageTabItem>,
    #[props(optional)] value: Option<String>,
    #[props(optional)] on_value_change: Option<EventHandler<String>>,
) -> Element
```

Behavior:
- Render a `div` with class `"flex border-b border-gray-700/50"`.
- For each item, render a button:
  ```
  button (class: "px-4 py-2 text-sm font-medium transition-colors border-b-2 -mb-px {active_class}",
          onclick: on_value_change.call(item.value.clone())) {
    "{item.label}"
  }
  ```
  Where `active_class`:
  - If `item.value == current_value`: `"border-white text-white"`
  - Else: `"border-transparent text-gray-400 hover:text-white hover:border-gray-500"`

### Step 5: Create the MarkdownBody component

**Create file:** `/home/entropybender/repos/lx/crates/lx-desktop/src/components/markdown_body.rs`

Reference: `reference/paperclip/ui/src/components/MarkdownBody.tsx`

For the Dioxus port, use `pulldown-cmark` to render markdown to HTML, then use `dangerous_inner_html`.

First, add `pulldown-cmark` to the dependencies. **Edit file:** `/home/entropybender/repos/lx/crates/lx-desktop/Cargo.toml` — add under `[dependencies]`:
```toml
pulldown-cmark = { version = "0.12", default-features = false, features = ["html"] }
```

```rust
use pulldown_cmark::{Options, Parser, html};

#[component]
pub fn MarkdownBody(
    children: String,
    #[props(optional)] class: Option<String>,
) -> Element
```

Behavior:
- Parse the `children` string with `pulldown_cmark::Parser::new_ext(&children, Options::all())`.
- Render to HTML string via `pulldown_cmark::html::push_html(&mut html_output, parser)`.
- Render:
  ```
  div (class: "prose prose-sm prose-invert max-w-none break-words overflow-hidden {class}",
       dangerous_inner_html: "{html_output}") {}
  ```

### Step 6: Create the MetricCard component

**Create file:** `/home/entropybender/repos/lx/crates/lx-desktop/src/components/metric_card.rs`

Reference: `reference/paperclip/ui/src/components/MetricCard.tsx`

```rust
#[component]
pub fn MetricCard(
    icon: String,
    value: String,
    label: String,
    #[props(optional)] description: Option<String>,
    #[props(optional)] to: Option<String>,
    #[props(optional)] onclick: Option<EventHandler<()>>,
) -> Element
```

The `icon` is a Material Symbols icon name string.

Behavior:
- Determine if clickable: `to.is_some() || onclick.is_some()`.
- Build inner div:
  ```
  div (class: "h-full px-5 py-5 rounded-lg transition-colors {hover_class}") {
    div (class: "flex items-start justify-between gap-3") {
      div (class: "flex-1 min-w-0") {
        p (class: "text-3xl font-semibold tracking-tight tabular-nums") { "{value}" }
        p (class: "text-sm font-medium text-gray-400 mt-1") { "{label}" }
        if description.is_some() {
          div (class: "text-xs text-gray-500 mt-1.5") { "{description}" }
        }
      }
      span (class: "material-symbols-outlined text-base text-gray-500 shrink-0 mt-1.5") { "{icon}" }
    }
  }
  ```
  Where `hover_class` is `"hover:bg-white/5 cursor-pointer"` if clickable, else `""`.
- If `to` is `Some(href)`, wrap in `Link { to: "{href}" }`.
- If `onclick` is `Some`, wrap in a `div` with the onclick handler.
- Otherwise render the inner div directly.

### Step 7: Create the ToastViewport component

Unit 3 owns `contexts/toast.rs` with the canonical `ToastState`. Do NOT recreate it. The `toast_viewport.rs` component uses `crate::contexts::toast::ToastState` from Unit 3.

**Create file:** `/home/entropybender/repos/lx/crates/lx-desktop/src/components/toast_viewport.rs`

Reference: `reference/paperclip/ui/src/components/ToastViewport.tsx`

```rust
#[component]
pub fn ToastViewport() -> Element
```

Behavior:
- Use `use_context::<crate::contexts::toast::ToastState>()`.
- If `toasts` is empty, return `None`.
- Render:
  ```
  aside (class: "pointer-events-none fixed bottom-3 left-3 z-[120] w-full max-w-sm px-1",
         "aria-live": "polite") {
    ol (class: "flex w-full flex-col-reverse gap-2") {
      for toast in toasts.read().iter() {
        {render_toast(toast, dismiss_handler)}
      }
    }
  }
  ```

Define tone class mappings (inline in the file):
- `Info` -> `"border-sky-500/25 bg-sky-950/60 text-sky-100"`
- `Success` -> `"border-emerald-500/25 bg-emerald-950/60 text-emerald-100"`
- `Warn` -> `"border-amber-500/25 bg-amber-950/60 text-amber-100"`
- `Error` -> `"border-red-500/30 bg-red-950/60 text-red-100"`

Tone dot classes:
- `Info` -> `"bg-sky-400"`
- `Success` -> `"bg-emerald-400"`
- `Warn` -> `"bg-amber-400"`
- `Error` -> `"bg-red-400"`

Each toast renders:
```
li (class: "pointer-events-auto rounded-sm border shadow-lg backdrop-blur-xl {tone_class}") {
  div (class: "flex items-start gap-3 px-3 py-2.5") {
    span (class: "mt-1 h-2 w-2 shrink-0 rounded-full {dot_class}") {}
    div (class: "min-w-0 flex-1") {
      p (class: "text-sm font-semibold leading-5") { "{toast.title}" }
      if toast.body.is_some() {
        p (class: "mt-1 text-xs leading-4 opacity-70") { "{toast.body}" }
      }
      if toast.action.is_some() {
        Link (to: "{action.href}",
              class: "mt-2 inline-flex text-xs font-medium underline underline-offset-4 hover:opacity-90") {
          "{action.label}"
        }
      }
    }
    button (class: "mt-0.5 shrink-0 rounded p-1 opacity-50 hover:bg-white/10 hover:opacity-100",
            onclick: dismiss(toast.id)) {
      span (class: "material-symbols-outlined text-sm") { "close" }
    }
  }
}
```

### Step 8: Create the InlineEditor component

**Create file:** `/home/entropybender/repos/lx/crates/lx-desktop/src/components/inline_editor.rs`

Reference: `reference/paperclip/ui/src/components/InlineEditor.tsx`

Simplified port — single-line only (no markdown/multiline autosave):

```rust
#[component]
pub fn InlineEditor(
    value: String,
    on_save: EventHandler<String>,
    #[props(default = "Click to edit...".to_string())] placeholder: String,
    #[props(optional)] class: Option<String>,
) -> Element
```

Behavior:
- `editing` signal: `use_signal(|| false)`.
- `draft` signal: `use_signal(|| value.clone())`. Sync with `value` prop via `use_effect`.
- If not editing: render a `span` with class `"cursor-pointer rounded hover:bg-white/5 transition-colors px-1 -mx-1 {class}"`. If value is empty, show placeholder in gray italic. On click, set editing = true.
- If editing: render an `input` element with class `"w-full bg-transparent rounded outline-none px-1 -mx-1 {class}"`. On blur, call `on_save` with the trimmed draft if changed, set editing = false. On Enter key, same. On Escape, reset draft to original value, set editing = false.

### Step 9: Create the InlineEntitySelector component

**Create file:** `/home/entropybender/repos/lx/crates/lx-desktop/src/components/inline_entity_selector.rs`

Reference: `reference/paperclip/ui/src/components/InlineEntitySelector.tsx`

Simplified port — a dropdown selector without the popover library:

```rust
#[derive(Clone, Debug, PartialEq)]
pub struct InlineEntityOption {
    pub id: String,
    pub label: String,
}

#[component]
pub fn InlineEntitySelector(
    value: String,
    options: Vec<InlineEntityOption>,
    #[props(default = "Select...".to_string())] placeholder: String,
    on_change: EventHandler<String>,
    #[props(optional)] class: Option<String>,
) -> Element
```

Behavior:
- `open` signal: `use_signal(|| false)`.
- `query` signal: `use_signal(|| String::new())`.
- Filter options: keep those whose `label.to_lowercase()` contains `query.to_lowercase()`.
- Find current option by matching `value` against `option.id`.
- Render a trigger button:
  ```
  button (class: "inline-flex min-w-0 items-center gap-1 rounded-md border border-gray-600 bg-gray-800/40 px-2 py-1 text-sm font-medium transition-colors hover:bg-white/5 {class}",
          onclick: toggle open) {
    if current_option { "{current_option.label}" }
    else { span (class: "text-gray-400") { "{placeholder}" } }
  }
  ```
- If `open`, render a dropdown below:
  ```
  div (class: "absolute z-50 mt-1 w-[min(20rem,calc(100vw-2rem))] rounded-md border border-gray-600 bg-gray-800 shadow-lg") {
    input (class: "w-full border-b border-gray-600 bg-transparent px-2 py-1.5 text-sm outline-none placeholder:text-gray-500",
           placeholder: "Search...",
           oninput: set query) {}
    div (class: "max-h-56 overflow-y-auto py-1") {
      for option in filtered {
        button (class: "flex w-full items-center gap-2 rounded px-2 py-1.5 text-left text-sm hover:bg-white/5",
                onclick: select option) {
          span (class: "truncate") { "{option.label}" }
          if option.id == value {
            span (class: "material-symbols-outlined text-sm text-gray-400 ml-auto") { "check" }
          }
        }
      }
      if filtered.is_empty() {
        p (class: "px-2 py-2 text-xs text-gray-400") { "No results." }
      }
    }
  }
  ```
- Add a click-outside handler: when open, render a `div (class: "fixed inset-0 z-40")` that closes the dropdown on click.

### Step 10: Create the CommentThread component

**Create file:** `/home/entropybender/repos/lx/crates/lx-desktop/src/components/comment_thread.rs`

Reference: `reference/paperclip/ui/src/components/CommentThread.tsx`

Simplified port — no image upload, no reassignment, no linked runs, no plugins:

```rust
#[derive(Clone, Debug, PartialEq)]
pub struct Comment {
    pub id: String,
    pub author_name: String,
    pub body: String,
    pub created_at: String,
}

#[component]
pub fn CommentThread(
    comments: Vec<Comment>,
    on_add: EventHandler<String>,
) -> Element
```

Behavior:
- `body` signal: `use_signal(|| String::new())`.
- `submitting` signal: `use_signal(|| false)`.
- Render:
  ```
  div (class: "space-y-4") {
    h3 (class: "text-sm font-semibold") {
      "Comments ({comments.len()})"
    }
    if comments.is_empty() {
      p (class: "text-sm text-gray-400") { "No comments yet." }
    }
    div (class: "space-y-3") {
      for comment in comments.iter() {
        div (class: "border border-gray-700 p-3 overflow-hidden min-w-0 rounded-sm") {
          div (class: "flex items-center justify-between mb-1") {
            Identity { name: comment.author_name.clone(), size: "sm".to_string() }
            span (class: "text-xs text-gray-400") { "{comment.created_at}" }
          }
          MarkdownBody { children: comment.body.clone(), class: "text-sm".to_string() }
        }
      }
    }
    div (class: "space-y-2") {
      textarea (class: "w-full bg-gray-800 border border-gray-600 rounded p-2 text-sm outline-none resize-none min-h-[60px] placeholder:text-gray-500",
                placeholder: "Leave a comment...",
                value: "{body}",
                oninput: move |evt| body.set(evt.value())) {}
      div (class: "flex items-center justify-end") {
        button (class: "px-3 py-1.5 bg-blue-600 hover:bg-blue-500 text-white text-sm rounded transition-colors disabled:opacity-50",
                disabled: body.read().trim().is_empty() || *submitting.read(),
                onclick: handle_submit) {
          if *submitting.read() { "Posting..." } else { "Comment" }
        }
      }
    }
  }
  ```

`handle_submit`: read body, call `on_add.call(body_text)`, clear body, set submitting false.

### Step 11: Update components/mod.rs

**Edit file:** `/home/entropybender/repos/lx/crates/lx-desktop/src/components/mod.rs`

Replace contents with:
```rust
pub mod comment_thread;
pub mod copy_text;
pub mod empty_state;
pub mod entity_row;
pub mod filter_bar;
pub mod identity;
pub mod inline_editor;
pub mod inline_entity_selector;
pub mod markdown_body;
pub mod metric_card;
pub mod page_skeleton;
pub mod page_tab_bar;
pub mod priority_icon;
pub mod status_badge;
pub mod status_colors;
pub mod status_icon;
pub mod toast_viewport;
```

### Step 12: Add ToastViewport to Shell

**Edit file:** `/home/entropybender/repos/lx/crates/lx-desktop/src/layout/shell.rs`

Note: `ToastState::provide()` is already called in Shell by Unit 3. Do NOT add it again.

Add `ToastViewport {}` as the last child inside the root `div` of the Shell (after `StatusBar {}`). Import: `use crate::components::toast_viewport::ToastViewport;`.

## Files Created

| File | Lines (approx) |
|---|---|
| `src/components/filter_bar.rs` | ~45 |
| `src/components/copy_text.rs` | ~45 |
| `src/components/page_skeleton.rs` | ~80 |
| `src/components/page_tab_bar.rs` | ~40 |
| `src/components/markdown_body.rs` | ~30 |
| `src/components/metric_card.rs` | ~55 |
| `src/components/toast_viewport.rs` | ~70 |
| `src/components/inline_editor.rs` | ~65 |
| `src/components/inline_entity_selector.rs` | ~80 |
| `src/components/comment_thread.rs` | ~70 |
## Files Modified

| File | Change |
|---|---|
| `src/components/mod.rs` | Edit (already exists) — add 10 new module declarations |
| `src/layout/shell.rs` | Add ToastViewport render (toast context provider already exists from Unit 3) |
| `Cargo.toml` | Add `pulldown-cmark` dependency |

## Definition of Done

1. `just diagnose` passes with no errors.
2. All 10 new components compile and are importable from `crate::components::*`.
3. `ToastViewport` renders in the shell and can display toasts pushed via `ToastState`.
4. `MarkdownBody` renders markdown strings as HTML using `pulldown-cmark`.
5. `InlineEditor` toggles between display and edit mode on click.
6. `InlineEntitySelector` opens a searchable dropdown and calls `on_change` when an option is selected.
7. `CommentThread` displays a list of comments and a textarea for adding new ones.
8. `FilterBar` renders removable filter badges and a "Clear all" button.
9. `PageSkeleton` renders loading placeholders for `"list"`, `"detail"`, and `"dashboard"` variants.
10. `MetricCard` renders a value/label card with an icon, optionally wrapped in a `Link`.
11. No file exceeds 300 lines.
