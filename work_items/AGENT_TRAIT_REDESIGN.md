# Goal

Redesign the Agent trait to include identity, context assembly, prompt building, lifecycle hooks, delegation, and health methods. Then update all lx programs (workgen, workrunner, brain) to use the new methods, eliminating boilerplate prompt construction, logging, and context assembly code.

# Why

Every workgen/workrunner agent repeats the same 10-15 line pattern: create Prompt, add sections, render, call think_with, log before, log after, save debug output. 11 of 12 prompt files in workgen/workrunner are single-line system prompt strings (the exception is `workrunner/prompt/context.lx` which has a `load` method). The brain agents scatter identity, health checking, and context management across separate modules. The new trait centralizes these patterns so agents declare what's unique (their system prompt, their sections, their tools) and the trait handles the rest.

# What Changes

**Agent trait (`crates/lx/std/agent.lx`)** — new default methods:

```lx
use std/prompt {Prompt}

+Trait Agent = {
  role = () { "" }
  goal = () { "" }
  system_prompt = () { "" }

  bootstrap_context = (msg) { {} }
  examples = () { [] }
  max_context_tokens = () { 100000 }

  init = () { }
  run = () { ... }

  perceive = (msg) { msg }
  reason = (perception) { perception }
  act = (plan) { plan }
  reflect = (result) { result }
  handle = (msg) { ... }

  build_prompt = (msg) { ... }

  on_turn_start = (msg) { }
  on_turn_end = (msg result) { }
  on_error = (err) { err }

  think = (prompt) { ... }
  think_with = (config) { ... }
  think_structured = (schema_def prompt) { ... }

  tools = () { [] }
  max_turns = () { 25 }
  use_tool = (name input) { Err "no tools configured" }

  delegate = (agent_handle task) { agent.ask agent_handle task }
  escalate = (reason) { Err {type: "escalation"; reason: reason} }

  describe = () { ... }
  health = () { {ok: true} }

  ask = (agent_handle msg) { ... }
  tell = (agent_handle msg) { ... }
}
```

# Technical Details

### `use std/prompt` required

`agent.lx` must add `use std/prompt {Prompt}` at the top. The `build_prompt` default creates `Prompt { system: self.system_prompt () }`. Without the import, `Prompt` is not in scope. No circular dependency — prompt.lx does not import agent.lx.

### `try` is a 2-arg builtin: `try func arg`

`try` takes a function and an argument. It catches `^` (propagate) errors and wraps them as `Err(value)`. It does NOT catch runtime errors (those still crash). Usage: `result = try my_func my_arg`. NOT `try { block }`.

The `handle` method must extract the OODA chain into a function to use try:

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

The logging pattern in workgen/workrunner agents needs both msg (for `msg.runner_dir`, `msg.slug`) and result (for `result.text`, `result.turns`). Signature must be `on_turn_end = (msg result) { }`, not `(result)`.

Example override:
```lx
on_turn_end = (msg result) {
  log.save_debug msg.runner_dir msg.slug "investigate.md" result.text
  log.log "  < done ({result.text | lines | len} lines)"
}
```

### `bootstrap_context` is forward-looking

Current workgen agents don't use it — they read msg fields directly in build_prompt. Current workrunner agents receive `msg.bootstrap_prompt` from the orchestrator and compose it in build_prompt via `p.compose msg.bootstrap_prompt`.

The method exists for future agents that need to dynamically fetch context (read files, query stores, check git state) before prompt construction. Current agents can leave the default `(msg) { {} }`.

### `act` default stays as pass-through

Do NOT change act's default to call build_prompt+think_with. Agents override act and call `self.build_prompt msg` inside their override. The trait provides build_prompt as a helper, not as an automatic pipeline step.

### `workrunner/prompt/context.lx` must NOT be deleted

This file has a `load` method that reads agent context files from disk (`agent/TICK.md`, `agent/INVENTORY.md`, etc.). It is imported by `workrunner/main.lx`. It is NOT a simple system string. Keep it.

