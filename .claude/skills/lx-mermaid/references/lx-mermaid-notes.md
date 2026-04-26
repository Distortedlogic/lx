# LX Mermaid Notes

## Render Contract

- The repo Just recipe runs:
  `mmdc -i "$f" -o "${f%.mmd}.png" -c mermaid.config.json -b "#000000" -s 8`
- `just diagrams` uses Bash `globstar` and loops over `**/*.mmd` recursively from the repo root.
- `-b` controls the exported page or canvas background.
- `themeVariables.background` does not set the exported page background. It only affects Mermaid's internal theme calculations.
- Keep background control in the render command, not in the config file.

## Config Contract

- Keep `"theme": "base"` when depending on `themeVariables`. Other built-in themes, especially `"dark"`, ignore many custom theme variable values.
- Keep `flowchart.wrappingWidth` at a very large value when you want manual line breaks to dominate. The repo uses `99999` to effectively disable auto-wrap.
- Use `themeCSS` for two repo-wide concerns:
  - force left-aligned node text with `.label foreignObject div { text-align: left !important; }`
  - define reusable stroke-color classes for `.node.<class> > rect` and `.node.<class> > polygon`
- The current config defines these shared color classes:
  - `teal`: `#4ec9b0`
  - `blue`: `#569cd6`
  - `purple`: `#c586c0`
  - `orange`: `#ff6600`
  - `yellow`: `#dcdcaa`
  - `red`: `#d16969`
  - `cyan`: `#4fc1ff`
  - `bluelight`: rectangle stroke `#569cd6` and label text `#9cdcfe`
- The current `themeVariables` drive the repo's black-background look:
  - `primaryColor`: `#000000`
  - `primaryTextColor`: `#ffffff`
  - `primaryBorderColor`: `#6fc3df`
  - `lineColor`: `#6fc3df`
  - `clusterBkg`: `#0a0a0a`
  - `clusterBorder`: `#6fc3df`
  - `fontSize`: `14px`
  - `fontFamily`: `monospace`

## Styling Rules

- Prefer config CSS for shared node classes instead of repeating the same `classDef` or `style` directives in each diagram.
- Preserve local `classDef` blocks when a diagram uses semantic categories that are specific to that file. `programs/workgen/main.mmd` is an example of diagram-local class usage.
- Use `!important` when overriding Mermaid's inline SVG styles from `themeCSS`.
- Keep targeting both `rect` and `polygon` when a class needs to style regular nodes and decision nodes.
- `:::classname` still works when the class behavior is defined in `themeCSS`.
- `clusterBkg` and `clusterBorder` apply uniformly to all subgraphs. For a one-off subgraph border, use `style subgraph_id stroke:#hex,stroke-width:2px`.

## Layout and Text

- Use backtick labels when you need markdown emphasis inside nodes.
- Use `\n` in regular labels for explicit line breaks.
- Remember that indentation inside backtick labels collapses. Use visible prefixes such as `->` or `..` when you need to fake indentation with ASCII.
- Treat 14px monospace text as roughly 8.4px per character when estimating manual wrap points.
- Reorder outgoing edges from a node to control which child lands on the left or right.
- Use `~~~` invisible links to force side-by-side columns without visible edges.
- Expect Dagre to struggle with back-edges and loops. Test ELK when layout is poor, but do not assume it is always better.

## Subgraphs

- Use `direction TB` inside a subgraph when you need a local top-to-bottom layout.
- Expect subgraph direction to be ignored when internal nodes connect directly to nodes outside the subgraph.
- Prefer edges to the subgraph ID, not internal node IDs, when trying to preserve internal subgraph direction.
- Flatten heavily looped, multi-subgraph control flows when Mermaid keeps producing tangled layouts.

## Workflow Diagram Convention

- Preserve the repo numbering convention when extending workflow diagrams:
  - `x`: major agent number
  - `y`: agent-related decision number
  - `z`: utility or non-agent step number such as start, done, or fmt plus commit
- Use `{{"text"}}` for decision diamonds.
- Keep diamond text shorter than rectangle text because the usable width is smaller.
