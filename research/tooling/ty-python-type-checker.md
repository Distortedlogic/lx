# ty — Astral's Python Type Checker

## Overview

ty is an extremely fast Python type checker and language server written in Rust, built by Astral (the team behind ruff and uv). Formerly known as "Red Knot", it was publicly released in beta in 2025. ty includes Ruff as a submodule — much of its implementation is developed within the Ruff codebase, which explains similar diagnostic output formatting.

- **Repository**: https://github.com/astral-sh/ty
- **Documentation**: https://docs.astral.sh/ty/
- **Configuration Reference**: https://docs.astral.sh/ty/reference/configuration/
- **CLI Reference**: https://docs.astral.sh/ty/reference/cli/
- **Rules Catalog**: https://docs.astral.sh/ty/rules/
- **Blog Announcement**: https://astral.sh/blog/ty

## Status (as of March 2026)

- **Current phase**: Public beta
- **Stable 1.0 target**: 2026
- **Adoption**: Astral uses ty exclusively on their own projects
- **Recommendation**: Ready for "motivated users" in production
- **Caveats**: Hundreds of open GitHub issues, occasional fatal errors reported, behavior shifts between versions

### Remaining roadmap items
- Completing the long tail of features in the Python typing specification
- First-class support for popular third-party libraries (Pydantic, Django)
- Strict mode
- Plugin system for custom rules
- Further performance optimizations

## Performance Benchmarks

| Scenario | ty | Pyright | Mypy | Pyrefly |
|---|---|---|---|---|
| Cold check (no cache) | 1x (baseline) | 10-60x slower | 10-60x slower | — |
| Incremental (PyTorch edit) | 4.7ms | 386ms (80x slower) | — | 2.38s (500x slower) |
| Django (2,871 files) | ~3.8 seconds | — | — | — |

ty leverages multi-core processor architecture for parallel execution. Distributed as a pre-compiled binary (no Rust compiler needed).

## Typing Spec Conformance

Tested against the Python Typing Council's conformance test suite (v0.0.1-alpha.19):

| Metric | ty | Pyrefly | Zuban |
|---|---|---|---|
| Full Passes | 20 (~15%) | 81 (~58%) | 97 (~69%) |
| False Positives | 371 | — | — |
| False Negatives | 603 | — | — |
| Total Test Cases | 119 | 119 | 119 |

- The high false negative count (603) means ty frequently **fails to flag actual type errors**
- False positives (371) mean it occasionally flags incorrect issues
- Pyrefly outperforms ty partly because "the pyrefly team devoted a lot of up-front time to solving some of the hard problems, such as generics"
- Zuban was developed privately for years before public release, explaining its lead
- **Important context**: Despite poor conformance scores, ty has been found "surprisingly effective at catching real bugs" in practice — conformance test metrics don't fully reflect real-world utility
- All three tools are alpha/beta-stage; conformance will improve

Source: https://sinon.github.io/future-python-type-checkers/

## Key Features

### Type System
- First-class intersection types
- Advanced type narrowing
- Sophisticated reachability analysis

### Language Server (LSP)
- Go to Definition
- Symbol Rename
- Auto-Complete
- Auto-Import
- Semantic Syntax Highlighting
- Inlay Hints
- Works in any editor implementing LSP
- Official VS Code extension available; other editors use LSP (e.g. LSP4IJ for PyCharm)

### File Discovery
- Recursively scans `.py`, `.pyi` (stub), and `.ipynb` (Jupyter) files
- Automatically skips virtual environments and standard library packages
- Recognizes patterns in `.gitignore` and `.ignore` files
- Supports both src and flat project layouts

### Module Resolution
- Detects packages in activated virtual environments
- Recognizes `.venv/` folders even when inactive
- Falls back to system Python when needed
- Accepts custom interpreter paths via `--python` option

### Rule System
- Three severity levels: `"error"`, `"warn"`, `"ignore"`
- Default assignments vary per rule (e.g. `unresolved-attribute` defaults to error; `division-by-zero` defaults to ignore)
- Rules adjustable globally or per-line

### Inline Suppression
- `# ty: ignore[rule1, rule2]` — ty-specific suppression
- `# type: ignore` — PEP 484 format also supported
- **Known limitation**: Multiline expression suppression doesn't consistently work

## Configuration

