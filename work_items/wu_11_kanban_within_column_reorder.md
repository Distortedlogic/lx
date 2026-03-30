# WU-11: Kanban within-column reordering

## Fixes
- Fix 1: Add drag_over_index signal to track insertion position within a column
- Fix 2: Track source column and index of the dragged item
- Fix 3: Emit a reorder event with (issue_id, target_status, target_index) when dropping within same column
- Fix 4: Show a visual drop indicator between cards at the target index
- Fix 5: Extend on_status_change to carry optional index or add a separate on_reorder handler
- Fix 6: Split kanban.rs into kanban.rs + kanban_card.rs if file exceeds 300 lines

## Files Modified
- `crates/lx-desktop/src/pages/issues/kanban.rs` (250 lines)

## Preconditions
- `KanbanBoardView` component at line 8 has props: `issues`, `agents`, `on_select`, `on_status_change: EventHandler<(String, String)>`
- `dragging_issue_id: Signal<Option<String>>` at line 14 tracks which issue is being dragged
- `drag_over_column: Signal<Option<String>>` at line 15 tracks which column the cursor is over
- `KanbanColumn` component at line 96 receives the full signal set and renders cards
- `KanbanCard` component at line 166 handles `onmousedown` to initiate drag
- `onmouseup` handler at line 47 calls `on_status_change.call((issue_id, target_status))` on drop
- The `Issue` struct (in `types.rs` line 4) has no `sort_order` field — reordering is index-based
- File is currently 250 lines

## Steps

### Step 1: Add drag_over_index signal to KanbanBoardView
- Open `crates/lx-desktop/src/pages/issues/kanban.rs`
- At line 19, after `let mut pending_drag_id = use_signal(|| Option::<String>::None);`, add:

```rust
  let mut drag_over_index = use_signal(|| Option::<usize>::None);
```

- Why: Need to track which position within a column the item will be inserted at

### Step 2: Add on_reorder prop to KanbanBoardView
- At line 13, after `on_status_change: EventHandler<(String, String)>,`, add:

```rust
  #[props(optional)] on_reorder: Option<EventHandler<(String, String, usize)>>,
```

- Why: A separate handler for within-column reorder events carrying (issue_id, status, target_index)

### Step 3: Update onmouseup to handle within-column reorder
- Find the `onmouseup` handler at lines 47-59. Replace the body with logic that checks if the drop target column is the same as the source column. If same column and drag_over_index is set, call `on_reorder`; otherwise call `on_status_change` as before:

```rust
      onmouseup: {
          let on_status_change = on_status_change;
          let on_reorder = on_reorder;
          move |_| {
              if *drag_active.read()
                  && let Some(issue_id) = dragging_issue_id.read().clone()
              {
                  if let Some(target_status) = drag_over_column.read().clone() {
                      let source_status = issues.iter()
                          .find(|i| i.id == issue_id)
                          .map(|i| i.status.clone());
                      if source_status.as_deref() == Some(target_status.as_str()) {
                          if let Some(idx) = *drag_over_index.read() {
                              if let Some(ref handler) = on_reorder {
                                  handler.call((issue_id, target_status, idx));
                              }
                          }
                      } else {
                          on_status_change.call((issue_id, target_status));
                      }
                  }
              }
              drag_active.set(false);
              dragging_issue_id.set(None);
              pending_drag_id.set(None);
              drag_over_column.set(None);
              drag_over_index.set(None);
          }
      },
```

- Why: Differentiates between cross-column status change and within-column reorder

### Step 4: Reset drag_over_index in onmouseleave
- At line 65, in the `onmouseleave` handler, add `drag_over_index.set(None);` after `drag_over_column.set(None);`

### Step 5: Pass drag_over_index to KanbanColumn
- At lines 69-82, add `drag_over_index,` to the `KanbanColumn` props.

### Step 6: Update KanbanColumn to accept and use drag_over_index
- At line 96, add `drag_over_index: Signal<Option<usize>>,` to the `KanbanColumn` component props.
- In the card rendering loop (lines 146-159), replace the simple `for issue in issues.iter()` with an enumerated loop that adds drop indicator divs:

```rust
        for (idx, issue) in issues.iter().enumerate() {
          if *drag_active.read() && is_drag_over && drag_over_index.read().as_ref() == Some(&idx) {
            div { class: "h-0.5 rounded bg-[var(--primary)] mx-1 my-0.5" }
          }
          div {
            onmouseenter: {
              let drag_active = drag_active;
              let is_this_column_over = status.clone();
              move |_| {
                if *drag_active.read() && drag_over_column.read().as_deref() == Some(is_this_column_over.as_str()) {
                  drag_over_index.set(Some(idx));
                }
              }
            },
            KanbanCard {
              issue: issue.clone(),
              agents: agents.clone(),
              dragging_issue_id,
              drag_active,
              pending_drag_id,
              pointer_start,
              on_click: {
                  let id = issue.identifier.clone().unwrap_or_else(|| issue.id.clone());
                  move |_| on_select.call(id.clone())
              },
            }
          }
        }
        if *drag_active.read() && is_drag_over && drag_over_index.read().as_ref() == Some(&issues.len()) {
          div { class: "h-0.5 rounded bg-[var(--primary)] mx-1 my-0.5" }
        }
```

- Why: Visual drop indicators show the user where the card will land; per-card mouse enter updates the index naturally

### Step 7: Update the column's onmouseleave to reset drag_over_index
- In the `onmouseleave` handler at lines 141-144, add `drag_over_index.set(None);` inside the if block.

## File Size Check
- `kanban.rs`: was 250 lines, now ~310 lines (over 300)
- Split required: Extract `KanbanCard` (lines 165-215) and `render_drag_overlay` (lines 217-250) into a new file `kanban_card.rs`

### Split Plan
1. Create `crates/lx-desktop/src/pages/issues/kanban_card.rs`
2. Move `KanbanCard` component and `render_drag_overlay` function into it
3. Add necessary imports to kanban_card.rs: `use dioxus::prelude::*;`, `use super::types::{AgentRef, Issue, priority_icon_class};`
4. In kanban.rs, add `mod kanban_card;` and `use kanban_card::{KanbanCard, render_drag_overlay};`
5. No change needed in `crates/lx-desktop/src/pages/issues/mod.rs` — it already has `mod kanban;` at line 4, and kanban_card.rs is a submodule of kanban (declared via `mod kanban_card;` inside kanban.rs), not a sibling module

After split:
- `kanban.rs`: ~210 lines (under 300)
- `kanban_card.rs`: ~100 lines (under 300)

## Post-Execution State

After WU-11 completes, the kanban code is split across two files. WU-12 depends on this state.

- `kanban.rs`: ~210 lines
  - `BOARD_STATUSES` const at line 5
  - `KanbanBoardView` component at line 7 with props: `issues`, `agents`, `on_select`, `on_status_change`, `on_reorder` (optional)
  - Signals: `dragging_issue_id`, `drag_over_column`, `drag_active`, `pointer_start`, `pointer_pos`, `pending_drag_id`, `drag_over_index`
  - `KanbanColumn` component at ~line 96 with props: `status`, `issues`, `agents`, `on_select`, `on_status_change`, `dragging_issue_id`, `drag_over_column`, `drag_active`, `pending_drag_id`, `pointer_start`, `pointer_pos`, `drag_over_index`
  - `mod kanban_card;` and `use kanban_card::{KanbanCard, render_drag_overlay};` at top

- `kanban_card.rs`: ~100 lines
  - `KanbanCard` component with props: `issue`, `agents`, `dragging_issue_id`, `drag_active`, `pending_drag_id`, `pointer_start`, `on_click`
  - `render_drag_overlay` function

## Verification
- Run `just diagnose` to confirm no compile errors or warnings
- Dragging a card within the same column should show a blue drop indicator line between cards
- Dropping within the same column should fire `on_reorder` with the target index (if handler provided)
- Dragging a card to a different column should still fire `on_status_change` as before (no regression)
- If `on_reorder` is not provided (None), within-column drops should be silently ignored
