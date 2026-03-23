# Domain-Specific Agent Harness Patterns

The harness -- not the model -- is the differentiating architecture. Manus rewrote theirs five times in six months, same models each time, each rewrite improving reliability. LangChain jumped from 52.8% to 66.5% on Terminal Bench 2.0 by changing only the harness. Vercel removed 80% of their agent's tools and got better results.

## 1. Coding Agent Harness

**Context priorities**: Codebase files > test output/compiler errors > project documentation (AGENTS.md) > conversation history. Claude Code's compressor activates at ~92% context window, summarizing older context and persisting to CLAUDE.md. Manus uses todo.md recitation to combat lost-in-the-middle degradation on ~50-tool-call tasks.

**Tool set**: Claude Code uses ~12 tools with a flat single-threaded loop (the nO master loop): View, LS, Glob, GrepTool (regex over embeddings), Edit (surgical diffs), Write (whole-file), Bash (persistent sessions with risk classification), WebFetch, NotebookRead/Edit, BatchTool, TodoWrite, and sub-agent dispatch. Devin provides shell, code editor, and browser in a sandboxed VM with vectorized codebase memory and full replay timeline. OpenDev uses a four-layer architecture (Entry/UI, Agent, Tool/Context, Persistence) with 9-pass fuzzy matching for file edits.

**Edit formats**: Claude Code uses search/replace diffs (Edit tool) as default, whole-file Write as fallback. OpenDev implements 9-pass fuzzy matching for partial edits to handle model imprecision. The Deep Agent architecture uses XML-wrapped YAML for action format, leveraging XML tags that LLMs generate reliably.

**Repo map pattern**: Claude Code bundles tree-sitter WASM modules for code structure understanding alongside vendored ripgrep binaries. OpenDev uses Language Server Protocol abstraction for multi-language semantic operations (symbol lookup, cross-file references, workspace-wide rename). Devin uses deepwiki.com and Devin Search for rapid codebase insight before delegation.

**Verification**: Tests, type checking, linting as feedback loops. Devin iterates autonomously until builds turn green. The Deep Agent architecture uses dedicated Explorer agents for post-implementation testing. OpenDev enforces Plan Mode (read-only) vs Normal Mode (full execution) at the schema level.

## 2. Research Agent Harness

**Context priorities**: Source documents > search results > synthesis notes > conversation history. Anthropic's lead agent saves strategy to external memory when approaching 200K tokens, spawning fresh subagents with clean contexts while maintaining continuity.

**Tool set**: Web search, document fetch, citation extraction, specialized search APIs. Agents receive heuristics: examine available tools first, match tool usage to intent, prefer specialized over generic. Bad tool descriptions send agents down completely wrong paths.

**Architecture**: Anthropic's system uses Opus 4 lead + Sonnet 4 subagents in an orchestrator-worker pattern. The lead agent analyzes queries, develops strategy, and spawns 3-5 subagents in parallel. This outperformed single-agent Opus 4 by 90.2%. Multi-agent systems consume ~15x more tokens than chat (single agents ~4x). Simple queries: 1 agent, 3-10 tool calls. Complex research: 10+ subagents with divided responsibilities.

**Verification**: A dedicated CitationAgent processes findings to identify specific citation locations, ensuring all claims are properly attributed. LLM-as-judge rubrics score factual accuracy, citation quality, completeness, source quality, and tool efficiency (0.0-1.0). Human evaluation discovered that early agents preferentially selected SEO-optimized content farms over authoritative academic PDFs.

**Deployment**: Rainbow deployments gradually shift traffic from old to new versions while keeping both running, because agent systems are stateful webs of prompts, tools, and execution logic that may be mid-task during updates.

## 3. Customer Support Agent Harness

**Context priorities**: Customer history + account data > knowledge base articles > conversation transcript > company policies. Sierra's Agent Data Platform unifies unstructured conversation data with structured data (billing, inventory, policies, transactions) into a single memory layer.

**Tool set**: CRM lookup, ticket management, knowledge base search, escalation triggers, refund processing. Sierra orchestrates 15+ frontier/open-weight/proprietary models via a constellation-of-models approach: fast models for order lookups (tight latency), specialized classifiers for behavior detection, long-context models for policy interpretation, tone-optimized models for sensitive interactions.

**Architecture**: Intercom's Fin uses a three-phase pipeline: Query Refinement (safety checks, workflow matching) -> Response Generation (RAG across content, dynamic data, third-party integrations) -> Validation (accuracy checks, grounding verification). Fin 2 achieved 99.9% accuracy and handles 50-80% of queries without human intervention.

**Verification**: Sierra uses supervisory oversight agents that enforce guardrails, policies, and quality checks. Built-in redundancy across model providers with automated failover when latency/error rates degrade. Microsoft's Quality Evaluation Agent grades support interactions.

**Handoff to human**: Sierra's Live Assist brings AI superpowers into human agent interactions -- real-time guidance, automatic detail capture, instant answer surfacing. The single biggest consumer frustration (cited by ~50% of consumers) is inability to reach a human when needed.