ty reads config from `pyproject.toml` (under `[tool.ty]`) or a dedicated `ty.toml` file. If both exist, `ty.toml` takes precedence.

### Full Configuration Structure

```toml
[tool.ty.environment]
python = "./.venv"              # Path to interpreter, venv dir, or sys.prefix
python-version = "3.14"         # Format: "M.m" (supports 3.7+)
python-platform = "linux"       # "win32" | "darwin" | "linux" | "ios" | "android" | "all"
root = ["./src", "./lib"]       # Priority-ordered list for first-party module discovery
extra-paths = ["./shared"]      # Custom module search paths
typeshed = "/path/to/typeshed"  # Custom type stubs

[tool.ty.src]
include = ["src", "tests"]                    # Patterns to include
exclude = ["generated", "*.proto", "tests/fixtures/**"]  # Patterns to exclude
respect-ignore-files = true                   # Respect .gitignore

[tool.ty.rules]
all = "warn"                                  # Set default severity for all rules
possibly-unresolved-reference = "error"       # Override individual rules
division-by-zero = "ignore"                   # Severity: "ignore" | "warn" | "error"

[tool.ty.analysis]
allowed-unresolved-imports = ["test.**", "!test.foo"]  # Suppress unresolved import errors
replace-imports-with-any = ["pandas.**"]               # Replace unresolvable imports with Any
respect-type-ignore-comments = true                    # Support `type: ignore` directives

[tool.ty.terminal]
error-on-warning = false         # Exit code 1 on warnings
output-format = "full"           # "full" | "concise" | "github" | "gitlab" | "junit"

# Per-path overrides (repeatable)
[[tool.ty.overrides]]
include = ["tests/**", "**/test_*.py"]
exclude = ["tests/fixtures/**"]

[tool.ty.overrides.rules]
possibly-unresolved-reference = "warn"

[tool.ty.overrides.analysis]
allowed-unresolved-imports = ["test.**"]
```

### Auto-detection behavior
- Python version: falls back to `project.requires-python` from `pyproject.toml`
- Virtual environment: auto-detects `.venv` in project root
- Multiple overrides can apply to the same file; later entries take precedence

## CLI Usage

```bash
# Basic type check
ty check path/to/code/

# Check with specific python version
ty check --python-version 3.14 path/

# Check with specific output format (CI-friendly)
ty check --output-format github path/

# Cross-platform checking
ty check --platform all path/

# Watch mode for continuous feedback
ty check --watch path/

# Quiet/silent modes
ty check -qq path/

# CLI rule overrides
ty check --error rule-name --warn other-rule --ignore third-rule path/

# Run as language server
ty server
```

### Output Formats
- **Default**: Colorized, multi-line with code snippets, context, and notes
- **Concise**: Single-line per diagnostic (suitable for CI/CD parsing)
- **GitHub/GitLab**: Native annotation format for CI
- **JUnit**: XML test report format

## Migration from Mypy

Source: https://www.blog.pythonlibrary.org/2026/01/09/how-to-switch-to-ty-from-mypy/

- Running ty without changing any settings is "very similar" to running mypy in strict mode
- **Critical gap**: ty does NOT flag missing type annotations by default. To enforce this, use Ruff's `ANN` rules (flake8-annotations):
  ```toml
  [tool.ruff.lint]
  extend-select = ["ANN"]
  ```
- ty lacks official pre-commit hook support (GitHub issue exists requesting it; community workarounds available)

### Installation methods
- `uv tool install ty@latest` (recommended)
- `uvx ty` (no prior installation needed)
- `pip install ty` (supported but not primary)
- Standalone installer (platform-dependent)

### GitHub Actions integration
Create `.github/workflows/ty.yml` with Python 3.12+ and `pip install ty==<version>`. Pin the version explicitly.

## Migration from Pyright / basedpyright

### Conceptual mapping

| Pyright | ty |
|---|---|
| `pyrightconfig.json` | `[tool.ty]` in `pyproject.toml` or `ty.toml` |
| `typeCheckingMode: "strict"` | `[tool.ty.rules] all = "warn"` |
| `pythonVersion` | `[tool.ty.environment] python-version` |
| `pythonPlatform` | `[tool.ty.environment] python-platform` |
| `venvPath` / `venv` | `[tool.ty.environment] python` |
| `include` / `exclude` | `[tool.ty.src] include` / `exclude` |
| `extraPaths` | `[tool.ty.environment] extra-paths` |
| `reportMissingImports` etc. | `[tool.ty.rules]` with ty's own rule names |

