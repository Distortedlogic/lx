# Goal

Redesign the Agent trait to include identity, context assembly, prompt building, lifecycle hooks, delegation, and health methods. Then update all lx programs (workgen, workrunner, brain) to use the new methods, eliminating boilerplate prompt construction, logging, and context assembly code.

# Why

Every workgen/workrunner agent repeats the same 10-15 line pattern: create Prompt, add sections, render, call think_with, log before, log after, save debug output. 11 of 12 prompt files in workgen/workrunner are single-line system prompt strings (the exception is `workrunner/prompt/context.lx` which has a `load` method that reads files from disk — it is imported by `workrunner/main.lx` and must NOT be deleted). The brain agents scatter identity, health checking, and context management across separate modules. The new trait centralizes these patterns so agents declare what's unique and the trait handles the rest.

# What Changes

**Agent trait full method set after redesign:**

```lx
use std/prompt {new_prompt}

+Trait Agent = {
  role = () { "" }
  goal = () { "" }
  system_prompt = () { "" }

  bootstrap_context = (msg) { {:} }
  examples = () { [] }
  max_context_tokens = () { 100000 }

  init = () { }
  run = () { ... yield loop ... }

  perceive = (msg) { msg }
  reason = (perception) { perception }
  act = (plan) { plan }
  reflect = (result) { result }
  handle = (msg) { ... enriched via bootstrap_context, wrapped in try/on_error ... }

  build_prompt = (msg) { new_prompt (self.system_prompt ()) with examples merged }

  on_turn_start = (msg) { }
  on_turn_end = (msg result) { }
  on_error = (err) { err }

  think = (prompt) { llm.prompt prompt }
  think_with = (config) { ... merges tools/max_turns ... }
  think_structured = (schema_def prompt) { llm.prompt_structured schema_def prompt }

  tools = () { [] }
  max_turns = () { 25 }
  use_tool = (name input) { Err "no tools configured" }

  delegate = (agent_handle task) { agent.ask agent_handle task }
  escalate = (reason) { Err {type: "escalation"; reason: reason} }

  describe = () { {name, role, goal, actions, tools} }
  health = () { {ok: true} }

  ask = (agent_handle msg) { agent.ask agent_handle msg }
  tell = (agent_handle msg) { agent.tell agent_handle msg }
}
```

# Technical Details

### `use std/prompt {new_prompt}` required in agent.lx

`Prompt { system: "..." }` creates a plain Record, not a Prompt Object with methods (render, add_section, etc.). The `new_prompt(system_text)` constructor in std/prompt creates a proper Prompt Object via `Class BasePrompt : [Prompt]`. All agents use `new_prompt` not `Prompt { ... }`. Agent.lx imports `{new_prompt}`. `try` returns raw value on success (not `Ok(value)`), so `handle` match uses `Err e -> ...; r -> ...` (catch-all for success, not `Ok r`). `bootstrap_context` returns `{:}` (empty record literal) not `{}` (which is a block evaluating to Unit).

### `try` is a 2-arg builtin: `try func arg`

`try` takes a function and an argument. It catches `^` (propagate) errors and wraps them as `Err(value)`. It does NOT catch runtime errors. Usage: `try my_func my_arg`. NOT `try { block }`.

The `handle` method extracts the OODA chain into a function to use try:

```lx
handle = (msg) {
  enriched = {..msg; ..(self.bootstrap_context msg)}
  self.on_turn_start enriched
  ooda = (m) {
    action = m.action
    method = action != None ? (method_of self action) : None
    method != None ? (method m) : {
      self.reflect (self.act (self.reason (self.perceive m)))
    }
  }
  result = try ooda enriched
  result ? {
    Ok r -> { self.on_turn_end enriched r; r }
    Err e -> self.on_error e
  }
}
```

### `on_turn_end` takes TWO args: `(msg result)`