**Latency**: Voice conversations demand sub-2s response. Sierra routes simple operations to low-latency models while complex reasoning uses heavier models. Orchestration and routing handled automatically by the platform.

## 4. Computer-Use Agent Harness

**Context priorities**: Current screenshot > DOM/accessibility tree state > action history > task description. Manus sends three inputs to the model per browser step: text content of current viewport, screenshot, and a second screenshot with bounding boxes overlaid on clickable elements.

**Tool set**: Primitive UI actions -- click, type, scroll, screenshot, keyboard shortcuts, drag. Claude's computer use tool provides screenshot capture, mouse control, keyboard input, and desktop automation in a sandboxed Xvfb + Mutter + Tint2 environment. OpenAI's CUA combines GPT-4o vision with reinforcement-learning-trained GUI interaction.

**Architecture**: CUA operates through perception (screenshots) -> reasoning (chain-of-thought with inner monologue) -> action (mouse/keyboard) loops. Manus runs each task in a dedicated Ubuntu VM with full Chromium browser, Python 3.10, Node.js 20. Operator runs in isolated virtual browsers with domain limits, session timeouts, and context isolation between tasks.

**Verification**: Screenshot-after-every-action pattern -- "take a screenshot and carefully evaluate if you have achieved the right outcome." CUA seeks user confirmation for sensitive actions (login, CAPTCHA). All interaction history recorded and auditable.

**Error recovery**: Manus implements three-retry diagnostic approach (analyze error, try alternative, request user intervention after three failures). Reflective chain-of-thought (planning + error detection + correction at every step) boosts success rates by 30%+ over non-reflective traces. On OSWorld 50-step tasks, best agents (Simular Agent S2) reach only 34.5%.

**Long-horizon error accumulation**: The central bottleneck. Early mistakes distort later reasoning. Agents lose track over long sequences or exceed token limits. Manus combat this with todo.md constant rewriting at context end and filesystem-as-memory for unlimited persistent storage.

## 5. Data Analysis Agent Harness

**Context priorities**: Database schema > query results > visualization output > analysis notes. Schema understanding is prerequisite -- agents without it generate syntactically valid but semantically wrong queries.

**Tool set**: SQL execution (DuckDB, PostgreSQL), Python/pandas code execution, chart generation. The code-execution-via-MCP pattern treats MCP servers as code APIs rather than direct tool calls: agents write code to interact with servers, loading tools on demand.

**Verification**: Result validation through sanity checks on row counts, null percentages, and statistical distributions. The pandas-mcp-server runs code in a sandbox with minimal builtins (no __import__, eval, exec) and timeouts.

**Performance**: Code-execution-via-MCP reduced token usage from 150K to 2K tokens (98.7% reduction) by filtering data locally before model exposure. Agents save reusable code functions for future executions, building persistent skill libraries. Sensitive data is tokenized automatically before reaching the model, then untokenized during tool calls.

**Vercel case study**: Reducing from 15 specialized tools to 2 general-purpose ones (bash + SQL) increased accuracy from 80% to 100%, improved speed 3.5x, reduced tokens 37%. General-purpose interfaces leverage existing model training on shell commands.

## 6. Cross-Domain Comparison

| Dimension | Coding | Research | Support | Computer-Use | Data Analysis |
|---|---|---|---|---|---|
| Tool count | ~12 focused | Many search/fetch | CRM-integrated | ~5 primitives | 2-3 general |
| Verification | Tests + linting | Citation checking | Policy compliance | Screenshot diff | Result sanity |
| Context priority | Code + errors | Sources + synthesis | Customer + KB | Screenshot + DOM | Schema + results |
| Feedback loop | Compiler/test output | Source cross-ref | Customer satisfaction | Visual confirmation | Query results |
| State persistence | Git + progress files | External memory | CRM records | Filesystem + todo.md | Cached datasets |
| Latency tolerance | Minutes acceptable | Minutes acceptable | Sub-2s required | Seconds per action | Seconds per query |
| Error recovery | Re-run tests, iterate | Spawn fresh subagent | Escalate to human | Retry with screenshot | Re-execute query |
| Agent topology | Single + subagent | Lead + N subagents | Multi-model constellation | Single loop | Single loop |

**Universal patterns across all domains**: (1) Context window as the fundamental bottleneck -- every domain implements compaction/offloading. (2) Filesystem as external memory. (3) Append-only context for KV-cache optimization. (4) Fewer tools outperform more tools. (5) Error traces kept in context for implicit belief updating. (6) Human-in-the-loop at safety boundaries.

**Domain-specific divergences**: Coding agents verify through deterministic feedback (tests pass/fail). Research agents verify through source attribution. Support agents verify through policy compliance and tone. Computer-use agents verify through visual state comparison. Data agents verify through result plausibility.

## 7. Domain-Specific Failure Modes

