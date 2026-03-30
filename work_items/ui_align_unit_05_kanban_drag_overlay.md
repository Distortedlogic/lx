# Unit 05: KanbanBoard drag overlay + activation threshold

## Goal
Add a floating card clone that follows the pointer during drag and a 5px activation threshold before engaging drag mode, replacing the current HTML5 drag-and-drop opacity-only feedback.

## Preconditions
- No other units required first
- `crates/lx-desktop/src/pages/issues/kanban.rs` exists at 157 lines with HTML5 drag-and-drop

## Files to Modify
- `crates/lx-desktop/src/pages/issues/kanban.rs`

## Current State

The file uses HTML5 native drag events (`ondragstart`, `ondragover`, `ondragleave`, `ondrop`, `ondragend`) with `draggable: "true"` on cards. During drag, the source card gets `opacity-30`. Columns get a ring highlight when hovered. There is no floating overlay card.

Paperclip uses `@dnd-kit` with `PointerSensor` (5px activation constraint), `SortableContext`, and a `DragOverlay` that renders a clone of the active card with `shadow-lg ring-1 ring-primary/20`.

## Steps

### Step 1: Add new signals to `KanbanBoardView`

In `KanbanBoardView`, add these signals alongside the existing `dragging_issue_id` and `drag_over_column`:

```rust
let drag_active = use_signal(|| false);
let pointer_start = use_signal(|| (0.0f64, 0.0f64));
let pointer_pos = use_signal(|| (0.0f64, 0.0f64));
let pending_drag_id = use_signal(|| Option::<String>::None);
```

- `drag_active`: becomes `true` only after the pointer moves 5px from the start point
- `pointer_start`: records the `(client_x, client_y)` where the pointer was pressed
- `pointer_pos`: tracks the current pointer position for overlay positioning
- `pending_drag_id`: holds the issue ID from pointerdown until the threshold is met or cancelled

### Step 2: Remove HTML5 drag attributes from KanbanCard

In the `KanbanCard` component:
- Remove `draggable: "true"` from the card div
- Remove `ondragstart` and `ondragend` handlers
- Replace them with `onmousedown` that sets `pending_drag_id` and `pointer_start`:

```rust
onmousedown: move |evt| {
    let coords = evt.client_coordinates();
    pointer_start.set((coords.x, coords.y));
    pending_drag_id.set(Some(drag_id.clone()));
},
```

### Step 3: Replace HTML5 drag events on KanbanColumn

In the `KanbanColumn` component:
- Remove `ondragover`, `ondragleave`, `ondrop` handlers from the drop zone div

### Step 4: Add pointer tracking to the board container

In `KanbanBoardView`, wrap the existing `div { class: "flex gap-3 overflow-x-auto pb-4 -mx-2 px-2" }` in an outer div that handles all pointer events at the board level:

```rust
div {
    class: "relative",
    onmousemove: move |evt| {
        let coords = evt.client_coordinates();
        pointer_pos.set((coords.x, coords.y));

        if pending_drag_id.read().is_some() && !*drag_active.read() {
            let (sx, sy) = *pointer_start.read();
            let dx = coords.x - sx;
            let dy = coords.y - sy;
            if (dx * dx + dy * dy).sqrt() >= 5.0 {
                drag_active.set(true);
                dragging_issue_id.set(pending_drag_id.read().clone());
            }
        }
    },
    onmouseup: {
        let on_status_change = on_status_change.clone();
        move |_| {
            if *drag_active.read() {
                if let Some(issue_id) = dragging_issue_id.read().clone() {
                    if let Some(target_status) = drag_over_column.read().clone() {
                        on_status_change.call((issue_id, target_status));
                    }
                }
            }
            drag_active.set(false);
            dragging_issue_id.set(None);
            pending_drag_id.set(None);
            drag_over_column.set(None);
        }
    },
    onmouseleave: move |_| {
        drag_active.set(false);
        dragging_issue_id.set(None);
        pending_drag_id.set(None);
        drag_over_column.set(None);
    },
    // ... existing flex div with columns goes here ...
}
```

### Step 5: Add drag-over detection to columns via onmousemove

Since we removed HTML5 drag events, columns need pointer-based hit detection. Add `onmouseenter` to the drop zone div in `KanbanColumn`:

```rust
onmouseenter: move |_| {
    if *drag_active.read() {
        drag_over_column.set(Some(status_over.clone()));
    }
},
onmouseleave: move |_| {
    if drag_over_column.read().as_deref() == Some(status_leave.as_str()) {
        drag_over_column.set(None);
    }
},
```

### Step 6: Pass new signals to child components

Update `KanbanColumn` signature from:
```rust
fn KanbanColumn(
  status: String, issues: Vec<Issue>, agents: Vec<AgentRef>,
  on_select: EventHandler<String>, on_status_change: EventHandler<(String, String)>,
  dragging_issue_id: Signal<Option<String>>, drag_over_column: Signal<Option<String>>,
) -> Element
```
to:
```rust
fn KanbanColumn(
  status: String, issues: Vec<Issue>, agents: Vec<AgentRef>,
  on_select: EventHandler<String>, on_status_change: EventHandler<(String, String)>,
  dragging_issue_id: Signal<Option<String>>, drag_over_column: Signal<Option<String>>,
  drag_active: Signal<bool>, pending_drag_id: Signal<Option<String>>,
  pointer_start: Signal<(f64, f64)>, pointer_pos: Signal<(f64, f64)>,
) -> Element
```

