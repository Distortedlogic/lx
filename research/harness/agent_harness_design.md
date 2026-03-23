# Agent Harness Design

## What Is an Agent Harness

An agent harness is every piece of code, configuration, and execution logic that wraps around an LLM to make it a reliable agent. The model is the engine; the harness is the car. Without steering, brakes, and fuel management, the engine goes nowhere useful. The equation: **Agent = Model + Harness**.

The harness controls five things: context management (what enters the context window, in what order, what gets evicted), tool selection (which capabilities the model can invoke), error recovery (how failed tool calls and reasoning dead-ends are handled), state management (how progress persists across turns and sessions), and external memory (how information is stored and retrieved beyond the context window).

"2025 was agents. 2026 is agent harnesses." The shift reflects a hard-won lesson: model capability is commodity; the harness is moat. On the CORE benchmark, the same Opus 4.5 model scored 78% with Claude Code's harness but only 42% with Smolagents. Manus rewrote their harness five times in six months with the same models, improving reliability each time. Vercel removed 80% of their agent's tools and got better results.

## Anthropic's Long-Running Agent Harness

Anthropic's approach uses a **two-agent initialization pattern**: an initializer agent sets up the environment on the first run, and a coding agent makes incremental progress in every subsequent session.

**JSON progress tracking**: Progress state is stored in JSON (`feature_list.json`), not Markdown. "The model is less likely to inappropriately change or overwrite JSON files compared to Markdown files." This prevents accidental corruption of critical state.

**One feature per session**: Coding agents work on only one feature at a time, preventing context exhaustion and the "one-shot" failure mode where agents attempt entire applications at once.

**Session initialization protocol** -- every session follows this sequence:
1. `pwd` to verify working directory
2. Git logs (20 lines) to review recent commits
3. Read `claude-progress.txt` for accumulated context
4. Consult `feature_list.json` for priorities (200+ features marked pass/fail)
5. Execute `init.sh` to start dev server
6. Run end-to-end verification tests before new work

Three artifacts bridge sessions: `init.sh` (reproducible environment), `claude-progress.txt` (session logs), and `feature_list.json` (comprehensive feature tracking that prevents premature completion).

## Context Management Techniques

**Just-in-time retrieval**: Rather than pre-loading all data, agents maintain lightweight identifiers (file paths, stored queries, URLs) and dynamically load context at runtime. Claude Code writes targeted queries and uses `head`/`tail` to analyze large datasets without loading full objects.

**Tool result clearing**: "One of the safest, lightest-touch forms of compaction." Once a tool executes deep in conversation history, the raw result becomes unnecessary. LangChain's Deep Agents offloads tool responses exceeding 20,000 tokens to the filesystem, substituting a file path reference and 10-line preview.

**Compaction hierarchy**: Raw conversation > compacted (redundant outputs removed) > summarized (LLM-generated structured summary with intent, artifacts, next steps). At 85% of the model's context window, older tool calls are truncated and replaced with pointers to files on disk.

**Sub-agent context compression**: Specialized sub-agents explore extensively using tens of thousands of tokens but return only a condensed summary of 1,000-2,000 tokens. This isolates context pressure from the lead agent.

**File-based memory**: Agents write plans, results, and notes to disk (`NOTES.md`, `todo.md`, scratch files) and read them back periodically. Manus treats the filesystem as unlimited, persistent, externalized memory. Plans are written to files for periodic re-reading; trajectories are stored for agent reference.

**Attention manipulation via recitation**: Manus creates and incrementally updates `todo.md` files, reciting objectives into the context's end to combat "lost-in-the-middle" phenomena across ~50 tool calls per task.

## Context Rot

Context rot is the performance degradation LLMs experience as input length increases, even when the context window is not close to full. A model with a 1M token window still exhibits context rot at 50K tokens. The degradation is non-linear: at 50% context usage everything is fine, at 65% nuance begins to fade, at 75% the agent is noticeably worse, and by 80% Claude Code's auto-compaction fires.

Research shows n-squared pairwise relationships for n tokens in transformer architecture, meaning every additional token interacts with all prior tokens, compounding attention dilution.

## Tool Harness Design

From Anthropic's "Writing Tools for Agents":

**Consolidate overlapping tools**: Instead of separate `list_users`, `list_events`, `create_event` tools, implement a single `schedule_event` that finds availability and schedules. Replace `read_logs` with `search_logs` returning only relevant lines with context. Fewer tools mean fewer steps, fewer tokens, faster responses.

