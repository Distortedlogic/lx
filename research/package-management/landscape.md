# Package Management Across Programming Languages

Research landscape for lx workspace, dependency, and registry design.

## 1. Cargo (Rust)

### Cargo.toml Manifest

```toml
[package]
name = "my-crate"
version = "0.1.0"
edition = "2021"
rust-version = "1.70"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
log = "0.4"

[dev-dependencies]
tempfile = "3.0"

[build-dependencies]
cc = "1.0"

[features]
default = ["std"]
std = []
avif = ["dep:ravif", "dep:rgb"]
```

**Dependency kinds**: `[dependencies]` (normal), `[dev-dependencies]` (tests/examples/benches), `[build-dependencies]` (build scripts).

**Dependency sources**: crates.io (default), git (`git = "url"`, with `branch`/`tag`/`rev`), path (`path = "../local"`), registry (alternative registries).

Source: [Cargo: Specifying Dependencies](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html)

### Features / Conditional Compilation

Features are named flags that enable optional code and dependencies:

```toml
[features]
default = ["std"]
std = []
serde = ["dep:serde", "rgb?/serde"]  # ? = only if rgb is enabled
```

**Feature unification** -- When multiple packages depend on a crate, Cargo builds it with the *union* of all enabled features. Features must be additive (enabling a feature must not disable functionality). The resolver v2 (edition 2021+) avoids unifying features across build/dev/platform boundaries.

**`dep:` prefix** (Rust 1.60+) -- Prevents optional dependencies from implicitly creating features. `"avif" = ["dep:ravif"]` means the `avif` feature enables the `ravif` dependency without exposing `ravif` as its own feature.

**`?` syntax** (Rust 1.60+) -- `"rgb?/serde"` enables the `serde` feature on `rgb` only if `rgb` is already enabled by something else.

