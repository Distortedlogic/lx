# Agent Tool Orchestration: What Works in Production (Early 2026)

## 1. Fewer Tools Wins

Production agents use surprisingly few tools. Claude Code operates with ~15 core tools (Read, Write, Edit, Bash, Glob, Grep, WebFetch, WebSearch, NotebookRead, NotebookEdit, MultiEdit, LS, TodoRead, TodoWrite, plus task/agent tools). Manus runs with fewer than 20 tools. Amp Code curates "only a few tools." The GitHub MCP server alone has 35 tools consuming ~26,000 tokens of definitions -- more than many entire agents carry.

Vercel removed 80% of their agent's tools and got better results: fewer steps, fewer tokens, faster responses, higher success rates. The principle: tool definitions overload the context window, and intermediate results consume additional tokens as usage scales. Overlapping tool functionality confuses models.

Consolidation in practice means merging multi-step workflows into single purpose-driven tools. Anthropic recommends replacing `list_users` + `list_events` + `create_event` with a single `schedule_event` tool. Replace `read_logs` with `search_logs` that returns relevant lines with context. Replace `get_customer_by_id` + `list_transactions` + `list_notes` with `get_customer_context`. This reduces context consumed by intermediate outputs while letting agents approach tasks more naturally.

Manus pushes this further with hierarchical action spaces: a small set of atomic tools (primarily bash) that leverage shell utilities, CLIs, or code to perform arbitrary actions -- chaining many operations without tool-level overhead.

## 2. Code Execution as Meta-Tool

HuggingFace's CodeAct research (arxiv 2402.01030) found code agents achieve up to 20% higher success rates compared to JSON tool-calling agents, requiring 30% fewer steps (translating to ~30% cost reduction). The Code-Mode library benchmarks show the gap widens with complexity: simple tasks (2-3 tools) run 67% faster, medium tasks (4-7 tools) 75% faster, complex tasks (8+ tools) 88% faster versus traditional iteration.

Anthropic's Programmatic Tool Calling (PTC) delivers 37% average token reduction (43,588 down to 27,297 tokens on complex research tasks). When Claude orchestrates 20+ tool calls in a single code block, it eliminates 19+ inference passes. A budget compliance check across 20 employees shrinks from 200KB of raw expense data to ~1KB of filtered results.

PTC improved knowledge retrieval benchmarks from 25.6% to 28.5% and GIA benchmarks from 46.5% to 51.2%. On BrowseComp and DeepSearchQA, PTC was "the key factor that fully unlocked agent performance."

The core pattern: instead of ping-ponging JSON tool calls through the model, Claude writes Python/JS that calls tools programmatically, processes results in-sandbox, and returns only the final output to its context window. Loops, conditionals, data transformations, and error handling are explicit in code rather than implicit in reasoning.

## 3. Tool Response Optimization

Anthropic implements a `ResponseFormat` enum exposing `DETAILED` (206 tokens) and `CONCISE` (72 tokens) modes -- roughly 1/3 token reduction. The concise version strips technical IDs (`thread_ts`, `channel_id`, `user_id`) that are only needed for downstream tool calls, returning them only in detailed mode.

Claude Code caps tool responses at 25,000 tokens by default. For large outputs, implement pagination, range selection, filtering, and truncation with sensible defaults. Harnesses "keep the head and tail tokens of tool outputs" above threshold limits, storing full results in the filesystem for model access if needed.

Replace arbitrary alphanumeric UUIDs with semantically meaningful identifiers or 0-indexed ID schemes. This "significantly improves Claude's precision in retrieval tasks by reducing hallucinations." Prioritize contextual relevance over flexibility -- exclude fields like `mime_type` and `256px_image_url` unless they inform downstream actions.

Structure selection (XML, JSON, Markdown) impacts performance; optimal format varies by task and agent. No single winner -- benchmark your specific use case.

## 4. Tool Naming and Description Engineering

Anthropic recommends namespace prefixes: `asana_search`, `jira_search` for service-level grouping; `asana_projects_search`, `asana_users_search` for resource-level. Prefix vs suffix namespacing produces measurable differences in evaluation performance, varying by LLM model.

