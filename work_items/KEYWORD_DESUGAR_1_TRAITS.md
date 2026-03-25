# Goal

Fix trait field default inheritance in the interpreter, add .lx-based std module support, then write Trait .lx files for each keyword's contract in `std/`: Tool, Prompt, Session, Guard, Workflow, Schema, plus copies of Agent, Connector, and Collection. Each new trait must be testable with manual `Class X : [Trait] = { ... }` syntax.

# Why

The keyword desugaring pipeline (Units 2-5) converts `Agent X = { ... }` into `Class X : [Agent] = { ... }`. The traits must exist before the desugaring can inject them. Writing traits first validates the contract design before building compiler infrastructure.

# Prerequisite: fix trait field default inheritance

Currently `inject_traits()` copies only method defaults into classes, NOT field defaults. This means `Class X : [Guard] = { max_turns: 3 }` would NOT get `turns`, `actions`, etc. from the Guard trait. This unit includes a task (Task 1) to fix this in the interpreter BEFORE writing traits.

The fix is in `exec_stmt.rs` ClassDecl evaluation (lines 116-137). After `inject_traits` copies method defaults into `method_map`, also iterate each trait's `fields` and inject their defaults into `defaults_map` (the class field defaults). Class-declared fields take precedence (only inject if the class didn't already declare the field).

The exact change — after line 127 (`Self::inject_traits(...)`) and before line 128 (`let val = LxVal::Class(...)`), add:

```rust
for tn in &data.traits {
    if let Some(LxVal::Trait(t)) = self.env.get(*tn) {
        for f in t.fields.iter() {
            if let Some(ref default) = f.default {
                if !defaults_map.contains_key(&f.name) {
                    defaults_map.insert(f.name, default.clone());
                }
            }
        }
    }
}
```

This is safe because: (1) trait field defaults are already-evaluated `LxVal`, (2) Store values get cloned per-instance in `apply.rs:92-96`, (3) no existing traits have field defaults so existing code is unaffected.

# What Changes

Nine .lx files in `std/` (6 new + 3 copies from pkg). All traits exported with `+Trait`. Plus interpreter fix and module resolver update.

# Files Affected

- `crates/lx/src/interpreter/exec_stmt.rs` — Fix trait field default inheritance
- `crates/lx/src/stdlib/mod.rs` — Add lx_std_module_source, update std_module_exists
- `crates/lx/src/interpreter/modules.rs` — Add load_module_from_source, update eval_use
- `std/tool.lx` — New file
- `std/prompt.lx` — New file
- `std/session.lx` — New file
- `std/guard.lx` — New file
- `std/workflow.lx` — New file
- `std/schema.lx` — New file
- `std/agent.lx` — New file (copy from pkg/agent.lx)
- `std/connector.lx` — New file (copy from pkg/core/connector.lx)
- `std/collection.lx` — New file (copy from pkg/core/collection.lx, add entries field)
- `tests/trait_tool.lx` — New test
- `tests/trait_prompt.lx` — New test
- `tests/trait_session.lx` — New test
- `tests/trait_guard.lx` — New test
- `tests/trait_workflow.lx` — New test
- `tests/trait_schema.lx` — New test

# Task List

### Task 1: Fix trait field default inheritance in interpreter

**Subject:** Make inject_traits also copy trait field defaults into class field defaults

**Description:** Edit `crates/lx/src/interpreter/exec_stmt.rs`. In the `Stmt::ClassDecl` arm (lines 116-137), after the `inject_traits` call at line 127 and before the `LxVal::Class` construction at line 128, add:

```rust
for tn in &data.traits {
    if let Some(LxVal::Trait(t)) = self.env.get(*tn) {
        for f in t.fields.iter() {
            if let Some(ref default) = f.default {
                if !defaults_map.contains_key(&f.name) {
                    defaults_map.insert(f.name, default.clone());
                }
            }
        }
    }
}
```

This iterates the class's declared traits, looks up each trait in the environment, and for each trait field that has a default value, inserts it into the class's `defaults_map` IF the class didn't already declare that field. Class-declared fields always take precedence.

