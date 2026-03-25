# Goal

Integration testing for all 12 keywords. Verify equivalence with hand-written `Class : [Trait]`. Verify keywords compose in a single program. Validate against workrunner patterns.

# Why

Units 1-5 implement keywords in isolation. This unit catches composition issues: import ordering, field initialization, method precedence between user-defined and trait-injected methods.

# Files Affected

- `tests/keyword_equivalence.lx` — New test
- `tests/keyword_composition.lx` — New test
- `tests/keyword_workrunner_patterns.lx` — New test

# Task List

### Task 1: Write equivalence tests

**Subject:** Verify keyword desugaring matches manual Class : [Trait] behavior

**Description:** Create `tests/keyword_equivalence.lx`. For each of the 8 simple keywords, instantiate both the keyword version and the manual version, then assert identical behavior.

```lx
use std/guard {Guard}

-- Manual
Class ManualGuard : [Guard] = { max_turns: 5 }

-- Keyword
Guard KeywordGuard = { max_turns: 5 }

m = ManualGuard {}
k = KeywordGuard {}

m.tick ()
k.tick ()
m.tick ()
k.tick ()

assert m.turns == k.turns
assert (m.is_tripped ()) == (k.is_tripped ())
assert (m.check () | ok?) == (k.check () | ok?)

-- Agent equivalence
use std/agent {Agent}

Class ManualAgent : [Agent] = {
  perceive = (msg) { {parsed: msg} }
}

Agent KeywordAgent = {
  perceive = (msg) { {parsed: msg} }
}

ma = ManualAgent {}
ka = KeywordAgent {}

assert (ma.perceive "hi").parsed == (ka.perceive "hi").parsed
assert (methods_of ma | sort) == (methods_of ka | sort)

-- Store equivalence
use std/collection {Collection}

Class ManualStore : [Collection] = { entries: Store () }

Store KeywordStore = {}

ms = ManualStore {}
ks = KeywordStore {}

ms.entries.set "a" 1
ks.entries.set "a" 1

assert (ms.get "a") == (ks.get "a")
assert (ms.len ()) == (ks.len ())

-- Tool equivalence
use std/tool {Tool}

Class ManualTool : [Tool] = {
  description: "test"
  params: {x: "Int"}
  run = (args) { Ok args.x }
}

Tool KeywordTool = {
  description: "test"
  params: {x: "Int"}
  run = (args) { Ok args.x }
}

mt = ManualTool {}
kt = KeywordTool {}

assert (mt.run {x: 42}) == (kt.run {x: 42})
assert (mt.schema ()) == (kt.schema ())
```

Run `just test`.

**ActiveForm:** Writing equivalence tests

---

### Task 2: Write composition tests

**Subject:** Verify multiple keywords compose in a single program

**Description:** Create `tests/keyword_composition.lx`:

```lx
Tool Logger = {
  description: "logs"
  params: {msg: "Str"}
  run = (args) { log.info args.msg; Ok () }
}

Guard Safety = { max_turns: 10 }

Store TaskLog = {}

Agent Worker = {
  guard: Safety {}
  task_log: TaskLog {}

  act = (plan) {
    self.guard.tick () ^
    self.guard.check () ^
    self.task_log.entries.set (to_str plan.id) plan
    Ok plan
  }
}

w = Worker {}

result = w.act {id: 1, name: "test task"}
assert (result | ok?)
assert (w.task_log.len ()) == 1
assert w.guard.turns == 1

w.act {id: 2, name: "task 2"}
assert (w.task_log.len ()) == 2
assert w.guard.turns == 2
```

This tests: Tool instantiation, Guard instantiation as Agent field, Store instantiation as Agent field, Agent with custom act method, cross-keyword interaction.

Run `just test`.

**ActiveForm:** Writing composition tests

---

### Task 3: Validate against workrunner patterns

**Subject:** Reproduce workrunner patterns with keywords

**Description:** Create `tests/keyword_workrunner_patterns.lx`. This test reproduces simplified versions of the workrunner's core patterns using keywords:

```lx
-- GradeResult as Schema (if Schema keyword works)
-- If Schema tests are passing from Unit 3:
Schema GradeResult = {
  score: Int = "0-100 weighted score"
  passed: Bool = "true if score >= threshold"
  feedback: Str = "human-readable summary"
}

result = GradeResult {score: 95, passed: true, feedback: "all good"}
assert result.score == 95
assert result.passed

-- Grader-like Tool
Tool ScoreChecker = {
  description: "checks if score meets threshold"
  params: {score: "Int", threshold: "Int"}
  run = (args) {
    args.score >= args.threshold ? Ok {passed: true} : Ok {passed: false}
  }
}

checker = ScoreChecker {}
r = checker.run {score: 90, threshold: 80}
assert (r | ok?)

-- Work status persistence with Store
Store RunStatus = {}

status = RunStatus {}
status.entries.set "task1" {status: "pass", score: 95}
status.entries.set "task2" {status: "fail", score: 60}
assert (status.len ()) == 2
passed = status.values () | filter (r) { r.status == "pass" }
assert (passed | len) == 1

-- Guard for work item processing
Guard WorkGuard = { max_turns: 50, max_time_ms: 600000 }

g = WorkGuard {}
g.tick ()
assert (g.check () | ok?)
```

If Schema keyword (Unit 3) hasn't landed yet, replace the Schema section with a manual Trait definition. The test should work regardless.

Run `just test`.

**ActiveForm:** Validating workrunner patterns

---

### Task 4: Run full test suite

**Subject:** Verify zero regressions across all tests

**Description:** Run `just test`. Every existing test must still pass. Every new keyword test from Units 1-6 must pass. If any fail, diagnose and fix.

Also run `just diagnose` to verify zero warnings in new code.

**ActiveForm:** Full regression testing

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

1. **Call `complete_task` after each task.**
2. **Call `next_task` to get the next task.**
3. **Do not add, skip, reorder, or combine tasks.**
4. **Tasks are implementation-only.**

---

## Task Loading Instructions

```
mcp__workflow__load_work_item({ path: "work_items/KEYWORD_DESUGAR_6_INTEGRATION.md" })
```
