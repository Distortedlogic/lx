# Goal

Write Trait .lx files for each keyword's contract: Tool, Prompt, Session, Guard, Workflow, Schema. These are the behavioral contracts that keyword desugaring will inject. Agent, Connector, and Collection traits already exist. Each new trait must be testable immediately with manual `Class X : [Trait] = { ... }` syntax — no compiler changes in this unit.

# Why

- The keyword desugaring pipeline (Units 2-5) converts `Agent X = { ... }` into `Class X : [Agent] = { ... }`. The traits must exist before the desugaring can inject them.
- Writing traits first validates the contract design. If a trait's defaults don't compose well with user-provided overrides, we discover that here — not after building compiler infrastructure.
- Existing patterns already demonstrate every trait: CircuitBreaker is the Guard pattern, MemoryStore is the Store/Collection pattern, prompt.lx's functional API defines the Prompt surface, workrunner's orchestrator demonstrates the Workflow shape.

# What Changes

**`pkg/core/tool.lx` — Trait Tool:**

Fields: `description: ""`, `params: {}`. Methods: `run(args)` (abstract — default returns Err "not implemented"), `schema()` (default returns self.params as-is), `validate(args)` (default checks args has all keys in self.params). Exported.

**`pkg/core/prompt_trait.lx` — Trait Prompt:**

Fields: `system: ""`, `sections: []`, `constraints: []`, `instructions: []`, `examples: []`. Methods: `render()` (assembles system + sections + constraints + instructions + examples into formatted string), `compose(other)` (merges two prompts, concatenating sections/constraints/instructions/examples, other.system overrides if non-empty), `with_section(name, content)` (returns new Prompt with section appended), `with_constraint(text)`, `with_instruction(text)`, `trim_to_fit(max_tokens)` (estimates token count via len/4 heuristic, trims oldest sections), `ask(fallback)` (renders then calls ai.prompt, returns result or fallback on error), `ask_with(opts, fallback)` (renders then calls ai.prompt_with with opts). Exported.

**`pkg/core/session.lx` — Trait Session:**

Fields: `messages: Store()`, `max_tokens: 200000`, `compaction_threshold: 0.7`, `checkpoints: Store()`. Methods: `add_message(msg)` (stores message, checks pressure, auto-compacts if above threshold), `compact()` (summarizes oldest messages into a single summary message, removes originals), `checkpoint()` (snapshots current messages to checkpoints store, returns checkpoint id), `resume(checkpoint_id)` (restores messages from checkpoint), `handoff()` (returns structured record: messages summary, token usage, key decisions), `token_usage()` (estimates total tokens across all messages), `pressure()` (token_usage / max_tokens). Exported.

**`pkg/core/guard.lx` — Trait Guard:**

Fields: `max_turns: 100`, `max_time_ms: 300000`, `max_actions: 1000`, `repetition_window: 5`, `turns: 0`, `actions: []`, `start_ms: 0`, `tripped: None`. Methods: `tick()` (increments turns, checks max_turns and max_time, sets tripped if exceeded), `record(action)` (appends to actions, checks repetition_window for repeated identical actions, sets tripped if detected), `check()` (returns Ok () if not tripped, Err with reason if tripped), `is_tripped()` (returns Bool), `reset()` (zeros turns, clears actions, clears tripped), `status()` (returns record with all fields). Default implementations extracted from existing CircuitBreaker logic in `pkg/core/circuit.lx`. Exported.

**`pkg/core/workflow.lx` — Trait Workflow:**

