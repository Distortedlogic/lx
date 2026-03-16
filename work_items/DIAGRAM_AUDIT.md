# Goal

Fix all six diagram quality violations in workgen/tests/fixtures/diagram_audit/src/diagram.py. The current Graphviz diagram suffers from vertical bloat, invisible edges that distort layout, overlapping loop-back edges, a combined pause/resume label on a single edge, a ghost cluster too small to read, and orphaned layout artifacts. After this work item, the diagram renders as a clean, horizontally compact flow with no spaghetti, no orphaned elements, and correct semantic separation of retry vs resume edges.

# Why

- Vertical bloat wastes vertical space — peer processing nodes B, C, D are stacked when they could sit side-by-side
- Invisible edges A→C and B→D impose conflicting rank constraints that distort layout unpredictably
- Two separate constraint=false edges from E→B overlap into unreadable spaghetti
- The "retry / resume" label on D→B conflates two distinct control-flow actions onto one edge
- cluster_inner contains a single empty plaintext node, rendering as a tiny unreadable box
- The invisible edges and cluster_inner are orphaned artifacts with no edges connecting them to the main flow

# What changes

1. **Horizontal rank grouping** — Wrap nodes B, C, D in a rank=same subgraph so they sit side-by-side instead of stacking vertically. Adjust edges between them to flow horizontally.

2. **Remove invisible edges** — Delete both invisible edges (A→C and B→D). They serve no discernible layout purpose and their rank constraints conflict with the visible flow path.

3. **Combine overlapping loop-back edges** — Replace the two separate constraint=false edges from E to B (labeled "fix" and "next task") with a single edge carrying a multi-line label "fix\nnext task".

4. **Split retry/resume into separate edges** — Replace the single D→B edge labeled "retry / resume" with two distinct edges: a retry edge from C (validation failure) back to B, and a resume edge from D back to B. Each carries its own label.

5. **Populate ghost cluster** — Replace the single empty plaintext node X in cluster_inner with three abbreviated-label nodes (Parse, Transform, Emit) connected by edges, giving the cluster enough visual weight to be readable. Connect the cluster to the main flow with at least one edge.

6. **Remove orphaned artifacts** — Both invisible edges are already removed in change 2. The cluster_inner is now populated and connected in change 5, so no orphaned elements remain.

# Files affected

- workgen/tests/fixtures/diagram_audit/src/diagram.py — all six fixes applied to the single diagram generation file

# Task List

## Task 1: Remove orphaned invisible edges and add horizontal rank grouping

**Subject:** Remove invisible edges and group B, C, D horizontally

**Description:**

Edit workgen/tests/fixtures/diagram_audit/src/diagram.py:

- Delete line 18: g.edge('A', 'C', style='invis')
- Delete line 19: g.edge('B', 'D', style='invis')
- After the node declarations (after line 10), add a rank=same subgraph containing nodes B, C, D using g.subgraph with graph_attr rank=same. The three nodes are already declared above, so the subgraph just references them to enforce horizontal placement.
- Keep the existing edges A→B, B→C, C→D, D→E intact — they now flow horizontally through the rank=same group and then down to E.

Verification: the file no longer contains any style='invis' edges and contains a rank=same subgraph grouping B, C, D.

Run: just fmt, git add workgen/tests/fixtures/diagram_audit/src/diagram.py, git commit with message "Remove orphaned invisible edges, group peer nodes horizontally"

## Task 2: Combine overlapping E→B loop-back edges into single edge

**Subject:** Merge duplicate E→B edges into one multi-line label

**Description:**

Edit workgen/tests/fixtures/diagram_audit/src/diagram.py:

- Delete both lines: g.edge('E', 'B', constraint='false', label='fix') and g.edge('E', 'B', constraint='false', label='next task')
- Replace with a single edge: g.edge('E', 'B', constraint='false', label='fix\nnext task')

Verification: only one edge exists from E to B, and its label contains both "fix" and "next task" separated by a newline.

Run: just fmt, git add workgen/tests/fixtures/diagram_audit/src/diagram.py, git commit with message "Combine overlapping E-to-B loop-back edges into single multi-line label"

## Task 3: Split retry/resume into separate edges from distinct source nodes

**Subject:** Split D→B retry/resume into two semantically correct edges

**Description:**

Edit workgen/tests/fixtures/diagram_audit/src/diagram.py:

- Delete the line: g.edge('D', 'B', label='retry / resume', style='dashed')
- Add two replacement edges:
  - g.edge('C', 'B', label='retry', style='dashed') — retry originates from Validate (C) on validation failure
  - g.edge('D', 'B', label='resume', style='dashed') — resume originates from Store (D) to continue processing

Verification: no edge exists with the label "retry / resume". Two separate dashed edges exist: C→B labeled "retry" and D→B labeled "resume".

Run: just fmt, git add workgen/tests/fixtures/diagram_audit/src/diagram.py, git commit with message "Split retry/resume into separate edges from correct source nodes"

## Task 4: Populate ghost cluster with readable content and connect to flow

**Subject:** Replace empty ghost cluster with three-node abbreviated structure

**Description:**

Edit workgen/tests/fixtures/diagram_audit/src/diagram.py:

- In the cluster_inner subgraph, remove the single node X with empty label and plaintext shape.
- Add three nodes inside the subgraph: s.node('X1', 'Parse'), s.node('X2', 'Transform'), s.node('X3', 'Emit') — these echo the processing structure of the main flow in abbreviated form.
- Add edges between them inside the subgraph: s.edge('X1', 'X2') and s.edge('X2', 'X3').
- After the subgraph block, connect the cluster to the main flow by adding an edge from B to X1 with label='detail' and style='dotted'. This makes the cluster reachable and semantically tied to the Process node.

Verification: cluster_inner contains three nodes with non-empty labels, two internal edges, and one external edge connecting it to the main flow. No plaintext-shaped empty node exists.

Run: just fmt, git add workgen/tests/fixtures/diagram_audit/src/diagram.py, git commit with message "Populate ghost cluster with readable nodes and connect to main flow"

## Task 5: Final verification — render review and full test suite

**Subject:** Run full verification suite

**Description:**

- Run: just test
- Run: just diagnose
- Run: just fmt

Confirm all tests pass, no diagnostics errors, and formatting is clean.

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.

# Task Loading Instructions

To begin executing this work item, run:

```
mcp__workflow__load_work_item({ path: "work_items/FIX_DIAGRAM_AUDIT_VIOLATIONS.md" })
```