# Mermaid Knowledge — Lessons Learned

Everything here was discovered through trial and error in this repo. None of it is in default training data.

## Background Color

- `mmdc` defaults `-b` to `"white"` — the page background is always white unless overridden
- The mermaid config file (`themeVariables.background`) does NOT control the page/canvas background — it controls internal color calculations
- The only way to set page background is `mmdc -b "#000000"` on the CLI
- Keep `-b` in the justfile recipe, not in the config

## Theme

- `"theme": "base"` is the only theme where `themeVariables` are fully modifiable
- `"theme": "dark"` ignores most custom `themeVariables` — it uses its own built-in values
- The classDefs in `.mmd` files (`fill:#000,stroke:#4ec9b0`) are inline overrides that work regardless of theme
- If classDefs duplicate what the theme provides, remove them from `.mmd` and let the theme handle it
- Per-node stroke colors (category coloring) can be hoisted into `themeCSS` as CSS classes so they don't repeat in every `.mmd` file

## themeCSS

- `themeCSS` in the config injects raw CSS into the rendered SVG
- Node class selectors: `.node.classname > rect` or `.node.classname > polygon`
- Left-align text in nodes: `.label foreignObject div { text-align: left !important; }`
- Example for custom stroke colors: `.node.orange > rect, .node.orange > polygon { stroke: #ff6600 !important; }`
- The `:::classname` syntax in `.mmd` files still works — it applies the CSS class to the node element
- `!important` is needed to override mermaid's inline styles

## Subgraph Styling

- `clusterBkg` themeVariable controls subgraph fill color
- `clusterBorder` themeVariable controls subgraph border color
- These apply uniformly to ALL subgraphs — no per-subgraph theming via config
- Per-subgraph stroke colors require inline `style` directives in the `.mmd` file: `style subgraph_id stroke:#ff6600,stroke-width:2px`

## Subgraph Direction

- `direction TB` inside a subgraph sets internal layout direction
- **CRITICAL**: if ANY node inside the subgraph connects to a node outside the subgraph, the direction is IGNORED and the subgraph inherits the parent graph's direction
- To preserve internal direction, edges must connect to the subgraph ID, not to internal node IDs
- Even then, complex flows with multiple subgraphs connected in a loop produce bad layouts
- For complex control flows, flat single-node layouts are more reliable than nested subgraphs

## Text in Nodes

- Backtick markdown labels (`` ` ``) support **bold**, *italic*, and explicit line breaks
- `\n` in regular labels creates line breaks
- Mermaid auto-wraps text at `flowchart.wrappingWidth` pixels (default 200)
- Set `"wrappingWidth": 99999` to effectively disable auto-wrapping — lines only break where the source has explicit line breaks
- Whitespace/indentation inside backtick labels is COLLAPSED — leading spaces are stripped
- To fake indentation, use a prefix character like `→` or `··`
- Text in nodes is CENTER-ALIGNED by default — use `themeCSS` with `text-align: left` to override
- Monospace 14px is roughly 8.4px per character — use this to estimate wrapping

## Node Ordering and Layout

- Declaration order of edges from a node controls left-to-right placement of child nodes
- First declared edge goes left, second goes right
- To control which branch appears on which side, reorder the edge declarations
- Mermaid's auto-layout (dagre) struggles with back-edges in loops — the loop-back arrow may cross other elements
- The ELK renderer (`"defaultRenderer": "elk"`) handles some layouts differently but is not universally better — test both

## Rendering with mmdc

- `-s 2` doubles the scale factor — produces 2x resolution PNGs
- `-w` and `-H` control page width/height
- `-c configFile` loads mermaid config — controls theme, variables, flowchart settings
- `-b` controls page background — separate from theme
- Config file is for mermaid diagram settings, `-b` is for mmdc's puppeteer page background
- The config file follows mermaid's `MermaidConfig` schema, not mmdc's CLI schema

## Flowchart Config Properties

- `wrappingWidth` (number, default 200) — width before text wraps
- `nodeSpacing` (integer, default 50) — spacing between nodes on same level
- `rankSpacing` (integer, default 50) — spacing between levels
- `diagramPadding` (integer, default 20) — padding around entire diagram
- `padding` (number, default 15) — padding between label and shape
- `curve` (string, default "basis") — edge curve style
- `defaultRenderer` (string) — "dagre-d3", "dagre-wrapper", or "elk"
- No text alignment property exists at the flowchart config level

## Invisible Links

- `~~~` creates invisible links between nodes — used to control horizontal positioning
- `A ~~~ B ~~~ C` places A, B, C side by side without visible edges
- Useful inside subgraphs to create column layouts

## Rendering

- After every `.mmd` edit, run `just diagrams` to regenerate PNGs
- Stale PNGs after a diagram edit is incomplete work

## Node Numbering Convention (x.y.z)

- `x` — each agent gets its own major number
- `y` — agent-related decision nodes get their own minor number
- `z` — non-agent nodes (utility steps like fmt + commit, start, done) get their own patch number

## Diamond Shapes

- `{{"text"}}` creates a hexagon/diamond decision node
- Text inside diamonds has less usable space than rectangles — keep it short
- Multi-line text in diamonds with `\n` works but gets cramped