Fields: `steps: []`, `on_fail: "abort"`, `max_retries: 3`, `status: Store()`. Methods: `run(context)` (topological sort of steps by `depends` field, executes sequentially, retries failed steps up to max_retries, records step status, calls rollback on abort), `rollback()` (iterates completed steps in reverse, calls each step's `undo` function if present), `step_status()` (returns record of step_id → status from store), `advance(step_id, result)` (manually marks step complete with result). Each step is a record: `{id: Str, run: Func, depends: [], undo: Func?}`. Exported.

**`pkg/core/schema.lx` — Trait Schema:**

Fields: none (schema fields come from the declaring trait's own fields). Methods: `schema()` (returns a record describing field names and types for JSON schema generation), `validate(data)` (checks data record has required fields, returns Ok data or Err with missing/invalid fields). Exported.

**Verify existing traits:** Confirm `pkg/agent.lx` (Agent), `pkg/core/connector.lx` (Connector), `pkg/core/collection.lx` (Collection) exist with correct signatures. No modifications expected.

# Files Affected

- `pkg/core/tool.lx` — New file: Tool trait
- `pkg/core/prompt_trait.lx` — New file: Prompt trait
- `pkg/core/session.lx` — New file: Session trait
- `pkg/core/guard.lx` — New file: Guard trait
- `pkg/core/workflow.lx` — New file: Workflow trait
- `pkg/core/schema.lx` — New file: Schema trait
- `tests/trait_tool.lx` — New test file
- `tests/trait_prompt.lx` — New test file
- `tests/trait_session.lx` — New test file
- `tests/trait_guard.lx` — New test file
- `tests/trait_workflow.lx` — New test file
- `tests/trait_schema.lx` — New test file

# Task List

### Task 1: Write Tool trait

**Subject:** Create pkg/core/tool.lx with Tool trait definition

**Description:** Create `pkg/core/tool.lx`. Define `+Trait Tool` with fields `description: ""`, `params: {}` and default methods `run`, `schema`, `validate` as described in What Changes. The `run` default should return `Err "Tool.run not implemented"`. The `schema` default should return `self.params`. The `validate` default should check that every key in `self.params` exists in `args` and return `Ok args` or `Err` with missing key names.

**ActiveForm:** Writing Tool trait definition

---

### Task 2: Write Prompt trait

**Subject:** Create pkg/core/prompt_trait.lx with Prompt trait definition

**Description:** Create `pkg/core/prompt_trait.lx`. Define `+Trait Prompt` with fields `system: ""`, `sections: []`, `constraints: []`, `instructions: []`, `examples: []` and default methods `render`, `compose`, `with_section`, `with_constraint`, `with_instruction`, `trim_to_fit`, `ask`, `ask_with` as described in What Changes. The `render` default assembles all fields into a formatted string with section headers. The `compose` method merges by concatenating list fields and preferring non-empty system. Use `use std/ai` for `ask` and `ask_with` implementations that call `ai.prompt` and `ai.prompt_with`.

**ActiveForm:** Writing Prompt trait definition

---

### Task 3: Write Session trait

**Subject:** Create pkg/core/session.lx with Session trait definition

**Description:** Create `pkg/core/session.lx`. Define `+Trait Session` with fields `messages: Store()`, `max_tokens: 200000`, `compaction_threshold: 0.7`, `checkpoints: Store()` and default methods `add_message`, `compact`, `checkpoint`, `resume`, `handoff`, `token_usage`, `pressure` as described in What Changes. Token estimation uses `len / 4` heuristic on stringified messages. Compaction takes the oldest 50% of messages, concatenates their content, stores as single summary message, removes originals.

**ActiveForm:** Writing Session trait definition

---

### Task 4: Write Guard trait

**Subject:** Create pkg/core/guard.lx with Guard trait definition

**Description:** Create `pkg/core/guard.lx`. Define `+Trait Guard` with fields and default methods as described in What Changes. Read `pkg/core/circuit.lx` (CircuitBreaker) first — extract its logic into the Guard trait defaults. The Guard trait should be a generalization of CircuitBreaker. `tick` checks turn limit and time limit. `record` checks action repetition within the window. `check` returns Ok/Err based on `tripped` field. `is_tripped` returns `tripped | some?`. `reset` zeros all counters. `status` returns a snapshot record.

**ActiveForm:** Writing Guard trait definition

---

### Task 5: Write Workflow trait

**Subject:** Create pkg/core/workflow.lx with Workflow trait definition

**Description:** Create `pkg/core/workflow.lx`. Define `+Trait Workflow` with fields and default methods as described in What Changes. The `run` default implements: (1) build dependency graph from steps' `depends` fields, (2) topological sort (error if cycle detected), (3) execute each step in order by calling `step.run(context)`, (4) on step failure: retry up to max_retries, (5) if still failing and on_fail is "abort", call `self.rollback()` and return Err, (6) if on_fail is "skip", mark step skipped and continue, (7) record each step's status in self.status store. The `rollback` default iterates completed steps in reverse order and calls `step.undo(result)` for steps that have an `undo` function.

**ActiveForm:** Writing Workflow trait definition

---

### Task 6: Write Schema trait

**Subject:** Create pkg/core/schema.lx with Schema trait definition

**Description:** Create `pkg/core/schema.lx`. Define `+Trait Schema` with methods `schema()` and `validate(data)`. The `schema` default returns a record mapping field names to their type names (extracted from the trait's field declarations at runtime via introspection or a manually-maintained mapping). The `validate` default checks that `data` is a record, that all required field names are present, and returns `Ok data` or `Err` with a record of `{missing: [...], invalid: [...]}`. Since lx Traits with fields already act as record constructors with validation, the Schema trait adds the `schema()` method for JSON schema generation and explicit `validate()` for programmatic checking.

**ActiveForm:** Writing Schema trait definition

---

### Task 7: Write tests for all new traits

**Subject:** Create test files validating each trait works with manual Class : [Trait] syntax

**Description:** Create six test files:

`tests/trait_tool.lx`: Define `Class Echo : [Tool] = { description: "echoes input", params: {text: "Str"}, run = (args) { args.text } }`. Instantiate, call run, call schema, call validate with valid and invalid args. Assert results.

`tests/trait_prompt.lx`: Define `Class Greeter : [Prompt] = { system: "You greet people" }`. Instantiate, call with_section, render, compose two prompts. Assert rendered output contains system and sections.

`tests/trait_session.lx`: Define `Class MySession : [Session] = {}`. Instantiate, add_message three times, check token_usage > 0, check pressure > 0, checkpoint, add more messages, resume checkpoint, verify messages restored.

`tests/trait_guard.lx`: Define `Class TurnGuard : [Guard] = { max_turns: 3 }`. Instantiate, tick three times, assert check returns Err on fourth tick. Reset, verify check returns Ok.

`tests/trait_workflow.lx`: Define a Workflow with three steps where step 2 depends on step 1. Run it. Assert steps executed in order. Define a workflow where a step fails. Assert rollback is called.

`tests/trait_schema.lx`: Define `Trait UserSchema : [Schema] = { name: Str, age: Int }` (manually, since Schema keyword doesn't exist yet). Call schema(), assert it describes name and age. Call validate with valid and invalid data.

Run `just test` to verify all pass.

**ActiveForm:** Writing trait validation tests

---

### Task 8: Verify existing traits

**Subject:** Confirm Agent, Connector, Collection traits exist with expected signatures

**Description:** Read `pkg/agent.lx`, `pkg/core/connector.lx`, `pkg/core/collection.lx`. Verify:
- Agent trait has methods: perceive, reason, act, reflect, handle, run, think, think_with, think_structured, use_tool, tools, describe, ask, tell — all with defaults
- Connector trait has methods: connect, disconnect, call, tools
- Collection trait has methods: get, keys, values, remove, query, len, has, save, load
If any are missing or have incorrect signatures, fix them. These three traits are targets for keyword desugaring (Agent, Connector, Store keywords respectively) and must have correct contracts.

**ActiveForm:** Verifying existing trait contracts

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.

---

## Task Loading Instructions

To execute this work item, load it with the workflow MCP:

```
mcp__workflow__load_work_item({ path: "work_items/KEYWORD_DESUGAR_1_TRAITS.md" })
```

Then call `next_task` to begin.
