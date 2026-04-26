---
name: lx-mermaid
description: Create, edit, and troubleshoot Mermaid `.mmd` diagrams in the `lx` repo using the repo's `mmdc` render workflow, `mermaid.config.json`, and established styling and layout conventions. Use when Codex needs to modify Mermaid source, adjust Mermaid theme/config settings, regenerate diagram PNGs, fix black-background rendering mismatches, diagnose layout or wrapping issues, or explain Mermaid behavior in this repository.
---

# LX Mermaid

## Overview

Use this skill for Mermaid work in the `lx` repo. Preserve the repo's render contract: Mermaid config controls diagram styling and layout, while the `mmdc` CLI command controls the page background used for exported PNGs.

## Workflow

1. Read [references/lx-mermaid-notes.md](references/lx-mermaid-notes.md) before changing layout, wrapping, subgraphs, or styling.
2. Edit the `.mmd` source first. Keep shared visual rules in `mermaid.config.json` instead of repeating inline node styles.
3. Treat `mermaid.config.json` and the render command as a pair.
   Keep `"theme": "base"` when relying on `themeVariables`.
   Keep page background on the CLI with `-b "#000000"`.
   Keep `flowchart.wrappingWidth` large unless explicit auto-wrapping is wanted.
4. Regenerate outputs after every Mermaid edit.
   Use `just diagrams` to render `**/*.mmd` recursively from the repo root.
   Reuse the same `mmdc` flags manually only when you need a one-off render outside the repo recipe.
5. Compare symptoms against the reference notes before adding ad hoc `style` or `classDef` overrides.
   If a `.png` exists beside the edited `.mmd`, stale rendered output is incomplete work.

## Repo Rules

- Prefer `themeCSS` plus `:::classname` for shared stroke-color styling.
- Reuse the config-defined classes: `teal`, `blue`, `purple`, `orange`, `yellow`, `red`, `cyan`, `bluelight`.
- Preserve diagram-local `classDef` blocks when they express one-off semantic categories; only hoist repeated repo-wide styling into config.
- Keep node text left-aligned via config CSS, not per-node markup hacks.
- Use inline `style subgraph_id ...` only when a specific subgraph needs a custom border.
- Preserve explicit numbering conventions already used in repo workflow diagrams.

## Debugging Order

- Check CLI `-b` first when the canvas background is wrong.
- Check `"theme": "base"` when theme variables seem ignored.
- Check `flowchart.wrappingWidth` and explicit line breaks when text wraps unexpectedly.
- Check whether internal subgraph nodes connect to outside nodes when subgraph direction is ignored.
- Check declaration order of edges when left/right branch placement is wrong.
