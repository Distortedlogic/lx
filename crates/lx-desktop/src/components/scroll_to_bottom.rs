use dioxus::prelude::*;

#[component]
pub fn ScrollToBottom(children: Element, #[props(optional)] class: Option<String>) -> Element {
  let extra = class.as_deref().unwrap_or("");
  let mut user_scrolled_up = use_signal(|| false);
  let mut render_count = use_signal(|| 0u64);
  let sentinel_id = "scroll-sentinel";

  render_count += 1;

  use_effect(move || {
    let _tick = render_count();
    if !user_scrolled_up() {
      spawn(async move {
        let js = format!("document.getElementById('{sentinel_id}')?.scrollIntoView({{ behavior: 'smooth' }})");
        let _ = document::eval(&js).await;
      });
    }
  });

  rsx! {
    div {
      class: "overflow-y-auto relative",
      class: "{extra}",
      onscroll: move |_evt| {
          spawn(async move {
              let js = format!(
                  r#"(function() {{
                                                                                                            var el = document.getElementById('{sentinel_id}');
                                                                                                            if (!el || !el.parentElement) return 'false';
                                                                                                            var parent = el.parentElement;
                                                                                                            var diff = parent.scrollHeight - parent.scrollTop - parent.clientHeight;
                                                                                                            return diff > 40 ? 'true' : 'false';
                                                                                                          }})()"#,
              );
              if let Ok(val) = document::eval(&js).await {
                  let is_up = val.to_string().contains("true");
                  user_scrolled_up.set(is_up);
              }
          });
      },
      {children}
      div { id: sentinel_id }
    }
  }
}
