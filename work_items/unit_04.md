# Unit 4: Layout Overhaul & Shared Components (Part 1)

## Scope

Rewrite the lx-desktop layout to match the Paperclip three-column structure (company_rail | sidebar | main+breadcrumb+properties_panel), update the shell component, and create shared display components: entity_row, status_icon, status_badge, priority_icon, identity, empty_state, and a status_colors module.

## Preconditions

- **Unit 3 is complete:** Context providers (theme, toast, dialog, panel, sidebar, breadcrumb, company) are wired in Shell. Route enum is finalized in routes.rs. No modifications to routes.rs are needed or allowed.
- **Unit 1 is complete:** `src/components/mod.rs` exists with `pub mod ui;` and the `cn` function in `ui/mod.rs`. `src/lib.rs` already contains `pub mod components;`.
- Dioxus 0.7.3 is the target framework (already in workspace Cargo.toml).
- The following files exist and are functional:
  - `/home/entropybender/repos/lx/crates/lx-desktop/src/layout/mod.rs`
  - `/home/entropybender/repos/lx/crates/lx-desktop/src/layout/shell.rs`
  - `/home/entropybender/repos/lx/crates/lx-desktop/src/layout/sidebar.rs`
  - `/home/entropybender/repos/lx/crates/lx-desktop/src/layout/menu_bar.rs`
  - `/home/entropybender/repos/lx/crates/lx-desktop/src/layout/status_bar.rs`
  - `/home/entropybender/repos/lx/crates/lx-desktop/src/routes.rs`
  - `/home/entropybender/repos/lx/crates/lx-desktop/src/styles.rs`
  - `/home/entropybender/repos/lx/crates/lx-desktop/src/lib.rs`
  - `/home/entropybender/repos/lx/crates/lx-desktop/src/contexts/mod.rs`
- The `tailwind.css` file at `/home/entropybender/repos/lx/crates/lx-desktop/src/tailwind.css` has Tailwind utility classes available.
- Routes enum is at `/home/entropybender/repos/lx/crates/lx-desktop/src/routes.rs`.

## Paperclip Source References

| lx-desktop target file | Paperclip reference file |
|---|---|
| `layout/shell.rs` | `reference/paperclip/ui/src/components/Layout.tsx` |
| `layout/sidebar.rs` | `reference/paperclip/ui/src/components/Sidebar.tsx`, `SidebarSection.tsx`, `SidebarNavItem.tsx` |
| `layout/company_rail.rs` | `reference/paperclip/ui/src/components/CompanyRail.tsx` |
| `layout/breadcrumb_bar.rs` | `reference/paperclip/ui/src/components/BreadcrumbBar.tsx` |
| `layout/properties_panel.rs` | `reference/paperclip/ui/src/components/PropertiesPanel.tsx` |
| `components/status_icon.rs` | `reference/paperclip/ui/src/components/StatusIcon.tsx` |
| `components/status_badge.rs` | `reference/paperclip/ui/src/components/StatusBadge.tsx` |
| `components/priority_icon.rs` | `reference/paperclip/ui/src/components/PriorityIcon.tsx` |
| `components/identity.rs` | `reference/paperclip/ui/src/components/Identity.tsx` |
| `components/empty_state.rs` | `reference/paperclip/ui/src/components/EmptyState.tsx` |
| `components/entity_row.rs` | `reference/paperclip/ui/src/components/EntityRow.tsx` |
| `components/status_colors.rs` | `reference/paperclip/ui/src/lib/status-colors.ts` |

## Steps

### Step 1: Create the status_colors module

**Create file:** `/home/entropybender/repos/lx/crates/lx-desktop/src/components/status_colors.rs`

This is a pure-data module with no Dioxus dependency. Port the color mapping constants from `reference/paperclip/ui/src/lib/status-colors.ts`.

Define the following public functions and constants:

```rust
pub fn issue_status_icon_class(status: &str) -> &'static str
```
Returns a Tailwind class string for the StatusIcon circle. Map these statuses:
- `"backlog"` -> `"text-gray-400 border-gray-400"`
- `"todo"` -> `"text-blue-400 border-blue-400"`
- `"in_progress"` -> `"text-yellow-400 border-yellow-400"`
- `"in_review"` -> `"text-violet-400 border-violet-400"`
- `"done"` -> `"text-green-400 border-green-400"`
- `"cancelled"` -> `"text-neutral-500 border-neutral-500"`
- `"blocked"` -> `"text-red-400 border-red-400"`
- anything else -> `"text-gray-400 border-gray-400"`

