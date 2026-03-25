use std::sync::{Arc, Mutex};

use common_pane_tree::{DividerInfo, Pane, PaneNode, Rect, SplitDirection, TabsState};
use dioxus::logger::tracing::error;
use dioxus::prelude::*;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::panes::DesktopPane;
use crate::terminal::tab_bar::TabBar;
use crate::terminal::toolbar::PaneToolbar;
use crate::terminal::use_tabs_state;
use crate::terminal::view::{AgentView, BrowserNavCtx, BrowserView, CanvasView, ChartView, EditorView, TerminalView};

fn create_new_tab(tabs_state: Signal<TabsState<DesktopPane>>) {
  let working_dir = std::env::current_dir().ok().map(|p| p.display().to_string()).unwrap_or_else(|| ".".into());
  let id = Uuid::new_v4().to_string();
  let title = format!("Terminal {}", tabs_state.read().tabs.len() + 1);
  crate::terminal::add_terminal_tab(tabs_state, id, title, working_dir, None);
}

fn split_pane(mut tabs_state: Signal<TabsState<DesktopPane>>, pane_id: &str, direction: SplitDirection) {
  let new_id = Uuid::new_v4().to_string();
  let new_pane = {
    let state = tabs_state.read();
    match state.active_tab() {
      Some(tab) => {
        let panes = tab.root().compute_pane_rects(Rect::default());
        let source = panes.iter().find(|(p, _)| p.pane_id() == pane_id);
        match source.map(|(p, _)| p) {
          Some(DesktopPane::Terminal { working_dir, .. }) => DesktopPane::Terminal { id: new_id, working_dir: working_dir.clone(), command: None, name: None },
          _ => DesktopPane::Terminal { id: new_id, working_dir: ".".into(), command: None, name: None },
        }
      },
      None => DesktopPane::Terminal { id: new_id, working_dir: ".".into(), command: None, name: None },
    }
  };
  tabs_state.write().split_pane(pane_id, direction, PaneNode::Leaf(new_pane));
}

fn close_pane(mut tabs_state: Signal<TabsState<DesktopPane>>, pane_id: &str) {
  tabs_state.write().close_pane_in_active_tab(pane_id);
}

#[component]
pub fn PaneArea() -> Element {
  let tabs_state = use_tabs_state();
  let state = tabs_state.read();
  let tabs = state.tabs.clone();
  let active_tab_id = state.active_tab_id.clone();
  let focused_pane_id = state.focused_pane_id.clone();
  let has_tabs = !tabs.is_empty();
  drop(state);

  rsx! {
    div { class: "flex flex-col h-full",
      div { class: "flex items-center border-b border-[var(--outline-variant)]/15",
        div { class: "flex-1",
          TabBar {
            tabs_state,
            on_new_tab: move |_| create_new_tab(tabs_state),
          }
        }
      }
      if has_tabs {
        div { class: "relative flex-1 min-h-0",
          for tab in tabs.iter() {
            {render_tab(tabs_state, tab, &active_tab_id, &focused_pane_id)}
          }
        }
      } else {
        div { class: "flex flex-1 items-center justify-center text-[var(--outline)]",
          div { class: "text-center",
            p { class: "text-lg mb-2", "No panes open" }
            button {
              class: "bg-gradient-to-r from-[var(--primary)] to-[var(--primary-container)] text-[var(--on-primary)] rounded-md px-4 py-2 text-sm font-medium",
              onclick: move |_| create_new_tab(tabs_state),
              "New Pane"
            }
          }
        }
      }
    }
  }
}

fn render_tab(
  tabs_state: Signal<TabsState<DesktopPane>>,
  tab: &common_pane_tree::Tab<DesktopPane>,
  active_tab_id: &Option<String>,
  focused_pane_id: &Option<String>,
) -> Element {
  let is_active = active_tab_id.as_deref() == Some(tab.id.as_str());
  let visibility = if is_active { "visible" } else { "hidden" };
  let tab_container_id = format!("tab-container-{}", tab.id);
  let full_rect = Rect::default();
  let pane_rects = tab.root().compute_pane_rects(full_rect);
  let dividers = tab.root().compute_dividers(full_rect);

  rsx! {
    div {
      id: "{tab_container_id}",
      key: "{tab.id}",
      class: "absolute inset-0 bg-[var(--surface)]",
      style: "visibility: {visibility};",
      for (pane , rect) in pane_rects.iter() {
        PaneItem {
          key: "{pane.pane_id()}",
          tabs_state,
          pane: DesktopPane::clone(pane),
          rect: *rect,
          focused_pane_id: focused_pane_id.clone(),
        }
      }
      for divider in dividers.iter() {
        {render_divider_item(tabs_state, divider, &tab_container_id)}
      }
    }
  }
}

