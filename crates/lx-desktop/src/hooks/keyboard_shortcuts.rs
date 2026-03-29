use dioxus::prelude::*;

pub struct ShortcutHandlers {
  pub on_new_issue: Option<EventHandler<()>>,
  pub on_toggle_sidebar: Option<EventHandler<()>>,
  pub on_toggle_panel: Option<EventHandler<()>>,
}

pub fn use_keyboard_shortcuts(handlers: &ShortcutHandlers) -> EventHandler<KeyboardEvent> {
  let input_focused = use_signal(|| false);
  let on_new_issue = handlers.on_new_issue;
  let on_toggle_sidebar = handlers.on_toggle_sidebar;
  let on_toggle_panel = handlers.on_toggle_panel;

  EventHandler::new(move |event: KeyboardEvent| {
    if input_focused() {
      return;
    }

    let key = event.key();
    let modifiers = event.modifiers();
    let has_modifier = modifiers.ctrl() || modifiers.meta() || modifiers.alt();

    if !has_modifier {
      match key {
        Key::Character(ref c) if c == "c" => {
          if let Some(ref handler) = on_new_issue {
            handler.call(());
          }
        },
        Key::Character(ref c) if c == "[" => {
          if let Some(ref handler) = on_toggle_sidebar {
            handler.call(());
          }
        },
        Key::Character(ref c) if c == "]" => {
          if let Some(ref handler) = on_toggle_panel {
            handler.call(());
          }
        },
        _ => {},
      }
    }
  })
}
