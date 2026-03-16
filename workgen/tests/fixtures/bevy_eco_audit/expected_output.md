# Goal

Fix Bevy ecosystem plugin violations: raw rand instead of bevy_rand, bevy_kira_audio non-spatial playback without volume/playback rate, manual physics instead of rapier systems, and unscoped egui windows.

# Why

`spawn_particles` uses `rand::thread_rng()` directly instead of `bevy_rand` which provides deterministic seeding and ECS integration. `play_sound` uses `audio.play(handle)` without configuring volume or playback rate — should use `.play(handle).with_volume(Volume::new(0.8))`. `apply_physics` manually updates transforms from velocity instead of using Rapier's built-in physics systems — Rapier handles velocity integration automatically. The egui debug window has no scoping, meaning it renders in all game states instead of only when debug is active.

# What changes

- Replace `rand::thread_rng()` with `bevy_rand` RNG resource for deterministic seeding
- Add volume/playback rate configuration to `audio.play()` call
- Remove manual velocity integration in `apply_physics` — let Rapier handle physics
- Scope egui windows to specific game states or debug conditions

# Files affected

- src/plugins.rs — raw rand usage, unconfigured audio playback, manual physics override, unscoped egui

# Task List

## Task 1: Fix RNG and audio

Replace rand::thread_rng with bevy_rand GlobalEntropy resource. Add volume/rate to audio playback.

```
just fmt
git add src/plugins.rs
git commit -m "fix: use bevy_rand for RNG, configure audio playback"
```

## Task 2: Fix physics and UI scoping

Remove manual velocity integration. Add state guard to egui debug window.

```
just fmt
git add src/plugins.rs
git commit -m "fix: use Rapier physics, scope egui to debug state"
```

## Task 3: Verification

```
just test
just diagnose
just fmt
git add -A
git commit -m "chore: verify bevy ecosystem audit fixes"
```

# CRITICAL REMINDERS

- Run `just fmt` after every file change
- Run `just test` and `just diagnose` before final commit
- No raw rand — use bevy_rand for deterministic seeding
- No manual physics when Rapier is available
- Always configure audio volume/rate

# Task Loading Instructions

Load these instructions by reading this file, then execute each task in order. After each task, run `just fmt` and commit.
