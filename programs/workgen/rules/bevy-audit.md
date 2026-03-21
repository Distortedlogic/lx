# Bevy Codebase Quality Audit

Every item below is a binary check — a violation either exists or it does not. The audit checks each item across all `.rs` files in `src/`. This codebase uses Bevy 0.17+ with Messages, `#[require]`, `ChildOf`, and the new query ergonomics.

Run the **High Frequency** list first — these violations are commonly introduced by both humans and AI agents. Run the **Low Frequency** list second — these are rarer structural issues.

---

## High Frequency Checks

### Plugin & App Architecture

- **Monolithic plugin** — A single `Plugin::build` method that registers 15+ systems, 10+ resources, or spans 80+ lines. Fix: split into focused sub-plugins grouped by responsibility (e.g., `CombatSpawningPlugin`, `CombatVisualsPlugin`), compose them in a parent plugin's `build` via `app.add_plugins((...))`.
  `rg 'impl Plugin for' --type rust src/ -A 50`
  For each plugin: count systems registered, resources initialized, and total lines. Flag if thresholds exceeded.

- **Systems registered in wrong schedule** — A system that reads `Time`, moves entities, or runs game logic registered in `PostUpdate`/`Last` instead of `Update`. A system that updates visual transforms or syncs UI registered in `Update` instead of `PostUpdate`. Fix: move to the correct schedule. Game logic in `Update`, visual sync in `PostUpdate`, one-time setup in `Startup`.
  `rg 'add_systems\(' --type rust src/`
  For each: check if the schedule matches the system's responsibility.

- **Missing run conditions** — A system that should only run in a specific `GameState` or `InCombat` state has no `.run_if()` guard. Fix: add `run_if(in_state(...))` or attach to a `SystemSet` that already has the condition.
  `rg 'add_systems\(' --type rust src/`
  For each: check if the system accesses state-specific resources/components but lacks a state guard.

- **Duplicated run conditions** — The same `.run_if(...)` chain is repeated on 3+ `add_systems` calls instead of being applied once to a `SystemSet` that all those systems belong to. Fix: create a `SystemSet`, configure it once with the run condition, and assign systems to the set.
  `rg '\.run_if\(' --type rust src/`
  Count identical run condition chains. Flag if 3+ duplicates.

- **System ordering without data dependency** — `.chain()` or `.before()`/`.after()` on systems that don't actually have a data dependency (no shared mutable resource, no event writer→reader relationship). Unnecessary ordering prevents parallelism. Fix: remove the ordering constraint.
  `rg '\.chain\(\)' --type rust src/`
  For each chain: verify adjacent systems share mutable state or have event writer→reader ordering.

- **Missing system ordering with data dependency** — Two systems in the same schedule that write and read the same resource/component but have no ordering constraint. This causes nondeterministic behavior. Fix: add `.chain()`, `.before()`/`.after()`, or assign to ordered `SystemSet`s.
  Manual review: for each `ResMut` or `Query<..., &mut ...>`, check if another system in the same set reads the same data without ordering.

### Components & Resources

- **Component without `#[reflect(Component)]`** — A `#[derive(Component)]` without `#[reflect(Component)]` prevents the inspector and scene serialization from working. Fix: add `Reflect` derive and `#[reflect(Component)]`. Exception: marker components with no fields where reflection is genuinely unnecessary.
  `rg '#\[derive\(Component' --type rust src/`
  For each: check if `#[reflect(Component)]` follows. Flag if missing on components with fields.

- **Resource without `#[reflect(Resource)]`** — Same issue for resources. Fix: add `Reflect` derive and `#[reflect(Resource)]`.
  `rg '#\[derive\(.*Resource' --type rust src/`
  `rg '#\[reflect\(Resource\)\]' --type rust src/`

- **`insert_resource(T::default())` instead of `init_resource::<T>()`** — Verbose and redundant when the type implements `Default`. Fix: use `init_resource::<T>()`.
  `rg 'insert_resource\(.*::default\(\)\)' --type rust src/`

- **Missing `#[require]` for mandatory component bundles** — A component is always inserted alongside 2+ other components, but the primary component doesn't use `#[require(...)]` to declare the dependency. Fix: add `#[require(ComponentA, ComponentB)]` on the primary component and remove explicit insertion of the required components at spawn sites.
  `rg '#\[require\(' --type rust src/`
  For each `commands.spawn((...))` or `.insert((...))`: check if the tuple contains components that the primary component already requires. Flag redundant insertions.

