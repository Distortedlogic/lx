# Linting Design Patterns

Cross-cutting architectural patterns, trade-offs, and lessons learned from
surveying linting tools across Python, Rust, JavaScript, Go, Ruby, Shell,
Elixir, and Lua.

---

## 1. Parsing Strategy Spectrum

Linters face a fundamental choice in how they obtain a code representation.
The approaches form a spectrum from fully integrated to fully independent.

### 1.1 Reuse the compiler's parser/AST

**Examples:** Clippy (rustc HIR/AST), go vet / staticcheck (go/packages + go/ast),
Credo (Elixir's Code.string_to_quoted), Pyflakes (Python's ast.parse)

**Advantages:**
- Zero divergence from language semantics
- Access to type information, scope resolution, trait/interface resolution
- No parser maintenance burden
- Semantic analysis "for free"

**Disadvantages:**
- Tied to compiler release cycle
- Cannot lint invalid/incomplete code (no error recovery)
- Performance bound by compiler's parsing speed
- Cannot operate on code fragments or partial files

### 1.2 Independent parser, same AST format

**Examples:** ESLint (Espree produces ESTree, same spec as other JS tools),
RuboCop (parser gem produces standard Ruby AST)

**Advantages:**
- Can evolve independently of the compiler
- Can add error recovery
- Shared AST format enables tooling ecosystem interop

**Disadvantages:**
- Must track language spec changes
- May diverge from compiler behavior on edge cases
- Still typically cannot lint across syntax errors

### 1.3 Fully custom parser

**Examples:** Ruff (hand-written recursive descent in Rust for Python),
Biome (per-language parsers producing CSTs), ShellCheck (Parsec-based parser)

**Advantages:**
- Maximum control over error recovery, performance, and output format
- Can parse invalid code (essential for linters)
- Can optimize for linting-specific needs
- Can produce CST (lossless) instead of AST (lossy)

**Disadvantages:**
- Significant implementation effort
- Must faithfully reproduce language semantics
- Parser bugs become linter bugs
- Test suite must cover the entire language grammar

### 1.4 No parser (line/token-based)

**Examples:** pycodestyle (regex on raw lines), Flake8 token checkers,
Pylint raw checkers

**Advantages:**
- Extremely fast
- Simple implementation
- Works on any text, even non-parseable files

**Disadvantages:**
- Limited to surface-level checks
- False positives from string literals, comments, etc.
- Cannot reason about code structure

**Key insight:** The most effective linters offer *multiple* levels. Pylint has
AST, token, and raw checkers. Ruff does AST-based semantic analysis but could
theoretically add token-based fast paths. The level should match the check's
requirements.

---

## 2. AST vs CST

A critical architectural decision with cascading consequences.

### Abstract Syntax Tree (AST)

**Used by:** Ruff, Pylint/astroid, ESLint, Clippy (HIR), go vet, Pyflakes,
RuboCop, Luacheck, Credo

The AST discards whitespace, comments, and formatting details. Nodes represent
semantic constructs (functions, expressions, statements).

**For linting:** Simpler to work with for semantic rules. Rule authors deal with
clean, normalized structures. Auto-fix is harder because you need separate
source-mapping to modify the original text.

### Concrete Syntax Tree (CST) / Lossless Syntax Tree

**Used by:** Biome (via biome_rowan), rust-analyzer (via rowan)

The CST preserves everything: whitespace, comments, parentheses, semicolons.
Biome uses the **Red-Green tree pattern** from Microsoft's Roslyn:

- **Green tree:** immutable, position-independent, stores SyntaxKind (u16) and
  text width. Can be shared/cached. No parent pointers.
- **Red tree:** on-demand wrapper over green tree. Adds parent pointers and
  absolute offsets. This is the API surface.

**For linting:** More complex rule authoring, but fixes are trivial -- modify
the tree and serialize. The same tree serves parsing, linting, formatting, and
refactoring. This is why Biome can run formatting and linting in the same pass.

**Trade-off summary:**

| Property | AST | CST |
|----------|-----|-----|
| Rule authoring complexity | Lower | Higher |
| Auto-fix implementation | Harder (need source map) | Trivial (edit tree, serialize) |
| Memory usage | Lower | Higher |
| Unified tooling (lint+fmt) | Difficult | Natural |
| Comment/whitespace awareness | Lost (must be recovered) | Preserved |

---

## 3. Rule Definition Patterns

### 3.1 Visitor callbacks

