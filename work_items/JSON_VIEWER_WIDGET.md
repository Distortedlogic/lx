# Goal

Enhance the JSON viewer widget with search/filter, path copying, and expand/collapse all controls. The current widget renders a collapsible tree which is functional but lacks the interactive features needed to navigate large JSON documents.

# Why

- Large JSON payloads (API responses, config files) are hard to navigate with only manual expand/collapse on each node
- No way to search for a key or value within the tree
- No way to copy a JSON path (e.g., `data.users[0].name`) for use in code
- Expand all / collapse all is essential for exploring unknown JSON structure

# What changes

**Add toolbar to `ts/widget-bridge/widgets/json-viewer.ts`:** A compact toolbar above the tree container with: (1) search input that highlights matching keys and values, (2) expand all / collapse all buttons, (3) click-to-copy path on any key (copies dotted path to clipboard).

**Add path tracking and data attributes to `renderNode`:** Each node tracks its JSON path (e.g., `users[0].name`). Children divs get `data-role="children"` for expand/collapse targeting. Toggle spans get `data-role="toggle"` so expand/collapse all can find and update them. Clicking a key copies the path to clipboard.

**Add instance tracking:** A `Map<string, { container: HTMLElement; toolbar: HTMLElement }>` tracks mounted instances for cleanup and toolbar access.

# Files affected

- EDIT: `ts/widget-bridge/widgets/json-viewer.ts` — add path param and data attributes to renderNode, add toolbar with search/expand/collapse/path copying

# Task List

### Task 1: Add path parameter and data attributes to renderNode

**Subject:** Extend renderNode with path tracking and DOM attributes

**Description:** In `ts/widget-bridge/widgets/json-viewer.ts`, modify the `renderNode` function to accept a `path: string` parameter. Construct child paths avoiding leading dots: for object entries use `parentPath ? \`${parentPath}.${key}\` : key`, for array entries use `parentPath ? \`${parentPath}[${index}]\` : \`[${index}]\``. Add `children.dataset.role = "children"` to children container divs. Add `toggle.dataset.role = "toggle"` to the toggle span (the `▼`/`▶` element) so expand/collapse all can find and update toggle arrows. On the `keySpan` element, add a `click` event listener that calls `navigator.clipboard.writeText(path).catch(() => {})` and briefly sets `keySpan.style.background = "#484848"` for 300ms via `setTimeout`, then clears it. Update all `renderNode` call sites to pass the path — the root call in `update` passes `""` as the initial path. Add an instance map `Map<string, { container: HTMLElement; toolbar: HTMLElement }>` at module level for tracking mounted instances.

**ActiveForm:** Adding path tracking and data attributes to renderNode

### Task 2: Add toolbar with search, expand/collapse all, and path display

**Subject:** Add interactive toolbar to JSON viewer widget

**Description:** In `ts/widget-bridge/widgets/json-viewer.ts`, in the `mount` function, use flexbox on the element: set `el.style.display = "flex"; el.style.flexDirection = "column"; el.style.height = "100%"`. Create a toolbar div with `display: flex; alignItems: center; gap: 4px; padding: 4px 8px; background: #131313; borderBottom: 1px solid #484848; fontSize: 11px;`. Set the container to `flex: 1; overflow-y: auto`.

Add a search input (dark background `#0a0a0a`, border `1px solid #484848`, light text `#e0e0e0`, 13px, flex:1). On input, run the search algorithm: (1) get all row divs in the container, (2) hide all rows with `style.display = "none"`, (3) for each row, check if any text content (key or value spans) contains the search string (case-insensitive), (4) for each matching row, show it (`style.display = ""`) and walk up through `parentElement` showing all ancestor rows until reaching the container, (5) if search is empty, show all rows.

Add "Expand All" and "Collapse All" buttons. Expand All: find all `[data-role="children"]` divs and set `display = "block"`, find all `[data-role="toggle"]` spans and set `textContent = "▼ "`. Collapse All: set children `display = "none"`, toggles `textContent = "▶ "`.

Append toolbar then container to the element. Store both in the instance map.

**ActiveForm:** Adding interactive toolbar to JSON viewer

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.
