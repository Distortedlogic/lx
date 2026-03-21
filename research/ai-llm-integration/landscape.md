# AI/LLM Integration Across Languages and Frameworks

Survey of how programming languages and frameworks integrate LLM capabilities at the language level. Research conducted March 2026.

## 1. DSPy (Stanford NLP)

**Repository:** https://github.com/stanfordnlp/dspy
**Philosophy:** Programming LLMs instead of prompting them. Shift from crafting prompt strings to composing declarative modules with typed signatures.

### Signatures

Signatures are declarative specs of input/output behavior. Two forms:

**Inline signatures** use shorthand string notation:
```python
"question -> answer"                                    # defaults to str
"context: list[str], question: str -> answer: str"      # typed fields
"question, choices: list[str] -> reasoning: str, selection: int"
```

**Class-based signatures** inherit from `dspy.Signature`:
```python
class Emotion(dspy.Signature):
    """Classify emotion."""
    sentence: str = dspy.InputField()
    sentiment: Literal['sadness', 'joy'] = dspy.OutputField()
```

Supported types: `str`, `int`, `bool`, `list[str]`, `dict[str, int]`, `Optional[float]`, Pydantic models, `dspy.Image`, `dspy.History`. DSPy auto-validates input field types at runtime.

### Modules

Each module applies a prompting strategy to a signature:

- `dspy.Predict` — direct signature execution
- `dspy.ChainOfThought` — teaches LM to reason step-by-step before answering
- `dspy.ReAct` — agent that can use tools to implement a signature
- `dspy.Refine` — replaces deprecated `dspy.Assert`/`dspy.Suggest`; validates output and retries with feedback

Modules compose into programs: a program is a Python class with `__init__` (declare sub-modules) and `forward` (execute logic).

### Optimizers (Compilation Pipeline)

Optimizers tune prompts and/or LM weights to maximize a metric. They need: a DSPy program, a metric function, and a training dataset (as few as 5-10 examples).

| Optimizer | Tunes | How | Data Needed |
|-----------|-------|-----|-------------|
| **BootstrapFewShot** | Few-shot demos | Teacher generates demos, metric validates | ~10 examples |
| **BootstrapFewShotWithRandomSearch** | Few-shot demos | Multiple random BootstrapFewShot runs, best wins | 50+ |
| **MIPROv2** | Instructions + demos | Bootstrap traces → grounded instruction proposals → Bayesian search | 50-200+ |
| **SIMBA** | Instructions + demos | Stochastic mini-batch sampling, failure introspection, self-reflective rules | varies |
| **GEPA** | Instructions | LM reflects on trajectories, identifies gaps, proposes improved prompts | varies |
| **COPRO** | Instructions only | Coordinate ascent / hill-climbing over instruction space | varies |
| **BootstrapFinetune** | LM weights | Distills prompt-based program into weight updates (fine-tuning) | varies |

Typical cost: ~$2 USD, ~10 minutes. Results: classification 66%→87%, RAG 53%→61%, ReAct agent 24%→51%.

Optimizers compose: run MIPROv2, then BootstrapFinetune on the output, or ensemble top-5 candidates with `dspy.Ensemble`.

### Assertion-Based Validation (deprecated → dspy.Refine)

Legacy assertions (`dspy.Assert`, `dspy.Suggest`) provided hard/soft constraints on LM output, with backtracking and dynamic signature modification on failure. Now replaced by `dspy.Refine` module.

**Relevance to lx:** DSPy's signature system maps closely to lx's `ai.prompt_structured` — typed input/output declarations that compile into prompts. The optimizer pipeline is the most sophisticated prompt optimization system available. lx's `try/grade/revise` refine loop is analogous to DSPy's assertion backtracking.