Store values in trait field defaults are safe because `apply.rs:92-96` already clones Store values per-instance during Object instantiation.

Verify: write a quick test that a class implementing a trait with field defaults inherits those defaults without redeclaring them. Something like:

```lx
Trait HasDefault = {
  x: Int = 42
  get_x = () { self.x }
}
Class Simple : [HasDefault] = {}
s = Simple {}
assert s.x == 42
assert (s.get_x ()) == 42
```

This test should pass after the fix and fail before it.

**ActiveForm:** Fixing trait field default inheritance

---

### Task 2: Add .lx-based std module support

**Subject:** Enable std/ modules written in lx, embedded in the binary via include_str

**Description:** The keyword traits must live in `std/` (e.g., `use std/tool {Tool}`). Currently all std modules are Rust-implemented. Add support for .lx-based std modules.

**Step 1:** Create `std/` directory at repo root. This will hold .lx files for language-level traits.

**Step 2:** Edit `crates/lx/src/stdlib/mod.rs`. Add a function that returns embedded .lx source for a given module name:

```rust
fn lx_std_module_source(name: &str) -> Option<&'static str> {
    match name {
        "agent" => Some(include_str!("../../../../std/agent.lx")),
        "tool" => Some(include_str!("../../../../std/tool.lx")),
        "prompt" => Some(include_str!("../../../../std/prompt.lx")),
        "connector" => Some(include_str!("../../../../std/connector.lx")),
        "collection" => Some(include_str!("../../../../std/collection.lx")),
        "session" => Some(include_str!("../../../../std/session.lx")),
        "guard" => Some(include_str!("../../../../std/guard.lx")),
        "workflow" => Some(include_str!("../../../../std/workflow.lx")),
        "schema" => Some(include_str!("../../../../std/schema.lx")),
        _ => None,
    }
}
```

The `include_str!` path is relative to the .rs file's location (`crates/lx/src/stdlib/mod.rs`), so `../../../../std/` reaches the repo root `std/` directory. Verify this path is correct.

Update `std_module_exists()` to also check `lx_std_module_source`:

```rust
pub(crate) fn std_module_exists(path: &[&str]) -> bool {
    if path.len() < 2 || path[0] != "std" { return false; }
    // Check Rust modules first
    matches!(path[1], "channel" | "checkpoint" | /* ... existing ... */ | "schema")
    // Then check .lx modules
    || lx_std_module_source(path[1]).is_some()
}
```

**Step 3:** Edit `crates/lx/src/interpreter/modules.rs`. Add a method to load a module from source string (not file path):

```rust
async fn load_module_from_source(&mut self, name: &str, source: &str, span: SourceSpan) -> Result<ModuleExports, LxError> {
    let (tokens, comments) = crate::lexer::lex(source)
        .map_err(|e| LxError::runtime(format!("std/{name}: {e}"), span))?;
    let result = crate::parser::parse(tokens, crate::source::FileId::new(0), comments, source);
    let surface = result.program
        .ok_or_else(|| LxError::runtime(format!("std/{name}: parse error"), span))?;
    let program = desugar(surface);
    let mut mod_interp = Interpreter::new(source, None, Arc::clone(&self.ctx));
    mod_interp.module_cache = Arc::clone(&self.module_cache);
    mod_interp.loading = Arc::clone(&self.loading);
    mod_interp.exec(&program).await
        .map_err(|e| LxError::runtime(format!("std/{name}: {e}"), span))?;
    Ok(collect_exports(&program, &mod_interp))
}
```

**Step 4:** In `eval_use()`, add a branch between the Rust std check and the workspace check:

```rust
let exports = if crate::stdlib::std_module_exists(&str_path) {
    if let Some(rust_exports) = crate::stdlib::get_std_module(&str_path) {
        rust_exports
    } else if let Some(lx_source) = crate::stdlib::lx_std_module_source(str_path[1]) {
        self.load_module_from_source(str_path[1], lx_source, span).await?
    } else {
        return Err(LxError::runtime(format!("unknown stdlib module: {}", str_path.join("/")), span));
    }
} else if let Some(file_path) = ...
```

This requires making `lx_std_module_source` public: `pub(crate) fn lx_std_module_source`.