The dominant pattern. Rules register interest in specific node types and receive
callbacks during traversal.

**Variations:**
- **Direct method naming:** Pylint `visit_functiondef`, RuboCop `on_send`,
  Clippy `check_expr`
- **Event subscription:** ESLint returns `{ NodeType(node) {} }` from `create()`
- **Type-parameterized:** Biome's `Query` associated type determines which
  CST nodes trigger the rule

**Bidirectional traversal:** ESLint and Pylint support both entry (`visit_*` /
`NodeType`) and exit (`leave_*` / `NodeType:exit`) callbacks, enabling rules
that track state across a subtree (e.g., counting returns in a function body).

### 3.2 Pattern matching DSLs

Some linters provide declarative pattern languages that avoid manual AST
traversal:

- **RuboCop NodePattern:** Regex-like syntax for AST matching.
  `(send nil? :require (str $_))` matches `require "foo"` and captures `"foo"`.
  Compiled to Ruby code for performance.

- **ESLint AST Selectors:** CSS-like syntax.
  `CallExpression[callee.name='eval']` matches `eval()` calls.
  Uses esquery internally.

- **GritQL:** Structural pattern matching across languages (external tool).

These DSLs dramatically reduce the code needed for simple pattern-matching rules
and make rules more readable.

### 3.3 Macro-generated rule scaffolding

Rust-based linters use macros to generate boilerplate:

- **Clippy:** `declare_clippy_lint!` generates static Lint references,
  LintInfo metadata, and documentation linkage from a single declaration.

- **Biome:** `declare_lint_rule!` generates RuleMetadata, trait implementations,
  and automatic group registration.

The pattern: a macro takes (name, category, description, metadata) and produces
everything the framework needs to discover, configure, and document the rule.

---

## 4. Rule Categorization Patterns

Every mature linter organizes rules into categories. Common groupings:

| Concept | Clippy | ESLint | Biome | RuboCop | Staticcheck |
|---------|--------|--------|-------|---------|-------------|
| Bugs/errors | correctness | problem | correctness | Lint | SA |
| Style | style | layout | style | Style, Layout | ST |
| Suggestions | complexity | suggestion | complexity | Refactoring | S |
| Performance | perf | (plugin) | performance | Performance* | (part of SA) |
| Security | (restriction) | (plugin) | security | Security | -- |
| Accessibility | -- | (plugin) | a11y | -- | -- |
| Incubating | nursery | -- | nursery | -- | -- |

**Default severity by category** is a pattern pioneered by Clippy:
correctness=deny, style/complexity=warn, pedantic/restriction=allow. This
gives users a sensible default while keeping niche rules available.

**Prefixed naming** (Staticcheck SA/S/ST/QF, Ruff F/E/W/PL/B) enables
coarse-grained rule selection by prefix rather than listing individual rules.

---

## 5. Configuration Approaches

### 5.1 Dedicated config files

| Tool | File | Format |
|------|------|--------|
| Clippy | clippy.toml | TOML |
| ESLint | eslint.config.js | JavaScript |
| Biome | biome.json | JSON |
| RuboCop | .rubocop.yml | YAML |
| Ruff | ruff.toml | TOML |
| golangci-lint | .golangci.yml | YAML |
| Staticcheck | staticcheck.toml | TOML |

### 5.2 Embedded in project manifest

- Ruff: `[tool.ruff]` in pyproject.toml
- Pylint: `[tool.pylint]` in pyproject.toml
- Clippy: `rust-version` in Cargo.toml (for MSRV)
- ESLint (legacy): `eslintConfig` in package.json

### 5.3 Configuration hierarchy

Ruff's three-level model is representative: CLI args > project config > user
config > defaults. Biome supports nested `biome.json` files discovered by
scanning. ESLint's flat config evaluates an array of config objects in order,
where later objects override earlier ones.

### 5.4 Inline suppression

Universal pattern: comments that suppress specific rules for specific lines.

| Tool | Syntax |
|------|--------|
| ESLint | `// eslint-disable-next-line no-eval` |
| Ruff | `# noqa: F841` |
| Clippy | `#[allow(clippy::needless_return)]` |
| RuboCop | `# rubocop:disable Style/StringLiterals` |
| ShellCheck | `# shellcheck disable=SC2162` |
| Biome | `// biome-ignore lint/style/noVar: reason` |

Block-level suppression (disable/enable pairs) is less common but supported by
Ruff (`# ruff: disable[N803]` ... `# ruff: enable[N803]`) and ESLint
(`/* eslint-disable */` ... `/* eslint-enable */`).