Logging needs both msg (for `msg.runner_dir`, `msg.slug`) and result (for `result.text`, `result.turns`). Override example:
```lx
on_turn_end = (msg result) {
  log.save_debug msg.runner_dir msg.slug "investigate.md" result.text
  log.log "  < done ({result.text | lines | len} lines)"
}
```

### `bootstrap_context` is forward-looking

Current workgen agents don't use it — they read msg fields directly in build_prompt. Current workrunner agents receive `msg.bootstrap_prompt` from the orchestrator and compose it in build_prompt via `p.compose msg.bootstrap_prompt`. The method exists for future agents that need to dynamically fetch context before prompt construction. Current agents leave the default `(msg) { {} }`.

### `act` default stays as pass-through `(plan) { plan }`

Do NOT change act's default. Agents override act and call `self.build_prompt msg` inside their override. The trait provides build_prompt as a helper, not an automatic pipeline step.

### `workrunner/prompt/context.lx` must NOT be deleted

This file has a `load` method that reads agent context files from disk (`agent/TICK.md`, `agent/INVENTORY.md`, `agent/REFERENCE.md`, `agent/STDLIB.md`). It is imported by `workrunner/main.lx` at line 12: `use ./prompt/context {BootstrapContext}`. Keep it.

### Prompt files that ARE simple system strings (deletable)

All verified to contain only `+Prompt Name = { system: "..." }` with no other methods:

- `workgen/prompt/investigate.lx` → `"You are a code auditor. Investigate the codebase thoroughly — read files, search for patterns. Produce a numbered findings list."`
- `workgen/prompt/compose.lx` → `"You are a markdown document generator. Output the document directly. Start with '# Goal'. No conversation."`
- `workgen/prompt/revise.lx` → `"You are revising a work item document. Fix every feedback item. Output the complete revised document."`
- `workgen/prompt/validate.lx` → `"You are validating a proposed approach before work item generation. Evaluate whether the solution is best practice, idiomatic, and sound."`
- `workgen/prompt/grade.lx` → `"You are a grader scoring work against a rubric. For each category, assign a score 0-100 and brief feedback. A category passes at score >= 70."`
- `workrunner/prompt/implement.lx` → `"You are implementing one task from a work item. Follow the task description exactly. Use justfile recipes. No code comments. 300 line file limit."`
- `workrunner/prompt/fix.lx` → `"Fix the issues found in this task. Fix ONLY what the grader flagged."`
- `workrunner/prompt/fix_wi.lx` → `"Fix issues found in work item grading. Fix ONLY what failed."`
- `workrunner/prompt/investigate.lx` → `"Investigate whether a work item is fully implemented. Check code quality and spec compliance. Be terse — under 2000 chars."`
- `workrunner/prompt/system_audit.lx` → `"You are running a final system-wide audit after all work items have been implemented. Check code quality across the full codebase and verify spec compliance for every executed work item."`
- `workrunner/prompt/grade.lx` → `"You are a grader scoring work against a rubric. For each category, assign a score 0-100 and brief feedback. A category passes at score >= 70."`

### Grader agents already have `build_prompt`

Both workgen and workrunner Grader agents define their own `build_prompt` method. The Agent trait's default `build_prompt` gets overridden. No name collision.

### Messages are always Records

All orchestrator call sites pass Records: `investigator.act {audit_content: ...; root: ...}`, `implementer.act {wi: ...; task: ...}`, etc. The `{..msg; ..(self.bootstrap_context msg)}` spread merge is safe.

### brain agents have in-file Prompt declarations

Brain agents define `Prompt PlanningPrompt = { ... }`, `Prompt AnalysisPrompt = { ... }` etc. in the same file as the agent. These are used by specialist methods that build custom prompts. Do NOT delete them — they're in-file declarations, not separate prompt files.

### brain dispatcher `~>?` syntax now works

