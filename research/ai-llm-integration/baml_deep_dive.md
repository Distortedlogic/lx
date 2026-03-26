# BAML (Boundary ML): Deep Dive

## Identity

BAML ("Basically, A Made-up Language") is a DSL for defining type-safe LLM functions with structured inputs and outputs. Created by Boundary (YC W23), founded by Vaibhav Gupta (ex-Microsoft HoloLens, Google ARCore, D.E. Shaw) and Aaron Villalpando Gonzalez (ex-AWS EC2). 7,828 GitHub stars, Apache-2.0 license, v0.220.0 (March 2026). Written in Rust. Tagline: "Terraform for prompts."

Core thesis: **schema engineering over prompt engineering.** Define the output type and let the compiler generate prompt instructions and the parser. Every prompt becomes a typed function.

## The Typed LLM Function Model

```baml
function ExtractResume(resume_text: string) -> Resume {
  client "openai/gpt-4o"
  prompt #"
    Extract the following information from the resume:
    {{ ctx.output_format }}
    Resume text:
    {{ resume_text }}
  "#
}
```

Compiles to native client code. Python: `result: Resume = await b.ExtractResume(text)`. Generated code handles API calls, output parsing, type validation, and retries. No JSON parsing or prompt-formatting code written by the developer.

## Type System

### Primitives and Modifiers

`bool`, `int`, `float`, `string`, `null`. Literal types: `"bug" | "enhancement" | "question"`. Optional: `string?`. Array: `string[]`. Map: `map<string, int>`. Union: `string | int` (order matters during parsing).

### Enums and Classes

```baml
enum EmailCategory {
  Spam
  Important
  @alias("newsletter_type")
  @description("Regular newsletter emails")
  Newsletter
}

class Resume {
  name string
  email string?
  skills string[]
  experience WorkHistory[]
}
```

No colon between field name and type. Field decorators: `@description("...")`, `@alias("json_key")`, `@check(name, {{ expression }})` (non-blocking), `@assert(name, {{ expression }})` (blocking). Class-level: `@@check`, `@@assert`, `@@alias`, `@@dynamic` (runtime schema modification).

### Multimodal Types

Built-in `image`, `audio`, `pdf`, `video` types. Auto-handles base64 conversion for providers that don't accept URLs.

### Recursive Types

```baml
class TreeNode {
  value string
  children TreeNode[]
}
```

Uses "hoisting" in prompt generation -- defines the type separately and references it, rather than infinite inline expansion.

### Cross-Language Type Mapping

| BAML | Python | TypeScript | Ruby | Go |
|------|--------|-----------|------|-----|
| `string` | `str` | `string` | `String` | `string` |
| `Type?` | `Optional[Type]` | `Type \| null` | `T.nilable` | `*Type` |
| `Type[]` | `List[Type]` | `Type[]` | `T::Array` | `[]Type` |
| `class` | `BaseModel` | `interface` | `T::Struct` | `struct` |

## Type-to-Prompt Compilation

`{{ ctx.output_format }}` compiles type definitions into token-efficient "jsonish" prompt instructions -- **~4x fewer tokens than JSON Schema** (~55% reduction).

A Resume type renders as:
```
Answer in JSON using this schema:
{
  name: string
  education: [{
    school: string
    degree: string
    year: int
  }]
  skills: string[]
}
```

Customizable: `prefix`, `always_hoist_enums`, `or_splitter`, `hoist_classes`, `hoisted_class_prefix`.

## Schema-Aligned Parsing (SAP)

SAP is BAML's core innovation. Instead of constraining generation (Outlines/Guidance) or retrying on failure (Instructor), SAP **post-processes** raw LLM output using the declared schema as a guide.

Inspired by edit-distance problems: "What is the least cost edit needed to transform model output into something parseable by the schema?"

**What SAP handles:** Markdown code fences, trailing commas, unquoted strings, missing colons, chain-of-thought text before structured output, type coercion (`"123"` → 123), key mismatches, broken JSON (comments, fractions, escaped character errors).

**What SAP does NOT do:** Hallucinate missing data (missing required fields = parse failure). Recover conceptual mismatches.

**Performance:** Error correction in **under 10ms** (Rust), orders of magnitude cheaper than a retry API call.

**Benchmarks** (Berkeley Function Calling Leaderboard, n=1,000):

| Model | Function Calling | SAP |
|-------|-----------------|-----|
| GPT-3.5-turbo | 87.5% | **92.0%** |
| GPT-4o | 87.4% | **93.0%** |
| Claude-3-Haiku | 57.3% | **91.7%** |
| GPT-4o-mini | 19.8% | **92.4%** |
| Claude-3.5-Sonnet | 78.1% | **94.4%** |

SAP maintains consistent performance even when function calling severely degrades.

