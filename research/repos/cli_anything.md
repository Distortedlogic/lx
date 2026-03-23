# CLI-Anything: Automatic CLI Generation for Agent-Native Software Control

CLI-Anything proves that **structured command-line interfaces generated from source code analysis** can replace fragile GUI automation, API wrappers, and screen-scraping for AI agent tool use. By running a 7-phase automated pipeline over any GUI application's codebase, it produces a stateful CLI with JSON output, undo/redo, REPL mode, and 100% test coverage -- giving agents reliable, deterministic control of professional software like GIMP, Blender, and LibreOffice. The project reached 10,800+ stars within 5 days of creation, signaling massive demand for the agent-native tooling pattern.

## Repository Overview

| Metric | Value |
|--------|-------|
| **GitHub** | [HKUDS/CLI-Anything](https://github.com/HKUDS/CLI-Anything) |
| **Stars** | 10,820 |
| **Forks** | 924 |
| **Language** | Python |
| **Created** | March 8, 2026 |
| **Last Updated** | March 13, 2026 |
| **Open Issues** | 39 |
| **Primary Author** | yuh-yang (22 commits) |
| **Organization** | Data Intelligence Lab @ HKU (Hong Kong University) |
| **Total Tests** | 1,508+ passing across 11 applications |
| **Python** | >= 3.10 |
| **CLI Framework** | Click >= 8.0 |

## What It Does

CLI-Anything transforms GUI applications into **agent-controllable command-line tools** by analyzing an application's source code, identifying its backend engine and data model, and generating a complete CLI harness that invokes the real software for rendering. The core thesis: agents should not reimplement software in Python or automate pixels -- they should call the real application through structured commands that return machine-parseable JSON.

The system operates as a **plugin for AI coding agents** (Claude Code, OpenCode, Qodercli, Codex). When an agent invokes `/cli-anything ./gimp`, the pipeline clones or reads the source, analyzes it, designs command groups, implements a Click-based CLI with REPL, writes comprehensive tests, documents results, and installs the CLI to PATH. The entire process is automated -- no human configuration required.

After generation, the resulting CLI supports both one-shot commands and interactive sessions:

```
cli-anything-gimp project new --width 1920 --height 1080 -o poster.json
cli-anything-gimp --json layer add -n "Background" --type solid --color "#1a1a2e"
cli-anything-gimp   # enters interactive REPL
```

## Architecture

### The 7-Phase Pipeline

The generation pipeline is the core intellectual contribution. Each phase has explicit inputs, outputs, and validation criteria:

| Phase | Name | Function | Output |
|-------|------|----------|--------|
| 0 | Source Acquisition | Clone repo or validate local path | Local source directory |
| 1 | Codebase Analysis | Identify backend engine, data model, GUI-to-API mappings, existing CLI tools, command/undo system | `<SOFTWARE>.md` analysis document |
| 2 | CLI Architecture Design | Define command groups, state model, output format, interaction model (REPL + subcommands) | Architecture specification |
| 3 | Implementation | Build Click CLI with namespace packages, REPL skin, JSON output, session management, backend integration | Full `cli_anything/<software>/` package |
| 4 | Test Planning | Create `TEST.md` with unit test plan, E2E test plan, workflow scenarios, estimated counts | Test specification document |
| 5 | Test Implementation | Write unit tests, format E2E tests, backend E2E tests, subprocess CLI tests | Test suite with 100% pass rate |
| 6 | Documentation | Append test results to `TEST.md`, update README | Complete documentation |
| 7 | Publishing | Create `setup.py` with namespace packages, `pip install -e .`, verify PATH installation | Installed CLI binary |

### Generated CLI Structure

Every generated CLI follows an identical directory layout:

```
<software>/agent-harness/
    <SOFTWARE>.md
    setup.py
    cli_anything/           # PEP 420 namespace package (NO __init__.py)
        <software>/         # Regular sub-package (HAS __init__.py)
            __init__.py
            __main__.py
            <software>_cli.py
            README.md
            core/
                project.py
                session.py
                export.py
                ...domain modules...
            utils/
                <software>_backend.py
                repl_skin.py
            tests/
                TEST.md
                test_core.py
                test_full_e2e.py
```

The **namespace package** pattern (`cli_anything/` with no `__init__.py`) is a critical design choice: multiple independently-installed CLIs coexist in the same Python environment without conflicts. `import cli_anything.gimp` and `import cli_anything.blender` resolve to their respective source directories.

### State Management Pattern

Generated CLIs use a **snapshot-based session model** rather than command objects:

- **Project state** is a nested dictionary serialized to JSON
- **Session** maintains an undo stack (max 50 snapshots), redo stack, and modification flag
- Before each mutation, `session.snapshot(description)` deep-copies current state onto the undo stack
- **Persistence** is JSON-based, making project files human-readable and version-control friendly
- The pattern trades memory for simplicity -- appropriate for agent workflows where predictability matters more than efficiency

### Backend Integration Pattern

The most important architectural rule: **never reimplement the software in Python**. Each CLI generates valid intermediate files (ODF XML, MLT XML, SVG, bpy scripts) and hands them to the real software for rendering via subprocess:

| Software | Backend Invocation | Intermediate Format |
|----------|-------------------|---------------------|
| GIMP | `gimp -i -b '(script-fu ...)'` | Script-Fu commands |
| Blender | `blender --background --python` | bpy Python scripts |
| Inkscape | `inkscape --actions="..."` | SVG XML |
| LibreOffice | `libreoffice --headless --convert-to` | ODF ZIP |
| Shotcut/Kdenlive | `melt` or `ffmpeg` | MLT XML |
| Audacity | `sox` | sox command chains |
| OBS Studio | `obs-websocket` | WebSocket API calls |

The backend module locates executables via `shutil.which()` and provides clear error messages with install instructions when software is missing. The software is a **hard dependency** -- no graceful degradation, no fallback libraries.

### REPL Skin System

All generated CLIs share a **unified REPL interface** (`ReplSkin` class) providing:

- Branded startup banners with box-drawing characters
- `prompt_toolkit` integration with history and auto-suggest
- Software-specific accent colors (GIMP orange, Blender orange, Inkscape blue, etc.)
- Standardized message types: `success()`, `error()`, `warning()`, `info()`
- Formatted table output for structured data
- Bottom toolbar showing session status
- `NO_COLOR` environment variable support

The REPL is the **default behavior** -- running `cli-anything-gimp` with no arguments enters interactive mode via `invoke_without_command=True`.

## Demonstrated Applications

| Software | Domain | Tests | Backend |
|----------|--------|-------|---------|
| GIMP | Image Editing | 107 | Pillow + GEGL |
| Blender | 3D Modeling | 208 | bpy Python |
| Inkscape | Vector Graphics | 202 | SVG/XML |
| Audacity | Audio Production | 161 | wave + sox |
| LibreOffice | Office Suite | 158 | ODF + headless |
| OBS Studio | Live Streaming | 153 | websocket |
| Kdenlive | Video Editing | 155 | MLT XML |
| Shotcut | Video Editing | 154 | MLT XML |
| Draw.io | Diagramming | 138 | mxGraph XML |
| Zoom | Conferencing | 22 | REST API |
| AnyGen | AI Content Gen | Pending | Cloud API |

**Total: 1,508+ tests across 11 applications, 100% pass rate claimed.**

## Agent Platform Integration

The plugin system adapts to different coding agent platforms:

| Platform | Integration Method | Status |
|----------|-------------------|--------|
| Claude Code | Plugin marketplace (`.claude-plugin/marketplace.json`) | Production |
| OpenCode | Slash commands copied to `~/.config/opencode/commands/` | Production |
| Qodercli | Shell setup script registering plugin | Production |
| Codex | Bundled skill installation via `install.sh` | Experimental |
| Cursor / Windsurf | Planned | Coming soon |

Each platform adapter consists of markdown command files that encode the 7-phase pipeline as agent instructions. The commands are: `cli-anything` (generate), `refine` (expand coverage), `test` (run tests), `validate` (check installation), and `list` (show installed CLIs).

## Key Design Decisions

**Why Click over argparse/Typer**: Click's decorator-based command groups map naturally to the nested command structure needed (e.g., `project new`, `layer add`, `filter apply`). The `invoke_without_command=True` pattern enables seamless REPL fallback. Click's composability supports the templated generation approach.

**Why JSON state over databases**: Agent workflows need inspectable, portable state. JSON project files can be checked into version control, diffed, and manually edited. The tradeoff is no concurrent access, but agent workflows are inherently sequential.

**Why subprocess over FFI**: Calling `blender --background --python script.py` is more reliable than linking against Blender's C library. Subprocess isolation means crashes in the target software don't crash the CLI. The cost is IPC overhead, which is negligible for creative workflows.

**Why namespace packages**: PEP 420 namespace packages allow `pip install cli-anything-gimp` and `pip install cli-anything-blender` to coexist without import conflicts. This is essential for agents that operate across multiple domains in a single environment.

**Why snapshot undo over command pattern**: Deep-copying state is wasteful but simple. For agent tool use, the priority is correctness and predictability over memory efficiency. A 50-snapshot cap bounds memory usage.

## Strengths

- **Real software integration** eliminates the toy-implementation problem that plagues most agent-tool demos
- **Deterministic, structured output** (JSON) is exactly what LLMs need for tool-use parsing
- **Comprehensive test methodology** with 4 layers (unit, format E2E, backend E2E, subprocess CLI) catches real integration failures
- **The HARNESS.md document** is an exceptionally well-written SOP that encodes years of integration lessons (rendering gap, filter translation pitfalls, timecode precision) into actionable rules
- **Incremental refinement** via `/refine` enables agents to expand coverage iteratively without destroying existing functionality
- **Platform-agnostic** command files work across Claude Code, OpenCode, Qodercli, and Codex

## Weaknesses

- **Agent-generated code quality is untested by third parties** -- the 1,508 tests are generated by the same pipeline, creating a circularity concern
- **No Windows support** -- open issues confirm failures on Windows paths; the backend invocation patterns assume Unix-like environments
- **Security concerns** -- Issue #60 flags Script-Fu injection vulnerabilities; subprocess calls with user-provided paths need sanitization
- **Session file locking** is absent (Issue #59) -- concurrent agents or processes could corrupt state
- **The "real software" claim is partially aspirational** -- GIMP's backend is listed as "Pillow + GEGL" rather than actual GIMP batch mode, suggesting some implementations use Python libraries as the backend rather than the actual application
- **Extremely rapid development** (22 commits, 5 days, 10k+ stars) raises questions about depth vs. viral marketing; the repo is very new
- **No benchmarks** comparing agent task completion rates with vs. without CLI-Anything

## Relevance to Agentic AI Patterns

### Agent Harness Pattern

CLI-Anything is the clearest implementation of the **agent harness** concept: a structured interface layer between an AI agent and a complex external system. The harness provides:

- **Tool discovery** via `--help` and `list` commands
- **State inspection** via `info`, `status`, `list` subcommands
- **Deterministic operations** via structured CLI commands
- **Error recovery** via explicit error messages with fix instructions
- **State rollback** via undo/redo

This pattern is directly applicable to any domain where agents need to control software: IDEs, databases, infrastructure tools, scientific instruments.

### Context Management

The session model addresses a core agentic challenge: **maintaining context across multi-step workflows**. The REPL preserves project state between commands, the undo stack enables recovery from agent mistakes, and JSON serialization enables state persistence across agent sessions. The `--json` flag ensures agents receive structured data they can parse without regex heuristics.

### Tool Orchestration

The generated CLIs demonstrate effective **tool composition**. An agent can chain commands:

1. `project new` -- create workspace
2. `layer add` -- build content
3. `filter apply` -- process content
4. `export render` -- produce output

Each step returns structured results the agent uses to parameterize the next step. This is the fundamental loop of agentic tool use: observe, decide, act, observe.

### Multi-Agent Coordination

The namespace package design enables multiple agents or agent workflows to operate different CLIs simultaneously in the same environment. However, the lack of session locking means true multi-agent coordination on the same project is unsafe.

### Code Generation as Tool Creation

The meta-pattern is perhaps the most interesting: **using a coding agent to generate tools for coding agents**. The pipeline itself runs inside Claude Code/OpenCode/Codex, meaning an LLM analyzes source code and writes the CLI that other LLMs will use. This recursive agent-tooling pattern will likely become common as agent ecosystems mature.

## Practical Takeaways

1. **Structured CLI > API wrappers** for agent tool use. CLIs with `--json` output, `--help` discovery, and consistent error formats are more reliable than REST API clients or SDK wrappers because they match the text-in/text-out paradigm LLMs operate in.

2. **The HARNESS.md methodology is reusable** independent of CLI-Anything. Its lessons about rendering gaps, filter translation, timecode precision, and subprocess backend integration apply to any agent-tool integration project.

3. **Namespace packages solve the multi-tool coexistence problem**. When agents need access to many tools, PEP 420 namespace packages prevent import conflicts without monorepo overhead.

4. **Snapshot-based undo is the right tradeoff for agent workflows**. Agents make mistakes; cheap rollback is more valuable than memory efficiency.

5. **REPL-first design with one-shot fallback** serves both interactive agent sessions and scripted pipelines. Making REPL the default (`invoke_without_command=True`) is the correct ergonomic choice for agents that maintain conversation context.

6. **Test methodology matters more than test count**. The 4-layer testing approach (unit, format validation, real-backend E2E, subprocess CLI) is more valuable than the 1,508 test count. Any agent-tool integration should verify at the subprocess boundary, not just at the Python API level.

7. **The "real software" principle is critical**. The difference between a useful agent tool and a toy demo is whether the tool produces professional-grade output. Calling `libreoffice --headless` produces real PDFs; reimplementing PDF generation in Python produces approximations.

## Sources

- [HKUDS/CLI-Anything GitHub Repository](https://github.com/HKUDS/CLI-Anything)
- [HARNESS.md - Complete 7-Phase Pipeline Specification](https://github.com/HKUDS/CLI-Anything/blob/main/cli-anything-plugin/HARNESS.md)
- [repl_skin.py - Unified REPL Interface Template](https://github.com/HKUDS/CLI-Anything/blob/main/cli-anything-plugin/repl_skin.py)
- [GIMP Agent Harness Implementation](https://github.com/HKUDS/CLI-Anything/tree/main/gimp/agent-harness)
- [Blender Agent Harness Implementation](https://github.com/HKUDS/CLI-Anything/tree/main/blender/agent-harness)
- [Claude Code Plugin Configuration](https://github.com/HKUDS/CLI-Anything/tree/main/.claude-plugin)
- [HKUDS - Data Intelligence Lab @ HKU](https://github.com/HKUDS)
