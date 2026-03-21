# Bevy Ecosystem Crate Audit

Every item below is a binary check. This audit covers bevy ecosystem crates, NOT bevy itself (see bevy-audit.md). Checks focus on cross-file patterns, non-obvious API idioms, and mistakes from quick pattern matching.

---

## bevy_rand

- **Unfiltered RNG query catches GlobalRng** — `Query<&mut WyRand>` without a `With<>` filter matches the global RNG entity AND all per-entity RNGs. Fix: use `Single<&mut WyRand, With<GlobalRng>>` for global, `Query<&mut WyRand, With<MyMarker>>` for per-entity.
  `rg 'Query<.*&mut WyRand' --type rust src/`

- **GlobalRng blocks parallelism** — Multiple systems taking `Single<&mut WyRand, With<GlobalRng>>` cannot run in parallel. Fix: fork per-entity RNGs at spawn via `global.fork_rng()`.
  `rg 'With<GlobalRng>' --type rust src/` — flag if 3+ systems access it in same schedule.

- **Query iteration + GlobalRng = nondeterministic** — Iterating a query and calling `global.next_u32()` per entity produces different results depending on iteration order. Fix: per-entity RNG.

- **Missing `reseed_linked()` after `with_target_rngs()`** — `with_target_rngs()` spawns linked entities but does NOT seed them. Must chain `.reseed_linked()`.

- **Manual RNG instead of bevy_rand** — `rand::thread_rng()`, `rand::rng()`, `StdRng::from_entropy()` break deterministic replay. Fix: use bevy_rand's GlobalRng or per-entity RNG.
  `rg 'thread_rng\|rand::rng\|from_entropy\|StdRng' --type rust src/`

- **New seed instead of forked** — `RngSeed::default()` uses `OsRng`/`ThreadLocalEntropy`, breaking the determinism chain. Fix: fork from parent via `global.fork_seed::<WyRand>()` (returns seed component with auto-hook that spawns the RNG) or `global.fork_rng()` (returns RNG directly). Cross-algorithm: `global.fork_as::<WyRand>()`.
  `rg 'RngSeed.*default\|RngSeed::new' --type rust src/`

- **Missing EntropyPlugin per algorithm** — `EntropyPlugin` must be registered once per RNG type. Using both `WyRand` and `ChaCha8Rng` requires two registrations. Cross-algorithm seeding also requires `EntropyRelationsPlugin::<Source, Target>`.
  `rg 'EntropyPlugin' --type rust src/`

## bevy_kira_audio

- **Typed channel not registered** — `Res<AudioChannel<T>>` without `app.add_audio_channel::<T>()` queues commands that never execute.
  `rg 'AudioChannel<' --type rust src/` cross-ref with `rg 'add_audio_channel' --type rust src/`

- **Channel state applies to new sounds** — If a channel is paused, newly played sounds on it start paused. Fix: resume channel before playing, or use a different channel.

- **Instance handle required for individual control** — Channel-level `audio.pause()` pauses ALL sounds. For single-sound control, capture `Handle<AudioInstance>` from `.play().handle()` and use `audio_instances.get_mut(&handle)`.

- **Missing audio channel separation** — Playing sounds on default `Audio` channel instead of project's `MusicChannel`/`SfxChannel`. Fix: use typed channels.
  `rg 'Res<Audio>' --type rust src/` — flag default channel usage.

- **Music not stopped before play** — `.play()` on music channel without `.stop()` causes overlapping tracks. Fix: stop current track first.

- **PlayAudioCommand handle not captured** — `PlayAudioCommand` (returned by `.play()`) auto-queues on drop. To control the sound later, call `.handle()` before the builder drops: `let h = channel.play(src).looped().handle();`. Forgetting `.handle()` = no way to pause/stop that specific sound.

- **Channel control deferred, instance control immediate** — `channel.pause()`, `.stop()`, `.set_volume()` are deferred to PostUpdate. Direct `AudioInstance` methods (via `Assets<AudioInstance>.get_mut()`) execute immediately on the audio thread. Mixing both in the same frame can cause ordering surprises.

