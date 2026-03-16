# Structured AI Output — Reference

## `ai.prompt_structured`

LLM generation validated against a Protocol schema, with automatic retry on violation.

```
ai.prompt_structured protocol prompt -> Result a StructuredErr
ai.prompt_structured_with protocol opts -> Result a StructuredErr
```

### Basic Usage

```
Protocol AnalysisResult = {findings: [Str]  severity: Str  confidence: Float}

result = ai.prompt_structured AnalysisResult "analyze this code for bugs" ^
```

### `ai.prompt_structured_with` Options

Accepts all `ai.prompt_with` options plus:

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `max_retries` | Int | 2 | Retry attempts on schema violation |
| `examples` | [a] | [] | Example outputs matching the Protocol (few-shot) |
| `strict` | Bool | true | If false, allow extra fields beyond Protocol |

### Schema Injection

The Protocol is appended to the prompt as a schema instruction:

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

### Retry Behavior

On validation failure, retries with error context appended to the continued session:

```
Your previous response did not match the required schema.
Error: missing field "confidence"
Your response: {"findings": ["bug1"], "severity": "high"}

Please respond again with a valid JSON object matching the schema.
```

Returns `Err {reason: Str  raw: Str  attempts: Int}` if all retries exhausted.

### Example

```
result = ai.prompt_structured AnalysisResult task ^
result = ai.prompt_structured AnalysisResult task ^ | (.findings) | filter (.critical)

results = items | map (item) {
  ai.prompt_structured ItemReview "review {item.name}" ^
}
```
