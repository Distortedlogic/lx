# Goal

Add `pkg/kit/canvas` — rich visual output via `std/pane`. Agents can render charts, tables, images, diagrams, and interactive widgets by opening a canvas pane and pushing structured widget descriptions. Pure lx, no Rust changes.

**Depends on: PANE_BACKEND work item must be completed first.**

# Why

- Agentic IDEs (Cursor, Windsurf) render rich inline results — charts, formatted tables, image previews. lx agents can only emit text or yield structured data. A canvas abstraction lets agents describe visual output declaratively.
- `std/diag` already generates Mermaid diagrams from ASTs. Canvas generalizes this: any visual widget described as a Record, rendered by the host.
- Built on `std/pane` — the canvas is a pane type. The host decides how to render (HTML, terminal, image).

# What Changes

**New file `pkg/kit/canvas.lx`:** Functions that create, update, and compose canvas pane content.

- `canvas.open title` — open a canvas pane
- `canvas.chart pane spec` — add a chart widget (bar, line, pie)
- `canvas.table pane headers rows` — add a formatted table
- `canvas.image pane src alt` — add an image (base64 or URL)
- `canvas.mermaid pane diagram` — add a Mermaid diagram
- `canvas.markdown pane text` — add rendered markdown
- `canvas.clear pane` — clear all widgets
- `canvas.close pane` — close the canvas pane

# Files Affected

- `pkg/kit/canvas.lx` — New file
- `tests/108_canvas.lx` — New test file

# Task List

### Task 1: Create pkg/kit/canvas.lx

**Subject:** Create canvas.lx with widget rendering via std/pane

**Description:** Create `pkg/kit/canvas.lx`:

```
-- Canvas -- rich visual output via pane system.
-- Agents describe widgets declaratively. Host renders them.
-- Each widget is a Record with a type and content. The pane accumulates widgets.

use std/pane

+open = (title) {
  pane.open "canvas" {title: title  widgets: []} ^
}

+chart = (canvas spec) {
  widget = {
    type: "chart"
    chart_type: spec.type ?? "bar"
    data: spec.data
    labels: spec.labels ?? []
    title: spec.title ?? ""
    x_label: spec.x_label ?? ""
    y_label: spec.y_label ?? ""
  }
  pane.update canvas.__pane_id {action: "add_widget"  widget: widget}
}

+table = (canvas headers rows) {
  widget = {
    type: "table"
    headers: headers
    rows: rows
  }
  pane.update canvas.__pane_id {action: "add_widget"  widget: widget}
}

+image = (canvas src alt) {
  widget = {
    type: "image"
    src: src
    alt: alt ?? ""
  }
  pane.update canvas.__pane_id {action: "add_widget"  widget: widget}
}

+mermaid = (canvas diagram) {
  widget = {
    type: "mermaid"
    source: diagram
  }
  pane.update canvas.__pane_id {action: "add_widget"  widget: widget}
}

+markdown = (canvas text) {
  widget = {
    type: "markdown"
    content: text
  }
  pane.update canvas.__pane_id {action: "add_widget"  widget: widget}
}

+text = (canvas content) {
  widget = {
    type: "text"
    content: content
  }
  pane.update canvas.__pane_id {action: "add_widget"  widget: widget}
}

+clear = (canvas) {
  pane.update canvas.__pane_id {action: "clear_widgets"}
}

+close = (canvas) {
  pane.close canvas.__pane_id
}
```

The widget Records are intentionally simple — the host interprets them. A desktop host might render charts with a JavaScript charting library, tables with HTML, mermaid diagrams with mermaid.js. A CLI host might render tables as ASCII and skip charts. The canvas abstraction doesn't prescribe rendering — it provides a structured vocabulary for visual intent.

**ActiveForm:** Creating canvas.lx with widget rendering

---

### Task 2: Write tests for pkg/kit/canvas

**Subject:** Write tests verifying canvas functions exist and are callable

**Description:** Create `tests/108_canvas.lx`:

```
use pkg/kit/canvas

-- Verify API surface
assert (type_of canvas.open == "Fn") "open exists"
assert (type_of canvas.chart == "Fn") "chart exists"
assert (type_of canvas.table == "Fn") "table exists"
assert (type_of canvas.image == "Fn") "image exists"
assert (type_of canvas.mermaid == "Fn") "mermaid exists"
assert (type_of canvas.markdown == "Fn") "markdown exists"
assert (type_of canvas.text == "Fn") "text exists"
assert (type_of canvas.clear == "Fn") "clear exists"
assert (type_of canvas.close == "Fn") "close exists"

log.info "108_canvas: all passed"
```

Like std/pane tests, actual rendering is integration-tested under an orchestrator. Unit tests verify the module loads and exports the correct API.

Run `just test` to verify.

**ActiveForm:** Writing tests for canvas package

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.

---

## Task Loading Instructions

To execute this work item, load it with the workflow MCP:

```
mcp__workflow__load_work_item({ path: "work_items/PKG_CANVAS.md" })
```

Then call `next_task` to begin.
