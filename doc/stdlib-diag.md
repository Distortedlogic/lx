# std/diag — Reference

## CLI: `lx diagram`

```
lx diagram file.lx              # Mermaid text to stdout
lx diagram file.lx -o flow.md   # Mermaid text to file
```

Parses `.lx`, walks AST, emits Mermaid flowchart text. No external dependencies.

## Library API

```
use std/diag

extract src                   -- Graph ^ ParseErr (parse lx source string, produce graph)
extract_file path             -- Graph ^ IoErr (read file then extract)
to_mermaid graph              -- Str (Mermaid flowchart text)
```

## AST-to-Diagram Mapping

| lx construct | Diagram element |
|---|---|
| `agent.spawn` | Node (box) |
| `agent.kill` | Node termination marker |
| `~>` | Directed edge (fire-and-forget, dashed) |
| `~>?` | Directed edge (request-response, solid) |
| `~>>?` | Directed edge (streaming, double-line) |
| `par { }` | Parallel lanes / fork-join group |
| `sel { }` | Race/choice diamond |
| `\|` pipe chain | Sequential flow arrows |
| `? { arms }` | Decision diamond with branches |
| `loop { }` | Cycle back-edge |
| `Protocol` | Edge label / contract annotation |
| `mcp.call` | External tool node (different shape) |
| `emit` | Output node (terminal) |

## Graph IR

```
Graph = {nodes: [Node]  edges: [Edge]}
Node = {id: Str  label: Str  kind: Str  children: [Node]}
Edge = {from: Str  to: Str  label: Str  style: Str}
```

`kind` values: `"agent"`, `"tool"`, `"decision"`, `"fork"`, `"join"`, `"terminal"`, `"emit"`.
`style` values: `"solid"`, `"dashed"`, `"double"`.

## Example

```
use std/diag

graph = diag.extract_file "flows/agentic_loop.lx" ^
graph.nodes | filter (.kind == "agent") | each (n) emit "agent: {n.label}"
graph.edges | filter (.style == "solid") | each (e) emit "{e.from} -> {e.to}: {e.label}"
```
