# REF: Effective Harnesses for Long-Running Agents (Anthropic)

Full extraction from https://www.anthropic.com/engineering/effective-harnesses-for-long-running-agents
Published November 26, 2025 by Justin Young (Code RL & Claude Code teams)

## The Problem

Agents must work in discrete sessions, each beginning with no memory. "Imagine a software project staffed by engineers working in shifts, where each new engineer arrives with no memory of what happened on the previous shift." Context windows are limited; complex projects cannot be completed in a single window.

## Two Failure Patterns

**Pattern 1 — One-shotting**: Agent tries to do too much at once. Runs out of context mid-implementation. Next session starts with half-implemented, undocumented feature. Agent guesses at what happened, wastes time getting app working again. Compaction doesn't fix this — "doesn't always pass perfectly clear instructions to the next agent."

**Pattern 2 — Premature victory**: After some features are built, a later agent sees progress and declares the job done.

## The Solution: Two-Agent Architecture

**Initializer agent** (first session only): Specialized prompt for environment setup. Creates `init.sh` script, `claude-progress.txt` file, initial git commit.

**Coding agent** (all subsequent sessions): Different prompt focused on incremental progress. Same system prompt, tools, and harness as initializer — only the user prompt differs.

## Feature List (JSON, Not Markdown)

Initializer writes 200+ features as JSON, all marked `passes: false`:

```json
{
    "category": "functional",
    "description": "New chat button creates a fresh conversation",
    "steps": [
      "Navigate to main interface",
      "Click the 'New Chat' button",
      "Verify a new conversation is created",
      "Check that chat area shows welcome state",
      "Verify conversation appears in sidebar"
    ],
    "passes": false
}
```

Coding agents may ONLY change the `passes` field. Strong constraint: "It is unacceptable to remove or edit tests because this could lead to missing or buggy functionality."

JSON chosen over Markdown because models are less likely to inappropriately modify structured data. Determined through experimentation.

## Incremental Progress

Coding agent works on ONE feature at a time. After each change:
- Commit to git with descriptive messages
- Write summary in progress file
- Git enables reverting bad changes and recovering working states

## Testing / Self-Verification

Major failure: Claude marks features complete without proper testing. Would do unit tests or curl commands but miss end-to-end failures.

Fix: Explicitly prompt to use browser automation (Puppeteer MCP) and test "as a human user would." Dramatically improved performance — agent found bugs invisible from code alone.

Limitation: Claude cannot see browser-native alert modals through Puppeteer MCP. Features relying on these modals were buggier.

## Session Initialization Protocol

Every coding agent starts with:

1. `pwd` — see working directory
2. Read git logs and progress files — understand recent work
3. Read feature list — choose highest-priority incomplete feature

Plus: Run `init.sh` to start dev server, then run basic end-to-end test before implementing anything new. In the claude.ai clone case, this meant starting local dev server, using Puppeteer to start a chat, send a message, receive response.

This catches if app was left broken. If agent started implementing a new feature on broken app, it would make things worse.

## Typical Session Startup

```
[Assistant] I'll start by getting my bearings...
[Tool] bash: pwd
[Tool] read: claude-progress.txt
[Tool] read: feature_list.json
[Assistant] Let me check the git log...
[Tool] bash: git log --oneline -20
[Assistant] Now let me check init.sh to restart servers.
<Starts development server>
[Assistant] Let me verify fundamental features still work.
<Tests basic functionality via Puppeteer>
[Assistant] Core features working. Let me review tests.json for next feature.
<Starts work on highest-priority incomplete feature>
```

## Failure Mode Summary Table

| Problem | Initializer Fix | Coding Agent Fix |
|---------|----------------|-----------------|
| Premature victory | JSON feature list, all marked failing | Read feature list first, pick single feature |
| Bugs / undocumented progress | Initial git repo + progress notes file | Read progress + git logs at start, commit + update at end |
| Features marked done prematurely | Feature list with pass/fail | Self-verify with browser automation before marking passing |
| Time wasted on setup | Write init.sh script | Read init.sh at session start |

## Future Work

1. Single agent vs multi-agent — unclear if specialized testing/QA/cleanup agents would outperform
2. Domain generalization — optimized for web apps, likely applicable to research, financial modeling
3. Testing tool limitations — browser automation constraints remain unsolved

## Code Examples

https://github.com/anthropics/claude-quickstarts/tree/main/autonomous-coding

## Related

Claude 4 prompting guide multi-context workflows: https://docs.claude.com/en/docs/build-with-claude/prompt-engineering/claude-4-best-practices#multi-context-window-workflows
