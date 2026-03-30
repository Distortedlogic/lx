# UNIT 13: Keyboard Shortcuts + Schedule Description

## Goal

Two independent fixes:
A) Wire the existing keyboard shortcuts hook into the app shell, adding Cmd+K for command palette and Escape to close dialogs.
B) Add a `describe_schedule` function that produces human-readable cron descriptions, displayed in the schedule editor.

## Files Modified

| File | Action |
|------|--------|
| `crates/lx-desktop/src/hooks/keyboard_shortcuts.rs` | Rewrite |
| `crates/lx-desktop/src/layout/shell.rs` | Edit (add keyboard listener, provide CommandPaletteOpen context) |
| `crates/lx-desktop/src/components/command_palette.rs` | Edit (consume context instead of providing it) |
| `crates/lx-desktop/src/pages/routines/cron_utils.rs` | Edit (add describe_schedule) |
| `crates/lx-desktop/src/pages/routines/schedule_editor.rs` | Edit (display description) |

## Part A: Keyboard Shortcuts

### Current State

`keyboard_shortcuts.rs` (45 lines) defines a `ShortcutHandlers` struct and `use_keyboard_shortcuts` function that returns an `EventHandler<KeyboardEvent>`. It is never called anywhere. The `CommandPalette` component in `command_palette.rs` (110 lines) provides `CommandPaletteOpen(Signal<bool>)` via context on line 38.

`shell.rs` (229 lines) does not reference keyboard_shortcuts at all. `DialogState` in `contexts/dialog.rs` has boolean signals: `new_issue_open`, `new_project_open`, `new_agent_open`, `onboarding_open`.

`CommandPaletteOpen` is currently provided inside `CommandPalette`, which is a child of `Shell` (rendered at shell.rs line 105). Since `use_context` requires the context to come from an ancestor, calling `use_context::<CommandPaletteOpen>()` in Shell would panic at runtime. Step A0 fixes this by moving the context provision into Shell before the keyboard hook call.

### Step A0: Move `CommandPaletteOpen` context provision from `CommandPalette` into `Shell`

`CommandPaletteOpen` is currently provided inside `CommandPalette` (a child of `Shell`). The keyboard hook runs in `Shell` and calls `use_context::<CommandPaletteOpen>()`, which requires the context to come from an ancestor. Move the provision into `Shell` so the context exists before the hook runs, and have `CommandPalette` consume it instead.

In `crates/lx-desktop/src/layout/shell.rs`, add the import:

Find:
```rust
use crate::components::command_palette::CommandPalette;
```

Replace with:
```rust
use crate::components::command_palette::{CommandPalette, CommandPaletteOpen};
```

In `crates/lx-desktop/src/layout/shell.rs`, add the context provision inside `Shell()`, after the `_onboarding` line and before the `use_effect`:

Find:
```rust
  let _onboarding = OnboardingCtx::provide();
  use_effect(move || {
```

Replace with:
```rust
  let _onboarding = OnboardingCtx::provide();
  use_context_provider(|| CommandPaletteOpen(Signal::new(false)));
  use_effect(move || {
```

In `crates/lx-desktop/src/components/command_palette.rs`, change `CommandPalette` to consume the context instead of providing it:

Find:
```rust
  let mut open = use_signal(|| false);
  let mut query = use_signal(String::new);

  use_context_provider(|| CommandPaletteOpen(open));
```

Replace with:
```rust
  let palette = use_context::<CommandPaletteOpen>();
  let mut open = palette.0;
  let mut query = use_signal(String::new);
```

### Step A1: Replace `crates/lx-desktop/src/hooks/keyboard_shortcuts.rs`

Replace the full file content with:

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

In `crates/lx-desktop/src/layout/shell.rs`, add the keyboard shortcuts import:

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
  format!("{display_h}:{m:02} {period}")
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
