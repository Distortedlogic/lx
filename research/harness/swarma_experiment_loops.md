# Swarma: Growth Experiment Loops for Agent Swarms

## Identity

Swarma is a Python tool (v0.1.0, 129 GitHub stars, MIT license) that applies growth-marketing A/B testing methodology to AI agent swarms. Created by Rabin Nuchtabek (`glitch-rabin`), VP of Growth at Hype Partners, a Web3 marketing agency. Repo: `github.com/glitch-rabin/swarma`. Website: `swarma.dev`.

Swarma is **not** an agent orchestration framework. It is a **feedback loop layer** that sits on top of any agent pipeline. CrewAI, AutoGen, LangGraph run pipelines; Swarma adds the experiment-verdict-strategy-update cycle that makes pipelines improve autonomously over time.

From the README: "every growth team at Uber, Spotify, Facebook, Airbnb runs the same loop: hypothesize, test, measure, learn, repeat. the ones that win aren't smarter -- they just run more experiments and actually listen to the data."

The bottleneck insight: "even the best growth teams max out at 2-5 experiments per week. AI agents remove that constraint entirely. a swarm can run 50 experiments while a human team runs 2. but only if something is actually closing the loop -- scoring results, issuing verdicts, evolving the strategy."

Inspired by Karpathy's autoresearch pattern -- an autonomous experiment loop where an AI agent iteratively modifies code, trains briefly, evaluates, keeps or discards, and repeats (~12 experiments/hour).

## Core Architecture: The Experiment Loop

```
strategy.md -> execute -> measure -> verdict -> updated strategy.md
     ^                                              |
     +----------------------------------------------+
```

1. Agent reads its `strategy.md` before every run
2. Produces output (content, research, analysis -- whatever the team does)
3. A **cheap LLM** (cheapest model in the routing table) scores the output against the agent's metric on a 1-10 scale with forced decimals (e.g., 7.3 not 7)
4. The evaluator sees: the output, the current strategy, the last 5 scores, and the metric definition. Returns score + reasoning + strategy suggestion.
5. Score + reasoning logged to `results.tsv`
6. After `min_sample_size` cycles (default 3-5), verdict is issued automatically
7. `strategy.md` updated with what was learned
8. Next cycle uses the evolved strategy

**Verdict thresholds:**
- **>20% improvement** over baseline = **keep** (pattern validated, strategy updated)
- **>20% decline** = **discard** (reverted)
- **In between** = **inconclusive** (logged, try again with more data)

After several experiments, `strategy.md` accumulates validated findings:

```markdown
## Validated (Exp 5)
contrarian opening + specific numbers in first line
> 23% improvement over baseline. keep this pattern.

## Inconclusive (Exp 2)
story-led hooks vs data-led hooks -- no significant difference (avg=8.1 vs baseline=7.9)
> next: increase sample size, results may be noise
```

The key insight: the strategy document is the persistent, evolving artifact. It compounds learnings across hundreds of cycles. The agent swarm's "playbook" emerges from data, not from human intuition.

## Configuration: Teams as Folders

A team is a directory:

```
teams/my-squad/
  team.yaml          # goal, flow, schedule, budget
  program.md         # team context and constraints
  agents/
    researcher.yaml
    writer.yaml
```

**team.yaml:**
```yaml
name: my-squad
goal: find what works.
flow: "researcher -> writer"
schedule: "0 8 * * 1-5"
```

**Agent config (writer.yaml):**
```yaml
id: writer
name: Writer
instructions: |
  turn research into a post. max 200 words.
  hook in the first line. practitioner voice.
metric:
  name: content_quality
  target: 8.0
experiment_config:
  min_sample_size: 5
  auto_propose: true
```

**Flow DSL:**
- Sequential: `a -> b`
- Parallel: `a -> [b, c, d]`
- Mixed: combinations of both

No code required. Teams, agents, flows, schedules, and metrics are all YAML + Markdown.

## Coordination Model

Not real-time agent coordination. No message passing, no shared blackboards, no voting. The coordination patterns are:

