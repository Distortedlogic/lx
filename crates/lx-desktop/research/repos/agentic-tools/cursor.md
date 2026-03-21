# Cursor: The IDE-First Agent with Background Agents and Two-Stage Apply

Cursor demonstrates that **the IDE is the right surface for AI coding when you optimize for flow state — fast autocomplete, integrated chat, parallel agent sessions, and a specialized Apply model that separates reasoning from file modification**. It captured 19% of "most loved" mentions (second only to Claude Code at 46%) and remains the strongest option for developers who want AI deeply integrated into their editing experience.

## Overview

| Metric | Value |
|--------|-------|
| **Website** | [cursor.com](https://cursor.com) |
| **Developer** | Anysphere (founded 2022, 4 MIT CS graduates) |
| **Platform** | VS Code fork |
| **Valuation** | $29.3B (Nov 2025), in talks for ~$50B (Mar 2026) |
| **Total Funding** | ~$3.5B (Seed $8M, Series A ~$60M, Series B $900M, Series D $2.3B) |
| **Revenue** | $2B+ ARR (March 2026, up from $500M in June 2025) |
| **Pricing** | Free / Pro $20/mo / Pro+ $60/mo / Ultra $200/mo / Business $40/user/mo |
| **Parallel Agents** | Up to 8 simultaneous background agents |
| **Tab Requests** | 400M+ per day across all users |
| **Survey Position** | 19% "most loved" (second to Claude Code's 46%) |

## Architecture

### Two-Stage Apply Model

Cursor's key architectural innovation: separating reasoning from file modification.

1. **Primary LLM** — Generates a change sketch focused on logic and intent
2. **Apply Model** — Specialized model that intelligently integrates the sketch into existing files

This separation means the primary model doesn't need to perfectly reproduce unchanged code — it focuses on what should change and why, while the Apply model handles the mechanical integration. This reduces errors from large-file edits where traditional approaches (whole-file replacement, search/replace) struggle.

### Composer Mode

Multi-file editing from a single prompt:
- Plans changes across multiple files
- Shows proposed changes in a diff view
- Applies changes atomically
- Supports iterative refinement

### Background Agents

Run up to 8 parallel agent sessions simultaneously:
- Each agent works independently on a separate task
- Agents can modify files, run commands, browse
- Developer reviews results when agents complete
- Async workflow similar to Antigravity's Manager view but within the IDE

### Tab Completion and Prediction

Cursor's autocomplete is its most-used feature:
- **Tab** — Accept AI-generated completion
- **Next Edit Prediction** — Predicts not just the current line but what the developer will edit next
- **Multi-cursor** — AI-aware multi-cursor editing
- Optimized for speed — completions must feel instant

### Context System

- **IDE LSP** — Full language server protocol for type information
- **Embeddings** — Codebase-level embeddings for semantic search
- **Open Files** — Current open files as immediate context
- **`.cursorrules`** — Project-level AI instructions (analogous to CLAUDE.md)
- **`.mdc` files** — More structured rules with description, globs, and alwaysApply fields

## Key Features

### BugBot / Code Reviews

AI-powered code review that runs on PRs:
- Identifies potential bugs, security issues
- Suggests improvements
- Integrates with GitHub PR workflow

### Memory

Project-level context that persists across sessions:
- Remembers coding patterns
- Stores project conventions
- User-configurable

### Model Flexibility

Supports multiple providers:
- Claude (Sonnet, Opus)
- GPT-4o, GPT-5.x
- Gemini
- Custom models via API key

## Competitive Position

Cursor's strength is flow — it optimizes for the developer staying in the IDE, editing fast, and getting instant feedback. Its weakness is complex multi-file architectural refactors where terminal-first agents (Claude Code) or spec-driven tools (Kiro) perform better.

| Aspect | Cursor | Claude Code | Antigravity |
|--------|--------|-------------|-------------|
| **Surface** | IDE | Terminal | IDE + Manager |
| **Flow** | Best (instant autocomplete) | Good (fast model) | Good (multiple surfaces) |
| **Deep Reasoning** | Limited | Best (Opus 4.6 + Think) | Good (Gemini 3 Pro) |
| **Multi-file** | Composer mode | Sub-agents | Multi-agent Manager |
| **Background** | 8 parallel agents | Sub-agents | Async agents |
| **Apply** | Two-stage Apply model | Direct edit | Direct edit |
| **Refactoring Accuracy** | 78% | Higher | 94% |
| **Price** | $20/mo | API costs | Free (credit-based) |

The criticism that Cursor struggles with large multi-file refactors and exhibits looping behavior on complex tasks has driven many developers to adopt a hybrid workflow: Cursor for quick edits and autocomplete, Claude Code for complex architectural work.
