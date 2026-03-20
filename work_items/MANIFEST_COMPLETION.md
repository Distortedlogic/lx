# Goal

Complete the lx package manifest system to match the spec in `spec/package-manifest.md`. The foundation exists — TOML parsing, workspace loading, git/path deps, lockfile, `lx install`/`lx update` — but several spec'd features are missing: `lx init` scaffolding, `[backends]` configuration, `[test]` threshold/runs fields, `[deps.dev]` dev-only dependencies, `version` as required field, and `[package]` metadata fields (authors, license, lx version constraint). 10 concrete tasks.

# Why

- `lx init` doesn't exist — every new project requires manually writing `lx.toml`, creating `src/main.lx`, and creating test directories. This is friction on every project start
- `PackageSection.version` is `Option<String>` but the spec requires it — a versionless package can be installed/published with no identity
- `[test]` section parses `dir`/`pattern` but spec defines `threshold`/`runs` — satisfaction tests can't read their config from the manifest
- `[backends]` section isn't parsed at all — packages can't declare which RuntimeCtx backends they need, so every deployment requires manual configuration
- `[deps.dev]` doesn't exist — test-only dependencies are installed for production runs, bloating the dependency tree
- `authors`/`license`/`lx` constraint fields are missing — no package metadata for a future registry
- Lockfile has no version pinning — `LockedPackage` stores `name`+`source` but not the resolved version, so `lx update` can't compare current vs available

# What changes

**CLI scaffolding (Tasks 1-2):** Add `lx init [name]` command that creates a project directory with `lx.toml`, `src/main.lx`, and `test/main_test.lx`. Add `--flow` flag that also creates `src/agents/` and `test/scenarios/`.

**Manifest schema completion (Tasks 3-6):** Make `version` required (error if missing). Add `authors`, `license`, `lx` fields to `PackageSection`. Parse `[backends]` section into `BackendsSection`. Parse `[test]` with `threshold`/`runs` fields (alongside existing `dir`/`pattern`). Parse `[deps.dev]` as separate dependency map.

**Lockfile version tracking (Task 7):** Add `version` field to `LockedPackage`. Record the resolved version (git commit, tag, or path) so `lx update` can detect stale locks.

**Dev dependency filtering (Task 8):** Wire `[deps.dev]` so `lx install` resolves them to `.lx/deps/` but `lx run` skips adding them to the module resolver. `lx test` includes them.

**Backend configuration (Task 9):** Wire `[backends]` so `RuntimeCtx` construction reads manifest backend preferences. Each backend key maps to a known implementation name.

**Test config propagation (Task 10):** Wire `[test].threshold`/`[test].runs` so `std/test` can read them from the manifest. Expose via a builtin or environment variable.

# Files affected

- `crates/lx-cli/src/main.rs` — Add `Init` command variant with `name` and `--flow` args
- `crates/lx-cli/src/manifest.rs` — Add `BackendsSection`, `authors`/`license`/`lx` to `PackageSection`, `threshold`/`runs` to `TestSection`, `dev_dependencies` to `RootManifest`
- `crates/lx-cli/src/lockfile.rs` — Add `version` field to `LockedPackage`
- `crates/lx-cli/src/install.rs` — Pass version to `lock.upsert`
- `crates/lx-cli/src/run.rs` — Filter dev deps from module resolver
- `crates/lx/src/backends/mod.rs` — Accept backend preferences in `RuntimeCtx` construction
- New file: `crates/lx-cli/src/init.rs` — Project scaffolding logic

# Task List

## Task 1: Add `lx init` command to CLI

**Subject:** Add lx init subcommand for project scaffolding
**ActiveForm:** Adding lx init command

Add an `Init` variant to the `Command` enum in `crates/lx-cli/src/main.rs` with `name: Option<String>` and `flow: bool` fields. Add a match arm that calls `init::run_init(name, flow)`. Create `crates/lx-cli/src/init.rs` with `run_init` that: creates the project directory (or uses cwd if no name given), writes `lx.toml` with `[package]` (name, version "0.1.0", entry "src/main.lx"), creates `src/main.lx` with a minimal hello-world program, creates `test/` directory. Register `mod init;` in `main.rs`.

Verify: `just diagnose` passes.

## Task 2: Add `--flow` flag to `lx init`

**Subject:** Flow-oriented project scaffold
**ActiveForm:** Adding --flow scaffold variant

In `crates/lx-cli/src/init.rs`, when `flow` is true, additionally: create `src/agents/` directory, create `test/scenarios/` directory, add `[test]` section with `threshold = 0.75` and `runs = 1` to the generated `lx.toml`. The base lx.toml already has `[package]` from Task 1.

