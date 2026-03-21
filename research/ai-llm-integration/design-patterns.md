# Design Patterns for LLM Integration

Structured output, prompt composition, refine loops, embeddings, and testing patterns across the ecosystem. Research conducted March 2026.

---

## 1. Structured Output Patterns

### 1.1 Three Approaches: JSON Mode vs Function Calling vs Constrained Decoding

**JSON Mode**
- Toggles the output format to valid JSON but does NOT enforce any schema
- Weakest guarantee: syntactically valid JSON, but fields/types are unconstrained
- Supported by all major providers (OpenAI, Anthropic, Gemini, Mistral)
- Use case: when you just need some JSON and will validate downstream

**Function Calling / Tool Use**
- Define tool schemas (name + description + JSON Schema parameters), LLM returns structured arguments
- Schema adherence is high but not guaranteed (the model can still hallucinate field values)
- The LLM does not execute the function — it generates the call, the application executes it
- Multi-turn pattern: LLM returns `tool_calls` → app executes → results feed back into conversation
- Provider support: OpenAI, Anthropic, Gemini (Mistral lacks it)

**Structured Outputs / Constrained Decoding**
- 100% schema compliance guaranteed by masking invalid tokens during generation
- Mathematical formulation: `P_constrained(x_t) = P(x_t) / sum(P(x) for x in V_valid)` if valid, else 0
- Approaches: FSM (Outlines), context-free grammars (XGrammar), pushdown automata (llguidance)
- Performance: O(1) valid token lookup with pre-compiled state machines, ~50us per token (llguidance)
- Trade-off: can degrade reasoning quality by 10-15% when format constraints force low-probability tokens
- Solution: two-step approach — let model reason freely, then constrain format in a second pass

**Recommendation for lx:** Support all three. `ai.prompt` returns free text. `ai.prompt_json` uses constrained decoding when available (local models) or function calling (API models). `ai.prompt_structured` maps to the strongest guarantee available from the backend.