- **Newtype wrapper missing `Deref`/`DerefMut` derives** — A tuple struct wrapping a single field (e.g., `struct Foo(pub Bar)`) that is accessed via `.0` at call sites instead of deriving `Deref` and `DerefMut` from Bevy's prelude. This forces verbose `.0.field` / `.0.method()` patterns everywhere the wrapper is used. Fix: add `Deref, DerefMut` to the derive list. Both are re-exported in `bevy::prelude::*`. Remove any manual accessor methods (e.g., `fn inner(&self) -> &T { &self.0 }`) that become dead code after adding the derives. Update all call sites to drop `.0`. Exception: newtypes that intentionally restrict the inner type's API surface (only exposing a subset of methods).
  `rg '\.0\.' --type rust src/`
  For each `.0.` access: trace back to the struct definition. Flag if it's a single-field tuple struct without `Deref`/`DerefMut` derives.

- **Redundant component insertion covered by `#[require]`** — A spawn or insert call explicitly includes components that are already required by another component in the same bundle. Fix: remove the redundant components from the spawn/insert tuple.
  `rg '#\[require\(' --type rust src/ -A 2`
  Cross-reference with spawn sites to find redundant insertions.

- **`commands.spawn((...))` missing `Name` component** — An entity spawned without a `Name` component, making it unidentifiable in the inspector and debug tools. Fix: add `Name::new("descriptive name")` to the spawn bundle. Exception: short-lived particle/effect entities where naming overhead is unjustified.
  `rg 'commands\.spawn\(' --type rust src/`
  `rg '\.spawn\(' --type rust src/`
  For each: check if `Name` is in the bundle.

### Queries & System Parameters

- **Overly broad query** — A query fetches components it never reads (e.g., `Query<(&Transform, &Health, &Enemy)>` but only uses `Transform`). Fix: remove unused components from the query. Unused fetched components cause unnecessary archetype matching and cache pressure.
  Manual review: for each `Query<(...)>`, verify every fetched component is used in the system body.

- **Query filter that should be a `With`/`Without`** — A query fetches a component reference (`&Component`) only to check its existence or type, never reading its fields. Fix: move it to a `With<Component>` filter.
  `rg 'Query<.*&' --type rust src/`
  For each queried component reference: check if its fields are accessed. Flag if only used for existence checking.

- **`query.iter()` when `query.single()` or `Single<>` suffices** — A query that is guaranteed to have exactly one result (e.g., the player entity, the camera) iterated with `for ... in &query` instead of using `Single<>` system parameter or `query.single()`. Fix: use `Single<>` as a system parameter or `query.single()`.
  `rg 'for .* in &.*query' --type rust src/`
  For each: check if the query logically has exactly one match (player, camera, etc.). Flag if so.

- **`query.single()` instead of `Single<>` parameter** — `Single<>` as a system parameter is more ergonomic and makes the system signature self-documenting. Fix: replace `query.single()` patterns with `Single<>` system parameter. Wrap in `Option<Single<>>` when the entity may not exist.
  `rg '\.single\(\)' --type rust src/`
  `rg '\.single_mut\(\)' --type rust src/`

- **Missing `Changed<T>` or `Added<T>` filter** — A system iterates all entities with a component every frame but only needs to act when the component changes. Fix: add `Changed<T>` or `Added<T>` query filter to avoid processing unchanged entities.
  Manual review: for each system that iterates a query and checks a condition based on component values, determine if `Changed<T>` would avoid unnecessary iteration.

- **Explicit lifetime annotations on queries** — Writing `Query<'_, '_, (...)>` with explicit lifetime placeholders where Bevy's lifetime elision handles it. Fix: use `Query<(...)>` without explicit lifetimes in system parameters where the compiler allows it. Note: the current codebase uses explicit lifetimes consistently — flag this as a codebase-wide cleanup opportunity, not per-instance.
  `rg "Query<'_" --type rust src/`

- **Conflicting `&mut` queries without `Without` filters** — Two or more `Query` parameters in the same system both mutably access a component (e.g., `&mut Health`) but are distinguished only by `With<A>` vs `With<B>` filters. Bevy cannot prove disjointness from `With` alone and panics at runtime with error B0001. Fix: add `Without<B>` to the first query and `Without<A>` to the second so Bevy can statically guarantee no overlap.
  `rg '&mut ' --type rust src/`
  For each system with 2+ Query parameters: check if any mutable component appears in more than one query. Flag if the queries lack `Without` filters to guarantee disjointness.

