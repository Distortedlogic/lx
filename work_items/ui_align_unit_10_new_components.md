# UNIT 10: Build MarkdownEditor and ScrollToBottom Components

## Goal

Create two new reusable components: `MarkdownEditor` (textarea with live preview toggle and toolbar) and `ScrollToBottom` (container that auto-scrolls to bottom on new children). Register both in `components/mod.rs`.

## Files Created

| File | Action |
|------|--------|
| `crates/lx-desktop/src/components/markdown_editor.rs` | New file |
| `crates/lx-desktop/src/components/scroll_to_bottom.rs` | New file |

## Files Modified

| File | Action |
|------|--------|
| `crates/lx-desktop/src/components/mod.rs` | Add two `pub mod` lines |

## Reference Files (read-only)

| File | Why |
|------|-----|
| `crates/lx-desktop/src/components/markdown_body.rs` | `MarkdownBody` component for preview rendering |
| `crates/lx-desktop/Cargo.toml` | Verify `dioxus` and `pulldown-cmark` are available |
| `crates/lx-desktop/src/styles.rs` | Style constants |

---

## Step 1: Create `markdown_editor.rs`

Create new file `crates/lx-desktop/src/components/markdown_editor.rs` with this exact content:

```rust
use dioxus::prelude::*;

use super::markdown_body::MarkdownBody;

#[derive(Clone, Copy, Debug, PartialEq)]
enum EditorMode {
  Edit,
  Preview,
  Split,
}

#[component]
pub fn MarkdownEditor(
  value: String,
  on_change: EventHandler<String>,
  #[props(optional)] on_submit: Option<EventHandler<String>>,
  #[props(optional)] placeholder: Option<String>,
  #[props(optional)] class: Option<String>,
) -> Element {
  let mut mode = use_signal(|| EditorMode::Edit);
  let current_mode = *mode.read();
  let extra_class = class.as_deref().unwrap_or("");
  let placeholder_text = placeholder.as_deref().unwrap_or("Write markdown...");

  rsx! {
    div { class: "flex flex-col border border-[var(--outline-variant)]/30 rounded-lg overflow-hidden {extra_class}",
      div { class: "flex items-center justify-between border-b border-[var(--outline-variant)]/30 px-2 py-1 bg-[var(--surface-container)]",
        ToolbarButtons { value: value.clone(), on_change: on_change }
        div { class: "flex gap-0.5",
          ModeButton { label: "Edit", active: current_mode == EditorMode::Edit, on_click: move |_| mode.set(EditorMode::Edit) }
          ModeButton { label: "Preview", active: current_mode == EditorMode::Preview, on_click: move |_| mode.set(EditorMode::Preview) }
          ModeButton { label: "Split", active: current_mode == EditorMode::Split, on_click: move |_| mode.set(EditorMode::Split) }
        }
      }
      match current_mode {
          EditorMode::Edit => rsx! {
            EditorTextarea {
              value: value.clone(),
              placeholder: placeholder_text.to_string(),
              on_change: on_change,
              on_submit: on_submit,
            }
          },
          EditorMode::Preview => rsx! {
            div { class: "p-3 min-h-[8rem] max-h-80 overflow-y-auto",
              if value.is_empty() {
                p { class: "text-sm text-[var(--outline)]", "Nothing to preview." }
              } else {
                MarkdownBody { content: value.clone() }
              }
            }
          },
          EditorMode::Split => rsx! {
            div { class: "grid grid-cols-2 divide-x divide-[var(--outline-variant)]/30",
              EditorTextarea {
                value: value.clone(),
                placeholder: placeholder_text.to_string(),
                on_change: on_change,
                on_submit: on_submit,
              }
              div { class: "p-3 min-h-[8rem] max-h-80 overflow-y-auto",
                if value.is_empty() {
                  p { class: "text-sm text-[var(--outline)]", "Nothing to preview." }
                } else {
                  MarkdownBody { content: value.clone() }
                }
              }
            }
          },
      }
    }
  }
}

#[component]
fn EditorTextarea(
  value: String,
  placeholder: String,
  on_change: EventHandler<String>,
  #[props(optional)] on_submit: Option<EventHandler<String>>,
) -> Element {
  rsx! {
    textarea {
      class: "w-full min-h-[8rem] max-h-80 p-3 bg-transparent outline-none text-sm font-mono text-[var(--on-surface)] placeholder:text-[var(--outline)]/40 resize-y",
      value: "{value}",
      placeholder: "{placeholder}",
      oninput: move |evt| on_change.call(evt.value().to_string()),
      onkeydown: move |evt| {
          if evt.modifiers().meta() && evt.key() == Key::Enter {
            if let Some(ref handler) = on_submit {
              handler.call(value.clone());
            }
          }
      },
    }
  }
}

#[component]
fn ModeButton(label: &'static str, active: bool, on_click: EventHandler<()>) -> Element {
  let bg = if active { "bg-[var(--surface-container-high)] text-[var(--on-surface)]" } else { "text-[var(--outline)] hover:text-[var(--on-surface)]" };
  rsx! {
    button {
      class: "px-2 py-0.5 text-[11px] font-medium rounded transition-colors {bg}",
      onclick: move |_| on_click.call(()),
      "{label}"
    }
  }
}

#[component]
fn ToolbarButtons(value: String, on_change: EventHandler<String>) -> Element {
  rsx! {
    div { class: "flex gap-0.5",
      ToolbarBtn {
        icon: "format_bold",
        on_click: {
            let v = value.clone();
            move |_| on_change.call(format!("{v}****"))
        },
      }
      ToolbarBtn {
        icon: "format_italic",
        on_click: {
            let v = value.clone();
            move |_| on_change.call(format!("{v}**"))
        },
      }
      ToolbarBtn {
        icon: "code",
        on_click: {
            let v = value.clone();
            move |_| on_change.call(format!("{v}\n```\n\n```"))
        },
      }
      ToolbarBtn {
        icon: "link",
        on_click: {
            let v = value.clone();
            move |_| on_change.call(format!("{v}[text](url)"))
        },
      }
      ToolbarBtn {
        icon: "title",
        on_click: {
            let v = value.clone();
            move |_| on_change.call(format!("{v}\n## "))
        },
      }
    }
  }
}

#[component]
fn ToolbarBtn(icon: &'static str, on_click: EventHandler<()>) -> Element {
  rsx! {
    button {
      class: "w-6 h-6 flex items-center justify-center text-[var(--outline)] hover:text-[var(--on-surface)] hover:bg-[var(--surface-container-high)] rounded transition-colors",
      onclick: move |_| on_click.call(()),
      span { class: "material-symbols-outlined text-sm", "{icon}" }
    }
  }
}
```