## Code Generation

Rust compiler reads `.baml` files, generates `baml_client/` directory. Targets: `python/pydantic`, `typescript`, `typescript/react`, `ruby/sorbet`, `go`, `rust`, `rest/openapi`.

All generated clients share a **single Rust runtime** via FFI (PyO3 for Python, NAPI-RS for TypeScript, Magnus for Ruby, CGO for Go). Guarantees behavioral consistency across languages.

## Client Specification and Resilience

**Named clients:**
```baml
client<llm> MyClient {
  provider "openai"
  options { model "gpt-4o", temperature 0.7 }
}
```

**Retry policies:** Exponential backoff or constant delay with configurable `max_retries`, `delay_ms`, `multiplier`, `max_delay_ms`.

**Fallback chains:** Try clients sequentially. Nesting supported. Round-robin for load balancing. Retry policies apply after entire fallback chain exhausts.

## Streaming with Partial Types

BAML auto-generates nullable partial types for streaming:

```python
stream = b.stream.ExtractResume(text)
async for partial in stream:
    print(partial.name)   # None until streamed
    print(partial.skills) # grows as tokens arrive
```

Streaming attributes: `@stream.done` (atomic, only appears when complete), `@stream.not_null` (containing object withheld until field has value), `@stream.with_state` (adds StreamState wrapper with incomplete/complete metadata).

## Compiler Architecture

7-phase Salsa-based incremental compilation: Lexer → Parser (pest grammar) → HIR (symbol resolution) → TIR (type checking) → VIR (constraint validation) → MIR (control flow graph) → Emit (code generation). ~30ms code generation.

## Dynamic Types

For runtime-dependent schemas (database-loaded categories, tenant-specific fields):

```baml
enum Category { @@dynamic }
```

```python
tb = TypeBuilder()
tb.Category.add_value('Electronics')
tb.Category.add_value('Clothing')
result = await b.Classify(text, {"tb": tb})
```

## Observability

```python
collector = Collector(name="prod")
result = await b.ExtractResume(text, baml_options={"collector": collector})
log = collector.last
# log.usage.input_tokens, log.timing.duration_ms, log.raw_llm_response, log.calls[0].http_request
```

## Comparison to Alternatives

| Dimension | BAML | DSPy | Instructor |
|-----------|------|------|-----------|
| Philosophy | Schema engineering + robust parsing | Prompt optimization via compilation | Runtime type validation |
| Language | Own DSL | Python library | Python library |
| Multi-language | 7+ targets | Python only | Python only |
| Error handling | SAP (deterministic, <10ms) | Retry-based | LLM retry on validation |
| Strength | Reliable structured output from any model | Automatic prompt optimization | Simplest setup |
| Prompt control | Full (co-located in .baml) | Compiler-managed | In application code |

DSPy and BAML are **complementary**. A DSPy `BAMLAdapter` exists. "Starting with Instructor for prototyping, then adopting BAML for production multi-service architectures."

## Deliberate Non-Features

No pipeline/DAG construct. No agent orchestration. Multi-step logic lives in the host language. "An agent is a while loop that calls a Chat BAML Function with some state."

## Relevance to lx

**SAP is the killer parsing insight.** Post-processing is cheaper than constraining or retrying. lx's runtime should use SAP-style parsing when extracting structured data from LLM responses -- edit-distance-based recovery rather than strict parsing or re-prompting.

**Type-to-prompt compilation.** BAML's "jsonish" format uses ~4x fewer tokens than JSON Schema. lx's type system should compile to token-efficient prompt instructions. The `ctx.output_format` pattern -- automatically generating output format instructions from declared types -- should be a built-in feature of lx's agent invocation.

**BAML fills the gap lx leaves.** BAML handles individual LLM function calls with type safety and robust parsing. lx handles orchestration between agents. These are complementary layers. lx programs could use BAML-style typed function definitions for individual LLM calls while providing the workflow graph, agent coordination, and state management that BAML deliberately omits.

**Streaming with partial types.** BAML's auto-generated nullable partial types with `@stream.done` / `@stream.not_null` is sophisticated. lx should consider how streaming affects its type system -- when an agent is mid-response, what type does its output have?

**Cross-language code generation from a single source.** BAML generates Python, TypeScript, Ruby, Go from the same `.baml` file. lx could potentially generate client code for multiple host languages from `.lx` programs.

**Dynamic types are the escape hatch.** `@@dynamic` + `TypeBuilder` breaks compile-time guarantees but is necessary for runtime-dependent schemas. lx will face the same tension and needs a similar valve.

**The "no orchestration" boundary is lx's opportunity.** BAML says "orchestration stays in the host language." lx IS the orchestration language. The natural integration: lx for workflow, BAML-style typed functions for individual LLM calls within those workflows.