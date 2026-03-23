# Token-Efficient Tool Calling: Definitive Reference

## 1. Token-Efficient Tool Results (Anthropic)

The beta header `token-efficient-tools-2025-02-19` enables compressed tool output representation, reducing output token consumption by up to 70% (average 14% across early users). Originally launched for Claude 3.7 Sonnet in March 2025, it required the beta header. All Claude 4+ models have token-efficient tool use built in -- the header is unnecessary and ignored on Claude 4.x. Available on Anthropic API, Amazon Bedrock, and Google Cloud Vertex AI.

The mechanism works by having Claude produce more compact representations when generating tool call arguments and structured responses, without changing the tool definition format or response schema.

## 2. Programmatic Tool Calling (PTC)

PTC lets Claude write Python code that orchestrates multiple tools within a sandboxed code execution container, instead of requiring separate LLM inference round-trips per tool call.

**How it works:** Claude generates a Python script that calls tools programmatically. The script runs in the code execution sandbox, pausing when it needs results from your tools. When you return tool results via the API, the script processes them locally. Only final outputs (stdout) enter the model's context window.

**Key numbers:**
- Token consumption: 43,588 to 27,297 tokens (37% reduction on research tasks)
- Budget compliance example: 20 tool calls returning 2,000+ expense line items (50KB+) reduced to 2-3 final results (1KB)
- Eliminates 19+ inference passes in 20-tool workflows
- Knowledge retrieval accuracy: 25.6% to 28.5%
- GIA benchmarks: 46.5% to 51.2%
- BrowseComp/DeepSearchQA: PTC was the key factor that fully unlocked agent performance on these agentic search benchmarks

**API configuration:**
- Beta header: `advanced-tool-use-2025-11-20`
- Mark tools with `allowed_callers: ["code_execution_20250825"]`
- Tool version `code_execution_20260120` is the latest (as of early 2026)
- Supported on Claude Opus 4.6, Opus 4.5, Opus 4.1, Sonnet 4.6, Sonnet 4.5, Sonnet 4, Sonnet 3.7, Haiku 4.5

**When to use PTC:**
- Processing large datasets requiring aggregation/filtering
- Multi-step workflows with 3+ dependent operations
- Intermediate data that should not influence reasoning
- Parallel operations across many items (uses `asyncio.gather()`)
- Idempotent, retry-safe operations

## 3. Code Execution via MCP (98.7% Reduction)

Instead of treating tools as direct function calls, MCP servers are presented as code APIs. The agent writes code to interact with MCP servers, loading only needed tools and processing data in the execution environment.

**Architecture:** Tools from MCP servers are exposed as a filesystem hierarchy (e.g., `/servers/google-drive/getDocument.ts`). Agents discover tools by exploring directory structure, then read specific definitions as needed. A `search_tools` function filters by detail level (name only, name+description, or full schema).

**Token reduction:** 150,000 tokens down to 2,000 tokens -- 98.7% reduction for a Google Drive to Salesforce workflow.

**Why it saves tokens:**
- Tool definitions not loaded upfront into context
- Intermediate results processed locally in execution environment
- Only filtered/transformed data returned to model
- Loops, conditionals, error handling run natively in code

**Security benefits:** Intermediate results stay in the execution environment. Sensitive data flows tool-to-tool without entering model context. PII can be tokenized before reaching the model. Deterministic security rules enforced at execution layer.

**Trade-off:** Requires secure sandboxing, resource limits, and monitoring infrastructure that direct tool calls avoid.

## 4. Tool Search / Deferred Loading (85% Reduction)

Tools marked with `defer_loading: true` are excluded from initial context. Claude sees only the Tool Search Tool (~500 tokens) plus any critical always-loaded tools. When Claude needs specific capabilities, it searches for and loads relevant tools on demand.

**Real-world overhead without Tool Search:**
- GitHub MCP: ~26K tokens (35 tools)
- Slack MCP: ~21K tokens (11 tools)
- Jira MCP: ~17K tokens
- Total for 5 servers: ~55K tokens before conversation starts
- Internal deployments observed: 134K tokens of tool definitions

**With Tool Search:**
- Tool Search Tool: ~500 tokens
- On-demand tools (3-5 relevant): ~3K tokens
- Total: ~8.7K tokens (85% reduction, 95% context preservation)