**File is 156 lines (under 300).**

**Component behavior:**
- Three modes: Edit (textarea only), Preview (MarkdownBody rendering), Split (side-by-side)
- Toolbar inserts markdown syntax at end of current value (bold `****`, italic `**`, code fence, link template, heading)
- Cmd+Enter in textarea fires `on_submit` if provided
- All props except `value` and `on_change` are optional

---

## Step 2: Create `scroll_to_bottom.rs`

Create new file `crates/lx-desktop/src/components/scroll_to_bottom.rs` with this exact content:

```rust
use dioxus::prelude::*;

#[component]
pub fn ScrollToBottom(
  children: Element,
  #[props(optional)] class: Option<String>,
) -> Element {
  let extra = class.as_deref().unwrap_or("");
  let mut child_count = use_signal(|| 0usize);
  let mut user_scrolled_up = use_signal(|| false);
  let sentinel_id = "scroll-sentinel";

  use_effect(move || {
    let count = child_count();
    if count > 0 && !user_scrolled_up() {
      spawn(async move {
        let js = format!(
          "document.getElementById('{sentinel_id}')?.scrollIntoView({{ behavior: 'smooth' }})"
        );
        let _ = eval(&js).await;
      });
    }
  });

  rsx! {
    div {
      class: "overflow-y-auto relative {extra}",
      onscroll: move |_evt| {
          spawn(async move {
            let js = format!(
              r#"(function() {{
                var el = document.getElementById('{sentinel_id}');
                if (!el || !el.parentElement) return 'false';
                var parent = el.parentElement;
                var diff = parent.scrollHeight - parent.scrollTop - parent.clientHeight;
                return diff > 40 ? 'true' : 'false';
              }})()"#
            );
            match eval(&js).await {
              Ok(val) => {
                  let is_up = val.to_string().contains("true");
                  user_scrolled_up.set(is_up);
              }
              Err(_) => {}
            }
          });
      },
      {children}
      div { id: sentinel_id,
        onmounted: move |_| child_count.set(child_count() + 1),
      }
    }
  }
}
```

