# Graph Kernel Unit 06: Persistence and newsfeed sample

## Depends On
- `work_items/graph_kernel_unit_02_flow_workspace_route_host.md`
- `work_items/graph_kernel_unit_04_rust_widget_command_bridge.md`
- `work_items/graph_kernel_unit_05_palette_inspector_and_validation.md`

## Goal
Finish the DAG editor as a durable workflow-authoring feature by adding document persistence and a concrete sample graph for the news/research aggregation use case. This unit should make `/flows` useful across sessions and leave the app with a tangible workflow example that exercises the shared graph kernel in a real domain.

## Why
- Without persistence, the editor only proves interaction mechanics and loses all value when the app closes.
- The user’s immediate product goal is a news/research workflow system, so the first shipped example should model that pipeline instead of an abstract demo graph.
- `lx-desktop` already depends on `dirs`, which gives this unit a straightforward place to persist local editor state without introducing a backend service yet.
- A concrete sample graph is the fastest way to prove the n8n-style stepping-stone approach before investing in execution/runtime work.

## Changes
- Add local flow-document storage under `pages/flows/storage.rs` using `dirs::data_local_dir()` with an `lx/flows` subdirectory and JSON serialization of the unit 01 graph document.
- Use `/flows/:flow_id` as the persisted-document route and make `/flows` open a sensible default flow or recent flow instead of a fresh anonymous graph every time.
- Add save, save-as-new, and reset-to-sample actions to the flow workspace toolbar.
- Check in a concrete newsfeed sample graph as a source asset, then load or copy it as the default starter workflow when no persisted flow exists yet.
- Ensure the sample exercises realistic workflow nodes such as topic input, curated source list, fetch, extract, dedupe, score, summarize, and feed output.
- Keep runtime execution, cron scheduling, and Codex/OpenAI orchestration out of scope. This unit is about editor durability and a representative authoring sample only.

## Files Affected
- `crates/lx-desktop/src/pages/flows/mod.rs`
- `crates/lx-desktop/src/pages/flows/storage.rs`
- `crates/lx-desktop/src/pages/flows/sample.rs`
- `crates/lx-desktop/src/pages/flows/workspace.rs`
- `crates/lx-desktop/src/pages/flows/controller.rs`
- `crates/lx-desktop/src/routes.rs`
- `crates/lx-desktop/assets/flows/newsfeed.json`

## Task List
1. Add `pages/flows/storage.rs` and implement local JSON persistence for graph documents using `dirs::data_local_dir().join("lx/flows")`. Keep the storage format aligned with the unit 01 document model instead of inventing a second persistence schema.
2. Update the flows controller and route handling so `/flows/:flow_id` loads persisted documents by id and `/flows` resolves to the default or most recent document.
3. Add toolbar actions for save, save-as-new, and reset-to-sample. The controller should own these actions so they work with the same canonical graph document used by the widget and inspector.
4. Check in `crates/lx-desktop/assets/flows/newsfeed.json` as the canonical starter sample and update `sample.rs` to load from that asset or mirror its exact structure for deterministic startup behavior.
5. Make the sample graph model the intended news/research aggregation workflow rather than a generic DAG. It should be realistic enough to guide later runtime and backend work.
6. Verify that editing a persisted flow, closing the app, and reopening the same route restores the saved graph correctly.

## Verification
- Run `just fmt`.
- Run `just rust-diagnose`.
- Run `just desktop`.
- In the running app, open the default sample flow, modify it, save it, restart the app, and reopen the same flow id. Confirm node layout, edges, selection defaults, and edited properties persist correctly.
