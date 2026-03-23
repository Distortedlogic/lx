# Goal

Enhance the JSON viewer widget with search/filter, path copying, and expand/collapse all controls. The current widget renders a collapsible tree which is functional but lacks the interactive features needed to navigate large JSON documents.

# Why

- Large JSON payloads (API responses, config files) are hard to navigate with only manual expand/collapse on each node
- No way to search for a key or value within the tree
- No way to copy a JSON path (e.g., `data.users[0].name`) for use in code
- Expand all / collapse all is essential for exploring unknown JSON structure

# What changes

**Add toolbar to `ts/widget-bridge/widgets/json-viewer.ts`:** A compact toolbar above the tree container with: (1) search input that highlights matching keys and values, (2) expand all / collapse all buttons, (3) click-to-copy path on any key (copies dotted path to clipboard).

**Add path tracking to `renderNode`:** Each node tracks its JSON path (e.g., `root.users[0].name`). Clicking a key copies the path to clipboard and briefly highlights it.

# Files affected

- EDIT: `ts/widget-bridge/widgets/json-viewer.ts` — add toolbar, search, expand/collapse all, path copying

# Task List

### Task 1: Add toolbar with expand/collapse all and search

**Subject:** Add interactive toolbar to JSON viewer widget

**Description:** In `ts/widget-bridge/widgets/json-viewer.ts`, in the `mount` function, before appending the `.json-viewer-container` div to the element, create a toolbar div with `display: flex; alignItems: center; gap: 4px; padding: 4px 8px; background: #131313; borderBottom: 1px solid #484848; fontSize: 11px;`. Add a search input (styled like the browser address bar: dark background, light text, 13px, flex:1) that on input filters the tree by highlighting nodes whose key or value contains the search text — set `style.display = "none"` on non-matching leaf nodes, keep parent nodes visible if any child matches. Add "Expand All" and "Collapse All" buttons that find all `.children` divs in the container and set their `display` to `"block"` or `"none"` respectively, updating toggle arrows. Append toolbar then container to the element.

**ActiveForm:** Adding interactive toolbar to JSON viewer

### Task 2: Add path copying on key click

**Subject:** Copy JSON path to clipboard on key click

**Description:** In `ts/widget-bridge/widgets/json-viewer.ts`, modify the `renderNode` function to accept a `path: string` parameter (initially `""` for root). For object entries, the path becomes `${parentPath}.${key}`. For array entries, the path becomes `${parentPath}[${index}]`. On the `keySpan` element, add a `click` event listener that calls `navigator.clipboard.writeText(path)` and briefly sets `keySpan.style.background = "#484848"` for 300ms as visual feedback, then clears it. Update all `renderNode` call sites to pass the path. The root call in `update` passes `""` as the initial path.

**ActiveForm:** Adding JSON path copy on key click

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.