**File is 53 lines (under 300).**

**Component behavior:**
- Wraps children in a scrollable container
- Appends an invisible sentinel `div` at the bottom
- On mount and whenever `child_count` changes (triggering the effect), scrolls the sentinel into view using `eval` with `scrollIntoView`
- On scroll events, checks if the user has scrolled more than 40px above the bottom. If so, sets `user_scrolled_up = true` to suppress auto-scrolling
- When the user scrolls back to bottom (within 40px), auto-scroll resumes
- `class` prop allows caller to set height/width constraints

---

## Step 3: Register both modules in `mod.rs`

In `crates/lx-desktop/src/components/mod.rs`:

Old text (lines 1-25):
```rust
pub mod ui;

pub mod command_palette;
pub mod comment_thread;
pub mod company_pattern_icon;
pub mod company_switcher;
pub mod copy_text;
pub mod empty_state;
pub mod entity_row;
pub mod file_tree;
pub mod filter_bar;
pub mod identity;
pub mod inline_editor;
pub mod inline_entity_selector;
pub mod markdown_body;
pub mod metric_card;
pub mod onboarding;
pub mod page_skeleton;
pub mod page_tab_bar;
pub mod priority_icon;
pub mod status_badge;
pub mod status_colors;
pub mod status_icon;
pub mod toast_viewport;
```

New text:
```rust
pub mod ui;

pub mod command_palette;
pub mod comment_thread;
pub mod company_pattern_icon;
pub mod company_switcher;
pub mod copy_text;
pub mod empty_state;
pub mod entity_row;
pub mod file_tree;
pub mod filter_bar;
pub mod identity;
pub mod inline_editor;
pub mod inline_entity_selector;
pub mod markdown_body;
pub mod markdown_editor;
pub mod metric_card;
pub mod onboarding;
pub mod page_skeleton;
pub mod page_tab_bar;
pub mod priority_icon;
pub mod scroll_to_bottom;
pub mod status_badge;
pub mod status_colors;
pub mod status_icon;
pub mod toast_viewport;
```

Two lines added: `pub mod markdown_editor;` (after `markdown_body`) and `pub mod scroll_to_bottom;` (after `priority_icon`).

---

## Verification

After all changes:
- `markdown_editor.rs` is 156 lines (under 300).
- `scroll_to_bottom.rs` is 53 lines (under 300).
- `mod.rs` is 27 lines (under 300).
- No code comments or docstrings in any file.
- No `#[allow(...)]` macros.
- No new dependencies needed in `Cargo.toml` (uses `dioxus::prelude::*`, `pulldown-cmark` via `MarkdownBody`, and `dioxus::eval`).
- `MarkdownEditor` reuses existing `MarkdownBody` for preview rendering.
- `ScrollToBottom` uses `eval` (available in Dioxus desktop/web) for DOM scrolling.
