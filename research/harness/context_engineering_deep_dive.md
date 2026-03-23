# Context Engineering Deep Dive (March 2026)

## 1. Context Engineering as a Discipline

Context engineering displaced "prompt engineering" in mid-2025. Andrej Karpathy defined it as "the delicate art and science of filling the context window with just the right information for the next step." The shift reflects a move from crafting individual prompts to designing systems that curate, manage, and optimize everything an LLM sees across its entire lifecycle.

Practitioners do fundamentally different work: they build context pipelines with named, ordered processors rather than concatenating strings. They treat the context window as RAM, not a text box. Cognition (Devin) reported that context engineering is effectively the #1 job for engineers building AI agents. Manus rebuilt their agent framework 4 times; LangChain re-architected Open Deep Research 4 times. The biggest performance gains came from removing complexity, not adding features.

In LangChain's November 2025 survey of 1,340 respondents, 57.3% had agents in production and 32% cited quality as the top blocker -- quality that is directly downstream of context management.

## 2. Context Window Budget Allocation

Production systems allocate the 200K token window (standard Claude) across five zones:

| Zone | Budget | Typical Tokens | Notes |
|---|---|---|---|
| System instructions | 10-15% | 3-8K (peak 12K) | Behavioral guidelines, safety constraints |
| Tool definitions | 15-20% | 2-5K per 10 tools | Each API tool adds 200-500 tokens |
| Knowledge / retrieval | 30-40% | Variable | RAG results, documents, code |
| Conversation history | 20-30% | 20-150K | Grows linearly per turn |
| Buffer reserve | 10-15% | 15-25K | Output generation headroom |

Manus observed a 100:1 input-to-output token ratio on average across agent sessions. This ratio means input token cost dominates and makes caching economics paramount. Anthropic's context awareness feature injects remaining budget as XML tags (`<budget:token_budget>200000</budget:token_budget>`) after each tool call so the model can self-manage.

Production guidance: maintain 60-80% utilization as the optimal operating range. Compact before 70%, not at 90%. Quality degrades in distinct zones: Green (0-50%), Yellow (50-70% -- slight precision loss), Orange (70-90% -- noticeable degradation), Red (90%+ -- hallucinations, contradictions).

## 3. Compaction Strategies Ranked

The hierarchy is: Raw > Compacted > Summarized. Only escalate when the lighter approach is exhausted.

**Raw context** -- full tool outputs and messages kept verbatim. Ideal when within 50% window utilization. Zero information loss.

**Compaction (reversible)** -- strips information that exists in the environment. File contents removed when file paths remain available; web page content dropped when URLs are retained. The agent can re-read via tools if needed. This is "the safest, lightest-touch form of compaction" (Anthropic). Target compression: 3:1 to 5:1 for conversation history, 10:1 to 20:1 for tool outputs.

**Summarization (lossy)** -- condenses older messages into structured summaries. Factory.ai evaluated three production systems on 36,611 messages across 178-message sessions spanning 89K tokens:

| System | Overall Quality (0-5) | Compression Ratio | Accuracy |
|---|---|---|---|
| Factory | 3.70 | 98.6% | 4.04 |
| Anthropic | 3.44 | 98.7% | 3.74 |
| OpenAI | 3.35 | 99.3% | 3.43 |

OpenAI achieves highest compression (99.3%) but sacrifices interpretability with opaque representations. Anthropic produces structured 7-12K character summaries with explicit sections for analysis, files, pending tasks. Structured approaches act as checklists preventing silent information loss from freeform summarization.

**Practical pattern**: summarize oldest 20 turns into JSON while preserving last 3 turns raw. Keep the most recent tool calls in full-detail format to preserve the model's "rhythm" and formatting style. ForgeCode triggers compaction when ANY threshold is hit: token_threshold (80K-180K), message_threshold (150-200 messages), or turn_threshold. The retention_window (6-10 messages) stays untouched; only the eviction_window (20-30% of history) gets compressed.

ACON (2025) demonstrated that optimized compressor prompts lower memory usage by 26-54% while maintaining task performance and enable distillation into smaller models preserving 95% of teacher accuracy.

## 4. Tool Result Management

Tool results are the largest source of context bloat in agent loops. A 5-step coding task accumulates context as: 500 tokens (issue) -> 8K (file reads) -> 14K (additional context) -> 19K (test files) -> 20K+ total.

**Clearing old results**: The safest compaction form. Once a tool result is deep in history and the agent has already acted on it, replace the full output with a stub referencing the file path or URL. Tool call/result pairs must stay atomic -- never separate them.

**Truncating large outputs**: Target 10:1 to 20:1 compression for tool outputs. Agentic systems should return only relevant sections rather than full file contents.

