# lx Testing

Complete reference for writing tests in lx — both unit tests for the language itself and satisfaction tests for lx programs.

## Part 1: lx Language Unit Tests

### How `lx test` Works

`lx test tests/` discovers all `.lx` files (and `main.lx` inside subdirectories) under the given path, runs each one, and reports pass/fail. A test file passes if all `assert` statements succeed. A failing assert is caught per-file (doesn't crash the whole suite).

```bash
just test              # runs: cargo run -p lx-cli -- test tests/
lx test tests/         # same thing via installed binary
lx test tests/09_errors.lx  # single file
```

### Writing Unit Tests

Each test file is a regular `.lx` program that uses `assert`:

```lx
assert (42 == 42)
assert (42 == 42) "optional message"
assert ("hello" | len == 5) "len of hello"
```

Test files begin with a `--` comment header noting what they test. No special setup — just assertions.

### File Naming Convention

```
tests/
  01_literals.lx        -- language primitives
  02_bindings.lx        -- =, :=, <-
  ...
  26_ai.lx              -- std/ai module
  28_audit.lx           -- std/audit module
  35_agents_grader.lx   -- std/agents/grader
  fixtures/             -- helper files for multi-file tests
```

Numbers indicate order of addition, not execution order. All files run independently.

### Testing Stdlib Modules

Import the module and assert against its functions:

```lx
use std/audit
assert (audit.is_empty "" == true) "is_empty: empty string"
assert (audit.is_empty "hello" == false) "is_empty: non-empty"
assert (audit.is_hedging "I think this might work" == true) "is_hedging: detected"
```

For `std/agents/grader`, use `grader.quick_grade` (keyword-matching, no LLM) for deterministic unit tests:

```lx
use std/agents/grader

rubric = [
  {name: "correctness" description: "fixes the bug and handles edge cases" weight: 3}
  {name: "testing" description: "includes unit tests" weight: 2}
]

result = grader.quick_grade {
  work: "The solution correctly fixes the bug and handles edge cases. It includes unit tests."
  task: "fix the bug"
  rubric: rubric
}
assert (result.passed == true)
assert (result.score > 0)
assert (len result.categories == 2)
```

`grader.quick_grade` uses keyword overlap scoring (no AI call). `grader.grade` uses live AI — gate it behind an env var:

```lx
use std/env
run_live = env.get "LX_TEST_AI" ?? ""
run_live != "" ? {
  true -> {
    result = grader.grade { work: "..." task: "..." rubric: rubric threshold: 70 }
    assert (result.score > 0)
  }
  false -> ()
}
```

### Testing AI Calls

AI tests are expensive. Gate behind `LX_TEST_AI`:

```lx
use std/ai
use std/env

run_live = env.get "LX_TEST_AI" ?? ""
run_live != "" ? {
  true -> {
    response = ai.prompt "What is 2+2? Reply with ONLY the digit." ^
    assert (len response > 0)
  }
  false -> ()
}
```

### Multi-File Module Tests

Put tests in a subdirectory with `main.lx` as the entry point:

```
tests/11_modules/
  main.lx          -- test entry point, uses ./lib_math, ./lib_types
  lib_math.lx      -- module under test
  lib_types.lx     -- module under test
```

`lx test` discovers `main.lx` inside directories and runs it.

### Test Fixtures

Helper files (echo agents, mock servers) go in `tests/fixtures/`:

```
tests/fixtures/
  agent_echo.lx             -- echo agent for std/agent tests
  mcp_test_server.py        -- MCP stdio server for std/mcp tests
```

Referenced from test files via relative import: `use ./fixtures/agent_echo`.

## Part 2: Satisfaction Testing (Spec + Scenarios)

For testing lx programs that produce AI-generated output (like workgen). The pattern: run the program for real, then grade the output with an AI-backed grader.

### Architecture

| File | Role |
|------|------|
| `tests/spec.lx` | Rubric, threshold, scenario list |
| `tests/run.lx` | Imports spec + program under test, runs scenarios, grades output |
| `tests/fixtures/<name>/` | Input fixtures with planted violations |

### Spec Structure (`tests/spec.lx`)

Exports:

```lx
+name = "suite name"
+threshold = 0.75
+threshold_pct = 75

+rubric = audit.rubric [
  {name: "structure"  description: "has required sections"  weight: 25}
  {name: "coverage"   description: "all findings addressed"  weight: 30}
  {name: "compliance" description: "follows process rules"   weight: 25}
  {name: "safety"     description: "substantive, not empty"  weight: 20}
]

+scenarios = [
  {
    name: "scenario-name"
    audit: "path/to/audit-list.md"
    root: "path/to/fixture"
    expected_findings: ["finding1" "finding2"]
    tags: ["smoke"]
  }
]
```

### Scenario Record Fields

| Field | Type | Purpose |
|-------|------|---------|
| `name` | Str | Display name |
| `audit` | Str | Path to the audit list file |
| `root` | Str | Path to fixture directory with source code |
| `expected_findings` | [Str] | Semantic keywords for grader coverage evaluation |
| `tags` | [Str] | For filtering with `TEST_TAG` env var |

### Runner Structure (`tests/run.lx`)

```lx
use std/fs
use std/env
use std/trace
use std/agents/grader
use ../main : program_under_test
use ./spec

run_scenario = (scenario) {
  result = program_under_test.run scenario.audit "rules.md" scenario.root
  result ? {
    Ok r -> {
      output = fs.read r.path ?? ""
      grade_result = grader.grade {
        work: output
        task: "task covering: {scenario.expected_findings | join ", "}"
        rubric: spec.rubric
        threshold: spec.threshold_pct
      }
      sn = scenario.name
      {name: sn  passed: grade_result.passed  score: grade_result.score}
    }
    Err e -> {
      sn = scenario.name
      {name: sn  passed: false  score: 0.0}
    }
  }
}

+main = () {
  tag_filter = env.get "TEST_TAG" ?? ""
  scenarios = tag_filter == "" ? {
    true -> spec.scenarios
    false -> spec.scenarios | filter (s) s.tags | contains? tag_filter
  }
  results = scenarios | pmap_n 4 run_scenario
  passed_count = results | filter (.passed) | len
  passed_count == (results | len) ? (Ok results) : (Err "some failed")
}

main ()
```

### Grading: `grader.grade` vs `grader.quick_grade`

| Function | How it works | When to use |
|----------|-------------|-------------|
| `grader.grade` | Makes an AI call, semantic evaluation | Satisfaction tests with real AI output |
| `grader.quick_grade` | Keyword overlap, no AI | Unit tests, deterministic checks |

`grader.grade` returns `{score passed categories feedback failed}`:
- `score`: 0-100 weighted average
- `passed`: Bool — true only if score >= threshold AND every category >= 70
- `categories`: list of `{name score passed feedback}` per rubric category
- `feedback`: summary string
- `failed`: list of failed category names

### Parallel Execution

Scenarios are independent — different inputs, different outputs. Use `pmap_n N` to run N concurrently:

```lx
results = scenarios | pmap_n 4 run_scenario    -- 4 concurrent
results = scenarios | pmap run_scenario         -- all concurrent
results = scenarios | map run_scenario          -- sequential
```

Each scenario makes 3 AI calls (investigate + compose + grade). Choose concurrency level based on API rate limits.

### Tag Filtering

```bash
TEST_TAG=smoke lx run tests/run.lx     -- only scenarios tagged "smoke"
lx run tests/run.lx                     -- all scenarios
```

Implemented in the runner:
```lx
tag_filter = env.get "TEST_TAG" ?? ""
scenarios = tag_filter == "" ? {
  true -> spec.scenarios
  false -> spec.scenarios | filter (s) s.tags | contains? tag_filter
}
```

## Part 3: AI Call Configuration

### The Claude CLI Backend

All `ai.prompt` / `ai.prompt_with` calls go through the Claude Code CLI (`claude -p --output-format json`). The CLI has its own system prompt that makes the model behave as a conversational assistant with tool access. This affects programmatic AI calls.

### `ai.prompt` vs `ai.prompt_with`

| Function | Returns | System prompt | Tools | Use case |
|----------|---------|---------------|-------|----------|
| `ai.prompt text` | Str | Claude Code default | All | Simple one-off prompts |
| `ai.prompt_with {prompt system tools ...}` | Record `{text session_id cost turns duration_ms model}` | Configurable | Configurable | Controlled AI calls |

`ai.prompt_with` returns `Ok(Record)`. Use `^` to unwrap, then `.text` for the response string.

### Options Record for `ai.prompt_with`

| Field | Type | CLI flag | Purpose |
|-------|------|----------|---------|
| `prompt` | Str | stdin | The prompt text (required) |
| `system` | Str | `--system-prompt` | Replaces Claude Code system prompt entirely |
| `append_system` | Str | `--append-system-prompt` | Adds to Claude Code system prompt |
| `tools` | [Str] | `--allowedTools` | Restricts available tools. `[]` = no tools |
| `max_turns` | Int | `--max-turns` | Limits conversation turns |
| `model` | Str | `--model` | Model override |
| `resume` | Str | `--resume` | Resume a previous session by ID |

### `system:` vs `append_system:`

- **`system:`** — replaces the Claude Code system prompt. The model loses all assistant behavior and tool awareness. Use for pure text/document generation where you want zero conversational responses.
- **`append_system:`** — adds to the Claude Code system prompt. The model keeps its instruction-following behavior. Use for structured output (JSON) where you want the base prompt's discipline plus your role constraints.

### `tools: []` — Why It Matters

Without tool restriction, the model can read files, run shell commands, etc. It may spend all its turns on tool calls and return empty text. With `tools: []`, the model has no tools and must respond with text immediately.

**Always set `tools: []` when all context is already in the prompt.** This is the root fix for:
- Empty responses (model used turns on tool calls)
- Conversational responses ("I need write permission...")
- Slow responses (model doing unnecessary file reads)

### Patterns for Common AI Call Types

**Pure text generation (investigation, composition):**
```lx
resp = ai.prompt_with {
  prompt: rendered_prompt
  system: "You are a [role]. Output [format] directly. No conversation."
  tools: []
} ^
out = resp.text
```

**Structured JSON from stdlib agents (grader, auditor, router):**
Configured in lx source at `crates/lx/src/stdlib/agents_*.rs`:
```rust
let opts = AiOpts {
    append_system: Some(system),  // append, don't replace
    max_turns: Some(1),
    tools: Some(vec![]),          // no tools
    ..AiOpts::default()
};
```

**AI-assisted with tool access (when the model needs to read files):**
```lx
resp = ai.prompt_with {
  prompt: "Analyze this codebase for issues"
  append_system: "Focus on error handling patterns"
  tools: ["Read" "Glob" "Grep"]
  max_turns: 5
} ^
```

## Part 4: RuntimeCtx Backend Architecture

All I/O in lx goes through swappable backends in `RuntimeCtx`:

| Backend | Trait | Default | Purpose |
|---------|-------|---------|---------|
| `ai` | `AiBackend` | `ClaudeCodeAiBackend` | LLM calls via Claude CLI |
| `emit` | `EmitBackend` | `StdoutEmitBackend` | Agent-to-human output |
| `http` | `HttpBackend` | `ReqwestHttpBackend` | HTTP requests |
| `shell` | `ShellBackend` | `ProcessShellBackend` | Shell command execution |
| `yield_` | `YieldBackend` | `StdinStdoutYieldBackend` | Coroutine orchestration |
| `log` | `LogBackend` | `StderrLogBackend` | Logging |
| `user` | `UserBackend` | `NoopUserBackend` | User interaction (confirm/choose/ask) |

Backends are `Arc<dyn Trait>` — swap them in Rust for testing or sandboxing:

```rust
let ctx = RuntimeCtx {
    ai: Arc::new(MockAiBackend { responses: vec!["mocked response"] }),
    ..RuntimeCtx::default()
};
let mut interp = Interpreter::new(source, source_dir, Arc::new(ctx));
```

The `AiBackend` trait has one method:
```rust
fn prompt(&self, text: &str, opts: &AiOpts, span: Span) -> Result<Value, LxError>;
```

`AiOpts` fields: `system`, `model`, `max_turns`, `resume`, `tools`, `append_system`. The `ClaudeCodeAiBackend` translates these to Claude CLI flags.

Response format from the backend: `Ok(Value::Ok(Record { text, session_id, cost, turns, duration_ms, model }))` on success, `Ok(Value::Err(Record { msg, subtype? }))` on AI error.

## Part 5: Key Stdlib APIs for Testing

### `std/prompt` — Prompt Builder

Build structured prompts for AI calls without string concatenation:

```lx
use std/prompt

p = prompt.create ()
  | prompt.system "You are a code auditor."
  | prompt.section "Code" source_code
  | prompt.instruction "Find all violations"
  | prompt.constraint "Report ONLY problems"

text = prompt.render p         -- renders to a single string
tokens = prompt.estimate p     -- approximate token count
```

`prompt.render` flattens everything into one string. When using `ai.prompt_with`, render the prompt for the `prompt:` field and pass the system instruction separately via `system:` or `append_system:` — do NOT use `prompt.system` when you need `--system-prompt` CLI flag behavior.

### `std/trace` — Session Tracing

Record test execution data for post-run analysis:

```lx
use std/trace

session = trace.create "/tmp/trace.json" ^
trace.record {name: "step1"  score: 0.85  output: "text"} session ^
should_stop = trace.should_stop {min_delta: 2.0  window: 3} session
rate = trace.improvement_rate 3 session
summary = trace.summary session
```

`trace.record` is data-last (session is last arg). `trace.should_stop` detects diminishing returns across recorded scores.

### `std/audit` — Structural Quality Checks

```lx
use std/audit

audit.is_empty str              -- Bool: whitespace-only or empty
audit.is_hedging str            -- Bool: contains hedging language
audit.is_refusal str            -- Bool: contains refusal patterns
audit.is_too_short str min      -- Bool: under min chars
audit.is_repetitive str         -- Bool: repeated content

audit.rubric [{name description weight}]  -- validates rubric structure, returns the list
audit.quick_check {output task}            -- returns {passed reasons}
```

`audit.rubric` validates that each record has `name` (Str), `description` (Str), `weight` (Int). Returns the same list if valid, errors if not.

### `std/md` — Markdown Parsing

```lx
use std/md

parsed = md.parse markdown_string
headings = md.headings parsed    -- [{type: "heading"  level: Int  text: Str}]
sections = md.sections parsed    -- [{type: "section"  level: Int  title: Str  content: Str}]
```

`md.headings` returns records, not strings. Always access `.text` for the heading text.

### `std/agents/grader` — Scoring

```lx
use std/agents/grader

-- AI-backed (live LLM call, semantic evaluation)
result = grader.grade {
  work: output_text
  task: "description of what to evaluate"
  rubric: [{name description weight}]
  threshold: 75
  previous_grades: []
}

-- Keyword-based (no LLM, deterministic)
result = grader.quick_grade {
  work: output_text
  task: "description"
  rubric: [{name description weight}]
}
```

Both return `{score passed categories feedback failed}`. `grader.grade` uses `append_system` + `tools: []` + `max_turns: 1` internally. `grader.quick_grade` does keyword overlap with zero AI.

### Module Import Patterns

```lx
use std/fs                    -- stdlib module, access as fs.read, fs.write
use std/agents/grader         -- nested stdlib, access as grader.grade
use ./spec                    -- relative import, access as spec.name, spec.scenarios
use ../main : workgen         -- parent-relative with alias, access as workgen.run
use std/json {parse encode}   -- selective import, access as parse, encode directly
```

Module files execute on import. Top-level `main ()` calls in imported modules WILL execute. Keep entry points in separate files from importable modules:

```
main.lx      -- exports +run, +main (functions defined but NOT called)
run.lx       -- imports main.lx, calls main() at bottom
tests/run.lx -- imports main.lx, calls workgen.run() per scenario
```

### Function Export Pattern

```lx
+exported_fn = (x) x * 2      -- + prefix = importable by other modules
private_fn = (x) x + 1        -- no prefix = private to this file
+main = () { ... }             -- exported but NOT auto-executed
main ()                        -- explicit call at bottom of entry point file
```

### Error Handling in Tests

```lx
-- ^ unwraps Ok/Some, propagates Err/None to function boundary
content = fs.read path ^

-- ?? coalesces Err/None to a default
content = fs.read path ?? ""

-- ? pattern match on Result
result ? {
  Ok value -> use_value value
  Err e -> handle_error e
}

-- ^ at function boundary: caught and converted to return value
my_func = () {
  data = fs.read "missing.txt" ^   -- if Err, my_func returns Err(...)
  process data
}
result = my_func ()                -- result is Ok(...) or Err(...)
```

## Part 6: Workgen File Layout

```
workgen/
  main.lx              -- core program, exports +run and +main
  run.lx               -- entry point, calls main()
  rules/               -- audit lists + work-item process rules
    rust-audit.md
    python-audit.md
    ...
    work-item.md        -- multi-phase process template
  tests/
    spec.lx             -- rubric, threshold, scenarios
    run.lx              -- test runner, imports workgen + spec
    fixtures/
      rust_audit/src/   -- Rust code with planted violations
      python_audit/src/ -- Python code with planted violations
      ...
```

### Justfile Recipes

```bash
just audit              -- interactive fzf chooser over workgen/rules/*audit*
just audit-file FILE    -- run workgen against specific audit file
just audit-test         -- run satisfaction tests (all scenarios)
just audit-test smoke   -- run only smoke-tagged scenarios
```

## Part 7: Gotchas and Lessons

### `lx run` vs `lx test`

- `lx run file.lx` — executes a single file. Errors crash the process. Used for entry points and satisfaction test runners.
- `lx test dir/` — discovers `.lx` files and `main.lx` in subdirectories, runs each independently. Failing asserts are caught per-file. Reports pass/fail per file with total count.
- `lx check file.lx` — type checks without executing. Validates annotations.

### 300-Line File Limit

No `.lx` or `.rs` file may exceed 300 lines (CLAUDE.md rule). Split files that approach this limit. For tests, each file should test one feature area.

### Type Error Messages

All builtin type errors include the actual type received: `"split: second arg must be Str, got Maybe"`. This makes debugging type mismatches immediate — you see what you passed, not just what was expected.

### Designing Effective Fixtures

- Plant 3-8 violations per fixture. More makes the AI output harder to grade, fewer doesn't exercise coverage.
- Make violations unambiguous — one clear instance of each audit item. Don't make the AI guess.
- Keep source files to 20-60 lines. The AI reads the full source via `gather_context`.
- Name fixtures after the audit list: `rust_audit/`, `python_audit/`, `perf_audit/`.
- Source goes in `src/` subdirectory matching the language: `src/main.rs`, `src/service.py`, `src/styles.css`.

### Expected Findings Keywords

- Use descriptive phrases the grader can match semantically: `"swallowed error"` not `"let _ ="`.
- The grader is AI-backed — it understands synonyms. `"unnecessary clone"` matches output saying `"clone where borrow suffices"`.
- Include the audit item's key concept, not the code pattern. `"intermediate collect into Vec"` not `".collect::<Vec"`.
- These go into the grader's `task` field: `"work item covering: {findings | join ", "}"`.

### Gotchas

- `first` and `last` return `Maybe` (Some/None), not raw values. Unwrap with `^` or `?? default` before piping to string functions like `split`, `replace`, `upper`.
- `md.headings` returns records `{type: "heading" level: Int text: Str}`, not strings. Use `h.text` to access heading text.
- `ai.prompt` returns just a Str. `ai.prompt_with` returns `Ok(Record)` — unwrap with `^` then access `.text`.
- Empty AI responses = model spent turns on tool calls. Fix: `tools: []`.
- Conversational AI responses ("I need permission...") = Claude Code system prompt is dominant. Fix: `system:` override.
- `grader.grade` requires ALL categories >= 70 individually. A high weighted average can still fail.
- `pmap_n N` for parallel execution of independent scenarios. `pmap` for unlimited parallelism. `map` for sequential.
- Record field `name:` in lx record literals is just a string key, never evaluated as a variable. `{name: x}` sets key "name" to value of `x`.
- String interpolation `{expr}` inside strings is lx interpolation. Use backtick raw strings to avoid: `` `no {interpolation} here` ``.
- `use ./module : alias` imports a module with an alias. The module file executes on import — keep entry points (`main ()`) in separate files from importable modules.
- `+fn_name` exports a function. Without `+`, it's private to the module.
- `^` unwraps both `Ok → value` and `Some → value`. Propagates `Err` and `None` as errors to the function boundary.
- Cross-module error spans: lx tracks source text per function via `LxFunc.source_text`. Errors in imported module functions show the correct file and line.
