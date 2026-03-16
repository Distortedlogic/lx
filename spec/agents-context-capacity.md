# Context Capacity Management

`std/context` tracks agent working memory capacity at runtime — how much context an agent has consumed, when it should summarize or evict, and how to degrade gracefully under pressure. This is the real-time constraint problem: LLM agents have finite context windows and silently lose information when they overflow.

Distinct from `std/ctx` (persistent key-value storage), `std/memory` (tiered long-term facts), and `std/budget` (cost accounting). Those track what's stored and what's spent. This tracks what's *in the window right now*.

## Problem

Every LLM agent has a hard context limit (200K tokens, 1M tokens, etc.). Currently there's no way to:

- Know how full the working window is
- Trigger compaction before information is silently lost
- Pin critical state that must never be evicted
- Choose eviction strategies when pressure is high
- Degrade gracefully (switch to shorter prompts, skip examples, summarize history)

Agents overflow silently. The orchestrator has no signal that an agent is about to lose context. Manual token counting is duplicated everywhere and doesn't compose with prompt assembly or LLM calls.

```
result = ai.prompt_with {prompt: huge_prompt} ^
// did this consume 80% of context? 95%? no way to know
// next call might silently lose the beginning of the conversation
```

## `std/context`

### Creating a Context Window

```
use std/context

win = context.create {capacity: 200000}
```

`capacity` is in tokens. The window tracks items added to it with metadata for eviction decisions.

### Adding Items

```
context.add win {
  key: "system_prompt"
  content: system_prompt
  tokens: 1200
  priority: :high
}

context.add win {
  key: "turn_3"
  content: turn_3_transcript
  tokens: 4500
  priority: :normal
}
```

Each item has a `key` (for lookup/eviction), `content` (the actual data), `tokens` (size), and optional `priority` (`:critical`, `:high`, `:normal`, `:low`). `tokens` can be computed via `context.estimate content` if not known.

### Querying Capacity

```
usage = context.usage win
// => {used: 5700  capacity: 200000  available: 194300  pct: 2.85}

pressure = context.pressure win
// => :low   (one of :low :moderate :high :critical)
```

Pressure thresholds (configurable):
- `:low` — < 50% used
- `:moderate` — 50-75% used
- `:high` — 75-90% used
- `:critical` — > 90% used

### Token Estimation

```
tokens = context.estimate "some text content"
// => 42 (approximate token count)

tokens = context.estimate_record {role: "user" content: "hello" examples: [...]}
// => 156 (recursive estimation)
```

Uses a fast approximation (chars / 4 for English, adjustable). Not exact — exact counting requires the target model's tokenizer.

### Pressure Callbacks

```
context.on_pressure win :high (level usage) {
  context.compact win :summarize
}

context.on_pressure win :critical (level usage) {
  context.evict win :lowest_priority
  log.warn "context critical: {usage.pct}% used"
}
```

Callbacks fire when pressure transitions to the specified level (or higher). Multiple callbacks per level are allowed.

### Pinning

```
context.pin win "system_prompt"
context.pin win "critical_state"

context.unpin win "critical_state"
```

Pinned items are never evicted by any strategy. They can still be explicitly removed.

### Eviction

```
context.evict win :oldest
context.evict win :lowest_priority
context.evict win :largest
context.evict_until win :oldest {target_pct: 60.0}
```

Strategies:
- `:oldest` — remove the oldest non-pinned item
- `:lowest_priority` — remove the lowest-priority non-pinned item
- `:largest` — remove the largest non-pinned item

`evict_until` repeats the strategy until usage drops below `target_pct`.

### Compaction

```
context.compact win :summarize
context.compact win :drop_examples
context.compact win :truncate {max_tokens_per_item: 500}
```

Compaction reduces size without removing items:
- `:summarize` — replace non-pinned, non-critical items with summaries (requires `std/ai`)
- `:drop_examples` — remove items tagged as examples
- `:truncate` — truncate items exceeding a token limit

### Querying Items

```
items = context.items win
// => [{key tokens priority pinned added_at} ...]

item = context.get win "turn_3"
// => Maybe {key content tokens priority pinned added_at}
```

### Removing Items

```
context.remove win "turn_3"
context.clear win  // removes all non-pinned items
```

## Integration Patterns

### With std/budget

```
use std/context
use std/budget

win = context.create {capacity: 200000}
b = budget.create {tokens: 50000}

context.on_pressure win :high (level usage) {
  budget.status b ? {
    :critical -> context.evict_until win :oldest {target_pct: 50.0}
    _ -> context.compact win :summarize
  }
}
```

### With std/prompt

```
use std/context
use std/prompt

win = context.create {capacity: 200000}
available = context.usage win | (.available)

p = prompt.create ()
  | prompt.system "You are a code reviewer"
  | prompt.section :task task_description
  | prompt.budget_trim available  // trim to fit available tokens
```

### With refine

```
result = refine draft {
  grade: (work) { ... }
  revise: (work feedback) {
    context.add win {key: "revision" content: work tokens: (context.estimate work)}
    context.pressure win == :critical ? {
      true -> context.compact win :summarize
      false -> ()
    }
    ai.prompt "Revise: {work}\nFeedback: {feedback}" ^
  }
  threshold: 85
  max_rounds: 5
}
```

## Implementation

`std/context` is a new stdlib module. The context window is a `Vec<ContextItem>` behind a `Mutex`. Each item holds key, content, token count, priority, pin status, and insertion timestamp.

Token estimation uses `content.len() / 4` as default approximation. The `:summarize` compaction strategy calls `std/ai` to generate summaries.

### Dependencies

- `parking_lot::Mutex` (thread-safe for `par` blocks)
- `std/ai` (optional, for `:summarize` compaction)
- `std/time` (timestamps on items)

## Cross-References

- Persistent storage: stdlib (`std/ctx`) — key-value context store, different concern
- Long-term memory: stdlib (`std/memory`) — tiered facts, different concern
- Cost budgets: [agents-budget.md](agents-budget.md) — cost accounting, complementary
- Prompt assembly: [agents-prompt.md](agents-prompt.md) — budget-aware rendering uses capacity
- Ambient context: [agents-ambient.md](agents-ambient.md) — capacity limits could flow as ambient