**Response format enums**: Tools expose a `response_format` parameter with "detailed" (206 tokens) or "concise" (72 tokens) options, yielding a **65% output reduction**. This lets the agent choose verbosity based on task needs.

**Naming effects**: Prefix- vs suffix-based namespacing produces non-trivial effects on tool-use evaluations that vary by LLM. Service-based (`asana_search`) vs resource-based (`asana_projects_search`) naming affects model performance.

**Claude-optimized implementations**: Internal Slack and Asana MCP tools showed measurable accuracy gains when rewritten by Claude versus human-authored versions, validated on held-out test sets.

**25K token response cap**: Claude Code restricts tool responses to 25,000 tokens by default. UUID-to-semantic name conversion significantly reduces hallucinations in retrieval tasks.

**Tool count**: Claude Code uses ~18 primitives in four categories. Manus uses <20 tools. GitHub MCP's ~26K tokens across 35 tools is explicitly cited as an anti-pattern.

## Multi-Agent Harness

From Anthropic's multi-agent research system:

**Architecture**: Claude Opus 4 as lead agent, Claude Sonnet 4 as subagents. The multi-agent system outperformed single-agent Opus 4 by 90.2%, cutting research time by up to 90% for complex queries.

**Parallelization**: Lead agent spawns 3-5 subagents in parallel, each using 3+ tools in parallel. Resource allocation scales: simple fact-finding (1 agent, 3-10 calls), comparisons (2-4 subagents, 10-15 calls each), complex research (10+ subagents).

**Token economics**: Agents use ~4x more tokens than chat; multi-agent systems consume ~15x more. Token usage explains **80% of variance** in task success. An additional 15% is attributed to tool call count and model selection.

**Tool ergonomics > prompt engineering**: A 40% decrease in task completion time was achieved through improved tool descriptions alone, proving more influential than prompt refinement.

**Google's 180-experiment study**: Multi-agent variants degraded performance by 39-70% on sequential tasks (planning, state-dependent reasoning). Multi-agent improved performance by 81% on parallelizable tasks. If a single agent succeeds 45%+ of the time, one agent is better. Independent agents amplify errors up to 17x when mistakes propagate unchecked.

**HuggingFace smolagents**: Code agents (writing Python) outperform JSON tool-calling agents by ~30% in steps and LLM calls on complex benchmarks.

## State Persistence Patterns

**File-based progress tracking**: `claude-progress.txt` documents what agents completed across sessions. `feature_list.json` tracks 200+ features as pass/fail. Git commit + progress update at session end creates checkpoints.

**Git as state store**: Git history provides rollback capability, experiment branching, and serves as cross-session communication channel. Agents revert bad changes using git history and recover working states.

**Session bridging**: The "Ralph Wiggum Loop" pattern reinjects prompts in fresh context windows while maintaining filesystem state. Context lives in files; progress communicates via git history.

**Checkpoint/resume**: LangChain's Deep Agents writes complete message history to disk during summarization, enabling recovery via search. The full conversation can be reconstructed from filesystem artifacts.

## Cache-Optimized Harness Design

KV-cache hit rate is the single most important metric for production agents. Cached tokens cost $0.30/MTok versus $3.00/MTok uncached -- a **10x cost difference**. With Manus's 100:1 input-to-output ratio, cache optimization dominates economics.

**Ordering for cache stability**: Keep static content (system prompt, tool definitions) at the beginning; push volatile content (user input, dynamic values) to the end. Even a single-token difference invalidates the KV cache from that point onward. Timestamps precise to the second in system prompts completely kill cache reuse.

**Append-only contexts**: Never modify prior actions/observations. Use deterministic JSON serialization with stable key ordering. Insert explicit cache breakpoints accounting for expiration windows.

**Tool masking over removal**: Dynamically removing tools breaks KV-cache and confuses models via stale references. Manus uses state machines to mask token logits during decoding, constraining actions without modifying definitions.

**Benchmarked savings**: System-prompt-only caching provides the most consistent benefits, with 45-80% API cost reduction across providers. One customer improved hit rate from 60% to 87%. Annual cost difference at 10K requests/day: $95K (cache-optimized) vs $333K (cache-breaking).

## Error Handling in Harnesses

**Compound error problem**: Even small per-step error rates compound exponentially. A 1% per-step error rate fails after 100 steps. Google's research shows independent agents amplify errors up to 17x when mistakes propagate unchecked, while centralized coordination limits propagation to ~4.4x.