Manus uses consistent prefixes to enable constraint-based tool selection: all browser tools start with `browser_`, command-line tools with `shell_`. This lets the system mask token logits during decoding to enforce action selection by group without dynamic tool addition/removal (which would invalidate KV-cache).

Tool descriptions function as API documentation for the LLM. Treat them like onboarding a new team member: make implicit knowledge explicit, use unambiguous parameter names (`user_id` not `user`), provide concrete examples and boundaries including what not to do, and add annotations for destructive operations or open-world access.

Anthropic's iterative optimization process -- building tools with Claude Code, generating evaluation tasks, running programmatic eval loops, analyzing transcripts, then refactoring -- achieved "state-of-the-art performance on SWE-bench Verified" through "precise refinements" to tool descriptions. Most guidance in their tools-for-agents article came from "repeatedly optimizing internal tool implementations with Claude Code."

## 5. Error Message Design

Transform opaque errors into actionable guidance. Anthropic contrasts unhelpful approaches (raw error codes, stack traces) with helpful ones that "clearly communicate specific and actionable improvements" steering agents toward token-efficient strategies.

When truncation occurs, instruct agents to pursue "more token-efficient strategies, like making many small and targeted searches instead of a single, broad search." Error responses become teaching moments with corrected examples.

Production patterns for error recovery use four layers: (1) retry with backoff for transient errors, (2) model fallback chains for provider outages, (3) error classification to route errors correctly, (4) checkpoint recovery for crashes. Circuit breakers track failure rates and skip broken tools rather than hanging. LangChain's harness engineering adds loop detection middleware that triggers reconsideration prompts after repeated edits to prevent "doom loops."

## 6. Tool Execution Sandboxing

Claude Code uses OS-level primitives without containers. On macOS: Seatbelt. On Linux/WSL2: bubblewrap. Both enforce filesystem and network isolation on all child processes.

Filesystem isolation: read/write to working directory, read-only elsewhere, blocked modification outside project. Network isolation: proxy server running outside the sandbox enforces domain restrictions, with user confirmation for new domains. Internal testing shows sandboxing "safely reduces permission prompts by 84%."

Anthropic open-sourced this as `@anthropic-ai/sandbox-runtime` (npm package, also on GitHub at anthropic-experimental/sandbox-runtime). It works as both CLI tool and TypeScript library, enabling any agent project to sandbox processes without container overhead.

NVIDIA's guidance recommends full VM isolation (Kata containers, unikernels) for production since bubblewrap/seatbelt share the host kernel. Additional controls: block file writes outside active workspace at OS level, protect config files (`.zshrc`, `.gitconfig`), restrict DNS to trusted resolvers, inject only necessary credentials per-task through short-lived token brokers, and use ephemeral sandboxes with periodic destruction.

## 7. Parallel Tool Execution

When tools are independent, run them concurrently. Five 500ms tool calls in parallel save 2 full seconds versus sequential. Four 300ms calls complete in ~300ms total. Augment Code reports parallel tool calls make turns "at least 2x faster."

PTC enables this via `asyncio.gather()` patterns -- fetching budgets and expenses simultaneously across 20 team members in a single gathering operation. The fan-out/fan-in pattern fires multiple specialized agents in parallel with timeouts and backoff retries, then aggregates results. One benchmark showed a content workflow drop from 6:10 to 3:56 average (36% improvement) with parallelism.

LangChain's harness engineering found that strategic reasoning budget allocation (a "reasoning sandwich" of xhigh-high-xhigh reasoning levels) outperformed maximum reasoning at all steps: 63.6% vs 53.9% on Terminal Bench 2.0, demonstrating that the orchestration pattern matters as much as raw capability.

## 8. Tool Result Lifecycle

Manus treats the file system as "the ultimate context" -- unlimited in size and directly operable by the agent. Compression strategies remain reversible: web page content can be dropped if the URL persists; document contents can be omitted if the file path remains accessible.

With PTC, tool results bypass Claude's context window entirely. Results are processed by the script in the sandboxed environment, and Claude controls what information actually enters its context. Only final filtered output returns to the model.

