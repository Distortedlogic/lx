# Subagent Lifecycle

Router-mediated subagent spawning, dispatch, and hookability.

## Target Goal

Route incoming prompts to the correct specialist agent using a catalog-based classifier. Spawn subagents with explicit configs (MCP servers, allowed tools, system prompt, max turns, budget). Distinguish terminal agents (cannot spawn children) from non-terminal orchestrators. Every tool call passes through hooks: langfuse trace, debug logging, budget guard, QC sampling.

## Scenarios

### Scenario 1: Clear Domain Match

Prompt: "audit running subagents for stuck loops." Router reads catalog, matches "security" domain to agent-auditor with confidence 0.9. Spawns agent-auditor (non-terminal). Agent-auditor spawns 3 terminal specialists.

**Success:** Router picks correct specialist in one step. Non-terminal agent successfully spawns children.

### Scenario 2: Multi-Domain Prompt

Prompt: "compare our rate limiting to what Redis does and find a Rust crate." Router scores: research=0.6, codebase=0.4, performance=0.3. Multiple domains relevant. Router delegates to research-analyst (non-terminal) which fans out to multiple specialists.

**Success:** Router picks the highest-confidence domain. Orchestrator handles multi-domain by spawning sub-specialists.

### Scenario 3: No Domain Match

Prompt: "what's the weather today?" Router scores all domains <0.2. No specialist matches. Falls back to caller handling directly.

**Success:** Router returns low confidence. Caller handles the prompt itself instead of spawning an irrelevant specialist.

### Scenario 4: Budget Guard Kills Runaway

Subagent is configured with max_turns=30. At turn 30, budget guard intervenes and kills the agent. Partial results returned to caller.

**Success:** Agent does not exceed configured limits. Caller receives partial results with an error indicating budget exhaustion.

### Scenario 5: Terminal Agent Cannot Spawn

Terminal specialist (e.g., researcher) attempts to spawn a child agent. The system rejects the spawn because terminal agents cannot create children.

**Success:** Spawn fails with clear error. Terminal agent must complete its work without delegation.

### Scenario 6: Utility Agent Reuse

Two different specialists both need web search. Both access the shared "researcher" utility agent. Results are independent — no cross-contamination between specialists.

**Success:** Utility agents serve multiple callers without shared state issues.
