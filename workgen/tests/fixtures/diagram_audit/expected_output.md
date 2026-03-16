# Goal

Fix diagram quality violations: vertical bloat, invisible edges causing layout shifts, spaghetti from multiple constraint=false edges, dummy clusters too small to read, and pause/resume on same edge.

# Why

The diagram stacks nodes vertically that could be horizontal peers. Invisible edges between A→C and B→D create unintended layout shifts without `constraint="false"`. Two separate `constraint="false"` edges from E→B ("fix" and "next task") overlap into spaghetti. The inner cluster has only a plaintext node X that renders as a tiny unreadable box. A single dashed edge labeled "retry / resume" combines two distinct actions that happen at different flow points.

# What changes

- Use `rank="same"` subgraphs for peer nodes to reduce vertical bloat
- Add `constraint="false"` to invisible edges to prevent layout shifts
- Combine the two E→B edges into a single edge with a multi-line label to fix spaghetti
- Give the inner cluster 3+ nodes with abbreviated labels so it has visual weight instead of a dummy ghost cluster
- Split "retry / resume" into separate edges from appropriate trigger points — pause and resume are distinct actions

# Files affected

- src/diagram.py — vertical bloat, invisible edge layout shifts, spaghetti edges, dummy cluster, combined pause/resume edge

# Task List

## Task 1: Fix layout and invisible edges

Group peer nodes with `rank="same"`. Add `constraint="false"` to invisible edges.

```
just fmt
git add src/diagram.py
git commit -m "fix: reduce vertical bloat, fix invisible edge constraints"
```

## Task 2: Fix spaghetti and dummy cluster

Combine duplicate constraint=false edges into single multi-label edge. Add substantive nodes to inner cluster.

```
just fmt
git add src/diagram.py
git commit -m "fix: combine spaghetti edges, expand dummy cluster"
```

## Task 3: Split pause/resume edge

Split "retry / resume" into separate edges from the appropriate trigger nodes.

```
just fmt
git add src/diagram.py
git commit -m "fix: split combined pause/resume into separate edges"
```

## Task 4: Verification

```
just test
just diagnose
just fmt
git add -A
git commit -m "chore: verify diagram audit fixes"
```

# CRITICAL REMINDERS

- Run `just fmt` after every file change
- Render and visually inspect after every structural change
- No invisible edges without constraint="false" unless layout shift is intentional

# Task Loading Instructions

Load these instructions by reading this file, then execute each task in order. Render after each change to verify.
