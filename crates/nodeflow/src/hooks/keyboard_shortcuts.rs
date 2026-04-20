use std::sync::Arc;

use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ShortcutPriority {
  Global = 0,
  Page = 1,
  Panel = 2,
  Modal = 3,
  Overlay = 4,
}

#[derive(Clone)]
struct ShortcutEntry {
  id: &'static str,
  priority: ShortcutPriority,
  matcher: Arc<dyn Fn(&KeyboardEvent) -> bool + Send + Sync>,
  handler: Callback<KeyboardEvent>,
}

#[derive(Clone, Copy)]
pub struct ShortcutRegistry {
  entries: Signal<Vec<ShortcutEntry>>,
}

impl ShortcutRegistry {
  pub fn provide() -> Self {
    let registry = Self { entries: Signal::new(Vec::new()) };
    use_context_provider(|| registry);
    registry
  }

  pub fn register(
    &self,
    id: &'static str,
    priority: ShortcutPriority,
    matcher: impl Fn(&KeyboardEvent) -> bool + Send + Sync + 'static,
    handler: Callback<KeyboardEvent>,
  ) {
    let mut entries = self.entries;
    let entry = ShortcutEntry { id, priority, matcher: Arc::new(matcher), handler };
    entries.write().push(entry);
    entries.write().sort_by(|a, b| b.priority.cmp(&a.priority));
  }

  pub fn unregister(&self, id: &'static str) {
    let mut entries = self.entries;
    entries.write().retain(|e| e.id != id);
  }

  pub fn dispatch(&self, event: &KeyboardEvent) {
    let entries = self.entries.read();
    for entry in entries.iter() {
      if (entry.matcher)(event) {
        entry.handler.call(event.clone());
        return;
      }
    }
  }
}

pub fn key_match(key: Key, cmd: bool) -> impl Fn(&KeyboardEvent) -> bool + Send + Sync + 'static {
  move |evt: &KeyboardEvent| {
    let mods = evt.modifiers();
    let cmd_held = mods.meta() || mods.ctrl();
    evt.key() == key && cmd_held == cmd
  }
}

pub fn escape_match() -> impl Fn(&KeyboardEvent) -> bool + Send + Sync + 'static {
  move |evt: &KeyboardEvent| evt.key() == Key::Escape
}

pub fn use_keyboard_shortcuts() -> (ShortcutRegistry, EventHandler<KeyboardEvent>) {
  let registry = use_context::<ShortcutRegistry>();

  let handler = EventHandler::new(move |event: KeyboardEvent| {
    registry.dispatch(&event);
  });

  (registry, handler)
}

pub fn use_shortcut(
  id: &'static str,
  priority: ShortcutPriority,
  matcher: impl Fn(&KeyboardEvent) -> bool + Send + Sync + 'static,
  handler: Callback<KeyboardEvent>,
) {
  let registry = use_context::<ShortcutRegistry>();

  use_hook(move || {
    registry.register(id, priority, matcher, handler);
  });

  use_drop(move || {
    registry.unregister(id);
  });
}
