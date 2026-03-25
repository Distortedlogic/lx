# Goal

Write Trait .lx files for each keyword's contract: Tool, Prompt, Session, Guard, Workflow, Schema. Agent (`pkg/agent.lx`), Connector (`pkg/core/connector.lx`), and Collection (`pkg/core/collection.lx`) already exist. Each new trait must be testable with manual `Class X : [Trait] = { ... }` syntax — no compiler changes in this unit.

# Why

The keyword desugaring pipeline (Units 2-5) converts `Agent X = { ... }` into `Class X : [Agent] = { ... }`. The traits must exist before the desugaring can inject them. Writing traits first validates the contract design before building compiler infrastructure.

# What Changes

Six new .lx files in `pkg/core/`. All traits are exported with `+Trait`.

# Files Affected

- `pkg/core/tool.lx` — New file
- `pkg/core/prompt_trait.lx` — New file
- `pkg/core/session.lx` — New file
- `pkg/core/guard.lx` — New file
- `pkg/core/workflow.lx` — New file
- `pkg/core/schema.lx` — New file
- `tests/trait_tool.lx` — New test
- `tests/trait_prompt.lx` — New test
- `tests/trait_session.lx` — New test
- `tests/trait_guard.lx` — New test
- `tests/trait_workflow.lx` — New test
- `tests/trait_schema.lx` — New test

# Task List

### Task 1: Write Tool trait

**Subject:** Create pkg/core/tool.lx

**Description:** Create `pkg/core/tool.lx` with the following exact content:

```lx
+Trait Tool = {
  description: Str = ""
  params: Record = {}

  run = (args) { Err "Tool.run not implemented" }

  schema = () { self.params }

  validate = (args) {
    missing = self.params | keys | filter (k) { not (args | keys | any? (== k)) }
    (missing | len) == 0 ? Ok args : Err {missing: missing}
  }
}
```

Note: The `params` field stores a record mapping parameter names to type descriptors (strings like "Str", "Int"). The `schema` default just returns this record. The `validate` default checks that every key in params exists in args.

**ActiveForm:** Writing Tool trait

---

### Task 2: Write Prompt trait

**Subject:** Create pkg/core/prompt_trait.lx

**Description:** Create `pkg/core/prompt_trait.lx`. The Prompt trait wraps the functional API from `pkg/core/prompt.lx` into a trait with self-referencing methods. Read `pkg/core/prompt.lx` first — the existing render logic is:

- System goes first
- Sections rendered as "SectionName:\ncontent"
- Constraints rendered as "Constraints:\n- item\n- item"
- Instructions rendered as "Instructions:\n- item\n- item"
- Examples rendered as "Example N:\nInput: ...\nOutput: ..."
- All joined by "\n\n"

Write this exact content:

```lx
+Trait Prompt = {
  system: Str = ""
  sections: List = []
  constraints: List = []
  instructions: List = []
  examples: List = []

  render = () {
    parts := []
    self.system | len > 0 ? (parts <- [..parts self.system])
    self.sections | each (sec) {
      parts <- [..parts "{sec.name}:\n{sec.content}"]
    }
    (self.constraints | len) > 0 ? {
      items = self.constraints | map (c) "- {c}" | join "\n"
      parts <- [..parts "Constraints:\n{items}"]
    }
    (self.instructions | len) > 0 ? {
      items = self.instructions | map (i) "- {i}" | join "\n"
      parts <- [..parts "Instructions:\n{items}"]
    }
    (self.examples | len) > 0 ? {
      items = self.examples | enumerate | map (pair) {
        (idx ex) = pair
        "Example {idx + 1}:\nInput: {ex.input}\nOutput: {ex.output}"
      } | join "\n\n"
      parts <- [..parts "Examples:\n{items}"]
    }
    parts | join "\n\n"
  }

  with_section = (name content) {
    {..self, sections: [..self.sections {name: name, content: content}]}
  }

  with_constraint = (text) {
    {..self, constraints: [..self.constraints text]}
  }

  with_instruction = (text) {
    {..self, instructions: [..self.instructions text]}
  }

  compose = (other) {
    sys = (other.system | len) > 0 ? other.system : self.system
    {system: sys
     sections: [..self.sections ..other.sections]
     constraints: [..self.constraints ..other.constraints]
     instructions: [..self.instructions ..other.instructions]
     examples: [..self.examples ..other.examples]}
  }

  trim_to_fit = (max_tokens) {
    text = self.render ()
    (text | len) // 4 <= max_tokens ? text : {
      trimmed := self
      loop {
        (trimmed.sections | len) == 0 ? break
        trimmed <- {..trimmed, sections: trimmed.sections | take ((trimmed.sections | len) - 1)}
        t = trimmed.render ()
        (t | len) // 4 <= max_tokens ? break
      }
      trimmed.render ()
    }
  }

  ask = (fallback) {
    rendered = self.render ()
    resp = ai.prompt rendered ?? {text: fallback}
    resp.text
  }

  ask_with = (opts fallback) {
    rendered = self.render ()
    resp = ai.prompt_with {..opts, prompt: rendered} ?? {text: fallback}
    resp.text
  }
}
```