Sources:
- [DSPy Signatures](https://dspy.ai/learn/programming/signatures/)
- [DSPy Optimizers](https://dspy.ai/learn/optimization/optimizers/)
- [DSPy Assertions](https://dspy.ai/learn/programming/7-assertions/)
- [DSPy GitHub](https://github.com/stanfordnlp/dspy)
- [IBM - What is DSPy?](https://www.ibm.com/think/topics/dspy)

---

## 2. BAML (BoundaryML)

**Repository:** https://github.com/BoundaryML/baml
**Philosophy:** A purpose-built DSL where every prompt is a type-safe function with declared inputs, outputs, and LLM client configuration.

### Language Syntax

`.baml` files declare types, functions, and clients:

```baml
class Resume {
  name string
  email string?          // optional
  skills string[]        // array
  experience WorkHistory[]
}

enum EmailCategory {
  SPAM
  IMPORTANT
  NEWSLETTER
}

function ClassifyMessage(input: string) -> Category {
  client "openai/gpt-4o"
  prompt #"
    Classify: {{ input }}
    {{ ctx.output_format }}
  "#
}
```

**Type system:** `string`, `int`, `float`, `bool`, `null`, optional (`Type?`), arrays (`Type[]`), unions (`TypeA | TypeB`), literals (`"value1" | "value2"`), media types (`image`, `audio`, `pdf`, `video`).

**Client configuration:**
```baml
client<llm> GPT4o {
  provider "openai"
  options { model "gpt-4o"  api_key env.OPENAI_API_KEY  temperature 0.7 }
}

retry_policy ExponentialBackoff {
  max_retries 3
  backoff_multiplier 2
}
```

**Prompt templating:** Jinja syntax inside raw strings (`#"..."#`). Special variables: `ctx.output_format` (auto-generated schema instructions), `_.role()` (message roles).

### Schema-Aligned Parsing (SAP)

BAML's SAP algorithm handles common LLM output failures without requiring native function-calling APIs:
- Fixes broken JSON (missing brackets, trailing commas)
- Extracts JSON from markdown code blocks
- Parses chain-of-thought reasoning before the actual output
- Coerces types intelligently (`"123"` → `123`)

This means BAML functions work on any model, not just those with structured output support.

### Code Generation

```baml
generator target {
  output_type "python/pydantic"
  output_dir "../baml_client"
}
```

Generates type-safe clients for: Python/Pydantic, TypeScript, Ruby/Sorbet, Go, REST/OpenAPI. Same `.baml` definitions produce consistent clients across all languages.

### Testing and Playground

BAML includes a built-in testing framework within `.baml` files and a visual playground for prompt iteration with immediate feedback. Tests validate prompt behavior before deployment.

**Relevance to lx:** BAML's approach of treating prompts as typed functions is the closest analog to lx's design. The SAP algorithm solves the same problem lx faces: getting structured output from any model reliably. BAML's cross-language code generation from a single DSL definition is a pattern lx could adopt for its AiBackend trait implementations.

Sources:
- [BAML GitHub](https://github.com/BoundaryML/baml)
- [BAML Documentation](https://docs.boundaryml.com/home)
- [BAML Language Reference (DeepWiki)](https://deepwiki.com/BoundaryML/baml/5-baml-language-reference)
- [BAML vs Instructor](https://www.glukhov.org/post/2025/12/baml-vs-instruct-for-structured-output-llm-in-python/)

---

## 3. Marvin (Prefect)

**Repository:** https://github.com/PrefectHQ/marvin
**Philosophy:** LLMs as a Python runtime. Type hints and docstrings ARE the prompt. Functions look and feel like regular Python but use LLMs as their execution engine.

### Core Primitives

**`@marvin.fn`** — AI-powered functions. The function signature, type hints, and docstring define the prompt:
```python
@marvin.fn
def sentiment(text: str) -> float:
    """Returns a sentiment score between -1 (negative) and 1 (positive)."""

sentiment("I love this!")  # 0.8
```

**`@marvin.model`** — Structured extraction via class instantiation:
```python
@marvin.model
class Location(BaseModel):
    city: str
    state: str

Location("The Big Apple")  # Location(city="New York", state="New York")
```

**`marvin.cast()`** — Transform text into structured outputs with custom instructions.

**`marvin.classify()`** — Classify text into predefined categories.

**`marvin.extract()`** — Extract key elements from text into typed structures.

### Marvin 3.0

Uses Pydantic AI for LLM interactions, supporting all providers Pydantic AI supports. Combines the DX of Marvin 2.0 with the agentic engine of ControlFlow.

**Relevance to lx:** Marvin's insight that type signatures can serve as the entire prompt specification is powerful. lx's `ai.prompt_structured` should explore whether the output type definition alone (without explicit prompt text) can drive generation for simple extraction tasks.

Sources:
- [Marvin GitHub](https://github.com/PrefectHQ/marvin)
- [Marvin API Reference](https://www.askmarvin.ai/api_reference/ai/text/)
- [Structured Data Extraction with Marvin](https://learnbybuilding.ai/tutorial/structured-data-extraction-with-marvin-ai-and-llms/)

---

## 4. Instructor

**Repository:** https://github.com/567-labs/instructor
**Philosophy:** Patch any LLM client with Pydantic validation and automatic retry. Minimal learning curve, maximum portability.

### How It Works

1. **Client patching:** `instructor.from_provider("openai/gpt-4o")` wraps the client's `create` method.
2. **Schema conversion:** `openai_schema()` converts Pydantic models into OpenAI-compatible function call schemas via `model_json_schema()`, parsing docstrings for parameter descriptions.
3. **Response model:** Pass `response_model=YourPydanticModel` to `create()` calls.
4. **Validation + retry loop:**
   - Call LLM with tool/function specs
   - Parse response against Pydantic model
   - On validation failure: append LLM response + error details to message history
   - Re-invoke LLM with corrected context ("fix the errors")
   - Repeat until validation passes or `max_retries` exceeded

### Mode Parameter

The `Mode` enum controls how structured output is requested:
- `TOOLS` (default) — OpenAI tool calling
- `TOOLS_STRICT` — OpenAI Structured Outputs mode
- `JSON` / `MD_JSON` — manual JSON prompting with parsing
- `FUNCTIONS` — deprecated function calling
- Provider-specific modes for Anthropic, Mistral, Cohere, Vertex AI, Gemini

### Streaming

```python
stream = client.create_partial(response_model=Model, messages=[...])
for partial in stream:
    print(partial)  # incrementally built Pydantic model
```

### Scale

3M+ monthly downloads, 11k+ stars, 100+ contributors. SDKs for Python, TypeScript, Go, Ruby.

**Relevance to lx:** Instructor's retry-with-validation-feedback loop is the simplest effective pattern for structured output. lx's `ai.prompt_structured` should implement the same pattern: attempt → validate → append error to context → retry. The mode abstraction (TOOLS vs JSON vs constrained decoding) is worth adopting.

Sources:
- [Instructor Documentation](https://python.useinstructor.com/)
- [How Does Instructor Work?](https://ivanleo.com/blog/how-does-instructor-work)
- [Instructor Getting Started](https://python.useinstructor.com/getting-started/)
- [LLM Validation Basics](https://python.useinstructor.com/learning/validation/basics/)

---

## 5. Guidance (Microsoft)

**Repository:** https://github.com/guidance-ai/guidance
**Philosophy:** Control LLM output token-by-token using grammars. Treat output requirements as context-free grammars enforced during inference.

### Core Primitives

- **`gen()`** — Generate text with optional constraints: `gen(regex=r"\d+")`, `gen(max_tokens=50)`
- **`select()`** — Restrict output to predefined options: `select(["A", "B", "C"])`
- **`@guidance` decorator** — Compose primitives into stateful functions that maintain state across generation

### Grammar-Based Constraints

Guidance enforces constraints by steering the model token-by-token at the inference layer. It supports:
- Regular expressions
- Context-free grammars
- JSON schemas (via Pydantic)
- Custom composable grammars built from simpler rules

### Token Fast-Forwarding

When grammar rules make certain tokens predictable (e.g., closing tags in HTML), Guidance skips model forward passes and injects tokens directly. This speeds up generation for structured formats.

### llguidance (Performance Layer)

The `llguidance` library is the high-performance backend. Performance: ~50us CPU time per token for a 128k tokenizer. OpenAI adopted llguidance in May 2025 for their structured output feature.

**Relevance to lx:** Guidance's grammar-based approach is the gold standard for local/self-hosted models. lx's AiBackend trait should support a constrained-decoding mode where the backend can accept grammar constraints alongside the prompt.

Sources:
- [Guidance GitHub](https://github.com/guidance-ai/guidance)
- [llguidance GitHub](https://github.com/guidance-ai/llguidance)
- [Microsoft Research - Guidance](https://www.microsoft.com/en-us/research/project/guidance-control-lm-output/)
- [LLGuidance Performance](https://guidance-ai.github.io/llguidance/llg-go-brrr)

---

## 6. LMQL (ETH Zurich)

**Repository:** https://github.com/eth-sri/lmql
**Paper:** [Prompting Is Programming: A Query Language for Large Language Models](https://arxiv.org/abs/2212.06094)
**Philosophy:** A query language for LLMs that blends SQL-like declarative constraints with imperative Python scripting.

### Syntax

```python
sample(temperature=1.0)       # decoder selection

"Q: {question}\n"             # prompt template with Python interpolation
"A: [ANSWER]"                 # [VAR] placeholder for LLM generation
where len(TOKENS(ANSWER)) < 100 and STOPS_AT(ANSWER, "\n")
```

### Constraint Types

| Constraint | Syntax | Effect |
|-----------|--------|--------|
| Stopping | `STOPS_AT(VAR, phrase)` | Halt generation at phrase |
| Stop before | `STOPS_BEFORE(VAR, phrase)` | Halt before phrase |
| Integer | `INT(VAR)` | Restrict to integer output |
| Set choice | `VAR in set(["A", "B"])` | Limit to predefined values |
| Length (chars) | `len(VAR) < N` | Character length constraint |
| Length (tokens) | `len(TOKENS(VAR)) < N` | Token length constraint |
| Regex | `REGEX(VAR, pattern)` | Regex pattern enforcement |

Constraints combine with `and`/`or` in `where` clauses. They operate at text level (not token level) — developers specify high-level requirements without managing tokenization.

### Scripted Prompting

Python control flow inside prompts:
```python
for i in range(5):
    "-[THING]" where STOPS_AT(THING, "\n")
    backpack.append(THING.strip())
"Most essential: [ESSENTIAL]" where ESSENTIAL in backpack
```

### Decoding Control

Supports `argmax`, `sample(temperature=...)`, `beam(n=...)`, `best_k(n=...)`.

### Performance

Speculative execution + constraint short-circuiting + tree-based caching = 75-85% fewer billable tokens compared to standard decoding.

**Relevance to lx:** LMQL's constraint system is directly relevant to lx. The `where` clause pattern maps naturally to lx's type system — output constraints could be expressed as type annotations. LMQL's text-level constraint abstraction (hiding tokenization) is the right design for a high-level language.

Sources:
- [LMQL Homepage](https://lmql.ai/)
- [LMQL Constraints](https://lmql.ai/docs/language/constraints.html)
- [LMQL Scripted Prompting](https://lmql.ai/docs/language/scripted-prompting.html)
- [LMQL Paper](https://arxiv.org/abs/2212.06094)

---

## 7. SGLang (LMSYS)

**Repository:** https://github.com/sgl-project/sglang
**Paper:** [SGLang: Efficient Execution of Structured Language Model Programs](https://arxiv.org/abs/2312.07104)
**Philosophy:** A frontend language + high-performance runtime for LLM programming. Optimize the full execution of multi-call LLM programs, not just individual calls.

### Frontend Primitives

- **`gen`** — Non-blocking LLM generation, stores result in variable
- **`select` / `choices`** — Constrained generation from predefined options
- **`fork`** — Create parallel copies of a prompt for concurrent generation
- **`join`** — Synchronize parallel branches
- **`[variable_name]`** — Retrieve generation results

Two execution modes: interpreter (eager) and compiler (dataflow graph optimization).

### RadixAttention (KV Cache Reuse)

Instead of discarding KV caches after generation, SGLang retains them in a radix tree:
- Maps token sequences → KV cache tensors on GPU (paged, one token per page)
- LRU eviction policy with recursive leaf node eviction
- Tree structure on CPU, cache tensors on GPU
- Enables automatic prefix sharing across: few-shot examples, chat history, self-consistency queries, tree-of-thought search

### Compressed Finite State Machines

For constrained decoding, SGLang compresses FSMs to skip unnecessary computation. Jump-forward decoding allows the model to skip predictable tokens entirely.

### Performance

Up to 5x higher throughput vs vLLM, Guidance, and HuggingFace TGI. Up to 6x on multimodal benchmarks. Deployed at scale: trillions of tokens/day in production (xAI, AMD, NVIDIA, Intel).

**Relevance to lx:** SGLang's fork/join parallelism maps directly to lx's `spawn`/message-passing model. The RadixAttention technique is relevant for lx programs that make many LLM calls with shared context (e.g., a refine loop where the system prompt is constant). lx's runtime should consider KV cache reuse when orchestrating multiple AI calls.

Sources:
- [SGLang Blog Post](https://lmsys.org/blog/2024-01-17-sglang/)
- [SGLang Paper](https://arxiv.org/abs/2312.07104)
- [SGLang GitHub](https://github.com/sgl-project/sglang)

---

## 8. Outlines (dottxt)

**Repository:** https://github.com/dottxt-ai/outlines
**Philosophy:** Structured generation guaranteed at decode time via finite state machines. Never parse and retry — prevent invalid output from being generated in the first place.

### How FSM-Based Generation Works

1. Convert output specification (JSON schema, regex, type constraint) into a finite state machine
2. Pre-compute vocabulary index: for each FSM state, which tokens are valid transitions
3. At each generation step, mask logits of invalid tokens
4. Renormalize probability distribution over valid tokens only
5. Sample from constrained distribution

### API

```python
model = outlines.models.transformers("mistralai/Mistral-7B")
generator = outlines.generate.json(model, YourPydanticModel)
result = generator(prompt)  # guaranteed valid

# Other modes:
outlines.generate.choice(model, ["Yes", "No"])
outlines.generate.regex(model, r"\d{3}-\d{3}-\d{4}")
outlines.generate.text(model)  # unconstrained
```

Supports: Literal types, primitive types (`int`, `str`, `float`), Pydantic models, Union types.

### Performance Characteristics

Pre-compiled FSMs enable O(1) valid token lookup per step. However, complex schemas (minItems, maxItems, large enums) can cause compilation times from 40 seconds to 10+ minutes as regex representations explode.

### Ecosystem

XGrammar (CMU/MLC, MLSys 2025) extends the approach using context-free grammars with pushdown automata. As of 2025, vLLM uses XGrammar by default. OpenAI credited llguidance for their structured output implementation (May 2025).

**Relevance to lx:** Outlines' approach is ideal for lx's `ai.prompt_json` — when the target schema is known at compile time, lx could pre-compile the FSM and pass it to local model backends. For API-based backends, the JSON schema alone suffices.

Sources:
- [Outlines GitHub](https://github.com/dottxt-ai/outlines)
- [Compressed FSM (LMSYS)](https://lmsys.org/blog/2024-02-05-compressed-fsm/)
- [Constrained Decoding Guide](https://mbrenndoerfer.com/writing/constrained-decoding-structured-llm-output)

---

## 9. Semantic Kernel and LangChain

### Semantic Kernel (Microsoft)

**Repository:** https://github.com/microsoft/semantic-kernel
**Languages:** C#, Python, Java

Core abstraction: **Plugins** containing semantic functions (LLM-powered) and native functions (code). The **Planner** decomposes complex requests into sequences of function calls, selecting appropriate plugins and orchestrating execution.

Key concepts:
- Plugins are collections of functions with descriptions
- The kernel routes between semantic and native functions
- Planners auto-decompose tasks into multi-step plans
- 27k+ GitHub stars as of March 2026

### LangChain

**Repository:** https://github.com/langchain-ai/langchain

Core abstraction: **Chains** compose LLM calls, tools, and retrieval into pipelines. **Agents** use LLMs to decide which tools to call and in what order.

Key concepts:
- Tool abstraction: any function with a name, description, and schema
- Chains: sequential composition of prompts and tools
- Agents: LLM-driven tool selection and execution
- LangGraph: state machine abstraction for complex agent workflows

### Common Patterns

Both frameworks solve the same problem: bridging LLMs and code. The tool/function abstraction follows OpenAI's format: name + description + JSON Schema parameters. The LLM sees tool schemas, decides which to call, returns structured arguments, the framework executes and feeds results back.

**Relevance to lx:** lx is a language, not a framework — it should provide the primitives that make frameworks like these unnecessary. lx's `spawn` + message passing replaces LangChain's chains. lx's type system replaces Pydantic schema generation. The key insight: frameworks add abstraction layers because the host language lacks native LLM primitives. lx has them built in.

Sources:
- [Semantic Kernel GitHub](https://github.com/microsoft/semantic-kernel)
- [Semantic Kernel 2026 Overview](https://is4.ai/blog/our-blog-1/semantic-kernel-microsoft-ai-tool-27338-stars-2026-280)
- [LangChain vs Semantic Kernel](https://www.leanware.co/insights/langchain-vs-semantic-kernel-which-ai-framework-is-right-for-your-next-project)
- [AI Orchestration Frameworks Comparison](https://servicesground.com/blog/ai-orchestration-frameworks-comparison/)

---

## Competitive Landscape Summary

| Tool | Approach | Strengths | Weaknesses |
|------|----------|-----------|------------|
| **DSPy** | Compile prompts from signatures | Automatic optimization, modular | Steep learning curve, Python-only |
| **BAML** | Purpose-built DSL | Type safety, multi-language codegen, SAP | New ecosystem, build step required |
| **Marvin** | Type hints as prompts | Minimal boilerplate, Pythonic | Limited control over prompt content |
| **Instructor** | Patch + validate + retry | Universal, simple, battle-tested | Runtime-only validation |
| **Guidance** | Grammar-constrained decoding | Token-level control, fast | Requires model access (not API-only) |
| **LMQL** | Query language with constraints | Expressive constraints, efficient | Academic, smaller community |
| **SGLang** | Frontend language + optimized runtime | Extreme performance, parallelism | Focused on serving, not application logic |
| **Outlines** | FSM-based structured generation | Guaranteed valid output | Compilation overhead for complex schemas |
| **Semantic Kernel** | Plugin + planner framework | Enterprise-ready, multi-language | Framework overhead |
| **LangChain** | Chain + agent framework | Huge ecosystem, rapid prototyping | Abstraction bloat, performance |

**lx's position:** lx is the only entry that is a language (not a library, framework, or DSL bolted onto Python). It has first-class AI calls, a refine loop, and pluggable backends — combining the best ideas from this landscape without the host-language tax.
