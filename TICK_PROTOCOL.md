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

## Start of Tick Protocol

When the user tells you to execute a TICK.md, run these steps in order before
writing any code:

1. **Read your TICK.md** — it has your identity, state, task, and reading list.
2. **Read every file in "Read These Files"** — these are required, not optional.
   Do not skim. Do not skip any. These were selected by the previous agent to
   give you exactly the context you need for THIS task.
3. **Verify the State section** — run `just test` and `just diagnose` to confirm
   the claimed test count and clean status are accurate. If they're wrong, fix
   the State section before starting work. Do not build on a false foundation.
4. **Begin task work** — now you have full context. Execute the task described in
   "This Tick." If the task is ambiguous, read PRIORITIES.md or the linked spec
   for clarification. If still ambiguous, ask the user.

## End of Tick Protocol

Execute every step below, in order, before declaring the tick complete. Do not
skip steps. Do not declare completion and then come back to finish. Run the
entire protocol as one uninterrupted sequence at the end of your task work.

### Step 1 — Verify (run ALL of these, no exceptions)

1. `just fmt`
2. `just diagnose` — must show 0 errors (pre-existing warnings OK)
3. `just test` — all tests must pass
4. `lx test -m <your-member>` if your domain has workspace tests
5. Check line counts on every file you created or modified — none may exceed 300

### Step 2 — Update context files (walk the FULL list)

READ every file below using the Read tool. For each one, either update it or
confirm it needs no change. Do not skip a file because you "think" it's
unaffected — you cannot know until you read its current content and compare
against what you shipped. If you did not open a file, you did not review it.

**Required for agent/ domain:**

| File | Read, then... |
|------|---------------|
| `agent/DEVLOG.md` | Add session entry to Session History table |
| `agent/PRIORITIES.md` | Mark shipped items, reorder if needed |
| `agent/INVENTORY.md` | Add any new features, update test/module counts |
| `agent/BUGS.md` | Delete fixed bugs, add new bugs discovered during this session |
| `agent/GOTCHAS.md` | Add new gotchas, remove resolved ones |
| `agent/LANGUAGE.md` | Update if syntax, semantics, module system, or operators changed |
| `agent/HEALTH.md` | Update session number, revise "What's Still Wrong" and "Bottom Line" |
| `agent/REFERENCE.md` | Update if file structure, codebase layout, or how-to patterns changed. Add new how-tos for implementation patterns you had to figure out from scratch |
| `agent/STDLIB.md` | Update if stdlib modules added or changed |
| `agent/AGENTS.md` | Update if agent system or extensions changed |

**Required for brain/ domain:**

| File | Read, then... |
|------|---------------|
| `brain/STATUS.md` | Add session entry |
| `brain/ARCHITECTURE.md` | Update if module structure changed |

**Required for workgen/ domain:**

| File | Read, then... |
|------|---------------|
| `workgen/REFERENCE.md` | Update if file structure or APIs changed |

### Step 3 — Update sibling TICKs (mandatory check)

Ask yourself: "Does my work change what a sibling agent can do or needs to
know?" Examples: new CLI commands, new infrastructure, new stdlib, changed
workspace manifests, fixed bugs they reported, broke something they depend on.

If YES → open each affected sibling TICK.md and add a concrete note (what
changed, what they can now do, what they need to update).

If NO → move on. But you must have actively considered it.

### Step 4 — Rewrite your TICK.md (handoff to next agent)

Follow this exact structure:
- **Identity** — keep as-is unless the domain description changed
- **Sibling Domains** — keep as-is (stable)
- **State** — update: session #, test count, what you shipped this session
- **This Tick** — set to next priority from PRIORITIES.md / STATUS.md
- **Read These Files** — only files the NEXT task needs (not your current task).
  Verify each file you list actually exists and contains relevant content. Do
  not point the next agent at a file you haven't opened.
- **Context Files** — keep as-is unless files were added/removed
- **Rules** — keep as-is (from CLAUDE.md)

Keep TICK.md under 100 lines. **Never delete context to hit a line limit.** If
over, factor stable content into domain context files or create new ones. The
next agent reads TICK.md first but can be directed to read additional files.

### Step 5 — Final confirmation

State which context files you updated and which you reviewed but found no
changes needed. This makes it auditable. Format:

```
Updated: DEVLOG, PRIORITIES, INVENTORY, LANGUAGE, HEALTH, REFERENCE
No change needed: BUGS, GOTCHAS, STDLIB, AGENTS
Sibling TICKs: updated brain/ and workgen/ (cross-member imports)
```