Source: [Cargo Features](https://doc.rust-lang.org/cargo/reference/features.html)

### Dependency Resolution

Cargo uses a **backtracking resolver** (historically) with work underway to adopt PubGrub.

Key behaviors:
- **Highest version preference**: Selects the highest version matching semver constraints.
- **Semver unification**: `"1.0"` means `>=1.0.0, <2.0.0`. Compatible versions are unified into a single build. Incompatible versions (e.g., `0.6` and `0.7`) result in duplicate builds.
- **Lockfile priority**: `Cargo.lock` entries take highest precedence for reproducibility.
- **Yanked versions**: Ignored unless already in `Cargo.lock` or explicitly requested with `--precise`.
- **`links` field**: Ensures only one copy of a native library (e.g., `links = "git2"` prevents two versions of `libgit2-sys`).

**Resolver versions**:
- v1 (editions < 2021): Global feature unification.
- v2 (edition 2021): Platform-specific, build, proc-macro, and dev-dependency features separated.
- v3 (edition 2024): Changes default for MSRV-incompatible versions to `fallback`.

Source: [Cargo: Dependency Resolution](https://doc.rust-lang.org/cargo/reference/resolver.html)

### Cargo.lock

TOML format. Records exact version, source, and checksum for every resolved dependency. Committed for binaries and applications; `.gitignore`d for libraries (to allow consumers to resolve their own versions).

By default, `cargo build` uses the lockfile but does not enforce it strictly. `cargo build --locked` fails if the lockfile would change. `cargo build --frozen` additionally prevents network access.

### Workspaces

```toml
[workspace]
members = ["crates/*"]
resolver = "2"

[workspace.package]
edition = "2021"
license = "MIT"

[workspace.dependencies]
serde = { version = "1.0", features = ["derive"] }
```

Workspace members share a single `Cargo.lock` and `target/` directory. `[workspace.dependencies]` centralizes version declarations; members reference them with `workspace = true`. Lockfile generation resolves as if all features of all members are enabled.

### crates.io Registry

- Sparse index (default since Rust 1.68): HTTP-based, fetches only needed index entries.
- Old git index: Full clone of the crates.io-index repository.
- API: REST endpoints for search, download, publish. Publishes are append-only (versions cannot be deleted, only yanked).
- Rate limits and size limits enforced.
- Build scripts (`build.rs`) can run arbitrary code at compile time -- a known supply chain risk.

---

## 2. npm / Yarn / pnpm (JavaScript)

### package.json Manifest

```json
{
  "name": "@scope/my-package",
  "version": "1.0.0",
  "type": "module",
  "main": "./dist/index.cjs",
  "module": "./dist/index.mjs",
  "exports": {
    ".": {
      "import": "./dist/index.mjs",
      "require": "./dist/index.cjs"
    }
  },
  "dependencies": { "lodash": "^4.17.21" },
  "devDependencies": { "jest": "^29.0.0" },
  "peerDependencies": { "react": ">=17.0.0" },
  "optionalDependencies": { "fsevents": "^2.3.0" }
}
```

**Dependency kinds**: `dependencies`, `devDependencies`, `peerDependencies` (expected to be provided by the consumer), `optionalDependencies` (install failure is non-fatal).

### node_modules and Hoisting

**npm/Yarn** use a **flat `node_modules`** structure with hoisting:
- Dependencies are lifted to the top-level `node_modules/` when possible.
- Conflicts (two packages need different versions of the same dependency) result in nested `node_modules/` within the conflicting package.
- **Phantom dependencies**: Packages can accidentally import dependencies they did not declare, because hoisting makes undeclared transitive dependencies accessible.

**pnpm** uses **strict isolation**:
- Content-addressable store on disk (shared globally via hard links).
- Each package's `node_modules/` contains only its declared dependencies (symlinks to the store).
- Undeclared imports throw errors. No phantom dependencies.
- 2-3x faster installs than npm, ~50% less disk usage.

Source: [pnpm docs](https://pnpm.io/settings), [npm registry docs](https://docs.npmjs.com/cli/v11/using-npm/registry/)

### Lockfile Formats

| Manager | File | Format | Contents |
|---------|------|--------|----------|
| npm | `package-lock.json` | JSON | Full dependency tree with resolved URLs, integrity hashes, physical layout |
| Yarn Classic | `yarn.lock` | Custom | Resolutions only (version + integrity hash per dependency) |
| Yarn Berry | `yarn.lock` | YAML | Resolutions + checksums |
| pnpm | `pnpm-lock.yaml` | YAML | Resolutions, no hoisting constraints |

npm's lockfile encodes the *physical* `node_modules` layout. pnpm and Yarn store only logical resolutions. npm lockfiles are notoriously large and hard to review; pnpm lockfiles are more readable.

### Workspaces

All three support workspaces via `package.json`:

```json
{
  "workspaces": ["packages/*"]
}
```

Workspace packages can depend on each other using the `"workspace:*"` protocol (pnpm/Yarn) or standard version ranges (npm). Hoisting behavior varies: pnpm uses symlinks, npm/Yarn hoist to root `node_modules/`.

### Peer Dependencies

Peer dependencies declare that a package expects the consumer to provide a dependency. Since npm v7, unmet peer dependencies trigger warnings (v3-6 auto-installed, v7+ warns but installs). pnpm is strict about peer dependency resolution.

---

## 3. pip / Poetry / uv (Python)

### Requirements Formats

**requirements.txt** -- Flat list of pinned or constrained versions:
```
requests>=2.28.0,<3.0
numpy==1.24.3
flask~=2.3.0
```

**pyproject.toml** (PEP 517/518/621) -- Modern standard:
```toml
[project]
name = "my-package"
version = "1.0.0"
requires-python = ">=3.9"
dependencies = [
    "requests>=2.28",
    "numpy>=1.24",
]

[project.optional-dependencies]
dev = ["pytest>=7.0", "black"]

[build-system]
requires = ["setuptools>=68.0"]
build-backend = "setuptools.build_meta"
```

**PEP 517**: Build system abstraction (any build backend, not just setuptools).
**PEP 518**: `[build-system]` table declaring build dependencies.
**PEP 621**: Standardized `[project]` metadata table.

### Dependency Resolution

**pip** -- Historically used a greedy installer (install first valid version, fail on conflict). Since pip 20.3, uses a backtracking resolver. Can be slow on complex graphs because it downloads packages to inspect metadata.

**Poetry** -- Uses a more sophisticated resolver (originally custom, now moving toward PubGrub). Manages virtual environments, lock files (`poetry.lock`), and publishing. Cross-platform lockfile.

**uv** -- Written in Rust by Astral (creators of Ruff). Uses PubGrub for resolution. 10-100x faster than pip. Features:
- **Forking resolver**: When platform-specific dependencies conflict, forks resolution into parallel branches.
- **Universal lockfile**: Single `uv.lock` covering all platforms and Python versions.
- **`pyproject.toml` native**: Uses PEP 621 format.
- Replaces pip, pip-tools, virtualenv, and pyenv in a single tool.

### Virtual Environments

Python's isolation mechanism. Each project gets its own `site-packages/`. Tools:
- `venv` (stdlib)
- `virtualenv` (third-party, faster)
- `uv venv` (fastest)
- `conda` (also manages non-Python dependencies)

---

## 4. Go Modules

### go.mod Format

```go
module example.com/myproject

go 1.23.0

require (
    golang.org/x/text v0.3.0
    golang.org/x/crypto v1.4.5 // indirect
)

replace golang.org/x/net => ./fork/net

exclude golang.org/x/old v1.2.3

retract [v1.9.0, v1.9.5]  // accidental release
```

**Directives**:
- `module`: Canonical module path (exactly one).
- `go`: Minimum Go version (mandatory since Go 1.21). Controls language features, automatic vendoring (1.14+), module graph pruning (1.17+).
- `require`: Module dependencies with minimum versions. `// indirect` marks transitive dependencies.
- `replace`: Redirects a module to an alternative source (local path or different module). Only effective in the main module's `go.mod`.
- `exclude`: Prevents specific versions from being used. Only effective in main module.
- `retract`: Marks versions as erroneous. Versions remain downloadable but hidden from `@latest` queries.

Source: [Go Modules Reference](https://go.dev/ref/mod)

### Minimum Version Selection (MVS)

Go's resolution algorithm is fundamentally different from other package managers. Instead of finding the *newest* compatible version, MVS selects the *minimum* version satisfying all constraints.

**Algorithm**:
1. Start at the main module.
2. Traverse the dependency graph, tracking the highest *required* version of each module.
3. The build list is exactly those highest-required versions.

**Example**: If A requires D >= 1.5 and B requires D >= 1.6, MVS selects D 1.6 (not the latest D 2.0).

**Key properties**:
- Deterministic without a lockfile (same inputs always produce same outputs).
- No NP-completeness -- polynomial-time algorithm.
- "Known good" versions rather than "latest compatible" versions.
- No backtracking needed.

The tradeoff: relies on the assumption of monotonic compatibility (if D 1.6 works with B, then D 1.7 should too). When this assumption breaks, the buggy version should be retracted.

Source: [Russ Cox: Version SAT](https://research.swtch.com/version-sat), [Ardan Labs: MVS](https://www.ardanlabs.com/blog/2019/12/modules-03-minimal-version-selection.html)

### go.sum and Checksum Database

`go.sum` records SHA-256 hashes for each dependency:
```
example.com/module v1.0.0 h1:abcdef...
example.com/module v1.0.0/go.mod h1:xyz...
```

Two entries per version: one for the module zip, one for `go.mod` alone. Verified on every download.

**Checksum database** (`sum.golang.org`): Public append-only log of module checksums. Since Go 1.13, `go` commands verify downloaded modules against this database by default. Prevents supply chain attacks where a module author pushes different content for the same version. `GONOSUMDB` and `GONOSUMCHECK` control opt-out.

### Module Proxy Protocol

Default proxy: `proxy.golang.org`. Environment variable `GOPROXY` controls the proxy chain (comma-separated, `direct` for VCS fallback).

**Endpoints**:
```
GET $GOPROXY/<module>/@v/list           -- list versions
GET $GOPROXY/<module>/@v/<version>.info -- version metadata (JSON)
GET $GOPROXY/<module>/@v/<version>.mod  -- go.mod file
GET $GOPROXY/<module>/@v/<version>.zip  -- module source archive
GET $GOPROXY/<module>/@latest           -- latest version (JSON)
```

Proxy benefits: Reduces load on Git hosts, ensures availability even if original repo is deleted, enables corporate caching.

### Vendoring

`go mod vendor` creates a `vendor/` directory with all dependency sources. `vendor/modules.txt` manifest tracks module versions. Since Go 1.14, vendoring is automatic if `vendor/` exists. Build commands use vendored sources instead of the module cache.

---

## 5. Mix (Elixir)

### mix.exs Manifest

```elixir
defmodule MyApp.MixProject do
  use Mix.Project

  def project do
    [
      app: :my_app,
      version: "0.1.0",
      elixir: "~> 1.14",
      deps: deps()
    ]
  end

  defp deps do
    [
      {:phoenix, "~> 1.7"},
      {:ecto, "~> 3.10"},
      {:my_lib, path: "../my_lib"},
      {:internal, in_umbrella: true}
    ]
  end
end
```

**Dependency sources**: Hex.pm (default), git, path, in_umbrella.

### Hex.pm Registry

Elixir/Erlang package registry. Features:
- Immutable package versions (no mutation after publish).
- Uses PubGrub for dependency resolution (adopted from Dart).
- Package signing with repository keys.
- Organization accounts for private packages.

### Umbrella Projects

Monorepo structure where multiple OTP applications share a repository:

```
my_umbrella/
  apps/
    app_a/mix.exs
    app_b/mix.exs
  mix.exs           # umbrella root
```

All configuration and dependencies are shared. Apps declare inter-dependencies with `{:app_a, in_umbrella: true}`. Dependencies must be explicit -- no implicit coupling. Apps can be compiled, tested, and deployed independently.

### Lockfile

`mix.lock` generated automatically on dependency resolution. Must be committed to version control. `mix deps.get` respects the lockfile. Format is Elixir terms (readable map of package names to version/hash/source tuples).

---

## Cross-Cutting Analysis

### Dependency Resolution Algorithms

| Algorithm | Used by | Behavior | Error Messages |
|-----------|---------|----------|---------------|
| **SAT solving** | Composer, DNF, Conda, Zypper, opam | Encodes constraints as boolean satisfiability. Complete (finds solution if one exists). Can prove unsatisfiability. | Typically opaque (raw conflict clauses). |
| **PubGrub** | Poetry, uv, Swift PM, Hex, Bundler, (Cargo migrating) | Domain-specific CDCL. Maintains incompatibilities, uses unit propagation and conflict-driven learning. | Excellent -- human-readable explanations of why resolution failed. |
| **Backtracking** | pip, Cargo (current), Cabal | Depth-first search with rollback. Tries versions in preference order (usually newest first). | Variable quality. Can be slow on pathological inputs. |
| **MVS** | Go, vcpkg | Selects minimum version satisfying all constraints. Polynomial time. Deterministic without lockfile. | Clear (constraint chain is obvious). |
| **Version mediation** | Maven, Gradle, NuGet, sbt | No solver; picks winner by convention (nearest-definition, highest-version). Fast but can silently break. | Minimal. |
| **Content-addressed** | Nix, Guix | Packages identified by input hash. No version selection at all. | N/A. |
| **ASP** | Spack | Logic programming rules handle compiler versions, build variants, microarchitecture alongside versions. | Domain-specific. |

Source: [Dependency Resolution Methods (Nesbitt, 2026)](https://nesbitt.io/2026/02/06/dependency-resolution-methods.html), [PubGrub Guide](https://pubgrub-rs-guide.pages.dev/testing/sat)

**PubGrub internals**:
- Maintains a set of *incompatibilities* -- facts about which version combinations cannot coexist.
- Uses *unit propagation* to narrow the search when only one possibility remains.
- On conflict, performs *conflict-driven learning*: derives a new incompatibility from the conflict, which prunes the search space.
- The derived incompatibility chain is the human-readable error message.

**Why version selection is NP-complete** (Russ Cox):
Four minimal assumptions make it NP-hard:
1. Dependencies can specify exact versions.
2. All dependencies must be installed.
3. Different versions have different dependencies.
4. Conflicting versions cannot coexist.

Go's MVS escapes NP-completeness by relaxing assumption 1 (only minimum versions, no upper bounds) and relying on *monotonicity* (if X works with dep v1.5, it works with v1.6+).

Source: [Russ Cox: Version SAT](https://research.swtch.com/version-sat)

### Lockfile Design

Comparative study from [The Design Space of Lockfiles (arXiv:2505.04834)](https://arxiv.org/html/2505.04834v1):

| Manager | Format | Transitive deps | Integrity hash | Source link | Default enforcement |
|---------|--------|-----------------|----------------|-------------|-------------------|
| npm | JSON | Tree structure | SHA-512 | `resolved` URL | Yes (on install) |
| pnpm | YAML | Tree structure | SHA-512 | No | Yes |
| Cargo | TOML | Tree structure | SHA-256 | `source` field | No (`--locked` required) |
| Poetry | TOML | Tree structure | SHA-256 | No | Yes |
| Go | Text (go.mod + go.sum) | Flat with `// indirect` | SHA-256 (go.sum) | Inferred from module path | Yes |
| Pipenv | JSON | Flat | SHA-256 | No | No (`--deploy` required) |
| Gradle | Text | Flat | None | No | No (`--write-locks` to generate) |

**Adoption rates**: Go 99.7%, Poetry 83.8%, Cargo 70.9%, npm ~53%, Gradle 0.9%.

**Key design lessons**:
- Format stability matters more than format elegance. Breaking changes to lockfile format impose costs on every tool in the ecosystem.
- Human readability correlates with adoption. Go's go.mod fits on a screen; npm's package-lock.json is "quite wild."
- Default enforcement drives adoption. Go and Poetry respect lockfiles by default; Gradle requires explicit flags and has near-zero adoption.
- `npm ci` (clean install from lockfile) vs `npm install` (updates lockfile) is a critical distinction for CI reproducibility.

Source: [Lockfile Format Design (Nesbitt, 2026)](https://nesbitt.io/2026/01/17/lockfile-format-design-and-tradeoffs.html)

### Workspace / Monorepo Support

| Ecosystem | Tool | Mechanism |
|-----------|------|-----------|
| Rust | Cargo workspaces | Single `Cargo.lock`, shared `target/`. `[workspace.dependencies]` for centralized versions. Members use `workspace = true`. |
| JavaScript | npm/Yarn/pnpm workspaces | `"workspaces"` field in root `package.json`. Hoisting (npm/Yarn) or symlinking (pnpm). |
| JavaScript | Turborepo | Build orchestration layer. Pipeline-based task scheduling with caching. Skips tasks whose inputs haven't changed. |
| JavaScript | Nx | Full monorepo framework. Affected detection (only rebuild/test what changed). Generators for scaffolding. Supports multiple languages. |
| JavaScript | Lerna | Original JS monorepo tool (2016). Bootstrapping, versioning (fixed or independent), publishing. Now maintained by Nx team. |
| Python | uv workspaces | `[tool.uv.workspace]` in `pyproject.toml`. Shared `uv.lock`. |
| Elixir | Umbrella projects | `apps/` directory structure. Shared config and dependencies. `in_umbrella: true` for inter-app deps. |
| Go | Multi-module workspaces | `go.work` file (Go 1.18+) lists module directories. Single build graph across modules. |
| Haskell | Cabal projects / Stack | `cabal.project` file lists packages. Stack uses `stack.yaml` with package directories. |

### Registry Design Comparison

| Registry | Language | Key Design Choices |
|----------|----------|-------------------|
| **crates.io** | Rust | Sparse HTTP index (since 1.68). Append-only (no version deletion, only yanking). Git-based index for backward compat. Build scripts are a supply chain risk. |
| **npm** | JavaScript | Largest registry (3M+ packages). Scoped packages (`@org/pkg`). `npm audit` for vulnerability scanning. Has suffered major supply chain attacks (ua-parser-js, colors, event-stream). Supports provenance attestations. |
| **PyPI** | Python | Simple repository API (PEP 503). JSON API for package metadata. Supports Trusted Publishers (OIDC-based publishing from CI). No native search API. |
| **Hex.pm** | Elixir/Erlang | Immutable versions. Repository signing. PubGrub resolver. Organization accounts. Relatively small ecosystem (~15K packages). |
| **Hackage** | Haskell | Package candidates for pre-release testing. Documentation auto-built and hosted. Revision system allows metadata-only updates post-publish. |

**Common patterns**:
- Immutability: Once published, a version's contents should never change. Violations (npm left-pad incident, 2016) led to stricter policies.
- Semver: All major registries use semantic versioning, though enforcement varies.
- Signing/provenance: npm Sigstore integration, PyPI Trusted Publishers, Hex repository keys.

### Vendoring and Offline Builds

| Tool | Command | Mechanism |
|------|---------|-----------|
| Go | `go mod vendor` | Copies sources to `vendor/`. `vendor/modules.txt` manifest. Automatic use when `vendor/` exists (Go 1.14+). |
| Cargo | `cargo vendor` | Copies sources to `vendor/`. Generates `.cargo/config.toml` to redirect source. |
| npm | `npm pack` / checked-in `node_modules` | Less common. Some use `npm ci --prefer-offline` with pre-populated cache. |
| pnpm | `pnpm store` | Content-addressable store can be pre-populated for offline use. |
| Python | `pip download` | Downloads wheels/sdists to a directory. `pip install --no-index --find-links=./vendor`. |

**Reproducible builds** require:
1. Locked versions (lockfile).
2. Locked content (integrity hashes).
3. Locked build environment (vendoring or hermetic builds like Bazel/Nix).

### Security

**Supply chain attack vectors**:
- **Typosquatting**: Registering packages with names similar to popular packages.
- **Account takeover**: Compromising maintainer credentials (npm Shai-Hulud attack, Sept 2025 -- 18 popular packages compromised via phishing).
- **Dependency confusion**: Private package names colliding with public registry names.
- **Malicious updates**: Maintainers injecting malicious code in new versions (colors/faker, 2022).
- **Build-time attacks**: Cargo build scripts, npm install scripts, pip setup.py execution.

**Defense mechanisms**:

| Mechanism | npm | Cargo | pip/uv | Go |
|-----------|-----|-------|--------|-----|
| Lockfile integrity | SHA-512 in package-lock.json | SHA-256 in Cargo.lock | SHA-256 in uv.lock/poetry.lock | SHA-256 in go.sum |
| Vulnerability scanning | `npm audit` | `cargo audit` (RustSec) | `pip-audit`, `safety` | `govulncheck` |
| Provenance attestation | Sigstore integration | RFC in progress | Trusted Publishers (OIDC) | Checksum database (sum.golang.org) |
| SBOM generation | `npm sbom` (CycloneDX/SPDX) | `cargo-sbom` | `cyclonedx-python` | `go version -m` extracts module info |
| Install script control | `--ignore-scripts` flag | N/A (build.rs always runs) | `--no-build-isolation` | N/A (no install scripts) |

**CISA guidance (2025)**: "Ensure all packages used are more than 60 days old" to avoid newly published malicious packages. Machine-readable SBOM formats (SPDX, CycloneDX) increasingly required for compliance.

Source: [npm supply chain security](https://bastion.tech/blog/npm-supply-chain-attacks-2026-saas-security-guide), [awesome-software-supply-chain-security](https://github.com/bureado/awesome-software-supply-chain-security)