```rust
pub fn status_badge_class(status: &str) -> &'static str
```
Returns Tailwind classes for the StatusBadge pill. Map these statuses:
- `"active"` -> `"bg-green-900/50 text-green-300"`
- `"running"` -> `"bg-cyan-900/50 text-cyan-300"`
- `"paused"` -> `"bg-orange-900/50 text-orange-300"`
- `"idle"` -> `"bg-yellow-900/50 text-yellow-300"`
- `"failed"` / `"error"` / `"terminated"` -> `"bg-red-900/50 text-red-300"`
- `"succeeded"` / `"done"` / `"achieved"` / `"completed"` / `"approved"` -> `"bg-green-900/50 text-green-300"`
- `"pending"` / `"pending_approval"` / `"revision_requested"` -> `"bg-amber-900/50 text-amber-300"`
- `"timed_out"` -> `"bg-orange-900/50 text-orange-300"`
- `"todo"` -> `"bg-blue-900/50 text-blue-300"`
- `"in_progress"` -> `"bg-yellow-900/50 text-yellow-300"`
- `"in_review"` -> `"bg-violet-900/50 text-violet-300"`
- `"blocked"` / `"rejected"` -> `"bg-red-900/50 text-red-300"`
- `"backlog"` / `"cancelled"` / `"archived"` / `"planned"` or default -> `"bg-gray-800 text-gray-400"`

```rust
pub fn priority_color_class(priority: &str) -> &'static str
```
Returns text color class:
- `"critical"` -> `"text-red-400"`
- `"high"` -> `"text-orange-400"`
- `"medium"` -> `"text-yellow-400"`
- `"low"` -> `"text-blue-400"`
- default -> `"text-yellow-400"`

```rust
pub fn priority_label(priority: &str) -> &'static str
```
Returns display label: `"Critical"`, `"High"`, `"Medium"`, `"Low"`.

```rust
pub fn status_label(status: &str) -> String
```
Replaces underscores with spaces and title-cases each word: `"in_progress"` -> `"In Progress"`.

### Step 2: Update components/mod.rs (created by Unit 1)

**Edit file:** `/home/entropybender/repos/lx/crates/lx-desktop/src/components/mod.rs`

Add module declarations for the Part 1 shared components. Add these lines:

```rust
pub mod status_colors;
pub mod status_icon;
pub mod status_badge;
pub mod priority_icon;
pub mod identity;
pub mod empty_state;
pub mod entity_row;
```

Note: `src/lib.rs` already contains `pub mod components;` (added by Unit 1). Do NOT re-add it.

### Step 3: Create the StatusIcon component

**Create file:** `/home/entropybender/repos/lx/crates/lx-desktop/src/components/status_icon.rs`

Reference: `reference/paperclip/ui/src/components/StatusIcon.tsx`

Define a `#[component]` function:

```rust
#[component]
pub fn StatusIcon(status: String, #[props(optional)] class: Option<String>) -> Element
```

Behavior:
- Look up `issue_status_icon_class(&status)` from `status_colors`.
- Render a `span` with classes: `"relative inline-flex h-4 w-4 rounded-full border-2 shrink-0"` plus the color class and optional extra class.
- If `status == "done"`, render an inner `span` with class `"absolute inset-0 m-auto h-2 w-2 rounded-full bg-current"`.

No popover/onChange behavior needed (the Dioxus port is read-only display for now).

### Step 4: Create the StatusBadge component

**Create file:** `/home/entropybender/repos/lx/crates/lx-desktop/src/components/status_badge.rs`

Reference: `reference/paperclip/ui/src/components/StatusBadge.tsx`

```rust
#[component]
pub fn StatusBadge(status: String) -> Element
```

Behavior:
- Look up `status_badge_class(&status)` from `status_colors`.
- Call `status_label(&status)` from `status_colors` to get display text.
- Render a `span` with classes: `"inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium whitespace-nowrap shrink-0"` plus the color class.
- Inner text is the status label.