**File-based offloading**: Write large outputs to disk, keep only the path in context. Manus treats the filesystem as unlimited persistent storage directly operable by agents. Web page content drops from context when URLs remain; document contents omit when file paths exist in the sandbox.

**Keep vs. discard heuristics**: Keep recent tool results (last 3-5 calls) in full. Keep error traces -- they help the model adjust behavior and represent "one of the clearest indicators of true agentic behavior" (Manus). Discard successful intermediate results that led to the current state.

## 5. Sub-Agent Context Isolation

Sub-agents explore extensively (tens of thousands of tokens or more) then return only condensed summaries (typically 1,000-2,000 tokens). The lead agent synthesizes results without being polluted by search context.

Design principle from Go concurrency: "Share memory by communicating, don't communicate by sharing memory." For discrete tasks, spin up fresh sub-agents with minimal context and pass only specific instructions. For complex reasoning where trajectory understanding is essential, share fuller history.

Anthropic's multi-agent research system reported "substantial improvement over single-agent systems on complex research tasks." The cost tradeoff: multi-agent can consume up to 15x more tokens than single-agent chat, but context isolation prevents degradation that would otherwise make the task impossible.

**Agent-as-Tool (MapReduce) pattern**: treat sub-agents as deterministic functions with defined goal, tools, and output schema (JSON). Returns structured results without parsing overhead.

**Planning efficiency**: Manus found their `todo.md` approach consumed ~30% of tokens in constant rewrites. Moving to a Planner sub-agent returning structured Plan objects, injected only when needed, solved this.

## 6. File-Based Memory Patterns

Claude playing Pokemon demonstrated the canonical example: given file access, Opus 4 spontaneously creates and maintains "memory files" -- navigation guides, objective tallies ("for the last 1,234 steps I've been training my Pokemon in Route 1"), strategic notes. This unlocked gameplay through tens of thousands of interactions across context resets.

**Progress files as session bridges**: Anthropic's long-running agent harness uses `claude-progress.txt` as persistent log across sessions. Each session reads git logs and progress files first, selects the next task, implements it, then updates documentation and commits. JSON feature lists over Markdown because "the model is less likely to inappropriately change or overwrite JSON files."

**Git as state store**: Commit after each feature with descriptive messages. Use git to revert failed changes. Progress files plus git history enable fast context recovery for new sessions.

**CLAUDE.md as persistent memory**: Loaded into context at session start, survives compaction (re-read from disk after /compact). Target under 200 lines. Auto-memory accumulates build commands, debugging insights, architecture notes across sessions.

**Scratchpad pattern**: Agents write to SCRATCHPAD.md to outline plans, list files to modify, track progress. This externalizes working memory and makes reasoning explicit without consuming permanent context.

## 7. KV Cache Optimization

Manus calls KV-cache hit rate "the single most important metric for a production-stage AI agent." With Claude Sonnet, cached tokens cost $0.30/MTok vs $3.00/MTok uncached -- a 10x price difference. Combined with batch processing, effective costs drop to $0.30 per million input tokens at 90% cache hit rate.

**Append-only ordering**: Keep system prompts static at the prefix. Even single-token differences invalidate all subsequent cache. Never put timestamps in prompts. Ensure deterministic JSON serialization with stable key ordering. Use session IDs for consistent routing across distributed workers.

**Tool masking over removal**: Manus masks token logits during decoding rather than dynamically removing tools mid-iteration. This preserves the KV-cache while constraining action selection. Tools use consistent prefixes (`browser_*`, `shell_*`) enabling logits-based constraints.

**Cache boundary control**: Exclude dynamic tool results from cached prefixes. Static components achieve 95%+ hit rates; semi-static content 60-80%; dynamic elements 0-20%. Strategic cache boundary control provides more consistent benefits than naive full-context caching, which paradoxically increases latency.

**Production economics**: Organizations implementing caching cut API costs by 75% for document Q&A. An arxiv study (2601.06007) found prompt caching reduces costs 45-80% and improves time-to-first-token 13-31% across providers. Full-context caching can paradoxically increase latency vs. strategic system-prompt-only caching.

## 8. Context Rot / Degradation

Chroma's 2025 research tested 18 frontier models with 194,480 LLM calls: every single model gets worse as input length increases.

**The 50% threshold**: When context is <50% full, models lose tokens in the middle (lost-in-the-middle effect). When >50% full, models lose the earliest tokens. Performance shows a gradient, not a cliff.

**Positional accuracy**: Position 1 (start) ~75% accuracy, Position 10 (middle) ~55% accuracy, Position 20 (end) ~72% accuracy. More than 30% performance reduction when relevant information sits in the middle (Liu et al., 2024).

