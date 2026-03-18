# Workspace Spec

lx projects can be single packages or workspaces containing multiple members. Inspired by Cargo workspaces — shared dependency resolution, unified test/check/run commands, cross-member imports by name.

## Problem

The lx repo has four lx program collections (tests/, brain/, workgen/, flows/) but no project-level awareness. Running tests is fragmented across different commands. There's no way to verify "did a language change break any lx program." Members can't import from each other by name — they use brittle relative paths like `use ../../brain/protocols`.

## Manifest: `lx.toml`

### Root workspace manifest

```toml
[workspace]
members = ["tests", "brain", "workgen", "flows"]

[workspace.deps]
# shared dependency declarations (future: external packages)
```

The root `lx.toml` declares the workspace. `members` is an ordered list of directory paths relative to the root. Each member directory must contain its own `lx.toml`.

### Member manifest

```toml
[package]
name = "brain"
version = "0.1.0"
entry = "main.lx"

[test]
dir = "tests/"
pattern = "test_*.lx"

[deps]
# empty for now — workspace members are auto-importable
```

### Package fields

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `name` | Str | yes | — | Package identifier (lowercase, hyphens) |
| `version` | Str | no | "0.0.0" | Semver version |
| `entry` | Str | no | "main.lx" | Main file for `lx run` |
| `description` | Str | no | — | One-line description |

### Test fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `dir` | Str | "tests/" | Directory containing test files |
| `pattern` | Str | "*.lx" | Glob pattern for test discovery |
| `runner` | Str | "suite" | Test runner: "suite" (assert-based) or "describe" (BDD) |

### Workspace deps

Workspace members are automatically available as imports. No `[deps]` entry needed:

```lx
use brain/protocols {Perception Response}
use workgen/main {run}
```

The module resolver checks workspace members before falling back to stdlib. Resolution: `use brain/X` → find member named "brain" → resolve `X.lx` from that member's root.

## CLI Changes

### `lx test` — workspace-aware testing

```
lx test              # run all member tests
lx test -m brain     # run just brain's tests
lx test -m tests     # run just the lx suite tests
lx test -m workgen   # run just workgen's tests
lx test --list       # list all members and their test counts
```

Without a workspace `lx.toml`, `lx test` behaves as today (runs tests in cwd).

With a workspace, `lx test` iterates members in order, runs each member's tests per its `[test]` config, reports per-member and aggregate results.

Output format:
```
brain        3 passed, 0 failed
tests       71 passed, 0 failed
workgen      5 passed, 1 failed
flows        0 passed, 0 failed (no tests)

TOTAL: 79 passed, 1 failed, 4 members
```

### `lx run` — workspace-aware execution

```
lx run brain         # resolves to brain/main.lx via manifest
lx run workgen       # resolves to workgen/run.lx via manifest
lx run brain/orchestrator.lx  # explicit path still works
```

When the argument matches a workspace member name, resolve to that member's `entry` field.

### `lx check` — workspace-aware type checking

```
lx check             # type-check all members
lx check -m brain    # type-check just brain
```

### `lx list` — workspace status

```
lx list
  brain      22 files  main.lx       3 tests
  tests      71 files  (test suite)  71 tests
  workgen     3 files  run.lx        1 test
  flows      43 files  (examples)    0 tests
```

### `lx init` — project scaffolding

```
lx init my-agent              # single package
lx init --workspace my-proj   # workspace with initial member
lx init --member agents       # add member to existing workspace
```

## Module Resolution Order

With workspace support, the import resolution becomes:

1. **Relative** (`use ./util`) — from current file's directory
2. **Workspace member** (`use brain/protocols`) — from named member's root
3. **Stdlib** (`use std/json`) — built-in

The workspace member check uses the `[workspace].members` list to map the first path segment to a member directory.

## Implementation

### Phase 1: Manifest parsing + test runner (MVP)

1. Add `toml` crate dependency to `lx-cli`
2. `lx.toml` parsing: `WorkspaceManifest` and `PackageManifest` structs
3. Walk up from cwd to find root `lx.toml` (same as Cargo)
4. `lx test` iterates members, runs each member's tests
5. `lx test -m name` filters to one member
6. `lx list` shows member summary

### Phase 2: Module resolver + run

7. Module resolver gains workspace step between relative and stdlib
8. `lx run member-name` resolves via manifest
9. `lx check` gains workspace iteration

### Phase 3: Dependencies + init

10. `[deps]` section with version strings
11. `lx init` scaffolding
12. `lx install` / `lx update` (future: external package registry)

## This Repo's Workspace

The lx repo itself becomes the first workspace:

```
lx.toml                    # root workspace manifest

tests/lx.toml              # lx language test suite
brain/lx.toml              # cognitive self-model
workgen/lx.toml            # work-item generator
flows/lx.toml              # agentic workflow examples
```

Root manifest:
```toml
[workspace]
members = ["tests", "brain", "workgen", "flows"]
```

tests/ manifest:
```toml
[package]
name = "tests"
version = "0.1.0"
description = "lx language test suite"

[test]
dir = "."
pattern = "*.lx"
runner = "suite"
```

brain/ manifest:
```toml
[package]
name = "brain"
version = "0.1.0"
entry = "main.lx"
description = "Claude cognitive self-model"

[test]
dir = "tests/"
pattern = "test_*.lx"
runner = "describe"
```

workgen/ manifest:
```toml
[package]
name = "workgen"
version = "0.1.0"
entry = "run.lx"
description = "Work-item generation from audit checklists"

[test]
dir = "tests/"
pattern = "*.lx"
```

flows/ manifest:
```toml
[package]
name = "flows"
version = "0.1.0"
description = "Agentic workflow examples and libraries"

[test]
dir = "tests/"
pattern = "*_flow.lx"
```

## Cross-References

- Package manifest (single-package): `spec/package-manifest.md`
- Module system: `agent/LANGUAGE.md` (Modules section)
- Test runner: `spec/testing-satisfaction.md`