### Step 5: Create the PriorityIcon component

**Create file:** `/home/entropybender/repos/lx/crates/lx-desktop/src/components/priority_icon.rs`

Reference: `reference/paperclip/ui/src/components/PriorityIcon.tsx`

```rust
#[component]
pub fn PriorityIcon(
    priority: String,
    #[props(optional)] class: Option<String>,
    #[props(default = false)] show_label: bool,
) -> Element
```

Behavior:
- Determine the SVG icon character based on priority:
  - `"critical"`: render a triangle-alert unicode char or an SVG path for AlertTriangle (use `"\u{26A0}"` or inline SVG `<path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/>` etc.)
  - `"high"`: up arrow SVG (`M12 19V5`, `M5 12l7-7 7 7`)
  - `"medium"`: minus SVG (`M5 12h14`)
  - `"low"`: down arrow SVG (`M12 5v14`, `M19 12l-7 7-7-7`)
- Look up `priority_color_class(&priority)` from `status_colors`.
- Render a `span` with classes `"inline-flex items-center justify-center shrink-0"` + color class + optional extra class.
- Inside, render an `svg` element (viewBox `"0 0 24 24"`, class `"h-3.5 w-3.5"`, stroke `"currentColor"`, fill `"none"`, stroke-width `"2"`) with the appropriate path(s).
- If `show_label` is true, wrap in an outer span `"inline-flex items-center gap-1.5"` and append a `span` with class `"text-sm"` containing `priority_label(&priority)`.

### Step 6: Create the Identity component

**Create file:** `/home/entropybender/repos/lx/crates/lx-desktop/src/components/identity.rs`

Reference: `reference/paperclip/ui/src/components/Identity.tsx`

```rust
#[derive(Clone, PartialEq, Props)]
pub struct IdentityProps {
    pub name: String,
    #[props(optional)]
    pub avatar_url: Option<String>,
    #[props(optional)]
    pub initials: Option<String>,
    #[props(default = "default".to_string())]
    pub size: String,
    #[props(optional)]
    pub class: Option<String>,
}

#[component]
pub fn Identity(props: IdentityProps) -> Element
```

Behavior:
- Derive initials: if `props.initials` is `Some`, use it. Otherwise take first char of first word + first char of last word of `props.name`, uppercased. If single word, take first 2 chars.
- Determine avatar size class based on `props.size`:
  - `"xs"` -> `"h-4 w-4 text-[8px]"`
  - `"sm"` -> `"h-5 w-5 text-[9px]"`
  - `"default"` -> `"h-6 w-6 text-[10px]"`
  - `"lg"` -> `"h-8 w-8 text-xs"`
- Text size: `"xs"` -> `"text-sm"`, `"sm"` -> `"text-xs"`, `"default"` -> `"text-sm"`, `"lg"` -> `"text-sm"`.
- Render:
  ```
  span (class: "inline-flex items-center gap-1.5" + extra class) {
    span (class: "inline-flex items-center justify-center rounded-full bg-gray-700 text-gray-300 shrink-0 {avatar_size}") {
      if avatar_url is Some: img (src, alt=name, class: "h-full w-full rounded-full object-cover")
      else: "{initials}"
    }
    span (class: "truncate {text_size}") { "{name}" }
  }
  ```

### Step 7: Create the EmptyState component

**Create file:** `/home/entropybender/repos/lx/crates/lx-desktop/src/components/empty_state.rs`

Reference: `reference/paperclip/ui/src/components/EmptyState.tsx`

```rust
#[component]
pub fn EmptyState(
    icon: String,
    message: String,
    #[props(optional)] action: Option<String>,
    #[props(optional)] on_action: Option<EventHandler<()>>,
) -> Element
```

The `icon` is a Material Symbols icon name string (e.g. `"dashboard"`, `"history"`).

Render:
```
div (class: "flex flex-col items-center justify-center py-16 text-center") {
  div (class: "bg-gray-800/50 p-4 mb-4") {
    span (class: "material-symbols-outlined text-4xl text-gray-500") { "{icon}" }
  }
  p (class: "text-sm text-gray-400 mb-4") { "{message}" }
  if action.is_some() && on_action.is_some() {
    button (class: "px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white text-sm rounded transition-colors",
            onclick: move |_| on_action.unwrap().call(())) {
      "{action.unwrap()}"
    }
  }
}
```