The `~>` and `~>?` operators are implemented (TELL_ASK_OPERATORS work item). `dispatcher.lx` uses `~>?` which now lexes and desugars correctly to `agent.ask`. No changes needed to dispatcher for operator syntax.

# Files Affected

- `crates/lx/std/agent.lx` — Rewrite trait with new methods + `use std/prompt`
- `programs/workgen/agent/investigator.lx` — Move system prompt inline, override build_prompt/hooks, simplify act
- `programs/workgen/agent/composer.lx` — Same
- `programs/workgen/agent/reviser.lx` — Same
- `programs/workgen/agent/validator.lx` — Same
- `programs/workgen/agent/grader.lx` — Add system_prompt, keep existing build_prompt/parse/assemble
- `programs/workrunner/agent/implementer.lx` — Same pattern as workgen
- `programs/workrunner/agent/fixer.lx` — Same (both TaskFixer and WorkItemFixer)
- `programs/workrunner/agent/investigator.lx` — Same
- `programs/workrunner/agent/system_auditor.lx` — Same
- `programs/workrunner/agent/grader.lx` — Same as workgen grader
- `programs/brain/agents/planner.lx` — Add role/goal/system_prompt
- `programs/brain/agents/analyst.lx` — Same
- `programs/brain/agents/researcher.lx` — Same
- `programs/brain/agents/critic.lx` — Same
- `programs/brain/agents/synthesizer.lx` — Same
- `programs/brain/agents/monitor.lx` — Wire health() to existing check_health
- `programs/brain/agents/dispatcher.lx` — Add role/goal/system_prompt, use delegate()
- `programs/workgen/prompt/investigate.lx` — DELETE
- `programs/workgen/prompt/compose.lx` — DELETE
- `programs/workgen/prompt/revise.lx` — DELETE
- `programs/workgen/prompt/validate.lx` — DELETE
- `programs/workgen/prompt/grade.lx` — DELETE
- `programs/workrunner/prompt/implement.lx` — DELETE
- `programs/workrunner/prompt/fix.lx` — DELETE
- `programs/workrunner/prompt/fix_wi.lx` — DELETE
- `programs/workrunner/prompt/investigate.lx` — DELETE
- `programs/workrunner/prompt/system_audit.lx` — DELETE
- `programs/workrunner/prompt/grade.lx` — DELETE
- `programs/workrunner/prompt/context.lx` — KEEP
- `tests/keywords.lx` — Update Agent test to verify new methods

# Task List

### Task 1: Rewrite Agent trait

**Subject:** Add new methods to Agent trait in crates/lx/std/agent.lx

**Description:** Rewrite `crates/lx/std/agent.lx`.

Add `use std/prompt {new_prompt}` at the top of the file.

New methods to add with their defaults:
- `role = () { "" }`
- `goal = () { "" }`
- `system_prompt = () { "" }`
- `bootstrap_context = (msg) { {} }`
- `examples = () { [] }`
- `max_context_tokens = () { 100000 }`
- `build_prompt = (msg) { p = new_prompt (self.system_prompt ()); exs = self.examples (); (exs | len) > 0 ? { p.examples <- exs }; p }`
- `on_turn_start = (msg) { }`
- `on_turn_end = (msg result) { }`
- `on_error = (err) { err }`
- `delegate = (agent_handle task) { agent.ask agent_handle task }`
- `escalate = (reason) { Err {type: "escalation"; reason: reason} }`
- `health = () { {ok: true} }`

Update `handle`:
```lx
handle = (msg) {
  enriched = {..msg; ..(self.bootstrap_context msg)}
  self.on_turn_start enriched
  ooda = (m) {
    action = m.action
    method = action != None ? (method_of self action) : None
    method != None ? (method m) : {
      self.reflect (self.act (self.reason (self.perceive m)))
    }
  }
  result = try ooda enriched
  result ? {
    Ok r -> { self.on_turn_end enriched r; r }
    Err e -> self.on_error e
  }
}
```

