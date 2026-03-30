use dioxus::prelude::*;

use crate::components::command_palette::CommandPaletteOpen;
use crate::contexts::dialog::DialogState;

pub fn use_keyboard_shortcuts() -> EventHandler<KeyboardEvent> {
  let mut palette_open = use_context::<CommandPaletteOpen>();
  let dialog = use_context::<DialogState>();

  EventHandler::new(move |event: KeyboardEvent| {
    let key = event.key();
    let modifiers = event.modifiers();
    let cmd_or_ctrl = modifiers.meta() || modifiers.ctrl();

    if key == Key::Escape {
      if *palette_open.0.read() {
        palette_open.0.set(false);
        return;
      }
      if *dialog.new_issue_open.read() {
        dialog.close_new_issue();
        return;
      }
      if *dialog.new_project_open.read() {
        dialog.close_new_project();
        return;
      }
      if *dialog.new_agent_open.read() {
        dialog.close_new_agent();
        return;
      }
      if *dialog.onboarding_open.read() {
        dialog.close_onboarding();
        return;
      }
    }

    if cmd_or_ctrl && key == Key::Character("k".into()) {
      event.prevent_default();
      let current = *palette_open.0.read();
      palette_open.0.set(!current);
    }
  })
}
