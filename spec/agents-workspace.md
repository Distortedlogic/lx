# Collaborative Workspace

Shared mutable artifact for concurrent multi-agent editing with region claiming and conflict resolution.

## Problem

`agent.reconcile` merges parallel results after the fact — agents work independently, then merge. `std/blackboard` (roadmap, no spec) is a key-value store for shared state.

Neither supports **concurrent editing of the same artifact**. In `software_diffusion.lx`, Stage 4 serially sends each function to an implementer because there's no way for multiple agents to edit the same codebase simultaneously. What's needed: multiple agents working on the same document/artifact with region claiming, incremental edits, and structured conflict resolution.

## Design

### Module: `std/workspace`

```lx
use std/workspace

ws = workspace.create "project-code" {
  content: initial_code
  conflict_strategy: "last-writer-wins"
} ^

region_a = workspace.claim ws "module_auth" {start: 0 end: 50} ^
region_b = workspace.claim ws "module_api" {start: 51 end: 120} ^

workspace.edit ws region_a new_auth_code ^
workspace.edit ws region_b new_api_code ^

workspace.release ws region_a
workspace.release ws region_b

final = workspace.snapshot ws ^
conflicts = workspace.conflicts ws ^
workspace.resolve ws conflict_id resolution ^
```

### Core Functions

| Function | Signature | Purpose |
|---|---|---|
| `workspace.create` | `(name: Str opts: Record) -> Result Workspace Str` | Create shared workspace |
| `workspace.claim` | `(ws: Workspace region: Str bounds: {start: Int end: Int}) -> Result Region Str` | Claim exclusive region |
| `workspace.claim_pattern` | `(ws: Workspace region: Str pattern: Str) -> Result Region Str` | Claim by regex match |
| `workspace.edit` | `(ws: Workspace region: Region content: Str) -> Result () Str` | Edit within claimed region |
| `workspace.append` | `(ws: Workspace region: Region content: Str) -> Result () Str` | Append to region |
| `workspace.release` | `(ws: Workspace region: Region) -> ()` | Release region claim |
| `workspace.snapshot` | `(ws: Workspace) -> Result Str Str` | Get current full content |
| `workspace.regions` | `(ws: Workspace) -> [{name: Str owner: Str bounds: {start: Int end: Int}}]` | List all regions |
| `workspace.conflicts` | `(ws: Workspace) -> [Conflict]` | List unresolved conflicts |
| `workspace.resolve` | `(ws: Workspace id: Str resolution: Str) -> Result () Str` | Resolve a conflict |
| `workspace.history` | `(ws: Workspace) -> [{region: Str editor: Str at: Str}]` | Edit history |
| `workspace.watch` | `(ws: Workspace handler: Fn) -> ()` | Notify on edits |

### Region Claiming

Regions are named, non-overlapping, claimed by agent identity. Attempts to claim an overlapping region return `Err "region overlaps with: {existing}"`. Agents can claim multiple non-overlapping regions.

For code workspaces, `claim_pattern` matches a regex (e.g., `r/fn auth_.*\{[\s\S]*?\}/`) and claims the matched range, so agents don't need to know line numbers.

### Conflict Resolution

Conflicts arise when:
- Two unclaimed edits overlap (if regions weren't claimed)
- A region's bounds shift due to an adjacent edit (insertion changes line numbers)

Built-in strategies:

| Strategy | Behavior |
|---|---|
| `"last-writer-wins"` | Latest edit takes precedence |
| `"first-writer-wins"` | Earliest edit takes precedence |
| `"merge-lines"` | Line-level 3-way merge |
| `"manual"` | Queue conflict for `workspace.resolve` |
| Custom function | `(a: Str b: Str base: Str) -> Str` |

### Bound Adjustment

When an edit in region A inserts/removes lines, regions after A automatically adjust bounds. This keeps region B's content stable even as the document grows.

### Multi-Agent Usage Pattern

```lx
ws = workspace.create "codebase" {content: code conflict_strategy: "merge-lines"} ^

modules | pmap (m) {
  region = workspace.claim ws m.name {start: m.start end: m.end} ^
  impl = implementer ~>? {module: m context: workspace.snapshot ws ^} ^
  workspace.edit ws region impl.code ^
  workspace.release ws region
}

final_code = workspace.snapshot ws ^
```

### In-Process Only

Workspaces are in-process shared state (like `agent.topic`). Cross-process workspaces would require `std/registry` (planned). The workspace content is protected by `Arc<Mutex<>>` — safe for concurrent access from `par`/`pmap` blocks.

## Implementation

Pure stdlib module. No parser changes. Core data structure: `Vec<u8>` content + `BTreeMap<String, Region>` claims + edit log. Approximately 200 lines of Rust.

Region adjustment on edit: when content changes, iterate all regions after the edit point and shift bounds by the delta.

Conflict detection: check if any unadjusted region overlaps with an edit. If so, apply conflict strategy or queue for manual resolution.

## Priority

Tier 3. Enables true parallel multi-agent editing patterns. No parser changes. No dependencies on other planned features (purely in-process). Integrates with `agent.provenance` (specced) for edit attribution.
