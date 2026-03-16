# Persistent Agent Profiles

Cross-session agent identity with accumulated knowledge, preferences, and relationship history.

## Problem

Agents are ephemeral. `std/memory` and `std/knowledge` are in-process only — process death loses everything. `agent.dialogue_save/load` (specced) preserves conversation history. `std/durable` (specced) preserves workflow checkpoints. `std/strategy` (specced) tracks approach outcomes.

None of these cover **agent identity continuity**: the accumulated preferences, domain knowledge, communication patterns, and relationship history that make an agent effective over time. When agent "reviewer-3" spawns tomorrow, it has zero memory of today. It doesn't remember which collaborators produce good work, what shortcuts it discovered, or what communication styles work with specific peers.

## Design

### Module: `std/profile`

File-backed agent profiles. Each profile is a named, persistent record of accumulated agent state.

```lx
use std/profile

p = profile.load "reviewer-3" ^
p = profile.load "reviewer-3" {create: true} ^

profile.learn p "parsing" {
  technique: "split on --- before regex"
  confidence: 0.9
  source: "self-discovered"
}

profile.learn p "collaborator:implementer-2" {
  quality: "high"
  response_time: "fast"
  preferred_protocol: "terse"
}

techniques = profile.recall p "parsing" ^
collabs = profile.recall_prefix p "collaborator:" ^

profile.preference p "output_format" "concise"
fmt = profile.get_preference p "output_format" ^

profile.save p ^
```

### Core Functions

| Function | Signature | Purpose |
|---|---|---|
| `profile.load` | `(name: Str) -> Result Profile Str` | Load or fail |
| `profile.load` | `(name: Str opts: {create: Bool}) -> Result Profile Str` | Load or create |
| `profile.save` | `(p: Profile) -> Result () Str` | Persist to disk |
| `profile.learn` | `(p: Profile domain: Str knowledge: Record) -> ()` | Store domain knowledge |
| `profile.recall` | `(p: Profile domain: Str) -> Result Record Str` | Retrieve domain knowledge |
| `profile.recall_prefix` | `(p: Profile prefix: Str) -> [Record]` | Retrieve all matching prefix |
| `profile.forget` | `(p: Profile domain: Str) -> ()` | Remove domain knowledge |
| `profile.preference` | `(p: Profile key: Str value: Any) -> ()` | Set preference |
| `profile.get_preference` | `(p: Profile key: Str) -> Result Any Str` | Get preference |
| `profile.history` | `(p: Profile) -> [{domain: Str learned_at: Str}]` | Learning timeline |
| `profile.merge` | `(a: Profile b: Profile) -> Profile` | Merge two profiles (b wins conflicts) |
| `profile.age` | `(p: Profile domain: Str) -> Result Int Str` | Seconds since domain was learned |
| `profile.decay` | `(p: Profile max_age_secs: Int) -> Int` | Remove stale entries, return count removed |

### Storage Format

JSON file at `{project_root}/.lx/profiles/{name}.json`. Fields:

```json
{
  "name": "reviewer-3",
  "created": "2026-03-16T10:00:00Z",
  "updated": "2026-03-16T14:30:00Z",
  "knowledge": {
    "parsing": {"technique": "split on ---", "confidence": 0.9, "learned_at": "..."},
    "collaborator:implementer-2": {"quality": "high", "learned_at": "..."}
  },
  "preferences": {
    "output_format": "concise"
  }
}
```

### Strategy Functions (absorbs `std/strategy`)

Strategy memory — "which approaches work for which problems" — is a specific domain of agent knowledge. Rather than a separate `std/strategy` module, strategy tracking lives in `std/profile` using the `strategy:` domain prefix convention.

```lx
profile.learn p "strategy:large_refactor:bottom_up" {
  score: 92
  context: {file_count: 45 language: "rust" complexity: :high}
}

profile.learn p "strategy:large_refactor:top_down" {
  score: 61
  context: {file_count: 45 language: "rust" complexity: :high}
}
```

Dedicated strategy helpers operate on `strategy:` prefixed domains:

| Function | Signature | Purpose |
|---|---|---|
| `profile.best_strategy` | `(p: Profile problem: Str) -> Result {approach: Str avg_score: Float count: Int trend: Str} Str` | Best approach by average score |
| `profile.rank_strategies` | `(p: Profile problem: Str) -> [{approach: Str avg_score: Float count: Int trend: Str}]` | All approaches ranked |
| `profile.adapt_strategy` | `(p: Profile problem: Str) -> {approach: Str mode: Str}` | Epsilon-greedy selection (explore/exploit) |
| `profile.adapt_strategy` | `(p: Profile problem: Str opts: {explore_rate: Float}) -> {approach: Str mode: Str}` | Custom explore rate |

Usage with `refine`:

```lx
best = profile.best_strategy p "code_review" ^
result = refine draft {
  grade: grade_fn
  revise: (work feedback) ai.prompt "Revise using {best.approach}: {work}\n{feedback}" ^
  threshold: 85
  max_rounds: 5
}
profile.learn p "strategy:code_review:{best.approach}" {score: result.final_score context: task_context}
```

Strategy data is stored in the same profile JSON file under knowledge domains prefixed with `strategy:`. The key format is `strategy:{problem}:{approach}`. Multiple scores per key are stored as a list (bounded to last 100), enabling trend and average computation.

### Relationship to Existing Modules

- `std/memory`: In-process episodic memory with tiers. `std/profile` is cross-session, domain-organized.
- `std/knowledge`: In-process knowledge cache. `std/profile` persists and is agent-scoped (not global).
- `agent.dialogue_save/load` (planned): Preserves conversation history. `std/profile` preserves learned knowledge.
- Eliminated: `std/strategy` — absorbed as `strategy:` domain prefix + helper functions.

### RuntimeCtx Integration

`profile.load`/`profile.save` use `ShellBackend`-adjacent file I/O (same as `std/knowledge`). No new backend trait needed — file path derived from `RuntimeCtx` project root config or cwd.

## Implementation

Pure stdlib module. No parser changes. Approximately 200 lines of Rust (150 core + 50 strategy helpers). File I/O reuses patterns from `std/knowledge` (JSON serialization, file locking).

## Priority

Tier 1. Fills a constant gap — every multi-session agent system needs persistent identity. Unblocked now. No dependencies on other planned features.
