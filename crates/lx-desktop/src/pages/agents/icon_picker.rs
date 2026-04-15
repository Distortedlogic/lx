use dioxus::prelude::*;

const AGENT_ICONS: &[&str] = &[
  "smart_toy",
  "psychology",
  "engineering",
  "terminal",
  "code",
  "bug_report",
  "build",
  "science",
  "analytics",
  "security",
  "support_agent",
  "manage_accounts",
  "group",
  "school",
  "lightbulb",
  "auto_fix_high",
  "memory",
  "hub",
  "device_hub",
  "dns",
];

#[component]
pub fn AgentIconPicker(value: Option<String>, on_change: EventHandler<String>) -> Element {
  let mut open = use_signal(|| false);
  let mut search = use_signal(String::new);
  let current = value.as_deref().unwrap_or("smart_toy");

  let filtered: Vec<&&str> = AGENT_ICONS
    .iter()
    .filter(|name| {
      let q = search.read();
      q.is_empty() || name.contains(q.as_str())
    })
    .collect();

  rsx! {
    div { class: "relative",
      button {
        class: "shrink-0 flex items-center justify-center h-12 w-12 rounded-lg bg-[var(--surface-container-high)] hover:bg-[var(--surface-container)] transition-colors",
        onclick: move |_| {
            let was_open = *open.read();
            open.set(!was_open);
        },
        span { class: "material-symbols-outlined text-lg", "{current}" }
      }
      if *open.read() {
        div { class: "absolute top-full left-0 mt-1 z-50 w-72 border border-[var(--outline-variant)] bg-[var(--surface-container)] shadow-lg rounded-lg p-3",
          input {
            class: "w-full rounded border border-[var(--outline-variant)] px-2 py-1.5 bg-transparent text-sm mb-2 outline-none placeholder:text-[var(--outline)]/40",
            placeholder: "Search icons...",
            value: "{search}",
            oninput: move |evt| search.set(evt.value().to_string()),
          }
          div { class: "grid grid-cols-7 gap-1 max-h-48 overflow-y-auto",
            for icon_name in filtered.iter() {
              button {
                class: {
                    let selected = **icon_name == current;
                    if selected {
                        "flex items-center justify-center h-8 w-8 rounded bg-[var(--primary)]/20 ring-1 ring-[var(--primary)]"
                    } else {
                        "flex items-center justify-center h-8 w-8 rounded hover:bg-[var(--surface-container-high)] transition-colors"
                    }
                },
                onclick: {
                    let name = icon_name.to_string();
                    move |_| {
                        on_change.call(name.clone());
                        open.set(false);
                        search.set(String::new());
                    }
                },
                span { class: "material-symbols-outlined text-sm", "{icon_name}" }
              }
            }
            if filtered.is_empty() {
              p { class: "col-span-7 text-xs text-[var(--outline)] text-center py-2",
                "No icons match"
              }
            }
          }
        }
      }
    }
  }
}
