use dioxus::prelude::*;

use super::cn;

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum ButtonVariant {
  #[default]
  Default,
  Destructive,
  Outline,
  Secondary,
  Ghost,
  Link,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum ButtonSize {
  #[default]
  Default,
  Xs,
  Sm,
  Lg,
  Icon,
  IconXs,
  IconSm,
  IconLg,
}

const BASE_BUTTON_CLASS: &str = "inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md text-sm font-medium transition-[color,background-color,border-color,box-shadow,opacity] disabled:pointer-events-none disabled:opacity-50 [&_svg]:pointer-events-none [&_svg:not([class*='size-'])]:size-4 shrink-0 [&_svg]:shrink-0 outline-none focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-[3px] aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive";

fn variant_class(variant: ButtonVariant) -> &'static str {
  match variant {
    ButtonVariant::Default => "bg-primary text-primary-foreground hover:bg-primary/90",
    ButtonVariant::Destructive => {
      "bg-destructive text-white hover:bg-destructive/90 focus-visible:ring-destructive/20 dark:focus-visible:ring-destructive/40 dark:bg-destructive/60"
    },
    ButtonVariant::Outline => {
      "border bg-background shadow-xs hover:bg-accent hover:text-accent-foreground dark:bg-input/30 dark:border-input dark:hover:bg-input/50"
    },
    ButtonVariant::Secondary => "bg-secondary text-secondary-foreground hover:bg-secondary/80",
    ButtonVariant::Ghost => "hover:bg-accent hover:text-accent-foreground dark:hover:bg-accent/50",
    ButtonVariant::Link => "text-primary underline-offset-4 hover:underline",
  }
}

fn size_class(size: ButtonSize) -> &'static str {
  match size {
    ButtonSize::Default => "h-10 px-4 py-2 has-[>svg]:px-3",
    ButtonSize::Xs => "h-6 gap-1 rounded-md px-2 text-xs has-[>svg]:px-1.5 [&_svg:not([class*='size-'])]:size-3",
    ButtonSize::Sm => "h-9 rounded-md gap-1.5 px-3 has-[>svg]:px-2.5",
    ButtonSize::Lg => "h-10 rounded-md px-6 has-[>svg]:px-4",
    ButtonSize::Icon => "size-10",
    ButtonSize::IconXs => "size-6 rounded-md [&_svg:not([class*='size-'])]:size-3",
    ButtonSize::IconSm => "size-9",
    ButtonSize::IconLg => "size-10",
  }
}

pub fn button_variant_class(variant: ButtonVariant, size: ButtonSize) -> String {
  cn(&[BASE_BUTTON_CLASS, variant_class(variant), size_class(size)])
}

fn variant_name(variant: ButtonVariant) -> &'static str {
  match variant {
    ButtonVariant::Default => "default",
    ButtonVariant::Destructive => "destructive",
    ButtonVariant::Outline => "outline",
    ButtonVariant::Secondary => "secondary",
    ButtonVariant::Ghost => "ghost",
    ButtonVariant::Link => "link",
  }
}

fn size_name(size: ButtonSize) -> &'static str {
  match size {
    ButtonSize::Default => "default",
    ButtonSize::Xs => "xs",
    ButtonSize::Sm => "sm",
    ButtonSize::Lg => "lg",
    ButtonSize::Icon => "icon",
    ButtonSize::IconXs => "icon-xs",
    ButtonSize::IconSm => "icon-sm",
    ButtonSize::IconLg => "icon-lg",
  }
}

#[component]
pub fn Button(
  #[props(default)] variant: ButtonVariant,
  #[props(default)] size: ButtonSize,
  #[props(default)] class: String,
  #[props(default)] disabled: bool,
  #[props(default)] r#type: String,
  children: Element,
) -> Element {
  let btn_type = if r#type.is_empty() { "button" } else { &r#type };
  let classes = cn(&[&button_variant_class(variant, size), &class]);
  rsx! {
    button {
      "data-slot": "button",
      "data-variant": variant_name(variant),
      "data-size": size_name(size),
      class: "{classes}",
      disabled,
      r#type: "{btn_type}",
      {children}
    }
  }
}