---

## 6. Auto-Fix Architecture

### 6.1 Text edits

The simplest model. Fixes are expressed as text replacements on the original
source: (range, replacement_text). Used by ESLint, ShellCheck, Ruff.

**Conflict resolution:** When multiple fixes affect overlapping ranges, the
linter must either reject conflicting fixes or iterate (apply fixes, re-parse,
re-lint, repeat until stable). Ruff does iterative application until
convergence.

### 6.2 Tree edits

Fixes modify the syntax tree directly, which is then serialized back to source.
Used by Biome (CST mutation) and RuboCop (TreeRewriter). More robust because
tree structure prevents invalid edits.

### 6.3 Safety classification

Modern linters classify fix safety:

| Level | Ruff | Biome | Clippy |
|-------|------|-------|--------|
| Safe / always apply | Safe | Always | MachineApplicable |
| Maybe wrong | Unsafe | MaybeIncorrect | MaybeIncorrect |
| Needs human | Display | -- | HasPlaceholders |

Ruff requires `--unsafe-fixes` flag; Clippy marks applicability per suggestion.
This prevents auto-fix from silently changing program semantics.

### 6.4 Suggestions vs fixes

ESLint distinguishes between **fixes** (auto-applied with `--fix`) and
**suggestions** (shown in editor, manually applied). This two-tier model lets
rules offer potentially-breaking transformations without risking auto-application.

---

## 7. Plugin and Extension Architectures

### 7.1 Entry-point / package-based plugins

**ESLint:** Plugins are npm packages exporting rules, configs, and processors.
Loaded via config. The dominant model for JS ecosystem.

**Flake8:** Plugins register via Python `entry_points`. Each plugin is a
callable that receives AST or lines and returns violations.

**RuboCop:** Extension gems register via `Inject` module. Generated with
`rubocop-extension-generator`.

**Credo:** Plugins are Elixir modules with `init/1` callback. Hook into
execution pipeline steps.

**Trade-offs:** Maximum extensibility, but plugin quality varies. Ecosystem
fragmentation (many plugins doing similar things). Performance unpredictable
(plugins can be slow). Version compatibility issues between plugin and host.

### 7.2 No plugin system (monolithic)

**Clippy:** All ~800 lints are first-party, shipping with the Rust toolchain.
No external plugin API.

**Ruff:** All 800+ rules are first-party Rust implementations. No plugin API
by design -- rules must be reimplemented in the ruff codebase.

**Biome:** All 450+ rules are first-party. Plugin system is a future goal but
not yet implemented.

**Trade-offs:** Consistent quality and performance across all rules. Rules can
share internal infrastructure freely. But users cannot add project-specific
rules without forking. This works when the rule set is comprehensive enough.

### 7.3 Framework-based (analyzer framework)

**Go (go/analysis):** The `Analyzer` type defines a standard interface. Any
tool can implement an analyzer and any driver can run it. Analyzers declare
dependencies and share results. This is the most principled approach.

**Key innovation:** Analyzers can export **facts** -- serialized data about
objects in a package. Other analyzers (or the same analyzer on different
packages) can import these facts. This enables modular cross-package analysis
without whole-program analysis.

---

## 8. Performance Patterns

### 8.1 Language choice

The single biggest performance factor. Ruff (Rust) is 10-100x faster than
pylint/flake8 (Python). Biome (Rust) is 15x faster than ESLint (JavaScript).

Haskell (ShellCheck) proved problematic: unpredictable optimization, space
leaks with multithreading, 10% regression from abstraction changes.

### 8.2 Parallelism

| Tool | Strategy |
|------|----------|
| Ruff | rayon `par_iter()` over files |
| Biome | Rust multithreading |
| Pylint | multiprocessing (`-j N`) with map/reduce |
| golangci-lint | concurrent linter execution, bounded |
| ESLint | None (single-threaded) |
| ShellCheck | Dropped (Haskell space leaks) |

File-level parallelism is the easy win: each file is independent for most
rules. Cross-file analysis (type checking, fact propagation) is inherently
harder to parallelize.

### 8.3 Caching

| Tool | Cache granularity | Cache key |
|------|-------------------|-----------|
| Ruff | Per-package | SHA-256(content + config + version) |
| ESLint | Per-file | Content hash + config hash |
| golangci-lint | Per-package | Content hash |
| Biome | (implicit via daemon) | -- |

