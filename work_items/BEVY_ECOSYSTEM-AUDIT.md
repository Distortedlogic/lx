# Goal

Fix five bevy ecosystem crate misuse patterns in the `bevy_eco_audit` fixture's `plugins.rs`: replace manual RNG with bevy_rand's `GlobalRng`, introduce a typed `SfxChannel` audio channel and capture the play handle, and fix the egui UI system to return `Result` and run in `EguiPrimaryContextPass`.

# Why

- `rand::thread_rng()` bypasses bevy_rand's determinism chain, making replay and testing nondeterministic
- All sounds on the default `Audio` channel prevents independent volume and pause control per sound category
- Dropping `PlayAudioCommand` without calling `.handle()` makes individual sound instance control impossible
- `setup_ui` returning `()` instead of `Result` will fail to compile when correctly placed in `EguiPrimaryContextPass`
- Running egui UI code outside `EguiPrimaryContextPass` breaks mandatory multi-pass widget measurement

# What changes

**bevy_rand integration** — Remove the `rand` crate import. Change `spawn_particles` to accept `Single<&mut WyRand, With<GlobalRng>>` instead of constructing `rand::thread_rng()`. Call `rng.gen_range(...)` on the bevy_rand-managed RNG. Add `bevy_rand` and `wyrand` imports. Ensure `EntropyPlugin::<WyRand>::default()` is registered (noted in task but actual plugin registration is outside this file's scope — the fixture only audits the system function signatures and usage patterns).

**Typed audio channel** — Define a `struct SfxChannel;` marker type. Change `play_sound` to accept `Res<AudioChannel<SfxChannel>>` instead of `Res<Audio>`. The channel must be registered via `app.add_audio_channel::<SfxChannel>()` in the plugin setup (outside this file's direct scope but the system signature must reflect the typed channel).

**Capture play handle** — Change `audio.play(handle)` to `audio.play(handle).handle()` and store the returned `Handle<AudioInstance>` in a local binding so the sound instance can be controlled later.

**egui Result return type** — Change `setup_ui` return type from implicit `()` to `Result` (using `bevy_egui`'s expected error type). Add `Ok(())` at the end of the function body.

**EguiPrimaryContextPass schedule** — The system signature change (returning `Result`) is what enables it to be registered in `EguiPrimaryContextPass`. The actual schedule registration is outside this file, but the function signature must be compatible.

# Files affected

- `workgen/tests/fixtures/bevy_eco_audit/src/plugins.rs` — All five fixes: replace `rand::thread_rng()` with `GlobalRng`, replace `Res<Audio>` with `Res<AudioChannel<SfxChannel>>`, capture play handle, change `setup_ui` return type to `Result`, add `Ok(())` return

# Task List

## Task 1: Replace manual RNG with bevy_rand GlobalRng

**File:** `workgen/tests/fixtures/bevy_eco_audit/src/plugins.rs`

Remove the `use rand::Rng;` import. Add imports for `bevy_rand::prelude::*` and `bevy_prng::WyRand`. Change the `spawn_particles` function signature to accept `mut rng: Single<&mut WyRand, With<GlobalRng>>` instead of `mut commands: Commands` being the only parameter (keep `mut commands: Commands` as well). Remove the `let mut rng = rand::thread_rng();` line. Use `rng.gen_range(...)` directly on the injected Single parameter (deref as needed).

Run `just fmt` then `git add workgen/tests/fixtures/bevy_eco_audit/src/plugins.rs` then `git commit -m "fix: replace rand::thread_rng with bevy_rand GlobalRng in bevy_eco_audit fixture"`.

## Task 2: Add typed SfxChannel and capture audio play handle

**File:** `workgen/tests/fixtures/bevy_eco_audit/src/plugins.rs`

Define `struct SfxChannel;` as a unit struct near the top of the file. Change the `play_sound` function parameter from `audio: Res<Audio>` to `audio: Res<AudioChannel<SfxChannel>>`. Change the `audio.play(handle);` call to `let _instance = audio.play(handle).handle();` so the `PlayAudioCommand` is consumed by `.handle()` and the resulting `Handle<AudioInstance>` is captured.

Run `just fmt` then `git add workgen/tests/fixtures/bevy_eco_audit/src/plugins.rs` then `git commit -m "fix: use typed SfxChannel and capture audio play handle in bevy_eco_audit fixture"`.

## Task 3: Fix egui UI system return type and schedule compatibility

**File:** `workgen/tests/fixtures/bevy_eco_audit/src/plugins.rs`

Change the `setup_ui` function signature to return `Result` (i.e., `fn setup_ui(mut contexts: EguiContexts) -> Result`). Add `Ok(())` as the final expression in the function body after the `egui::Window` call. Add `use bevy_egui::*;` if not already imported (the file currently uses `EguiContexts` which implies some import exists — verify and add the full prelude if needed).

Run `just fmt` then `git add workgen/tests/fixtures/bevy_eco_audit/src/plugins.rs` then `git commit -m "fix: change setup_ui to return Result for EguiPrimaryContextPass compatibility"`.

## Task 4: Verify all fixes compile and pass

Run `just test` to confirm the test suite passes. Run `just diagnose` to confirm no warnings or errors. Run `just fmt` to confirm formatting is clean. If any failures occur, fix them before marking complete.

---

# CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.

# Task Loading Instructions

To begin execution, run:

```
mcp__workflow__load_work_item({ path: "work_items/FIX_BEVY_ECO_AUDIT_CRATE_MISUSE.md" })
```