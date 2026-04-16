use dioxus::prelude::*;
use dioxus_primitives::avatar::{Avatar, AvatarFallback, AvatarImage};

#[derive(Clone, PartialEq, Props)]
pub struct IdentityProps {
  pub name: String,
  #[props(optional)]
  pub avatar_url: Option<String>,
  #[props(optional)]
  pub initials: Option<String>,
  #[props(default = "default".to_string())]
  pub size: String,
  #[props(optional)]
  pub class: Option<String>,
}

#[component]
pub fn Identity(props: IdentityProps) -> Element {
  let initials = match &props.initials {
    Some(i) => i.clone(),
    None => derive_initials(&props.name),
  };

  let avatar_size = match props.size.as_str() {
    "xs" => "h-4 w-4 text-[8px]",
    "sm" => "h-5 w-5 text-[9px]",
    "default" => "h-6 w-6 text-[10px]",
    "lg" => "h-8 w-8 text-xs",
    _ => "h-6 w-6 text-[10px]",
  };

  let text_size = match props.size.as_str() {
    "xs" => "text-sm",
    "sm" => "text-xs",
    "default" => "text-sm",
    "lg" => "text-sm",
    _ => "text-sm",
  };

  let extra = props.class.as_deref().unwrap_or("");
  let avatar_class = [
    "inline-flex items-center justify-center rounded-full bg-[var(--surface-container-high)] text-[var(--on-surface-variant)] shrink-0 overflow-hidden",
    avatar_size,
  ]
  .join(" ");

  rsx! {
    span { class: "inline-flex items-center gap-1.5", class: "{extra}",
      Avatar { class: avatar_class,
        if let Some(url) = props.avatar_url.clone() {
          AvatarImage {
            src: url,
            alt: Some(props.name.clone()),
            class: "h-full w-full rounded-full object-cover",
          }
        }
        AvatarFallback { class: "inline-flex h-full w-full items-center justify-center",
          "{initials}"
        }
      }
      span { class: "truncate", class: "{text_size}", "{props.name}" }
    }
  }
}

fn derive_initials(name: &str) -> String {
  let words: Vec<&str> = name.split_whitespace().collect();
  match words.len() {
    0 => String::new(),
    1 => words[0].chars().take(2).collect::<String>().to_uppercase(),
    _ => {
      let first = words[0].chars().next().unwrap_or_default();
      let last = words[words.len() - 1].chars().next().unwrap_or_default();
      format!("{first}{last}").to_uppercase()
    },
  }
}
