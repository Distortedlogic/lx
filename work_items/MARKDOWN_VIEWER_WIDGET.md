# Goal

Upgrade the markdown viewer widget from a raw innerHTML setter to a proper markdown renderer that parses markdown source text into styled HTML. Currently the widget expects pre-processed HTML from the Rust side — it should accept raw markdown and render it client-side using a lightweight markdown parser.

# Why

- The current widget just does `container.innerHTML = data as string` — it's an HTML injector, not a markdown viewer
- Any consumer sending raw markdown (which is the natural API) gets unrendered text
- Adding a client-side markdown parser (marked.js, ~30KB) lets the widget accept raw markdown strings and render them properly with headings, code blocks, lists, links, etc.
- The widget already has CSS rules for h1-h6, code, pre, blockquote, links — these will work once the markdown is actually parsed into HTML

Note on XSS: The markdown widget renders HTML from parsed markdown. In the desktop webview context this is acceptable because all content comes from the application (not user-uploaded). If this widget is ever used with untrusted input, add DOMPurify sanitization.

# What changes

**Add marked.js dependency:** Install `marked` in the widget-bridge package. It's a lightweight, widely-used markdown parser with no dependencies.

**Update markdown widget to parse markdown:** In the `update` method, call `marked.parse(data)` to convert markdown source to HTML before setting innerHTML. The `mount` function stays the same. The existing CSS styles for headings, code blocks, etc. already handle the rendered output.

# Files affected

- EDIT: `ts/widget-bridge/package.json` — add `marked` dependency
- EDIT: `ts/widget-bridge/widgets/markdown.ts` — import marked, use in update method

# Task List

### Task 1: Install marked.js

**Subject:** Add marked markdown parser to widget-bridge

**Description:** Run `pnpm add marked` in the `ts/widget-bridge/` directory. `@types/marked` is NOT needed — marked v4.0.10+ includes built-in TypeScript types.

**ActiveForm:** Installing marked.js markdown parser

### Task 2: Update markdown widget to parse markdown

**Subject:** Use marked.parse in markdown widget update method

**Description:** In `ts/widget-bridge/widgets/markdown.ts`, add `import { marked } from "marked";` at the top. In the `update` method (currently `container.innerHTML = data as string;`), change to `container.innerHTML = marked.parse(data as string) as string;`. The `marked.parse` function is synchronous by default when no async extensions are loaded, so the `as string` cast is safe. The existing CSS styles in the mount function (h1-h6, code, pre, blockquote, link styling) already handle the rendered output. Also update the placeholder in mount from `'<p style="color: #757575;">Markdown viewer — no content loaded</p>'` to use the same pattern — this is already HTML so it stays as-is.

**ActiveForm:** Integrating marked.js parser into markdown widget

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.