**Step 5:** Verify with a minimal test. Create `std/agent.lx` by copying `pkg/agent.lx` content. Write a test that does `use std/agent {Agent}` and verify it works.

Note: The existing `pkg/agent.lx` remains for backward compatibility. Programs can use either `use pkg/agent {Agent}` or `use std/agent {Agent}`. The keyword desugarer will generate `use std/agent {Agent}`.

**ActiveForm:** Adding .lx-based std module support

---

### Task 3: Write Tool trait

**Subject:** Create std/tool.lx

**Description:** Create `std/tool.lx` with the following exact content:

```lx
+Trait Tool = {
  description: Str = ""
  params: Record = {}

  run = (args) { Err "Tool.run not implemented" }

  schema = () {
    props = self.params | keys | fold {} (acc k) {
      json_type = lx_type_to_json (self.params.get k)
      {..acc, (k): {type: json_type}}
    }
    required = self.params | keys
    {type: "object", properties: props, required: required}
  }

  validate = (args) {
    missing = self.params | keys | filter (k) { not (args | keys | any? (== k)) }
    (missing | len) == 0 ? Ok args : Err {missing: missing}
  }
}

lx_type_to_json = (t) {
  t ? {
    "Int" -> "integer"
    "Float" -> "number"
    "Str" -> "string"
    "Bool" -> "boolean"
    "List" -> "array"
    _ -> "object"
  }
}
```

Note: The `params` field stores a record mapping parameter names to lx type names (strings like "Str", "Int"). The `schema()` default returns a JSON-schema-compatible record: `{type: "object", properties: {text: {type: "string"}}, required: ["text"]}`. The helper `lx_type_to_json` maps lx type names to JSON Schema type strings. The `validate` default checks that every key in params exists in args.

**ActiveForm:** Writing Tool trait

---

### Task 3: Write Prompt trait

**Subject:** Create std/prompt.lx

