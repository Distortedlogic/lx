# LX Desktop Pi Runtime Integration

## Goal
Add a desktop-global runtime layer in lx-desktop that is shaped around lx runtime concepts, implement Pi RPC as backend adapter #1, and drive a native runtime surface from that registry so the desktop can launch, observe, and control live Pi-backed agent sessions without coupling the app to Pi-specific UI semantics.

## Why
- The current Agents page is placeholder-only and has no live runtime backing.
- The current desktop has no runtime registry that matches lx agent, control, event, and tool-activity concepts.
- Pi already exposes a structured RPC runtime, so the correct short-term path is an adapter into lx-shaped desktop state, not a Pi-native terminal embed.
- Flow execution needs a grouping seam now so lx-native runtime can replace Pi later without redoing desktop state.

## Changes
- Add a new crates/lx-desktop/src/runtime module with desktop runtime types, registry storage, controller methods, and a Pi RPC backend adapter.
- Add a desktop-global runtime provider above routing so all pages can open, inspect, and control live runtime agents.
- Add a Pi runtime page and native widget components for transcript, tool activity, prompt input, and control actions.
- Replace placeholder Agents page sourcing with runtime-registry-backed data and route agent detail actions into the runtime controller.
- Add flow launch controls that create Pi-backed runtime sessions grouped by flow_id so flow authoring and runtime state are connected.

## Files Affected
- crates/lx-desktop/src/lib.rs
- crates/lx-desktop/src/app.rs
- crates/lx-desktop/src/routes.rs
- crates/lx-desktop/src/runtime/mod.rs
- crates/lx-desktop/src/runtime/types.rs
- crates/lx-desktop/src/runtime/registry.rs
- crates/lx-desktop/src/runtime/controller.rs
- crates/lx-desktop/src/runtime/pi_backend.rs
- crates/lx-desktop/src/widgets/mod.rs
- crates/lx-desktop/src/widgets/pi_widget.rs
- crates/lx-desktop/src/widgets/pi_transcript.rs
- crates/lx-desktop/src/widgets/pi_tool_activity.rs
- crates/lx-desktop/src/widgets/pi_input.rs
- crates/lx-desktop/src/pages/tools/mod.rs
- crates/lx-desktop/src/pages/tools/pi_page.rs
- crates/lx-desktop/src/pages/agents/mod.rs
- crates/lx-desktop/src/pages/agents/detail.rs
- crates/lx-desktop/src/pages/agents/run_detail.rs
- crates/lx-desktop/src/pages/agents/types.rs
- crates/lx-desktop/src/pages/agent_detail.rs
- crates/lx-desktop/src/pages/flows/mod.rs

## Task List
1. Add desktop runtime foundation types and registry primitives that store live agents, lx-shaped events, tool activity, flow grouping, and view-model helpers for transcripts and runs.
2. Add a desktop runtime controller and Pi backend adapter that can spawn pi --mode rpc --no-session, send prompt, steer, follow_up, abort, read JSONL events, update runtime state, and degrade unsupported pause and resume actions cleanly.
3. Add a desktop-global runtime provider and route wiring so runtime state is available across the app and a dedicated Pi runtime page can open one selected runtime widget.
4. Add native Pi widget components that read only desktop runtime state, render transcript and tool activity, and send control actions through the runtime controller.
5. Replace placeholder Agents page data with runtime-registry-backed summaries and details, including live transcript and tool activity in the runs surface and a direct open-widget action for Pi-backed agents.
6. Add flow launch controls that create runtime entries grouped by flow_id and surface grouped Pi-backed sessions from the flow workspace route wrapper without coupling the flow UI to Pi-specific event payloads.
7. Run formatting, diagnostics, and tests with the repo justfile recipes and fix all resulting issues without expanding scope beyond this runtime integration.

## Verification
- just fmt
- just diagnose
- just test
