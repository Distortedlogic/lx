# Google Antigravity: Agent-First IDE Built on Windsurf DNA and Gemini 3

Google Antigravity proves that **the IDE of the agentic era is not an editor with AI bolted on, but a mission-control surface where humans orchestrate autonomous agents across editor, terminal, and browser simultaneously**. Announced November 18, 2025 alongside Gemini 3, Antigravity is a heavily modified VS Code fork that shipped Google's $2.4B Windsurf acquisition team's autonomous coding research at Google-scale infrastructure. Rather than the chat-sidebar pattern of Cursor/Copilot, Antigravity introduces a dual-surface architecture: a traditional Editor view for synchronous coding and a Manager view for spawning, orchestrating, and observing multiple agents working asynchronously across workspaces.

## Overview

| Metric | Value |
|--------|-------|
| **Developer** | Google (informed by Windsurf/Codeium acquisition, July 2025, $2.4B) |
| **Announced** | November 18, 2025 (alongside Gemini 3 launch) |
| **Platform** | VS Code fork (macOS, Windows, Linux) |
| **Default Model** | Gemini 3.1 Pro (2M token context window) |
| **Additional Models** | Claude Sonnet 4.5/4.6, Claude Opus 4.6, GPT-OSS 120B |
| **SWE-bench Verified** | 76.2% (1% behind Claude Sonnet 4.5) |
| **Terminal-Bench 2.0** | 54.2% (exceeds GPT-5.1's 47.6%) |
| **Pricing** | Free public preview (credit-based tiers introduced March 2026) |
| **Download** | antigravity.google/download |

## Architecture

### Three-Surface Model

Antigravity operates through three integrated command surfaces that agents traverse autonomously:

1. **Editor** — AI-enhanced coding with tab completions and inline commands for synchronous work
2. **Terminal** — Agents execute shell commands, launch dev servers, run test suites
3. **Browser** — Agents test in real browsers, take screenshots, validate visual output

An agent can scaffold code → launch a dev server → run E2E tests → identify visual issues → patch fixes → verify results — all without human interruption.

### Dual Interface Design

**Editor View** — The traditional coding surface with AI-powered completions, inline commands, and chat. Handles the synchronous workflow developers already know.

**Manager View** — Mission control for autonomous agents. Spawn multiple agents, assign them to different workspaces, observe progress asynchronously. Agents execute 30-minute tasks independently while humans review results. This restructures the development workday by eliminating constant back-and-forth.

### Development Modes

| Mode | Description | Use Case |
|------|-------------|----------|
| **Agent-Driven** | Complete autonomous execution | Greenfield projects |
| **Agent-Assisted** | Agents propose and execute with verification checkpoints | Recommended default |
| **Review-Driven** | Strict human approval for each step | Critical payment/auth paths |
| **Custom** | Mixed modes per task-specific risk profiles | Enterprise workflows |

## Skills System

The Skills system is Antigravity's extension mechanism for teaching agents domain-specific behavior. A Skill is a directory-based package built around a SKILL.md definition file.

### SKILL.md Format

SKILL.md is an open standard using **Progressive Disclosure** to minimize token costs. The IDE only registers a skill's name and description until invoked — the full instructions load on demand.

```
skill-name/
├── SKILL.md          # Definition file (required)
├── scripts/          # Python, Bash, or Node scripts (optional)
├── references/       # Documentation or templates (optional)
└── assets/           # Static assets like images (optional)
```

Semantic triggering works by comparing the user's natural language input against the `description` field in SKILL.md metadata. When a match is found, the full skill instructions are loaded into context.

### Skill Ecosystem (March 2026)

- 16 specialized agents shipped by Google
- 40+ domain-specific skills covering frontend, backend, testing
- 11 pre-configured commands
- Model Context Protocol (MCP) integration for external service connections
- Community skill libraries (e.g., `antigravity-awesome-skills` — 26K+ GitHub stars, 1,273+ skills)

### Cross-Tool Compatibility

The SKILL.md format has been adopted beyond Antigravity. The Agency Agents project converts skills to formats for Claude Code, Cursor (.mdc), Gemini CLI, OpenCode, OpenClaw, Aider, Windsurf, and Qwen Code.

## Artifacts System

Instead of exposing raw tool call logs, agents produce **Artifacts** — structured deliverables designed for human validation:

- Task lists and implementation plans
- Screenshots and browser recordings
- Annotated code diffs
- Compliance documentation and audit trails

Users provide feedback on Artifacts using a Google Docs-style commenting interface. Agents incorporate feedback without interrupting execution.

## Knowledge Base

Agents maintain persistent learning systems that capture:

- Code patterns and project standards
- Naming conventions and style preferences
- Team workflow patterns

After initial tasks, agents anticipate user style without explicit instruction. This is similar to Claude Code's memory system but integrated at the IDE level.

## Performance

| Benchmark | Antigravity | Competitor |
|-----------|------------|------------|
| SWE-bench Verified | 76.2% | Claude Sonnet 4.5: 77.2% |
| Terminal-Bench 2.0 | 54.2% | GPT-5.1: 47.6% |
| Next.js + Supabase task speed | 42 seconds | Cursor: 68 seconds |
| Large repo navigation | 40% faster than Cursor 2.0 | — |
| Refactoring accuracy | 94% | Cursor: 78% |

## Strengths and Weaknesses

**Excels at:**
- Greenfield projects with well-defined requirements
- Boilerplate-heavy tasks (routing, DB init, CRUD endpoints)
- Feature scaffolding on standard architectures
- Multi-agent parallel workloads (Manager view)
- Projects using standard libraries and conventional patterns

**Struggles with:**
- Legacy codebases with undocumented custom validation libraries
- Homegrown frameworks absent from training data
- Contexts requiring deep institutional knowledge
- Codebase-specific conventions without explicit documentation

## Enterprise Considerations

All code processing occurs on Google's servers — no local-first option. This is a potential disqualifier for fintech, healthcare, government, or IP-sensitive organizations. Enterprise governance features (agent permissions, API access guardrails, decision auditing) remain under development.

## Ecosystem and Community

| Project | Stars | Description |
|---------|-------|-------------|
| `antigravity-awesome-skills` | 26,076 | 1,273+ installable agentic skills with CLI installer |
| `opencode-antigravity-auth` | 9,697 | OAuth bridge letting OpenCode use Antigravity's rate limits |
| `ui-ux-pro-max-skill` | 46,607 | AI SKILL for professional UI/UX design intelligence |

The `opencode-antigravity-auth` project is notable: it enables OpenCode (an open-source CLI agent) to authenticate via Google OAuth and use Antigravity's generous rate limits to access Gemini 3 Pro, Claude Opus 4.6, and other models. This pattern of piggybacking on Antigravity's free tier mirrors what happened with Cursor's API proxy.

## Pricing (March 2026)

| Tier | Price | Details |
|------|-------|---------|
| **Free** | $0/month | All models, rate-limited (~5-hour refresh) |
| **AI Pro** | $20/month | Higher limits + built-in credits |
| **AI Ultra** | $249.99/month | Consistent high-volume access |
| **Credits** | $25 per 2,500 | On-demand purchase |

No BYOK (bring-your-own-key) support. The March 2026 transition from free preview to credit-based pricing triggered significant community backlash — users described the degraded experience as "bait and switch." Deep Think "Thinking Tokens" count against quota without being visible to users.

## Strategic Position

Antigravity represents Google's counter to the Claude Code + Cursor duopoly. The emerging hybrid pattern positions tools as complementary rather than competitive:

- **Antigravity**: New features, architectural scaffolding, multi-agent parallel workflows
- **Cursor**: Surgical edits in critical paths requiring minimal, safe diffs
- **Claude Code**: Terminal-native deep reasoning for complex architectural problems

The Windsurf acquisition ($2.4B for ~40 engineers including CEO Varun Mohan and co-founder Douglas Chen) gave Google years of autonomous coding research implemented at scale. Rather than building from scratch, Antigravity reflects Windsurf's Cascade agent architecture rebuilt on Google infrastructure with Gemini 3's 1M-token context window.
