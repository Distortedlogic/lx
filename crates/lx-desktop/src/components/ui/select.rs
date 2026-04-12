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
  let mut open = use_signal(|| false);
  let mut search_query = use_signal(String::new);
  let mut focused_index = use_signal(|| 0usize);

  let filtered: Vec<SelectOption> = options
    .iter()
    .filter(|opt| if !searchable || search_query.read().is_empty() { true } else { opt.label.to_lowercase().contains(&search_query.read().to_lowercase()) })
    .cloned()
    .collect();

  let display_label = options.iter().find(|o| o.value == value).map(|o| o.label.clone());

  let filtered_len = filtered.len();
  let filtered_for_trigger = filtered.clone();
  let onchange_for_trigger = onchange;
  let trigger_key_handler = move |evt: KeyboardEvent| {
    handle_select_key(&evt, open, focused_index, search_query, filtered_len, &filtered_for_trigger, onchange_for_trigger);
  };

  let filtered_for_search = filtered.clone();
  let onchange_for_search = onchange;
  let search_key_handler = move |evt: KeyboardEvent| {
    handle_select_key(&evt, open, focused_index, search_query, filtered_len, &filtered_for_search, onchange_for_search);
  };

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
        onkeydown: trigger_key_handler,
        if let Some(ref label) = display_label {
          span { class: "text-[var(--on-surface)]", "{label}" }
        } else {
          span { class: "text-[var(--outline)]",
            if placeholder.is_empty() {
              "Select..."
            } else {
              "{placeholder}"
            }
          }
        }
        span { class: "material-symbols-outlined text-sm text-[var(--outline)]",
          if open() {
            "expand_less"
          } else {
            "expand_more"
          }
        }
      }
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
                onkeydown: search_key_handler,
              }
            }
          }

          for (idx , opt) in filtered.iter().enumerate() {
            {
                render_select_item(opt, idx, &value, focused_index, open, search_query, onchange)
            }
          }

          if filtered.is_empty() {
            div { class: "px-3 py-2 text-sm text-[var(--outline)]", "No results" }
          }
        }
      }
    }
  }
}

fn handle_select_key(
  evt: &KeyboardEvent,
  mut open: Signal<bool>,
  mut focused_index: Signal<usize>,
  mut search_query: Signal<String>,
  filtered_len: usize,
  filtered: &[SelectOption],
  onchange: EventHandler<String>,
) {
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
    },
    Key::ArrowUp => {
      evt.prevent_default();
      if open() {
        let cur = *focused_index.read();
        if cur > 0 {
          focused_index.set(cur - 1);
        }
      }
    },
    Key::Enter => {
      evt.prevent_default();
      if open() {
        let idx = *focused_index.read();
        if let Some(opt) = filtered.get(idx)
          && !opt.disabled
        {
          onchange.call(opt.value.clone());
          open.set(false);
          search_query.set(String::new());
        }
      } else {
        open.set(true);
      }
    },
    Key::Escape => {
      evt.prevent_default();
      evt.stop_propagation();
      open.set(false);
      search_query.set(String::new());
    },
    _ => {},
  }
}

fn render_select_item(
  opt: &SelectOption,
  idx: usize,
  current_value: &str,
  mut focused_index: Signal<usize>,
  mut open: Signal<bool>,
  mut search_query: Signal<String>,
  onchange: EventHandler<String>,
) -> Element {
  let is_selected = opt.value == current_value;
  let is_focused = *focused_index.read() == idx;
  let val = opt.value.clone();
  let opt_disabled = opt.disabled;

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
          if !opt_disabled {
              onchange.call(val.clone());
              open.set(false);
              search_query.set(String::new());
          }
      },
      if is_selected {
        span { class: "material-symbols-outlined text-sm text-[var(--primary)]",
          "check"
        }
      } else {
        span { class: "w-5" }
      }
      span { "{opt.label}" }
    }
  }
}