- **`SystemParam` with unnecessary lifetime parameters** — A custom `#[derive(SystemParam)]` struct declares lifetime parameters it doesn't need, or uses `'s` when only `'w` is required. Fix: remove unnecessary lifetime parameters.
  `rg '#\[derive\(SystemParam\)\]' --type rust src/ -A 5`

### Events & Messages

- **`Event` where `Message` should be used** — An event type derives `Event` but is used with `MessageWriter`/`MessageReader` (Bevy 0.17 messages). Messages should derive `Message`, not `Event`. Fix: replace `#[derive(Event)]` with `#[derive(Message)]` for types used via message APIs, and register with `app.add_message::<T>()` instead of `app.add_event::<T>()`.
  `rg '#\[derive.*Event' --type rust src/`
  `rg 'MessageWriter|MessageReader' --type rust src/`
  Cross-reference: types used with MessageWriter/MessageReader should derive Message, not Event.

- **`Event` not registered with `add_event`** — An event type is defined but never registered with `app.add_event::<T>()` or `app.add_message::<T>()`. Fix: register it in the appropriate plugin.
  `rg '#\[derive.*Event.*\]' --type rust src/`
  `rg 'add_event::<' --type rust src/`
  `rg 'add_message::<' --type rust src/`
  Cross-reference: every Event/Message type must have a registration call.

- **Event reader in wrong system order** — A system that reads events via `EventReader` or `MessageReader` runs before the system that writes them, causing events to be missed until the next frame. Fix: ensure writers run before readers via `.chain()` or `SystemSet` ordering.
  `rg 'MessageReader|EventReader' --type rust src/`
  `rg 'MessageWriter|EventWriter' --type rust src/`
  For each event type: verify writer systems are ordered before reader systems.

### State Management

- **`NextState::set()` called without checking current state** — Setting a state unconditionally when the current state might already be the target, causing unnecessary exit/enter system runs. Fix: check `state.get() != target` before calling `next_state.set(target)`, or use state transition events that already guard against this.
  `rg 'next_state\.set\(' --type rust src/`

- **State-dependent resource accessed without state guard** — A resource that is only meaningful in a specific state (e.g., `ResponseState` only during `GameState::Response`) accessed by a system without `run_if(in_state(...))`. Fix: add the state guard.
  `rg 'Res<.*ResponseState\|Res<.*SurgeSpawner\|Res<.*CombatResources' --type rust src/`
  For each: verify the system has an appropriate state run condition.

- **Manual state tracking instead of `ComputedStates`** — A boolean resource or component tracks whether a game phase is active, duplicating what a `ComputedStates` type would provide. Fix: define a `ComputedStates` type that derives its value from the source `States`.
  `rg 'ComputedStates' --type rust src/`
  Manual review: look for boolean resources that mirror state membership.

### Entity Lifecycle

- **`commands.entity(e).despawn()` instead of `try_despawn()`** — `despawn()` panics if the entity doesn't exist. In systems processing events or deferred operations, the entity may already be despawned. Fix: use `try_despawn()` which silently handles missing entities.
  `rg '\.despawn\(\)' --type rust src/`
  Flag any `despawn()` that isn't `try_despawn()`.

- **`despawn_recursive` instead of `try_despawn`** — In Bevy 0.17, `try_despawn()` already handles children. `despawn_recursive()` is deprecated/redundant. Fix: use `try_despawn()`.
  `rg 'despawn_recursive' --type rust src/`

- **Entity stored as field but never validated** — A component stores an `Entity` reference (e.g., `target: Entity`) but never checks if the entity still exists before using it in queries. Fix: use `query.get(entity)` with proper `Ok`/`Err` handling, or use `Option<Entity>` with validation.
  `rg 'Entity' --type rust src/`
  For each Entity field in a component: verify that systems using it handle the case where the entity no longer exists.

- **Spawning children with `commands.spawn()` + `add_child()` instead of `with_child()`/`with_children()`** — Manually spawning then attaching children is verbose and error-prone. Fix: use `.with_child(bundle)` for single children or `.with_children(|builder| { ... })` for multiple.
  `rg 'add_child\(' --type rust src/`

### Timers & Time