**Premature completion**: Agents declare projects "finished" without meeting requirements. Anthropic's solution: 200+ features all initially marked "failing" so coding agents see exactly what remains. `PreCompletionChecklistMiddleware` intercepts agent completion to force verification passes before exit.

**Doom loop detection**: `LoopDetectionMiddleware` tracks per-file edit counts. After N edits to the same file, it suggests reconsidering the approach to escape repetitive failure patterns.

**Error preservation**: Manus keeps failed actions and stack traces in context, allowing implicit model belief updates and prior shifts away from repeated mistakes. Error recovery itself indicates genuine agentic behavior.

**Reasoning budget allocation**: LangChain's "reasoning sandwich" distributes reasoning depth: high for planning, xhigh for implementation, high for verification. Constant max-reasoning causes timeouts. The xhigh-only approach scored 53.9% vs high-only at 63.6%.

## Real Harness Implementations

**Claude Code**: ~18 primitive tools in four categories (command-line, file interaction, web access, orchestration). Core loop is Think-Act-Observe-Repeat (`while(tool_call)`). No DAG orchestration, no competing agent personas. Six-layer memory loads at session start. Auto-compaction fires at ~80% context usage. Terminal Bench showed harness optimization alone moved rankings from Top 30 to Top 5.

**OpenAI Codex**: Sandboxed container execution with internet access disabled during tasks. Skills system bundles instructions, resources, and scripts for reliable workflow execution. Worker provisions container, launches App Server, maintains long-lived JSON-RPC over stdio. LLM inference is quadratic in conversation JSON size; prompt caching makes it linear.

**Manus**: <20 tools, ~50 tool calls per typical task. Five harness rewrites in six months. Context engineering strategy: reduce (aggressive compaction), offload (filesystem memory), isolate (sub-agent summaries). Todo recitation into context end combats goal drift.

**OpenHands**: Event stream architecture where all agent-environment interactions flow as typed events through a central hub. V1 refactored from monolithic sandbox-centric design to modular SDK with opt-in sandboxing. Model-agnostic, open-source.

**LangChain Deep Agents**: Terminal Bench 2.0 score improved from 52.8% to 66.5% (+13.7 points) through harness engineering alone (same GPT-5.2-Codex model). Three optimization areas: system prompts, tools configuration, middleware hooks.

## Harness Anti-Patterns

**Context flooding**: Giant instruction files crowd out the task, code, and relevant docs. The agent either misses key constraints or optimizes for the wrong ones. Too much guidance becomes non-guidance.

**Unbounded tool access**: GitHub MCP's 35 tools consuming ~26K tokens is the canonical anti-pattern. Successful agents use progressive disclosure -- tool definitions indexed and retrieved on-demand, MCP descriptions synced to folders for selective access.

**Storing everything in context**: Treating the context window as the primary storage medium instead of offloading to the filesystem. Context is a finite resource with diminishing marginal returns.

**Aggressive summarization**: Compression must be restorable. Manus drops web page content while preserving URLs, omits document text while maintaining file paths. LangChain writes complete messages to disk before summarizing, enabling recovery.

**Over-instrumentation**: Unnecessary data collection escalates costs and storage without improving agent performance. Collect only data that provides actionable insights.

**Dynamic tool modification**: Adding or removing tools mid-conversation breaks KV-cache and confuses models via stale references in conversation history. Use logit masking instead.

## Sources

- https://rlancemartin.github.io/2026/01/09/agent_design/
- https://www.anthropic.com/engineering/effective-harnesses-for-long-running-agents
- https://www.anthropic.com/engineering/effective-context-engineering-for-ai-agents
- https://www.anthropic.com/engineering/writing-tools-for-agents
- https://www.anthropic.com/engineering/multi-agent-research-system
- https://manus.im/blog/Context-Engineering-for-AI-Agents-Lessons-from-Building-Manus
- https://blog.langchain.com/the-anatomy-of-an-agent-harness/
- https://blog.langchain.com/context-management-for-deepagents/
- https://blog.langchain.com/improving-deep-agents-with-harness-engineering/
- https://openai.com/index/harness-engineering/
- https://openai.com/index/unrolling-the-codex-agent-loop/
- https://openai.com/index/unlocking-the-codex-harness/
- https://arxiv.org/html/2603.05344v1
- https://arxiv.org/html/2601.06007v1
- https://arxiv.org/html/2512.08296v1
- https://huggingface.co/blog/smolagents
- https://simonwillison.net/guides/agentic-engineering-patterns/anti-patterns/
- https://martinfowler.com/articles/exploring-gen-ai/harness-engineering.html
