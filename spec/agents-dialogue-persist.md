# Dialogue Persistence

`agent.dialogue_save` and `agent.dialogue_load` persist multi-turn dialogue sessions to storage. Dialogues survive process restarts, enabling long-running agent-to-agent and agent-to-user conversations that span hours or days.

## Problem

`agent.dialogue` creates ephemeral sessions held in memory. If the process dies mid-dialogue, the entire conversation history is gone. This matters for:

- Agent-to-user coding sessions that span multiple work sessions
- Multi-day review dialogues between agents
- Handoff between orchestrator restarts (orchestrator crashes, restarts, needs to resume dialogue with worker agents)
- Audit trails requiring complete dialogue history

`std/durable` (planned) persists workflow state, but workflow state and conversation state are different. A workflow knows "step 3 of 5 completed." A dialogue knows "turns 1–17 of a conversation about auth refactoring, with full message history and accumulated context."

## API

### Save / Load

```
use std/agent

session = agent.dialogue worker {role: "reviewer"  context: "auth module"} ^
r1 = agent.dialogue_turn session "check error handling" ^
r2 = agent.dialogue_turn session "what about the middleware?" ^

agent.dialogue_save session "review-auth-2026-03" ^

-- later, possibly different process
session = agent.dialogue_load "review-auth-2026-03" worker ^
r3 = agent.dialogue_turn session "any final concerns?" ^
```

### `agent.dialogue_save`

```
agent.dialogue_save session id -> () ^ DialogueErr
```

Persists the session's full state (config, turn history, accumulated context) to storage under the given `id`. Overwrites if `id` already exists.

### `agent.dialogue_load`

```
agent.dialogue_load id agent -> DialogueSession ^ DialogueErr
```

Loads a previously saved session and binds it to the given agent. The agent may be a different process instance than the original — only the conversation state transfers, not the process handle.

### `agent.dialogue_list`

```
agent.dialogue_list () -> [DialogueInfo] ^ DialogueErr
```

Lists all saved dialogues:

```
DialogueInfo = {
  id: Str
  role: Str
  turns: Int
  created: Str
  updated: Str
  context_preview: Str
}
```

### `agent.dialogue_delete`

```
agent.dialogue_delete id -> () ^ DialogueErr
```

### Error type

```
DialogueErr = | NotFound Str | StorageErr Str | CorruptState Str
```

## Persisted State

```
DialogueState = {
  id: Str
  config: {role: Maybe Str  context: Maybe Str  max_turns: Maybe Int}
  turns: [{
    index: Int
    direction: Str
    message: Str
    response: Maybe Str
    timestamp: Str
  }]
  created: Str
  updated: Str
}
```

`direction` is `"outbound"` (orchestrator → agent) or `"inbound"` (agent → orchestrator).

## Storage Backend

Dialogue state serializes to JSON. Default storage is file-based:

```
.lx/dialogues/
  review-auth-2026-03.json
  deploy-planning.json
```

The storage location is relative to the project root (where `lx.toml` lives). Falls back to `~/.lx/dialogues/` if no project root is found.

### Custom storage

Embedders can implement `DialogueStorage` trait on `RuntimeCtx` for database-backed or remote storage:

```rust
trait DialogueStorage: Send + Sync {
    fn save(&self, id: &str, state: &DialogueState) -> Result<(), String>;
    fn load(&self, id: &str) -> Result<DialogueState, String>;
    fn list(&self) -> Result<Vec<DialogueInfo>, String>;
    fn delete(&self, id: &str) -> Result<(), String>;
}
```

Default implementation: `FileDialogueStorage` using `std/fs` and `std/json`.

## Patterns

### Resume coding session

```
session_id = "feature-auth-refactor"

session = agent.dialogue_load session_id coder ?? {
  agent.dialogue coder {role: "developer"  context: fs.read "CONTEXT.md" ^} ^
}

response = agent.dialogue_turn session user_input ^
agent.dialogue_save session session_id ^
```

Load if exists, create if not. Save after every turn.

### Audit trail

```
sessions = agent.dialogue_list () ^
  | filter (d) d.turns > 5
  | sort_by (.updated)
  | rev

sessions | each (d) {
  state = agent.dialogue_load d.id dummy_agent ^
  emit "Dialogue {d.id}: {d.turns} turns, last updated {d.updated}"
  agent.dialogue_history state ^ | each (t) emit "  [{t.index}] {t.message}"
}
```

### Cross-agent handoff with dialogue history

```
session = agent.dialogue_load "review-session" reviewer1 ^
history = agent.dialogue_history session ^
context = history | map (.message) | join "\n---\n"

handoff = {
  from: "reviewer1"
  to: "reviewer2"
  reason: "shift change"
  context: context
  _protocol: "Handoff"
}

session2 = agent.dialogue reviewer2 {
  role: "reviewer"
  context: agent.as_context handoff
} ^
```

## Implementation

### Additions to `std/agent`

Four new functions in `agent.rs`: `dialogue_save`, `dialogue_load`, `dialogue_list`, `dialogue_delete`.

### Serialization

`DialogueState` serializes to/from JSON using the same `std/json` infrastructure. Turn history is a list of records.

### Dependencies

- `std/fs` (file-based storage)
- `std/json` (serialization)
- `std/time` (timestamps)

## Cross-References

- Dialogue system: `agent.dialogue` / `agent.dialogue_turn` / `agent.dialogue_history` in [stdlib-agents.md](stdlib-agents.md)
- Workflow persistence: [agents-durable.md](agents-durable.md) — workflow state vs conversation state
- Handoff: `agent.as_context` + `Handoff` Protocol
- Context: [agents-context-capacity.md](agents-context-capacity.md) — dialogue history as context pressure
- Package manifest: [package-manifest.md](package-manifest.md) — storage location relative to project root
