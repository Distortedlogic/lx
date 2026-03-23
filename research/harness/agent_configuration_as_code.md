# Agent Configuration as Code: Declarative Behavior Patterns for AI Agents

As of early 2026, every major coding agent reads a project-level instruction file before doing work. The ecosystem has converged on markdown-based configuration checked into version control, supplemented by structured settings files for permissions and hooks. This document covers practical patterns teams use today.

## 1. Project-Level Instruction Files

Each tool reads its own file, but the formats are converging toward plain markdown with optional YAML frontmatter.

| Tool | File | Format | Notes |
|------|------|--------|-------|
| Claude Code | `CLAUDE.md` | Markdown, `@import` syntax | Hierarchical: `~/`, project root, child dirs |
| OpenAI Codex | `AGENTS.md` | Markdown | `AGENTS.override.md` for temporary overrides |
| GitHub Copilot | `.github/copilot-instructions.md` | Markdown | Also reads `AGENTS.md`, `CLAUDE.md`, `GEMINI.md` |
| Copilot (path-specific) | `.github/instructions/*.instructions.md` | Markdown + YAML frontmatter (`applyTo` globs) | `excludeAgent` field scopes to code-review or coding-agent |
| Cursor | `.cursor/rules/*.mdc` | Markdown + frontmatter | Legacy `.cursorrules` still works. 6000 char limit per file, 12000 total |
| Windsurf | `.windsurf/rules/rules.md` | Markdown | Legacy `.windsurfrules` still works. Four activation modes: Always On, Manual, Model Decision, Auto |
| Aider | `CONVENTIONS.md` + `.aider.conf.yml` | Markdown + YAML | `read: CONVENTIONS.md` in YAML config loads conventions |

**Cross-tool convergence**: AGENTS.md is recognized by 60,000+ repos and read by Codex, Cursor, Copilot, Amp, Windsurf, and others. Teams use AGENTS.md as canonical source and mirror tool-specific sections where needed.

**Arize AI's SWE-bench evaluation**: Optimizing instruction files alone improved GPT-4.1 accuracy by 10-15% on SWE-bench, closing much of the gap with Sonnet without model changes. Sonnet 4-5 gained ~6%, being closer to its ceiling.

## 2. Instruction File Engineering

### Structure that works

Organize by activity, not category. Front-load commands and completion criteria before style preferences:

```markdown
## When Writing Code
- Run `ruff check . --fix` after every change
- Run `pytest -v --tb=short` before committing

## When Reviewing Code
- Run `bandit -r app/` for security checks

## Done Criteria
A task is complete when ALL pass:
1. `ruff check .` exits 0
2. `pytest -v` exits 0 with no failures
3. Commit message follows conventional format
```

### Emphasis tuning

Claude Code docs confirm you can add emphasis like `IMPORTANT` or `YOU MUST` to improve adherence. This works because the model's attention mechanism weights capitalized directives higher.

### Pruning strategy

The official guidance: "For each line, ask: would removing this cause Claude to make mistakes? If not, cut it." Research shows frontier LLMs follow ~150-200 instructions reliably, but the system prompt already consumes ~50. Recommended sizes:
- Root CLAUDE.md: 50-100 lines, with `@imports` for details
- AGENTS.md: under 150 lines total (Codex enforces 32 KiB default)
- Cursor rules: 6000 chars per file, 12000 total

### Principle-based vs prescriptive

Prose without commands produces no measurable behavior change. "We value clean, well-tested code" does nothing. "Run `pytest -v` after every change" does. Research from ICLR 2026 (AMBIG-SWE) found agents default to non-interactive behavior without explicit encouragement, dropping resolve rates from 48.8% to 28%.

## 3. Agent Skills System

The Agent Skills format (agentskills.io) was developed by Anthropic and released as an open standard in late 2025. It is now adopted by Claude Code, Cursor, GitHub Copilot, OpenAI Codex, Windsurf, Gemini CLI, Roo Code, JetBrains Junie, Spring AI, Databricks, Snowflake, and 20+ other tools.

### Progressive disclosure

Skills use three layers to minimize context consumption:

1. **Metadata** (~100 tokens): `name` and `description` loaded at startup for all skills
2. **Instructions** (<5000 tokens recommended): Full `SKILL.md` body loaded on activation
3. **Resources** (as needed): Files in `scripts/`, `references/`, `assets/` loaded on demand

### Skill folder structure