### Step 8: Create the EntityRow component

**Create file:** `/home/entropybender/repos/lx/crates/lx-desktop/src/components/entity_row.rs`

Reference: `reference/paperclip/ui/src/components/EntityRow.tsx`

```rust
#[component]
pub fn EntityRow(
    title: String,
    #[props(optional)] leading: Option<Element>,
    #[props(optional)] identifier: Option<String>,
    #[props(optional)] subtitle: Option<String>,
    #[props(optional)] trailing: Option<Element>,
    #[props(default = false)] selected: bool,
    #[props(optional)] to: Option<String>,
    #[props(optional)] onclick: Option<EventHandler<()>>,
    #[props(optional)] class: Option<String>,
) -> Element
```

Behavior:
- Build a class string: `"flex items-center gap-3 px-4 py-2 text-sm border-b border-gray-700/50 last:border-b-0 transition-colors"`.
- If `to.is_some() || onclick.is_some()`: append `" cursor-pointer hover:bg-white/5"`.
- If `selected`: append `" bg-white/[0.03]"`.
- Append any extra `class`.
- Inner content layout:
  ```
  if leading { div (class: "flex items-center gap-2 shrink-0") { {leading} } }
  div (class: "flex-1 min-w-0") {
    div (class: "flex items-center gap-2") {
      if identifier { span (class: "text-xs text-gray-400 font-mono shrink-0") { "{identifier}" } }
      span (class: "truncate") { "{title}" }
    }
    if subtitle { p (class: "text-xs text-gray-400 truncate mt-0.5") { "{subtitle}" } }
  }
  if trailing { div (class: "flex items-center gap-2 shrink-0") { {trailing} } }
  ```
- If `to` is `Some(href)`, wrap content in a `Link { to: "{href}" }`.
- Otherwise wrap in a `div` with `onclick` handler if provided.

### Step 9: Create the CompanyRail component

**Create file:** `/home/entropybender/repos/lx/crates/lx-desktop/src/layout/company_rail.rs`

Reference: `reference/paperclip/ui/src/components/CompanyRail.tsx`

For the lx-desktop port, the company rail is a simplified vertical strip. There is no multi-company support in lx, so this renders a static branding column.

```rust
#[component]
pub fn CompanyRail() -> Element
```

Render:
```
div (class: "flex flex-col items-center w-[72px] shrink-0 h-full bg-[var(--surface-container-lowest)] border-r border-gray-700/50") {
  div (class: "flex items-center justify-center h-12 w-full shrink-0") {
    span (class: "text-xl font-bold text-[var(--primary)]") { "lx" }
  }
  div (class: "flex-1") {}
  div (class: "w-8 h-px bg-gray-700/50 mx-auto shrink-0") {}
  div (class: "flex items-center justify-center py-2 shrink-0") {
    span (class: "text-xs text-gray-500") { "v0.1" }
  }
}
```

### Step 10: Create the BreadcrumbBar component

**Create file:** `/home/entropybender/repos/lx/crates/lx-desktop/src/layout/breadcrumb_bar.rs`

Reference: `reference/paperclip/ui/src/components/BreadcrumbBar.tsx`

Use the canonical `BreadcrumbEntry` type from `crate::contexts::breadcrumb::BreadcrumbEntry` (created by Unit 3). Do NOT define a local `Breadcrumb` struct.

```rust
use crate::contexts::breadcrumb::{BreadcrumbEntry, BreadcrumbState};

#[component]
pub fn BreadcrumbBar() -> Element
```

Use `use_context::<BreadcrumbState>()` to read breadcrumbs from context (provided in Shell by Unit 3).

Behavior:
- If breadcrumbs is empty: render `div (class: "border-b border-gray-700/50 px-6 h-12 shrink-0 flex items-center")` with no content.
- If exactly 1 breadcrumb: render a heading `h1 (class: "text-sm font-semibold uppercase tracking-wider truncate")` with the label.
- If multiple: render a breadcrumb trail with `/` separators. Each non-last crumb with an `href` is a `Link`; the last is plain text.

