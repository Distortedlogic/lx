# Multi-Turn Agent Dialogue

`~>?` is single request-response. Real agent collaboration is multi-turn — two agents debugging a problem, negotiating scope, or iteratively refining an approach. `dialogue` provides stateful conversation sessions where context accumulates across exchanges. Dialogue also subsumes the negotiation pattern — multi-round proposal/counter-proposal is a dialogue with specific Protocol shapes.

## Why Not Repeated `~>?`

Repeated `~>?` calls require the caller to manually thread conversation state:

```
r1 = agent ~>? {msg: "look at auth" history: []} ^
r2 = agent ~>? {msg: "check payments too" history: [r1]} ^
r3 = agent ~>? {msg: "propose unified approach" history: [r1 r2]} ^
```

With `dialogue`, the runtime maintains conversation state on both sides:

```
use std/agent

session = agent.dialogue agent {role: "reviewer" context: codebase_summary} ^
r1 = agent.dialogue_turn session "look at the auth module" ^
r2 = agent.dialogue_turn session "now check if that pattern repeats in payments" ^
r3 = agent.dialogue_turn session "propose a unified approach" ^
agent.dialogue_end session
```

Each turn carries accumulated context.

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

The runtime manages `session_id` assignment and history accumulation.

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

## Negotiation Pattern

Dialogue subsumes negotiation. Multi-round proposal/counter-proposal uses dialogue with Proposal/Counter/Contract protocols:

```
Protocol Proposal = {
  type: Str = "proposal"
  task: Any
  constraints: Record = {}
  acceptance_criteria: [Str] = []
  budget: Record = {}
}

Protocol Contract = {
  task: Any
  constraints: Record
  acceptance_criteria: [Str]
  budget: Record
  parties: [Str]
  rounds: Int
}
```

### Negotiation via dialogue

```
session = agent.dialogue contractor {role: "negotiation"} ^
offer = agent.dialogue_turn session Proposal {
  task: "review"
  constraints: {max_files: 50}
  budget: {tokens: 10000}
} ^
offer ? {
  {accept: true} -> {
    contract = Contract {..offer.terms parties: ["caller" "contractor"] rounds: 1}
    agent.dialogue_turn session {action: "execute" contract} ^
  }
  {counter} -> {
    revised = agent.dialogue_turn session Proposal counter ^
    revised
  }
  {reject: true} -> Err "negotiation failed: {offer.reason}"
}
agent.dialogue_end session
```

The receiving agent's handler processes proposals:

```
handler = (msg) {
  msg.type == "proposal" ? {
    true -> {
      can_afford msg.budget ? {
        true -> {accept: true terms: msg}
        false -> {counter: {..msg budget: {tokens: msg.budget.tokens * 2}} reason: "need more budget"}
      }
    }
    false -> process msg
  }
}
```

## Error Handling

If the agent disconnects mid-dialogue, the next `dialogue_turn` returns `Err Disconnected`. If `max_turns` is exceeded, returns `Err {exceeded: "max_turns"}`. `dialogue_end` on an already-ended session is a no-op.

## Implementation Status

Planned. Depends on `std/agent` subprocess protocol extension.

## Cross-References

- Core agent primitives: [agents.md](agents.md)
- Handoff: [agents-handoff.md](agents-handoff.md) (handoff can initialize a dialogue)
- Streaming (related but distinct): [agents.md](agents.md#streaming)
