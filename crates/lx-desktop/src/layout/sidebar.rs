use dioxus::prelude::*;

use crate::routes::Route;

#[component]
pub fn Sidebar(collapsed: Signal<bool>) -> Element {
  rsx! {
    nav {
      class: "bg-[var(--surface-container-low)] flex flex-col transition-all duration-200",
      class: if collapsed() { "w-12 px-1 py-4" } else { "w-56 p-4" },
      if !collapsed() {
        div { class: "mb-4",
          h1 { class: "text-xl font-bold text-[var(--primary)] font-[var(--font-display)]", "lx" }
          p { class: "text-[10px] text-[var(--outline)] tracking-[0.08em] uppercase mt-0.5", "V0.1.0-DEV" }
        }
      }
      div { class: "flex-1 flex flex-col gap-0.5 overflow-y-auto",
        NavItem { to: Route::Agents {}, label: "AGENTS", icon: "\u{2B21}", collapsed: collapsed() }
        NavItem { to: Route::Activity {}, label: "ACTIVITY", icon: "\u{25C8}", collapsed: collapsed() }
        NavItem { to: Route::Tasks {}, label: "TASKS", icon: "\u{2610}", collapsed: collapsed() }
        if !collapsed() { div { class: "h-px bg-[var(--outline-variant)]/15 my-1.5 mx-2" } }
        NavItem { to: Route::Terminals {}, label: "TERMINALS", icon: "\u{25A3}", collapsed: collapsed() }
        NavItem { to: Route::Workspaces {}, label: "WORKSPACES", icon: "\u{229E}", collapsed: collapsed() }
        NavItem { to: Route::Voice {}, label: "VOICE", icon: "\u{266A}", collapsed: collapsed() }
        if !collapsed() { div { class: "h-px bg-[var(--outline-variant)]/15 my-1.5 mx-2" } }
        NavItem { to: Route::Dashboard {}, label: "DASHBOARD", icon: "\u{25E7}", collapsed: collapsed() }
        NavItem { to: Route::Repos {}, label: "REPOS", icon: "\u{229F}", collapsed: collapsed() }
        NavItem { to: Route::Search {}, label: "SEARCH", icon: "\u{2315}", collapsed: collapsed() }
        NavItem { to: Route::Files {}, label: "FILES", icon: "\u{22A1}", collapsed: collapsed() }
        if !collapsed() { div { class: "h-px bg-[var(--outline-variant)]/15 my-1.5 mx-2" } }
        NavItem { to: Route::Settings {}, label: "SETTINGS", icon: "\u{2699}", collapsed: collapsed() }
      }
      if !collapsed() {
        div { class: "mt-3",
          p { class: "uppercase tracking-[0.05em] text-xs font-[var(--font-body)] text-[var(--outline)] mb-2 px-3", "LAYOUT MANAGER" }
          div { class: "flex flex-col gap-1",
            LayoutPill { name: "BACKENDDEV", active: true }
            LayoutPill { name: "FRONTEND TEST", active: false }
          }
        }
        div { class: "mt-3",
          p { class: "uppercase tracking-[0.05em] text-xs font-[var(--font-body)] text-[var(--outline)] mb-2 px-3", "AGENT MONITORING" }
          button {
            class: "w-full text-left px-3 py-1.5 text-xs text-[var(--outline)] hover:bg-[var(--surface-container-high)] hover:text-[var(--on-surface)] rounded transition-colors duration-150",
            "\u{2299} SAVE CURRENT"
          }
        }
      }
      button {
        class: "mt-2 mx-auto w-6 h-6 flex items-center justify-center text-[var(--outline)] hover:text-[var(--on-surface)] text-xs rounded hover:bg-[var(--surface-container-high)] transition-colors duration-150",
        onclick: move |_| collapsed.set(!collapsed()),
        if collapsed() { "\u{203A}" } else { "\u{2039}" }
      }
    }
  }
}

#[component]
fn NavItem(to: Route, label: &'static str, icon: &'static str, collapsed: bool) -> Element {
  let cls = if collapsed {
    "flex items-center justify-center px-1 py-2 rounded text-sm text-[var(--outline)] hover:bg-[var(--surface-container-high)] hover:text-[var(--on-surface)] transition-colors duration-150".to_string()
  } else {
    "flex items-center gap-2 px-3 py-1.5 rounded text-sm text-[var(--outline)] hover:bg-[var(--surface-container-high)] hover:text-[var(--on-surface)] transition-colors duration-150".to_string()
  };
  rsx! {
    Link {
      to,
      active_class: "!text-[var(--primary)] bg-[var(--surface-container-high)] border-l-2 border-[var(--primary)]",
      class: "{cls}",
      span { class: "text-xs", "{icon}" }
      if !collapsed {
        span { class: "uppercase text-xs tracking-[0.05em]", "{label}" }
      }
    }
  }
}

#[component]
fn LayoutPill(name: &'static str, active: bool) -> Element {
  let class = if active {
    "px-3 py-1.5 rounded text-xs bg-[var(--surface-container-high)] text-[var(--primary)] transition-colors duration-150 flex items-center gap-1 text-left"
  } else {
    "px-3 py-1.5 rounded text-xs text-[var(--outline)] hover:bg-[var(--surface-container-high)] transition-colors duration-150 text-left"
  };
  rsx! {
    button { class,
      "{name}"
      if active {
        span { class: "text-[var(--primary)]", " \u{2605}" }
      }
    }
  }
}