Harnesses implement context compaction (intelligent summarization to continue work) and tool output offloading (storing full results in filesystem, keeping only head/tail tokens in context). Manus reports an average input-to-output token ratio of 100:1, with typical tasks requiring ~50 tool calls. With Claude Sonnet, cached tokens cost $0.30/MTok versus $3/MTok uncached (10x difference), making KV-cache stability critical -- even a single-token difference in prompt prefix invalidates the cache.

## 9. Dynamic Tool Sets

Static tool loading breaks at scale. A 400-tool MCP server consumes 405,000 tokens of definitions -- exceeding Claude's 200,000-token context. Two approaches to dynamic loading:

**Progressive discovery**: hierarchical meta-tools that the LLM queries to discover needed tools. Initial cost: ~2,500 tokens for 400 tools. Simple query completion: ~6,000 total tokens.

**Semantic search**: vector-based tool lookup. Initial cost: ~1,300 tokens regardless of tool count. Simple query: ~5,000 total tokens.

Both achieve roughly 100x token reduction versus static loading, maintaining constant token usage as tool count scales. Static approaches become impossible at 200+ tools.

Claude Code implements deferred tool loading -- tool schemas load only when the LLM explicitly requests them via a lightweight `ToolSearch` registry tool. Microsoft's agent-skills framework and Pydantic AI are adopting similar patterns. The consensus: progressive disclosure is the defining pattern for context management in agents with large tool sets.

## 10. Agent-Assisted Tool Development

Anthropic's iterative loop: (1) build prototype tools using Claude Code with SDK docs, (2) generate dozens of evaluation tasks as prompt-response pairs grounded in realistic workflows, (3) run evals programmatically using simple agentic while-loops alternating LLM calls and tool execution, (4) analyze transcripts and metrics to identify issues, (5) use Claude Code to analyze results and refactor tools.

Track: total runtime per tool call and complete task, total tool calls per task, total token consumption, tool error frequencies. "Lots of redundant tool calls might suggest some rightsizing of pagination or token limit parameters is warranted."

LangChain's harness engineering improved Terminal Bench 2.0 scores from 52.8% to 66.5% (13.7 percentage points, top 30 to top 5) through systematic tool and middleware optimization. They built an automated "Trace Analyzer Skill" that fetches experiment traces from LangSmith, analyzes errors in parallel, and synthesizes improvement suggestions -- automating hours of manual debugging.

Strong evaluation tasks require complex multi-step workflows: "Schedule a meeting with Jane next week to discuss our latest Acme Corp project. Attach the notes from our last project planning meeting and reserve a conference room." Weak tasks are overly simplistic: "Schedule a meeting with jane@acme.corp next week." Use held-out test sets to prevent overfitting to training evaluations.

## Sources

- https://www.anthropic.com/engineering/writing-tools-for-agents
- https://www.anthropic.com/engineering/advanced-tool-use
- https://www.anthropic.com/engineering/claude-code-sandboxing
- https://platform.claude.com/docs/en/agents-and-tools/tool-use/programmatic-tool-calling
- https://code.claude.com/docs/en/sandboxing
- https://huggingface.co/blog/smolagents
- https://huggingface.co/papers/2402.01030
- https://github.com/universal-tool-calling-protocol/code-mode
- https://github.com/anthropic-experimental/sandbox-runtime
- https://blog.langchain.com/the-anatomy-of-an-agent-harness/
- https://blog.langchain.com/improving-deep-agents-with-harness-engineering/
- https://rlancemartin.github.io/2026/01/09/agent_design/
- https://manus.im/blog/Context-Engineering-for-AI-Agents-Lessons-from-Building-Manus
- https://www.speakeasy.com/blog/100x-token-reduction-dynamic-toolsets
- https://developer.nvidia.com/blog/practical-security-guidance-for-sandboxing-agentic-workflows-and-managing-execution-risk/
- https://medium.com/@barunsaha/codeagent-the-evolution-beyond-tool-calling-7792781e19f4
- https://www.codeant.ai/blogs/parallel-tool-calling
- https://cookbook.openai.com/examples/agents_sdk/parallel_agents
