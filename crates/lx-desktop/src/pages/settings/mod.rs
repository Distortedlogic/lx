mod env_vars;
mod quotas;
mod task_priority;

use dioxus::prelude::*;

use self::env_vars::EnvVarsPanel;
use self::quotas::QuotasPanel;
use self::task_priority::{ArchitectCard, SystemNotice, TaskPriorityPanel};

#[component]
pub fn Settings() -> Element {
  rsx! {
    div { class: "flex flex-col h-full p-4 overflow-auto gap-4",
      div { class: "flex items-center justify-between",
        span { class: "text-sm text-[var(--outline)] uppercase tracking-wider",
          "SYSTEM / AGENT_CONFIG_V2"
        }
        div { class: "flex gap-2",
          button { class: "border border-[var(--outline)] text-[var(--on-surface)] rounded px-4 py-2 text-xs uppercase tracking-wider hover:bg-[var(--surface-container-high)] transition-colors duration-150",
            "DISCARD CHANGES"
          }
          button { class: "bg-[var(--warning)] text-[var(--on-primary)] rounded px-4 py-2 text-xs uppercase tracking-wider font-semibold hover:brightness-110 transition-all duration-150",
            "EXECUTE DEPLOYMENT"
          }
        }
      }
      div { class: "flex gap-4 flex-1",
        div { class: "flex-[6] flex flex-col gap-4",
          EnvVarsPanel {}
          QuotasPanel {}
        }
        div { class: "flex-[4] flex flex-col gap-4",
          TaskPriorityPanel {}
          ArchitectCard {}
          SystemNotice {}
        }
      }
    }
  }
}
