# Agency Agents: A Curated Library of AI Agent Personas for Multi-Tool Orchestration

Agency Agents proves that **structured persona definitions in plain markdown are the most portable unit of AI agent behavior** across the current fragmented landscape of coding assistants. Rather than building a runtime or framework, the project provides 130+ hand-crafted agent personality files with YAML frontmatter and a shell-based conversion pipeline that targets 10 different AI tools. The approach trades execution capability for universal compatibility -- any tool that reads system prompts can consume these agents immediately.

## Repository Overview

| Metric | Value |
|--------|-------|
| **GitHub** | [msitarzewski/agency-agents](https://github.com/msitarzewski/agency-agents) |
| **Stars** | 38,314 |
| **Forks** | 5,759 |
| **Language** | Shell (conversion/install scripts) + Markdown (agent definitions) |
| **License** | MIT |
| **Created** | October 13, 2025 |
| **Last Updated** | March 13, 2026 |
| **Repository Size** | 543 KB |
| **Open Issues** | 27 |
| **Subscribers** | 337 |
| **Agent Count** | ~130 across 12 divisions |
| **Supported Tools** | Claude Code, GitHub Copilot, Cursor, Aider, Windsurf, Gemini CLI, Antigravity, OpenCode, OpenClaw, Qwen Code |

## Architecture

The entire system is markdown files plus shell scripts. There is no runtime, no SDK, no server process.

### Core Components

**Agent Definition Files** -- Each agent is a standalone `.md` file with YAML frontmatter containing `name`, `description`, `color`, `emoji`, and `vibe` fields, followed by a structured markdown body. The body follows a consistent template: Identity & Memory, Core Mission, Critical Rules, Technical Deliverables (with real code examples), Workflow Process, Communication Style, Learning & Memory, Success Metrics, and Advanced Capabilities.

**Conversion Pipeline** (`scripts/convert.sh`) -- A ~400-line bash script that reads all agent files from 12 category directories and converts them into tool-specific formats. Each target tool has its own converter function that transforms frontmatter and body content into the expected format: `.mdc` files for Cursor, `SKILL.md` for Antigravity/Gemini CLI, YAML-frontmattered markdown for OpenCode, split `SOUL.md`/`AGENTS.md`/`IDENTITY.md` for OpenClaw, and concatenated single files for Aider (CONVENTIONS.md) and Windsurf (.windsurfrules).

**Installation Script** (`scripts/install.sh`) -- A ~500-line bash script with an interactive TUI selector that detects installed tools on the host system and copies converted agents to their expected locations. Supports `--tool <name>` for non-interactive use.

**Lint Script** (`scripts/lint-agents.sh`) -- Validates agent files for required frontmatter fields (`name`, `description`, `color`), recommended sections (`Identity`, `Core Mission`, `Critical Rules`), and minimum body length (50 words).

### Directory Layout

| Directory | Agent Count | Focus |
|-----------|------------|-------|
| `engineering/` | 23 | Frontend, backend, AI/ML, DevOps, security, embedded, blockchain |
| `marketing/` | 26 | Content, social media (platform-specific), SEO, growth, China market |
| `specialized/` | 23 | Orchestrator, MCP builder, compliance, identity, blockchain audit |
| `design/` | 8 | UI/UX, brand, visual storytelling, accessibility, whimsy |
| `testing/` | 8 | Evidence-based QA, API testing, performance, accessibility audit |
| `sales/` | 8 | Outbound, discovery, deal strategy, proposals, pipeline analysis |
| `paid-media/` | 7 | PPC, programmatic, social ads, tracking, auditing |
| `support/` | 6 | Customer support, analytics, finance, infrastructure, legal |
| `project-management/` | 6 | Sprint planning, studio ops, Jira workflows, experiment tracking |
| `spatial-computing/` | 6 | VisionOS, WebXR, Metal, cockpit interfaces |
| `game-development/` | 5+ dirs | Designers, audio, technical art + engine-specific (Unity, Unreal, Godot, Roblox) |
| `product/` | 4 | Sprint prioritization, trend research, feedback synthesis, behavioral nudges |
| `strategy/` | 3 + subdirs | Coordination playbooks and runbooks |

## Conversion Pipeline Details

The conversion pipeline (`scripts/convert.sh`) handles 8 distinct output formats with tool-specific transformations:

| Target Tool | Output Format | Key Transformation |
|------------|---------------|-------------------|
| Claude Code | Raw `.md` copied directly | No transformation -- native format |
| GitHub Copilot | Raw `.md` copied directly | Same as Claude Code |
| Cursor | `.mdc` with description/globs/alwaysApply frontmatter | Strips original frontmatter, adds Cursor-specific fields |
| Antigravity | `SKILL.md` with risk/source/date_added frontmatter | Rewrites frontmatter with slug-based naming (`agency-<name>`) |
| Gemini CLI | `SKILL.md` + `gemini-extension.json` manifest | Minimal frontmatter (name + description only) |
| OpenCode | `.md` with hex color codes | Maps named colors to `#RRGGBB` via 20-entry lookup table |
| OpenClaw | Split into `SOUL.md` + `AGENTS.md` + `IDENTITY.md` | Heuristic section routing by header keywords |
| Aider | Single concatenated `CONVENTIONS.md` | All agents merged into one file with `---` separators |
| Windsurf | Single concatenated `.windsurfrules` | All agents merged with `===` separator blocks |
| Qwen Code | `.md` with optional `tools` field | Preserves `${variable}` templating for dynamic context |

The OpenClaw conversion is the most sophisticated. It reads each `##` header, classifies it as persona or operations based on keyword matching (headers containing "identity," "communication," "style," or "critical rule" route to SOUL.md; everything else routes to AGENTS.md), and writes a third `IDENTITY.md` file from the frontmatter's emoji and vibe fields.

## Agent Structure Deep Dive

Each agent file follows a two-part semantic structure that the conversion pipeline exploits:

**Persona sections** (who the agent is): Identity & Memory, Communication Style, Critical Rules. These map to OpenClaw's `SOUL.md` and define the agent's voice, personality, and behavioral constraints.

**Operations sections** (what the agent does): Core Mission, Technical Deliverables, Workflow Process, Success Metrics, Advanced Capabilities. These map to OpenClaw's `AGENTS.md` and contain actionable instructions, code templates, and measurable outcomes.

### Frontmatter Schema

Every agent requires this YAML frontmatter:

| Field | Required | Purpose |
|-------|----------|---------|
| `name` | Yes | Display name used as agent identifier |
| `description` | Yes | One-line summary of the agent's specialty |
| `color` | Yes | Named color or hex code for UI display |
| `emoji` | No | Single emoji for visual identification |
| `vibe` | No | One-line personality hook |
| `services` | No | External API/platform dependencies with name, URL, and pricing tier |
| `tools` | No | Qwen Code-specific tool declarations |

The lint script enforces the three required fields and warns on missing recommended body sections.

### Example: Agents Orchestrator

The most architecturally interesting agent is `specialized/agents-orchestrator.md`. It defines a **multi-agent pipeline manager** with:

- **Phase-based workflow**: PM -> ArchitectUX -> [Developer <-> QA loop] -> Integration
- **Quality gates**: Every task must pass QA validation before the pipeline advances
- **Retry logic**: Maximum 3 attempts per task with specific QA feedback passed back to developer agents
- **Agent spawning protocol**: The orchestrator issues natural-language commands to spawn specialist agents with explicit context and file references
- **Status reporting templates**: Structured markdown templates for pipeline progress and completion summaries
- **Agent catalog**: Lists 50+ available specialist agents categorized by function, enabling the orchestrator to select the right agent for each task type

This is purely prompt-engineering -- the orchestrator has no programmatic control flow. It relies on the LLM interpreting the retry logic, phase transitions, and agent selection instructions.

## Key Design Decisions

**Markdown over code** -- By making every agent a markdown file with YAML frontmatter, the project achieves maximum portability. Any AI tool that accepts system prompts can use these files with zero dependencies. The tradeoff is no runtime enforcement of the behavioral rules.

**No shared state built in** -- Agents are stateless by default. The `examples/workflow-with-memory.md` demonstrates how an MCP memory server can add persistent state, but the core agents don't require it. Context passing between agents relies on the user manually copying outputs or using an external memory server.

**Shell-based tooling** -- The conversion and installation scripts are pure bash with no external dependencies beyond standard POSIX utilities. This makes the project immediately usable on any Unix system without installing a language runtime.

**Personality-first design** -- Each agent has a distinct voice, communication style, and domain perspective rather than being a generic "helpful assistant." The CONTRIBUTING.md explicitly rejects agents that use "I am a helpful assistant" framing. This aligns with research showing that persona-grounded prompts produce more consistent and specialized outputs.

**Quantitative success metrics** -- Each agent defines specific measurable outcomes (e.g., "page load times under 3 seconds on 3G," "Lighthouse scores exceeding 90," "10,000+ combined karma across accounts"). These serve as evaluation criteria, not runtime checks.

## Multi-Agent Coordination Patterns

The project documents three coordination patterns through its example workflows:

### Sequential Handoff (Manual)

Each agent's output is manually copy-pasted as input to the next agent. Used in `examples/workflow-startup-mvp.md` which walks through a 4-week MVP build with 7 agents activated in sequence. The user is the explicit message router.

### Sequential Handoff (Memory-Backed)

Same pattern but with an MCP memory server handling state. Agents store deliverables tagged with project name and receiving agent name, enabling automatic recall. Documented in `examples/workflow-with-memory.md`. Enables rollback to previous checkpoints when QA fails.

### Orchestrated Pipeline

The Agents Orchestrator agent attempts to automate the coordination loop entirely. It maintains pipeline state, spawns specialist agents, enforces quality gates, and handles retry logic -- all through natural language instructions interpreted by the LLM. This is the most ambitious pattern but also the most fragile since it depends entirely on the LLM's ability to follow complex procedural instructions.

| Pattern | State Management | Reliability | Automation |
|---------|-----------------|-------------|------------|
| Manual handoff | User clipboard | High (human in loop) | None |
| Memory-backed | MCP memory server | Medium (depends on tagging) | Partial |
| Orchestrated pipeline | LLM working memory | Low (prompt following) | Full |

## Relationship to Agentic AI Patterns

**Harness Pattern** -- Agency Agents provides the prompt content that harnesses consume. It is not itself a harness but a library of behaviors that any harness (Claude Code, Cursor, etc.) can load. The install script maps agent definitions into each harness's expected directory structure.

**Context Management** -- The project addresses context at the persona level (what an agent remembers and cares about) but delegates runtime context management to the host tool or an MCP memory server. The memory-backed workflow example shows how `remember`, `recall`, and `rollback` operations can bridge sessions.

**Tool Orchestration** -- The MCP Builder agent (`specialized/specialized-mcp-builder.md`) is specifically designed to create MCP servers, providing TypeScript templates and design principles for building tools that extend agent capabilities. This is meta-tooling -- an agent that builds the tools other agents use.

**Multi-Agent Coordination** -- The Agents Orchestrator demonstrates coordination through natural language rather than programmatic APIs. This approach has the advantage of needing no custom runtime but the disadvantage of unreliable execution on complex pipelines. The quality gate and retry logic patterns are well-structured but depend on the LLM faithfully executing multi-step conditional logic.

## Notable Specialist Agents

Beyond the orchestrator, several agents stand out for their architectural relevance:

**MCP Builder** (`specialized/specialized-mcp-builder.md`) -- Provides TypeScript templates for building Model Context Protocol servers with Zod validation, structured output patterns, and stateless tool design. Includes a complete `McpServer` skeleton with `StdioServerTransport`. The agent enforces six critical rules: descriptive tool names, typed parameters, structured output, graceful failure, stateless tools, and real-agent testing.

**Evidence Collector** (`testing/testing-evidence-collector.md`) -- A QA agent that requires screenshot evidence for every validation decision. Defaults to finding 3-5 issues per review cycle. This agent is central to the orchestrator's quality gate pattern.

**Reality Checker** (`testing/testing-reality-checker.md`) -- The final gate in the orchestrated pipeline. Defaults to "NEEDS WORK" unless overwhelming evidence proves production readiness. This pessimistic default is a deliberate design choice to prevent premature shipping.

**Senior Developer** (`engineering/engineering-senior-developer.md`) -- The premium implementation agent, specializing in Laravel/Livewire/FluxUI stack. Demonstrates how agents can encode opinionated technology preferences alongside general engineering principles.

## Strengths

- **Massive breadth**: 130+ agents covering engineering, design, marketing, sales, product, support, testing, spatial computing, and game development -- more domain coverage than any comparable project
- **True portability**: A single source of agent definitions targets 10 different AI tools through format conversion, avoiding vendor lock-in
- **Battle-tested structure**: The consistent template (identity, mission, rules, deliverables, workflow, metrics) produces agents that are specific and actionable rather than vague
- **Low barrier to entry**: Zero dependencies, pure markdown, shell scripts. Anyone can read, modify, or contribute agents
- **Quality guardrails**: Lint script enforces structural consistency, CONTRIBUTING.md enforces design standards, and the template structure prevents lowest-common-denominator agents
- **China market coverage**: Unusual depth in Chinese platform marketing (Xiaohongshu, Douyin, Bilibili, Kuaishou, WeChat, Weibo, Zhihu, Baidu) reflecting real-world demand
- **Community velocity**: 38K+ stars and 5,700+ forks in 5 months indicates strong product-market fit
- **External service declarations**: The `services` frontmatter field lets agents declare API/platform dependencies with pricing tier (free/freemium/paid), enabling automated dependency tracking

## Weaknesses

- **No runtime enforcement**: Behavioral rules, quality gates, and retry logic are purely prompt-based. A sufficiently long context or ambiguous instruction will cause agents to deviate from their defined behavior
- **No composition primitives**: There is no formal mechanism for agents to invoke other agents. The orchestrator pattern relies on the user or LLM interpreting spawn instructions, not programmatic dispatch
- **Stateless by default**: Without an external memory server, all context passing between agents requires manual copy-paste. The memory-backed workflow is documented but not built in
- **Conversion is lossy**: Different tools support different frontmatter fields and body formats. Converting a rich OpenClaw split (SOUL/AGENTS/IDENTITY) back to a single Cursor `.mdc` file loses structural information
- **No testing framework**: The lint script validates structure but cannot test whether an agent actually produces its claimed deliverables. There is no behavioral testing or regression suite
- **Shell scripts at scale**: The convert and install scripts are 400-500 lines of bash each. As the project grows and adds more target tools, these will become increasingly difficult to maintain
- **Orchestrator fragility**: The autonomous pipeline orchestrator is the highest-value agent but also the one most dependent on perfect prompt-following. Complex conditional logic expressed in natural language has inherent reliability limits

## Comparison to Alternative Approaches

| Aspect | Agency Agents | Code-Based Frameworks (e.g., LangGraph, CrewAI) | MCP-Native Agents |
|--------|--------------|------------------------------------------------|-------------------|
| **Definition format** | Markdown + YAML frontmatter | Python/TypeScript classes | JSON schema + tool implementations |
| **Runtime required** | None (consumed by host tool) | Yes (framework runtime) | Yes (MCP server) |
| **Tool portability** | 10 tools via conversion scripts | Single framework | Any MCP client |
| **Composition** | Manual or prompt-based | Programmatic graph/pipeline | Tool chaining via protocol |
| **State management** | None built in | Framework-managed | Server-managed |
| **Behavioral enforcement** | Prompt-based (no guarantees) | Code-enforced | Schema-enforced inputs, prompt-based behavior |
| **Community contribution** | Write a markdown file | Write code + tests | Write server implementation |
| **Agent count** | 130+ | Varies (typically < 20 examples) | Varies |

The fundamental tradeoff: Agency Agents maximizes portability and contribution ease at the cost of execution guarantees. Code-based frameworks maximize reliability at the cost of vendor lock-in.

## Practical Takeaways

**For building agent systems**: The project demonstrates that a well-structured prompt template with consistent sections (identity, rules, deliverables, metrics) produces more useful agents than ad-hoc system prompts. The two-part persona/operations split maps cleanly to different consumption patterns.

**For multi-tool support**: The frontmatter-plus-body format with tool-specific converters is an effective pattern for portable agent definitions. The key insight is that different tools need different metadata but the behavioral content (the markdown body) is largely universal.

**For orchestration**: Natural-language orchestration (as in the Agents Orchestrator) works for simple linear pipelines but breaks down for complex conditional flows. For production reliability, programmatic dispatch and state management are needed on top of the prompt-based behavioral definitions.

**For quality**: Including specific, quantitative success metrics in agent definitions (rather than vague qualitative goals) creates accountability even when those metrics aren't automatically measured. They serve as evaluation rubrics for human reviewers.

**For community growth**: The project's rapid adoption suggests strong demand for curated, portable agent personas. The low contribution barrier (write a markdown file, follow the template) enables community scaling in ways that code-based agent frameworks cannot match.

**For MCP integration**: The memory-backed workflow pattern shows that MCP servers can bridge the gap between stateless prompt-based agents and stateful pipelines. The key operations -- `remember`, `recall`, `rollback`, and tag-based `search` -- form a minimal but sufficient state management interface for multi-agent workflows.

## Sources

- [GitHub Repository: msitarzewski/agency-agents](https://github.com/msitarzewski/agency-agents)
- [README.md](https://github.com/msitarzewski/agency-agents/blob/main/README.md) -- Full agent catalog and project overview
- [CONTRIBUTING.md](https://github.com/msitarzewski/agency-agents/blob/main/CONTRIBUTING.md) -- Agent design guidelines and template specification
- [scripts/convert.sh](https://github.com/msitarzewski/agency-agents/blob/main/scripts/convert.sh) -- Multi-tool format conversion pipeline
- [scripts/install.sh](https://github.com/msitarzewski/agency-agents/blob/main/scripts/install.sh) -- Interactive multi-tool installer
- [scripts/lint-agents.sh](https://github.com/msitarzewski/agency-agents/blob/main/scripts/lint-agents.sh) -- Agent file validation
- [specialized/agents-orchestrator.md](https://github.com/msitarzewski/agency-agents/blob/main/specialized/agents-orchestrator.md) -- Multi-agent pipeline orchestrator
- [specialized/specialized-mcp-builder.md](https://github.com/msitarzewski/agency-agents/blob/main/specialized/specialized-mcp-builder.md) -- MCP server builder agent
- [examples/workflow-startup-mvp.md](https://github.com/msitarzewski/agency-agents/blob/main/examples/workflow-startup-mvp.md) -- Manual multi-agent workflow example
- [examples/workflow-with-memory.md](https://github.com/msitarzewski/agency-agents/blob/main/examples/workflow-with-memory.md) -- Memory-backed workflow with MCP integration