**The 35-minute problem**: Every agent's success rate decreases after 35 minutes of operation. Doubling task duration quadruples the failure rate. Some runs consume 10x more tokens than others on similar tasks, driven by search efficiency rather than coding ability.

**Attention scaling**: 10K tokens = 100M pairwise relationships; 100K tokens = 10B; 1M tokens = 1T. The n-squared scaling means attention dilution grows quadratically with context length.

**GPT-4 structure sensitivity**: Performance degrades from 98.1% to 64.1% accuracy based solely on how information is structured within the context window.

**Detection signals**: Model asks for previously-provided information (50-70% utilization). Suggests generic solutions without code specifics (70-85%). Implements only partial requests (85-95%). Hallucinates code entities, contradicts earlier statements (95%+).

**Effective context window**: The high-quality operating range is typically <256K tokens for most models regardless of advertised limits. Anthropic's internal testing showed quality drop-off begins around 70% context utilization across all models.

## 9. Just-In-Time Retrieval

Instead of pre-loading everything, maintain lightweight identifiers (file paths, queries, links) and dynamically load via tools. This mirrors human cognition -- using external organization systems rather than memorization.

**Claude Code exemplifies this**: CLAUDE.md files load upfront for speed (procedural memory), while tools like `glob` and `grep` enable just-in-time retrieval, bypassing stale indexing and complex syntax trees.

**Handle pattern for artifacts**: Keep metadata (file path, URL, query) in context as a "handle." Load full content only when the agent's current step requires it. Metadata provides behavioral signals -- file hierarchies, naming conventions, timestamps hint at purpose.

**RAG for tool selection**: Applying RAG to tool descriptions improves accuracy by 3x when selecting from large tool collections. Reduces confusion from overlapping descriptions.

**Tradeoff**: Runtime exploration is slower than pre-computed retrieval. But it avoids stale indexes and pre-loading waste. Factory.ai injects repository overviews at session start that would otherwise cost "thousands of exploratory tokens," then uses targeted file system commands with explicit line number specifications for everything else.

**Hierarchical action space** (Manus/Phil Schmid): Level 1 (~20 atomic tools, stable, cache-friendly), Level 2 (sandbox tools like `bash`, `mcp-cli`), Level 3 (code/packages for complex chains -- let agents write scripts instead of multiple LLM roundtrips).

## 10. Emergent Context Behaviors

When given file access without explicit instructions to take notes, Claude Opus 4 spontaneously develops organizational behaviors:

**Note-taking**: Creates memory files tracking objectives, progress tallies, and strategic observations. In Pokemon, it maintained precise step counts and objective tracking across thousands of game steps.

**Map-keeping**: Built navigation guides mapping game locations, recording which areas had been explored and which remained.

**Achievement tracking**: Maintained lists of completed milestones and pending objectives, updating them after each significant event.

**Todo recitation**: Manus observed that agents naturally create and update `todo.md` files, "reciting objectives into the end of context" which counteracts the lost-in-the-middle effect after ~50 tool calls per task.

**Reflexive memory**: Generative Agents (2023) and Reflexion pioneered reflection-based memory. By 2025, ChatGPT, Cursor, and Windsurf auto-generate memories across sessions. Claude's auto-memory saves build commands, debugging insights, architecture notes, and workflow habits without user intervention.

These behaviors emerge because models are trained on internet-era developer workflows and are "unusually competent with developer-native interfaces like repos, folders, markdown, logs, and CLI-style interactions" -- which is why filesystems keep appearing in modern agent stacks.

## Sources

- https://www.anthropic.com/engineering/effective-context-engineering-for-ai-agents
- https://www.anthropic.com/engineering/effective-harnesses-for-long-running-agents
- https://manus.im/blog/Context-Engineering-for-AI-Agents-Lessons-from-Building-Manus
- https://factory.ai/news/context-window-problem
- https://factory.ai/news/evaluating-compression
- https://research.trychroma.com/context-rot
- https://www.morphllm.com/context-rot
- https://blog.langchain.com/context-engineering-for-agents/
- https://www.langchain.com/state-of-agent-engineering
- https://www.philschmid.de/context-engineering-part-2
- https://jxnl.co/writing/2025/08/30/context-engineering-compaction/
- https://arxiv.org/abs/2601.06007v1
- https://arxiv.org/abs/2510.00615 (ACON)
- https://platform.claude.com/docs/en/build-with-claude/context-windows
- https://deepwiki.com/FlorianBruniaux/claude-code-ultimate-guide/3.3-context-window-management
- https://forgecode.dev/docs/context-compaction/
- https://google.github.io/adk-docs/context/compaction/
- https://www.getmaxim.ai/articles/context-engineering-for-ai-agents-production-optimization-strategies/
- https://www.anthropic.com/news/claude-4
- https://www.producttalk.org/context-rot/
