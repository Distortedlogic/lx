# Package Manifest

Every lx project has a `lx.toml` manifest at its root. The manifest declares identity, dependencies, entry points, and runtime configuration. `lx init` creates it.

## Problem

lx has `use ./path` (relative imports) and `use std/...` (stdlib), but no concept of a project boundary. There's no way to:

- Name or version a package
- Declare dependencies on other lx packages
- Specify which file is the entry point
- Configure runtime backends
- Set minimum lx version requirements
- Distinguish "this directory is an lx project" from "these are loose .lx files"

Without a manifest, there's no package ecosystem — no publishing, no sharing, no versioning.

## `lx.toml`

```toml
[package]
name = "code-reviewer"
version = "0.1.0"
description = "Multi-agent code review orchestrator"
entry = "src/main.lx"
lx = ">=0.3.0"

[deps]
"github.com/org/lx-git-utils" = "0.2.0"
"github.com/org/lx-review-lib" = { version = "1.0.0", path = "../review-lib" }

[deps.dev]
"github.com/org/lx-test-helpers" = "0.1.0"

[backends]
ai = "claude-code"
shell = "process"
http = "reqwest"
emit = "stdout"
yield = "stdin-stdout"
log = "stderr"

[test]
threshold = 0.75
runs = 3

[profile.release]
sandbox = true
deny = ["shell", "fs-write"]
```

### `[package]` fields

| Field         | Type   | Required | Description                                    |
| ------------- | ------ | -------- | ---------------------------------------------- |
| `name`        | Str    | yes      | Package identifier (lowercase, hyphens allowed) |
| `version`     | Str    | yes      | Semver version                                 |
| `description` | Str    | no       | One-line description                           |
| `entry`       | Str    | no       | Main file (default: `src/main.lx`)             |
| `lx`          | Str    | no       | Minimum lx version (semver range)              |
| `authors`     | [Str]  | no       | Author list                                    |
| `license`     | Str    | no       | SPDX license identifier                        |

### `[deps]` and `[deps.dev]`

Dependencies are keyed by package URL. Values are either a version string or an inline table with `version` and optional `path` (for local development overrides).

`[deps.dev]` dependencies are only available during `lx test`. They are not resolved for `lx run` or `lx build`.

### `[backends]`

Override default `RuntimeCtx` backend implementations. Each key maps to a backend trait. Values are implementation names. This lets packages declare "I need a real AI backend" vs "I'm fine with the mock."

### `[test]`

Configuration for the satisfaction-based test runner (see `spec/testing-satisfaction.md`).

| Field       | Type  | Default | Description                        |
| ----------- | ----- | ------- | ---------------------------------- |
| `threshold` | Float | 0.75    | Global satisfaction pass threshold |
| `runs`      | Int   | 1       | Default number of runs per scenario |

### `[profile.*]`

Named profiles for different execution contexts. `[profile.release]` is used by `lx build`. Each profile can override sandbox settings, deny capabilities, and set environment variables.

## `lx init`

`lx init` creates a new project:

```
$ lx init my-flow
Created my-flow/
  lx.toml
  src/main.lx
  test/main_test.lx
```

`lx init --flow` creates a flow-oriented project:

```
$ lx init --flow review-pipeline
Created review-pipeline/
  lx.toml
  src/main.lx
  src/agents/
  test/main_test.lx
  test/scenarios/
```

## Resolution

### Import resolution order

1. Relative paths (`use ./util`) — resolved from current file
2. Package deps (`use review-lib/scoring`) — resolved from `[deps]` via package name
3. Stdlib (`use std/json`) — built-in

### Dependency resolution

Dependencies are fetched to a global cache (`~/.lx/cache/`) and symlinked into the project's `.lx/deps/` directory. Version resolution uses semver compatibility.

```
~/.lx/cache/
  github.com/org/lx-git-utils/0.2.0/
  github.com/org/lx-review-lib/1.0.0/
```

### Lock file

`lx.lock` records exact resolved versions. Committed to version control. `lx install` resolves and locks. `lx update` refreshes within semver constraints.

## Patterns

### Minimal flow package

```toml
[package]
name = "hello"
version = "0.1.0"
entry = "main.lx"
```

### Multi-agent orchestration package

```toml
[package]
name = "review-pipeline"
version = "1.0.0"
description = "Parallel code review with reconciliation"
entry = "src/main.lx"
lx = ">=0.4.0"

[deps]
"github.com/lx-lang/git" = "0.2.0"

[backends]
ai = "claude-code"

[test]
threshold = 0.80
runs = 5
```

## Implementation

### CLI changes

- `lx init [name]` — scaffold project with `lx.toml`
- `lx install` — resolve and lock dependencies
- `lx update` — update dependencies within semver constraints
- All `lx` subcommands (`run`, `test`, `check`) walk up from cwd to find `lx.toml`

### Module resolver changes

The interpreter's module resolver (`interpreter/modules.rs`) gains a new resolution step between relative and stdlib: check `[deps]` in the nearest `lx.toml` and resolve package imports from `.lx/deps/`.

### Dependencies

- `toml` crate (parsing `lx.toml`)
- `semver` crate (version resolution)

## Cross-References

- Module system: FEATURES.md (Modules section)
- Test runner: [testing-satisfaction.md](testing-satisfaction.md)
- Toolchain: [toolchain.md](toolchain.md) (`lx init`, `lx test`)
- Sandboxing: [toolchain.md](toolchain.md) (Sandboxing section)