```
.claude/skills/fix-issue/
  SKILL.md          # Required: YAML frontmatter + markdown instructions
  scripts/          # Optional: executable automation
  references/       # Optional: documentation loaded into context
  assets/           # Optional: templates, schemas, data files
```

### SKILL.md format

```markdown
---
name: fix-issue
description: Fix a GitHub issue end-to-end. Use when given an issue number.
allowed-tools: Bash(git:*) Read Write
---
Analyze and fix the GitHub issue: $ARGUMENTS.

1. Use `gh issue view` to get details
2. Search the codebase for relevant files
3. Implement changes and write tests
4. Run tests and linting
5. Commit and create a PR
```

Triggering is pure LLM reasoning -- the agent matches user intent against skill descriptions using its native understanding.

## 4. Hooks and Lifecycle Events

Claude Code supports 17 lifecycle events with four handler types: `command` (shell), `http` (POST), `prompt` (single LLM eval), and `agent` (multi-turn subagent).

### Configuration example (settings.json)

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [{
          "type": "command",
          "command": ".claude/hooks/block-rm.sh"
        }]
      }
    ],
    "PostToolUse": [
      {
        "matcher": "Edit|Write",
        "hooks": [{
          "type": "command",
          "command": ".claude/hooks/lint-check.sh"
        }]
      }
    ]
  }
}
```

Key events: `PreToolUse` (only event that can block actions), `PostToolUse`, `SessionStart`, `Stop`, `SubagentStart/Stop`, `PreCompact`, `InstructionsLoaded`, `ConfigChange`. The handler reads JSON on stdin and optionally returns `permissionDecision: "deny"`. Exit 0 with no output means allow.

Hooks differ from CLAUDE.md: instructions are advisory, hooks are deterministic. Use hooks for invariants.

## 5. Permission and Security Configuration

### Claude Code settings hierarchy

1. **Managed policy** (`/etc/claude-code/managed-settings.json`): Enterprise-wide, cannot be overridden
2. **User settings** (`~/.claude/settings.json`): Personal defaults
3. **Project settings** (`.claude/settings.json`): Shared team config, committed to git
4. **Local settings** (`.claude/settings.local.json`): Personal project overrides, gitignored

If a tool is denied at any level, no other level can allow it.

### Permission patterns

```json
{
  "permissions": {
    "allow": ["Read(./src/**)", "Bash(git *)", "Bash(npm test:*)"],
    "deny": ["Read(**/.env)", "Bash(sudo:*)", "Bash(rm -rf:*)"],
    "ask": ["WebFetch", "Bash(curl:*)"]
  }
}
```

### OpenAI Codex sandbox modes

Codex offers three modes: `read-only`, `workspace-write` (edits within project dirs), and `danger-full-access`. The `approval_policy` setting controls when Codex pauses: `untrusted`, `on-request`, or `never`.

Teams typically start with maximum oversight and relax as trust builds: begin with `ask` on all tools, move known-safe commands to `allow`, use `deny` for hard boundaries.

## 6. Environment-Specific Configuration

### Layering patterns

Claude Code layers config from global to local: `~/.claude/CLAUDE.md` (all sessions) + `./CLAUDE.md` (project) + `./subdir/CLAUDE.md` (subdirectory, loaded on demand). Child dirs override parent dirs.

Codex uses the same model: `~/.codex/AGENTS.md` (global) + walk from git root to CWD, one file per directory level.

### CI/CD vs local

Use `claude -p "prompt"` for non-interactive CI. Pair with `--allowedTools` to scope permissions:

```bash
for file in $(cat files.txt); do
  claude -p "Migrate $file from React to Vue" \
    --allowedTools "Edit,Bash(git commit *)"
