# Kiro: AWS's Spec-Driven Agentic IDE

Kiro proves that **spec-driven development — where the agent writes zero lines of code before producing requirements, architecture, and task plans — is a viable alternative to the vibe-coding pattern** that dominates tools like Cursor and Claude Code. Built by AWS on a VS Code (Code OSS) fork, Kiro transforms natural language prompts into EARS-notation requirements, technical design documents, and sequenced implementation tasks before writing a single line.

## Overview

| Metric | Value |
|--------|-------|
| **Website** | [kiro.dev](https://kiro.dev) |
| **GitHub** | [kirodotdev/Kiro](https://github.com/kirodotdev/Kiro) |
| **Developer** | Amazon Web Services (AWS) |
| **Preview Launch** | July 15, 2025 (AWS Summit New York) |
| **GA Launch** | November 17, 2025 |
| **Platform** | VS Code (Code OSS) fork, Electron-based |
| **License** | Proprietary (VS Code fork via MIT Code OSS) |
| **Models** | Claude Haiku/Sonnet/Opus, DeepSeek 3.2, MiniMax M2.1, Qwen3 Coder Next |
| **Interfaces** | IDE, CLI |

## Pricing

| Tier | Price | Credits |
|------|-------|---------|
| **Free** | $0 | 50 credits/month |
| **Pro** | $20/month | Included credits |
| **Pro+** | $40/month | More credits |
| **Power** | $200/month | Highest credit allocation |

Credits are variable-cost units — simple prompts cost <1 credit, complex spec tasks cost >1. First-time users get 500 bonus credits within 30 days.

## Architecture

### Spec-Driven Workflow

The core workflow follows three phases:

#### 1. Requirements (EARS Notation)
Kiro transforms prompts into structured requirements using EARS (Easy Approach to Requirements Syntax):
```
WHEN [condition] THE SYSTEM SHALL [expected behavior]
```
Example: `WHEN a user submits invalid form data, THE SYSTEM SHALL display validation errors next to relevant fields.`

#### 2. Design
Technical architecture and implementation approach — system design, tech stack decisions, component architecture.

#### 3. Implementation Tasks
Sequenced coding tasks with dependency ordering. Each task includes unit tests, integration tests, and links back to requirements.

### Workflow Variants

| Variant | Starting Point | Flow |
|---------|---------------|------|
| **Requirements-First** | Behavior description | Requirements → Design → Tasks |
| **Design-First** | Technical architecture | Design → Requirements → Tasks |

### Three Pillars

**Specs** — The requirements → design → tasks pipeline described above.

**Steering** — Markdown files with YAML front matter that provide long-lived project context:
- Architecture decisions
- Coding standards and naming conventions
- Domain rules

Inclusion modes configured via YAML front matter (`always`, `manual`, `automatic`). These live alongside the codebase and are consistently applied when generating code, reviewing changes, or performing automated tasks.

**Hooks** — Event-driven automations written in natural language. "GitHub Actions for your local development environment, powered by AI."

## Hook System

### Trigger Types

| Type | When | Use Case |
|------|------|----------|
| **Pre Tool Use** | Before agent executes a tool | Block/modify actions, validate |
| **Post Tool Use** | After tool execution | Formatting, documentation, logging |
| **File Created** | New file matching pattern | Apply templates, boilerplate |
| **File Saved** | File matching pattern saved | Linting, formatting, tests |
| **File Deleted** | File matching pattern deleted | Cleanup references |
| **Pre Task Execution** | Before spec task begins | Setup scripts, prerequisite validation |
| **Post Task Execution** | After spec task completes | Run tests, lint, notify |
| **Agent Lifecycle** | Agent start/stop | Context loading, cleanup |

## Autonomous Agent

Kiro's autonomous agent operates beyond individual coding sessions:
- Works independently across multiple repositories
- Maintains persistent context between sessions
- Learns from code review feedback
- Integrates with GitHub (creates feature branches, opens PRs with detailed descriptions)
- Integrates with Jira for task management

## Powers

Powers are specialized packages that enhance agents with prebuilt expertise for specific development tasks. HashiCorp is a launch partner. Powers are similar to Antigravity's Skills — they add domain-specific context and tools on demand without overloading the base context.

## Competitive Position

Kiro's spec-driven approach is unique in the market. While Cursor and Claude Code optimize for speed-to-code, Kiro optimizes for correctness-before-code. The trade-off is velocity — Kiro writes zero lines of code in its first phase, which feels slow for quick fixes but produces more reliable results for complex features.

| Aspect | Kiro | Cursor | Claude Code |
|--------|------|--------|-------------|
| **Philosophy** | Spec-driven (requirements first) | Speed-first (edit fast) | Reasoning-first (think deep) |
| **Pre-code** | Requirements + Design + Tasks | None | Optional plan mode |
| **Hooks** | File events, tool events, task events | No | Pre/post tool hooks |
| **Autonomous** | Multi-repo, persistent, async | Background agents | Sub-agents |
| **Platform** | VS Code fork | VS Code fork | Terminal + IDE extensions |
| **Model** | Claude (via AWS), DeepSeek, Qwen | Multi-provider | Claude only |