**Description:** Create `std/prompt.lx`. The Prompt trait wraps the functional API from `pkg/core/prompt.lx` into a trait with self-referencing methods. Read `pkg/core/prompt.lx` first — the existing render logic is:

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

  add_section = (name content) {
    self.sections <- [..self.sections {name: name, content: content}]
    self
  }

  add_constraint = (text) {
    self.constraints <- [..self.constraints text]
    self
  }

  add_instruction = (text) {
    self.instructions <- [..self.instructions text]
    self
  }

  compose = (other) {
    (other.system | len) > 0 ? (self.system <- other.system)
    self.sections <- [..self.sections ..other.sections]
    self.constraints <- [..self.constraints ..other.constraints]
    self.instructions <- [..self.instructions ..other.instructions]
    self.examples <- [..self.examples ..other.examples]
    self
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

Note: `with_section`, `with_constraint`, `with_instruction`, `compose` MUTATE self and return self. They cannot return `{..self, field: new}` record spreads because self is an Object — the returned record would lose trait methods and `p2.render()` would fail. Mutation + return self preserves the Object identity and all methods. The `ask` and `ask_with` methods call `ai.prompt` / `ai.prompt_with` which are globals provided by the runtime — no `use std/ai` needed at the trait level because traits are evaluated in the consuming module's scope.

**ActiveForm:** Writing Prompt trait

---

### Task 4: Write Session trait

**Subject:** Create std/session.lx

**Description:** Create `std/session.lx`:

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

### Task 5: Write Guard trait

**Subject:** Create std/guard.lx

**Description:** Create `std/guard.lx`. This is a generalization of the existing CircuitBreaker in `pkg/core/circuit.lx`. The logic is extracted from CircuitBreaker's methods (already verified — see `circuit.lx` source). Write:

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

### Task 6: Write Workflow trait

**Subject:** Create std/workflow.lx

**Description:** Create `std/workflow.lx`. Each step is a record `{id: Str, run: Func, depends: [Str], undo: Func?}`. The `run` method does topological sort then sequential execution. Write:

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

### Task 7: Write Schema trait

**Subject:** Create std/schema.lx

**Description:** Create `std/schema.lx`. The Schema trait provides base `validate` and `schema` methods. Note: `TraitDeclData.requires` does NOT work at runtime — the `requires` field is stored but `inject_traits()` never reads it. So Schema cannot be inherited via `requires`. When the Schema keyword desugars (Unit 3), it directly injects richer `schema()` and `validate()` that include field descriptions and JSON-schema-compatible output with `type`, `properties` (each with `type` + `description`), and `required` array. This base trait exists for manual use and provides the method signatures:

```lx
+Trait Schema = {
  validate = (data) {
    (type_of data) != "Record" ? (Err {error: "expected Record, got {type_of data}"}) : (Ok data)
  }

  schema = () {
    {type: "object", properties: {}, required: []}
  }
}
```

The base `schema()` returns a minimal JSON-schema envelope. The Schema keyword desugaring (Unit 3) overrides this with generated implementations that populate `properties` with per-field `{type, description}` records and `required` with field name strings. Type mapping: `Int` → `"integer"`, `Float` → `"number"`, `Str` → `"string"`, `Bool` → `"boolean"`, `List` → `"array"`, other → `"object"`.

**ActiveForm:** Writing Schema trait

---

### Task 8: Write tests for all new traits

**Subject:** Create test files validating each trait

**Description:** Create six test files:

`tests/trait_tool.lx`:
```lx
use std/tool {Tool}

Class Echo : [Tool] = {
  description: "echoes input"
  params: {text: "Str"}
  run = (args) { Ok args.text }
}

t = Echo {}
assert t.description == "echoes input"
s = t.schema ()
assert s.type == "object"
assert s.properties.text.type == "string"
assert (s.required | len) == 1
result = t.run {text: "hello"}
assert result == Ok "hello"
valid = t.validate {text: "hi"}
assert (valid | ok?)
invalid = t.validate {}
assert (invalid | err?)
```

`tests/trait_prompt.lx`:
```lx
use std/prompt {Prompt}

Class Greeter : [Prompt] = {
  system: "You greet people"
}

p = Greeter {}
p.add_section "Name" "Alice"
rendered = p.render ()
assert (rendered | contains? "You greet people")
assert (rendered | contains? "Alice")
```

Note: Greeter only declares `system`. The other fields (`sections`, `constraints`, `instructions`, `examples`) are inherited from the Prompt trait via the trait field default inheritance fix (Task 1).

`tests/trait_session.lx`:
```lx
use std/session {Session}

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

Note: Chat only overrides `max_tokens`. `messages`, `checkpoints`, `next_id`, `compaction_threshold` are inherited from Session trait field defaults.

`tests/trait_guard.lx`:
```lx
use std/guard {Guard}

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
use std/workflow {Workflow}

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
use std/schema {Schema}

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

### Task 9: Update Collection trait and verify existing traits

**Subject:** Add `entries` field default to Collection trait, verify Agent and Connector

**Description:** The Collection trait currently declares methods only (get, keys, values, etc.) that reference `self.entries`, but does NOT declare `entries` as a field with a default. With the trait field inheritance fix (Task 1), adding `entries: Store = Store()` to Collection will make it automatically available to implementing classes. This eliminates the need for Store keyword to special-case field injection.

Create `std/collection.lx` by copying `pkg/core/collection.lx` content. Add an `entries` field declaration with default `Store()`:

```lx
+Trait Collection = {
  entries: Store = Store ()

  get = (key) { self.entries.get key }
  keys = () { self.entries.keys () }
  values = () { self.entries.values () }
  remove = (key) { self.entries.remove key }
  query = (pred) { self.entries.values () | filter pred }
  len = () { self.entries.len () }
  has = (key) { self.entries.has key }
  save = (path) { self.entries.save path }
  load = (path) { self.entries.load path }
}
```

Verify existing code that uses `Class X : [Collection] = { entries: Store() }` still works — the class-declared `entries` takes precedence over the trait default (the inheritance fix uses `if !defaults_map.contains_key`). So existing code is unaffected.

Read `pkg/agent.lx` — Agent has methods only, no fields. No changes needed.
Read `pkg/core/connector.lx` — Connector has methods only, no fields. No changes needed.

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
