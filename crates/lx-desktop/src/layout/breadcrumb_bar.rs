use dioxus::prelude::*;

use crate::contexts::breadcrumb::BreadcrumbState;

#[component]
pub fn BreadcrumbBar() -> Element {
  let bc = use_context::<BreadcrumbState>();
  let crumbs = bc.entries();

  rsx! {
    div { class: "border-b border-gray-700/50 px-6 h-12 shrink-0 flex items-center",
      match crumbs.len() {
          0 => rsx! {},
          1 => rsx! {
            h1 { class: "text-sm font-semibold uppercase tracking-wider truncate", "{crumbs[0].label}" }
          },
          _ => rsx! {
            {
                crumbs
                    .iter()
                    .enumerate()
                    .map(|(i, entry)| {
                        let is_last = i == crumbs.len() - 1;
                        let separator = if i > 0 { Some(rsx! {
                          span { class: "mx-2 text-gray-500", "/" }
                        }) } else { None };
                        if is_last {
                            rsx! {
                              {separator}
                              span { class: "text-sm text-white truncate", "{entry.label}" }
                            }
                        } else if let Some(ref href) = entry.href {
                            rsx! {
                              {separator}
                              Link {
                                to: "{href}",
                                class: "text-sm text-gray-400 hover:text-white transition-colors",
                                "{entry.label}"
                              }
                            }
                        } else {
                            rsx! {
                              {separator}
                              span { class: "text-sm text-gray-400", "{entry.label}" }
                            }
                        }
                    })
            }
          },
      }
    }
  }
}
