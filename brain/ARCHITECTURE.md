-- Brain Architecture: stable reference for fast orientation
-- Read this once when you need to understand the brain. Not required every tick.

## Overview

22 lx files modeling Claude's cognitive process. Not a demo — a working self-model
that exercises every major lx feature. If lx has a feature and the brain doesn't use it,
that's a gap.

## Entry Points

| File | Use case | How it works |
|------|----------|--------------|
| main.lx | Single-turn | Direct fn calls to lib modules, sel racing, with...as resources |
| orchestrator.lx | Continuous session | Saga pipeline, agent subprocesses via dispatcher, pub/sub, cron |

Both share the same lib modules, protocols, and traits.

## Module Map

### Core (data contracts)

| File | What it defines |
|------|-----------------|
| protocols.lx | 20 message protocols (Perception, Thought, ReasoningChain, Response, etc.) + Timestamped base + CognitiveEvent union + CognitivePhase ADT. All bounded fields have `where` constraints. |
| traits.lx | 12 behavioral contracts: 8 cognitive (Perceiver, Reasoner, Planner, Reflector, ToolUser, MemoryKeeper, SelfMonitor, Communicator) + 4 specialist (Analyst, Critic, Researcher, Synthesizer) |

### Lib (cognitive modules)

| File | Responsibility |
|------|---------------|
| perception.lx | Input → structured Perception (intent, complexity, entities, domain, git context) |
| reasoning.lx | 4 strategies: direct, decompose, analogical, adversarial. All use raw string prompts. |
| memory_mgr.lx | Tiered memory (working/episodic/semantic) via std/memory + std/knowledge + std/profile |
| tools.lx | MCP CognitiveTools declaration, curried filters (make_tool_filter), retry+circuit-breaker |
| context_mgr.lx | Context window pressure monitoring, eviction, compression |
| identity.lx | Core values, user.confirm + agent.gate destructive action pipeline, alignment checks |
| quality.lx | Refine loops with trace.should_stop diminishing returns, rubric-based grading |
| monitor.lx | Circuit breakers, budget, health guards, introspect.strategy_shift on non-continue |
| reflection.lx | Post-action evaluation, pattern extraction, batch reflection |
| dialogue.lx | Multi-turn conversation state + agent.dialogue wrappers |
| introspection.lx | Doom loop detection, time pressure, strategy analysis, self-narration |
| cognitive_saga.lx | Saga step definitions with Handoff-based context transfer and real compensation |

### Agents (specialist subprocesses)

| File | Agent Name | Trait | Advertised domains |
|------|-----------|-------|--------------------|
| analyst.lx | DeepAnalyst | Analyst | analyst, analysis, investigation, comparison |
| planner.lx | TaskPlanner | Planner | (not yet advertising) |
| critic.lx | InnerCritic | Critic | critic, review, verification, quality |
| researcher.lx | InfoGatherer | Researcher | researcher, research, search, information |
| synthesizer.lx | ResponseSynth | Synthesizer | synthesizer, response, formatting, composition |
| dispatcher.lx | (coordinator) | — | Spawns, routes (capability-based), fan_out (pmap_n 3), reconcile, supervise |

All specialist agents: advertise capabilities at startup, run yield/loop message handlers.

## Data Flow

```
Input
  → Perception (classify intent, extract entities, assess complexity, detect domain, git context)
  → Memory Recall (parallel: working + episodic + semantic)
  → Reasoning (strategy selected by complexity; sel racing for complex+ambiguous inputs)
  → Tool Execution (MCP-declared tools, retry with circuit breaker)
  → Response Assembly (integrate tool results or use reasoning conclusion)
  → Quality Gate (refine loop with diminishing returns detection)
  → Reflection (evaluate outcome, extract lessons, store memories)
  → Output (emit response)
```

Orchestrator wraps this in a saga with compensation. Each step has undo logic.
Context transfer between stages uses Handoff protocol + agent.as_context.

## Key lx Patterns in Use

Protocols with `where` constraints, protocol composition via `{..Base}`.
Traits with MCP-style method signatures. Agent declarations conforming to traits.
agent.spawn + ~>? messaging. agent.advertise + agent.capabilities for routing.
agent.intercept middleware for tracing. agent.gate + user.confirm for safety.
Pub/sub via agent.topic/subscribe/publish. Supervision via agent.supervise.
Refine loops with trace.should_stop. Saga with std/saga for compensation.
Record patterns with guards: `{field: value ..} & (guard)`.
Curried functions: `make_tool_filter = (cat) (tools) ...`.
Raw strings (backticks) for prompt literals.
Scoped resources: `with trace.create ... as session { ... }`.
sel racing for complex inputs. par for parallel work. pmap_n for rate-limited fan-out.

## What's NOT Used Yet (remaining gaps)

See brain/STATUS.md "Known Remaining Gaps" for the full list.
Key: agent.mock (no tests), std/agents/* (unused stdlib agents), lib-as-Agents pattern.
