# Goal

Remediate nine audit findings in the Bevy game plugin at `src/game.rs`: fix missing reflection and convenience derives on components and resources, replace verbose resource initialization, split a monolithic plugin into focused sub-plugins, correct system schedules, add missing state-based run conditions, deduplicate repeated run conditions via a shared `SystemSet`, and remove unnecessary system ordering that blocks parallelism.

# Why

- `Health`, `Speed`, and `GameConfig` lack `Reflect` and `#[reflect(...)]`, making them invisible to `bevy-inspector-egui` and unusable in scene serialization
- `Health` and `Speed` are single-field newtypes without `Deref`/`DerefMut`, forcing verbose `.0` access at every call site
- `GameConfig` is initialized with `insert_resource(GameConfig::default())` instead of the idiomatic `init_resource::<GameConfig>()`
- `GamePlugin::build` registers 19 systems in a single method, well above the 15-system threshold, making it hard to reason about responsibilities
- `sync_transforms` and `update_health_bar` perform visual synchronization but run in `Update` instead of `PostUpdate`, so they execute before game logic has finished mutating state
- The 16 chained game-logic systems have no `run_if(in_state(GameState::Playing))` guard and therefore execute in every state including menus
- The same `.run_if(in_state(GameState::Playing))` is duplicated on three separate `add_systems` calls instead of being applied once to a `SystemSet`
- All 16 systems are blanket-`.chain()`ed, forcing serial execution even between systems with no data dependency (e.g., `play_sounds` and `animate_sprites`)

# What changes

**Component derives (Health, Speed):** Add `Reflect` to the derive list, add `#[reflect(Component)]` attribute, and add `Deref` and `DerefMut` derives. Both `Deref` and `DerefMut` are re-exported from `bevy::prelude`.

**Resource derives (GameConfig):** Add `Reflect` to the derive list and add `#[reflect(Resource)]` attribute.

**Resource initialization:** Replace `.insert_resource(GameConfig::default())` with `.init_resource::<GameConfig>()`.

**Plugin split:** Replace the single `GamePlugin` with a parent plugin that composes focused sub-plugins:
- `PlayerPlugin` — registers `move_player`, `handle_input`, `check_boundaries`
- `CombatPlugin` — registers `check_collisions`, `spawn_enemies`, `despawn_dead`, `process_powerups`, `update_score`
- `PhysicsPlugin` — registers `apply_gravity`
- `UiPlugin` — registers `update_health_bar`, `update_ui_text`, `update_menu`, `render_debug`, `save_progress`
- `AudioVisualPlugin` — registers `play_sounds`, `animate_sprites`, `sync_transforms`, `update_camera`, `handle_pause`

The parent `GamePlugin::build` calls `app.add_plugins((...))` with all sub-plugins and handles resource initialization.

**Schedule corrections:** In their respective sub-plugins, `sync_transforms` and `update_health_bar` are registered in `PostUpdate` instead of `Update`.

**Run conditions:** All game-logic systems (those currently in the 16-system chain) receive a `run_if(in_state(GameState::Playing))` guard, either directly or via a `SystemSet`.

**SystemSet deduplication:** Define a `PlayingSet` system set with `.run_if(in_state(GameState::Playing))` configured once. Assign `render_debug`, `update_menu`, and `save_progress` to this set instead of repeating the run condition on each.

**Ordering cleanup:** Remove the blanket `.chain()`. Add targeted `.before()`/`.after()` or `.chain()` only between systems with actual data dependencies:
- `handle_input` before `move_player` (input drives movement)
- `move_player` before `check_collisions` (position must update before collision detection)
- `check_collisions` before `despawn_dead` (collision results drive death)
- `despawn_dead` before `update_score` (score updates after kills)
- `apply_gravity` before `check_boundaries` (gravity affects position checked by boundaries)

All other systems run unordered to maximize parallelism.

# Files affected

- `src/game.rs` — All changes: derive additions on `Health`, `Speed`, `GameConfig`; plugin split into sub-plugins; schedule corrections; run condition additions; `SystemSet` creation; ordering cleanup; `init_resource` replacement

# Task List

## Task 1: Add missing derives to Health and Speed components

**Files:** `src/game.rs`

In `src/game.rs`, update the `Health` struct:
- Change the derive to include `Component`, `Reflect`, `Deref`, and `DerefMut`
- Add `#[reflect(Component)]` attribute above the struct

Update the `Speed` struct identically:
- Change the derive to include `Component`, `Reflect`, `Deref`, and `DerefMut`
- Add `#[reflect(Component)]` attribute above the struct

Both `Deref`, `DerefMut`, and `Reflect` are available from `bevy::prelude::*`. After this change, both structs have four derives and the reflect attribute.

