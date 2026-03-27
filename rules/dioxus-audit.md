# Dioxus Codebase Audit

## Store API

- **Verify Store API utilization** - Verify the repo fully utilizes the Dioxus Store API.
  `rg 'use_store|#\[derive\(Store\)\]|Store<' --type rust crates/`
  `grit apply '`#[derive($traits)]`where { $traits <: contains`Store` }' crates/ --language rust --dry-run`
- **Lens impls vs struct impls** - Ensure lens impls (`impl<Lens> Store<T, Lens>`) are used for methods that access store field accessors (`.field_name()`) or call `.read()`/`.write()` on sub-stores. Struct impls (`impl T`) are only for pure data logic with no store interaction.
  `rg 'impl.*Store<' --type rust crates/`
  `rg '\.(read|write)\(\)' --type rust crates/`
- **Signal where Store needed** - Look for `Signal<T>` usage on types with 2+ fields that are accessed at field granularity by different components. These should be `Store<T>` with `#[derive(Store)]` instead.
  `grit apply '`$field: Signal<$inner>`' crates/ --language rust --dry-run`
  `rg 'Signal<' --type rust crates/`
  Cross-reference with: `grit apply '`#[derive($traits)]`where { $traits <: contains`Store` }' crates/ --language rust --dry-run`

## dioxus-primitives

- **Verify primitives utilization** - Verify the repo fully utilizes the dioxus-primitives components. Read `docs/dioxus-primitives-ref.md` for the full component catalog.
  `rg 'dioxus.primitives|use dioxus' --type rust crates/`
- **Custom code duplicating primitives** - Ensure there is no custom code or custom components where a dioxus-primitive component already covers the use case.
  `grit apply '`fn $name($props) -> Element { $body }`' crates/ --language rust --dry-run`
  `rg '#\[component\]' --type rust crates/`
  RSX element content is opaque to GritQL — use `rg` for element-level matching inside `rsx!` blocks.

## Server-Side State Management

- **LazyLock for non-per-request state** - Any state that does not need to be created per request must be in a `LazyLock` if creation is not async, or in a Dioxus `Lazy` if creation needs async. `LazyLock` must be used directly as a `static` — no wrapper structs, newtypes, or helper functions that merely forward to a `LazyLock`. The `LazyLock` itself is the abstraction.
  `rg 'LazyLock|once_cell' --type rust crates/`
  `rg 'static .+:' --type rust crates/`
- **Extension extractor only for per-request state** - State must ONLY be an extension extractor if it needs to be created per request.
  `rg 'Extension|FromRequestParts|extract' --type rust crates/`

## Server Functions

- **Explicit HTTP method macros** - All server functions must use the explicit `get`/`post`/etc method macros instead of `#[server]`.
  `grit apply '`#[server]`' crates/ --language rust --dry-run`
  `rg '#\[server\b' --type rust crates/`
  `rg '#\[(get|post|put|delete)\b' --type rust crates/`

## Dioxus Re-exports

- **Use dioxus re-exports** - The repo must use re-exports from the `dioxus` crate itself instead of directly installing or importing crates that are re-exported through `dioxus`.
  `rg 'use dioxus_' --type rust crates/`
  `rg 'use dioxus::' --type rust crates/`

## Logging

- **Built-in Dioxus logger only** - ALL logging in the repo must use the built-in Dioxus logger. No other logging crate should be used directly.
  `rg 'tracing::|log::|env_logger|tracing_subscriber' --type rust crates/`
  `rg 'dioxus::logger' --type rust crates/`

## Component Design

- **Wrapper component forwarding props** - a `#[component]` whose body renders exactly one child component and passes through all received props without adding logic, state, or layout. Fix: remove the wrapper component and use the child component directly at all call sites.
  `rg '#\[component\]' --type rust crates/`
  For each component: read the body. If it contains only a single `rsx!` with a single child component and no hooks, signals, or conditional logic, it is a pure wrapper.

- **Single-use component** - a `#[component]` that is rendered at exactly one call site. Fix: inline the RSX body into the parent. Exception: inlining would push the parent file over 300 lines.
  `rg '#\[component\]' --type rust crates/`
  For each component: `rg 'ComponentName {' --type rust crates/` — if count is 1, inline.

## Frontend Structure

- **Router in own file** - The router must be in its own file.
  `rg '#\[derive\(Routable\)\]' --type rust crates/`
- **Layout in own file** - The layout must be in its own file.
  `rg 'Layout|Outlet' --type rust crates/`
- **App component in own file** - The main `App` component must be in its own file.
  `grit apply 'function_item(name = $n) where { $n <: `App` }' crates/ --language rust --dry-run`
  `rg 'fn App\b' --type rust crates/`