- **Missing SpatialAudioPlugin** — Spatial audio requires `SpatialAudioPlugin` added separately. Without it, `SpatialAudioEmitter`/`SpatialAudioReceiver` components have no effect. Receiver must be unique (`Single<>`). Default radius is 25.0 units (override with `DefaultSpatialRadius` resource or per-emitter `SpatialRadius` component).
  `rg 'SpatialAudioEmitter\|SpatialAudioReceiver' --type rust src/`

## bevy_hanabi

- **Missing `Attribute::LIFETIME` = invisible particles** — Particles without lifetime init modifier live forever and block capacity. Fix: always include `SetAttributeModifier::new(Attribute::LIFETIME, expr)` in `.init()`.
  `rg 'EffectAsset::new' --type rust src/` — verify `.init()` chain includes LIFETIME.

- **Init vs Update vs Render modifier misplacement** — Position/velocity/lifetime → `.init()`. Forces/drag/kill zones → `.update()`. Color/size gradients/orientation → `.render()`. Wrong placement produces no error but wrong behavior.

- **SpawnerSettings `emit_on_start` vs `starts_active`** — `once(n).with_emit_on_start(false)` waits for `spawner.reset()`. `burst(n, t).with_starts_active(false)` is inactive until toggled. Confusing these = silent failures.

- **Properties are CPU-side, attributes are GPU-side** — `EffectProperties.set("key", val)` = runtime CPU control (uploaded to GPU each frame when changed). `Attribute::*` = per-particle GPU state. Use `EffectProperties::set_if_changed()` to avoid unnecessary GPU uploads.

- **Spawner modification after TickSpawners** — Systems modifying `EffectSpawner` (toggling `active`, calling `reset()`) must run BEFORE `EffectSystems::TickSpawners` in PostUpdate. Running after = one frame late.
  `rg 'EffectSpawner' --type rust src/`

- **Wrong SimulationSpace for moving emitters** — `SimulationSpace::Global` (default) = particles stay in world space when emitter moves. For particles following the emitter (aura, shield), use `SimulationSpace::Local`. Also: `SimulationCondition::Always` needed for temporal continuity when emitter leaves camera view.
  `rg 'SimulationSpace\|SimulationCondition' --type rust src/`

- **Effect asset recreation in Update** — `EffectAsset::new` in Update systems recreates assets every frame. Fix: create in Startup, store handles in a resource, spawn `ParticleEffect` with stored handle.
  `rg 'EffectAsset::new' --type rust src/` — flag if in Update system.

## bevy_tweening

- **Component is `TweenAnim`, not `Animator`** — The Bevy 0.17 animation component is `TweenAnim::new(tween)`. `Animator` is the old name.
  `rg 'Animator' --type rust src/` — flag if should be `TweenAnim`.

- **Lens receives pre-eased ratio** — `ratio` in `Lens::lerp()` is already eased. The lens does linear interpolation only. Don't apply easing inside the lens.

- **Bare `Delay` in `TweenAnim` panics** — `TweenAnim::new(Delay::new(...))` panics (untyped). Fix: wrap in sequence: `Delay::new(dur).then(tween)`.

- **`AnimTarget` required for cross-entity animation** — Without `AnimTarget`, tween targets same entity. For other entity: `AnimTarget::component::<T>(entity)`. For resource: `AnimTarget::resource::<T>()`.

- **`MirroredRepeat` counts each direction as a cycle** — `RepeatCount::Finite(2)` with `MirroredRepeat` = start→end + end→start (not 2 round trips).

- **Manual interpolation instead of tweening** — Manual `lerp`/`delta * speed` for visual properties (position, scale, color, alpha) where `Tween` with `EaseFunction` would be cleaner. Fix: use `Tween::new`.
  `rg 'lerp\(' --type rust src/` — check if animating a visual property.

- **Sequence with mixed target types** — All tweenables in a `Sequence` must target the same component type. Mixing (e.g., Transform lens + Sprite lens) causes panic. Fix: use separate `TweenAnim` entities per target type.
  `rg 'Sequence::new' --type rust src/`

- **Infinite tween blocks sequence** — An infinite-repeat tween placed before other items in a `Sequence` prevents subsequent tweenables from ever executing. Fix: use infinite tweenables standalone or as the last sequence item.

