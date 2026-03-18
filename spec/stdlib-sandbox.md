# std/sandbox — Process and Scope Sandboxing

Capability-based sandboxing for agent spawns, shell commands, and scoped execution blocks. Deny-by-default policies restrict what code can do at both the lx runtime level (RuntimeCtx backend restriction) and the OS level (Landlock + seccomp for spawned processes).

## Problem

lx agents spawn sub-agents, execute shell commands, connect to MCP servers, and make network calls. Today there's no way to restrict what a spawned agent or code block can access. An LLM-generated workflow with `agent.spawn` hands the child process full system access. A tool-generating agent that calls `$` can run arbitrary shell commands.

`agent.eval_sandbox` (specced) handles one narrow case: sandboxing dynamically eval'd code strings. But it doesn't cover:

- **Spawned agent processes** — `agent.spawn` gives the child full OS access
- **Shell blocks** — `$` and `${}` execute with the parent's full permissions
- **Scoped restriction** — no way to say "this entire block runs with read-only filesystem"
- **Capability attenuation** — no way to give a sub-agent fewer permissions than the parent
- **Policy composition** — no way to combine restrictions from multiple sources (e.g., user policy + workflow policy)

The research consensus: Landlock + seccomp is the simplest and most performant approach for process-level sandboxing (~0% overhead, instant startup, no containers, no root). This is what OpenAI Codex CLI, Cursor, and Anthropic's sandbox-runtime shipped. For lx-level restriction, RuntimeCtx backend swapping (already architected) provides the mechanism.

## Design

### Policies

A policy is a deny-by-default capability set. Everything not explicitly granted is forbidden.

```lx
use std/sandbox

-- Preset policies
pure = sandbox.policy :pure             -- no I/O, pure computation
readonly = sandbox.policy :readonly     -- read filesystem, no writes/network/shell
local = sandbox.policy :local           -- filesystem + shell, no network
network = sandbox.policy :network       -- filesystem + network, no shell
full = sandbox.policy :full             -- unrestricted (equivalent to no sandbox)
```

Custom policies with fine-grained capabilities:

```lx
policy = sandbox.policy {
  fs: {
    read: ["/data" "/tmp" "./config"]
    write: ["/tmp/output"]
  }
  net: {
    allow: ["api.example.com:443" "*.internal.corp:*"]
  }
  shell: {allow: ["git" "rg" "jq"]}
  env: {inherit: ["PATH" "HOME" "LANG"]}
  agent: true
  mcp: {allow: ["read_file" "list_dir"]}
  ai: true
  max_memory_mb: 512
  max_time_ms: 30000
}
```

### Policy Fields

| Field | Type | Default | Description |
|---|---|---|---|
| `fs.read` | `[Str]` | `[]` | Filesystem paths allowed for reading (recursive) |
| `fs.write` | `[Str]` | `[]` | Filesystem paths allowed for writing (recursive) |
| `net.allow` | `[Str]` | `[]` | Network destinations (`host:port`, globs supported) |
| `shell` | `Bool \| {allow: [Str]}` | `false` | Shell access: false=denied, true=unrestricted, list=command allowlist |
| `env.inherit` | `[Str] \| Bool` | `false` | Env vars: false=none, true=all, list=specific vars |
| `agent` | `Bool` | `false` | Can spawn sub-agents via `agent.spawn` |
| `mcp` | `Bool \| {allow: [Str]}` | `false` | MCP tool access: false=denied, true=all, list=tool allowlist |
| `ai` | `Bool` | `false` | LLM access via `std/ai` |
| `max_memory_mb` | `Int` | `0` (unlimited) | Memory ceiling for spawned processes |
| `max_time_ms` | `Int` | `0` (unlimited) | Hard time limit (enforced via timeout, not cooperative) |

### Presets

| Preset | fs.read | fs.write | net | shell | agent | mcp | ai |
|---|---|---|---|---|---|---|---|
| `:pure` | none | none | none | no | no | no | no |
| `:readonly` | cwd | none | none | no | no | no | no |
| `:local` | cwd | cwd | none | yes | no | no | no |
| `:network` | cwd | cwd | all | no | no | no | yes |
| `:full` | all | all | all | yes | yes | yes | yes |

### Scoped Restriction

Apply a policy to a code block. Everything inside runs under the policy's restrictions.

```lx
with sandbox.scope policy ^ as ctx {
  data = fs.read "/data/input.json" ^
  processed = data | json.parse ^ | transform
  fs.write "/tmp/output/result.json" (json.encode processed) ^

  $curl http://evil.com ^    -- Err: shell access denied by sandbox policy
}
```

`sandbox.scope` returns a resource handle compatible with `with...as`. On scope exit, restrictions are lifted. Nested scopes intersect — inner scope can only narrow, never widen.

```lx
with sandbox.scope outer_policy ^ as _ {
  with sandbox.scope inner_policy ^ as _ {
    -- effective policy = intersection of outer and inner
    -- inner cannot grant capabilities that outer denies
  }
}
```