**Accuracy improvements:**
- Opus 4: 49% to 74%
- Opus 4.5: 79.5% to 88.1%

**Implementation:** Search via regex pattern matching or BM25 natural language search. Tool Search Tool type: `tool_search_tool_regex_20251119`. Best practice: keep 3-5 most-used tools with `defer_loading: false`, defer the rest.

**When to use:** 10+ tools available, definitions exceed 10K tokens, multiple MCP servers. Avoid when fewer than 10 tools or all tools see frequent use per session.

## 5. Prompt Caching

Anthropic's prompt caching stores and reuses frequently accessed context between API calls. Every block is cacheable: tool definitions, system messages, and message content.

**Pricing (Anthropic):**
- 5-minute cache write: 1.25x base input price
- 1-hour cache write: 2x base input price
- Cache read: 0.1x base input price (90% discount)
- Cache read tokens no longer count against ITPM rate limits for Claude 3.7 Sonnet+

**Latency:** Up to 85% reduction. 100K-token prompt: 11.5s to 2.4s with caching.

**Interaction with Tool Search:** Deferred tools are excluded from the initial prompt entirely, so system prompt and core tool definitions remain cacheable. This combines the 85% token reduction from Tool Search with the 90% cost reduction on cached reads.

**Simplified caching (March 2025):** Claude automatically reads from the longest previously cached prefix. No manual cache point tracking required.

