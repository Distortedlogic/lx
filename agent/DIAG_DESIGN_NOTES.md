# stdlib/diag — Design Notes & Future Work

## Current State (2026-03-17)

The diag module generates Mermaid flowcharts from lx AST. It handles every
Stmt variant, every Expr variant, and all flow-relevant stdlib module calls.
Functions are grouped into Mermaid subgraphs. Output validates against
`@mermaid-js/mermaid-cli`.

## Architecture

Four files under 300 lines each:
- `diag.rs` — stdlib API (extract, extract_file, to_mermaid) + mermaid rendering
- `diag_walk.rs` — Walker struct, visit_program (pre-registration), visit_binding, visit_agent_decl, visit_use, etc.
- `diag_walk_expr.rs` — visit_expr_diag (Apply handler with uncurry_call/classify_call/handle_call)
- `diag_helpers.rs` — pure helper functions (extract_field_call_parts, resolve_target, etc.)

## Design Decisions

**Utility modules excluded from diagrams**: prompt, json, math, re, md, env,
time, test, introspect, describe, diag, audit, ctx. These are data-processing
calls, not flow steps. Including them creates massive noise (e.g., prompt
builder chains generate ~10 nodes per AI call site).

**Pre-registration pass**: visit_program scans all function bindings first to
populate fn_nodes before walking any bodies. This solves forward-reference
issues (main calls run, but run is declared after main).

**Resource argument scanning**: Resource action calls (trace.record, knowledge.store)
scan ALL curried arguments for tracked resource variables, not just the first.
In lx's curried calling convention, the resource handle is typically the last arg
(e.g., `trace.record {data} session`).

## Known Limitations & Future Work

### 1. Prompt builder visibility
Currently prompt.* calls are completely hidden. A better approach: detect the
prompt builder chain pattern and collapse it into a single "prompt" annotation
on the subsequent ai.prompt_with node. The ai.prompt_with node could show
which sections were built. This requires pattern detection across sequential
statements within a function body.

### 2. Subgraph nesting
Currently subgraphs are flat (one level). Functions called from within a loop
or match arm could benefit from nested subgraphs showing the scope hierarchy.
Mermaid supports nested subgraphs.

### 3. Edge labels for function calls
When `run` calls `investigate`, the edge has no label. It could show the
arguments being passed, or at minimum the function name as a label. Currently
fn_nodes edges are unlabeled.

### 4. Duplicate node deduplication
Multiple calls to the same stdlib function (e.g., `fs.read` called twice in
`run`) create separate nodes. Could optionally merge identical nodes and show
edge count or fan-in.

### 5. AgentDecl body walking improvements
Agent declarations walk init/on/methods, but the `uses` field (MCP tool
imports) isn't represented in the diagram. Each `uses` entry could generate
a tool node connected to the agent.

### 6. Pipe chain handling
Pipe expressions (`x | map f | filter g | join ","`) are walked via default
walk_expr but don't show in the diagram. For pipes that feed into significant
operations (e.g., `items | pmap (item) { agent.spawn ... }`), the pipe
structure could be relevant.

### 7. lx language considerations for diag
- **Builder pattern annotation**: lx could have a `@builder` or `@pipe-chain`
  annotation that tells the diagram generator to collapse a sequence of calls
  into a single labeled node. Without this, the walker has to guess which call
  chains are "builders" vs "flow steps."
- **Diagram hints in comments**: Header comments (`--`) could contain structured
  directives like `-- @diag:collapse prompt` or `-- @diag:label "Phase 1"`
  that guide diagram generation without changing runtime behavior.
- **Subgraph boundaries from `with` blocks**: The `with` expression introduces
  a named scope. These could naturally map to nested subgraphs in the diagram.