### Sandboxed Agent Spawn

Spawn a sub-agent process under OS-level restrictions (Landlock + seccomp):

```lx
worker = sandbox.spawn policy {command: "lx" args: ["run" "worker.lx"]} ^

result = worker ~>? {task: "analyze" file: "main.rs"} ^
agent.kill worker
```

`sandbox.spawn` wraps `agent.spawn` — same interface, same message protocol. The child process runs under Landlock filesystem restrictions and seccomp syscall filtering. The policy's `fs`, `net`, `shell`, and `env` fields translate directly to OS-level rules.

### Sandboxed Shell Execution

Run shell commands under a sandbox without wrapping in a full scope:

```lx
result = sandbox.exec policy "git log --oneline -10" ^

result = sandbox.exec policy {
  cd /tmp
  git clone {url}
  cd repo
  make build
} ^
```

### Policy Composition

```lx
-- Merge: intersection (most restrictive wins)
strict = sandbox.merge [user_policy workflow_policy]

-- Attenuate: narrow an existing policy (can only remove capabilities)
child = sandbox.attenuate policy {
  fs: {read: ["/data/subset"]}
  net: {allow: []}
}
-- child can read /data/subset (if parent allowed /data), network denied
-- attenuate errors if you try to grant capabilities the parent doesn't have
```

### Introspection

```lx
-- Describe what a policy allows (human-readable)
sandbox.describe policy
-- {fs_read: ["/data" "/tmp"]  fs_write: ["/tmp/output"]  net: ["api.example.com:443"]  shell: ["git" "rg"]  ...}

-- Check if this platform supports OS-level enforcement
sandbox.available () ^
-- {landlock: true  seccomp: true  version: "landlock_v4"}

-- Verify a specific operation would be allowed
sandbox.permits policy :fs_read "/data/input.json"   -- true
sandbox.permits policy :fs_write "/etc/passwd"        -- false
sandbox.permits policy :shell "rm"                    -- false
sandbox.permits policy :net "evil.com:80"             -- false
```

## Use Cases

### Sandboxed Tool Generation

```lx
use std/sandbox
use std/ai

code = ai.prompt "Write a shell script that processes {spec}" ^
policy = sandbox.policy {
  fs: {read: ["./data"]  write: ["./output"]}
  shell: {allow: ["awk" "sed" "sort" "uniq" "jq"]}
  env: {inherit: ["PATH"]}
}
result = sandbox.exec policy code.text ^
```

### Defense-in-Depth for Untrusted Agents

```lx
use std/sandbox

untrusted_policy = sandbox.policy {
  fs: {read: ["./workspace"]  write: ["./workspace/output"]}
  net: {allow: []}
  shell: false
  ai: true
  max_memory_mb: 256
  max_time_ms: 60000
}

worker = sandbox.spawn untrusted_policy {
  command: "lx" args: ["run" "community_agent.lx"]
} ^

result = worker ~>? {task: "analyze" data: input} ^
```

### Pipeline Stage Isolation

```lx
use std/sandbox
use std/pipeline

pipe = pipeline.create "audit" {resume: true} ^

findings = pipeline.stage pipe "scan" code {
  scan_policy = sandbox.policy {
    fs: {read: [code]}
    shell: {allow: ["rg" "semgrep"]}
  }
  with sandbox.scope scan_policy ^ as _ {
    run_scanners code ^
  }
} ^
```

### Capability Attenuation Across Agent Hierarchy

```lx
use std/sandbox

org_policy = sandbox.policy {
  fs: {read: ["/repo"]  write: ["/repo/output"]}
  net: {allow: ["*.internal.corp:*"]}
  shell: {allow: ["git" "rg"]}
  ai: true
  agent: true
}

-- Each sub-agent gets a narrower slice
reviewer_policy = sandbox.attenuate org_policy {
  fs: {read: ["/repo/src"]  write: []}
  shell: {allow: ["rg"]}
}

implementer_policy = sandbox.attenuate org_policy {
  fs: {read: ["/repo"]  write: ["/repo/output"]}
  shell: {allow: ["git" "rg"]}
}

reviewer = sandbox.spawn reviewer_policy {command: "lx" args: ["run" "reviewer.lx"]} ^
implementer = sandbox.spawn implementer_policy {command: "lx" args: ["run" "impl.lx"]} ^
```

## Core Functions

