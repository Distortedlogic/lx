---
name: mermaid-chart
description: Create or revise Mermaid charts for this lx repository using the repo's Mermaid knowledge file, shared config, justfile rendering recipe, and existing .mmd diagrams as style references. Use when the user asks for a Mermaid diagram, wants an existing .mmd updated, or wants chart structure/styling that matches this repo.
---

# Create Mermaid Chart

Use this skill only for Mermaid chart work in this repository.

## Required context load

Before drafting or editing any chart, load the local Mermaid context:

1. Read `MERMAID_KNOWLEDGE.md`
2. Read `mermaid.config.json`
3. List existing Mermaid files with Glob: `**/*.mmd`
4. Read the most relevant existing `.mmd` files for the requested chart, and read all of them if the request is about repo-wide chart style or conventions
5. Read the `diagrams:` recipe in `justfile` if you need to confirm rendering behavior

Do not skip the context load. The repo-specific Mermaid rules are not standard Mermaid defaults.

## House rules

Apply the repo conventions from `MERMAID_KNOWLEDGE.md` and the existing diagrams:

- Prefer the same overall visual language already used in repo diagrams
- Reuse the shared theme and node classes from `mermaid.config.json` instead of restating duplicate styling in every `.mmd`
- Rendering uses `mmdc -c mermaid.config.json -b "#000000" -s 8` — keep the dark-background assumption
- Treat `theme: base` as authoritative for custom theme variables
- Use explicit line breaks where needed; the config disables auto-wrap via a very large `wrappingWidth`
- Favor concise labels; decision diamonds get cramped quickly
- When left/right branch placement matters, order outgoing edge declarations deliberately
- Use subgraphs only when they help and the layout stays stable; if subgraph direction would be broken by external edges, prefer a flatter layout
- Use invisible links only when needed for positioning
- Follow the repo's node numbering convention for process-oriented charts: `x.y.z` where major numbers group agent lanes, minor numbers mark agent-related decisions, and patch numbers cover utility/start/end steps
- If a class or stroke color already exists in config CSS, use `:::classname` instead of repeating inline styles

## Output rules

When the user wants a file edited or created:

1. Create or update the `.mmd`
2. Keep the file clean and minimal
3. The PostToolUse hook auto-renders the PNG next to the `.mmd` after every Edit/Write using the same flags as `just diagrams`. If the hook reports a render failure, fix the Mermaid source and save again.
4. For bulk changes (renames, deletes, many files at once), run `just diagrams` to re-render everything

When the user wants a chart in chat instead of a file:

1. Respond with a fenced `mermaid` block
2. Keep it valid and stylistically consistent with this repo
3. Mention any repo-specific styling or layout choices briefly after the block

## Authoring approach

For each request:

1. Infer the chart type from the goal, but default to `flowchart TD` unless the request clearly needs something else
2. Mirror the repo's label density and formatting style from nearby `.mmd` examples
3. Prefer structural clarity over clever Mermaid tricks
4. If Mermaid layout constraints prevent the requested structure, explain the constraint and choose the cleanest repo-consistent compromise
5. If the request conflicts with the repo's Mermaid knowledge, follow the knowledge doc and say what constraint forced the adjustment

## Final response

Always report:

- which Mermaid context files you used
- which `.mmd` files you created or changed
- whether the auto-render hook succeeded (or whether you ran `just diagrams` for a bulk change)
- any notable layout constraint or styling decision
