# Real-time Peripherals: Mic Volume and MCP Panel

## Goal

Wire two independent UI features: real-time microphone volume visualization in VoiceBanner driven by VAD RMS data, and the MCP panel displaying actual server names from `.mcp.json`.

## Why

- The mic volume indicator is a static CSS `animate-pulse` animation on four bar characters — it does not reflect actual input level
- The VAD already computes RMS on every audio frame but discards the value — surfacing it requires minimal changes
- The MCP panel shows three hardcoded fake modules (POSTGRES_INTERFACE, AWS_CONSOLE_BRIDGE, O_WORKSPACE_SYNC) instead of the eight real MCP servers configured in `.mcp.json`
- The INSTALL MODULE button has no handler and no purpose since MCP servers are configured via `.mcp.json`

## Depends on

`VOICE_CONTEXT_EXTRACTION` must be completed first. The `rms: Signal<f32>` field in VoiceContext is where mic volume data lands.

## RMS value characteristics

The VAD computes RMS as `sqrt(sum(s²) / count)` where samples are float32 in range [-1, 1]. Observed values: silence ~0.001, normal speech 0.05-0.2, loud speech 0.3+. The normalization in Task 3 clamps at 0.3 to give good visual range for typical speech.

## `.mcp.json` structure

The file is at the repo root. Format:
```json
{
  "mcpServers": {
    "server-name": { "command": "...", "args": [...] },
    ...
  }
}
```
There are 8 servers: context-engine, gritql, inference, infisical, plane, playwright, recipe, workflow. Only the key names are needed for display.

## Files affected

| File | Change |
|------|--------|
| `dioxus-common/ts/audio-capture/src/vad.ts` | Return `rms` from `feed()` alongside existing booleans |
| `dioxus-common/ts/audio-capture/src/capture.ts` | Add `onRms` callback, fire it after every `vad.feed()` call |
| `dioxus-common/ts/widget-bridge/widgets/voice.ts` | Wire `capture.onRms` to send `rms` message to Rust |
| `crates/lx-desktop/src/pages/agents/voice_banner.rs` | Handle `rms` message, replace static pulse animation with RMS-driven bars |
| `crates/lx-desktop/src/pages/agents/mcp_panel.rs` | Read `.mcp.json`, display real server names, remove install button |

## Task List

### Task 1: Surface RMS from VAD

In `dioxus-common/ts/audio-capture/src/vad.ts`:

Change the return type of `feed()` from `{ isSpeech: boolean; silenceExceeded: boolean }` to `{ isSpeech: boolean; silenceExceeded: boolean; rms: number }`.

The `rms` variable is already computed on lines 20-22. Add it to the return object on line 33. The return statement becomes:
```
return { isSpeech, silenceExceeded, rms };
```

### Task 2: Add onRms callback to AudioCapture and wire in voice widget

In `dioxus-common/ts/audio-capture/src/capture.ts`:

1. Add a new public callback field on the `AudioCapture` class, after `onSilence` (line 22):
   ```
   onRms: ((rms: number) => void) | null = null;
   ```

2. In the `workletNode.port.onmessage` handler (line 55), after the `this.vad.feed(samples)` call and BEFORE the `if (silenceExceeded)` check, add:
   ```
   this.onRms?.(rms);
   ```
   Also update the destructure on that line from `const { silenceExceeded }` to `const { silenceExceeded, rms }`.

The `onRms` fires on every audio frame (~every 250ms at 4000 samples / 16kHz), including during silence. This ensures the UI shows volume decay when the user stops talking rather than abruptly dropping.

In `dioxus-common/ts/widget-bridge/widgets/voice.ts`:

In the `mount` function, after setting `capture.onSilence` (after line 40), add:
```
capture.onRms = (rms: number) => {
  dx.send({ type: "rms", level: rms });
};
```

### Task 3: Handle RMS message and render volume bars in VoiceBanner

In `crates/lx-desktop/src/pages/agents/voice_banner.rs`:

1. Add a new match arm in the message loop, after the `audio_playing` arm:
   ```
   Some("rms") => {
     if let Some(level) = msg["level"].as_f64() {
       ctx.rms.set(level as f32);
     }
   },
   ```

2. In the component body (before the `rsx!` block), add this local variable:
   ```
   let volume = (ctx.rms() / 0.3).min(1.0);
   ```