**OpenAI prompt caching:** Automatic for prompts 1024+ tokens. Cached input tokens cost 50% of normal (vs Anthropic's 90% discount). Caches the entire prefix: messages, images, audio, tool definitions, structured output schemas. No additional fees.

## 6. Extended Thinking and Token Management

**Extended thinking (`budget_tokens`):** Minimum 1,024 tokens. With interleaved thinking (beta header `interleaved-thinking-2025-05-14`), budget_tokens can exceed max_tokens because it represents total budget across all thinking blocks in one turn.

**Tool use constraint:** Only `tool_choice: {"type": "auto"}` or `"none"` supported with extended thinking. Forced tool selection (`"any"` or specific tool) errors.

**Adaptive thinking (Claude 4.6):** Replaces fixed budget_tokens. Claude decides if/when to think based on problem complexity. Configure via `thinking: {"type": "adaptive"}` plus `output_config: {"effort": "high|medium|low"}`. No beta header needed on Opus 4.6.

**Practical impact on tool budgets:** Adaptive thinking skips reasoning on simple tool calls, reserves deep thinking for complex decisions. Reduces total token spend on multi-step workflows where most steps are straightforward.

## 7. Tool Use Examples (Anthropic)

Concrete usage examples in tool definitions improve parameter handling accuracy from 72% to 90%. Configured via `input_examples` on tool definitions. Show 1-5 examples per tool covering minimal, partial, and complete parameter specifications.

**When to use:** Complex nested structures, similar-named tools needing disambiguation, domain-specific conventions (date formats, ID patterns), many optional parameters.

## 8. OpenAI Equivalent Techniques

**Parallel function calling:** Models can issue multiple tool calls in a single response. Supported on GPT-4.1 and GPT-4o. Note: structured outputs are not compatible with parallel function calls (`parallel_tool_calls` must be false when using strict mode).

**Strict mode (structured outputs):** Setting `strict: true` ensures function calls reliably adhere to the schema. Recommended always. Uses constrained decoding for guaranteed schema compliance.

**Predicted outputs:** Speed up responses when output is partially known ahead of time (e.g., editing existing code). Available on GPT-4o, GPT-4.1 family. Does not save cost -- rejected prediction tokens are still billed. Latency optimization only.

**Prompt caching:** Automatic for 1024+ token prompts. 50% discount on cached input tokens. Caches tool definitions along with other static prefix content. No opt-in required.

**Token reduction strategies:** Limit number of functions loaded, shorten descriptions, use tool search so deferred tools load only when needed (OpenAI now recommends this pattern too).

## 9. Google Gemini Equivalent Techniques

**Function calling modes:** AUTO (default, model decides), ANY (forces function call, guarantees schema adherence), NONE (no function calls), VALIDATED (preview, schema compliance without forcing calls).

**Parallel function calling:** Supports multiple independent function calls simultaneously. Uses `tool_use_id` for result mapping. Results can arrive out of order.

**Compositional function calling:** Sequential chaining where Gemini invokes functions in dependency order. Python SDK supports automatic execution with type hints and docstrings.

**Code execution:** Built-in Python code execution where Gemini generates and runs code iteratively. Gemini 3+ supports code execution with images and multimodal function responses.

**MCP migration:** Google migrating from Tool Calling API to MCP by March 2026. Gemini Code Assist supports MCP as of October 2025.

**Optimization guidance:** Low temperature (0-0.2) for deterministic parameter extraction. Limit to 10-20 tools maximum. Simplify descriptions to reduce input token overhead.

## 10. Other Provider Optimizations

**Mistral:** Sparse mixture-of-experts architecture (Mistral Large 3: 41B active / 675B total parameters). Inherently token-efficient at inference due to sparse activation. $0.50/1M input, $1.50/1M output.

**Groq:** Custom LPU hardware delivers sub-100ms TTFT and 800+ tokens/second on Llama 3. 5-10x faster than GPU-based providers. Speed reduces wall-clock cost of multi-step tool workflows.

**Together AI:** High-performance inference for 200+ open-source models with sub-100ms latency. Supports tool use on most chat models.

**Model tiering pattern:** Use budget models (GPT-4.1-nano at $0.10/1M input, Haiku 4.5 at $1/1M) for simple tool routing and classification. Reserve flagship models for complex reasoning. 15-50x cost differential.

## 11. Practical Decision Tree

**Start here -- what is your primary bottleneck?**

1. **Too many tool definitions consuming context** -> Tool Search with `defer_loading: true` (85% reduction). Combine with prompt caching for cached core tools.

2. **Large intermediate results bloating context** -> Programmatic Tool Calling. Claude filters/aggregates in sandbox, returns only final output (37-98% reduction depending on workflow).

3. **Too many sequential LLM round-trips** -> PTC for code-based orchestration. Single script replaces 20+ inference passes. Use `asyncio.gather()` for parallel operations.

4. **Tool parameter errors** -> Add `input_examples` to tool definitions (72% to 90% accuracy improvement).

5. **Repeated static content across requests** -> Prompt caching. Place tool definitions and system prompt at prefix start. 90% cost reduction on Anthropic, 50% on OpenAI.

6. **All of the above at scale** -> Layer techniques: Tool Search (context) + PTC (execution) + Prompt Caching (cost) + Examples (accuracy). Anthropic's recommended deployment order: address context bloat first, then execution overhead, then parameter accuracy.

**Combining optimizations -- real-world measurements:**
- Tool Search alone: 85% token reduction
- PTC alone: 37% token reduction
- Code execution via MCP: 98.7% token reduction
- Prompt caching: 90% cost reduction on cached input
- All combined: 70-80% total cost reduction is realistic with good implementation
- Model tiering adds another 15-50x reduction on routable tasks

## Sources

- https://claude.com/blog/token-saving-updates
- https://www.anthropic.com/engineering/advanced-tool-use
- https://www.anthropic.com/engineering/code-execution-with-mcp
- https://platform.claude.com/docs/en/agents-and-tools/tool-use/programmatic-tool-calling
- https://platform.claude.com/docs/en/agents-and-tools/tool-use/token-efficient-tool-use
- https://www.geeky-gadgets.com/anthropic-tool-calling-updates/
- https://medium.com/ai-software-engineer/anthropic-just-solved-ai-agent-bloat-150k-tokens-down-to-2k-code-execution-with-mcp-8266b8e80301
- https://platform.openai.com/docs/guides/function-calling
- https://developers.openai.com/api/docs/guides/prompt-caching/
- https://platform.openai.com/docs/guides/predicted-outputs
- https://ai.google.dev/gemini-api/docs/function-calling
- https://ai.google.dev/gemini-api/docs/code-execution
- https://redis.io/blog/llm-token-optimization-speed-up-apps/
- https://platform.claude.com/docs/en/build-with-claude/extended-thinking
- https://platform.claude.com/docs/en/build-with-claude/adaptive-thinking
- https://fast.io/resources/ai-agent-token-cost-optimization/