- **`set_tweenable()` without zero-step** — `anim.set_tweenable(new_tween)` swaps the animation but does NOT apply new state to the target until the next frame. Fix: call `TweenAnim::step_one(world, Duration::ZERO, entity)` after to force-apply immediately (required for smooth dynamic updates like cursor following).
  `rg 'set_tweenable' --type rust src/`

- **`destroy_on_completion` default is true** — `TweenAnim` auto-removes itself when animation finishes. Set `.with_destroy_on_completed(false)` to keep it for reuse or state checking. If true, the entity that owns the animation may still exist but the animation component is gone.

## bevy_rapier2d

- **Missing `ActiveEvents` = no collision events** — Collider without `ActiveEvents::COLLISION_EVENTS` produces no `CollisionEvent`. Fix: always add `ActiveEvents` to entities needing collision detection.
  `rg 'CollisionEvent' --type rust src/` cross-ref with `rg 'ActiveEvents' --type rust src/`

- **Sensor + `CONTACT_FORCE_EVENTS` = silent failure** — Sensors produce collision events but NO contact forces. `CONTACT_FORCE_EVENTS` on a `Sensor` entity produces nothing.

- **Ray direction length affects `max_toi`** — `max_toi` multiplies direction length, not distance. `cast_ray(o, Vec2::new(2,0), 100, true)` = max distance 200. Fix: normalize direction or account for length.

- **CollisionGroups requires BOTH-way pass** — Two colliders interact only if `(a.memberships & b.filters != 0) AND (b.memberships & a.filters != 0)`. One-sided = no collision.
  `rg 'CollisionGroups' --type rust src/`

- **Never write GlobalTransform on physics entities** — Rapier manages GlobalTransform for dynamic bodies. Writing it directly causes desync. Fix: write to `Transform`, `Velocity`, or `ExternalForce`.

- **Collider without RigidBody = implicit Fixed** — Entity with `Collider` but no `RigidBody` = `RigidBody::Fixed`. Intentional but easy to miss when debugging "why doesn't it move."

- **`pixels_per_meter` scales gravity** — With `pixels_per_meter(100.0)`, gravity = 981 px/s² (not 9.81). Colliders/forces must scale accordingly.

- **Missing collision groups** — Entities with `Collider` but no `CollisionGroups`, colliding with everything by default. Fix: assign groups.
  `rg 'Collider::' --type rust src/` — check if `CollisionGroups` also inserted.