Update `KanbanCard` signature from:
```rust
fn KanbanCard(issue: Issue, agents: Vec<AgentRef>, dragging_issue_id: Signal<Option<String>>, on_click: EventHandler<()>) -> Element
```
to:
```rust
fn KanbanCard(
  issue: Issue, agents: Vec<AgentRef>,
  dragging_issue_id: Signal<Option<String>>, drag_active: Signal<bool>,
  pending_drag_id: Signal<Option<String>>, pointer_start: Signal<(f64, f64)>,
  on_click: EventHandler<()>,
) -> Element
```

Pass the new signals at the call sites in `KanbanBoardView` and `KanbanColumn`.

### Step 7: Update KanbanCard opacity logic

Change the `is_dragging` check in `KanbanCard` to also require `drag_active`:

```rust
let is_dragging = *drag_active.read() && dragging_issue_id.read().as_deref() == Some(issue.id.as_str());
```

The card classes remain the same (opacity-30 when dragging, cursor-grab otherwise).

### Step 8: Render the drag overlay

At the bottom of the outer `div` in `KanbanBoardView` (after the columns flex container), conditionally render the overlay:

```rust
if *drag_active.read() {
    if let Some(ref active_id) = *dragging_issue_id.read() {
        if let Some(issue) = issues.iter().find(|i| &i.id == active_id) {
            {render_drag_overlay(issue, &agents, *pointer_pos.read())}
        }
    }
}
```

### Step 9: Create `render_drag_overlay` function

Add a standalone function (not a component, since it needs direct props):

```rust
fn render_drag_overlay(issue: &Issue, agents: &[AgentRef], pos: (f64, f64)) -> Element {
    let id_display = issue.identifier.as_deref().unwrap_or(&issue.id[..8.min(issue.id.len())]);
    let assignee_name = issue.assignee_agent_id.as_ref().and_then(|aid| agents.iter().find(|a| &a.id == aid).map(|a| a.name.clone()));
    let style = format!(
        "position: fixed; left: {}px; top: {}px; width: 240px; pointer-events: none; z-index: 50; transform: translate(-50%, -50%);",
        pos.0, pos.1
    );

    rsx! {
        div {
            style: "{style}",
            div {
                class: "rounded-md border border-[var(--outline-variant)]/30 bg-[var(--surface-container)] p-2.5 shadow-lg ring-1 ring-[var(--primary)]/20",
                div { class: "flex items-start gap-1.5 mb-1.5",
                    span { class: "text-xs text-[var(--outline)] font-mono shrink-0", "{id_display}" }
                }
                p { class: "text-sm leading-snug text-[var(--on-surface)] line-clamp-2 mb-2",
                    "{issue.title}"
                }
                div { class: "flex items-center gap-2",
                    span { class: "material-symbols-outlined text-xs {priority_icon_class(&issue.priority)}",
                        match issue.priority.as_str() {
                            "critical" => "priority_high",
                            "high" => "arrow_upward",
                            "low" => "arrow_downward",
                            _ => "remove",
                        }
                    }
                    if let Some(name) = assignee_name {
                        span { class: "text-xs text-[var(--outline)]", "{name}" }
                    }
                }
            }
        }
    }
}
```

Key CSS on the overlay wrapper:
- `position: fixed` so it floats above everything
- `pointer-events: none` so it does not intercept mouse events
- `z-index: 50`
- `transform: translate(-50%, -50%)` to center on pointer
- `width: 240px` to match column card width (column is 260px with 1px padding)

Key CSS on the overlay card:
- Same base card classes as `KanbanCard` (`rounded-md border border-[var(--outline-variant)]/30 bg-[var(--surface-container)] p-2.5`)
- Added: `shadow-lg ring-1 ring-[var(--primary)]/20` (matches Paperclip's overlay styling)

### Step 10: Prevent text selection during drag

Add `user-select: none` to the board container div when drag is active:

```rust
div {
    class: "relative",
    style: if *drag_active.read() { "user-select: none;" } else { "" },
    // ... rest of board ...
}
```

## Verification
1. Run `just diagnose` -- must compile with no errors or warnings
2. Launch the app, navigate to an issue board with multiple columns and cards
3. Click and release a card without moving -- should fire `on_click` (normal selection), no drag behavior
4. Click a card and move less than 5px -- should not start drag, no overlay appears
5. Click a card and move more than 5px -- overlay card appears following the pointer, source card shows `opacity-30`
6. While dragging, hover over a different column -- column shows the ring highlight (`ring-1 ring-[var(--primary)]/40`)
7. Release over a different column -- card moves to that column (status change fires)
8. Release outside any column -- drag cancels, card returns to original position
9. Move pointer out of the board area -- drag cancels cleanly
10. Overlay card has `shadow-lg ring-1 ring-[var(--primary)]/20` visible around it
11. File stays under 300 lines
