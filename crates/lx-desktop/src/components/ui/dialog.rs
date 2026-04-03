use dioxus::prelude::*;

use super::cn;

#[component]
pub fn Dialog(open: Signal<bool>, children: Element) -> Element {
  rsx! {
    div { "data-slot": "dialog", {children} }
  }
}

#[component]
pub fn DialogContent(open: Signal<bool>, #[props(default)] class: String, #[props(default = true)] show_close_button: bool, children: Element) -> Element {
  let init = open();
  let mut visible = use_signal(move || init);
  let mut closing = use_signal(|| false);
  let mut prev_open = use_signal(move || init);
  let mut close_generation = use_signal(|| 0u64);

  if open() && !prev_open() {
    visible.set(true);
    closing.set(false);
    close_generation.set(close_generation() + 1);
  }
  if !open() && prev_open() {
    closing.set(true);
    let generation = close_generation();
    spawn(async move {
      tokio::time::sleep(std::time::Duration::from_millis(200)).await;
      if close_generation() == generation {
        visible.set(false);
        closing.set(false);
      }
    });
  }
  prev_open.set(open());

  if !visible() {
    return rsx! {};
  }

  let mut open = open;
  let overlay_anim = if closing() { "animate-dialog-overlay-out" } else { "animate-dialog-overlay-in" };
  let content_anim = if closing() { "animate-dialog-content-out" } else { "animate-dialog-content-in" };
  rsx! {
    div {
      "data-slot": "dialog-overlay",
      class: "fixed inset-0 z-50 bg-black/50 {overlay_anim}",
      onclick: move |_| open.set(false),
    }
    div {
      "data-slot": "dialog-content",
      role: "dialog",
      "aria-modal": "true",
      tabindex: "0",
      class: cn(
          &[
              &format!(
                  "bg-background fixed top-[50%] left-[50%] z-50 grid w-full max-w-[calc(100%-2rem)] gap-4 rounded-lg border p-6 shadow-lg sm:max-w-lg {content_anim} outline-none",
              ),
              &class,
          ],
      ),
      onmounted: move |evt| {
          let el = evt.data();
          spawn(async move {
              let _ = el.set_focus(true).await;
          });
      },
      onkeydown: move |evt: KeyboardEvent| {
          if evt.key() == Key::Escape {
              evt.stop_propagation();
              open.set(false);
              return;
          }
          if evt.key() == Key::Tab {
              evt.prevent_default();
              let shift = evt.modifiers().shift();
              spawn(async move {
                  let direction = if shift { "backward" } else { "forward" };
                  let js = format!(
                      r#"(function() {{
                                var dialog = document.querySelector('[data-slot="dialog-content"]');
                                if (!dialog) return;
                                var focusable = dialog.querySelectorAll(
                                    'button:not([disabled]), [href], input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])'
                                );
                                if (focusable.length === 0) return;
                                var arr = Array.from(focusable);
                                var idx = arr.indexOf(document.activeElement);
                                if ('{direction}' === 'forward') {{
                                    var next = (idx + 1) % arr.length;
                                    arr[next].focus();
                                }} else {{
                                    var prev = (idx - 1 + arr.length) % arr.length;
                                    arr[prev].focus();
                                }}
                            }})()"#,
                  );
                  let _ = document::eval(&js).await;
              });
          }
      },
      if show_close_button {
        button {
          "data-slot": "dialog-close",
          class: "ring-offset-background focus:ring-ring absolute top-4 right-4 rounded-xs opacity-70 transition-opacity hover:opacity-100 focus:ring-2 focus:ring-offset-2 focus:outline-hidden disabled:pointer-events-none [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4",
          onclick: move |_| open.set(false),
          svg { view_box: "0 0 24 24", class: "size-4",
            line {
              x1: "18",
              y1: "6",
              x2: "6",
              y2: "18",
              stroke: "currentColor",
              stroke_width: "2",
              stroke_linecap: "round",
            }
            line {
              x1: "6",
              y1: "6",
              x2: "18",
              y2: "18",
              stroke: "currentColor",
              stroke_width: "2",
              stroke_linecap: "round",
            }
          }
          span { class: "sr-only", "Close" }
        }
      }
      {children}
    }
  }
}

#[component]
pub fn DialogHeader(#[props(default)] class: String, children: Element) -> Element {
  rsx! {
    div {
      "data-slot": "dialog-header",
      class: cn(&["flex flex-col gap-2 text-center sm:text-left", &class]),
      {children}
    }
  }
}

#[component]
pub fn DialogFooter(#[props(default)] class: String, children: Element) -> Element {
  rsx! {
    div {
      "data-slot": "dialog-footer",
      class: cn(&["flex flex-col-reverse gap-2 sm:flex-row sm:justify-end", &class]),
      {children}
    }
  }
}

#[component]
pub fn DialogTitle(#[props(default)] class: String, children: Element) -> Element {
  rsx! {
    h2 {
      "data-slot": "dialog-title",
      class: cn(&["text-lg leading-none font-semibold", &class]),
      {children}
    }
  }
}

#[component]
pub fn DialogDescription(#[props(default)] class: String, children: Element) -> Element {
  rsx! {
    p {
      "data-slot": "dialog-description",
      class: cn(&["text-muted-foreground text-sm", &class]),
      {children}
    }
  }
}
