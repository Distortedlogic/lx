# Diagram Quality Audit (Graphviz / Python)

Every item below is a binary check — a violation either exists or it does not. There is no "partially violates" or "could be improved." The audit checks each diagram generation file and its rendered output.

Run the **High Frequency** list first — these violations are commonly introduced by AI agents generating graphviz diagrams. Run the **Low Frequency** list second — these are rarer structural issues.

---

## High Frequency Checks

- **Vertical bloat** - nodes that could be arranged horizontally on the same rank are stacked vertically, wasting vertical space. Loops, linear chains, and peer nodes within a cluster should use `rank="same"` subgraphs to sit side by side. Fix: group peer nodes into rank=same subgraphs and use horizontal edges between them.

- **Unverified render output** - a diagram change is declared correct without visually inspecting the rendered image. Every structural change must be followed by rendering and honest assessment of the result against the intent. Fix: render after every change, describe what is actually visible in the image, flag discrepancies.

- **Claiming arrows route cleanly when they don't** - describing arrow routing (e.g., "goes straight down," "routes around") without verifying against the rendered image. Graphviz edge routing is unpredictable and frequently produces curves, crossings, or diagonal paths that contradict the code's intent. Fix: inspect the rendered image and report the actual arrow path honestly. If the arrow doesn't route as intended, say so and iterate.

- **Invisible edges creating unintended layout shifts** - invisible edges (`style="invis"`) used for node ordering that pull clusters or nodes away from their intended position. Invisible edges are constraining by default and affect rank placement of the entire cluster they touch. Fix: audit every invisible edge for unintended side effects on surrounding layout. Use `constraint="false"` where the edge should hint ordering without affecting vertical placement.

- **Cross-cluster rank=same conflicts** - `rank="same"` subgraphs that reference nodes already in a cluster-internal rank=same group, causing graphviz warnings and nodes being yanked out of their clusters. Fix: use `newrank="true"` on the graph, or avoid cross-cluster rank constraints. Prefer invisible edges or shared wrapper clusters for cross-cluster alignment.

- **Port hints that don't improve routing** - compass port specifications (`:n`, `:s`, `:e`, `:w`) added to edges expecting them to fix routing, when the actual issue is node/cluster positioning. Port hints only control which side of a node an edge connects to — they do not move nodes. Fix: address the root cause (node positioning via rank constraints, invisible edges, or cluster restructuring) before resorting to port hints.

- **Node ordering fights with edge direction** - using invisible edges in one direction to force horizontal ordering while the visible flow edges go the opposite direction, creating visual back-tracking and spaghetti arrows. Fix: ensure invisible ordering edges and visible flow edges agree on direction, or use node definition order within rank=same subgraphs instead of invisible edges.

- **Spaghetti from multiple constraint=false edges** - multiple `constraint="false"` loop-back edges between the same pair of nodes (e.g., separate "fix" and "next task" edges both going from A back to B) that overlap and become unreadable. Fix: combine into a single edge with a multi-line label, or use different port hints to separate the paths visually.

- **Dummy/ghost clusters too small to read** - placeholder clusters intended to represent abbreviated versions of a detailed cluster that use `plaintext` shape or minimal content, rendering as tiny unreadable boxes. Fix: give ghost clusters enough internal nodes (3+ with abbreviated labels) to have visual weight and clearly echo the structure they represent.

- **Arrows targeting specific nodes when cluster boundary suffices** - edges from external clusters pointing to a specific internal node when the semantic is "enters this cluster." Fix: use `compound="true"` on the graph and `lhead="cluster_name"` on the edge to terminate arrows at the cluster boundary.

- **Pause/resume on same edge** - a single edge labeled "pause / resume" connecting two nodes when pause and resume are separate actions that happen at different points in the flow. Fix: split into separate edges from the appropriate trigger nodes — pause from the entry point, resume from the completion point.

- **Horizontal ordering not matching flow** - nodes within a cluster ordered left-to-right in a way that creates unnecessarily long edges from upstream/downstream nodes. Fix: order horizontal nodes so that entry points are closest to upstream connections and exit points are closest to downstream connections.

---

## Low Frequency Checks

- **Cluster nesting not reflecting logical hierarchy** - peer concepts at the same logical level placed as siblings when they should share a parent cluster, or concepts at different logical levels placed in the same cluster. Fix: add wrapper clusters for logical groupings, rename inner clusters if the wrapper takes their name.

- **Missing hierarchical node labels** - nodes in a complex diagram have no systematic labeling scheme for cross-referencing in discussion. Fix: add hierarchical numbering (e.g., `1.1`, `1.5.2`, `2.3.1`) where each `.` denotes a deeper level of nesting.

- **Weight attributes ignored** - `weight` attribute used on edges expecting it to fix node alignment, when the real issue is competing rank constraints or invisible edges. `weight` only influences how strongly graphviz prefers a short, straight edge — it cannot override rank assignments. Fix: address the root cause (rank constraints, invisible edges) instead of adding weight.

- **Orphaned layout artifacts** - invisible edges, rank constraints, or wrapper clusters left over from previous iterations that no longer serve a purpose or actively fight the current layout. Fix: audit all layout-control elements after restructuring and remove those that are stale.