Note: `with_section`, `with_constraint`, `with_instruction`, `compose` return new records (not mutations of self) following lx's functional record update `{..self, field: new}` pattern. The `ask` and `ask_with` methods call `ai.prompt` / `ai.prompt_with` which are globals provided by the runtime — no `use std/ai` needed at the trait level because traits are evaluated in the consuming module's scope.

**ActiveForm:** Writing Prompt trait

---

### Task 3: Write Session trait

**Subject:** Create pkg/core/session.lx

**Description:** Create `pkg/core/session.lx`:

```lx
use std/time

+Trait Session = {
  messages: Store = Store ()
  max_tokens: Int = 200000
  compaction_threshold: Float = 0.7
  checkpoints: Store = Store ()
  next_id: Int = 0

  add_message = (msg) {
    id = self.next_id
    self.next_id <- id + 1
    self.messages.set (to_str id) msg
    pressure = self.pressure ()
    pressure >= self.compaction_threshold ? (self.compact ())
    Ok id
  }

  compact = () {
    all_keys = self.messages.keys ()
    count = all_keys | len
    count <= 2 ? (Ok ())
    half = count // 2
    old_keys = all_keys | take half
    summaries = old_keys | map (k) {
      msg = self.messages.get k
      msg ?? ""
    }
    summary_text = summaries | map to_str | join "\n---\n"
    old_keys | each (k) { self.messages.remove k }
    self.messages.set "__summary" {role: "summary", content: summary_text, at: time.now ().iso}
    Ok ()
  }

  checkpoint = () {
    cp_id = "cp_{self.next_id}"
    self.next_id <- self.next_id + 1
    snapshot = self.messages.keys () | map (k) { {key: k, value: self.messages.get k} }
    self.checkpoints.set cp_id snapshot
    Ok cp_id
  }

  resume = (cp_id) {
    snapshot = self.checkpoints.get cp_id
    snapshot ? {
      None -> Err "checkpoint not found: {cp_id}"
      Some entries -> {
        self.messages.keys () | each (k) { self.messages.remove k }
        entries | each (e) { self.messages.set e.key e.value }
        Ok ()
      }
    }
  }

  handoff = () {
    all = self.messages.keys () | map (k) { self.messages.get k }
    {messages: all, token_usage: self.token_usage (), pressure: self.pressure ()}
  }

  token_usage = () {
    self.messages.values () | map (m) { (to_str m | len) // 4 } | fold 0 (+)
  }

  pressure = () {
    self.max_tokens == 0 ? 0.0 : ((self.token_usage () | to_float) / (self.max_tokens | to_float))
  }
}
```

**ActiveForm:** Writing Session trait

---

### Task 4: Write Guard trait

**Subject:** Create pkg/core/guard.lx

**Description:** Create `pkg/core/guard.lx`. This is a generalization of the existing CircuitBreaker in `pkg/core/circuit.lx`. The logic is extracted from CircuitBreaker's methods (already verified — see `circuit.lx` source). Write:

```lx
use std/time

+Trait Guard = {
  max_turns: Int = 100
  max_time_ms: Int = 300000
  max_actions: Int = 1000
  repetition_window: Int = 5
  turns: Int = 0
  actions: List = []
  start_ms: Int = 0
  tripped: Str = None

  tick = () {
    self.start_ms == 0 ? (self.start_ms <- (time.now ()).ms)
    self.turns <- self.turns + 1
    self.turns >= self.max_turns ? (self.tripped <- "max_turns: {self.turns} >= {self.max_turns}")
    elapsed = (time.now ()).ms - self.start_ms
    elapsed >= self.max_time_ms ? (self.tripped <- "max_time: {elapsed}ms >= {self.max_time_ms}ms")
    Ok ()
  }

  record = (action) {
    self.start_ms == 0 ? (self.start_ms <- (time.now ()).ms)
    self.actions <- [..self.actions action]
    (self.actions | len) >= self.max_actions ? (self.tripped <- "max_actions: {self.actions | len} >= {self.max_actions}")
    w = self.repetition_window
    w > 0 && (self.actions | len) >= w ? {
      window = self.actions | drop ((self.actions | len) - w)
      head = window | first ^
      all_same = window | all? (== head)
      all_same ? (self.tripped <- "repetition: last {w} actions identical")
    }
    Ok ()
  }

  check = () {
    self.tripped != None ? Err {reason: self.tripped} : Ok ()
  }

  is_tripped = () { self.tripped != None }

  reset = () {
    self.turns <- 0
    self.actions <- []
    self.start_ms <- (time.now ()).ms
    self.tripped <- None
    Ok ()
  }

  status = () {
    elapsed = self.start_ms > 0 ? ((time.now ()).ms - self.start_ms) : 0
    {turns: self.turns, elapsed_ms: elapsed, action_count: self.actions | len, tripped: self.tripped != None, reason: self.tripped}
  }
}
```

**ActiveForm:** Writing Guard trait

---

### Task 5: Write Workflow trait

**Subject:** Create pkg/core/workflow.lx

**Description:** Create `pkg/core/workflow.lx`. Each step is a record `{id: Str, run: Func, depends: [Str], undo: Func?}`. The `run` method does topological sort then sequential execution. Write:

```lx
+Trait Workflow = {
  steps: List = []
  on_fail: Str = "abort"
  max_retries: Int = 3
  status: Store = Store ()

  run = (context) {
    ordered = topo_sort self.steps
    ordered ? {
      Err e -> Err e
      Ok sorted -> {
        results := {}
        sorted | each (step) {
          self.status.set step.id "running"
          result = run_with_retry step.run context self.max_retries
          result ? {
            Err e -> {
              self.status.set step.id "failed"
              self.on_fail == "abort" ? {
                self.rollback ()
                Err {step: step.id, error: e}
              }
            }
            Ok v -> {
              self.status.set step.id "done"
              results <- {..results, (step.id): v}
            }
          }
        }
        Ok results
      }
    }
  }

  rollback = () {
    completed = self.status.keys () | filter (k) { self.status.get k == "done" } | reverse
    completed | each (step_id) {
      step = self.steps | find (s) s.id == step_id
      step ? {
        Some s -> s.undo != None ? (s.undo (self.status.get step_id))
        None -> ()
      }
    }
    Ok ()
  }

  step_status = () {
    self.status.keys () | map (k) { {id: k, status: self.status.get k} }
  }
}

run_with_retry = (f context retries) {
  attempt := 0
  result := Err "not started"
  loop {
    attempt <- attempt + 1
    result <- try (f context)
    result | ok? ? break
    attempt >= retries ? break
  }
  result
}

topo_sort = (steps) {
  sorted := []
  remaining := steps
  done := []
  loop {
    (remaining | len) == 0 ? break
    ready = remaining | filter (s) {
      s.depends | all? (d) { done | any? (== d) }
    }
    (ready | len) == 0 ? (Err "cycle detected in workflow steps")
    ready | each (s) {
      sorted <- [..sorted s]
      done <- [..done s.id]
    }
    remaining <- remaining | filter (s) { not (done | any? (== s.id)) }
  }
  Ok sorted
}
```

**ActiveForm:** Writing Workflow trait

---

### Task 6: Write Schema trait

**Subject:** Create pkg/core/schema.lx

**Description:** Create `pkg/core/schema.lx`. The Schema trait provides `validate` — it checks that a data record has the required fields. Note: `TraitDeclData.requires` does NOT work at runtime (the `requires` field is stored but `inject_traits()` never reads it). So Schema cannot be inherited via `requires`. Instead, when the Schema keyword desugars a trait (Unit 3), it will directly inject these methods as defaults on the generated TraitDecl. For now, write the standalone trait:

```lx
+Trait Schema = {
  validate = (data) {
    type_of data != "Record" ? (Err {error: "expected Record, got {type_of data}"}) : (Ok data)
  }

  schema = () {
    {type: "object"}
  }
}
```