**Limitation:** Cross-file analysis (typed linting) fundamentally breaks
per-file caching. A change to file A may change the diagnostics for file B.
ESLint's `--cache` is "fundamentally broken by cross-file information."

### 8.4 Single-pass architecture

Ruff and Biome both parse each file once and run all rules in a single
traversal. Compare with the Flake8 model where pyflakes parses the AST,
pycodestyle scans raw lines, and each plugin may parse independently.

### 8.5 Error-recovering parsers

Ruff's hand-written parser can parse invalid Python, producing a partial AST.
Biome's parser produces "bogus nodes" for broken syntax. This avoids the
pathological case where a single syntax error prevents all linting.

---

## 9. Compiler-Integrated vs Standalone Linting

### Compiler-integrated (Clippy, go vet)

**Pros:**
- Access to full type information, trait resolution, lifetime analysis
- No parser divergence -- always matches the compiler's interpretation
- Can leverage compiler infrastructure (diagnostics, suggestions)
- Runs as part of the normal build workflow

**Cons:**
- Tied to compiler version and release cycle
- Cannot lint code that doesn't compile
- Performance limited by compiler startup cost
- Cannot evolve parser independently (no error recovery)
- Clippy's reliance on rustc internals creates maintenance burden (unstable APIs)

### Standalone (Ruff, ESLint, Biome, ShellCheck)

**Pros:**
- Can lint broken/incomplete code
- Independent release cycle
- Can optimize for linting-specific needs
- Can run much faster (skip type checking, skip codegen)
- Easier to install and configure

**Cons:**
- Must maintain its own parser (potential for divergence)
- Limited or no type information (Ruff has lightweight semantic analysis;
  Biome is building its own type system)
- Cannot catch type-level bugs

### Hybrid approaches

- **typescript-eslint:** ESLint (standalone) + TypeScript compiler API for
  type-aware rules. Gets type info by running tsc as a library.
- **Staticcheck:** Standalone binary but uses Go's `go/packages` loader and
  SSA builder, getting compiler-quality type information without being a
  compiler plugin.
- **Biome's plan:** Build a custom type synthesizer rather than depending on
  the TypeScript compiler, getting type awareness without compiler coupling.

**Emerging consensus:** The best modern linters are standalone (for speed and
error recovery) but incorporate lightweight semantic analysis (scope, bindings,
name resolution) that covers 80-90% of what type information provides. Full type
checking is reserved for a separate, opt-in pass.

---

## 10. Incremental Analysis

The hardest unsolved problem in linting architecture.

### File-level invalidation

Most linters treat files as independent units. Change a file, re-lint that file.
This works for single-file rules but breaks for cross-file checks.

### Dependency-aware invalidation

TypeScript's `.tsbuildinfo` approach: track which files depend on which, and
re-check only the transitive dependents of changed files. No mainstream linter
fully implements this yet.

### Salsa / demand-driven computation

Ruff's type checker (`ty`) uses the **Salsa** framework (from rust-analyzer)
for incremental computation. Salsa memoizes pure functions keyed by their
inputs. When inputs change, only downstream computations are invalidated.
This is the most principled approach but adds significant complexity.

### Daemon / LSP mode

Biome and Ruff both run as long-lived daemon processes, keeping parsed trees
and analysis results in memory. On file change, they re-parse only the changed
file and re-run affected rules. This is effectively in-memory incremental
analysis.

---

## 11. Error Reporting Patterns

### Structured diagnostics

Modern linters produce rich diagnostics beyond "file:line: message":

- **Primary span:** The exact source range of the problem
- **Secondary spans / related information:** Other relevant locations
- **Fix suggestions:** One or more proposed code changes
- **Severity:** Error, warning, info, hint
- **Category / rule ID:** Machine-readable identifier
- **Documentation URL:** Link to detailed explanation

### Output formats

Every linter supports multiple output formats:

| Format | Purpose |
|--------|---------|
| Human-readable (TTY) | Developer terminal with colors and carets |
| JSON | Machine consumption, editor integration |
| SARIF | GitHub code scanning, CI integration |
| JUnit XML | CI test reporting |
| GCC-compatible | `file:line:col: severity: message` |
| Checkstyle XML | Jenkins integration |

### Educational diagnostics

ShellCheck pioneered per-warning documentation pages (SC2162, SC2034, etc.)
with examples, explanations, and caveats. Clippy's `--explain` flag shows
detailed documentation. This teaching-oriented approach is now standard.

