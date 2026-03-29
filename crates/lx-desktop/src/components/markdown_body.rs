use dioxus::prelude::*;
use pulldown_cmark::{Options, Parser, html};

#[component]
pub fn MarkdownBody(content: String, #[props(optional)] class: Option<String>) -> Element {
  let extra = class.as_deref().unwrap_or("");
  let parser = Parser::new_ext(&content, Options::all());
  let mut html_output = String::new();
  html::push_html(&mut html_output, parser);

  rsx! {
    div {
      class: "prose prose-sm prose-invert max-w-none break-words overflow-hidden {extra}",
      dangerous_inner_html: "{html_output}",
    }
  }
}