## Hooks

- **use_action for event handlers** - All Dioxus components that call server functions or async operations from event handlers must use `use_action` instead of spawning tasks manually or using `use_future` for user-triggered operations. Do not combine `use_action` with `use_effect` to react to action completion — await the action result inside a `spawn` block within the event handler that triggered it.
  `grit apply '`fn $n($p) -> Element { $b }` where { $b <: contains `spawn` }' crates/ --language rust --dry-run`
  `grit apply '`use_effect($arg)`' crates/ --language rust --dry-run`
  `rg 'use_action|use_future|spawn' --type rust crates/`
- **use_loader for data loading** - `use_loader` is the default for ALL data loading in fullstack apps. It is SSR-serialized and suspense-compatible. `use_resource` must not be used for data that can be loaded with `use_loader`. `use_server_function` is acceptable only when `use_loader`'s suspense behavior is explicitly undesirable.
  `grit apply '`use_resource($arg)`' crates/ --language rust --dry-run`
  `rg 'use_resource|use_loader|use_server_future|use_server_function' --type rust crates/`

## Hook Misuse

- **No use_resource in fullstack apps** - Detect `use_resource` in fullstack apps. Replace with `use_loader` for data loading (SSR-serialized, suspense-compatible) or `use_action` for user-triggered mutations. `use_resource` is only acceptable for client-only reactive computations that genuinely cannot use `use_loader`. `use_server_function` is acceptable only when `use_loader`'s suspense behavior is explicitly undesirable — flag for manual review.
  `grit apply '`use_resource($arg)`' crates/ --language rust --dry-run`
  `rg 'use_resource' --type rust crates/`
- **No use_effect reacting to use_action** - Detect `use_effect` that reads `use_action` state to trigger side effects when an action completes. This is an anti-pattern. Instead, await the `use_action` result inside a `spawn` block within the event handler that triggered the action. This keeps cause-and-effect co-located.
  `grit apply '`fn $n($p) -> Element { $b }`where { $b <: contains`use_effect`, $b <: contains `use_action` }' crates/ --language rust --dry-run`
  `rg 'use_effect' --type rust crates/`

## Store Memo Redundancy

- **No use_memo wrapping a plain Store field read** - Detect `use_memo(move || store_field())` where `store_field` is a `Store` handle (e.g., from `transpose()` destructuring or a field accessor like `store.name()`). A `Store` is already reactive — it implements `Readable`, tracks subscriptions, implements `Copy`, and can be used directly in RSX. Wrapping a plain store field read in `use_memo` adds pointless indirection. Use the `Store` directly.
- **Memoize store impl method calls that derive values** - Store lens impl methods (methods in `#[store] impl` blocks) are NOT memoized — they re-execute their full computation on every call with no caching or value deduplication. Wrap these in `use_memo` when: (1) the method iterates a collection or performs non-trivial computation (e.g., `store.incomplete_count()`, `store.active_items()`), (2) the result feeds conditional rendering or child props, or (3) the component has multiple reactive sources causing unrelated re-renders that would redundantly re-execute the method. The canonical pattern from the official Dioxus `todomvc_store` example is `use_memo(move || store.computed_method())`. Do NOT memoize trivially cheap single-field reads or methods that simply forward a store value without computation.
  `grit apply '`use_memo(move || $body)`' crates/ --language rust --dry-run`
  `grit apply '`fn $n($p) -> Element { $b }`where { $b <: contains`use_memo`, $b <: contains `transpose` }' crates/ --language rust --dry-run`
  `rg 'use_memo' --type rust crates/`

## Store API Granularity

- **Nested structs must derive Store** - Detect structs used as fields of a `#[derive(Store)]` struct that have 2+ fields but do not themselves derive `Store`. All nested structs in store hierarchies must derive `Store` to enable granular field-level subscriptions on nested data. Without this, accessing any field of the nested struct triggers a subscription on the entire nested value.
  `grit apply '`#[derive($traits)]`where { $traits <: contains`Store` }' crates/ --language rust --dry-run`
  `rg '#\[derive\(Store\)\]' -A 10 --type rust crates/`
  Extract field types, then verify each struct type also has `#[derive(Store)]`.
- **No .read()/.cloned() for partial field access** - Detect `.read()` or `.cloned()` on a `Store` when only a subset of fields is needed. Use field accessor methods (e.g., `my_store.field_name()` returns `Store<FieldType, _>`) or `transpose()` for multi-field destructuring. This ensures components only re-render when the specific fields they use change, not when any field changes.
  `grit apply '`$x.read().$method()`' crates/ --language rust --dry-run`
  `grit apply '`$x.cloned()`' crates/ --language rust --dry-run`
  `rg '\.(read|cloned)\(\)' --type rust crates/`