---

## 12. Lessons for New Linter Design

### Start with the parser

The parser is the foundation. Key decisions:
1. **Error recovery** -- essential for a linter. Users lint broken code.
2. **Lossless vs lossy** -- CST enables unified lint+format; AST is simpler.
3. **Speed** -- hand-written recursive descent beats parser generators by 2x+.
4. **Token spans** -- capture full ranges from the start. Retrofitting is painful.

### Rule authoring ergonomics matter

The easier it is to write rules, the more rules you get. Key enablers:
- Pattern matching DSLs (RuboCop NodePattern, ESLint selectors)
- Code generation from declarations (Clippy macros, Biome macros)
- Rich context objects (ESLint's context.sourceCode, Clippy's clippy_utils)
- Good test infrastructure (ESLint RuleTester, RuboCop expect_offense)

### Semantic analysis without full type checking

Ruff's `SemanticModelBuilder` provides scope analysis, name binding, and
use-def chains without running a type checker. This lightweight semantic
layer catches most real bugs (undefined names, unused variables, shadowing)
at a fraction of the cost.

### Safety-classified auto-fix

The safe/unsafe fix distinction (Ruff, Biome, Clippy) is essential for
trust. Users will `--fix` confidently only if they know safe fixes preserve
semantics. Unsafe fixes should require explicit opt-in.

### Monolithic > plugin for performance

Ruff and Biome prove that reimplementing all rules in a fast language,
without plugin overhead, produces dramatically better performance than a
plugin architecture. The trade-off is extensibility, but for a linter with
comprehensive built-in rules, this is acceptable.

### Cross-package / cross-file analysis via facts

Go's `analysis.Fact` system is the most elegant solution to modular
cross-package analysis. Facts are serialized data exported by an analyzer
for one package and imported when analyzing dependents. This enables checks
like "this function wraps printf" to propagate across package boundaries
without whole-program analysis.

---

## Sources

### Python
- [Pylint checker docs](https://pylint.pycqa.org/en/latest/development_guide/how_tos/custom_checkers.html)
- [Astroid library](https://github.com/pylint-dev/astroid)
- [Flake8 plugin handling](https://flake8.pycqa.org/en/7.0.0/internal/plugin_handling.html)
- [Ruff repo](https://github.com/astral-sh/ruff)
- [Ruff v0.4.0 parser blog](https://astral.sh/blog/ruff-v0.4.0)
- [Ruff internals deep dive](https://compileralchemy.substack.com/p/ruff-internals-of-a-rust-backed-python)
- [DeepWiki Ruff overview](https://deepwiki.com/astral-sh/ruff/1-ruff-overview)

### Rust
- [Clippy lint passes](https://doc.rust-lang.org/nightly/clippy/development/lint_passes.html)
- [Clippy adding lints](https://doc.rust-lang.org/nightly/clippy/development/adding_lints.html)
- [DeepWiki Clippy](https://deepwiki.com/rust-lang/rust-clippy)

### JavaScript / TypeScript
- [ESLint architecture](https://eslint.org/docs/latest/contribute/architecture/)
- [ESLint custom rules](https://eslint.org/docs/latest/extend/custom-rules)
- [Biome architecture](https://biomejs.dev/internals/architecture/)
- [Biome rule engine](https://deepwiki.com/biomejs/biome/5.2-rule-engine-architecture)
- [Biome linter](https://biomejs.dev/linter/)

### Go
- [go/analysis package](https://pkg.go.dev/golang.org/x/tools/go/analysis)
- [Staticcheck checks](https://staticcheck.dev/docs/checks)
- [go-tools repo](https://github.com/dominikh/go-tools)
- [golangci-lint](https://golangci-lint.run/docs/welcome/quick-start/)

### Other
- [ShellCheck lessons blog](https://www.vidarholen.net/contents/blog/?p=859)
- [ShellCheck repo](https://github.com/koalaman/shellcheck)
- [RuboCop custom cops](https://evilmartians.com/chronicles/custom-cops-for-rubocop-an-emergency-service-for-your-codebase)
- [RuboCop NodePattern](https://docs.rubocop.org/rubocop-ast/node_pattern.html)
- [Credo repo](https://github.com/rrrene/credo)
- [Luacheck repo](https://github.com/lunarmodules/luacheck)
- [Linter architecture essay (Josh Goldberg)](https://www.joshuakgoldberg.com/blog/if-i-wrote-a-linter-part-1-architecture/)
