# UNIT 13: Keyboard Shortcuts + Schedule Description

## Goal

Two independent fixes:
A) Wire the existing keyboard shortcuts hook into the app shell, adding Cmd+K for command palette and Escape to close dialogs.
B) Add a `describe_schedule` function that produces human-readable cron descriptions, displayed in the schedule editor.

## Files Modified

| File | Action |
|------|--------|
| `crates/lx-desktop/src/hooks/keyboard_shortcuts.rs` | Rewrite |
| `crates/lx-desktop/src/layout/shell.rs` | Edit (add keyboard listener) |
| `crates/lx-desktop/src/pages/routines/cron_utils.rs` | Edit (add describe_schedule) |
| `crates/lx-desktop/src/pages/routines/schedule_editor.rs` | Edit (display description) |

## Part A: Keyboard Shortcuts

### Current State

`keyboard_shortcuts.rs` (45 lines) defines a `ShortcutHandlers` struct and `use_keyboard_shortcuts` function that returns an `EventHandler<KeyboardEvent>`. It is never called anywhere. The `CommandPalette` component in `command_palette.rs` (110 lines) provides `CommandPaletteOpen(Signal<bool>)` via context on line 38.

`shell.rs` (229 lines) does not reference keyboard_shortcuts at all. `DialogState` in `contexts/dialog.rs` has boolean signals: `new_issue_open`, `new_project_open`, `new_agent_open`, `onboarding_open`.

### Step A1: Replace `crates/lx-desktop/src/hooks/keyboard_shortcuts.rs`

Replace the full file content with:

```rust
use dioxus::prelude::*;

use crate::components::command_palette::CommandPaletteOpen;
use crate::contexts::dialog::DialogState;

pub fn use_keyboard_shortcuts() {
  let mut palette_open = use_context::<CommandPaletteOpen>();
  let dialog = use_context::<DialogState>();

  use_global_shortcut(move |event: Event<KeyboardData>| {
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
  });
}
```

**Note on `use_global_shortcut`:** Dioxus does not provide a built-in `use_global_shortcut`. If this function does not exist, use the following alternative implementation instead:

```rust
use dioxus::prelude::*;

use crate::components::command_palette::CommandPaletteOpen;
use crate::contexts::dialog::DialogState;

pub fn use_keyboard_shortcuts() {
  let mut palette_open = use_context::<CommandPaletteOpen>();
  let dialog = use_context::<DialogState>();

  use_effect(move || {
    let handler = document::eval(r#"
      document.addEventListener("keydown", function(e) {
        dioxus.send(JSON.stringify({key: e.key, ctrl: e.ctrlKey, meta: e.metaKey}));
      });
    "#);
    spawn(async move {
      loop {
        if let Ok(val) = handler.recv::<serde_json::Value>().await {
          let key = val.get("key").and_then(|v| v.as_str()).unwrap_or("");
          let ctrl = val.get("ctrl").and_then(|v| v.as_bool()).unwrap_or(false);
          let meta = val.get("meta").and_then(|v| v.as_bool()).unwrap_or(false);
          let cmd_or_ctrl = ctrl || meta;

          if key == "Escape" {
            if *palette_open.0.read() {
              palette_open.0.set(false);
              continue;
            }
            if *dialog.new_issue_open.read() {
              dialog.close_new_issue();
              continue;
            }
            if *dialog.new_project_open.read() {
              dialog.close_new_project();
              continue;
            }
            if *dialog.new_agent_open.read() {
              dialog.close_new_agent();
              continue;
            }
            if *dialog.onboarding_open.read() {
              dialog.close_onboarding();
              continue;
            }
          }

          if cmd_or_ctrl && key == "k" {
            let current = *palette_open.0.read();
            palette_open.0.set(!current);
          }
        }
      }
    });
  });
}
```

The implementing agent must check which approach compiles against the project's Dioxus version. Try the `document::eval` approach first since it is confirmed to work in Dioxus desktop. If `handler.recv` does not exist, use `document::eval` with an `onkeydown` handler on the root div in shell.rs instead:

### Step A2 (fallback): Add onkeydown to shell.rs root div

If neither `use_global_shortcut` nor `document::eval` with `recv` works, use the onkeydown approach on the root div.

Replace `keyboard_shortcuts.rs` with:

```rust
use dioxus::prelude::*;

use crate::components::command_palette::CommandPaletteOpen;
use crate::contexts::dialog::DialogState;

pub fn handle_global_keydown(event: Event<KeyboardData>) {
  let palette_open = use_context::<CommandPaletteOpen>();
  let dialog = use_context::<DialogState>();

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
}
```

And in `shell.rs`, edit the root div:

Find:
```rust
    div { class: "relative h-screen overflow-hidden bg-[var(--surface)] text-[var(--on-surface)] flex flex-col",
```

Replace with:
```rust
    div {
      class: "relative h-screen overflow-hidden bg-[var(--surface)] text-[var(--on-surface)] flex flex-col",
      tabindex: "0",
      onkeydown: crate::hooks::keyboard_shortcuts::handle_global_keydown,
```

