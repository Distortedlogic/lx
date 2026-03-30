use dioxus::prelude::*;

use crate::routes::Route;

#[derive(Clone)]
pub struct CommandPaletteOpen(pub Signal<bool>);

struct PaletteItem {
  label: &'static str,
  icon: &'static str,
  route: Option<Route>,
}

const ACTIONS: &[(&str, &str)] = &[("Create new issue", "add"), ("Create new agent", "add")];

fn page_items() -> Vec<PaletteItem> {
  vec![
    PaletteItem { label: "Agents", icon: "smart_toy", route: Some(Route::Agents {}) },
    PaletteItem { label: "Activity", icon: "pulse_alert", route: Some(Route::Activity {}) },
    PaletteItem { label: "Tools", icon: "build", route: Some(Route::Tools {}) },
    PaletteItem { label: "Settings", icon: "settings", route: Some(Route::Settings {}) },
    PaletteItem { label: "Accounts", icon: "account_circle", route: Some(Route::Accounts {}) },
  ]
}

fn matches_query(label: &str, query: &str) -> bool {
  if query.is_empty() {
    return true;
  }
  label.to_lowercase().contains(&query.to_lowercase())
}

#[component]
pub fn CommandPalette() -> Element {
  let palette = use_context::<CommandPaletteOpen>();
  let mut open = palette.0;
  let mut query = use_signal(String::new);

  use_effect(move || {
    if !open() {
      query.set(String::new());
    }
  });

  if !open() {
    return rsx! {};
  }

  let q = query();
  let pages = page_items();
  let filtered_actions: Vec<_> = ACTIONS.iter().filter(|(label, _)| matches_query(label, &q)).collect();
  let filtered_pages: Vec<_> = pages.iter().filter(|item| matches_query(item.label, &q)).collect();

  rsx! {
    div {
      class: "fixed inset-0 z-[60] bg-black/50",
      onclick: move |_| open.set(false),
      div {
        class: "fixed top-[20%] left-1/2 -translate-x-1/2 w-full max-w-md bg-[var(--surface-container)] border border-[var(--outline)] shadow-2xl z-[60]",
        onclick: move |e| e.stop_propagation(),
        input {
          class: "w-full px-4 py-3 bg-transparent border-b border-[var(--outline-variant)] text-[var(--on-surface)] text-sm outline-none placeholder:text-[var(--outline)]",
          placeholder: "Search pages, actions...",
          value: "{query}",
          oninput: move |e| query.set(e.value()),
          autofocus: true,
        }
        div { class: "max-h-64 overflow-y-auto py-1",
          for (label , icon) in filtered_actions {
            div {
              class: "flex items-center gap-3 px-4 py-2 text-sm cursor-pointer hover:bg-[var(--surface-container-highest)] text-[var(--on-surface)]",
              onclick: move |_| open.set(false),
              span { class: "material-symbols-outlined text-lg text-[var(--outline)]",
                "{icon}"
              }
              span { "{label}" }
            }
          }
          for item in filtered_pages {
            {render_page_item(item.label, item.icon, &item.route, open)}
          }
        }
      }
    }
  }
}

fn render_page_item(label: &str, icon: &str, route: &Option<Route>, mut open: Signal<bool>) -> Element {
  let nav = navigator();
  let route = route.clone();
  let label = label.to_string();
  let icon = icon.to_string();
  rsx! {
    div {
      class: "flex items-center gap-3 px-4 py-2 text-sm cursor-pointer hover:bg-[var(--surface-container-highest)] text-[var(--on-surface)]",
      onclick: move |_| {
          if let Some(ref r) = route {
              nav.push(r.clone());
          }
          open.set(false);
      },
      span { class: "material-symbols-outlined text-lg text-[var(--outline)]",
        "{icon}"
      }
      span { "{label}" }
    }
  }
}
