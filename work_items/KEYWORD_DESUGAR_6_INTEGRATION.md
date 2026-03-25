# Goal

Full integration testing and validation of all 12 keywords. Verify equivalence with hand-written `Class : [Trait]` code. Refactor one module of the workrunner program to use keywords and confirm identical behavior.

# Why

- Units 1-5 implement the keywords in isolation. This unit verifies they compose correctly in real programs.
- Equivalence testing catches subtle differences between keyword-generated code and hand-written code (import ordering, field initialization order, method precedence).
- Refactoring workrunner validates the keywords work in the most complex lx program in the repo.

# What Changes

**Equivalence tests:** For each keyword, write two versions of the same program — one using the keyword, one using manual `Class : [Trait]`. Assert identical runtime behavior.

**Composition tests:** Write programs that use multiple keywords together (Agent with Tools, Workflow with Guards, MCP Connector used by an Agent).

**Workrunner refactoring:** Convert `programs/workrunner/lib/grader.lx` to use keywords where applicable. The Grader class becomes `Agent Grader` or stays as Class with `Prompt` and `Schema` keywords for its prompt template and grade result. This is a minimal refactor to validate real-world usage.

# Files Affected

- `tests/keyword_equivalence.lx` — New test: keyword vs manual equivalence
- `tests/keyword_composition.lx` — New test: multiple keywords together
- `tests/keyword_workrunner_refactor.lx` — New test: refactored workrunner module

# Task List

### Task 1: Write equivalence tests

**Subject:** Verify keyword desugaring produces identical behavior to hand-written Class : [Trait]

**Description:** Create `tests/keyword_equivalence.lx`. For each of the 8 simple keywords, write two versions:

```
-- Manual version
use pkg/core/guard {Guard}
Class ManualGuard : [Guard] = { max_turns: 5 }

-- Keyword version
Guard KeywordGuard = { max_turns: 5 }

-- Both should behave identically
m = ManualGuard {}
k = KeywordGuard {}

m.tick ()
k.tick ()
m.tick ()
k.tick ()

assert (m.turns == k.turns) "turns should match"
assert (m.is_tripped () == k.is_tripped ()) "tripped state should match"
```

Do this for Agent (compare perceive/think), Tool (compare run/schema), Prompt (compare render/compose), Connector (compare connect/tools), Store (compare get/keys/len), Session (compare add_message/pressure), Guard (compare tick/check), Workflow (compare run with simple steps).

Run `just test`.

**ActiveForm:** Writing keyword equivalence tests

---

### Task 2: Write composition tests

**Subject:** Verify multiple keywords compose correctly in a single program

**Description:** Create `tests/keyword_composition.lx`. Write a program that uses multiple keywords together:

1. Define a Schema for messages: `Schema TaskMessage = { task: Str, priority: Int }`.
2. Define a Tool: `Tool Logger = { description: "logs a message", params: {msg: "Str"}, run = (args) { log.info args.msg; Ok () } }`.
3. Define a Guard: `Guard SafetyNet = { max_turns: 10 }`.
4. Define a Store: `Store TaskLog = {}`.
5. Define a Prompt: `Prompt TaskPrompt = { system: "You execute tasks" }`.
6. Define an Agent that references the other keywords:
   ```
   Agent Worker = {
     guard: SafetyNet {}
     task_log: TaskLog {}

     act = (plan) {
       self.guard.tick ()
       self.guard.check () ^
       self.task_log.entries.set plan.id plan
       plan
     }
   }
   ```
7. Instantiate the Agent. Call act with a mock plan. Assert guard ticked. Assert task_log has the entry.

Run `just test`.

**ActiveForm:** Writing keyword composition tests

---

### Task 3: Refactor workrunner grader to use keywords

**Subject:** Convert Grader and GradeResult to use keywords as validation

**Description:** Create `tests/keyword_workrunner_refactor.lx` that reproduces a simplified version of the workrunner grader using keywords:

1. `Schema GradeResult = { score: Int, passed: Bool, categories: List, feedback: Str, failed: List }` — replaces the current `Trait GradeResult`.
2. `Schema CategoryScore = { name: Str, score: Int, passed: Bool, feedback: Str }` — replaces the current `Trait CategoryScore`.
3. `Prompt GradingPrompt = { system: "You are a grader scoring work against a rubric." }` — replaces the functional prompt building in Grader.build_prompt.

Test that Schema instances can be created, validated, and that the Prompt renders correctly.

Do NOT modify the actual workrunner program — this is a validation test only. If the test passes, the keywords can replace the manual code in a future PR.

Run `just test`.

**ActiveForm:** Validating keywords against workrunner patterns

---

### Task 4: Run full test suite

**Subject:** Run just test and verify zero regressions

**Description:** Run `just test`. All existing tests must pass. All new keyword tests must pass. If any failures, investigate and fix. The keyword desugaring must not break any existing program that uses Class, Trait, or any other existing syntax.

Also run `just diagnose` to verify zero compiler warnings in the new code.

**ActiveForm:** Running full test suite validation

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
mcp__workflow__load_work_item({ path: "work_items/KEYWORD_DESUGAR_6_INTEGRATION.md" })
```

Then call `next_task` to begin.