Update `describe`:
```lx
describe = () {
  {name: to_str self; role: self.role (); goal: self.goal (); actions: methods_of self; tools: self.tools ()}
}
```

Keep ALL existing methods unchanged: init, run, perceive, reason, act, reflect, think, think_with, think_structured, tools, max_turns, use_tool, ask, tell. Do NOT change act's default.

**ActiveForm:** Rewriting Agent trait

---

### Task 2: Update workgen agents and delete prompt files

**Subject:** Simplify 5 workgen agents, delete 5 prompt files

**Description:** For each agent in `programs/workgen/agent/`:

Transformation pattern for each agent:
1. Remove the `use ../prompt/... {SomePrompt}` import
2. Add `use std/prompt {new_prompt}` (needed by build_prompt)
3. Add `system_prompt = () { "the system string from the deleted prompt file" }`
4. Add `build_prompt = (msg) { p = new_prompt (self.system_prompt ()); p.add_section ...; p.add_instruction ...; p.add_constraint ...; p }` — move section/instruction/constraint additions from act into build_prompt
5. Add `on_turn_start = (msg) { log.log "  > claude: ..." }` with the agent's current pre-work log message
6. Add `on_turn_end = (msg result) { log.save_debug ...; log.log "  < done ..." }` with the agent's current post-work log/save_debug calls
7. Simplify `act` to: `p = self.build_prompt msg; result = self.think_with {prompt: (p.render ())} ^; result.text` (or whatever the agent returns)

**investigator.lx**: system_prompt = `"You are a code auditor. Investigate the codebase thoroughly — read files, search for patterns. Produce a numbered findings list."`. build_prompt adds "Audit Checklist" section + 4 instructions + 2 constraints from msg. on_turn_end saves to "investigate.md".

**composer.lx**: system_prompt = `"You are a markdown document generator. Output the document directly. Start with '# Goal'. No conversation."`. build_prompt adds "Audit Checklist", "Process Rules", "Investigation Findings" sections + 2 instructions + 2 constraints. on_turn_end saves to "compose.md".

**reviser.lx**: system_prompt = `"You are revising a work item document. Fix every feedback item. Output the complete revised document."`. build_prompt adds "Current Document", "Feedback to Fix" sections + 1 instruction + 2 constraints. on_turn_end saves to "revise_round_{msg.round_num}.md".

**validator.lx**: system_prompt = `"You are validating a proposed approach before work item generation. Evaluate whether the solution is best practice, idiomatic, and sound."`. build_prompt adds "Audit Findings", "Codebase Root" sections + 4 instructions + 2 constraints. on_turn_end saves to "validate.md".

**grader.lx**: system_prompt = `"You are a grader scoring work against a rubric. For each category, assign a score 0-100 and brief feedback. A category passes at score >= 70."`. Keep existing build_prompt/grading_schema/parse_response/assemble/empty_result. Grader has no logging to move.

After updating all agents, delete these 5 files:
- `programs/workgen/prompt/investigate.lx`
- `programs/workgen/prompt/compose.lx`
- `programs/workgen/prompt/revise.lx`
- `programs/workgen/prompt/validate.lx`
- `programs/workgen/prompt/grade.lx`

**ActiveForm:** Simplifying workgen agents

---

### Task 3: Update workrunner agents and delete prompt files

**Subject:** Simplify 6 workrunner agents, delete 6 prompt files

**Description:** Same transformation as Task 2 for `programs/workrunner/agent/`.

**implementer.lx**: system_prompt = `"You are implementing one task from a work item. Follow the task description exactly. Use justfile recipes. No code comments. 300 line file limit."`. build_prompt does `p.compose msg.bootstrap_prompt` then adds "Work Item Context" (from `context_for_implement msg.wi`), "Current Task" (from `task_context msg.task`) sections + 1 instruction + 2 constraints. Keep `use ../schema/work_item {task_context context_for_implement}` import. on_turn_end saves to "task_{msg.task.num}_implement.md".

