# Standard Library — Program Visualization (Future)

**Status: Implemented.** Session 36. `std/diag` library + `lx diagram` CLI subcommand.

## std/diag — Program Visualization (Mermaid)

Two entry points: `lx diagram` CLI subcommand for users, `std/diag` library for lx programs.

### CLI: `lx diagram`

```
lx diagram file.lx              # Mermaid text to stdout
lx diagram file.lx -o flow.md   # Mermaid text to file
```

Parses the `.lx` file, walks the AST to extract a workflow graph (agents, messages, control flow), and emits Mermaid flowchart text. No external dependencies — the output IS the diagram. Paste into GitHub markdown, VS Code preview, or any Mermaid renderer.

### Library: `std/diag`

Exposes the same pipeline programmatically for lx programs that want to inspect, filter, or transform the graph IR before emitting.

```
extract src                   -- Graph ^ ParseErr (parse lx source, walk AST, produce graph)
extract_file path             -- Graph ^ IoErr (read file then extract)
to_mermaid graph              -- Str (Mermaid flowchart text)
```

### AST → Diagram Mapping

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

### Graph IR

The intermediate representation is plain lx records — no opaque types.

```
Graph = {nodes: [Node]  edges: [Edge]}
Node = {id: Str  label: Str  kind: Str  children: [Node]}
Edge = {from: Str  to: Str  label: Str  style: Str}
```

`kind` values: `"agent"`, `"tool"`, `"decision"`, `"fork"`, `"join"`, `"terminal"`, `"emit"`.
`style` values: `"solid"`, `"dashed"`, `"double"`.

### Patterns

CLI — the primary way users generate diagrams:
```
lx diagram flows/agentic_loop.lx
lx diagram flows/agentic_loop.lx -o flow.md
```

Agent inspecting the graph IR programmatically:
```
use std/diag

graph = diag.extract_file "flows/scenario_research.lx" ^
graph.nodes | filter (.kind == "agent") | each (n) emit "agent: {n.label}"
graph.edges | filter (.style == "solid") | each (e) emit "{e.from} -> {e.to}: {e.label}"
```

Agent generating Mermaid text:
```
use std/diag
use std/fs

graph = diag.extract_file "flows/agentic_loop.lx" ^
text = diag.to_mermaid graph
fs.write "flow.md" text ^
```

### Why Mermaid

lx targets agents and developers working with agents. Mermaid is the right format for this use case:

- **Text-native** — agents/LLMs can read and reason about Mermaid directly. No binary format to decode.
- **Renders everywhere** — GitHub, VS Code preview, Notion, any markdown viewer. Zero install for the user.
- **Workflow DAGs are simple graphs** — labeled nodes and directed edges. Mermaid handles this well. Publication-quality layout doesn't matter for working visualizations of agent workflows.

The output IS the rendered diagram — Mermaid text is immediately useful without a conversion step. No external binaries (`d2`, `mmdc`) needed, no shell-out, no format selection.

## Cross-References

- Data ecosystem modules: [stdlib-data.md](stdlib-data.md) (std/df, std/plot, std/db, etc.)
- Agent ecosystem modules: [stdlib-agents.md](stdlib-agents.md) (std/agent, std/mcp, std/ctx, std/md)
- Core stdlib modules: [stdlib-modules.md](stdlib-modules.md)
- Built-in functions and conventions: [stdlib.md](stdlib.md)
- Stdlib loader design: [impl-stdlib.md](../design/impl-stdlib.md)
- Stdlib roadmap: [stdlib_roadmap.md](../design/stdlib_roadmap.md)
