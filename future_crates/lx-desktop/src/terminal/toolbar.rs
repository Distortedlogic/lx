use dioxus::prelude::*;
use lx_ui::pane_tree::{PaneKind, PaneNode};
use uuid::Uuid;

#[component]
pub fn PaneToolbar(
  pane_node: PaneNode,
  on_split_h: EventHandler,
  on_split_v: EventHandler,
  on_close: EventHandler,
  on_navigate: Option<EventHandler<String>>,
  on_convert: EventHandler<PaneNode>,
) -> Element {
  let initial_url = match &pane_node {
    PaneNode::Browser { url, .. } => url.clone(),
    _ => String::new(),
  };
  let mut url_input = use_signal(|| initial_url);
  let mut conversion_open = use_signal(|| false);
  let current_kind = pane_node.pane_kind();

  let (icon, _icon_label) = match &pane_node {
    PaneNode::Terminal { .. } => ("\u{25B8}", "Terminal"),
    PaneNode::Browser { .. } => ("\u{1F310}", "Browser"),
    PaneNode::Editor { .. } => ("\u{25C7}", "Editor"),
    PaneNode::Agent { .. } => ("\u{25CF}", "Agent"),
    PaneNode::Canvas { .. } => ("\u{25FB}", "Canvas"),
    PaneNode::Chart { .. } => ("\u{25A3}", "Chart"),
    PaneNode::FlowGraph { .. } => ("\u{25C8}", "Flow Graph"),
    PaneNode::Split { .. } => unreachable!(),
  };

  let all_kinds = [
    (PaneKind::Terminal, "\u{25B8}", "Terminal"),
    (PaneKind::Browser, "\u{1F310}", "Browser"),
    (PaneKind::Editor, "\u{25C7}", "Editor"),
    (PaneKind::Agent, "\u{25CF}", "Agent"),
    (PaneKind::Canvas, "\u{25FB}", "Canvas"),
    (PaneKind::FlowGraph, "\u{25C8}", "Flow Graph"),
  ];

  let left_section = match &pane_node {
    PaneNode::Terminal { working_dir, .. } => {
      let truncated = truncate_path(working_dir, 2);
      rsx! { span { class: "truncate", "{truncated}" } }
    },
    PaneNode::Browser { .. } => {
      let nav = on_navigate;
      let nav2 = on_navigate;
      let nav3 = on_navigate;
      let nav4 = on_navigate;
      rsx! {
          button {
              class: "px-1.5 py-0.5 bg-gray-700 rounded hover:bg-gray-600",
              onclick: move |_| { if let Some(ref h) = nav { h.call("back".into()); } },
              "\u{2190}"
          }
          button {
              class: "px-1.5 py-0.5 bg-gray-700 rounded hover:bg-gray-600",
              onclick: move |_| { if let Some(ref h) = nav2 { h.call("forward".into()); } },
              "\u{2192}"
          }
          button {
              class: "px-1.5 py-0.5 bg-gray-700 rounded hover:bg-gray-600",
              onclick: move |_| { if let Some(ref h) = nav3 { h.call("refresh".into()); } },
              "\u{21BB}"
          }
          input {
              class: "flex-1 bg-gray-900 border border-gray-600 rounded text-xs px-1.5 py-0.5",
              value: "{url_input}",
              oninput: move |evt| url_input.set(evt.value()),
              onkeypress: move |evt: KeyboardEvent| {
                  if evt.key() == Key::Enter && let Some(ref h) = nav4 {
                      h.call(url_input());
                  }
              },
          }
      }
    },
    PaneNode::Editor { file_path, .. } => {
      let basename = file_path.rsplit('/').next().unwrap_or(file_path);
      rsx! { span { class: "truncate", "{basename}" } }
    },
    PaneNode::Agent { model, .. } => {
      rsx! { span { class: "truncate", "{model}" } }
    },
    PaneNode::Canvas { widget_type, .. } => {
      rsx! { span { class: "truncate", "{widget_type}" } }
    },
    PaneNode::Chart { title, .. } => {
      let label = title.as_deref().unwrap_or("Chart");
      rsx! { span { class: "truncate", "{label}" } }
    },
    PaneNode::FlowGraph { source_path, .. } => {
      let basename = source_path.rsplit('/').next().unwrap_or(source_path);
      rsx! { span { class: "truncate", "{basename}" } }
    },
    PaneNode::Split { .. } => unreachable!(),
  };

  rsx! {
      div {
          class: "flex items-center h-8 px-2 gap-1 bg-gray-800 border-b border-gray-700 opacity-0 group-hover:opacity-100 transition-opacity text-xs shrink-0",
          div {
              class: "relative",
              span {
                  class: "cursor-pointer hover:opacity-70",
                  onclick: move |evt| {
                      evt.stop_propagation();
                      conversion_open.set(!conversion_open());
                  },
                  "{icon}"
              }
              if conversion_open() {
                  div {
                      class: "absolute top-full left-0 z-30 mt-1 py-1 bg-gray-800 border border-gray-600 rounded-md shadow-lg min-w-36",
                      for (kind, kind_icon, kind_name) in all_kinds.iter() {
                          if Some(*kind) != current_kind {
                              {
                                  let kind = *kind;
                                  let kind_icon = *kind_icon;
                                  let kind_name = *kind_name;
                                  rsx! {
                                      button {
                                          class: "flex items-center gap-2 w-full px-3 py-1.5 text-left hover:bg-gray-700",
                                          onclick: move |evt| {
                                              evt.stop_propagation();
                                              let new_id = Uuid::new_v4().to_string();
                                              let new_node = make_default_pane(kind, new_id);
                                              on_convert.call(new_node);
                                              conversion_open.set(false);
                                          },
                                          span { "{kind_icon}" }
                                          span { "{kind_name}" }
                                      }
                                  }
                              }
                          }
                      }
                  }
              }
          }
          {left_section}
          div { class: "flex-1" }
          button {
              class: "px-1.5 py-0.5 bg-gray-700 rounded hover:bg-gray-600",
              onclick: move |evt| {
                  evt.stop_propagation();
                  on_split_h.call(());
              },
              "\u{21E5}"
          }
          button {
              class: "px-1.5 py-0.5 bg-gray-700 rounded hover:bg-gray-600",
              onclick: move |evt| {
                  evt.stop_propagation();
                  on_split_v.call(());
              },
              "\u{21E4}"
          }
          button {
              class: "px-1.5 py-0.5 bg-gray-700 rounded hover:bg-gray-600",
              onclick: move |evt| {
                  evt.stop_propagation();
                  on_close.call(());
              },
              "\u{00D7}"
          }
      }
  }
}

fn make_default_pane(kind: PaneKind, id: String) -> PaneNode {
  match kind {
    PaneKind::Terminal => PaneNode::Terminal { id, working_dir: ".".into(), command: None },
    PaneKind::Browser => PaneNode::Browser { id, url: "about:blank".into(), devtools: false },
    PaneKind::Editor => PaneNode::Editor { id, file_path: String::new(), language: None },
    PaneKind::Agent => PaneNode::Agent { id: id.clone(), session_id: Uuid::new_v4().to_string(), model: "claude-sonnet-4-6".into() },
    PaneKind::Canvas => PaneNode::Canvas { id, widget_type: "markdown".into(), config: serde_json::Value::Object(Default::default()) },
    PaneKind::Chart => PaneNode::Chart { id, chart_json: String::new(), title: None },
    PaneKind::FlowGraph => PaneNode::FlowGraph { id, source_path: String::new() },
  }
}

fn truncate_path(path: &str, components: usize) -> String {
  let parts: Vec<&str> = path.rsplitn(components + 1, '/').collect();
  if parts.len() <= components { path.to_string() } else { parts[..components].iter().rev().copied().collect::<Vec<_>>().join("/") }
}