**Verify:** `just diagnose` compiles without errors or warnings.

**Then run:** `just fmt`, then `git add src/game.rs`, then `git commit -m "Add Reflect, Deref, DerefMut derives to Health and Speed components"`

## Task 2: Add missing derives to GameConfig and fix resource initialization

**Files:** `src/game.rs`

In `src/game.rs`, update the `GameConfig` struct:
- Change the derive to include `Resource`, `Default`, and `Reflect`
- Add `#[reflect(Resource)]` attribute above the struct

In `GamePlugin::build`, replace `.insert_resource(GameConfig::default())` with `.init_resource::<GameConfig>()`.

**Verify:** `just diagnose` compiles without errors or warnings.

**Then run:** `just fmt`, then `git add src/game.rs`, then `git commit -m "Add Reflect derive to GameConfig and use init_resource"`

## Task 3: Define PlayingSet system set

**Files:** `src/game.rs`

In `src/game.rs`, define a new `PlayingSet` type:
- Derive `SystemSet`, `Debug`, `Clone`, `PartialEq`, `Eq`, `Hash`
- Make it a unit struct

This set will be configured in a later task when the plugin is restructured. For now, just define the type.

**Verify:** `just diagnose` compiles without errors or warnings.

**Then run:** `just fmt`, then `git add src/game.rs`, then `git commit -m "Define PlayingSet system set"`

## Task 4: Split GamePlugin into focused sub-plugins

**Files:** `src/game.rs`

Replace the monolithic `GamePlugin` with a parent plugin that composes five sub-plugins. Define each sub-plugin as a struct implementing `Plugin`:

**PlayerPlugin::build** — Register in `Update`: `handle_input`, `move_player`, `check_boundaries`. Chain `handle_input` before `move_player` (input drives movement). No other ordering.

**CombatPlugin::build** — Register in `Update`: `check_collisions`, `spawn_enemies`, `despawn_dead`, `process_powerups`, `update_score`. Chain in this order: `check_collisions` then `despawn_dead` then `update_score`. `spawn_enemies` and `process_powerups` are unordered.

**PhysicsPlugin::build** — Register in `Update`: `apply_gravity`. Add ordering: `apply_gravity` before `check_boundaries` (which is in PlayerPlugin — use `.before()` referencing the function).

**UiPlugin::build** — Register `update_health_bar` in `PostUpdate` (visual sync). Register `update_ui_text` in `Update`. Configure `PlayingSet` with `.run_if(in_state(GameState::Playing))`. Register `render_debug`, `update_menu`, and `save_progress` in `Update` with `.in_set(PlayingSet)` instead of individual `.run_if()` calls.

**AudioVisualPlugin::build** — Register `sync_transforms` in `PostUpdate` (visual sync). Register `play_sounds`, `animate_sprites`, `update_camera`, and `handle_pause` in `Update`, unordered.

**GamePlugin::build** — Call `app.add_plugins((PlayerPlugin, CombatPlugin, PhysicsPlugin, UiPlugin, AudioVisualPlugin))` and `.init_resource::<GameConfig>()`. Register `GameConfig` type for reflection with `app.register_type::<GameConfig>()`. Also register `Health` and `Speed` types with `app.register_type`.

Add `run_if(in_state(GameState::Playing))` to all game-logic systems across all sub-plugins (every system registered in `Update`). Apply it per-system or per-tuple as appropriate.

Remove the old `GamePlugin::build` body entirely. The 16-system `.chain()` no longer exists.

**Verify:** `just diagnose` compiles without errors or warnings.

**Then run:** `just fmt`, then `git add src/game.rs`, then `git commit -m "Split GamePlugin into focused sub-plugins with correct schedules and ordering"`

## Task 5: Verification

Run the full verification suite:

1. `just fmt` — confirm no formatting issues
2. `just diagnose` — confirm no compiler errors or clippy warnings
3. `just test` — confirm all tests pass

**Then run:** `git add -A`, then `git commit -m "Bevy audit: verify all changes pass fmt, diagnose, and test"`

---

# CRITICAL REMINDERS

Re-read before starting each task:

1. **Run `just fmt`, `git add`, and `git commit` after each task** as specified in the task.
2. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
3. **Do not add code comments or doc strings** per project rules.
4. **Do not use `#[allow(...)]` macros** — leave warnings visible if any arise.
5. **File must not exceed 300 lines** — if `src/game.rs` exceeds 300 lines after edits, split into multiple files.

# Task Loading Instructions

To execute this work item, load it with the workflow MCP:

```
mcp__workflow__load_work_item({ path: "work_items/BEVY_AUDIT_GAME_PLUGIN.md" })
```

Then call `mcp__workflow__next_task` to begin the first task.