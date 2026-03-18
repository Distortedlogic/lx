-- Shared tick-loop protocol. All three TICK.md files reference this.
-- Stable boilerplate lives here so TICKs stay focused on variable content.
-- This file IS context. Treat it like any other context file — update, don't delete.

## Sibling Cross-Read Guide

| Your domain | Cross-read | When |
|-------------|------------|------|
| agent/ | brain/TICK.md | When brain/ surfaces lx bugs or needs new features |
| agent/ | workgen/TICK.md | When workgen/ surfaces lx bugs or needs new features |
| brain/ | agent/TICK.md | When you need new language features or hit lx bugs |
| brain/ | workgen/TICK.md | When you want patterns for real lx programs in action |
| workgen/ | agent/TICK.md | When you hit lx bugs or need syntax from LANGUAGE.md |
| workgen/ | brain/TICK.md | When you want patterns for agent.mock or complex lx usage |

Do not modify sibling files unless your task requires it. But when your work
changes what a sibling agent needs to do (new infrastructure, new manifests, new
commands), update their TICK.md with actionable instructions — not just a mention.

## End of Tick Protocol

When you finish (or run out of scope):

### 1. Verify

- `just diagnose` (0 errors)
- `just test` (71/71+)
- `just fmt`
- `lx test -m <your-member>` if your domain has workspace tests
- All files under 300 lines

### 2. Update Context Files

Every file you touched or whose domain changed:
- `DEVLOG.md` (agent/) or `STATUS.md` (brain/) — add session entry
- `BUGS.md` — delete fixed bugs, add new ones
- `INVENTORY.md` — add new features
- `GOTCHAS.md` — add/remove quirks
- `LANGUAGE.md` — update if syntax/semantics changed
- `HEALTH.md` — update if assessment shifted
- Sibling TICK files — if your work affects sibling domains (e.g., new infrastructure)

These files are your memory. The next you has no other way to know what happened.
If a context file doesn't exist yet and you need it, create it. Don't compress context
into fewer files at the cost of losing information.

### 3. Handoff — Rewrite TICK.md

Follow this exact structure:
- **Identity** — keep as-is unless the domain description changed
- **Sibling Domains** — keep as-is (stable)
- **State** — update with current session #, test count, last session summary
- **This Tick** — set to next priority from PRIORITIES.md / STATUS.md
- **Read These Files** — only files needed for that specific task
- **Context Files** — keep as-is unless files were added/removed
- **Rules** — keep as-is (from CLAUDE.md)

Keep TICK.md under 100 lines. **Never delete context to hit a line limit.** If you're
over, factor stable content into domain context files or create new ones. The next agent
reads TICK.md first but can be directed to read additional files. Destroying information
to fit a limit means the next you starts with a corrupted mental model.
