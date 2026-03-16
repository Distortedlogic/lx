# Structured AI Output via Protocol

`ai.prompt_structured` bridges Protocol validation with LLM generation. The LLM generates output matching a declared Protocol schema, with automatic retry on schema violation.

## Problem

Every standard agent does manual JSON parsing of LLM responses:

```
raw = ai.prompt_with {prompt: task} ^
parsed = json.parse (ai.extract_llm_text raw.response) ^
parsed.score ?? 0
```

This is fragile — the LLM might return markdown-wrapped JSON, missing fields, wrong types, or free text. Every agent has its own parsing/validation/retry logic. `ai::parse_llm_json` and `ai::strip_json_fences` exist precisely because this is unreliable.

## Solution

```
Protocol AnalysisResult = {findings: [Str]  severity: Str  confidence: Float}

result = ai.prompt_structured AnalysisResult "analyze this code for bugs" ^
```

`ai.prompt_structured` is a new function in `std/ai`. It:
1. Injects the Protocol's field names and types into the prompt as a schema instruction
2. Calls `ai.prompt_with` with the augmented prompt
3. Parses the response as JSON
4. Validates the parsed result against the Protocol
5. On validation failure, retries with the error message appended (up to `max_retries`)
6. Returns `Ok result` or `Err {reason: Str  raw: Str  attempts: Int}`

### Full Signature

```
ai.prompt_structured protocol prompt -> Result a StructuredErr
ai.prompt_structured_with protocol opts -> Result a StructuredErr
```

The `_with` variant accepts the same options as `ai.prompt_with` plus:

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `max_retries` | Int | 2 | Retry attempts on schema violation |
| `examples` | [a] | [] | Example outputs matching the Protocol (few-shot) |
| `strict` | Bool | true | If false, allow extra fields beyond Protocol |

### Schema Injection

The Protocol is serialized as a schema instruction appended to the prompt:

```
{original_prompt}

Respond with a JSON object matching this exact schema:
{
  "findings": [string],
  "severity": string,
  "confidence": number (float)
}
```

Type mapping: `Str` -> `string`, `Int` -> `integer`, `Float` -> `number (float)`, `Bool` -> `boolean`, `[T]` -> `[T_mapped]`, nested records -> nested objects.

### Retry on Failure

When validation fails, the retry prompt includes:

```
Your previous response did not match the required schema.
Error: missing field "confidence"
Your response: {"findings": ["bug1"], "severity": "high"}

Please respond again with a valid JSON object matching the schema.
```

The session is continued (not restarted) so the LLM has context.

### Composition

```
result = ai.prompt_structured AnalysisResult task ^
result = ai.prompt_structured AnalysisResult task ^ | (.findings) | filter (.critical)

results = items | map (item) {
  ai.prompt_structured ItemReview "review {item.name}" ^
}
```

### With Skill

Skills declare output Protocols. `skill.run` can use `ai.prompt_structured` internally:

```
Skill analyze = {
  description: "Analyze code for bugs"
  input: {code: Str  language: Str}
  output: AnalysisResult
  requires: [:ai]
  handler: (input) {
    ai.prompt_structured AnalysisResult
      "Analyze this {input.language} code:\n{input.code}" ^
  }
}
```

### With refine

```
result = refine (ai.prompt_structured Plan task ^) {
  grade: (work) -> ai.prompt_structured PlanGrade "evaluate: {work}" ^
  revise: (work feedback) -> ai.prompt_structured Plan "revise: {work}\nfeedback: {feedback}" ^
  threshold: 85
  max_rounds: 3
}
```

## Implementation

Library function in `std/ai`, not a keyword. Uses existing `ai.prompt_with` + Protocol validation machinery. The Protocol's field/type information is already available at runtime (used by `~>` validation). Schema serialization is a new utility function.

### Dependencies

- Protocol runtime type info (already exists for `~>` validation)
- `ai.prompt_with` (exists)
- `json.parse` (exists)
- `ai::strip_json_fences` (exists)

## Cross-References

- Protocol declarations: [agents-protocol.md](agents-protocol.md)
- AI module: stdlib_roadmap (`std/ai`)
- Skill declarations: [agents-skill.md](agents-skill.md) (skills use structured output for typed handlers)
- Refinement: [agents-refine.md](agents-refine.md) (refine loops benefit from structured grading)
