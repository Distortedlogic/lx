# Multi-Turn Agent Dialogue

`~>?` is single request-response. Real agent collaboration is multi-turn — two agents debugging a problem, negotiating scope, or iteratively refining an approach. `dialogue` provides stateful conversation sessions where context accumulates across exchanges.

## Why Not Repeated `~>?`

Repeated `~>?` calls require the caller to manually thread conversation state:

```
r1 = agent ~>? {msg: "look at auth" history: []} ^
r2 = agent ~>? {msg: "check payments too" history: [r1]} ^
r3 = agent ~>? {msg: "propose unified approach" history: [r1 r2]} ^
```

Every call must carry the full history. The agent must parse and reconstruct context from the history field. No shared understanding accumulates — each message is structurally independent.

With `dialogue`, the runtime maintains conversation state on both sides:

```
use std/agent

session = agent.dialogue agent {role: "reviewer" context: codebase_summary} ^
r1 = agent.dialogue_turn session "look at the auth module" ^
r2 = agent.dialogue_turn session "now check if that pattern repeats in payments" ^
r3 = agent.dialogue_turn session "propose a unified approach" ^
agent.dialogue_end session
```

Each turn carries accumulated context. The agent on the other end sees the full conversation, not isolated messages.

## API

```
agent.dialogue agent config       -- Session ^ AgentErr
                                  --   config: {role?: Str  context?: Str  max_turns?: Int}
agent.dialogue_turn session msg   -- a ^ AgentErr (send turn, get response)
agent.dialogue_history session    -- [{role: Str  content: a  time: Str}]
agent.dialogue_end session        -- () (close session, release resources)
```

`Session` is an opaque type. It holds the agent reference, conversation history, and configuration.

## Protocol

Dialogue sessions communicate via JSON-line protocol. Each turn:

```
--> {"type":"dialogue_turn","session_id":"abc","content":"look at auth","history":[...]}
<-- {"type":"dialogue_response","session_id":"abc","content":{...}}
```

The runtime manages `session_id` assignment and history accumulation. The receiving agent sees the full history on every turn without the sender explicitly passing it.

## Composition

Dialogue turns compose with existing operators:

```
session = agent.dialogue worker {role: "analyst"} ^
findings = agent.dialogue_turn session "analyze the auth module" ^ | (.findings)
refined = agent.dialogue_turn session "focus on the critical issues" ^ | (.findings) | filter (.critical)
agent.dialogue_end session
```

## With `par`

Multiple dialogues can run concurrently:

```
(sec_result perf_result) = par {
  with s = agent.dialogue sec_agent {role: "security"} ^ {
    agent.dialogue_turn s "audit auth" ^
    r = agent.dialogue_turn s "now check for injection" ^
    agent.dialogue_end s
    r
  }
  with s = agent.dialogue perf_agent {role: "performance"} ^ {
    agent.dialogue_turn s "profile hot paths" ^
    r = agent.dialogue_turn s "suggest optimizations" ^
    agent.dialogue_end s
    r
  }
}
```

Each dialogue session is independent — no shared state between sessions.

## With Negotiation

Dialogue subsumes the negotiation pattern. Instead of single-shot Offer/Accept/Reject, agents negotiate over multiple turns:

```
session = agent.dialogue contractor {role: "negotiation"} ^
offer = agent.dialogue_turn session Offer {task: "review" constraints: {max_files: 50} budget: 10000} ^
offer ? {
  {commitment} -> agent.dialogue_turn session {action: "execute"} ^
  {reason counter_offer} -> {
    revised = agent.dialogue_turn session Offer counter_offer ^
    revised
  }
}
agent.dialogue_end session
```

## Error Handling

If the agent disconnects mid-dialogue, the next `dialogue_turn` returns `Err Disconnected`. If `max_turns` is exceeded, returns `Err {exceeded: "max_turns"}`. `dialogue_end` on an already-ended session is a no-op.

## Implementation Status

Planned. Depends on `std/agent` subprocess protocol extension.

## Cross-References

- Core agent primitives: [agents.md](agents.md)
- Single request-response: [agents.md](agents.md#~>?--ask)
- Negotiation pattern: [standard_agents.md](../design/standard_agents.md#negotiation)
- Streaming (related but distinct): [agents.md](agents.md#streaming)
