use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum BadgeVariant {
  Idle,
  Active,
}

#[component]
pub fn StatusBadge(label: String, variant: BadgeVariant) -> Element {
  let class = match variant {
    BadgeVariant::Idle => "border border-[var(--outline)] text-[var(--outline)] rounded px-2 py-0.5 text-[10px] uppercase tracking-wider font-semibold",
    BadgeVariant::Active => "bg-[var(--success)] text-[var(--on-primary)] rounded px-2 py-0.5 text-[10px] uppercase tracking-wider font-semibold",
  };
  rsx! {
    span { class, "{label}" }
  }
}