The 11 files that ARE simple system strings and CAN be deleted:
- `workgen/prompt/investigate.lx` — `system: "You are a code auditor..."`
- `workgen/prompt/compose.lx` — `system: "You are a markdown document generator..."`
- `workgen/prompt/revise.lx` — `system: "You are revising a work item document..."`
- `workgen/prompt/validate.lx` — `system: "You are validating a proposed approach..."`
- `workgen/prompt/grade.lx` — `system: "You are a grader scoring work against a rubric..."`
- `workrunner/prompt/implement.lx` — `system: "You are implementing one task..."`
- `workrunner/prompt/fix.lx` — `system: "Fix the issues found in this task..."`
- `workrunner/prompt/fix_wi.lx` — `system: "Fix issues found in work item grading..."`
- `workrunner/prompt/investigate.lx` — `system: "Investigate whether a work item is fully implemented..."`
- `workrunner/prompt/system_audit.lx` — `system: "You are running a final system-wide audit..."`
- `workrunner/prompt/grade.lx` — `system: "You are a grader scoring work against a rubric..."`

### brain dispatcher `~>?` syntax does not work

The `~>?` and `~>` operators were never added to the lexer. `~` lexes as `Tilde→Bang`. Lines in `dispatcher.lx` using `~>?` are non-functional dead code. Replace with `self.delegate agent_handle msg` or `agent.ask agent_handle msg`.

### Grader agents already have `build_prompt`

Both workgen and workrunner Grader agents define their own `build_prompt` method. The Agent trait adds a default `build_prompt` — the grader's existing method overrides it. No name collision; lx class method overrides work correctly.

### Messages are always Records

All orchestrator call sites pass Records to `.act`: `investigator.act {audit_content: ...; root: ...}`, `implementer.act {wi: ...; task: ...}`, etc. The `{..msg; ..(self.bootstrap_context msg)}` spread merge is safe because msg is always a Record.

# Files Affected

- `crates/lx/std/agent.lx` — Rewrite trait with new methods + `use std/prompt`
- `programs/workgen/agent/investigator.lx` — Move system prompt inline, override build_prompt/on_turn_start/on_turn_end, simplify act
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
- `programs/brain/agents/dispatcher.lx` — Replace `~>?` dead code with delegate/agent.ask
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
- `programs/workrunner/prompt/context.lx` — KEEP (has load method, imported by main.lx)
- `tests/keywords.lx` — Update Agent test to verify new methods

# Task List

### Task 1: Rewrite Agent trait

**Subject:** Add new methods to Agent trait in crates/lx/std/agent.lx

**Description:** Rewrite `crates/lx/std/agent.lx`.

Add `use std/prompt {Prompt}` at the top of the file.

New methods to add with their defaults:
- `role = () { "" }`
- `goal = () { "" }`
- `system_prompt = () { "" }`
- `bootstrap_context = (msg) { {} }`
- `examples = () { [] }`
- `max_context_tokens = () { 100000 }`
- `build_prompt = (msg) { p = Prompt { system: self.system_prompt () }; exs = self.examples (); (exs | len) > 0 ? { p.examples <- exs }; p }`
- `on_turn_start = (msg) { }`
- `on_turn_end = (msg result) { }`
- `on_error = (err) { err }`
- `delegate = (agent_handle task) { agent.ask agent_handle task }`
- `escalate = (reason) { Err {type: "escalation"; reason: reason} }`
- `health = () { {ok: true} }`

Update `handle` — extract OODA into a function for `try`:
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

Keep ALL existing methods unchanged: init, run, perceive, reason, act, reflect, think, think_with, think_structured, tools, max_turns, use_tool, ask, tell. Do NOT change act's default — it stays as `(plan) { plan }`.

**ActiveForm:** Rewriting Agent trait

---

### Task 2: Update workgen agents and delete prompt files

**Subject:** Simplify 5 workgen agents, delete 5 prompt files

**Description:** For each agent in `programs/workgen/agent/`, apply this transformation:

