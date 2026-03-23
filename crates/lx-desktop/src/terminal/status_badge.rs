use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum BadgeVariant {
  Idle,
  Active,
  Running,
}

#[component]
pub fn StatusBadge(label: String, variant: BadgeVariant) -> Element {
  let class = match variant {
    BadgeVariant::Idle => "border border-[var(--primary)] text-[var(--primary)] rounded px-2 py-0.5 text-[10px] uppercase tracking-wider font-semibold",
    BadgeVariant::Active => "bg-[var(--success)] text-[var(--on-primary)] rounded px-2 py-0.5 text-[10px] uppercase tracking-wider font-semibold",
    BadgeVariant::Running => "bg-[var(--warning)] text-[var(--on-primary)] rounded px-2 py-0.5 text-[10px] uppercase tracking-wider font-semibold",
  };
  rsx! {
    span { class, "{label}" }
  }
}
