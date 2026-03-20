# Prompt / Instruction Composition

`std/prompt` provides typed, composable prompt assembly. Agents construct prompts from named sections, few-shot examples, constraints, and templates — then render to a final string or budget-trim to fit a token limit. Replaces ad-hoc string concatenation for LLM input construction.

Distinct from `ai.prompt_structured` (which validates LLM *output* against a Trait). This structures the *input* side.

## Problem

Every agent builds prompts via string interpolation:

```
prompt = "You are a {role}.\n\nTask: {task}\n\nExamples:\n{examples | map format_example | join "\n"}\n\nConstraints:\n{constraints | join "\n- "}"
```

This is fragile (ordering matters, whitespace breaks, sections get lost), not composable (can't merge two prompts), and wastes tokens (no way to trim to fit a budget). Common patterns:

- Building from reusable sections (system + task + constraints + examples)
- Merging base prompt with task-specific additions
- Trimming when context is tight (drop examples first, then constraints)
- Rendering differently for different LLM providers

All hand-rolled every time.

## `std/prompt`

### Creating Prompts

```
use std/prompt

p = prompt.create ()
```

Returns an empty prompt builder. All builder functions return a new prompt (immutable), enabling pipes.

### Adding Sections

```
p = prompt.create ()
  | prompt.system "You are a senior code reviewer"
  | prompt.section :task "Review this diff for security issues:\n{diff}"
  | prompt.section :output_format "Respond as JSON with {findings: [{severity, description, line}]}"
```

`prompt.system` sets the system/role section. `prompt.section` adds a named section. Section names are symbols — they're used for ordering, merging, and selective removal.

### Constraints and Instructions

```
p = p
  | prompt.constraint "Focus on OWASP Top 10 vulnerabilities"
  | prompt.constraint "Ignore style and formatting issues"
  | prompt.instruction "If no issues found, return an empty findings array"
```

`constraint` and `instruction` are sugar for specific section types. They accumulate (multiple constraints form a list).

### Few-Shot Examples

```
p = p
  | prompt.example {
      input: "def login(user, pw): db.query(f'SELECT * FROM users WHERE name={user}')"
      output: "{findings: [{severity: :critical  description: \"SQL injection\"  line: 1}]}"
    }
  | prompt.example {
      input: "def greet(name): return f'Hello, {html.escape(name)}'"
      output: "{findings: []}"
    }
```

Examples are rendered as input/output pairs. They're the first thing trimmed under budget pressure.

### Composing Prompts

```
base = prompt.create ()
  | prompt.system "You are a code reviewer"
  | prompt.constraint "Be concise"

security_overlay = prompt.create ()
  | prompt.section :focus "Focus specifically on security vulnerabilities"
  | prompt.constraint "Flag any unsanitized user input"

full = prompt.compose [base security_overlay]
```

`prompt.compose` merges prompts left-to-right:
- `system` sections concatenate with newline
- Same-named sections concatenate
- Constraints accumulate
- Examples accumulate
- Later prompts' sections appear after earlier ones

### Rendering

```
text = prompt.render p
// => "You are a senior code reviewer\n\nTask:\nReview this diff...\n\nConstraints:\n- Focus on OWASP Top 10...\n..."
```

Default render order: system, task, sections (in insertion order), constraints, instructions, examples, output_format.

### Budget-Aware Rendering

```
text = prompt.render_within p 4000
```

`render_within` trims to fit a token budget. Trim order (configurable):
1. Drop examples (least recent first)
2. Truncate long sections
3. Drop constraints (least recent first)
4. Never drop system or task

```
text = prompt.render_within p 4000 {
  trim_order: [:examples :constraints :sections]
  preserve: [:system :task :output_format]
}
```

### Token Estimation

```
tokens = prompt.estimate p
// => 3200 (approximate token count of rendered prompt)
```

### Section Inspection

```
sections = prompt.sections p
// => [:system :task :focus :output_format]

has_examples = prompt.has_section? p :examples
// => true
```

### Removing Sections

```
p2 = prompt.without p :examples
p3 = prompt.without p [:constraints :examples]
```

## Integration Patterns

### With std/ai

```
p = prompt.create ()
  | prompt.system "You are an analyst"
  | prompt.section :task "Analyze: {data}"

result = ai.prompt (prompt.render p) ^
```

### With ai.prompt_structured

```
p = prompt.create ()
  | prompt.system "You are a grader"
  | prompt.section :task "Grade this work: {work}"
  | prompt.section :output_format "Return {score: Int  feedback: Str}"

result = ai.prompt_structured Grade (prompt.render p) ^
```

### With std/context

```
available = context.usage win | (.available)
text = prompt.render_within p available
```

### Standard Agent Prompts

```
review_prompt = prompt.create ()
  | prompt.system "You are a code reviewer specializing in {language}"
  | prompt.section :task task
  | prompt.constraint "Only flag issues with severity >= medium"
  | prompt.example {input: good_example output: "No issues"}

grading_prompt = prompt.create ()
  | prompt.system "You are a rubric grader"
  | prompt.section :rubric (rubric | map format_criterion | join "\n")
  | prompt.section :work work_to_grade
```

### Reusable Prompt Libraries

```
+base_reviewer = prompt.create ()
  | prompt.system "You are a code reviewer"
  | prompt.constraint "Be specific — cite line numbers"
  | prompt.constraint "Explain why each issue matters"
```

Exported prompts can be imported and composed:

```
use ./prompts/base_reviewer

p = prompt.compose [base_reviewer task_specific_additions]
```

## Implementation

`std/prompt` is a new stdlib module. A prompt is a record with ordered sections:

```
{
  system: Maybe Str
  sections: [{name: Symbol  content: Str}]
  constraints: [Str]
  instructions: [Str]
  examples: [{input: Str  output: Str}]
}
```

Rendering concatenates sections with headers. `render_within` estimates tokens per section and drops lowest-priority sections until the total fits. Token estimation uses the same `chars / 4` approximation as `std/context`.

### Dependencies

- `std/context` (optional — `context.estimate` for accurate token counting)

## Cross-References

- LLM integration: stdlib (`std/ai`) — `prompt.render` feeds `ai.prompt`
- Structured output: [agents-structured-output.md](agents-structured-output.md) — validates output side
- Context capacity: [agents-context-capacity.md](agents-context-capacity.md) — budget-aware rendering
- Standard agents: ROADMAP (`std/agents/*`) — all build prompts internally