1. Remove the `use ../prompt/... {SomePrompt}` import
2. Add `system_prompt = () { "the system string from the deleted prompt file" }`
3. Move prompt section/instruction/constraint additions from `act` into `build_prompt = (msg) { p = Prompt { system: self.system_prompt () }; p.add_section ...; p }`
4. Add `on_turn_start = (msg) { log.log "  > claude: doing thing..." }` with the agent's current log message
5. Add `on_turn_end = (msg result) { log.save_debug msg.runner_dir msg.slug "filename.md" result.text; log.log "  < done ..." }` with the agent's current log/save_debug calls
6. Simplify `act` to: build prompt, call think_with, extract text/result

Specific system prompt strings to inline (copy exactly from the prompt files):
- **investigator.lx**: `"You are a code auditor. Investigate the codebase thoroughly — read files, search for patterns. Produce a numbered findings list."`
- **composer.lx**: `"You are a markdown document generator. Output the document directly. Start with '# Goal'. No conversation."`
- **reviser.lx**: `"You are revising a work item document. Fix every feedback item. Output the complete revised document."`
- **validator.lx**: `"You are validating a proposed approach before work item generation. Evaluate whether the solution is best practice, idiomatic, and sound."`
- **grader.lx**: `"You are a grader scoring work against a rubric. For each category, assign a score 0-100 and brief feedback. A category passes at score >= 70."` — grader already has its own `build_prompt` method, just add `system_prompt` and keep everything else.

After updating all agents, delete these 5 files:
- `programs/workgen/prompt/investigate.lx`
- `programs/workgen/prompt/compose.lx`
- `programs/workgen/prompt/revise.lx`
- `programs/workgen/prompt/validate.lx`
- `programs/workgen/prompt/grade.lx`

Add `use std/prompt {Prompt}` to each agent file that creates Prompt instances in build_prompt (previously the Prompt came from the prompt file import).

**ActiveForm:** Simplifying workgen agents

---

### Task 3: Update workrunner agents and delete prompt files

**Subject:** Simplify 6 workrunner agents, delete 6 prompt files

**Description:** Same transformation as Task 2 for `programs/workrunner/agent/`.

System prompt strings to inline:
- **implementer.lx**: `"You are implementing one task from a work item. Follow the task description exactly. Use justfile recipes. No code comments. 300 line file limit."`
- **fixer.lx (TaskFixer)**: `"Fix the issues found in this task. Fix ONLY what the grader flagged."`
- **fixer.lx (WorkItemFixer)**: `"Fix issues found in work item grading. Fix ONLY what failed."`
- **investigator.lx**: `"Investigate whether a work item is fully implemented. Check code quality and spec compliance. Be terse — under 2000 chars."`
- **system_auditor.lx**: `"You are running a final system-wide audit after all work items have been implemented. Check code quality across the full codebase and verify spec compliance for every executed work item."`
- **grader.lx**: `"You are a grader scoring work against a rubric. For each category, assign a score 0-100 and brief feedback. A category passes at score >= 70."` — same as workgen grader, just add system_prompt.

For implementer.lx and fixer.lx: the current code does `p.compose msg.bootstrap_prompt`. In the new version, this goes inside `build_prompt`: `p = Prompt { system: self.system_prompt () }; p.compose msg.bootstrap_prompt; p.add_section ...`.

After updating agents, delete these 6 files:
- `programs/workrunner/prompt/implement.lx`
- `programs/workrunner/prompt/fix.lx`
- `programs/workrunner/prompt/fix_wi.lx`
- `programs/workrunner/prompt/investigate.lx`
- `programs/workrunner/prompt/system_audit.lx`
- `programs/workrunner/prompt/grade.lx`

DO NOT delete `programs/workrunner/prompt/context.lx` — it has a `load` method that reads files from disk and is imported by `workrunner/main.lx`.

Add `use std/prompt {Prompt}` to each agent file that creates Prompt instances.

**ActiveForm:** Simplifying workrunner agents

---

### Task 4: Update brain agents

