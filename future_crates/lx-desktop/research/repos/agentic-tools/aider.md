# Aider: The Pioneer of Structured AI Pair Programming

Aider proves that **a CLI-first, git-native tool with a repository map technique and structured edit formats can remain competitive against billion-dollar competitors by focusing on what matters most: reliable code editing across any model**. Created by Paul Gauthier, Aider pioneered multiple techniques now industry-standard: tree-sitter repo maps, search/replace edit blocks, architect mode, and the Polyglot benchmark for model evaluation.

## Overview

| Metric | Value |
|--------|-------|
| **GitHub** | [Aider-AI/aider](https://github.com/Aider-AI/aider) |
| **Stars** | 39,000+ |
| **Installations** | 4.1M+ |
| **Language** | Python |
| **License** | Apache 2.0 |
| **Install** | `pip install aider-chat` |
| **Best Models** | Claude Sonnet (84.2% edit score), DeepSeek R1/V3, GPT-5.x, Gemini 2.5 Pro |
| **Architect SOTA** | 85.7% (Sonnet + DeepSeek editor) |
| **Languages** | 100+ programming languages |

## Architecture

### Repository Map (Tree-sitter + PageRank)

Aider's most influential contribution — adopted across the industry:

1. **Code Parsing** — Tree-sitter parses source files into ASTs
2. **Symbol Extraction** — Function signatures, class definitions extracted as graph nodes
3. **Dependency Analysis** — PageRank-based ranking of symbols by importance
4. **Token Budget** — Map compressed to ~1,000 tokens representing the most important parts of the codebase

This gives the LLM a structural understanding of the repository without consuming the entire context window.

### Edit Format Taxonomy

Aider supports multiple edit formats, chosen based on model capabilities:

| Format | Description | Accuracy |
|--------|-------------|----------|
| **EditBlock (search/replace)** | Clearly delimited original/replacement blocks | High |
| **Unified Diff (udiff)** | Standard diff format (reduced GPT-4 Turbo "lazy coding" by 3x) | High |
| **Whole File** | Complete file replacement | 60-75% |
| **Editor-Diff** | New format combining editor intelligence with diff precision | High |
| **Editor-Whole** | Editor-aware whole file replacement | Medium-High |

Layered matching fallbacks: exact → whitespace-insensitive → indentation-preserving → Levenshtein fuzzy matching.

### Architect Mode

Two-model approach for state-of-the-art results:

1. **Architect Model** — Describes how to solve the problem (reasoning, planning)
2. **Editor Model** — Translates the plan into file edits

Sonnet 4.5 as architect + DeepSeek as editor = 85.7% on the edit leaderboard. `--auto-accept-architect` flag (default: true) skips confirmation for architect suggestions.

## Key Features

### Git-Native Integration

- **Automatic Commits** — Every AI change gets a Conventional Commits message
- **Undo** — `git diff` and `git revert` to undo any AI change
- **Branch-aware** — Understands the current git state

### Voice Coding

`/voice` command starts recording, press ENTER when done. Fluidly switch between voice and text chat. Request features, tests, or bug fixes by speaking.

### Watch Mode

`--watch-files` monitors all repo files for AI instructions in comments:
- Comments starting/ending with `AI`, `AI!`, or `AI?` trigger Aider
- Honors `--subtree-only` to watch specific directories
- Enables IDE-agnostic AI coding — write instructions in any editor

### Lint and Test Integration

Automatic linting and testing after every change:
- Run linter on modified files
- Execute test suite
- Auto-fix lint errors
- Configurable via `--lint-cmd` and `--test-cmd`

### `/context` Command

Automatically identifies which files need to be edited for a given request — reducing manual file selection.

## Polyglot Benchmark

Aider's own benchmark, now industry-standard for model evaluation:

- **225 problems** from Exercism's hardest exercises
- **6 languages** — C++, Go, Java, JavaScript, Python, Rust
- End-to-end evaluation: natural language → code → passing tests
- Latest leaders (March 2026): GPT-5 (88.0%), Grok 4 (79.6%), DeepSeek V3.2 (74.2%)

## Competitive Position

Aider's strength is universality — it works with any model via litellm, any git repo, any editor (via watch mode). Its weakness is polish — the TUI is basic compared to OpenCode/Crush, and it lacks the deep IDE integration of Cursor.

| Aspect | Aider | Claude Code | OpenCode |
|--------|-------|-------------|----------|
| **Repo Understanding** | Tree-sitter + PageRank map | On-demand reads | LSP integration |
| **Edit Format** | 5+ formats (model-dependent) | Direct edit | Direct edit |
| **Model Lock-in** | None (any via litellm) | Claude only | 75+ providers |
| **Git Integration** | Deepest (auto-commit, undo) | Good | Good |
| **Voice** | Yes (/voice command) | No | No |
| **Watch Mode** | Yes (IDE-agnostic) | No | No |
| **TUI** | Basic readline | Ink (React) | Bubble Tea (polished) |
| **Architect Mode** | Yes (two-model) | Think tool | No |
