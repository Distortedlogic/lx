# Dialogue Branching

Fork dialogue sessions for parallel exploration, then compare and select the best path.

## Problem

`agent.dialogue` tracks linear multi-turn sessions. But agents frequently need to explore multiple approaches in parallel:

- Tree-of-thought: "explore option A in one fork, option B in another, pick the best"
- Best-of-N sampling: "ask 3 differently-prompted agents the same question, grade results"
- Speculative execution: "try the risky approach and the safe approach simultaneously"

Today you manually create two separate dialogue sessions, drive them independently with duplicated conversation context, then use `agent.reconcile` to compare. The conversation history isn't shared — each fork starts with a copy, but there's no structural relationship between them, no way to compare at the dialogue level, and no way to merge a fork back into the parent.

## Design

### Extensions to `agent.dialogue`

```lx
use std/agent

session = agent.dialogue worker {role: "architect" context: project_spec} ^
r1 = agent.dialogue_turn session "Design the auth module" ^

(fork_a fork_b) = agent.dialogue_fork session ["Try JWT approach" "Try session-cookie approach"] ^

a_result = agent.dialogue_turn fork_a "Implement the JWT design" ^
b_result = agent.dialogue_turn fork_b "Implement the session design" ^

comparison = agent.dialogue_compare [fork_a fork_b] {
  grade: (session) {
    history = agent.dialogue_history session ^
    score = grader ~>? {work: history} ^ | (.score)
    {score  summary: last history | (.content)}
  }
} ^

best = comparison.best
agent.dialogue_merge session best ^

r3 = agent.dialogue_turn session "Now add rate limiting to the chosen approach" ^
```

### Core Functions

| Function | Signature | Purpose |
|---|---|---|
| `agent.dialogue_fork` | `(session: Session prompts: [Str]) -> Result (Session...) Str` | Fork into N branches, each starting with its prompt |
| `agent.dialogue_compare` | `(sessions: [Session] opts: {grade: Fn}) -> Result CompareResult Str` | Grade and rank forked sessions |
| `agent.dialogue_merge` | `(parent: Session winner: Session) -> Result () Str` | Merge winning fork's history back into parent |
| `agent.dialogue_branches` | `(session: Session) -> [Session]` | List active forks of a session |

### `dialogue_fork`

Creates N new sessions that share the parent's history up to the fork point. Each fork receives an initial prompt as its first turn. Forks are independent — turns on fork A don't affect fork B.

The parent session is suspended while forks are active. Attempting `dialogue_turn` on a suspended session returns `Err "session has active forks"`.

Returns a tuple of N sessions (same order as prompts list).

### `dialogue_compare`

Takes a list of forked sessions and a grading function. The grading function receives each session and returns `{score: Float summary: Str ...}`. Returns:

```lx
{
  rankings: [{session score summary}]  -- sorted by score descending
  best: Session                         -- highest-scoring fork
  spread: Float                         -- score range (max - min)
}
```

### `dialogue_merge`

Appends the winning fork's post-fork history to the parent session, then closes all forks. The parent resumes with the full context of the chosen path.

If you want to keep exploring, don't merge — just continue on the fork directly.

### Nested Forks

Forks can themselves be forked (tree of depth > 1). `dialogue_branches` returns only direct children. `dialogue_merge` at any level merges into the immediate parent.

### Integration

- `agent.reconcile` — compare works at the dialogue level; reconcile works at the result level. Use compare when you care about the conversation path, reconcile when you only care about final output.
- `agent.dialogue_save/load` (planned) — forks are part of the session state, so save/load preserves the fork tree.
- `std/trace` — each fork gets its own trace span, parented to the fork-point span.

## Implementation

Extension to existing `agent.dialogue` in `stdlib/agent_dialogue.rs`. Fork creates new session entries sharing a `parent_id` and a snapshot of history at fork time. Approximately 100 lines of additional Rust.

No parser changes. No new keywords.

## Priority

Tier 3. Enables tree-of-thought and best-of-N patterns that are common in sophisticated agentic workflows. Depends on existing `agent.dialogue` (implemented). Benefits from `agent.dialogue_save/load` (planned Tier 3) for persisting fork trees.