done
```

Codex supports profiles in config.toml: `profiles.<name>.*` allows context-dependent overrides for model, sandbox mode, and web search behavior.

## 7. Dynamic Configuration

### Runtime changes

Claude Code fires `ConfigChange` hooks when settings files change during a session, enabling live config updates. The `InstructionsLoaded` hook fires when CLAUDE.md files are loaded, allowing validation or transformation of instructions.

### Feature flags (Codex)

Codex config.toml supports a `[features]` table for experimental capabilities: `features.multi_agent`, `web_search` (disabled/cached/live), `features.unified_exec`. These enable gradual rollout of new behaviors.

### Cursor automations (2026)

Cursor now supports automations: always-on agents triggered by schedules or events from Slack, Linear, GitHub, PagerDuty, and webhooks. Each automation runs in a cloud sandbox with configured rules, MCPs, and model selection.

## 8. Configuration Anti-Patterns

**Rules too long**: When CLAUDE.md exceeds ~200 lines, critical rules get lost in noise. The official diagnostic: "If Claude keeps doing something you don't want despite having a rule against it, the file is probably too long."

**Prose without commands**: "We value clean code" produces zero behavior change. Every rule needs a verifiable action or explicit command.

**Contradictory priorities**: Conflicting rules without explicit hierarchy cause agents to skip verification. AMBIG-SWE research showed a 42% drop in resolve rates from unranked constraints.

**Style guides without enforcement**: Preferences detached from verification commands become suggestions, not rules.

**Ambiguous directives**: "Be careful," "optimize where possible," "handle gracefully" -- none constrain actions.

**Kitchen sink sessions**: Mixing unrelated tasks fills context with noise. Fix: `/clear` between tasks.

**Stale rules**: Rules from a previous codebase version. Treat CLAUDE.md like code: review and prune regularly.

## 9. Configuration Testing and Validation

**Recitation test**: Ask the agent to recite your build commands verbatim. Inability to reproduce them indicates the file is too verbose, too vague, or undiscovered. GitHub's analysis of 2,500 repos found "vagueness, not technical limitations, causes most failures."

**Eval-driven optimization**: Arize AI's prompt learning: generate outputs, evaluate with tests, create feedback explaining why patches succeeded/failed, use a meta-prompt to improve rules. This produced 20-50 optimized rules per project.

**Behavioral observation**: After adding a rule, run a task that would violate it and verify the rule holds.

**Skills validation**: `skills-ref validate ./my-skill` checks SKILL.md programmatically. For Codex: `codex --ask-for-approval never "Summarize current instructions."` echoes loaded guidance.

## 10. Real-World Configuration Examples

### Enterprise managed policy

```json
{
  "env": {
    "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC": 1,
    "DISABLE_TELEMETRY": 1
  },
  "cleanupPeriodDays": 7,
  "permissions": {
    "disableBypassPermissionsMode": "disable",
    "deny": ["Read(**/.env)", "Bash(sudo:*)", "Bash(curl:*)", "Bash(ssh:*)"]
  }
}
```

### Effective AGENTS.md structure (from SWE-bench top performers)

```markdown
## Verification Commands
- Lint: `ruff check . --fix`
- Test: `pytest -v --tb=short`
- Type check: `mypy src/`

## Escalation Rules
If tests fail after 3 attempts: stop and report.
Never: delete files to resolve errors, force push, skip tests.

## When Modifying JavaScript Files
Always run `npm test` after changes.

## Done Criteria
All of: `ruff check .` exits 0, `pytest -v` passes, commit follows conventional format.
```

### Copilot path-specific instructions

```markdown
---
applyTo: "src/api/**/*.ts"
excludeAgent: "code-review"
---
Use Zod for request validation. All endpoints must return typed responses.
Error responses use the ApiError class from src/lib/errors.ts.
```

### Docker cagent YAML (2025)

```yaml
name: code-reviewer
model: gpt-5
instructions: |
  Review code for security issues, performance, and style violations.
tools:
  - name: bash
    allowed_commands: ["rg", "git diff", "git log"]
delegates_to: [security-scanner, style-checker]
```

## Sources

- https://code.claude.com/docs/en/best-practices -- https://code.claude.com/docs/en/hooks
- https://agentskills.io/specification -- https://managed-settings.com/
- https://developers.openai.com/codex/guides/agents-md/ -- https://developers.openai.com/codex/config-reference/
- https://docs.github.com/copilot/customizing-copilot/adding-custom-instructions-for-github-copilot
- https://www.humanlayer.dev/blog/writing-a-good-claude-md
- https://arize.com/blog/optimizing-coding-agent-rules-claude-md-agents-md-clinerules-cursor-rules-for-improved-accuracy/
- https://blakecrosley.com/blog/agents-md-patterns
- https://agentic-patterns.com/patterns/codebase-optimization-for-agents/
- https://leehanchung.github.io/blogs/2025/10/26/claude-skills-deep-dive/
- https://simonwillison.net/guides/agentic-engineering-patterns/anti-patterns/
- https://docs.windsurf.com/windsurf/cascade/memories -- https://aider.chat/docs/usage/conventions.html
- https://www.agentrulegen.com/guides/cursor-rules-guide -- https://cursor.com/changelog