#[component]
fn PaneItem(mut tabs_state: Signal<TabsState<DesktopPane>>, pane: DesktopPane, rect: Rect, focused_pane_id: Option<String>) -> Element {
  let pid = pane.pane_id().to_string();
  let is_focused = focused_pane_id.as_deref() == Some(pid.as_str());
  let border = if is_focused { "border-[var(--primary)]" } else { "border-[var(--outline-variant)]/30" };
  let pid_focus = pid.clone();
  let pid_sh = pid.clone();
  let pid_sv = pid.clone();
  let pid_close = pid.clone();
  let pid_convert = pid.clone();
  let is_browser = matches!(&pane, DesktopPane::Browser { .. });
  let pane_toolbar = pane.clone();
  let pane_view = pane.clone();

  let current_url: Signal<String> = use_signal(|| match &pane {
    DesktopPane::Browser { url, .. } => url.clone(),
    _ => String::new(),
  });
  let nav_ctx = use_hook(|| {
    let (tx, rx) = mpsc::unbounded_channel::<String>();
    BrowserNavCtx { tx, rx: Arc::new(Mutex::new(Some(rx))), current_url }
  });
  provide_context(nav_ctx.clone());

  let on_nav = if is_browser {
    let tx = nav_ctx.tx.clone();
    Some(EventHandler::new(move |cmd: String| {
      if let Err(e) = tx.send(cmd) {
        error!("nav send failed: {e}");
      }
    }))
  } else {
    None
  };

  rsx! {
    div {
      key: "{pid}",
      class: "group absolute flex flex-col border {border}",
      style: "left: {rect.left}%; top: {rect.top}%; width: {rect.width}%; height: {rect.height}%;",
      onclick: move |_| {
          tabs_state.write().focused_pane_id = Some(pid_focus.clone());
      },
      PaneToolbar {
        pane: pane_toolbar,
        on_split_h: move |_| split_pane(tabs_state, &pid_sh, SplitDirection::Horizontal),
        on_split_v: move |_| split_pane(tabs_state, &pid_sv, SplitDirection::Vertical),
        on_close: move |_| close_pane(tabs_state, &pid_close),
        on_navigate: on_nav,
        on_convert: move |new_node: PaneNode<DesktopPane>| {
            tabs_state.write().convert_pane_in_active_tab(&pid_convert, new_node);
        },
        current_url: ReadSignal::from(current_url),
      }
      div { class: "flex-1 min-h-0", {render_pane_view(&pane_view)} }
    }
  }
}

fn render_pane_view(pane: &DesktopPane) -> Element {
  match pane {
    DesktopPane::Terminal { id, working_dir, command, .. } => rsx! {
      TerminalView {
        terminal_id: id.clone(),
        working_dir: working_dir.clone(),
        command: command.clone(),
      }
    },
    DesktopPane::Browser { id, url, devtools, .. } => rsx! {
      BrowserView {
        browser_id: id.clone(),
        url: url.clone(),
        devtools: *devtools,
      }
    },
    DesktopPane::Editor { id, file_path, language, .. } => rsx! {
      EditorView {
        editor_id: id.clone(),
        file_path: file_path.clone(),
        language: language.clone(),
      }
    },
    DesktopPane::Agent { id, session_id, model, .. } => rsx! {
      AgentView {
        agent_id: id.clone(),
        session_id: session_id.clone(),
        model: model.clone(),
      }
    },
    DesktopPane::Canvas { id, widget_type, config, .. } => rsx! {
      CanvasView {
        canvas_id: id.clone(),
        widget_type: widget_type.clone(),
        config: config.clone(),
      }
    },
    DesktopPane::Chart { id, chart_json, title, .. } => rsx! {
      ChartView {
        chart_id: id.clone(),
        chart_json: chart_json.clone(),
        title: title.clone(),
      }
    },
  }
}

fn render_divider_item(mut tabs_state: Signal<TabsState<DesktopPane>>, divider: &DividerInfo, container_id: &str) -> Element {
  let (div_class, div_style) = match divider.direction {
    SplitDirection::Horizontal => (
      "absolute z-20 cursor-col-resize hover:bg-[var(--primary)]/50 bg-[var(--surface-bright)]",
      format!("left: {}%; top: {}%; width: 4px; height: {}%;", divider.rect.left, divider.rect.top, divider.rect.height,),
    ),
    SplitDirection::Vertical => (
      "absolute z-20 cursor-row-resize hover:bg-[var(--primary)]/50 bg-[var(--surface-bright)]",
      format!("left: {}%; top: {}%; width: {}%; height: 4px;", divider.rect.left, divider.rect.top, divider.rect.width,),
    ),
  };
  let split_id = divider.split_id.clone();
  let parent_rect = divider.parent_rect;
  let direction = divider.direction;
  let cid = container_id.to_owned();

  rsx! {
    div {
      key: "div-{split_id}",
      class: "{div_class}",
      style: "{div_style}",
      onmousedown: move |evt| {
          evt.stop_propagation();
          let sid = split_id.clone();
          let cid = cid.clone();
          let is_h = matches!(direction, SplitDirection::Horizontal);
          let p_start = if is_h { parent_rect.left } else { parent_rect.top };
          let p_size = if is_h { parent_rect.width } else { parent_rect.height };
          spawn(async move {
              let mut eval = document::eval("WidgetBridge.runDividerDrag(dioxus)");
              let _ = eval
                  .send(
                      serde_json::json!(
                          { "containerId" : cid, "direction" : if is_h { "horizontal" }
                          else { "vertical" }, "parentStart" : p_start, "parentSize" :
                          p_size }
                      ),
                  );
              while let Ok(msg) = eval.recv::<serde_json::Value>().await {
                  match msg["type"].as_str() {
                      Some("ratio") => {
                          if let Some(v) = msg["value"].as_f64() {
                              tabs_state.write().set_active_tab_ratio(&sid, v);
                          }
                      }
                      _ => break,
                  }
              }
          });
      },
    }
  }
}
