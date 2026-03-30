# WU-12: Kanban live issue pulsing

## Dependencies: WU-11 must run first.

WU-12 modifies files produced by the WU-11 split. All line references below are based on the post-WU-11 state documented in WU-11's Post-Execution State section. After WU-11, `KanbanCard` and `render_drag_overlay` live in `kanban_card.rs` (~100 lines), while `KanbanBoardView` and `KanbanColumn` live in `kanban.rs` (~210 lines).

## Fixes
- Fix 1: Add `active_issue_ids: Vec<String>` prop to `KanbanBoardView` to identify issues an agent is currently working on
- Fix 2: Propagate `active_issue_ids` through `KanbanColumn` to `KanbanCard`
- Fix 3: Add `is_active: bool` prop to `KanbanCard`
- Fix 4: Apply `animate-pulse` CSS class to active cards
- Fix 5: Add a subtle glowing border/ring on active cards to distinguish them from idle ones

## Files Modified
- `crates/lx-desktop/src/pages/issues/kanban.rs` (~210 lines post-WU-11)
- `crates/lx-desktop/src/pages/issues/kanban_card.rs` (~100 lines post-WU-11)

## Preconditions
- Post-WU-11 state of `kanban.rs`:
  - `KanbanBoardView` component at line 7 with props: `issues`, `agents`, `on_select`, `on_status_change`, `on_reorder` (optional)
  - `KanbanColumn` component at ~line 96 with props: `status`, `issues`, `agents`, `on_select`, `on_status_change`, `dragging_issue_id`, `drag_over_column`, `drag_active`, `pending_drag_id`, `pointer_start`, `pointer_pos`, `drag_over_index`
  - Card rendering loop in KanbanColumn uses `for (idx, issue) in issues.iter().enumerate()` with `onmouseenter` wrappers and drop indicators
- Post-WU-11 state of `kanban_card.rs`:
  - `KanbanCard` component with props: `issue`, `agents`, `dragging_issue_id`, `drag_active`, `pending_drag_id`, `pointer_start`, `on_click`
  - `card_cls` variable determines card styling based on `is_dragging`
  - `render_drag_overlay` function renders the floating drag ghost
- The `Issue` struct has `status: String` and `assignee_agent_id: Option<String>` fields
- Tailwind `animate-pulse` class is available (standard Tailwind utility)

## Steps

### Step 1: Add active_issue_ids prop to KanbanBoardView
- Open `crates/lx-desktop/src/pages/issues/kanban.rs`
- In the `KanbanBoardView` component props (after `on_reorder`), add:

```rust
  #[props(default)] active_issue_ids: Vec<String>,
```

### Step 2: Pass active_issue_ids to KanbanColumn
- In the KanbanBoardView rsx where `KanbanColumn` is rendered, add `active_issue_ids: active_issue_ids.clone(),` to the props.

### Step 3: Add active_issue_ids prop to KanbanColumn
- In the `KanbanColumn` component signature, add `active_issue_ids: Vec<String>,` after `agents: Vec<AgentRef>,`.

### Step 4: Compute is_active per card and pass to KanbanCard
- In the KanbanColumn card rendering loop (the `for (idx, issue) in issues.iter().enumerate()` block), compute `is_active` before rendering each card and pass it to `KanbanCard`:

Inside the per-card `div` with `onmouseenter`, add `is_active` to the `KanbanCard` props:

```rust
          {
            let is_active = active_issue_ids.contains(&issue.id);
            rsx! {
              KanbanCard {
                issue: issue.clone(),
                agents: agents.clone(),
                dragging_issue_id,
                drag_active,
                pending_drag_id,
                pointer_start,
                is_active,
                on_click: {
                    let id = issue.identifier.clone().unwrap_or_else(|| issue.id.clone());
                    move |_| on_select.call(id.clone())
                },
              }
            }
          }
```

### Step 5: Add is_active prop to KanbanCard
- Open `crates/lx-desktop/src/pages/issues/kanban_card.rs`
- In the `KanbanCard` component props, add `#[props(default)] is_active: bool,` after `pointer_start: Signal<(f64, f64)>,`.

### Step 6: Apply animate-pulse and glow ring to active cards
- In `kanban_card.rs`, replace the `card_cls` logic:

```rust
  let is_dragging = *drag_active.read() && dragging_issue_id.read().as_deref() == Some(issue.id.as_str());
  let card_cls = if is_dragging {
    "rounded-md border border-[var(--outline-variant)]/30 bg-[var(--surface-container)] p-2.5 w-full text-left opacity-30 transition-opacity cursor-grabbing"
  } else {
    "rounded-md border border-[var(--outline-variant)]/30 bg-[var(--surface-container)] p-2.5 w-full text-left hover:shadow-sm transition-shadow cursor-grab"
  };
```

With:

```rust
  let is_dragging = *drag_active.read() && dragging_issue_id.read().as_deref() == Some(issue.id.as_str());
  let card_cls = if is_dragging {
    "rounded-md border border-[var(--outline-variant)]/30 bg-[var(--surface-container)] p-2.5 w-full text-left opacity-30 transition-opacity cursor-grabbing"
  } else if is_active {
    "rounded-md border border-[var(--tertiary)]/40 bg-[var(--surface-container)] p-2.5 w-full text-left hover:shadow-sm transition-shadow cursor-grab ring-1 ring-[var(--tertiary)]/30 animate-pulse"
  } else {
    "rounded-md border border-[var(--outline-variant)]/30 bg-[var(--surface-container)] p-2.5 w-full text-left hover:shadow-sm transition-shadow cursor-grab"
  };
```

### Step 7: Add active indicator icon to active cards
- In `kanban_card.rs`, inside the card header div (`div { class: "flex items-start gap-1.5 mb-1.5",`), after the id_display span, add:

```rust
        if is_active {
          span { class: "material-symbols-outlined text-xs text-[var(--tertiary)] shrink-0", "bolt" }
        }
```

### Step 8: Update render_drag_overlay to not pulse
- The `render_drag_overlay` function in `kanban_card.rs` constructs its own static card and does not use the `is_active` prop. No action needed.

## File Size Check
- `kanban.rs`: was ~210 lines (post-WU-11), now ~215 lines (under 300)
- `kanban_card.rs`: was ~100 lines (post-WU-11), now ~115 lines (under 300)

## Verification
- Run `just diagnose` to confirm no compile errors or warnings
- Pass `active_issue_ids: vec!["some-issue-id".to_string()]` to `KanbanBoardView`
- The card for that issue should have a subtle tertiary-colored ring and pulse animation
- A small bolt icon should appear next to the issue identifier
- Cards not in `active_issue_ids` should render normally without pulse or ring
- Dragging an active card should suppress the pulse (opacity-30 drag state takes priority)
- The drag overlay ghost card should not pulse
- If `active_issue_ids` is empty (default), all cards render normally (no regression)
