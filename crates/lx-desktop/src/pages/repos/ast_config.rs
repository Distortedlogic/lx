use dioxus::prelude::*;

#[component]
pub fn AstConfig() -> Element {
  rsx! {
    div { class: "flex-1 flex flex-col p-4 overflow-auto min-w-0",
      div { class: "flex items-center justify-between mb-4",
        span { class: "text-sm font-semibold uppercase tracking-wider text-[var(--on-surface)]",
          "AST CONFIGURATION & ANALYSIS"
        }
        span { class: "text-xs text-[var(--outline)] uppercase tracking-wider",
          "NODE_COUNT: 14,282"
        }
      }
      div { class: "flex gap-0 mb-4",
        span { class: "bg-[var(--primary)] text-[var(--on-primary)] px-6 py-2 text-xs uppercase tracking-wider font-semibold rounded-l cursor-pointer",
          "SYNTACTIC"
        }
        span { class: "bg-[var(--surface-container)] text-[var(--outline)] px-6 py-2 text-xs uppercase tracking-wider border border-[var(--outline-variant)]/30 cursor-pointer hover:text-[var(--on-surface)] transition-colors duration-150",
          "SEMANTIC"
        }
        span { class: "bg-[var(--surface-container)] text-[var(--outline)] px-6 py-2 text-xs uppercase tracking-wider border border-[var(--outline-variant)]/30 rounded-r cursor-pointer hover:text-[var(--on-surface)] transition-colors duration-150",
          "HYBRID"
        }
      }
      div { class: "bg-[var(--surface-container)] border border-[var(--outline-variant)]/30 rounded-lg p-4 mb-4",
        div { class: "flex items-center gap-2 mb-2",
          span { class: "text-xs font-semibold uppercase tracking-wider text-[var(--on-surface)]",
            "AST_ROOT_SELECTOR"
          }
          span { class: "text-[var(--warning)]", "\u{2B50}" }
        }
        p { class: "text-xs text-[var(--outline)] font-mono mb-3",
          "// Target specific tree depth"
        }
        input {
          r#type: "range",
          class: "w-full accent-[var(--primary)] mb-3",
        }
        div { class: "flex gap-4",
          div { class: "flex-1",
            p { class: "text-[10px] uppercase tracking-wider text-[var(--outline)] mb-1",
              "STRATEGY"
            }
            div { class: "bg-[var(--surface-container-low)] rounded px-3 py-1.5 text-xs text-[var(--on-surface-variant)] flex items-center justify-between",
              span { "DFS_TRAVERSAL" }
              span { class: "text-[var(--outline)]", "\u{25BC}" }
            }
          }
          div { class: "flex-1",
            p { class: "text-[10px] uppercase tracking-wider text-[var(--outline)] mb-1",
              "PARSER_V"
            }
            div { class: "bg-[var(--surface-container-low)] rounded px-3 py-1.5 text-xs text-[var(--on-surface-variant)]",
              "8.4.2-STABLE"
            }
          }
        }
      }
      div { class: "bg-[var(--surface-container)] border border-[var(--outline-variant)]/30 rounded-lg p-4 flex-1 overflow-auto",
        p { class: "text-xs font-semibold uppercase tracking-wider text-[var(--on-surface)] mb-3",
          "PREVIEW_AST_JSON"
        }
        pre { class: "text-xs font-mono text-[var(--on-surface-variant)] whitespace-pre leading-relaxed",
          "{{\n  \"type\": \"Program\",\n  \"body\": [\n    {{\n      \"type\": \"VariableDeclaration\",\n      \"declarations\":\n        {{\n          \"type\": \"VariableDeclarator\",\n          \"id\": {{ \"type\": \"Identifier\", \"name\": \"RAG_ENGINE\" }},\n          \"init\": {{ \"type\": \"Literal\", \"value\": \"ACTIVE\" }}\n        }},\n      \"kind\": \"const\"\n    }}\n  ]\n}}"
        }
      }
    }
  }
}