- **ExternalImpulse only fires on change detection** — `ExternalImpulse` applies only when the component changes (Bevy's `Changed<>` filter). Setting the same value twice = no effect. Must call `.reset()` to zero it out after use, then set a new value next time. Contrast with `ExternalForce` which applies continuously.
  `rg 'ExternalImpulse' --type rust src/`

- **Collision events use MessageReader, not EventReader** — `CollisionEvent` and `ContactForceEvent` are Bevy Messages (0.17+). Use `MessageReader<CollisionEvent>`, not `EventReader`. Messages clear each frame automatically.
  `rg 'EventReader.*Collision\|EventReader.*ContactForce' --type rust src/`

- **ActiveCollisionTypes excludes kinematic pairs** — Default enables `DYNAMIC_DYNAMIC | DYNAMIC_KINEMATIC | DYNAMIC_STATIC` only. Kinematic-kinematic and kinematic-static are disabled. Fix: explicitly add `KINEMATIC_KINEMATIC | KINEMATIC_STATIC` if kinematic bodies need to interact.
  `rg 'KinematicPositionBased\|KinematicVelocityBased' --type rust src/`

- **ContactForceEventThreshold required for force events** — `ContactForceEvent` requires BOTH `ActiveEvents::CONTACT_FORCE_EVENTS` AND `ContactForceEventThreshold(f32)` on at least one collider. Without the threshold, force events never emit.
  `rg 'ContactForceEvent' --type rust src/`

- **RapierQueryPipeline must be scoped** — `RapierQueryPipeline` is no longer a component (v0.31+). It borrows interior data and must be created fresh via `RapierQueryPipeline::new_scoped()` with a closure. Cannot be stored in a field or resource.
  `rg 'RapierQueryPipeline' --type rust src/`

- **sync_removals must be PostUpdate** — Entity removal syncing must run in PostUpdate even if physics is in FixedUpdate, because Bevy's removal events only fire in PostUpdate. Misplacement causes crashes on entity despawn.

## bevy_egui

- **Systems MUST use `&mut EguiContexts`** — Immutable `&EguiContexts` prevents scheduler from ordering correctly. Always take mutable reference.
  `rg 'EguiContexts' --type rust src/` — flag any without `mut`.

- **UI systems must return `Result`** — Systems in `EguiPrimaryContextPass` must return `Result<(), _>`. Returning `()` = compile error.

- **Camera required** — Egui context attaches to a camera. No camera = no UI. Fix: spawn at least one camera.

- **Input not consumed by default** — Egui doesn't prevent Bevy from seeing the same input. Fix: `run_if(not(egui_wants_any_pointer_input))` on game input systems.
  `rg 'egui_wants' --type rust src/`

- **egui code outside `EguiPrimaryContextPass`** — UI systems using `EguiContexts` must run in `EguiPrimaryContextPass`, not `Update`.
  `rg 'EguiContexts' --type rust src/` — verify schedule.

- **Raw egui colors instead of theme** — Hardcoded `Color32::from_rgb(...)` instead of `ProteanTheme` palette. Fix: use theme constants.
  `rg 'Color32::from_rgb' --type rust src/`

- **Multi-pass mode is mandatory** — egui widgets (Grid, ComboBox, etc.) need multiple render passes to measure content. This is always-on. Systems in `Update` don't see egui input; all egui UI code must be in `EguiPrimaryContextPass` or custom schedule with `EguiMultipassSchedule`.

- **EguiUserTextures lifecycle leak** — `contexts.add_image(handle)` registers a bevy texture for egui rendering. Using `Strong` handles keeps textures alive even after asset removal. Fix: use `Weak` handles for textures that may be removed, and call `contexts.remove_image(asset_id)` on cleanup.

## leafwing_input_manager

- **`pressed()` for discrete actions** — `pressed()` fires every frame while held. `just_pressed()` fires once. Using `pressed()` for jump/toggle/activate = repeated triggers.
  `rg '\.pressed\(' --type rust src/` — flag discrete actions.

- **Action kind mismatch panics** — `insert_dual_axis()` on a `Button` action panics at runtime. `#[actionlike(Kind)]` must match insert method.

- **Gamepad not set = any gamepad accepted** — Without `input_map.set_gamepad(entity)`, any connected gamepad provides input. Fine for single-player, breaks multiplayer.

- **`VirtualDPad::wasd()` for movement** — Don't create four button actions for WASD. Use one `#[actionlike(DualAxis)]` action with `VirtualDPad::wasd()`. Auto-normalizes diagonals.

- **ClashStrategy for chord bindings** — If `S` and `Ctrl+S` are both bound, default `PrioritizeLongest` prevents `S` from firing during `Ctrl+S`. With `PressAll`, both fire. Forgetting this = unwanted triggers.

- **Missing gamepad bindings** — `InputMap` supports many-to-many: chain `.with(Action, GamepadButton::South)` and `.with_dual_axis(Action, GamepadStick::LEFT)` alongside keyboard bindings. Missing gamepad bindings = broken controller support.
  `rg 'InputMap' --type rust src/` — verify both keyboard and gamepad bindings.

- **ActionState spawned without InputMap** — `InputMap<A>` auto-requires `ActionState<A>` via `#[require]`. Spawning `ActionState` alone = entity that never receives input. Fix: always spawn `InputMap`.
  `rg 'ActionState' --type rust src/`

- **Processing pipeline not applied** — `VirtualDPad`/`GamepadStick` accept `.with_circle_deadzone()`, `.inverted_y()`, etc. Raw analog input without deadzones = drift. Fix: apply at least `with_circle_deadzone(0.1)` on analog inputs.
  `rg 'GamepadStick\|VirtualDPad' --type rust src/`

## bevy_asset_loader

- **Assets unavailable during loading state** — Collections become `Res<T>` only AFTER `continue_to_state` transition. Accessing during loading panics.
  `rg 'GameAssets' --type rust src/` — verify systems run after loading completes.

- **No failure state = app stalls** — Missing `.on_failure_continue_to_state()` + asset load failure = app stuck forever.
  `rg 'LoadingState::new' --type rust src/` — verify failure state configured.

- **`configure_loading_state` for modular plugins** — Don't create multiple `LoadingState`s for the same state. Use `configure_loading_state(LoadingStateConfig::new(State).load_collection::<T>())` from secondary plugins.

- **Assets not in AssetCollection** — `asset_server.load()` for static assets that should be in the `AssetCollection` for preloading.
  `rg 'asset_server\.load\(' --type rust src/`

## bevy_ecs_ldtk

- **`LdtkProjectHandle` wrapper, not raw `Handle`** — Bevy 0.15+ handles can't be components. Use `LdtkProjectHandle`. Query: `With<LdtkProjectHandle>`.
  `rg 'Handle<LdtkProject>' --type rust src/` — flag raw handle usage.

- **Worldly entities survive level despawn** — `#[worldly]` entities are reparented to world and persist across level transitions. Won't respawn if already existing.

- **`LevelEvent::Transformed` = safe for GlobalTransform** — `LevelEvent::Spawned` fires before GlobalTransform propagation. Wait for `Transformed`.
  `rg 'LevelEvent' --type rust src/`

- **Don't set layer z-values manually** — Plugin computes z-ordering (z=0 bg, z=1+ layers). Manually setting Transform.z breaks ordering.

- **Multiple tilemaps per layer** — IntGrid+AutoTile layers produce multiple tilemaps for collision. Don't assume one tilemap per layer.

- **No LevelSelection/LevelSet = nothing spawns** — Must insert `LevelSelection` resource or set `LevelSet` component on world entity.

- **Coordinate origin mismatch** — LDtk uses top-left origin; `GridCoords` uses bottom-left origin. Raw LDtk coordinates without conversion = upside-down positioning. Fix: use `ldtk_grid_coords_to_grid_coords()` or `ldtk_pixel_coords_to_translation()` from `bevy_ecs_ldtk::utils`.
  `rg 'GridCoords' --type rust src/`

- **GridCoords changes don't sync to Transform** — Plugin does NOT auto-update `Transform` when `GridCoords` is modified. Must write a `Changed<GridCoords>` system calling `grid_coords_to_translation()`. Without this, moving entities by modifying GridCoords has no visual effect.

- **SpatialBundle auto-inserted after user bundle** — Plugin inserts `SpatialBundle` AFTER your `#[derive(LdtkEntity)]` bundle. Including Transform/GlobalTransform/Visibility in your bundle = overwritten. Fix: omit spatial components.
  `rg 'derive.*LdtkEntity' --type rust src/`

- **Registration priority chain** — Priority: (1) layer+entity, (2) entity-only, (3) layer-only, (4) default catch-all. `register_default_ldtk_entity` fires for ALL unregistered entities on ALL layers. Same priority chain applies to int cells.

- **ProcessLdtkApi schedule runs after Update** — `LevelSelection` changes in Update are processed in `ProcessLdtkApi` (after Update). Level spawn/despawn has a minimal one-frame delay. `LevelEvent` messages (Spawned, Despawned, Changed) are emitted in this schedule.

## bevy_prototype_lyon

- **Must call `.fill()`/`.stroke()` before `.build()`** — `ShapeBuilder::with(geom)` returns intermediate type. Must call `.fill(color)` and/or `.stroke((color, width))` to get `ReadyShapeBuilder`, then `.build()` for `Shape` component.

- **`Shape` auto-requires `Mesh2d` and `MeshMaterial2d<ColorMaterial>`** — Don't manually add render components. `#[require(...)]` handles it.

- **Tessellation errors logged, not panicked** — Complex paths silently fail. Check logs if shapes don't appear.

- **Geometry trait for shape composition** — Custom shapes implement `Geometry<Builder>` and can be composed via `ShapeBuilder::with(shape1).add(shape2)`. Don't create separate entities for combined shapes when composition works.

## bevy_spatial

- **KDTree fully rebuilt each update** — Not incremental. Large entity counts + short intervals = CPU cost. Tune `with_frequency()`.

- **Timer-based updates = stale results** — Queries return data from last rebuild (default 50ms), not current frame. Moved entities appear at old positions.

- **Marker component required** — Only entities with the marker component are indexed. Missing marker = invisible to spatial queries.
  `rg 'AutomaticUpdate' --type rust src/`

- **`GlobalTransform` mode needs ordering** — With `TransformMode::GlobalTransform`, spatial update must run after transform propagation or positions lag by TWO frames (double-frame delay). Fix: use `TransformMode::Transform` (default) or reorder with `.with_set()`.

- **Query results have Option<Entity>** — `k_nearest_neighbour`, `within_distance`, and `nearest_neighbour` return `(Vec3, Option<Entity>)`. Entity is `Option` because the KDTree stores positions; entity may have been despawned since last rebuild. Fix: always handle `None`.

## bevy_pkv

- **Swallowed PkvStore errors** — `.ok()`, `.unwrap_or_default()`, or `let _ =` on `get`/`set` hides persistence failures. Fix: handle explicitly.
  `rg 'store\.set\|store\.get\|PkvStore' --type rust src/`

- **`GetError::NotFound` vs storage errors** — Must differentiate "key doesn't exist" (expected first run) from actual failures. Don't `.unwrap_or_default()` both.

- **`PersistentResourcePlugin<T>` auto-saves on change** — Resources registered with this plugin auto-save in PostUpdate via `resource_changed::<T>()`. Uses type name as key.

- **Backend serialization format differs** — Native backends use MessagePack (`rmp-serde`) binary format. WASM uses `serde_json` (localStorage limitation). Data is NOT portable between backends. Types must implement `serde::Serialize + Deserialize`.

## bevy_steamworks

- **MUST add before `DefaultPlugins`** — `SteamworksPlugin::init_app(id)` must precede `DefaultPlugins` (which includes `RenderPlugin`). Wrong order = crash.

- **Steam callbacks are Messages, not Events** — Callbacks processed via `MessageReader<SteamworksEvent>`, not `EventReader`. System runs in `First` schedule before `MessageUpdateSystems`. Use `SteamworksSystem::RunCallbacks` set for ordering.
  `rg 'SteamworksEvent' --type rust src/`

## bevy-inspector-egui

- **Requires `register_type::<T>()`** — Types must be registered AND derive `Reflect` with `#[reflect(Component)]`/`#[reflect(Resource)]` to appear in inspector.

- **Missing DefaultInspectorConfigPlugin** — Inspector UI requires `DefaultInspectorConfigPlugin` registered before any inspector plugins. Without it, built-in types (Vec2, Quat, etc.) don't render. Add before `WorldInspectorPlugin` or `ResourceInspectorPlugin`.
  `rg 'InspectorPlugin\|WorldInspectorPlugin' --type rust src/`

- **Clone egui context for world access** — Manual inspector UI via `bevy_inspector::ui_for_world(world, ui)` requires `&mut World`, but the egui context is also part of the world. Fix: clone the context before calling world-mutating inspector functions.

## bevy_framepace

- **Disabled on WASM** — `spin_sleep` unavailable. Plugin is a no-op on WASM.

---

## Cross-Crate

- **Manual nearest-neighbor instead of spatial index** — Iterating all entities with `.distance()` when `KDTree2::k_nearest_neighbour` or `within_distance` is available.
  `rg '\.distance\(' --type rust src/combat/` cross-ref with `rg 'k_nearest\|within_distance' --type rust src/`

- **Duplicate functionality across crates** — Using rapier intersection AND manual distance checks, or bevy_spatial AND manual iteration for same purpose. Fix: use one approach.

- **Ecosystem crate prelude not used** — Importing individual types from a crate that provides a prelude (e.g., `use bevy_rapier2d::dynamics::RigidBody` instead of `use bevy_rapier2d::prelude::*`). Fix: use the prelude.
  `rg 'use bevy_rapier2d::(?!prelude)\|use bevy_hanabi::(?!prelude)\|use bevy_ecs_ldtk::(?!prelude)' --type rust src/`
  Exception: types not in the prelude, or aliasing to avoid name conflicts.

- **Ecosystem crate version/branch mismatch** — An ecosystem crate pinned to a branch or tag incompatible with the current bevy version (0.17.3). Fix: verify compatibility.
  `rg 'bevy_' Cargo.toml`
