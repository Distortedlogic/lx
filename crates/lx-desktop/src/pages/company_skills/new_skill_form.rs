use dioxus::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct NewSkillPayload {
  pub name: String,
  pub slug: Option<String>,
  pub description: Option<String>,
}

#[component]
pub fn NewSkillForm(on_create: EventHandler<NewSkillPayload>, on_cancel: EventHandler<()>, is_pending: bool) -> Element {
  let mut name = use_signal(String::new);
  let mut slug = use_signal(String::new);
  let mut description = use_signal(String::new);

  rsx! {
    div { class: "border-b border-[var(--outline-variant)] px-4 py-4",
      div { class: "space-y-3",
        input {
          class: "w-full h-9 border-0 border-b border-[var(--outline-variant)] bg-transparent px-0 text-sm outline-none text-[var(--on-surface)] placeholder-[var(--outline)]",
          placeholder: "Skill name",
          value: "{name}",
          oninput: move |evt| name.set(evt.value()),
        }
        input {
          class: "w-full h-9 border-0 border-b border-[var(--outline-variant)] bg-transparent px-0 text-sm outline-none text-[var(--on-surface)] placeholder-[var(--outline)]",
          placeholder: "optional-shortname",
          value: "{slug}",
          oninput: move |evt| slug.set(evt.value()),
        }
        textarea {
          class: "w-full min-h-20 border-0 border-b border-[var(--outline-variant)] bg-transparent px-0 text-sm outline-none text-[var(--on-surface)] placeholder-[var(--outline)]",
          placeholder: "Short description",
          value: "{description}",
          oninput: move |evt| description.set(evt.value()),
        }
        div { class: "flex items-center justify-end gap-2",
          button {
            class: "px-3 py-1.5 text-xs rounded hover:bg-[var(--surface-container)]",
            disabled: is_pending,
            onclick: move |_| on_cancel.call(()),
            "Cancel"
          }
          button {
            class: "bg-[var(--primary)] text-[var(--on-primary)] px-3 py-1.5 text-xs rounded font-semibold",
            disabled: is_pending || name().trim().is_empty(),
            onclick: move |_| {
                on_create
                    .call(NewSkillPayload {
                        name: name().trim().to_string(),
                        slug: if slug().trim().is_empty() {
                            None
                        } else {
                            Some(slug().trim().to_string())
                        },
                        description: if description().trim().is_empty() {
                            None
                        } else {
                            Some(description().trim().to_string())
                        },
                    });
            },
            if is_pending {
              "Creating..."
            } else {
              "Create skill"
            }
          }
        }
      }
    }
  }
}