| Function | Signature | Purpose |
|---|---|---|
| `sandbox.policy` | `(preset: Symbol \| config: Record) -> Result Policy Str` | Create a sandbox policy |
| `sandbox.scope` | `(policy: Policy body: Fn) -> Result Any Str` | Execute block under policy restrictions |
| `sandbox.spawn` | `(policy: Policy config: Record) -> Result AgentHandle Str` | Spawn sandboxed agent process |
| `sandbox.exec` | `(policy: Policy cmd: Str) -> Result ShellResult Str` | Run shell command under policy |
| `sandbox.merge` | `(policies: [Policy]) -> Policy` | Combine policies (intersection) |
| `sandbox.attenuate` | `(parent: Policy overrides: Record) -> Result Policy Str` | Narrow a policy (errors on widening) |
| `sandbox.describe` | `(policy: Policy) -> Record` | Describe policy capabilities |
| `sandbox.available` | `() -> Result Record Str` | Check platform sandbox support |
| `sandbox.permits` | `(policy: Policy cap: Symbol target: Str) -> Bool` | Check if operation is allowed |

## Implementation

### Two Enforcement Layers

**Layer 1 — lx runtime (RuntimeCtx restriction):**

`sandbox.scope` creates a child `RuntimeCtx` with restricted backends. Same mechanism as `agent.eval_sandbox`:

- `:pure` / no shell → `DenyShellBackend` that returns `Err("shell access denied by sandbox policy")`
- No network → `DenyHttpBackend`
- No AI → `DenyAiBackend`
- fs restrictions → `RestrictedShellBackend` that checks paths before delegating to `ProcessShellBackend`
- MCP restrictions → `RestrictedMcpBackend` that checks tool names before delegating

This layer catches violations at the lx function level before any syscall happens.

**Layer 2 — OS enforcement (Landlock + seccomp):**

`sandbox.spawn` and `sandbox.exec` apply OS-level restrictions to child processes:

- **Landlock** (Linux 5.13+): filesystem access rules. Map `fs.read`/`fs.write` paths to Landlock `path_beneath` rules with appropriate access flags.
- **seccomp** (Linux 3.5+): syscall filtering. Block `connect`/`sendto` when network denied. Block `execve` when shell denied.
- **cgroups v2** (optional): enforce `max_memory_mb` via memory controller.

On platforms without Landlock/seccomp support, `sandbox.spawn` returns `Err "OS-level sandboxing unavailable (requires Linux 5.13+)"`. `sandbox.scope` (Layer 1 only) works everywhere.

### Rust Dependencies

- `landlock` crate (well-maintained, Rust-native Landlock bindings)
- `seccompiler` crate (Amazon's seccomp BPF compiler, used by Firecracker)
- No container runtime, no root, no daemon

### File Structure

- `crates/lx/src/stdlib/sandbox.rs` — module entry, `build()`, policy creation, describe, permits
- `crates/lx/src/stdlib/sandbox_scope.rs` — scope enforcement, restricted backend wrappers
- `crates/lx/src/stdlib/sandbox_spawn.rs` — OS-level sandboxing for spawn/exec
- `crates/lx/src/backends/restricted.rs` — `Deny*Backend` and `Restricted*Backend` implementations

### Policy Nesting Invariant

When scopes nest, the effective policy is always the intersection. Implementation: each `sandbox.scope` reads the current policy from a thread-local (or RuntimeCtx field), intersects with the new policy, and sets the result. On scope exit, the previous policy is restored. `sandbox.attenuate` enforces the same invariant at policy-creation time — it's a compile-time version of runtime intersection.

### Relationship to `agent.eval_sandbox`

`agent.eval_sandbox` is a convenience wrapper for a common case: sandbox + parse + eval + return function. It's equivalent to:

```lx
fn = sandbox.scope (sandbox.policy :pure) {
  eval code  -- (hypothetical eval, but eval_sandbox handles the parsing)
}
```

When `std/sandbox` ships, `agent.eval_sandbox` can be reimplemented on top of it. The permission symbols (`:pure`, `:read_fs`, `:ai`, `:network`, `:full`) map directly to sandbox presets.

## Integration

- **`agent.spawn`** — `sandbox.spawn` wraps it. When the `with context` ambient propagation (Tier 3) ships, policies can auto-propagate to spawns without explicit `sandbox.spawn`.
- **`std/deadline`** — `max_time_ms` is a hard kill; `std/deadline` is cooperative. Use both: deadline for graceful degradation, sandbox timeout as backstop.
- **`std/budget`** — orthogonal. Budget tracks cost, sandbox restricts capabilities.
- **`with...as`** — `sandbox.scope` is a resource that participates in the existing scoped resource protocol.
- **`std/pipeline`** — pipeline stages can each run under different sandbox policies for stage-level isolation.
- **`Trait` capabilities** — `Trait.requires: [:fs :ai]` declares what an agent needs. A sandbox policy can be derived from trait requirements: grant exactly what the trait requires, nothing more.

## Priority

Tier 2. Sandboxing is foundational for any agentic system running untrusted or LLM-generated code. No parser changes needed. Pure stdlib module with optional OS-level enhancement. The lx-level enforcement (Layer 1) is ~200 lines of Rust reusing the existing `agent.eval_sandbox` mechanism. OS-level enforcement (Layer 2) adds ~150 lines using established crates.