**fixer.lx (TaskFixer)**: system_prompt = `"Fix the issues found in this task. Fix ONLY what the grader flagged."`. build_prompt does `p.compose msg.bootstrap_prompt` then adds "Task" (from `task_context msg.task`), "Grader Feedback" sections + 1 instruction + 1 constraint. on_turn_end saves to "task_{msg.task.num}_fix.md".

**fixer.lx (WorkItemFixer)**: system_prompt = `"Fix issues found in work item grading. Fix ONLY what failed."`. build_prompt does `p.compose msg.bootstrap_prompt` then adds "Work Item" (from `context_for_investigate msg.wi`), "Grader Feedback", "Failed" sections + 1 instruction. on_turn_end saves to "fix_wi.md".

**investigator.lx**: system_prompt = `"Investigate whether a work item is fully implemented. Check code quality and spec compliance. Be terse — under 2000 chars."`. build_prompt adds "Work Item" section + 6 instructions + 4 constraints. on_turn_end saves to "investigate.md".

**system_auditor.lx**: system_prompt = `"You are running a final system-wide audit after all work items have been implemented. Check code quality across the full codebase and verify spec compliance for every executed work item."`. build_prompt adds "Completed Work Items" section + 5 instructions + 2 constraints. on_turn_end saves to "system_audit.md".

**grader.lx**: system_prompt = `"You are a grader scoring work against a rubric. For each category, assign a score 0-100 and brief feedback. A category passes at score >= 70."`. Keep existing build_prompt/grading_schema/parse_response/assemble/empty_result. No logging to move.

After updating agents, delete these 6 files:
- `programs/workrunner/prompt/implement.lx`
- `programs/workrunner/prompt/fix.lx`
- `programs/workrunner/prompt/fix_wi.lx`
- `programs/workrunner/prompt/investigate.lx`
- `programs/workrunner/prompt/system_audit.lx`
- `programs/workrunner/prompt/grade.lx`

DO NOT delete `programs/workrunner/prompt/context.lx`.

**ActiveForm:** Simplifying workrunner agents

---

### Task 4: Update brain agents

**Subject:** Add role/goal/system_prompt to brain agents, wire health and delegate

**Description:** For each agent in `programs/brain/agents/`:

**planner.lx (TaskPlanner)** — Add:
- `role = () { "task decomposition and planning specialist" }`
- `goal = () { "create precise, minimal execution plans" }`
- `system_prompt = () { "You create precise, minimal execution plans." }`
Keep all specialist methods (plan, replan, estimate_cost) unchanged. Keep in-file PlanningPrompt/ReplanPrompt declarations.

**analyst.lx (DeepAnalyst)** — Add role/goal/system_prompt from AnalysisPrompt system string: `"You are an analytical specialist. You find what others miss."`. Keep analyze/compare/investigate. Keep in-file Prompt declarations.

**researcher.lx (InfoGatherer)** — Add role = `"information gathering specialist"`, goal = `"search, read, and synthesize findings"`, system_prompt from SynthesisPrompt: `"You synthesize research findings into a clear answer."`. Keep research/search.

**critic.lx (InnerCritic)** — Add role = `"inner critic"`, goal = `"challenge assumptions, find flaws, prevent overconfidence"`, system_prompt from CritiquePrompt: `"You are a rigorous critic. Find flaws, gaps, and risks. Be honest, not kind."`. Keep critique/challenge/verify_code.

**synthesizer.lx (ResponseSynth)** — Add role = `"response synthesis specialist"`, goal = `"combine reasoning, results, and identity into coherent output"`, system_prompt: `"You synthesize responses."`. Keep synthesize/format/compose_parts.