3. In the RSX, replace the static bar animation block:
   ```
   if is_active {
     span { class: "text-[var(--primary)] text-sm ml-1 animate-pulse",
       "\u{2581}\u{2582}\u{2583}\u{2584}"
     }
   }
   ```
   With:
   ```
   if is_active {
     div { class: "flex items-end gap-[2px] h-4 ml-1",
       span {
         class: "w-1 bg-[var(--primary)] rounded-sm transition-all duration-75",
         style: "height: {(volume * 40.0).max(2.0)}%;",
       }
       span {
         class: "w-1 bg-[var(--primary)] rounded-sm transition-all duration-75",
         style: "height: {(volume * 70.0).max(2.0)}%;",
       }
       span {
         class: "w-1 bg-[var(--primary)] rounded-sm transition-all duration-75",
         style: "height: {(volume * 100.0).max(2.0)}%;",
       }
       span {
         class: "w-1 bg-[var(--primary)] rounded-sm transition-all duration-75",
         style: "height: {(volume * 60.0).max(2.0)}%;",
       }
     }
   }
   ```

The four bars have different scaling factors (40%, 70%, 100%, 60%) to create an asymmetric equalizer look. `transition-all duration-75` gives smooth 75ms interpolation between RMS updates. `max(2.0)` ensures bars are always minimally visible when active.

### Task 4: Rewrite MCP panel with real server data

Replace the entire contents of `crates/lx-desktop/src/pages/agents/mcp_panel.rs`.

New imports:
```
use dioxus::prelude::*;
```

Add a helper function that reads and parses `.mcp.json`:
```
fn load_mcp_server_names() -> Vec<String> {
  let Ok(content) = std::fs::read_to_string(".mcp.json") else {
    return Vec::new();
  };
  let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) else {
    return Vec::new();
  };
  let Some(servers) = json.get("mcpServers").and_then(|v| v.as_object()) else {
    return Vec::new();
  };
  servers.keys().cloned().collect()
}
```

This reads from the current working directory which is the repo root when launched via `just desktop` / `dx serve`. Falls back to empty list if file is missing or malformed.

The `McpPanel` component:

1. Load server names once with `use_hook`: `let servers = use_hook(load_mcp_server_names);`

2. Keep the MCP_EXTENSIONS divider heading unchanged (the "MCP_EXTENSIONS" text with horizontal lines on each side).

3. Render a `grid grid-cols-4 gap-3` div. For each server name in `servers`:
   ```
   div { class: "bg-[var(--surface-container-low)] border border-[var(--outline-variant)]/30 rounded-lg p-4 flex flex-col gap-2",
     span { class: "text-2xl text-[var(--primary)]", "\u{1F5C4}" }
     span { class: "text-xs font-semibold uppercase tracking-wider text-[var(--on-surface)]",
       "{name}"
     }
     span { class: "text-[10px] uppercase tracking-wider text-[var(--outline)]",
       "CONFIGURED"
     }
   }
   ```

4. Remove the INSTALL MODULE button entirely. Do not render any dashed-border placeholder card.

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

- **Do not modify voice_context.rs or agent_card.rs.** Those files are handled by other work items.
- **The `onRms` callback fires on every audio frame (~250ms).** This is frequent but each message is one float. Keep the Rust handler minimal — just set the signal.
- **`onRms` must fire BEFORE the `silenceExceeded` early return.** Otherwise volume drops to zero immediately when silence starts instead of showing natural decay during the 2-second silence timeout.
- **Volume bar heights are computed BEFORE the `rsx!` block** as `let volume = ...`. Dioxus RSX does not support arbitrary let bindings inline.
- **`.mcp.json` is at the repo root** which is the working directory for `dx serve`. Reading with `std::fs::read_to_string(".mcp.json")` is correct. Do NOT use `CARGO_MANIFEST_DIR` — that points to `crates/lx-desktop/`, not the repo root.
- **`use_hook` for file reading, not `use_resource`.** `std::fs::read_to_string` is synchronous. `use_resource` is for async operations.
- **The widget-bridge `build.rs` watches `ts/widget-bridge/widgets/`.** Since `voice.ts` is modified in Task 2, the build will automatically trigger `pnpm build` for widget-bridge. However, `ts/audio-capture/` is NOT watched — it is a dependency of widget-bridge compiled by Vite, so changes to `vad.ts` and `capture.ts` are picked up transitively when widget-bridge rebuilds.
- **300 line file limit.** mcp_panel.rs should be under 50 lines after rewrite. voice_banner.rs will grow slightly from the volume bar RSX but stays well under 300.

---

To execute this work item, load it with the workflow MCP:

```
mcp__workflow__load_work_item({ path: "work_items/REALTIME_PERIPHERALS.md" })
```

Then call `next_task` to begin.