1. **Sequential/parallel pipeline execution** via the flow DSL
2. **Shared learning via strategy.md** -- each agent reads its evolving strategy document
3. **Cross-team learning via QMD** -- when QMD is configured, every agent output gets indexed and any agent can search what other agents learned
4. **Verdict-driven strategy evolution** -- the system autonomously updates strategies based on statistical outcomes

## Integration

**MCP server** (stdio or HTTP):
```bash
swarma serve --mcp              # stdio
swarma serve --mcp --mcp-port 8383   # HTTP
```

Works with Hermes Agent (Nous Research), Claude Code, Claude Desktop, or any MCP client. Also exposes a REST API with 30+ endpoints (`swarma serve --port 8282`).

**Model access** via OpenRouter (500+ models). Scoring uses the cheapest available model to minimize cost.

**Knowledge layer** via QMD -- local BM25 + vector + LLM rerank semantic search. Without QMD, falls back to SQLite for metadata-only storage.

## What Swarma Explicitly Is Not

From the README:
- **Not memory** -- "Honcho does memory. swarma does learning loops."
- **Not automation** -- "n8n/Make do workflows. swarma runs experiments."
- **Not a prompt library** -- "agency-agents has 135 templates. swarma teaches them what works."
- **Not orchestration** -- "crewai/autogen run pipelines. swarma adds the feedback loop that makes pipelines improve."

## Positioning Against Orchestration Frameworks

| Framework | Core Purpose |
|-----------|-------------|
| **CrewAI** | Role-based multi-agent orchestration with task delegation |
| **AutoGen** | Conversational multi-agent collaboration |
| **LangGraph** | Graph-based agent workflows with explicit state management |
| **OpenAI Swarm** | Lightweight educational multi-agent handoffs (now superseded by Agents SDK) |
| **Swarma** | Experiment loop / A/B testing layer that sits **on top of** any pipeline |

Swarma complements rather than replaces these. You run CrewAI pipelines, wrap them with Swarma, and get autonomous strategy evolution.

## Status

Early stage. v0.1.0. 129 stars. Not on PyPI (install via `git clone` + `pip install -e .`). No HN front-page discussion. No significant community adoption yet. 10 pre-built example squads (hook-lab, format-wars, voice-finder, cta-optimizer, topic-radar, timing-lab, repurpose-engine, thread-lab, newsletter-lab, defi-alpha) -- all growth-marketing focused.

## Relevance to lx

**Experiment loops as a first-class pattern.** Swarma's core loop (execute → measure → verdict → update strategy → repeat) is a pattern lx could express natively. An `experiment` block with `metric`, `threshold`, and `verdict` semantics would let lx programs define self-improving agent workflows without external tooling. The loop is just: run an agent, score the output, decide keep/discard/inconclusive, update persistent state, repeat.

**Strategy-as-file evolution.** The `strategy.md` pattern -- a persistent document that accumulates validated findings across cycles -- is a form of durable agent memory that compounds. In lx terms, this could be a `state` block that persists across loop iterations and gets mutated by verdict logic. The key is that the strategy document is both readable by the agent (context) and writable by the system (verdict outcomes).

**LLM-as-judge for scoring.** Using a cheap model to score outputs against a metric is a lightweight eval pattern. lx could support `eval` blocks where a secondary model (or the same model) scores agent outputs against declared metrics. The forced-decimal scoring (7.3 not 7) to increase discrimination is a small but useful technique.

**Verdict thresholds as control flow.** The >20% keep / >20% decline / inconclusive trichotomy is a simple statistical control flow pattern. lx could express this as conditional branching on experiment results: `on keep { ... }`, `on discard { ... }`, `on inconclusive { ... }`.

**Teams-as-config (no code).** Swarma's YAML+Markdown configuration for teams, agents, flows, and schedules is exactly the declarative approach lx takes. The flow DSL (`a -> b`, `a -> [b, c]`) maps directly to lx's `pipe` and parallel execution semantics. Validates lx's design direction.

**Complementary layer, not replacement.** Swarma explicitly positions itself as a layer on top of orchestration frameworks, not a replacement. This suggests lx should be composable enough that experiment-loop semantics can wrap any workflow, not just specific agent patterns. A meta-workflow pattern.