**Coding agents**: (1) Premature completion -- agents declare victory before tests actually finish (Copilot's ~10s timeout bug). (2) Tests pass but code is wrong -- 45% of AI-generated code contains security vulnerabilities that pass functional tests (Veracode 2025). (3) Context loss on large codebases leading to inconsistent multi-file changes. (4) Retry loops on the same failed approach. Mitigation: human-critic feedback increased completion rates by 30%, reaching 80-90%.

**Research agents**: (1) Source bias -- preferring SEO-optimized content farms over authoritative PDFs. (2) Hallucinated citations -- claims attributed to sources that don't contain them, requiring dedicated CitationAgent verification. (3) Redundant investigation across subagents without shared Context Store. (4) Lead agent bottleneck from synchronous subagent execution. Mitigation: LLM-as-judge rubrics + human evaluation for edge cases.

**Support agents**: (1) Empathy gap -- 56% of professionals cite "AI with no empathy" as top challenge; only 60% consumer satisfaction with AI vs 88% with humans. (2) Policy violations when underlying business processes are broken. (3) Inability to handle novel situations outside knowledge base. (4) Escalation failure -- not knowing when to hand off. Mitigation: Sierra's multi-model constellation routing sensitive interactions to tone-optimized models.

**Computer-use agents**: (1) Error accumulation over long horizons -- early misclicks cascade through subsequent steps undetected. (2) Stuck states in navigation loops. (3) Visual misinterpretation when multiple similar elements exist. (4) Context overflow from screenshot history. Best OSWorld 50-step score is only 34.5%. Mitigation: reflective CoT at every step, todo.md recitation, three-retry diagnostic.

**Data analysis agents**: (1) Semantically wrong queries that are syntactically valid. (2) Hallucinated statistics not grounded in actual data. (3) Sandbox escape attempts through code generation. Mitigation: sandboxed execution with minimal builtins, result sanity checks against known distributions.

## 8. Tool Design Per Domain

**Coding -- few, powerful tools**: Claude Code's ~12 tools with ~50 lines of orchestration logic. The nO loop is a single-threaded `while(tool_call) -> execute -> feed results -> repeat`. Power from radical simplicity: regex over embeddings, markdown files over databases, flat message history over complex threading. Sub-agents cannot spawn sub-agents (depth limit prevents recursive explosion).

**Research -- many search/fetch tools**: Tool descriptions are critical. Agents need heuristics to choose between web search (broad exploration), specialized search (targeted queries), and document fetch (deep reading). Progressive tool discovery via filesystem hierarchy rather than loading all definitions upfront.

**Support -- CRM-integrated tools**: Tools tightly coupled to business systems. Sierra's Integration Library connects in minutes. The model doesn't choose tools -- the constellation-of-models platform routes to specialized models per task type automatically.

**Computer-use -- primitive UI actions**: 5 basic actions (click, type, scroll, screenshot, keyboard shortcut) combined through model reasoning. No API shortcuts -- the model must visually interpret every state. Manus augments with code execution (CodeAct paradigm) for non-browser tasks, using Python scripts as the universal action format.

**Data analysis -- general-purpose execution**: Vercel proved fewer tools win. Bash + SQL outperformed 15 specialized tools. The code-execution-via-MCP pattern lets agents write code that calls MCP servers, combining operations, conditionals, and library access in single code blocks.

## Sources

- https://www.anthropic.com/engineering/multi-agent-research-system
- https://manus.im/blog/Context-Engineering-for-AI-Agents-Lessons-from-Building-Manus
- https://sierra.ai/blog/constellation-of-models
- https://blog.promptlayer.com/claude-code-behind-the-scenes-of-the-master-agent-loop/
- https://gist.github.com/renschni/4fbc70b31bad8dd57f3370239dccd58f
- https://devin.ai/agents101
- https://dev.to/epappas/the-agent-harness-is-the-architecture-and-your-model-is-not-the-bottleneck-3bjd
- https://www.philschmid.de/agent-harness-2026
- https://sierra.ai/blog/agent-os-2-0
- https://www.anthropic.com/engineering/code-execution-with-mcp
- https://arxiv.org/html/2603.05344v1
- https://dev.to/apssouza22/a-deep-dive-into-deep-agent-architecture-for-ai-coding-assistants-3c8b
- https://www.nxcode.io/resources/news/harness-engineering-complete-guide-ai-agent-codex-2026
- https://platform.claude.com/docs/en/agents-and-tools/tool-use/computer-use-tool
- https://o-mega.ai/articles/the-2025-2026-guide-to-ai-computer-use-benchmarks-and-top-ai-agents
- https://www.intercom.com/help/en/articles/9929230-the-fin-ai-engine
- https://fayedigital.com/blog/fin-ai-agent/
- https://aakashgupta.medium.com/2025-was-agents-2026-is-agent-harnesses-heres-why-that-changes-everything-073e9877655e
- https://openai.com/index/computer-using-agent/
- https://workos.com/blog/anthropics-computer-use-versus-openais-computer-using-agent-cua
- https://blog.bytebytego.com/p/how-anthropic-built-a-multi-agent
