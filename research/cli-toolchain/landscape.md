# Language Toolchain CLIs: Landscape Survey

Research date: 2026-03-20

## 1. Cargo (Rust)

### Subcommand Architecture

Cargo organizes its subcommands into five categories:

| Category | Commands |
|----------|----------|
| **Build** | `build`, `check`, `clean`, `doc`, `fetch`, `fix`, `run`, `rustc`, `rustdoc`, `test`, `bench` |
| **Manifest** | `add`, `generate-lockfile`, `locate-project`, `metadata`, `pkgid`, `remove`, `tree`, `update`, `vendor`, `verify-project` |
| **Package** | `init`, `install`, `new`, `search`, `uninstall` |
| **Publishing** | `login`, `logout`, `owner`, `package`, `publish`, `yank` |
| **General** | `help`, `version` |

Source: [The Cargo Book - Commands](https://doc.rust-lang.org/cargo/commands/)

### Plugin System

Cargo discovers plugins via PATH convention: any binary named `cargo-<name>` becomes invokable as `cargo <name>`. No registration required. Users install plugins with `cargo install cargo-<name>`, then invoke as `cargo <name>`.

Notable plugins: `cargo-watch`, `cargo-nextest`, `cargo-hack`, `cargo-expand`, `cargo-audit`, `cargo-flamegraph`.

Source: [Extending Cargo with Custom Commands](https://doc.rust-lang.org/book/ch14-05-extending-cargo.html)

### Workspace Support

Workspaces unify multiple crates under one `Cargo.lock` and shared `target/` directory. Package selection uses:

- `-p <name>` / `--package <name>` -- target a specific member
- `--workspace` -- operate on all members
- `default-members` key in root `Cargo.toml` -- controls default when neither flag is given

When no flag is specified and the CWD is the workspace root, `default-members` is used. When CWD is inside a member, that member is used.

Source: [Workspaces - The Cargo Book](https://doc.rust-lang.org/cargo/reference/workspaces.html)

### Build Profiles

Four built-in profiles: `dev` (default for `cargo build`), `release` (default for `cargo install`, activated by `--release`), `test`, and `bench` (inherits from `release`).

Custom profiles are defined in `Cargo.toml` and invoked via `cargo build --profile <name>`. They must inherit from a built-in profile:

```toml
[profile.production]
inherits = "release"
lto = true
```

Configurable knobs: `opt-level`, `debug`, `split-debuginfo`, `strip`, `debug-assertions`, `overflow-checks`, `lto`, `panic`, `incremental`, `codegen-units`, `rpath`.

Source: [Profiles - The Cargo Book](https://doc.rust-lang.org/cargo/reference/profiles.html)

### Feature Flags

Features enable conditional compilation. Syntax: `cargo build --features "feat1,feat2"`, `--all-features`, `--no-default-features`. Workspace members addressed as `package-name/feature-name`. With `resolver = "2"`, features are only unified within the same dependency kind (dev vs normal vs build).

Source: [Features - The Cargo Book](https://doc.rust-lang.org/cargo/reference/features.html)

### Caching and Fingerprinting

Cargo uses a fingerprinting system stored in `target/{debug,release}/.fingerprint/`. Each compilation unit gets a directory containing a hash file. The fingerprint incorporates: source file contents, dependency tree, build flags, profile settings, compiler version, and environment variables. On match, compilation is skipped. Incremental compilation artifacts live in `target/debug/incremental/`.

Source: [Build Cache - The Cargo Book](https://doc.rust-lang.org/cargo/reference/build-cache.html), [cargo fingerprint module](https://doc.rust-lang.org/beta/nightly-rustc/cargo/core/compiler/fingerprint/index.html)

### cargo-xtask Pattern

An unofficial but common pattern for project-specific automation. An `xtask` crate lives in the workspace and is invoked via `cargo xtask <task>`. Unlike shell scripts, xtask commands are written in Rust, cross-platform, and share workspace dependencies. Cargo itself uses this pattern. Limitation: xtask cannot intercept stock `cargo build`; it must wrap it.

Alias in `.cargo/config.toml`: `[alias] xtask = "run --package xtask --"`

Source: [cargo-xtask](https://github.com/matklad/cargo-xtask)


## 2. Go Toolchain

### Subcommand Architecture

Go takes a minimalist "batteries included" approach. All tools ship in one `go` binary:

| Command | Purpose |
|---------|---------|
| `build` | Compile packages and dependencies |
| `run` | Compile and run a Go program |
| `test` | Run tests with coverage, benchmarks, race detection |
| `fmt` | Reformat sources (gofmt) |
| `vet` | Static analysis for likely mistakes |
| `get` | Add/update dependencies |
| `install` | Compile and install packages |
| `mod` | Module maintenance (init, tidy, vendor, download, graph, why, edit, verify) |
| `work` | Workspace management (init, use, sync, edit, vendor) |
| `generate` | Process `//go:generate` directives |
| `doc` | Show documentation |
| `clean` | Remove cached files |
| `list` | List packages or modules |
| `tool` | Run low-level tools (asm, compile, link, pprof, trace, etc.) |
| `env` | Print Go environment |
| `fix` | Apply fixes from static checkers |
| `bug` | File a bug report |
| `telemetry` | Manage telemetry |
| `version` | Print version |

Source: [go command reference](https://pkg.go.dev/cmd/go)

### Minimal Philosophy

Go intentionally avoids external tooling dependencies. `go fmt` eliminates style debates with a single enforced format. `go vet` provides static analysis without third-party linters. `go test` handles unit tests, benchmarks, coverage, and race detection with zero setup. The philosophy: if every project needs it, it belongs in the standard toolchain.

Source: [Go's Toolchain is a Superpower](https://elsyarifx.medium.com/gos-toolchain-is-a-superpower-an-ode-to-go-fmt-go-vet-and-go-test-1b1efd02edd5)

### go generate

Not part of the build. A tool for package authors to run code generators. Scans `.go` files for `//go:generate command args...` directives and executes them sequentially. Environment variables `GOFILE`, `GOLINE`, `GOPACKAGE`, `GOARCH`, `GOOS` are available to generators. The `-run` flag filters which directives to execute by regex.

Design principle: generated code is checked into the repo. Consumers never run `go generate`; they use `go build` normally.

Source: [go generate proposal](https://go.googlesource.com/proposal/+/refs/heads/master/design/go-generate.md), [Go blog: Generating code](https://go.dev/blog/generate)

### Common Build Flags

Shared across `build`, `test`, `run`, `install`, `list`, `clean`, `get`:

```
-C dir          Change directory before running
-a              Force rebuild
-n              Dry run (print commands)
-p n            Parallel compilation (default GOMAXPROCS)
-race           Enable race detector
-cover          Enable coverage
-v              Verbose (print package names)
-x              Print commands as executed
-tags tag,list  Build constraint tags
-ldflags        Linker flags
-gcflags        Compiler flags
-mod mode       Module mode (readonly, vendor, mod)
```

### Workspace Support (go work)

Multi-module workspaces via `go.work` file. Commands: `go work init`, `go work use <dir>`, `go work sync`, `go work edit`, `go work vendor`.


## 3. Mix (Elixir)

### Task System Architecture

Mix is Elixir's build tool. Every command is a "task" implementing the `Mix.Task` behaviour. Tasks are modules under the `Mix.Tasks.*` namespace. The module name determines the command name: `Mix.Tasks.Deps.Clean` becomes `mix deps.clean`.

Source: [Mix.Task docs](https://hexdocs.pm/mix/Mix.Task.html)

### Defining Custom Tasks

```elixir
defmodule Mix.Tasks.Echo do
  @shortdoc "Echoes arguments"
  @moduledoc "Printed when the user requests `mix help echo`"
  use Mix.Task

  @impl Mix.Task
  def run(args) do
    Mix.shell().info(Enum.join(args, " "))
  end
end
```

### Task Attributes

| Attribute | Purpose |
|-----------|---------|
| `@shortdoc` | One-line description for `mix help`. Omitting hides the task from listings |
| `@moduledoc` | Full help text shown by `mix help <task>` |
| `@recursive` | When `true`, task runs in each umbrella child app |
| `@requirements` | List of prerequisite tasks (e.g., `["app.config"]`) |
| `@preferred_cli_env` | Environment to use (e.g., `:test`) |

### Task Execution Model

Tasks run once by default; subsequent calls return `:noop`. To re-run: `Mix.Task.reenable/1` then `Mix.Task.run/2`, or use `Mix.Task.rerun/2`. This prevents duplicate work in dependency chains.

Discovery: `Mix.Task.load_all/0` scans all code paths. Tasks don't need to be in any specific file path--only the module name matters.

### Core Commands

`new`, `compile`, `test`, `deps.get`, `deps.compile`, `format`, `release`, `run`, `do` (run multiple tasks), `escript.build`, `hex.publish`, `phx.new` (Phoenix), `phx.server`, `phx.gen.*`.

### Umbrella Projects

Umbrella projects hold multiple OTP apps under `apps/`. All apps share configuration, build cache, lockfile, and `mix.exs`. Tasks marked `@recursive true` run in each child app. `mix test` in an umbrella runs all child tests. Individual apps targeted via `mix cmd --app <name> <command>`.

Source: [Umbrella Projects - Elixir School](https://elixirschool.com/en/lessons/advanced/umbrella_projects), [Elixir getting started](https://elixir-lang.org/getting-started/mix-otp/dependencies-and-umbrella-projects.html)


## 4. npm / yarn / pnpm

### Command Structure

All three share a common command surface:

| Operation | npm | yarn | pnpm |
|-----------|-----|------|------|
| Install all | `npm install` | `yarn` | `pnpm install` |
| Add dep | `npm install <pkg>` | `yarn add <pkg>` | `pnpm add <pkg>` |
| Remove dep | `npm uninstall <pkg>` | `yarn remove <pkg>` | `pnpm remove <pkg>` |
| Run script | `npm run <name>` | `yarn <name>` | `pnpm <name>` |
| Test | `npm test` | `yarn test` | `pnpm test` |
| Init | `npm init` | `yarn init` | `pnpm init` |
| Publish | `npm publish` | `yarn npm publish` | `pnpm publish` |
| Execute | `npx <pkg>` | `yarn dlx <pkg>` | `pnpm dlx <pkg>` |

### Scripts in package.json

Scripts are arbitrary shell commands defined in `package.json`:
```json
{
  "scripts": {
    "build": "tsc",
    "test": "jest",
    "lint": "eslint .",
    "start": "node dist/index.js"
  }
}
```

### Lifecycle Hooks

npm supports automatic pre/post hooks: `pretest` runs before `test`, `posttest` runs after. Key lifecycle events:

- `preinstall` -> `install` -> `postinstall` -- package installation
- `prepare` -- runs after install and before pack/publish (builds the package)
- `prepublishOnly` -- runs before publish only
- `prepack` -> `postpack` -- around tarball creation

Hooks execute in topological order: dependency hooks run before dependent hooks.

Source: [npm scripts docs](https://docs.npmjs.com/cli/v11/using-npm/scripts/)

Note: yarn Berry disables pre/post hooks by default (`enableScripts`). pnpm requires `enablePrePostScripts` configuration.

### npx / dlx Architecture

`npx` (npm) / `pnpm dlx` / `yarn dlx` download a package to a temporary cache and execute its binary without global installation. Useful for one-off tools like `create-react-app` or `eslint`.

### Workspace/Monorepo Support

**npm**: `workspaces` field in root `package.json`, `--workspace=<name>` flag, `--workspaces` for all.

**yarn**: Workspaces with `packages` field in root `package.json`. Yarn Berry adds Plug'n'Play (PnP): replaces `node_modules` with a `.pnp.cjs` lookup file.

**pnpm**: `pnpm-workspace.yaml` defines package patterns. Rich `--filter` (`-F`) syntax:
- `--filter <name>` -- exact package
- `--filter <name>...` -- package + all dependencies
- `--filter ...<name>` -- package + all dependents
- `--filter <pattern>` -- glob match (`@scope/*`)
- `--filter "[origin/main]"` -- packages changed since git ref

Commands run in parallel respecting the dependency graph.

Source: [pnpm filtering](https://pnpm.io/filtering), [pnpm workspaces](https://pnpm.io/workspaces)

### Dependency Architecture Differences

- **npm**: Flat `node_modules` with hoisting
- **yarn Berry (PnP)**: `.pnp.cjs` lookup file, no `node_modules`
- **pnpm**: Content-addressable store (`~/.pnpm-store/`), symlinked `node_modules`. Each version stored once globally. Saves disk space significantly.


## 5. Poetry / uv (Python)

### Poetry

#### Subcommands

| Category | Commands |
|----------|----------|
| **Project** | `new`, `init` |
| **Dependencies** | `add`, `remove`, `install`, `update`, `lock` |
| **Execution** | `run`, `shell` |
| **Build/Publish** | `build`, `publish` |
| **Info** | `show`, `search`, `check`, `version`, `env info`, `env list` |
| **Config** | `config`, `source add/remove/show` |
| **Cache** | `cache clear`, `cache list` |

`poetry run <cmd>` executes inside the managed virtualenv. `poetry shell` spawns a new shell with the venv activated.

Source: [Poetry CLI docs](https://python-poetry.org/docs/cli/)

#### Virtual Environment Management

Poetry auto-creates `.venv` on first `poetry install`. Environments are per-project. `poetry env use <python>` switches Python versions. Poetry reads `pyproject.toml` (PEP 621 style) for project metadata and dependencies.

### uv

#### Architecture

uv is a single Rust binary replacing pip, pip-tools, pipx, poetry, pyenv, virtualenv, and twine. Written by Astral (same team as ruff). 10-100x faster than pip for cached installs.

#### Subcommand Organization

| Category | Commands |
|----------|----------|
| **Project** | `init`, `add`, `remove`, `run`, `lock`, `sync`, `build`, `publish` |
| **Python** | `python install`, `python pin`, `python list`, `python find` |
| **Tools** | `tool run` (aliased as `uvx`), `tool install`, `tool list` |
| **pip compat** | `pip compile`, `pip sync`, `pip install`, `pip freeze` |
| **Environments** | `venv` |
| **Scripts** | `add --script`, `run <script.py>` |
| **Cache** | `cache clean`, `cache dir`, `cache prune` |

Source: [uv docs](https://docs.astral.sh/uv/)

#### Key Design Decisions

- **Auto-managed venvs**: `uv run` auto-creates/activates `.venv`, installs deps, and selects correct Python version
- **Universal lockfile**: `uv.lock` is cross-platform (unlike Poetry's platform-specific lock)
- **Global cache**: Content-addressable deduplication across projects
- **`.python-version` file**: Specifies Python version; `uv run` auto-installs it
- **Inline script metadata**: Dependencies declared inside `.py` files via PEP 723 `# /// script` blocks


## 6. Deno

### Integrated Toolchain

Deno bundles everything into one binary. No separate linter, formatter, or test runner needed.

| Command | Purpose |
|---------|---------|
| `run` | Execute scripts (local, URL, or stdin) |
| `test` | Run tests |
| `bench` | Run benchmarks |
| `fmt` | Format code (uses dprint engine) |
| `lint` | Lint code (built-in ESLint-compatible rules) |
| `check` | Type-check without running |
| `compile` | Create standalone executable |
| `doc` | Generate documentation |
| `task` | Run tasks from `deno.json` |
| `init` | Create new project |
| `install` / `uninstall` | Manage tools |
| `add` / `remove` | Manage dependencies |
| `publish` | Publish to JSR |
| `serve` | Run with `Deno.serve` |
| `eval` | Evaluate expression |
| `repl` | Interactive shell |
| `completions` | Generate shell completions |
| `info` | Show dependency tree |
| `types` | Print runtime type declarations |
| `lsp` | Language server protocol |
| `jupyter` | Jupyter kernel |

Source: [Deno CLI reference](https://docs.deno.com/runtime/getting_started/command_line_interface/)

### deno.json Configuration

Central config file replacing multiple tool configs:

```json
{
  "tasks": {
    "dev": "deno run --watch main.ts",
    "test": "deno test --allow-read"
  },
  "fmt": {
    "indentWidth": 2,
    "singleQuote": true
  },
  "lint": {
    "rules": { "exclude": ["no-unused-vars"] }
  },
  "imports": {
    "std/": "https://deno.land/std@0.220.0/"
  }
}
```

Source: [deno.json configuration](https://docs.deno.com/runtime/fundamentals/configuration/)

### Watch Mode

Built-in `--watch` flag on `run`, `test`, `fmt`, `lint`. Auto-restarts on file changes. Also supports `--watch-hmr` for hot module replacement.

### Permission System

Unique among CLIs: Deno requires explicit permissions (`--allow-read`, `--allow-net`, `--allow-env`, etc.). This extends the CLI design into a security model.


## 7. Zig

### CLI Design

Zig's CLI is minimal. The `zig` binary has relatively few top-level commands:

| Command | Purpose |
|---------|---------|
| `build` | Execute `build.zig` DAG |
| `build-exe` | Compile executable |
| `build-lib` | Compile library |
| `build-obj` | Compile object file |
| `test` | Run tests |
| `run` | Compile and run |
| `fmt` | Format source |
| `cc` | C compiler frontend |
| `c++` | C++ compiler frontend |
| `ar` | Archiver |
| `objcopy` | Object copy |
| `translate-c` | Translate C to Zig |
| `init` | Initialize project |
| `fetch` | Fetch dependencies |

### build.zig as Build System

The build system uses Zig itself as the configuration language. `build.zig` contains a `pub fn build(b: *std.Build)` function that constructs a directed acyclic graph (DAG) of build steps:

- Steps are independently and concurrently executed
- Output paths are never hardcoded (enables caching and composability)
- Configuration options via `b.option()` generate help menus automatically
- `zig build -l` lists all available steps
- Users add custom steps that appear as subcommands: `zig build run`, `zig build test`, `zig build my-step`

Source: [Zig Build System](https://ziglang.org/learn/build-system/)

### Design Principles

- Build configuration is Zig code, not a DSL -- full language power available
- Cross-compilation is a first-class feature (pass `--target` to build for any platform)
- Zig doubles as a C/C++ compiler (`zig cc`), reducing toolchain fragmentation
- Dependencies are Zig packages fetched and cached by the build system
- No external build tool needed (no cmake, make, ninja)


## 8. Just

### What It Is

A command runner (not a build system) written in Rust. Recipes are defined in a `justfile` and invoked via `just <recipe>`.

Source: [just manual](https://just.systems/man/en/), [GitHub](https://github.com/casey/just)

### Justfile Syntax

```justfile
# List available recipes
default:
    @just --list

# Build the project
build:
    cargo build --release

# Run tests with optional filter
test filter="":
    cargo test {{filter}}

# Deploy to environment
deploy env="staging":
    ./scripts/deploy.sh {{env}}

# Recipe in Python
gen-docs:
    #!/usr/bin/env python3
    import generate
    generate.docs()
```

### Key Features

| Feature | Description |
|---------|-------------|
| **Arguments** | Recipes accept positional args with optional defaults |
| **Variables** | `name := "value"`, referenced as `{{name}}` |
| **Env loading** | Loads `.env` files automatically |
| **Shebang recipes** | Write recipes in any language (Python, Ruby, Node, etc.) |
| **Conditional logic** | `if`/`else` expressions in variables |
| **OS functions** | `os()`, `arch()`, `os_family()` for cross-platform |
| **Recipe listing** | `just --list` shows all recipes with descriptions |
| **Subdirectory search** | `just` finds `justfile` by walking up the directory tree |
| **Shell config** | `set shell := ["bash", "-cu"]` to configure execution shell |
| **Dependencies** | `recipe: dep1 dep2` runs dependencies first |
| **Private recipes** | Prefix with `_` to hide from `--list` |

### How Just Differs from Make

- No `.PHONY` needed (recipes are commands, not file targets)
- Recipes run from the justfile's directory, not CWD
- Errors are reported with source context
- Unknown recipes and circular dependencies detected before execution
- Proper argument passing to recipes
- Cross-platform: works with sh, PowerShell, cmd.exe
- No implicit variables or pattern rules


## Cross-Ecosystem Comparison

### Subcommand Coverage Matrix

| Capability | Cargo | Go | Mix | npm | Poetry | uv | Deno | Zig | Just |
|-----------|-------|-----|-----|-----|--------|-----|------|-----|------|
| Build/compile | Y | Y | Y | scripts | N | N | Y | Y | N |
| Run | Y | Y | Y | scripts | Y | Y | Y | Y | Y |
| Test | Y | Y | Y | scripts | N | N | Y | Y | N |
| Format | ext | Y | Y | scripts | N | N | Y | Y | N |
| Lint | ext | Y | N | scripts | N | N | Y | N | N |
| Init/new | Y | Y | Y | Y | Y | Y | Y | Y | N |
| Dep mgmt | Y | Y | Y | Y | Y | Y | Y | Y | N |
| Publish | Y | N | Y | Y | Y | Y | Y | N | N |
| Doc gen | Y | Y | N | N | N | N | Y | N | N |
| Bench | Y | Y | N | N | N | N | Y | Y | N |
| Watch | ext | N | Y | ext | N | N | Y | N | N |
| REPL | N | N | iex | N | N | N | Y | N | N |
| Task runner | ext | N | Y | Y | N | N | Y | Y | Y |
| Completions | ext | N | N | Y | Y | Y | Y | N | Y |

Legend: Y = built-in, ext = via plugin/external tool, N = not available, scripts = via package.json scripts

### Design Philosophy Spectrum

**Maximalist (everything built in)**: Deno > Go > Cargo > Mix
**Minimalist (do one thing)**: Just > Zig > npm > Poetry
**Unified replacement**: uv (replaces 7+ Python tools) > Deno (replaces Node + eslint + prettier + jest)
**Plugin-extensible**: Cargo > npm > Mix > oclif-based CLIs

### Workspace / Monorepo Approaches

| Tool | Mechanism | Selection syntax |
|------|-----------|-----------------|
| Cargo | `[workspace]` in `Cargo.toml` | `-p <name>`, `--workspace` |
| Go | `go.work` file | `go work use <dir>` |
| Mix | Umbrella projects in `apps/` | `--app <name>`, `@recursive` attribute |
| npm | `workspaces` in `package.json` | `--workspace=<name>`, `--workspaces` |
| pnpm | `pnpm-workspace.yaml` | `--filter <pattern>`, `--filter ...<name>` |
| yarn | `workspaces` in `package.json` | `yarn workspace <name>` |
| uv | `[tool.uv.workspace]` in `pyproject.toml` | `--package <name>` |

Sources:
- [The Cargo Book](https://doc.rust-lang.org/cargo/)
- [Go command reference](https://pkg.go.dev/cmd/go)
- [Mix.Task docs](https://hexdocs.pm/mix/Mix.Task.html)
- [npm scripts](https://docs.npmjs.com/cli/v11/using-npm/scripts/)
- [Poetry CLI](https://python-poetry.org/docs/cli/)
- [uv docs](https://docs.astral.sh/uv/)
- [Deno CLI](https://docs.deno.com/runtime/getting_started/command_line_interface/)
- [Zig Build System](https://ziglang.org/learn/build-system/)
- [just manual](https://just.systems/man/en/)
- [clig.dev - CLI Guidelines](https://clig.dev/)