Sources:
- [Function Calling vs Structured Outputs vs JSON Mode](https://www.vellum.ai/blog/when-should-i-use-function-calling-structured-outputs-or-json-mode)
- [Constrained Decoding Guide](https://mbrenndoerfer.com/writing/constrained-decoding-structured-llm-output)
- [Why All LLMs Need Structured Output Modes](https://fireworks.ai/blog/why-do-all-LLMs-need-structured-output-modes)

### 1.2 Schema-Driven Generation

The dominant pattern across all frameworks: define a schema, hand it to the LLM, validate the output.

**Pydantic Models (Python ecosystem standard)**
```python
class User(BaseModel):
    name: str
    age: int = Field(ge=0, le=150)
    email: Optional[str]
```
Used by: Instructor, Marvin, DSPy, Outlines, Guidance, LangChain. The model's `model_json_schema()` method produces the JSON Schema that gets injected into the prompt or function call definition.

**TypeScript/Zod**
```typescript
const UserSchema = z.object({
    name: z.string(),
    age: z.number().min(0).max(150),
    email: z.string().email().optional()
});
```
Used by: Vercel AI SDK, Instructor-TS, BAML (TypeScript codegen). Typia can generate function calling schemas directly from pure TypeScript types at compile time.

**BAML Classes**
```baml
class User {
    name string
    age int
    email string?
}
```
BAML's SAP (Schema-Aligned Parsing) algorithm handles: broken JSON, markdown code blocks, chain-of-thought before output, type coercion. Works on any model without native function calling.

**DSPy Signatures**
```python
class Extract(dspy.Signature):
    text: str = dspy.InputField()
    user: User = dspy.OutputField()  # Pydantic model as output type
```
DSPy compiles the signature + type into a prompt that instructs the LM how to format the output.

**Relevance to lx:** lx needs a native type system that serves the same role as Pydantic/Zod schemas. The lx type declaration should be sufficient to generate: (a) the prompt injection describing the output format, (b) the JSON Schema for function calling, (c) the FSM/grammar for constrained decoding, and (d) the runtime validator.

Sources:
- [Pydantic for LLMs](https://pydantic.dev/articles/llm-intro)
- [Typia LLM Schema](https://typia.io/docs/llm/schema/)
- [BAML vs Instructor](https://www.glukhov.org/post/2025/12/baml-vs-instruct-for-structured-output-llm-in-python/)

### 1.3 Validation and Retry

The universal pattern: attempt → validate → feed error back → retry.

**Instructor's retry loop:**
1. Call LLM with tool/function specs derived from Pydantic model
2. Parse response into Pydantic model
3. On `ValidationError`: append the LLM's response AND the validation error to message history
4. Re-invoke LLM with message: "fix the errors" + error details
5. Repeat until valid or `max_retries` exceeded

**DSPy Refine (replacing assertions):**
1. Execute module, get output
2. Run metric/validator on output
3. On failure: dynamically modify signature by adding `past_output` and `instruction` fields
4. Re-execute with enhanced context
5. Backtrack through module chain if needed

**BAML retry:**
- Framework-level retry with configurable `retry_policy` (max retries, backoff multiplier)
- Automatic reprompting when SAP parsing fails
- No manual error message construction needed

**Key insight from Instructor:** The validation error message IS the correction prompt. Pydantic's error messages are detailed enough that the LLM can fix its output just from reading them. This is why Pydantic (not custom validators) is the right foundation.

**Relevance to lx:** lx's `try/grade/revise` loop should follow Instructor's pattern: the grading output (not just pass/fail, but the specific error/feedback) becomes part of the revision prompt. The retry budget should be configurable per call site.

Sources:
- [How Instructor Works (internals)](https://ivanleo.com/blog/how-does-instructor-work)
- [Instructor Validation Basics](https://python.useinstructor.com/learning/validation/basics/)
- [DSPy Assertions/Refine](https://dspy.ai/learn/programming/7-assertions/)

### 1.4 Streaming Structured Output

**The problem:** When streaming JSON, you don't have valid JSON until the last closing bracket. Users see nothing until generation is complete.

**Solutions:**

**Partial JSON parsing:** Complete the incomplete JSON programmatically — add closing quotes, braces, fill nulls for missing values. Libraries: `partial-json` (Python), `PartialJSON` (Swift), `openai-partial-stream` (JS).

**Instructor's `create_partial`:**
```python
stream = client.create_partial(response_model=User, messages=[...])
for partial in stream:
    print(partial.name)  # available as soon as the name field is generated
    print(partial.age)   # None until generated
```
Each yield is a valid (partial) Pydantic model instance.

**BAML streaming:** Generated client code includes type-safe stream handlers. Streaming works with the same type safety as non-streaming calls.

**Constrained decoding + streaming:** When using FSM-based generation, each token is guaranteed valid, so partial output is always a valid prefix of the final JSON. No repair needed.

**Relevance to lx:** lx should support streaming structured output natively. Since lx has message passing, a natural pattern is: the AI call sends partial results as messages to the caller, which can process them incrementally.

Sources:
- [Structured Output Streaming](https://medium.com/@prestonblckbrn/structured-output-streaming-for-llms-a836fc0d35a2)
- [Streaming AI Responses and Incomplete JSON](https://www.aha.io/engineering/articles/streaming-ai-responses-incomplete-json)
- [LLM Structured Output in 2026](https://dev.to/pockit_tools/llm-structured-output-in-2026-stop-parsing-json-with-regex-and-do-it-right-34pk)

### 1.5 Type-Safe LLM Calls

The goal: give LLM calls type signatures that the type checker can verify at compile time, not just runtime.

**BAML:** Compile-time type checking across multiple languages. `.baml` definitions generate typed client code before runtime. The function signature is verified by the target language's type checker.

**Typia (TypeScript):** Generates function calling schemas directly from pure TypeScript types. The schema is guaranteed to match the type because both are derived from the same source.

**Zod + TypeScript:** Define a Zod schema, library translates it into an LLM-friendly interface, injects into prompt, validates response, retries with error feedback — all type-safe via Zod's inferred types.

**Instructor/Pydantic:** Runtime-only. Python type hints provide IDE support, but validation happens at execution time.

**The spectrum:**
1. Full compile-time (BAML, Typia) — errors caught before running
2. Schema-synced runtime (Zod, Pydantic) — types and validators from same source
3. Runtime-only (raw JSON parsing) — no guarantees

**Relevance to lx:** As a compiled language, lx can provide the strongest guarantee: the output type of `ai.prompt_structured<T>(...)` is checked at compile time, the JSON schema is generated from `T` at compile time, and the runtime validator is generated from `T`. This is lx's key advantage over Python-based solutions.

Sources:
- [Type-Safe LLM Outputs (pydantic-llm-io)](https://dev.to/yuu1ch13/type-safe-llm-outputs-why-i-built-pydantic-llm-io-1a6p)
- [Typia LLM Parameters](https://typia.io/docs/llm/parameters/)
- [TypeScript & LLMs in Production](https://johnchildseddy.medium.com/typescript-llms-lessons-learned-from-9-months-in-production-4910485e3272)

---

## 2. Prompt Engineering at the Language Level

### 2.1 Prompt Composition and Template Systems

**Jinja2 / Mustache / Handlebars:**
The dominant approach. Variables as placeholders swapped at runtime. Control flow (if/else, for loops) for dynamic content. Used by: BAML (Jinja in `#"..."#` blocks), LangChain (PromptTemplate), Guidance (handlebars-like).

**LMQL's embedded approach:**
Python code IS the template. Top-level strings become prompt content. Variables in `[BRACKETS]` trigger LLM generation. Control flow is native Python:
```python
for topic in topics:
    "Write about {topic}: [RESPONSE]" where STOPS_AT(RESPONSE, "\n")
```

**SGLang's primitive approach:**
Generation primitives (`gen`, `select`) interleave with Python. Fork/join for parallel prompt exploration. The template is the program.

**DSPy's signature approach:**
No explicit templates. The field names and descriptions ARE the template. The optimizer generates the actual prompt text. This is the most radical departure from traditional templating.

**Anthropic's context engineering principles (2025):**
- Start with minimal prompt, add instructions based on observed failures (not preemptively)
- Use XML tags or Markdown headers to organize sections
- System prompts should be "the minimal set of information that fully outlines expected behavior"
- Examples are "pictures worth a thousand words" — prefer examples over exhaustive rules

**Relevance to lx:** lx should support string interpolation (`"Hello {name}"`) and multi-line template literals. But the deeper insight from DSPy is: if the type signature is expressive enough, explicit templates become unnecessary for simple tasks. lx should offer both: explicit templates for complex prompts, type-driven auto-prompting for simple extraction.

Sources:
- [Template Syntax for LLM Prompts](https://latitude.so/blog/template-syntax-basics-for-llm-prompts)
- [Effective Context Engineering (Anthropic)](https://www.anthropic.com/engineering/effective-context-engineering-for-ai-agents)
- [Prompt Template Analysis](https://arxiv.org/html/2504.02052v2)

### 2.2 Few-Shot and Dynamic Examples

**Static few-shot:** Fixed examples in the prompt template. Simple but inflexible. Diminishing returns after 2-3 examples.

**Dynamic few-shot (retrieval-based):**
1. Maintain a knowledge base of (input, gold-standard output) pairs
2. When a new input arrives, embed it and retrieve the most similar examples
3. Inject retrieved examples into the prompt
4. This is a form of RAG where the "documents" are examples

Used by: DSPy (BootstrapFewShot automatically generates and selects demonstrations), LangChain (ExampleSelector), PromptLayer (dynamic template variables).

**DSPy's approach is the most sophisticated:** Optimizers automatically find which examples maximize a metric. MIPROv2 does data-aware, demonstration-aware example selection using Bayesian optimization. The developer never manually writes few-shot examples.

**Relevance to lx:** lx should support an `examples` parameter on AI calls that accepts a list of (input, output) pairs. For optimization, lx's refine loop could automatically collect successful (input, output) pairs and use them as examples for future calls.

Sources:
- [Few-Shot Prompting Guide](https://www.promptingguide.ai/techniques/fewshot)
- [Dynamic Few-Shot Prompting](https://brandencollingsworth.com/posts/dynamic/)
- [DSPy Optimizers](https://dspy.ai/learn/optimization/optimizers/)

### 2.3 System/User/Assistant Message Construction

All modern LLM APIs use a message array with roles:
```json
[
    {"role": "system", "content": "You are a helpful assistant."},
    {"role": "user", "content": "What is 2+2?"},
    {"role": "assistant", "content": "4"},
    {"role": "user", "content": "And 3+3?"}
]
```

**Framework patterns:**

- **BAML:** `_.role("user")` / `_.role("system")` directives within Jinja templates
- **Instructor:** Standard `messages=[...]` array, response_model added transparently
- **LangChain:** `SystemMessage`, `HumanMessage`, `AIMessage` classes
- **Guidance:** Implicit from template structure, `system()` / `user()` / `assistant()` context managers

**Key insight from function calling:** When tools are defined, the message array gains a `tool` role for tool results, creating a multi-turn cycle: user → assistant (tool_call) → tool (result) → assistant (final answer).

**Relevance to lx:** lx's `ai.prompt` should accept messages as a list of tagged values: `[system: "...", user: "...", assistant: "..."]`. The runtime constructs the message array from these tags. For simple cases, a plain string defaults to a single user message.

### 2.4 Token Budgeting and Context Window Management

Context windows range from 4K to 1M+ tokens but more context does not mean better results. Key findings:

**Context rot:** As tokens increase, the model's ability to recall information from context decreases. The transformer's n-squared pairwise token relationships stretch attention thin.

**Guiding rule (Anthropic):** Find "the smallest set of high-signal tokens that maximize the likelihood of the desired outcome."

**Strategies:**

| Strategy | Mechanism | Savings |
|----------|-----------|---------|
| **Truncation** | Keep beginning (summaries) or end (recent data) | Variable |
| **Compression** | Remove filler words, redundant phrases | 40-60% |
| **Summarization** | LLM compresses history into summary | 60-80% |
| **Hierarchical memory** | Working (verbatim) → Episodic (summaries) → Semantic (facts) | Scales indefinitely |
| **Just-in-time retrieval** | Load context on demand via tools, not pre-loaded | Dramatic |
| **Sub-agent architecture** | Specialized sub-agents with clean context windows | Per-task |
| **Prompt caching** | Anthropic: 90% cost/latency reduction for cached prefixes | 90% |

**Anthropic's compaction pattern:** When approaching context limits, pass conversation history to the model to summarize. Preserve architectural decisions, unresolved bugs, implementation details. Safest shortcut: just clear old tool call results.

**Relevance to lx:** lx programs can spawn sub-agents with clean context (via `spawn`). The runtime should track token usage per AI call and support automatic compaction. lx's `ai.prompt` could accept a `budget` parameter that triggers summarization if the context exceeds the budget.

Sources:
- [Effective Context Engineering (Anthropic)](https://www.anthropic.com/engineering/effective-context-engineering-for-ai-agents)
- [Managing Token Budgets](https://apxml.com/courses/getting-started-with-llm-toolkit/chapter-3-context-and-token-management/managing-token-budgets)
- [Context Length Management](https://agenta.ai/blog/top-6-techniques-to-manage-context-length-in-llms)
- [Context Window Management Strategies](https://www.getmaxim.ai/articles/context-window-management-strategies-for-long-context-ai-agents-and-chatbots/)

---

## 3. LLM-in-the-Loop Patterns

### 3.1 Refine/Iterate Loops

The core pattern: generate → evaluate → revise → repeat until convergence.

**Instructor's validation retry:**
```
loop:
    output = llm(prompt, schema)
    errors = validate(output)
    if no errors: return output
    prompt += [output, errors]  # append failed attempt + error to context
```

**DSPy Refine (replacing assertions):**
- Backtrack to failing module
- Add `past_output` and `instruction` fields to signature
- Re-execute with enhanced context
- Supports both hard (halt) and soft (log + continue) failure modes

**LLMLOOP (code generation):**
```
loop:
    code = llm(spec)
    test_results = run_tests(code)
    if all pass: return code
    feedback = format_failures(test_results)
    prompt += feedback
```
Iterates until the test suite passes.

**Confusion-Aware Rubric Optimization (CARO):**
- Decompose error signals via confusion matrix into distinct misclassification modes
- Diagnose and repair each mode individually
- Monotonic improvement with stable convergence around 0.80
- Mode-specific updates converge faster than consolidated feedback

**Convergence criteria used in practice:**
- Metric scores match over consecutive rounds (STRIVE)
- Confusion matrix analysis stabilizes (CARO)
- Test suite passes (LLMLOOP)
- Validation errors reach zero (Instructor)
- Max iterations exceeded (universal fallback)

**Relevance to lx:** lx's `try/grade/revise` loop should support:
1. A grading function that returns structured feedback (not just pass/fail)
2. The feedback automatically injected into the revision prompt
3. Configurable convergence criteria (metric threshold, max iterations, or custom predicate)
4. The ability to grade with an LLM (LLM-as-judge) or with deterministic code

Sources:
- [LLMLOOP Paper](https://valerio-terragni.github.io/assets/pdf/ravi-icsme-2025.pdf)
- [Confusion-Aware Rubric Optimization](https://arxiv.org/abs/2603.00451)
- [Iterative Rubric Refinement](https://www.emergentmind.com/topics/iterative-rubric-refinement)

### 3.2 Tool Use / Function Calling

**The universal pattern:**
1. Define tools as: name + description + parameter JSON Schema
2. Pass tool definitions to LLM alongside user message
3. LLM returns `tool_calls` with function name and arguments (JSON)
4. Application deserializes JSON, executes function, returns result
5. Result appended to conversation as `tool` role message
6. LLM generates final answer incorporating tool results

**Schema generation approaches:**

| Approach | How | Used By |
|----------|-----|---------|
| Manual JSON Schema | Hand-write tool definitions | Raw API calls |
| From Pydantic models | `model_json_schema()` | Instructor, LangChain |
| From Python functions | Inspect type hints + docstrings | LangChain, Semantic Kernel |
| From TypeScript types | Compile-time schema extraction | Typia, Vercel AI SDK |
| From BAML definitions | Code-generated schemas | BAML |
| From MCP servers | Dynamic discovery via `/tools` endpoint | MCP protocol |

**Model Context Protocol (MCP):**
Standardized client-server architecture where MCP Servers expose tools via a discovery endpoint. Agents query capabilities at runtime rather than hardcoding them. Gaining rapid adoption (proposed by Anthropic, adopted across ecosystem).

**Parallel tool calling:** OpenAI's `tool_calls` response can contain multiple calls to execute simultaneously. The application runs them in parallel and returns all results.

**Error handling best practices:**
- Try-catch around tool execution
- Input validation via denylist guards
- LLM-based screening for injection attempts
- Conditional dispatch (never `eval`)
- Graceful error messages returned as tool results

**Relevance to lx:** lx's tool exposure is already natural — any lx function can be exposed as a tool to an AI call by declaring it in the `tools` parameter. The function's type signature generates the schema automatically. lx should support MCP server mode where a running lx program exposes its functions as MCP tools.

Sources:
- [Function Calling with LLMs (Martin Fowler)](https://martinfowler.com/articles/function-call-LLM.html)
- [Function Calling Guide](https://mbrenndoerfer.com/writing/function-calling-llm-structured-tools)
- [MCP Specification](https://modelcontextprotocol.io/specification/2025-06-18/server/tools)
- [Schema Generation for LLM Function Calling](https://medium.com/@wangxj03/schema-generation-for-llm-function-calling-5ab29cecbd49)

### 3.3 RAG Patterns

**Standard RAG pipeline:**
1. **Index:** Chunk documents → embed chunks → store in vector database
2. **Retrieve:** Embed query → find similar chunks → return top-K
3. **Generate:** Inject retrieved chunks into prompt → LLM generates answer

**Chunking strategies:**

| Strategy | Description | Best For |
|----------|-------------|----------|
| Fixed-size | Split at N tokens/characters | Simple documents |
| Sentence-based | Split at sentence boundaries | Prose text |
| Heading-aware | Split at section headers | Structured documents |
| Semantic | Split when embedding similarity drops below threshold | Mixed content |
| Proposition-based | Extract atomic claims per sentence | High-precision retrieval |
| Adaptive | Variable windows with backtracking to avoid mid-sentence cuts | General purpose |

**Hybrid search:**
Combine keyword search (BM25) and vector search, fuse results with Reciprocal Rank Fusion (RRF). Standard practice as of 2025 — pure vector search misses exact matches, pure keyword search misses semantic similarity.

**Advanced patterns:**
- **Self-RAG:** Model decides when to retrieve and critiques its own outputs
- **HyDE (Hypothetical Document Embeddings):** Generate a hypothetical answer, embed it, retrieve real documents similar to the hypothesis
- **Reranking:** After retrieval, use a cross-encoder to re-score results for higher precision
- **Long RAG:** Process entire sections or documents instead of small chunks

**Relevance to lx:** lx's `ai.embed` is the foundation for RAG. lx should provide a built-in chunking utility and a vector similarity function. The RAG pipeline itself is naturally expressed as an lx program: chunk → embed → store → (at query time) embed query → search → prompt with context.

Sources:
- [RAG Enterprise Guide 2025](https://datanucleus.dev/rag-and-agentic-ai/what-is-rag-enterprise-guide-2025)
- [Common RAG Techniques (Microsoft)](https://www.microsoft.com/en-us/microsoft-cloud/blog/2025/02/04/common-retrieval-augmented-generation-rag-techniques-explained/)
- [Chunking Strategies for RAG](https://medium.com/@adnanmasood/chunking-strategies-for-retrieval-augmented-generation-rag-a-comprehensive-guide-5522c4ea2a90)

### 3.4 Multi-Model Orchestration

**Routing patterns:**

| Pattern | Mechanism | Savings |
|---------|-----------|---------|
| **Complexity routing** | Simple queries → cheap model, complex → expensive | ~30% cost reduction |
| **Cascade routing** | Try small model first, escalate if confidence low | Variable |
| **xRouter** | RL-trained router observes query + context, decides model | Up to 21.6% higher success |
| **Pick and Spin** | Unified deployment with hybrid routing (cost, latency, accuracy) | 33% GPU cost reduction |

**Fallback chains:**
```
try: model_a(prompt)
catch: model_b(prompt)     # different provider
catch: model_c(prompt)     # different architecture
catch: graceful_degradation()
```
With exponential backoff between retries. Frameworks like Bifrost auto-create fallback chains when multiple providers are configured.

**LLM Gateways (2025-2026):**
Routing and control layers between applications and model providers. Unified API, automatic failover, cost optimization, observability. Examples: Portkey AI, LiteLLM, Bifrost.

**Relevance to lx:** lx's `AiBackend` trait already supports pluggable providers. The natural extension is a `fallback` combinator: `ai.prompt("...", backend: fallback([anthropic, openai, local]))`. The runtime tries each backend in order. Cost-based routing could be a runtime policy.

Sources:
- [LLM Orchestration 2026](https://aimultiple.com/llm-orchestration)
- [xRouter Paper](https://arxiv.org/html/2510.08439v1)
- [Multi-Provider LLM Orchestration 2026](https://dev.to/ash_dubai/multi-provider-llm-orchestration-in-production-a-2026-guide-1g10)

### 3.5 Evaluation and Testing

**Types of LLM tests:**

| Type | Method | When |
|------|--------|------|
| **Deterministic** | Exact match, regex, contains | Simple format checks |
| **Statistical** | BLEU, ROUGE, cosine similarity | Comparison to reference |
| **LLM-as-judge** | Another LLM scores output against rubric | Quality assessment |
| **Golden dataset** | Run model on curated (input, expected_output) pairs | Regression testing |
| **A/B evaluation** | Compare two model/prompt versions on same inputs | Improvement validation |

**Golden datasets:**
- Curated (input, expected_output) pairs, typically human-labeled
- "Goldens" are pending test cases: they have inputs and expected outputs but no actual_output yet
- Build from real failures: every bad production output becomes a test case
- Continuously evolve — add new cases as new failure modes are discovered

**LLM-as-judge:**
- Use a stronger/different LLM to evaluate outputs
- Provide clear rubrics with evaluation criteria
- Include reference answers for better judgment
- Aggregate scores across test items for run-level metrics
- Realistic thresholds: 95% for critical functionality, 70% for experimental features

**CI/CD integration:**
```yaml
# GitHub Actions pattern
- name: Run LLM tests
  run: pytest tests/llm/ --tb=short
  env:
    OPENAI_API_KEY: ${{ secrets.OPENAI_API_KEY }}
```

**Promptfoo's LLM rubric:**
Define rubrics in YAML, run LLM-as-judge evaluations, track scores over time.

**Relevance to lx:** lx's test system should support non-deterministic assertions: `assert_similar(actual, expected, threshold: 0.9)` using embedding similarity, and `assert_quality(output, rubric: "...")` using LLM-as-judge. The `try/grade/revise` loop naturally doubles as a test harness when max_iterations=1.

Sources:
- [Testing LLM Applications (Langfuse)](https://langfuse.com/blog/2025-10-21-testing-llm-applications)
- [Golden Datasets (Confident AI)](https://www.confident-ai.com/docs/llm-evaluation/core-concepts/test-cases-goldens-datasets)
- [LLM-as-Judge Pipeline](https://machinelearningplus.com/gen-ai/llm-evaluation-pipeline/)
- [LLM Rubric (Promptfoo)](https://www.promptfoo.dev/docs/configuration/expected-outputs/model-graded/llm-rubric/)
- [Who Watches the Watchers? (Stack Overflow)](https://stackoverflow.blog/2025/10/09/who-watches-the-watchers-llm-on-llm-evaluations/)

### 3.6 Caching and Memoization

**Exact-match caching:**
- Hash the (prompt, parameters) tuple, return cached response on match
- Simplest implementation: in-memory dict or Redis
- Cache hit rate: depends on query diversity (typically 10-30%)

**Semantic caching:**
- Embed the query, find cached queries within a similarity threshold
- Captures redundancy that exact-match misses (31% of LLM queries exhibit semantic similarity)
- Demonstrated results: 73% cost reduction, 67% cache hit rate
- Similarity threshold: start at 0.8, tune per query type based on precision/recall
- Overhead: ~20ms for embedding + vector search (vs 850ms+ for LLM call)

**Provider prompt caching:**
- **Anthropic:** Cache writes at 25% premium, cache reads at 90% discount. 90% cost reduction, 85% latency reduction for long prompts.
- **OpenAI:** Automatic caching enabled by default, 50% cost savings for repeated prefixes.

**Multi-layer architecture:**
```
Request → Exact Cache (100% savings)
        → Semantic Cache (100% savings, ~20ms overhead)
        → Prefix Cache (50-90% savings, provider-level)
        → Full Inference
```

**Cache invalidation:**
- TTL-based: dynamic responses (30s-minutes), static knowledge (hours-days)
- Event-based: invalidate when source data changes
- Staleness detection: re-validate periodically

**Relevance to lx:** lx should support caching at the runtime level. A `cache` decorator or parameter on `ai.prompt` calls: `ai.prompt("...", cache: semantic(ttl: 3600))`. The runtime maintains the cache across the program's lifetime. For sub-agent architectures, the cache should be shared across agents.

Sources:
- [Semantic Caching Cost Reduction (VentureBeat)](https://venturebeat.com/orchestration/why-your-llm-bill-is-exploding-and-how-semantic-caching-can-cut-it-by-73/)
- [Prompt Caching Infrastructure](https://introl.com/blog/prompt-caching-infrastructure-llm-cost-latency-reduction-guide-2025)
- [AWS Caching Guide](https://aws.amazon.com/blogs/database/optimize-llm-response-costs-and-latency-with-effective-caching/)
- [GPT Semantic Cache Paper](https://arxiv.org/abs/2411.05276)

---

## 4. Embeddings and Vector Operations

### 4.1 Embedding APIs

**Major providers (March 2026):**

| Provider | Model | Dimensions | Strengths |
|----------|-------|-----------|-----------|
| **OpenAI** | text-embedding-3-small | 1536 (truncatable) | Best cost/quality balance, Matryoshka representation learning |
| **OpenAI** | text-embedding-3-large | 3072 (truncatable) | Highest quality from OpenAI |
| **Voyage AI** | voyage-3-large | 1024 | Highest benchmark scores overall |
| **Voyage AI** | voyage-multilingual-2 | 1024 | Best multilingual, outperforms others by 5.6% |
| **Cohere** | embed-v4.0 | 1024 | Maximizes distance between distinct pairs, designed for reranker pairing |

**Key technical details:**
- OpenAI embeddings are normalized to length 1 → cosine similarity = dot product (faster)
- Matryoshka representation learning: front-loads general information, enabling vector truncation without retraining
- Cohere and Voyage perform better when you specify whether input is a query or document (asymmetric embeddings)
- Trend for 2026: unified multimodal embeddings (text + image + audio + video in single space)

**Relevance to lx:** lx's `ai.embed` should support: provider selection, dimension control (for Matryoshka models), and query/document mode. The return type should be a first-class `Vector` type with built-in similarity operations.

Sources:
- [Embedding Models Comparison 2026](https://reintech.io/blog/embedding-models-comparison-2026-openai-cohere-voyage-bge)
- [Top Embedding Models 2026](https://artsmart.ai/blog/top-embedding-models-in-2025/)
- [OpenAI Embeddings Guide](https://platform.openai.com/docs/guides/embeddings)
- [Text Embedding Models Compared](https://document360.com/blog/text-embedding-model-analysis/)

### 4.2 Vector Similarity

**Distance functions:**

| Function | Formula | Properties |
|----------|---------|-----------|
| **Cosine similarity** | `dot(a,b) / (norm(a) * norm(b))` | Scale-invariant, range [-1, 1], industry standard |
| **Dot product** | `sum(a_i * b_i)` | Equivalent to cosine for normalized vectors, faster |
| **Euclidean distance** | `sqrt(sum((a_i - b_i)^2))` | Sensitive to magnitude, lower = more similar |

For normalized embeddings (OpenAI), cosine similarity = dot product. This is a common optimization: normalize once at index time, use dot product at query time.

**Relevance to lx:** lx should provide `vector.cosine(a, b)`, `vector.dot(a, b)`, `vector.euclidean(a, b)` as built-in operations. Since most embeddings are normalized, dot product should be the default similarity function.

### 4.3 Vector Storage

**Landscape as of March 2026:**

| Solution | Type | Scale | Best For |
|----------|------|-------|----------|
| **In-memory** (Vec<f32>) | Embedded | <100K vectors | Prototyping, small datasets |
| **SQLite-vec** | Embedded extension | <1M vectors | Edge/mobile, offline, zero-dependency |
| **pgvector / pgvectorscale** | PostgreSQL extension | 10M-100M vectors | Existing Postgres deployments, 471 QPS at 99% recall on 50M vectors |
| **Qdrant** | Purpose-built | 100M+ vectors | High-throughput vector-first workloads |
| **Pinecone** | Managed serverless | 100M+ vectors | Enterprise SLAs, elastic scaling |
| **Milvus** | Purpose-built | 1B+ vectors | Billion-scale operations |
| **Chroma** | Embedded/server | <10M vectors | Developer experience, prototyping |

**SQLite-vec** (most relevant for embedded/edge):
- Zero-dependency C extension, runs everywhere: laptops, mobile, WASM, Raspberry Pi
- SQL-native vector search: ANN queries, distance computation, vector manipulation
- Respects SQLite's transactional semantics (atomic inserts, updates, deletes)
- Language bindings: Python, Ruby, Node.js, Go, Rust

**2026 trend:** Market shifting back toward extended relational databases. pgvectorscale benchmarks at 471 QPS vs Qdrant's 41 QPS at 99% recall on 50M vectors. Cost break-even for managed vs self-hosted: ~80-100M queries/month.

**Relevance to lx:** lx should integrate with SQLite-vec for embedded use cases (the most natural fit for a language runtime) and support external vector stores via a trait/interface. The default should be in-memory for small datasets, SQLite-vec for persistence.

Sources:
- [SQLite-vec GitHub](https://github.com/asg017/sqlite-vec)
- [SQLite-vec Documentation](https://alexgarcia.xyz/sqlite-vec/)
- [Best Vector Databases 2026](https://encore.dev/articles/best-vector-databases)
- [pgvector Guide 2026](https://www.instaclustr.com/education/vector-database/pgvector-key-features-tutorial-and-pros-and-cons-2026-guide/)
- [Vector Databases Compared](https://letsdatascience.com/blog/vector-databases-compared-pinecone-qdrant-weaviate-milvus-and-more)

### 4.4 Semantic Search Patterns

**Basic semantic search:**
1. Embed query using same model as indexing
2. Find K nearest neighbors in vector store
3. Return results ranked by similarity score

**Chunking for search:**
- Short enough to be precise, long enough to preserve context
- Overlap between chunks (typically 10-20%) to avoid splitting relevant information
- Metadata per chunk: source document, section, position, timestamp

**Hybrid search:**
```
keyword_results = bm25_search(query)
vector_results = vector_search(embed(query))
final_results = reciprocal_rank_fusion(keyword_results, vector_results)
```

**Reranking:**
After initial retrieval (fast, approximate), re-score with a cross-encoder (slow, precise). Cohere's reranker is designed to pair with their embeddings. Cross-encoders consider the full query-document interaction, not just independent embeddings.

**HyDE (Hypothetical Document Embeddings):**
```
hypothetical_answer = llm("Answer this question: " + query)
similar_docs = vector_search(embed(hypothetical_answer))
final_answer = llm(query + context=similar_docs)
```
Particularly useful for sparse or underspecified queries where the query embedding alone is poor.

**Relevance to lx:** lx should provide `ai.search(query, collection, k: 10)` as a high-level primitive that handles embedding, vector search, and reranking. The `collection` is a vector store created by `ai.index(documents)`. Hybrid search should be the default when a text index is available alongside the vector index.

Sources:
- [RAG Enterprise Guide 2025](https://datanucleus.dev/rag-and-agentic-ai/what-is-rag-enterprise-guide-2025)
- [Common RAG Techniques (Microsoft)](https://www.microsoft.com/en-us/microsoft-cloud/blog/2025/02/04/common-retrieval-augmented-generation-rag-techniques-explained/)

---

## 5. Summary: Design Decisions for lx

### What lx already has right
1. **First-class AI calls** (`ai.prompt`, `ai.prompt_structured`, `ai.prompt_json`, `ai.embed`) — no other language has this
2. **Refine loop** (`try/grade/revise`) — maps to the universal validate-and-retry pattern
3. **Pluggable backends** (`AiBackend` trait) — supports the multi-model orchestration pattern
4. **Message passing** (`spawn`) — natural substrate for agent orchestration without framework overhead

### What lx should adopt from this research

| Pattern | Source | lx Implementation |
|---------|--------|-------------------|
| Type-driven schema generation | BAML, Typia | Output type of `ai.prompt_structured<T>` generates JSON Schema, FSM, and validator from `T` at compile time |
| Validation error as correction prompt | Instructor | `try/grade/revise` injects grading feedback into revision prompt automatically |
| Constrained decoding support | Guidance, Outlines | `AiBackend` trait method for grammar-constrained generation |
| Dynamic few-shot | DSPy | `examples` parameter on AI calls, auto-collected from successful past executions |
| Semantic caching | Industry | `cache` parameter on AI calls: `ai.prompt("...", cache: semantic(ttl: 3600))` |
| Fallback chains | Industry | `ai.prompt("...", backend: fallback([a, b, c]))` |
| Streaming structured output | Instructor, BAML | AI calls send partial results as messages to the caller |
| Vector operations | Industry | Built-in `Vector` type with similarity functions, `ai.search` high-level primitive |
| Token budgeting | Anthropic | `budget` parameter that triggers compaction/summarization |
| LLM-as-judge testing | Industry | `assert_quality(output, rubric: "...")` in test framework |