### Key differences
- ty does NOT use pyright's `report*` rule names — it has its own rule catalog
- `all = "warn"` is the closest equivalent to pyright's `strict` mode
- Individual rules can be tuned to `"error"` or `"ignore"` from there
- No separate config file needed — lives alongside ruff config in `pyproject.toml`

## Known Limitations

- **Conformance**: ~15% pass rate on typing conformance suite (alpha stage; improving rapidly)
- **Missing annotation detection**: Does not flag missing type hints — must use Ruff `ANN` rules
- **IDE support**: Official VS Code extension only; other editors need LSP setup
- **Multiline suppression**: `# ty: ignore` doesn't consistently work across multiple lines
- **Pre-commit**: No official pre-commit hook yet
- **Rapid churn**: Behavior and features shift between versions — pin versions in CI
- **Generics**: Still catching up (Pyrefly invested more up-front effort here)

## Migration Plan for This Repo

### Install
```bash
uv tool install ty
```

### Changes needed

1. **Add `[tool.ty]` config to `pyproject.toml`** (before `[tool.ruff]` section):
```toml
[tool.ty.environment]
python = "./.venv"
python-version = "3.14"
python-platform = "linux"

[tool.ty.src]
include = ["inference"]
exclude = ["**/node_modules", "**/__pycache__", "**/.*", "reference"]

[tool.ty.rules]
all = "warn"

[tool.ty.terminal]
error-on-warning = true
```

2. **Update `justfile`** — change `py-lint` recipe:
```
# Change:  basedpyright inference/ 2>&1
# To:      ty check inference/ 2>&1
```

3. **Delete `pyrightconfig.json`** (used by basedpyright) — config now lives in `pyproject.toml`.

4. **Optional dep cleanup** — these dev deps are fully superseded by ruff:
   - `flake8`, `flake8-plugin-utils`, `flake8-type-checking` → ruff lint rules
   - `pylint` → ruff `PL` rules
   - `isort` → ruff `I` rules
   - `autoflake` → ruff `F` rules + `--fix`
   - `autopep8` → `ruff format`

## Relationship with ruff

ty and ruff are complementary:
- **ruff**: Linter (style, imports, best practices) + formatter
- **ty**: Type checker (type correctness, type inference) + language server

Both are written in Rust by Astral. ty includes Ruff as a submodule and shares infrastructure. Use both — ruff for linting/formatting, ty for type checking. Notably, ty relies on Ruff's `ANN` rules to catch missing type annotations, since ty itself doesn't flag those.

## Competitor Landscape

| Tool | Language | Maintainer | Status | Conformance |
|---|---|---|---|---|
| **mypy** | Python | Python community | Stable, production | Reference implementation |
| **Pyright** | TypeScript | Microsoft | Stable, production | High |
| **ty** | Rust | Astral | Beta | ~15% (improving) |
| **Pyrefly** | Rust | Meta | Alpha | ~58% |
| **Zuban** | ? | Private | Alpha | ~69% |
| **Pyre** | OCaml | Meta | Stable | High |

## References

- [ty announcement blog post](https://astral.sh/blog/ty)
- [ty documentation](https://docs.astral.sh/ty/)
- [ty GitHub repository](https://github.com/astral-sh/ty)
- [ty configuration reference](https://docs.astral.sh/ty/reference/configuration/)
- [ty CLI reference](https://docs.astral.sh/ty/reference/cli/)
- [Python type checker ty now in beta — InfoWorld](https://www.infoworld.com/article/4108979/python-type-checker-ty-now-in-beta.html)
- [ty Beta announcement — PyDevTools](https://pydevtools.com/blog/ty-beta/)
- [How to Switch to ty from Mypy — Mouse Vs Python](https://www.blog.pythonlibrary.org/2026/01/09/how-to-switch-to-ty-from-mypy/)
- [How Well Do New Python Type Checkers Conform?](https://sinon.github.io/future-python-type-checkers/)
- [ty — Python Developer Tooling Handbook](https://pydevtools.com/handbook/reference/ty/)
- [Astral's ty — Real Python](https://realpython.com/python-ty/)
- [Astral's ty: 60x Faster Than Mypy — ByteIota](https://byteiota.com/astral-ty-python-type-checker-60x-faster-than-mypy/)