- **`Timer::from_seconds` with wrong `TimerMode`** — A timer intended to fire once uses `TimerMode::Repeating`, or a timer intended to repeat uses `TimerMode::Once`. Fix: match the `TimerMode` to the intended behavior.
  `rg 'Timer::from_seconds' --type rust src/`
  For each: verify the TimerMode matches usage (`.just_finished()` in a loop = Repeating, `.is_finished()` for one-shot = Once).

- **`time.delta_secs()` called multiple times** — The same `time.delta_secs()` called repeatedly in a system instead of binding to a `let dt`. Fix: bind once at the top of the system.
  `rg 'time\.delta_secs\(\)' --type rust src/`
  For each system: count calls. Flag if 2+.

- **Timer not ticked** — A `Timer` component or resource is checked with `.just_finished()` or `.is_finished()` but never `.tick(time.delta())` in the same system or a system that runs before it. Fix: ensure every timer is ticked before being checked.
  `rg '\.just_finished\(\)|\.is_finished\(\)' --type rust src/`
  For each: verify `.tick(` is called on the same timer.

- **Manual elapsed tracking instead of `Timer`** — A `f32` field manually decremented by `dt` each frame to implement timeout/cooldown behavior, when `Timer` provides the same functionality with `TimerMode`, `just_finished()`, and `percent()`. Fix: replace with `Timer`. Exception: trivial one-liner countdowns where `Timer` would be overweight.
  `rg 'remaining.*-=.*dt\|remaining.*-=.*delta' --type rust src/`

### Transform & Hierarchy

- **`Transform` and `GlobalTransform` confusion** — Reading `Transform` when `GlobalTransform` is needed (entity has a parent, so local transform != world position), or writing to `GlobalTransform` which is computed automatically. Fix: read `GlobalTransform` for world-space position of parented entities, write to `Transform` for local-space changes.
  `rg 'GlobalTransform' --type rust src/`
  Manual review: verify parented entities use `GlobalTransform` for world-space reads.

- **`ChildOf` query without `Without` filter** — A system queries both parent and child entities with overlapping component types but doesn't use `Without<ChildOf>` or similar filters to prevent aliasing. Fix: add appropriate `Without` filters.
  `rg 'ChildOf' --type rust src/`

### Performance Patterns

- **`query.iter().count()` for existence check** — Using `.iter().count() > 0` or `.iter().count() == 0` instead of `query.is_empty()` or `!query.is_empty()`. Fix: use `query.is_empty()`.
  `rg '\.iter\(\)\.count\(\)' --type rust src/`

- **Per-frame allocation in hot system** — A `Vec::new()`, `String::new()`, `HashMap::new()`, or `format!()` inside a system that runs every frame. Fix: use `Local<Vec<T>>` with `.clear()`, pre-allocate with `Local<T>`, or restructure to avoid allocation.
  `rg 'Vec::new\(\)|Vec::with_capacity|HashMap::new\(\)|String::new\(\)' --type rust src/`
  For each: check if the system runs every frame (no `Changed`/`Added` filter, no timer guard).

- **`.collect::<Vec<_>>()` for immediate iteration** — An iterator collected into a `Vec` only to be iterated again. Fix: chain the iterator directly. Exception: collecting is required to release a borrow before mutating (common with `Query`).
  `rg '\.collect::<Vec' --type rust src/`
  For each: check if the Vec is immediately iterated and whether the collect is needed for borrowck.

- **Distance checks using `distance()` instead of `distance_squared()`** — Using `.distance()` (which computes sqrt) in hot loops for comparison against a threshold. Fix: compare `distance_squared()` against `threshold * threshold`.
  `rg '\.distance\(' --type rust src/`
  For each in a loop: check if the result is only compared to a threshold. Flag if sqrt is unnecessary.

- **Hash-based determinism instead of `bevy_rand`** — Using manual hash-based pseudo-randomness (e.g., `(x * 1000.0 + y * 7919.0) as u32 % 1000`) instead of the `bevy_rand` `GlobalRng` or per-entity `EntropyComponent`. Hash-based approaches are fragile, poorly distributed, and not reproducible across platforms. Fix: use `bevy_rand`'s `GlobalRng` resource or attach `EntropyComponent` to entities needing randomness.
  `rg 'as u32 % |as usize %' --type rust src/`
  For each: check if this is pseudo-random number generation. Flag if so.

### Ecosystem Plugin Usage

See `rules/bevy-ecosystem-audit.md` for the full ecosystem crate audit (bevy_rand, bevy_kira_audio, bevy_hanabi, bevy_tweening, bevy_rapier2d, bevy_egui, leafwing_input_manager, bevy_asset_loader, bevy_ecs_ldtk, bevy_prototype_lyon, bevy_spatial, bevy_pkv, bevy_steamworks, bevy-inspector-egui, bevy_framepace). The items below are the highest-signal cross-checks.

