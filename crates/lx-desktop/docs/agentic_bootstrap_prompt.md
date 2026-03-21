# Agentic Tool Design — Bootstrap Context

Read these files in order to build your understanding of the modern agentic coding tool landscape, what features exist, and what developers actually find valuable. These are first-party research and analysis docs from this repo.

## Step 1: Read the feature matrix
This is the deduplicated, generic feature list extracted from analysis of 20+ agentic coding tools. It covers agent loops, tool use, editing formats, sub-agents, context engineering, security, hooks, model support, reasoning, extensibility, git integration, UI surfaces, terminal features, observability, voice/multimodal, code quality, and scaffolding.

```
Read: docs/agentic_tool_feature_matrix.md
```

## Step 2: Read the developer value rankings
This is what developers actually find useful in practice — synthesized from the Pragmatic Engineer survey (900 engineers), Anthropic's 2026 Agentic Coding Trends Report, real-world developer reviews, Reddit threads, and workflow analyses from Simon Willison and Addy Osmani. Features are tiered by impact, with quantitative data on adoption and effectiveness. Includes anti-patterns (what doesn't work).

```
Read: docs/agentic_feature_developer_value.md
```

## Step 3 (optional): Read individual tool deep-dives for implementation details
These are detailed research files on specific tools. Read whichever are relevant to the task at hand.

```
research/repos/agentic-tools/antigravity.md    — Google's agent-first IDE, Skills/SKILL.md, Manager view
research/repos/agentic-tools/claude-code.md    — TAOR loop, sub-agents, hooks, Agent SDK, memory
research/repos/agentic-tools/codex-cli.md      — Rust-built, V4A patch format, platform-native sandboxing
research/repos/agentic-tools/gemini-cli.md     — 98K stars, GEMINI.md, 4 sandbox methods, A2A protocol
research/repos/agentic-tools/cursor.md         — Two-stage Apply model, background agents, Composer, Fusion
research/repos/agentic-tools/kiro.md           — Spec-driven development, EARS notation, hooks, Powers
research/repos/agentic-tools/amp.md            — Code graph intelligence, 3 agent modes, thread sharing
research/repos/agentic-tools/augment-code.md   — Context Engine (500K files), #1 SWE-bench Pro
research/repos/agentic-tools/devin.md          — Full VM sandbox, 67% PR merge rate, scheduled sessions
research/repos/agentic-tools/windsurf.md       — Cascade agent, SWE-1.5 model, action tracking
research/repos/agentic-tools/github-copilot.md — 15M users, Copilot CLI, coding agent for issues
research/repos/agentic-tools/opencode.md       — Bubble Tea TUI, 126K stars, 3-project lineage
research/repos/agentic-tools/aider.md          — Tree-sitter repo map, 5+ edit formats, Polyglot benchmark
research/repos/agentic-tools/ghostty.md        — Terminal emulator, libghostty, why agents prefer it
research/repos/agentic-tools/emerging-tools.md — Zed, Cline, Roo Code, Trae, Void, PearAI, Qodo, etc.
```
