# Goal

Fix Dioxus framework violations: #[server] instead of HTTP method macros, use_resource instead of use_loader, use_effect reacting to use_action, tracing instead of Dioxus logger, wrapper component forwarding props, string interpolation mixed with static classes, Signal where Store needed, and free function accessing struct fields.

# Why

`#[server]` defaults to POST and loses semantic clarity — should use `#[get]` for data retrieval. `use_resource` should be `use_loader` for SSR-serialized data loading. `use_effect` watching `use_action` state is an anti-pattern — await the action result inside the event handler's spawn block. `tracing::info` should use the built-in Dioxus logger. The Wrapper component only forwards children with no added logic — inline it. The class attribute mixes `{nav_width}` interpolation with static classes in one string — split into separate class attributes. AppState with 2+ fields uses `Signal` implicitly where `Store` with `#[derive(Store)]` would enable granular subscriptions. `format_item` is a free function accessing `AppState` fields — should be a method. `use_memo` wrapping a plain store field read is redundant.

# What changes

- Replace `#[server]` with `#[get]` on `get_data`
- Replace `use_resource` with `use_loader` for data loading
- Remove `use_effect` reacting to `use_action` — await action result in spawn block
- Replace `tracing::info` with Dioxus built-in logger
- Remove Wrapper component — inline the div at call sites
- Split mixed class attribute into separate static and interpolated class attributes
- Add `#[derive(Store)]` to AppState for granular field subscriptions
- Move `format_item` into an impl block on AppState
- Remove redundant `use_memo` wrapping plain store field read

# Files affected

- src/app.rs — #[server] attribute, use_resource, use_effect+use_action, tracing logger, wrapper component, mixed class interpolation, Signal vs Store, free function, redundant use_memo

# Task List

## Task 1: Fix server function and data loading

Replace `#[server]` with `#[get]`. Replace `use_resource` with `use_loader`.

```
just fmt
git add src/app.rs
git commit -m "fix: use explicit HTTP method, use_loader for data loading"
```

## Task 2: Fix hooks and logging

Remove `use_effect` reacting to `use_action`. Replace tracing with Dioxus logger. Remove redundant use_memo on store field.

```
just fmt
git add src/app.rs
git commit -m "fix: remove use_effect+use_action anti-pattern, use Dioxus logger"
```

## Task 3: Fix component and class patterns

Remove Wrapper component. Split mixed class interpolation. Add Store derive. Move free function to method.

```
just fmt
git add src/app.rs
git commit -m "fix: inline wrapper, split class attrs, Store derive, method"
```

## Task 4: Verification

```
just test
just diagnose
just fmt
git add -A
git commit -m "chore: verify dioxus audit fixes"
```

# CRITICAL REMINDERS

- Run `just fmt` after every file change
- Run `just test` and `just diagnose` before final commit
- No #[server] — use explicit HTTP method macros
- No use_resource in fullstack — use use_loader
- No use_effect reacting to use_action

# Task Loading Instructions

Load these instructions by reading this file, then execute each task in order. After each task, run `just fmt` and commit.
