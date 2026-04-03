use dioxus::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ImportStep {
  SelectSource,
  Preview,
  Applying,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ImportSourceKind {
  FileUpload,
  Url,
  GitHub,
}

#[component]
pub fn CompanyImport() -> Element {
  let mut step = use_signal(|| ImportStep::SelectSource);
  let mut source_kind = use_signal(|| ImportSourceKind::FileUpload);
  let mut url_input = use_signal(String::new);
  let mut github_input = use_signal(String::new);

  rsx! {
    div { class: "flex flex-col h-full",
      div { class: "flex items-center gap-2 px-4 py-3 border-b border-[var(--outline-variant)]",
        span { class: "material-symbols-outlined text-sm text-[var(--outline)]",
          "upload"
        }
        h1 { class: "text-lg font-semibold text-[var(--on-surface)]", "Import Company Package" }
      }
      div { class: "flex-1 overflow-auto p-6",
        match step() {
            ImportStep::SelectSource => rsx! {
              div { class: "max-w-2xl mx-auto space-y-6",
                h2 { class: "text-base font-semibold text-[var(--on-surface)]", "Select Import Source" }
                div { class: "grid grid-cols-3 gap-4",
                  for (kind , label , icon) in [
                      (ImportSourceKind::FileUpload, "Upload File", "upload_file"),
                      (ImportSourceKind::Url, "From URL", "link"),
                      (ImportSourceKind::GitHub, "From GitHub", "code"),
                  ]
                  {
                    {
                        let is_selected = source_kind() == kind;
                        let border = if is_selected {
                            "border-[var(--primary)] ring-1 ring-[var(--primary)]"
                        } else {
                            "border-[var(--outline-variant)] hover:border-[var(--outline)]"
                        };
                        rsx! {
                          button {
                            class: "flex flex-col items-center gap-2 p-4 rounded-lg border cursor-pointer {border}",
                            onclick: move |_| source_kind.set(kind),
                            span { class: "material-symbols-outlined text-lg text-[var(--outline)]", "{icon}" }
                            span { class: "text-sm font-medium text-[var(--on-surface)]", "{label}" }
                          }
                        }
                    }
                  }
                }
                match source_kind() {
                    ImportSourceKind::FileUpload => rsx! {
                      div { class: "rounded-lg border border-dashed border-[var(--outline-variant)] p-8 text-center",
                        span { class: "material-symbols-outlined text-xl text-[var(--outline)] mb-2", "cloud_upload" }
                        p { class: "text-sm text-[var(--outline)]", "Drop a .zip file here or click to browse" }
                        input { r#type: "file", accept: ".zip", class: "mt-2" }
                      }
                    },
                    ImportSourceKind::Url => rsx! {
                      div { class: "space-y-2",
                        label { class: "text-xs font-medium text-[var(--on-surface)]", "Package URL" }
                        input {
                          class: "w-full rounded-md border border-[var(--outline-variant)] bg-transparent px-3 py-2 text-sm outline-none text-[var(--on-surface)]",
                          placeholder: "https://example.com/company-package.zip",
                          value: "{url_input}",
                          oninput: move |evt| url_input.set(evt.value()),
                        }
                      }
                    },
                    ImportSourceKind::GitHub => rsx! {
                      div { class: "space-y-2",
                        label { class: "text-xs font-medium text-[var(--on-surface)]", "GitHub Repository" }
                        input {
                          class: "w-full rounded-md border border-[var(--outline-variant)] bg-transparent px-3 py-2 text-sm outline-none text-[var(--on-surface)]",
                          placeholder: "owner/repo",
                          value: "{github_input}",
                          oninput: move |evt| github_input.set(evt.value()),
                        }
                      }
                    },
                }
                div { class: "flex justify-end",
                  button {
                    class: "bg-[var(--primary)] text-[var(--on-primary)] rounded px-4 py-2 text-sm font-semibold",
                    onclick: move |_| step.set(ImportStep::Preview),
                    "Continue"
                  }
                }
              }
            },
            ImportStep::Preview => rsx! {
              div { class: "max-w-4xl mx-auto space-y-6",
                h2 { class: "text-base font-semibold text-[var(--on-surface)]", "Import Preview" }
                p { class: "text-sm text-[var(--outline)]", "Review the contents before importing." }
                div { class: "flex justify-between",
                  button {
                    class: "border border-[var(--outline-variant)] rounded px-4 py-2 text-sm",
                    onclick: move |_| step.set(ImportStep::SelectSource),
                    "Back"
                  }
                  button {
                    class: "bg-[var(--primary)] text-[var(--on-primary)] rounded px-4 py-2 text-sm font-semibold",
                    onclick: move |_| step.set(ImportStep::Applying),
                    "Apply Import"
                  }
                }
              }
            },
            ImportStep::Applying => rsx! {
              div { class: "flex flex-col items-center justify-center py-16",
                span { class: "material-symbols-outlined text-xl text-[var(--primary)] animate-spin mb-4",
                  "progress_activity"
                }
                p { class: "text-sm text-[var(--outline)]", "Importing..." }
              }
            },
        }
      }
    }
  }
}
