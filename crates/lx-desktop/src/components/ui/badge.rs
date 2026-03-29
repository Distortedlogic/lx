use dioxus::prelude::*;

use super::cn;

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum BadgeVariant {
  #[default]
  Default,
  Secondary,
  Destructive,
  Outline,
  Ghost,
  Link,
}

const BASE_BADGE_CLASS: &str = "inline-flex items-center justify-center rounded-full border border-transparent px-2 py-0.5 text-xs font-medium w-fit whitespace-nowrap shrink-0 [&>svg]:size-3 gap-1 [&>svg]:pointer-events-none focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-[3px] aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive transition-[color,box-shadow] overflow-hidden";

fn variant_class(variant: BadgeVariant) -> &'static str {
  match variant {
    BadgeVariant::Default => "bg-primary text-primary-foreground [a&]:hover:bg-primary/90",
    BadgeVariant::Secondary => "bg-secondary text-secondary-foreground [a&]:hover:bg-secondary/90",
    BadgeVariant::Destructive => {
      "bg-destructive text-white [a&]:hover:bg-destructive/90 focus-visible:ring-destructive/20 dark:focus-visible:ring-destructive/40 dark:bg-destructive/60"
    },
    BadgeVariant::Outline => "border-border text-foreground [a&]:hover:bg-accent [a&]:hover:text-accent-foreground",
    BadgeVariant::Ghost => "[a&]:hover:bg-accent [a&]:hover:text-accent-foreground",
    BadgeVariant::Link => "text-primary underline-offset-4 [a&]:hover:underline",
  }
}

pub fn badge_variant_class(variant: BadgeVariant) -> String {
  cn(&[BASE_BADGE_CLASS, variant_class(variant)])
}

fn variant_name(variant: BadgeVariant) -> &'static str {
  match variant {
    BadgeVariant::Default => "default",
    BadgeVariant::Secondary => "secondary",
    BadgeVariant::Destructive => "destructive",
    BadgeVariant::Outline => "outline",
    BadgeVariant::Ghost => "ghost",
    BadgeVariant::Link => "link",
  }
}

#[component]
pub fn Badge(#[props(default)] variant: BadgeVariant, #[props(default)] class: String, children: Element) -> Element {
  let classes = cn(&[&badge_variant_class(variant), &class]);
  rsx! {
    span {
      "data-slot": "badge",
      "data-variant": variant_name(variant),
      class: "{classes}",
      {children}
    }
  }
}
