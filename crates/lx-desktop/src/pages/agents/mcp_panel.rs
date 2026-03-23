use dioxus::prelude::*;

struct McpModule {
  name: &'static str,
  version: &'static str,
  status: &'static str,
}

const MODULES: &[McpModule] = &[
  McpModule { name: "POSTGRES_INTERFACE", version: "v2.1.0", status: "ONLINE" },
  McpModule { name: "AWS_CONSOLE_BRIDGE", version: "v1.4.3", status: "ONLINE" },
  McpModule { name: "O_WORKSPACE_SYNC", version: "", status: "OFFLINE" },
];

#[component]
pub fn McpPanel() -> Element {
  rsx! {
    div { class: "flex items-center gap-3 mb-4",
      div { class: "h-px flex-1 bg-[var(--outline-variant)]" }
      span { class: "text-xs uppercase tracking-wider font-semibold text-[var(--on-surface)]",
        "MCP_EXTENSIONS"
      }
      div { class: "h-px flex-1 bg-[var(--outline-variant)]" }
    }
    div { class: "grid grid-cols-4 gap-3",
      for module in MODULES {
        div { class: "bg-[var(--surface-container-low)] border border-[var(--outline-variant)]/30 rounded-lg p-4 flex flex-col gap-2",
          span { class: "text-2xl text-[var(--primary)]", "\u{1F5C4}" }
          span { class: "text-xs font-semibold uppercase tracking-wider text-[var(--on-surface)]",
            "{module.name}"
          }
          if module.status == "OFFLINE" {
            if module.version.is_empty() {
              span { class: "text-[10px] uppercase tracking-wider text-[var(--error)]",
                "{module.status}"
              }
            } else {
              span { class: "text-[10px] uppercase tracking-wider text-[var(--outline)]",
                "{module.version} // "
                span { class: "text-[var(--error)]", "{module.status}" }
              }
            }
          } else {
            if module.version.is_empty() {
              span { class: "text-[10px] uppercase tracking-wider text-[var(--outline)]",
                "{module.status}"
              }
            } else {
              span { class: "text-[10px] uppercase tracking-wider text-[var(--outline)]",
                "{module.version} // {module.status}"
              }
            }
          }
        }
      }
      div { class: "bg-[var(--surface-container-low)] border border-dashed border-[var(--outline-variant)] rounded-lg p-4 flex flex-col items-center justify-center gap-2 cursor-pointer hover:border-[var(--primary)] transition-colors duration-150",
        span { class: "text-2xl text-[var(--outline)]", "+" }
        span { class: "text-xs uppercase tracking-wider text-[var(--outline)]",
          "INSTALL MODULE"
        }
      }
    }
  }
}
