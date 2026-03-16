# Goal

Fix Bevy ECS violations: monolithic plugin, system ordering without data dependency, duplicated run conditions, missing reflect derives, insert_resource(default()) pattern, missing Deref/DerefMut on newtype wrappers, and sync_transforms in wrong schedule.

# Why

GamePlugin registers 16+ systems in a single `build` method — split into focused sub-plugins. All 16 systems are `.chain()`ed without verifying data dependencies, preventing parallelism. Three `add_systems` calls duplicate `.run_if(in_state(GameState::Playing))` instead of using a SystemSet. `Health` and `Speed` components lack `#[reflect(Component)]`. `GameConfig` uses `insert_resource(GameConfig::default())` instead of `init_resource`. `Speed` is a newtype wrapper accessed via `.0` without Deref/DerefMut derives. `sync_transforms` does visual sync in Update instead of PostUpdate.

# What changes

- Split GamePlugin into focused sub-plugins (movement, combat, rendering, UI)
- Remove `.chain()` — only order systems with actual data dependencies
- Create a SystemSet with `.run_if(in_state(GameState::Playing))` and assign the 3 systems to it
- Add `Reflect` derive and `#[reflect(Component)]` to Health and Speed
- Add `#[reflect(Resource)]` to GameConfig
- Replace `insert_resource(GameConfig::default())` with `init_resource::<GameConfig>()`
- Add `Deref, DerefMut` derives to Speed newtype
- Move `sync_transforms` to PostUpdate schedule

# Files affected

- src/game.rs — monolithic plugin, chained systems without data dependency, duplicated run conditions, missing reflects, insert_resource default, missing Deref, wrong schedule

# Task List

## Task 1: Split monolithic plugin

Create sub-plugins for movement, combat, rendering, UI. Compose in GamePlugin.

```
just fmt
git add src/game.rs
git commit -m "fix: split monolithic plugin into focused sub-plugins"
```

## Task 2: Fix system ordering and run conditions

Remove blanket .chain(). Create SystemSet with shared run condition. Move sync_transforms to PostUpdate.

```
just fmt
git add src/game.rs
git commit -m "fix: remove unnecessary chain, deduplicate run conditions, fix schedule"
```

## Task 3: Fix derives and resource init

Add Reflect and reflect attributes to components/resources. Use init_resource. Add Deref/DerefMut to Speed.

```
just fmt
git add src/game.rs
git commit -m "fix: add reflect derives, init_resource, Deref on newtype"
```

## Task 4: Verification

```
just test
just diagnose
just fmt
git add -A
git commit -m "chore: verify bevy audit fixes"
```

# CRITICAL REMINDERS

- Run `just fmt` after every file change
- Run `just test` and `just diagnose` before final commit
- No monolithic plugins — split by responsibility
- No system chain without data dependencies
- All components need #[reflect(Component)]

# Task Loading Instructions

Load these instructions by reading this file, then execute each task in order. After each task, run `just fmt` and commit.