Container class: `"border-b border-gray-700/50 px-6 h-12 shrink-0 flex items-center"`.

Breadcrumb separator: `span (class: "mx-2 text-gray-500") { "/" }`.
Non-last crumb link: `class: "text-sm text-gray-400 hover:text-white transition-colors"`.
Last crumb: `span (class: "text-sm text-white truncate")`.

### Step 11: Create the PropertiesPanel component

**Create file:** `/home/entropybender/repos/lx/crates/lx-desktop/src/layout/properties_panel.rs`

Reference: `reference/paperclip/ui/src/components/PropertiesPanel.tsx`

Use the canonical `PanelState` type from `crate::contexts::panel::PanelState` (created by Unit 3). Do NOT define a local `PanelState` struct.

```rust
use crate::contexts::panel::PanelState;

#[component]
pub fn PropertiesPanel() -> Element
```

Use `use_context::<PanelState>()`.

Behavior:
- If `content_id` is `None`, return `None` (render nothing).
- Otherwise render:
```
aside (class: "border-l border-gray-700/50 bg-[var(--surface-container)] flex-col shrink-0 overflow-hidden transition-[width,opacity] duration-200 ease-in-out",
       style: width = if visible { "320px" } else { "0px" }, opacity = if visible { "1" } else { "0" }) {
  div (class: "w-80 flex-1 flex flex-col min-w-[320px]") {
    div (class: "flex items-center justify-between px-4 py-2 border-b border-gray-700/50") {
      span (class: "text-sm font-medium") { "Properties" }
      button (class: "p-1 hover:bg-white/10 rounded transition-colors",
              onclick: set visible to false) {
        span (class: "material-symbols-outlined text-sm") { "close" }
      }
    }
    div (class: "flex-1 overflow-y-auto p-4") {
      {content}
    }
  }
}
```

### Step 12: Rewrite the Sidebar component

**Edit file:** `/home/entropybender/repos/lx/crates/lx-desktop/src/layout/sidebar.rs`

Replace the entire contents. Reference: `reference/paperclip/ui/src/components/Sidebar.tsx`, `SidebarSection.tsx`, `SidebarNavItem.tsx`.

The new sidebar must have:
1. A top bar with title "lx" and a search icon button (48px / h-12 height).
2. A scrollable `nav` area with sections using Paperclip's structure.

Define a helper `SidebarSection` component:
```rust
#[component]
fn SidebarSection(label: &'static str, children: Element) -> Element
```
Renders:
```
div {
  div (class: "px-3 py-1.5 text-[10px] font-medium uppercase tracking-widest font-mono text-gray-500") { "{label}" }
  div (class: "flex flex-col gap-0.5 mt-0.5") { {children} }
}
```

Define a helper `SidebarNavItem` component:
```rust
#[component]
fn SidebarNavItem(to: Route, label: &'static str, icon: &'static str) -> Element
```
Renders a `Link` with:
- `active_class: "bg-white/10 text-white"`
- Default class: `"flex items-center gap-2.5 px-3 py-2 text-[13px] font-medium transition-colors text-gray-400 hover:bg-white/5 hover:text-white"`
- Inner: `span (class: "material-symbols-outlined text-base") { icon }` + `span (class: "flex-1 truncate") { label }`.

The main `Sidebar` component renders:
```
aside (class: "w-60 h-full min-h-0 border-r border-gray-700/50 bg-[var(--surface-container-lowest)] flex flex-col") {
  div (class: "flex items-center gap-1 px-3 h-12 shrink-0") {
    span (class: "flex-1 text-sm font-bold text-white truncate pl-1") { "lx workspace" }
  }
  nav (class: "flex-1 min-h-0 overflow-y-auto flex flex-col gap-4 px-3 py-2") {
    div (class: "flex flex-col gap-0.5") {
      SidebarNavItem { to: Route::Agents {}, label: "Agents", icon: "smart_toy" }
      SidebarNavItem { to: Route::Activity {}, label: "Activity", icon: "pulse_alert" }
    }
    SidebarSection { label: "System",
      SidebarNavItem { to: Route::Tools {}, label: "Tools", icon: "build" }
      SidebarNavItem { to: Route::Settings {}, label: "Settings", icon: "settings" }
    }
    SidebarSection { label: "Account",
      SidebarNavItem { to: Route::Accounts {}, label: "Accounts", icon: "account_circle" }
    }
  }
}
```

