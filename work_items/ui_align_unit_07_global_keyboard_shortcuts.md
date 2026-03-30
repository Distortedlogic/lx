# Unit 07: Global keyboard shortcuts registration system

## Goal
Expand the keyboard_shortcuts hook into a registration-based system where components can register/unregister shortcuts dynamically, with proper priority ordering so modals and popovers override global shortcuts.

## Preconditions
- No other units required first
- `crates/lx-desktop/src/hooks/keyboard_shortcuts.rs` exists (44 lines)
- `crates/lx-desktop/src/layout/shell.rs` wires `use_keyboard_shortcuts()` to `onkeydown` on the root div (line 76)
- `crates/lx-desktop/src/components/command_palette.rs` has `CommandPaletteOpen` context (line 6)
- `crates/lx-desktop/src/contexts/dialog.rs` has `DialogState` with 5 boolean signals for open dialogs

## Files to Modify
- `crates/lx-desktop/src/hooks/keyboard_shortcuts.rs` (rewrite)
- `crates/lx-desktop/src/layout/shell.rs` (minor update to wiring)
- `crates/lx-desktop/src/components/command_palette.rs` (register its own shortcuts)

## Current State

`keyboard_shortcuts.rs` returns a single `EventHandler<KeyboardEvent>`. It hard-codes all shortcut logic:
- Escape: cascading close (palette > new_issue > new_project > new_agent > onboarding)
- Cmd/Ctrl+K: toggle command palette

`shell.rs` calls `use_keyboard_shortcuts()` and wires the returned handler to the root div's `onkeydown` (line 76).

`command_palette.rs` renders its own input with `autofocus` but does not register any keyboard shortcuts -- the Cmd+K binding lives in `keyboard_shortcuts.rs`.

## Steps

### Step 1: Define the ShortcutEntry struct and ShortcutRegistry context

In `keyboard_shortcuts.rs`, define:

```rust
use std::sync::{Arc, Mutex};
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
    handler: Arc<Mutex<Option<EventHandler<KeyboardEvent>>>>,
}
```

- `id`: unique string per registration (e.g., `"cmd_palette_toggle"`, `"dialog_escape"`)
- `priority`: higher priority entries are checked first
- `matcher`: pure function that returns true if this entry matches the event
- `handler`: the action to run when matched; wrapped in `Arc<Mutex>` for thread safety

### Step 2: Define the ShortcutRegistry context type

```rust
#[derive(Clone)]
pub struct ShortcutRegistry {
    entries: Signal<Vec<ShortcutEntry>>,
}

impl ShortcutRegistry {
    pub fn provide() -> Self {
        let registry = Self { entries: Signal::new(Vec::new()) };
        use_context_provider(|| registry.clone());
        registry
    }
}
```

### Step 3: Add register/unregister methods

```rust
impl ShortcutRegistry {
    pub fn register(
        &self,
        id: &'static str,
        priority: ShortcutPriority,
        matcher: impl Fn(&KeyboardEvent) -> bool + Send + Sync + 'static,
        handler: EventHandler<KeyboardEvent>,
    ) {
        let mut entries = self.entries;
        let entry = ShortcutEntry {
            id,
            priority,
            matcher: Arc::new(matcher),
            handler: Arc::new(Mutex::new(Some(handler))),
        };
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
                if let Ok(guard) = entry.handler.lock() {
                    if let Some(ref handler) = *guard {
                        handler.call(event.clone());
                        return;
                    }
                }
            }
        }
    }
}
```

The `dispatch` method iterates in priority order (highest first). The first matching entry handles the event and stops propagation (early return).

### Step 4: Create matcher helper functions

Add convenience functions for common patterns:

```rust
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
```

### Step 5: Rewrite use_keyboard_shortcuts

Replace the entire function body. It now just creates/retrieves the registry and returns a dispatch handler:

```rust
pub fn use_keyboard_shortcuts() -> (ShortcutRegistry, EventHandler<KeyboardEvent>) {
    let registry = use_context::<ShortcutRegistry>();

    let handler = EventHandler::new(move |event: KeyboardEvent| {
        registry.dispatch(&event);
    });

    (registry, handler)
}
```

### Step 6: Update shell.rs to provide the registry and wire the handler

In `Shell()`, replace:
```rust
let key_handler = use_keyboard_shortcuts();
```

With:
```rust
let _shortcut_registry = ShortcutRegistry::provide();
let (_registry, key_handler) = use_keyboard_shortcuts();
```

The `ShortcutRegistry::provide()` call must come before `use_keyboard_shortcuts()` so the context exists. Add the import:

```rust
use crate::hooks::keyboard_shortcuts::ShortcutRegistry;
```

The `onkeydown` wiring stays unchanged:
```rust
onkeydown: move |e| key_handler.call(e),
```

### Step 7: Register global shortcuts in Shell

After the registry is provided in `Shell()`, register the global Cmd+K and Escape shortcuts. These were previously hard-coded in `use_keyboard_shortcuts`:

```rust
let palette_open_sig = use_context::<CommandPaletteOpen>();
let dialog = use_context::<DialogState>();

use_hook(move || {
    let registry = _shortcut_registry.clone();

    registry.register(
        "global_cmd_k",
        ShortcutPriority::Global,
        key_match(Key::Character("k".into()), true),
        EventHandler::new(move |evt: KeyboardEvent| {
            evt.prevent_default();
            let current = *palette_open_sig.0.read();
            palette_open_sig.0.set(!current);
        }),
    );

    registry.register(
        "global_escape",
        ShortcutPriority::Global,
        escape_match(),
        EventHandler::new(move |_evt: KeyboardEvent| {
            if *dialog.onboarding_open.read() {
                dialog.close_onboarding();
            } else if *dialog.new_agent_open.read() {
                dialog.close_new_agent();
            } else if *dialog.new_project_open.read() {
                dialog.close_new_project();
            } else if *dialog.new_issue_open.read() {
                dialog.close_new_issue();
            }
        }),
    );
});
```

### Step 8: Register command palette Escape at Modal priority

In `CommandPalette` component (`command_palette.rs`), register an Escape handler at `Modal` priority that only activates when the palette is open. This takes priority over the global Escape:

```rust
use crate::hooks::keyboard_shortcuts::{ShortcutRegistry, ShortcutPriority, escape_match};
```

Inside `CommandPalette`:

```rust
let registry = use_context::<ShortcutRegistry>();

use_effect(move || {
    if open() {
        registry.register(
            "cmd_palette_escape",
            ShortcutPriority::Modal,
            escape_match(),
            EventHandler::new(move |_: KeyboardEvent| {
                open.set(false);
            }),
        );
    } else {
        registry.unregister("cmd_palette_escape");
    }
});
```

This replaces the Escape handling that was previously in `keyboard_shortcuts.rs` for the palette.

### Step 9: Add use_shortcut convenience hook

For components that want a simple one-liner registration with automatic cleanup on unmount:

```rust
pub fn use_shortcut(
    id: &'static str,
    priority: ShortcutPriority,
    matcher: impl Fn(&KeyboardEvent) -> bool + Send + Sync + 'static,
    handler: EventHandler<KeyboardEvent>,
) {
    let registry = use_context::<ShortcutRegistry>();

    use_hook(move || {
        registry.register(id, priority, matcher, handler);
    });

    use_drop(move || {
        registry.unregister(id);
    });
}
```

This hook handles the full lifecycle: register on mount, unregister on drop. Components using it do not need to manually call `register`/`unregister`.

### Step 10: Verify imports are correct

In `keyboard_shortcuts.rs`, the imports needed:

```rust
use std::sync::{Arc, Mutex};
use dioxus::prelude::*;
```

No other crate dependencies. `Key`, `KeyboardEvent`, `Signal`, `EventHandler`, `use_context`, `use_context_provider`, `use_hook`, `use_drop`, `use_effect` all come from `dioxus::prelude::*`.

In `shell.rs`, add to imports:
```rust
use crate::hooks::keyboard_shortcuts::{ShortcutRegistry, ShortcutPriority, key_match, escape_match};
```

In `command_palette.rs`, add to imports:
```rust
use crate::hooks::keyboard_shortcuts::{ShortcutRegistry, ShortcutPriority, escape_match};
```

## Verification
1. Run `just diagnose` -- must compile with no errors or warnings
2. Launch the app
3. Press Cmd+K (or Ctrl+K) -- command palette opens
4. Press Cmd+K again -- command palette closes (toggle)
5. Open command palette, press Escape -- palette closes
6. Open the new issue dialog, press Escape -- dialog closes
7. Open command palette AND new issue dialog (if possible), press Escape -- command palette closes first (Modal > Global priority), press Escape again -- dialog closes
8. No shortcut fires twice (early return in dispatch prevents double-handling)
9. `keyboard_shortcuts.rs` stays under 300 lines
10. `shell.rs` stays under 300 lines (currently 235)
11. `command_palette.rs` stays under 300 lines (currently 109)