**monitor.lx** — NOT APPLICABLE. This file has no Agent declaration — it is a standalone function module exporting `check_health`, `tick`, etc. Cannot add Agent trait overrides to a non-Agent. Skip.

**dispatcher.lx** — NOT APPLICABLE. This file has no Agent declaration — it is a standalone function module exporting `dispatch_task`, `fan_out`, etc. The `~>?` operator calls work correctly. Cannot add Agent trait overrides to a non-Agent. Skip.

Do NOT delete any in-file Prompt/Schema declarations in brain agents.

**ActiveForm:** Updating brain agents

---

### Task 5: Update keyword test

**Subject:** Verify new Agent methods exist in keyword test

**Description:** In `tests/keywords.lx`, replace the existing Agent section with:

```lx
Agent TestAgent = {
  role = () { "test agent" }
  goal = () { "verify trait methods" }
  system_prompt = () { "You are a test." }
  perceive = (msg) { {intent: msg} }
}

a = TestAgent {}
assert (a.perceive "hello").intent == "hello"
assert (methods_of a | any? (== "think"))
assert (methods_of a | any? (== "handle"))
assert (methods_of a | any? (== "build_prompt"))
assert (methods_of a | any? (== "health"))
assert (methods_of a | any? (== "role"))
assert (methods_of a | any? (== "delegate"))
assert (methods_of a | any? (== "escalate"))
assert (methods_of a | any? (== "on_turn_start"))
assert (methods_of a | any? (== "on_error"))
assert (methods_of a | any? (== "bootstrap_context"))
assert (a.role () == "test agent")
assert (a.goal () == "verify trait methods")
assert (a.health ().ok == true)
```

The existing Agent section starts at the `-- Agent` comment and ends before `-- Tool`. Replace only that section.

**ActiveForm:** Updating keyword test

---

### Task 6: Run full test suite and fix regressions

**Subject:** Verify no regressions, fix any failures

**Description:**

1. Run `just rust-diagnose` — fix any compile errors.
2. Run `just test` — fix any test failures.
3. Run `cargo test -p lx --test formatter_roundtrip` — verify formatter roundtrip.

Likely failure sources:
- Missing `use std/prompt {new_prompt}` in agent files that create Prompt instances in build_prompt
- `on_turn_end` signature: agents must pass 2 args, callers in handle pass `enriched` and `r`
- `handle`'s try pattern: `try ooda enriched` — ooda must be a function accepting one arg
- Deleted prompt file imports not removed from agent files
- `try` only catches `^` errors — if an agent's act method throws a runtime error (not `^`), it won't be caught by on_error. This is by design — runtime errors are bugs, not expected failures.

**ActiveForm:** Full regression testing

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

1. **Call `complete_task` after each task.**
2. **Call `next_task` to get the next task.**
3. **Do not add, skip, reorder, or combine tasks.**
4. **Tasks are implementation-only.**
5. **Do not add code comments or doc strings** (exception: `--` header comments on .lx program files).
6. **Use `just rust-diagnose` not raw cargo commands.**
7. **`try` takes 2 args: `try func arg`.** Extract OODA into `ooda = (m) { ... }` then call `try ooda enriched`.
8. **`on_turn_end` takes 2 args: `(msg result)`.** Handle passes `self.on_turn_end enriched r`.
9. **DO NOT delete `workrunner/prompt/context.lx`** — has load method, imported by main.lx.
10. **`act` default stays as `(plan) { plan }`.** Do not change it.
11. **`use std/prompt {new_prompt}` must be added** to agent.lx and to each agent file that creates Prompt in build_prompt.
12. **Brain agent in-file Prompt declarations** (PlanningPrompt, AnalysisPrompt, etc.) must NOT be deleted.
13. **`~>?` in dispatcher.lx now works** — the operators are implemented. Do not replace them.

---

## Task Loading Instructions

```
mcp__workflow__load_work_item({ path: "work_items/AGENT_TRAIT_REDESIGN.md" })
```