## Store API Convenience

- **Lens methods for deep access** - Detect deeply nested store access patterns (e.g., `store.a().b().c()`) repeated across multiple components. The parent store should have an `impl` block with the `#[store]` attribute providing a lens method that returns the nested store directly (e.g., `#[store] impl MyStore { fn deep_thing(&self) -> &DeepType { &self.a.b.c } }`).
  `rg '#\[store\]' --type rust crates/`
  `rg -o '\w+\(\)\.\w+\(\)\.\w+\(\)' --type rust --no-filename crates/ | sort | uniq -c | sort -rn | head -20`
- **Direct store methods instead of .read().method()** - Detect `.read().len()`, `.read().is_empty()`, `.cloned().len()`, `.cloned().is_empty()`, or similar patterns on `Store<Vec<T>>`, `Store<Option<T>>`, `Store<HashMap<K,V>>`, `Store<Result<T,E>>`. Call the method directly on the store (e.g., `store.len()`, `store.is_empty()`, `store.is_some()`, `store.is_ok()`). The Store API provides these methods directly with better subscription granularity.
  `grit apply '`$x.read().len()`' crates/ --language rust --dry-run`
  `grit apply '`$x.read().is_empty()`' crates/ --language rust --dry-run`
  `grit apply '`$x.cloned().len()`' crates/ --language rust --dry-run`
  `rg '\.read\(\)\.(len|is_empty|is_some|is_none|is_ok|is_err)\(' --type rust crates/`
  `rg '\.cloned\(\)\.(len|is_empty|is_some|is_none|is_ok|is_err)\(' --type rust crates/`
- **store() call syntax instead of .cloned()** - Detect `.cloned()` calls on stores. Replace with the `store()` call syntax which is the idiomatic shorthand for the same operation. Exception: when the call site would result in double parentheses `()()` that harm readability, `.cloned()` is acceptable.
  `grit apply '`$x.cloned()`' crates/ --language rust --dry-run`
  `rg '\.cloned\(\)' --type rust crates/`

## Server-Side Anti-Patterns

- **No #[server] attribute** - Detect `#[server]` attribute on server functions. Replace with the appropriate HTTP method macro based on operation semantics: `#[get]` for data retrieval, `#[post]` for creation/mutations, `#[put]` for updates, `#[delete]` for deletion. `#[server]` defaults to POST and loses semantic clarity.
  `rg '#\[server\b' --type rust crates/`
- **No manual ServerFnError construction** - Detect manual `ServerFnError` construction or `.map_err(|e| ServerFnError::...)` chains in server functions. Use `anyhow::Result` (or `thiserror` for structured errors) with the `?` operator — errors propagate directly to the server function boundary without manual conversion.
  `rg 'ServerFnError' --type rust crates/`
  `rg '\.map_err\(' --type rust crates/`

## RSX Class Attributes

- **No string interpolation mixed with static classes** - Detect RSX `class` attributes that mix string interpolation (dynamic values like `"{nav_width}"`) with static Tailwind classes in a single string. Split into multiple `class` attributes: one for all static classes and separate ones for each interpolated value. This improves readability and makes dynamic class application explicit.
  `rg 'class:\s*"[^"]*\{[^}]+\}[^"]*"' --type rust crates/`
  Bad: `class: "{nav_width} bg-card border-r border-border {nav_padding} flex flex-col transition-all duration-200"`
  Good:
  ```
  class: "bg-card border-r border-border flex flex-col transition-all duration-200",
  class: "{nav_width}",
  class: "{nav_padding}",
  ```

## Ecosystem Utilization

- **dioxus-sdk over raw web_sys/gloo** - Detect raw `web_sys` or `gloo` usage for functionality that `dioxus-sdk` already provides. Replace with the corresponding `dioxus-sdk` hook. Read `docs/dioxus-sdk-ref.md` for the full hook catalog.
  `rg 'web_sys|gloo' --type rust crates/`
  `rg 'window\(\)\.inner_width|localStorage|setTimeout|setInterval|navigator\.geolocation' --type rust crates/`
- **dioxus-primitives over hand-rolled components** - Detect hand-rolled UI components that duplicate functionality provided by `dioxus-primitives`. Replace with the corresponding primitive. Read `docs/dioxus-primitives-ref.md` for the full component catalog.
  `rg '#\[component\]' --type rust crates/`
  `rg 'Dialog|Modal|Dropdown|Tooltip|Tab|Accordion|Toast|ContextMenu' --type rust crates/`
  For RSX element usage: `macro_invocation(macro = \`rsx\`, argument = $body) where { $body <: contains \`Dialog\` }` etc. — text search inside macro bodies works.