This is intentionally minimal. The Schema keyword desugaring (Unit 3) will generate richer `schema()` and `validate()` implementations based on the declared fields. This trait exists as the base contract.

**ActiveForm:** Writing Schema trait

---

### Task 7: Write tests for all new traits

**Subject:** Create test files validating each trait

**Description:** Create six test files:

`tests/trait_tool.lx`:
```lx
use pkg/core/tool {Tool}

Class Echo : [Tool] = {
  description: "echoes input"
  params: {text: "Str"}
  run = (args) { Ok args.text }
}

t = Echo {}
assert t.description == "echoes input"
assert (t.schema () | keys | len) == 1
result = t.run {text: "hello"}
assert result == Ok "hello"
valid = t.validate {text: "hi"}
assert (valid | ok?)
invalid = t.validate {}
assert (invalid | err?)
```

`tests/trait_prompt.lx`:
```lx
use pkg/core/prompt_trait {Prompt}

Class Greeter : [Prompt] = {
  system: "You greet people"
}

p = Greeter {}
p2 = p.with_section "Name" "Alice"
rendered = p2.render ()
assert (rendered | contains? "You greet people")
assert (rendered | contains? "Alice")
```

`tests/trait_session.lx`:
```lx
use pkg/core/session {Session}

Class Chat : [Session] = { max_tokens: 1000 }

s = Chat {}
s.add_message {role: "user", content: "hello"}
s.add_message {role: "assistant", content: "hi there"}
assert (s.token_usage ()) > 0
assert (s.pressure ()) > 0.0
cp = s.checkpoint () ^
s.add_message {role: "user", content: "more stuff"}
s.resume cp ^
```

`tests/trait_guard.lx`:
```lx
use pkg/core/guard {Guard}

Class TurnLimit : [Guard] = { max_turns: 3 }

g = TurnLimit {}
g.tick ()
g.tick ()
g.tick ()
assert (g.check () | ok?)
g.tick ()
assert (g.is_tripped ())
assert (g.check () | err?)
g.reset ()
assert (not (g.is_tripped ()))
assert (g.check () | ok?)
```

`tests/trait_workflow.lx`:
```lx
use pkg/core/workflow {Workflow}

Class TwoStep : [Workflow] = {
  steps: [
    {id: "a", run: (ctx) { 1 }, depends: [], undo: None}
    {id: "b", run: (ctx) { 2 }, depends: ["a"], undo: None}
  ]
}

w = TwoStep {}
result = w.run {}
assert (result | ok?)
```

`tests/trait_schema.lx`:
```lx
use pkg/core/schema {Schema}

Class UserValidator : [Schema] = {
  validate = (data) {
    (type_of data) != "Record" ? (Err "not a record") : {
      missing = ["name" "age"] | filter (k) { not (data | keys | any? (== k)) }
      (missing | len) == 0 ? Ok data : Err {missing: missing}
    }
  }
}

v = UserValidator {}
good = v.validate {name: "Alice", age: 30}
assert (good | ok?)
bad = v.validate {name: "Bob"}
assert (bad | err?)
```

Run `just test`.

**ActiveForm:** Writing trait tests

---

### Task 8: Verify existing traits

**Subject:** Confirm Agent, Connector, Collection have correct signatures

**Description:** Read `pkg/agent.lx` and verify Agent trait has: init, perceive, reason, act, reflect, handle, run, think, think_with, think_structured, use_tool, tools, describe, ask, tell — all with defaults. This has been verified. No changes needed.

Read `pkg/core/connector.lx` and verify Connector trait has: connect, disconnect, call, tools. This has been verified — the methods are signature-only with no defaults. No changes needed.

Read `pkg/core/collection.lx` and verify Collection trait has: get, keys, values, remove, query, len, has, save, load — all with defaults that delegate to `self.entries`. This has been verified. No changes needed.

**ActiveForm:** Verifying existing traits

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

1. **Call `complete_task` after each task.**
2. **Call `next_task` to get the next task.**
3. **Do not add, skip, reorder, or combine tasks.**
4. **Tasks are implementation-only.**

---

## Task Loading Instructions

```
mcp__workflow__load_work_item({ path: "work_items/KEYWORD_DESUGAR_1_TRAITS.md" })
```