Verify: `just diagnose` passes.

## Task 3: Add missing `[package]` metadata fields

**Subject:** Complete PackageSection with authors, license, lx
**ActiveForm:** Adding package metadata fields

In `crates/lx-cli/src/manifest.rs`, add to `PackageSection`: `authors: Option<Vec<String>>`, `license: Option<String>`, `lx: Option<String>`. These are parse-only for now — no validation of semver ranges or SPDX identifiers. Leave `version` as `Option<String>` (Task 4 handles validation).

Verify: `just diagnose` passes. Existing `lx.toml` files without these fields still parse.

## Task 4: Make version required with validation

**Subject:** Validate version field presence in package manifests
**ActiveForm:** Adding version validation

In `crates/lx-cli/src/manifest.rs`, add a `validate_manifest` function that checks: if `[package]` exists, `version` must be present and non-empty. Call it from `load_manifest` after parsing. The workspace root manifest (which may have only `[workspace]`) is exempt. Member manifests that have `[package]` require `version`.

Verify: `just diagnose` passes. All existing member `lx.toml` files have version fields.

## Task 5: Parse `[backends]` section

**Subject:** Add backends section to manifest parser
**ActiveForm:** Parsing backends section

In `crates/lx-cli/src/manifest.rs`, add `BackendsSection` struct with optional fields: `ai`, `shell`, `http`, `emit`, `yield_backend` (serde rename "yield"), `log`, `user` — all `Option<String>`. Add `backends: Option<BackendsSection>` to `RootManifest`. Parse-only for now — wiring to RuntimeCtx is Task 9.

Verify: `just diagnose` passes. A `lx.toml` with `[backends]` section parses without error.

## Task 6: Parse `[test]` threshold/runs and `[deps.dev]`

**Subject:** Complete test config and dev dependency parsing
**ActiveForm:** Parsing test threshold/runs and deps.dev

In `crates/lx-cli/src/manifest.rs`: add `threshold: Option<f64>` and `runs: Option<u32>` to `TestSection`. Add `dev_dependencies: Option<HashMap<String, DepSpec>>` to `RootManifest` with `#[serde(alias = "deps.dev")]` (TOML nested table `[deps.dev]`). Parse-only — wiring is in later tasks.

Verify: `just diagnose` passes.

## Task 7: Add version tracking to lockfile

**Subject:** Track resolved version in lockfile entries
**ActiveForm:** Adding version to LockedPackage

In `crates/lx-cli/src/lockfile.rs`, add `version: Option<String>` to `LockedPackage`. Update `LockFile::upsert` to accept an optional version parameter. In `crates/lx-cli/src/install.rs`, pass the resolved version (git tag/branch/rev or "path") when calling `lock.upsert`. Existing lockfiles without `version` still deserialize (it's Option).

Verify: `just diagnose` passes.

## Task 8: Filter dev dependencies from `lx run`

**Subject:** Dev deps only available during lx test
**ActiveForm:** Filtering dev dependencies

In `crates/lx-cli/src/install.rs`: `run_install` should also install `dev_dependencies` alongside `dependencies`. In the module resolver setup (where `try_load_dep_dirs` or `try_load_workspace_members` is called for `lx run`), add a flag or separate function that excludes dev-only deps. The simplest approach: `lx install` writes a `.lx/deps/.dev-deps` marker file listing dev dep names. `try_load_dep_dirs` accepts a `bool include_dev` param and filters accordingly.

Verify: `just diagnose` passes.

## Task 9: Wire `[backends]` to RuntimeCtx

**Subject:** Apply manifest backend preferences to runtime
**ActiveForm:** Wiring backends to RuntimeCtx

In the CLI entry point (where `RuntimeCtx` is constructed), read `manifest.backends` if present. For each specified backend, select the matching implementation. Initially support: `ai` = "claude-code" (default), `emit` = "stdout" (default) or "noop", `log` = "stderr" (default) or "noop", `user` = "stdin-stdout" or "noop" (default). Unknown backend names produce a warning. This is a light touch — just enough to let packages declare their needs.

Verify: `just diagnose` passes.

## Task 10: Propagate `[test]` config to satisfaction runner

**Subject:** Make test threshold/runs available to std/test
**ActiveForm:** Propagating test config

When the CLI runs tests, read `[test].threshold` and `[test].runs` from the manifest. Inject them as environment variables (`LX_TEST_THRESHOLD`, `LX_TEST_RUNS`) before executing test files, so `std/test` can read them via `std/env`. Alternatively, inject them into `RuntimeCtx` as a test config field. Whichever is simpler — env vars require no Rust struct changes.

Verify: `just diagnose` passes.

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.