**Important:** The implementing agent must try all three approaches in order (use_global_shortcut, document::eval with recv, onkeydown on root div) and use whichever compiles first. The `onkeydown` approach is the most portable fallback. However, `handle_global_keydown` cannot call `use_context` because it is not a hook context -- it would need to receive the signals as parameters. Here is the correct fallback:

### Step A2 (definitive fallback): onkeydown with closure in shell.rs

Replace `keyboard_shortcuts.rs` with:

```rust
use dioxus::prelude::*;

use crate::components::command_palette::CommandPaletteOpen;
use crate::contexts::dialog::DialogState;

pub fn use_keyboard_shortcuts() -> EventHandler<KeyboardEvent> {
  let palette_open = use_context::<CommandPaletteOpen>();
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
```

And in `shell.rs`:

Add import at top of file, after existing use statements (after line 20):

Find:
```rust
use crate::terminal::{add_tab, use_provide_tabs};
```

Replace with:
```rust
use crate::hooks::keyboard_shortcuts::use_keyboard_shortcuts;
use crate::terminal::{add_tab, use_provide_tabs};
```

Add the hook call and onkeydown handler. Find (inside the Shell function, before the rsx! macro):

Find:
```rust
  spawn_terminal_listener(tabs_state, &spawn_channel.1);
  rsx! {
    div { class: "relative h-screen overflow-hidden bg-[var(--surface)] text-[var(--on-surface)] flex flex-col",
```

Replace with:
```rust
  spawn_terminal_listener(tabs_state, &spawn_channel.1);
  let key_handler = use_keyboard_shortcuts();
  rsx! {
    div {
      class: "relative h-screen overflow-hidden bg-[var(--surface)] text-[var(--on-surface)] flex flex-col",
      tabindex: "0",
      onkeydown: move |e| key_handler.call(e),
```

## Part B: Schedule Description

### Current State

`cron_utils.rs` (77 lines) has `parse_cron_to_preset` and `build_cron` functions. No human-readable description exists.

`schedule_editor.rs` (280 lines) renders the preset selector and time pickers. No description text is shown.

### Step B1: Add `describe_schedule` to `crates/lx-desktop/src/pages/routines/cron_utils.rs`

Append at the end of the file (after line 77):

Find:
```rust
    SchedulePreset::Custom => String::new(),
  }
}
```

Replace with:
```rust
    SchedulePreset::Custom => String::new(),
  }
}

pub fn describe_schedule(cron: &str) -> String {
  let parsed = parse_cron_to_preset(cron);
  let h: u32 = parsed.hour.parse().unwrap_or(0);
  let m: u32 = parsed.minute.parse().unwrap_or(0);
  let time_str = format_time(h, m);

  match parsed.preset {
    SchedulePreset::EveryMinute => "Every minute".to_string(),
    SchedulePreset::EveryHour => format!("Every hour at minute {m:02}"),
    SchedulePreset::EveryDay => format!("Every day at {time_str}"),
    SchedulePreset::Weekdays => format!("Weekdays at {time_str}"),
    SchedulePreset::Weekly => {
      let dow_name = match parsed.day_of_week.as_str() {
        "0" => "Sunday",
        "1" => "Monday",
        "2" => "Tuesday",
        "3" => "Wednesday",
        "4" => "Thursday",
        "5" => "Friday",
        "6" => "Saturday",
        _ => "Monday",
      };
      format!("Every {dow_name} at {time_str}")
    },
    SchedulePreset::Monthly => {
      let dom: u32 = parsed.day_of_month.parse().unwrap_or(1);
      let suffix = match dom {
        1 | 21 | 31 => "st",
        2 | 22 => "nd",
        3 | 23 => "rd",
        _ => "th",
      };
      format!("Monthly on the {dom}{suffix} at {time_str}")
    },
    SchedulePreset::Custom => {
      let trimmed = cron.trim();
      if trimmed.is_empty() {
        "No schedule set".to_string()
      } else {
        format!("Custom: {trimmed}")
      }
    },
  }
}

fn format_time(h: u32, m: u32) -> String {
  let (display_h, period) = match h {
    0 => (12, "AM"),
    1..=11 => (h, "AM"),
    12 => (12, "PM"),
    _ => (h - 12, "PM"),
  };
  if m == 0 {
    format!("{display_h}:{m:02} {period}")
  } else {
    format!("{display_h}:{m:02} {period}")
  }
}
```

### Step B2: Display description in `crates/lx-desktop/src/pages/routines/schedule_editor.rs`

Add the import. Find:

```rust
use super::cron_utils::{build_cron, parse_cron_to_preset};
```

Replace with:

```rust
use super::cron_utils::{build_cron, describe_schedule, parse_cron_to_preset};
```

Add the description display below the preset selector. Find:

```rust
    div { class: "flex flex-col gap-3",
      select {
```

Replace with:

```rust
    div { class: "flex flex-col gap-3",
      p { class: "text-xs text-[var(--outline)] italic",
        "{describe_schedule(&value)}"
      }
      select {
```

## Verification

Run `just diagnose` and confirm no compiler errors in `crates/lx-desktop`.