**Subject:** Add role/goal/system_prompt to brain agents, fix dispatcher dead code

**Description:** For each agent in `programs/brain/agents/`:

**planner.lx (TaskPlanner)** — Add:
```lx
role = () { "task decomposition and planning specialist" }
goal = () { "create precise, minimal execution plans" }
system_prompt = () { "You create precise, minimal execution plans." }
```
Keep all specialist methods (plan, replan, estimate_cost) unchanged.

**analyst.lx (DeepAnalyst)** — Add role/goal/system_prompt from existing AnalysisPrompt system string. Keep analyze/compare/investigate.

**researcher.lx (InfoGatherer)** — Add role/goal/system_prompt. Keep research/search.

**critic.lx (InnerCritic)** — Add role/goal/system_prompt from CritiquePrompt system string. Keep critique/challenge/verify_code.

**synthesizer.lx (ResponseSynth)** — Add role/goal/system_prompt. Keep synthesize/format/compose_parts.

**monitor.lx** — Add `health = () { check_health state }` wired to existing check_health function. Add role/goal.

**dispatcher.lx** — The file uses `~>?` syntax (lines 52, 64, 109, 110) which does NOT work — `~` lexes as Tilde→Bang, the operator was never implemented. Replace all `worker ~>?` and `workforce.analyst ~>?` calls with `agent.ask worker` or `self.delegate worker`. This is fixing dead code, not refactoring working code.

Brain agents have separate Prompt declarations (PlanningPrompt, AnalysisPrompt, etc.) defined in the same file. These are used by specialist methods that build custom prompts. Do NOT delete them — they're in-file declarations, not separate prompt files.

**ActiveForm:** Updating brain agents

---

### Task 5: Update keyword test

**Subject:** Verify new Agent methods exist in keyword test

**Description:** In `tests/keywords.lx`, update the Agent section:

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

**ActiveForm:** Updating keyword test

---

### Task 6: Run full test suite and fix regressions

**Subject:** Verify no regressions, fix any failures

**Description:**
1. Run `just rust-diagnose` — fix any compile errors.
2. Run `just test` — fix any test failures.
3. Check that `tests/keywords.lx` passes with the updated Agent test.
4. Check that workgen/workrunner tests pass (if they exist in programs/workgen/tests/ and programs/workrunner/tests/).

Likely failure sources:
- Missing `use std/prompt {Prompt}` in agent files that now create Prompt instances
- `on_turn_end` signature mismatch if any agent passes wrong number of args
- `handle`'s try/ooda pattern: if `try` doesn't see a function+arg it will error. Verify `try ooda enriched` works (ooda is a function, enriched is its argument).
- Deleted prompt file imports that weren't removed from agent files

**ActiveForm:** Full regression testing

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

1. **Call `complete_task` after each task.**
2. **Call `next_task` to get the next task.**
3. **Do not add, skip, reorder, or combine tasks.**
4. **Tasks are implementation-only.**
5. **Do not add code comments or doc strings** (exception: `--` header comments on .lx program files).
6. **Use `just rust-diagnose` not raw cargo commands.**
7. **`try` takes 2 args: `try func arg`.** NOT `try { block }`. Extract the OODA chain into `ooda = (m) { ... }` then call `try ooda enriched`.
8. **`on_turn_end` takes 2 args: `(msg result)`.** Logging needs both msg (runner_dir, slug) and result (text, turns).
9. **DO NOT delete `workrunner/prompt/context.lx`** — it has a load method and is imported by main.lx.
10. **`~>?` in brain/dispatcher.lx is dead code** — the operator was never implemented. Replace with agent.ask/self.delegate.
11. **`act` default stays as `(plan) { plan }` pass-through.** Do not change it to auto-call build_prompt.
12. **`use std/prompt {Prompt}` must be added to agent.lx** and to any agent file that creates Prompt instances in build_prompt.

---

## Task Loading Instructions

```
mcp__workflow__load_work_item({ path: "work_items/AGENT_TRAIT_REDESIGN.md" })
```