- **Raw `bevy_egui` context access without theme** — Using `bevy_egui`'s `EguiContexts` to render UI without applying the project's `ProteanTheme`. Fix: ensure `ProteanTheme::apply(ctx)` is called before rendering.
  `rg 'EguiContexts|egui::Context' --type rust src/`

- **Direct physics API instead of component-based** — Using `bevy_rapier2d` by directly calling physics APIs instead of inserting physics components (`Collider`, `RigidBody`, `Sensor`, `KinematicCharacterController`). Fix: use the component-based API.
  `rg 'rapier' --type rust src/`

- **`leafwing-input-manager` action checked without `just_pressed`** — Using `pressed()` where `just_pressed()` is needed (discrete actions like toggling build mode, using abilities). `pressed()` fires every frame the button is held. Fix: use `just_pressed()` for discrete actions, `pressed()` only for continuous actions (movement).
  `rg '\.pressed\(&' --type rust src/`
  For each: check if the action is discrete (toggle, fire, activate). Flag if `just_pressed` should be used.

---

## Low Frequency Checks

- **God resource** — A single `Resource` struct with 8+ fields that tracks unrelated concerns. Fix: split into multiple focused resources. Each resource should represent one coherent piece of state.
  `rg '#\[derive\(.*Resource' --type rust src/ -A 15`
  For each resource: count fields. Flag if 8+ and fields serve different concerns.

- **Component used as resource** — A type derives both `Component` and `Resource`, or a `Component` is only ever attached to a single global entity used as a pseudo-resource. Fix: make it a `Resource` if it's global state, or a `Component` if it's per-entity.
  `rg '#\[derive\(.*Component.*Resource\|#\[derive\(.*Resource.*Component' --type rust src/`

- **Marker component with fields** — A component named `*Marker` that has fields beyond what a zero-sized marker should have. Fix: if it has data, remove `Marker` from the name. If it's truly a marker, remove the fields.
  `rg 'Marker' --type rust src/`
  For each Marker component: check if it has fields.

- **System with 8+ parameters** — A system function signature with 8+ parameters, indicating it does too much. Fix: split into multiple systems, or group related parameters into a custom `SystemParam`.
  `rg '^pub fn \w+\(' --type rust src/ -A 10`
  Count parameters for each system function.

- **`Local<Option<T>>` instead of proper initialization** — Using `Local<Option<T>>` with `get_or_insert_with` pattern for lazy initialization. Fix: use `FromWorld` to initialize the resource/local at system registration time, or use a dedicated initialization system in `Startup`.
  `rg 'Local<.*Option' --type rust src/`

- **Unused `SystemSet`** — A `SystemSet` enum variant defined but never referenced in `configure_sets` or `.in_set()`. Fix: remove the unused variant.
  `rg 'SystemSet' --type rust src/`
  For each variant: `rg 'VariantName' --type rust src/` — flag if only the definition appears.

- **Plugin registers systems for components it doesn't own** — A plugin registers systems that query components defined in a different module's plugin, creating a hidden dependency. Fix: move the system to the plugin that owns the components, or make the dependency explicit via plugin ordering.
  Manual review: for each plugin's systems, verify the queried components are defined or re-exported by the same module.

- **Orphaned cleanup** — An `OnExit` system that despawns entities or resets resources, but the corresponding `OnEnter` system that spawns/initializes them is in a different plugin. Fix: co-locate enter/exit systems in the same plugin.
  `rg 'OnEnter\|OnExit' --type rust src/`
  For each OnExit: verify the matching OnEnter is in the same plugin.

- **`Changed` filter on component that changes every frame** — Using `Changed<T>` on a component like `Transform` that is updated every frame by the engine, defeating the purpose of the filter. Fix: use a custom component that only changes when logically meaningful, or remove the `Changed` filter.
  `rg 'Changed<Transform>|Changed<GlobalTransform>' --type rust src/`

- **Spawn-site component bundle inconsistency** — The same logical entity type (e.g., enemies, towers) is spawned in multiple places with different component sets. Fix: create a builder function or use `#[require]` to ensure consistent bundles.
  `rg '\.spawn\(' --type rust src/`
  For each entity type spawned in 2+ places: compare the component sets. Flag inconsistencies.