### Step 13: Rewrite the Shell component

**Edit file:** `/home/entropybender/repos/lx/crates/lx-desktop/src/layout/shell.rs`

Update the Shell to use the new layout structure: CompanyRail | Sidebar | (BreadcrumbBar + main content + PropertiesPanel).

Keep the existing `ResizeHandles`, `TerminalSpawnRequest`, `spawn_terminal_listener` logic.

Note: Breadcrumb and Panel context providers are already wired in Shell by Unit 3 (`BreadcrumbState::provide()` and `PanelState::provide()`). Do NOT add duplicate providers.

New render structure:
```
div (class: "relative h-screen overflow-hidden bg-[var(--surface)] text-[var(--on-surface)] flex flex-col") {
  ResizeHandles {}
  MenuBar {}
  div (class: "flex flex-1 min-h-0") {
    CompanyRail {}
    Sidebar {}
    div (class: "flex min-w-0 flex-col flex-1 h-full") {
      BreadcrumbBar {}
      div (class: "flex flex-1 min-h-0") {
        main (class: "flex-1 flex flex-col p-0 min-h-0") {
          div (class: "flex-1 min-h-0 overflow-auto p-6") {
            ErrorBoundary { ... }
              SuspenseBoundary { ... }
                Outlet::<Route> {}
          }
        }
        PropertiesPanel {}
      }
    }
  }
  StatusBar {}
}
```

### Step 14: Update layout/mod.rs

**Edit file:** `/home/entropybender/repos/lx/crates/lx-desktop/src/layout/mod.rs`

Add the new modules:
```rust
pub mod breadcrumb_bar;
pub mod company_rail;
pub mod menu_bar;
pub mod properties_panel;
pub mod shell;
pub mod sidebar;
pub mod status_bar;
```

### Step 15: Add new imports to shell.rs

Add `use super::company_rail::CompanyRail;`, `use super::breadcrumb_bar::BreadcrumbBar;`, and `use super::properties_panel::PropertiesPanel;` to the top of shell.rs. The `BreadcrumbEntry` and `PanelState` types are imported from `crate::contexts::breadcrumb` and `crate::contexts::panel` respectively (provided by Unit 3).

## Files Created

| File | Lines (approx) |
|---|---|
| `src/components/mod.rs` | ~10 |
| `src/components/status_colors.rs` | ~90 |
| `src/components/status_icon.rs` | ~30 |
| `src/components/status_badge.rs` | ~25 |
| `src/components/priority_icon.rs` | ~80 |
| `src/components/identity.rs` | ~60 |
| `src/components/empty_state.rs` | ~30 |
| `src/components/entity_row.rs` | ~65 |
| `src/layout/company_rail.rs` | ~25 |
| `src/layout/breadcrumb_bar.rs` | ~65 |
| `src/layout/properties_panel.rs` | ~45 |

## Files Modified

| File | Change |
|---|---|
| `src/lib.rs` | No change needed — `pub mod components;` already added by Unit 1 |
| `src/layout/mod.rs` | Add `pub mod breadcrumb_bar;`, `pub mod company_rail;`, `pub mod properties_panel;` |
| `src/layout/shell.rs` | Rewrite render layout, add context providers, add imports |
| `src/layout/sidebar.rs` | Full rewrite to Paperclip-style nav sections |

## Definition of Done

1. `just diagnose` passes with no errors.
2. The application renders with the three-column layout: CompanyRail (72px) | Sidebar (240px) | Main content area.
3. BreadcrumbBar renders at the top of the main content area with a bottom border.
4. PropertiesPanel can be toggled via its `Signal<PanelState>` context.
5. All seven shared components (`StatusIcon`, `StatusBadge`, `PriorityIcon`, `Identity`, `EmptyState`, `EntityRow`, `status_colors`) compile and are importable from `crate::components::*`.
6. The Sidebar shows the correct nav items (Agents, Activity, Tools, Settings, Accounts) grouped into sections.
7. No file exceeds 300 lines.
