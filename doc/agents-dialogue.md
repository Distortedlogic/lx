# Multi-Turn Agent Dialogue — Reference

## API

```
agent.dialogue agent config       -- Session ^ AgentErr  config: {role?: Str  context?: Str  max_turns?: Int}
agent.dialogue_turn session msg   -- a ^ AgentErr
agent.dialogue_history session    -- [{role: Str  content: a  time: Str}]
agent.dialogue_end session        -- ()
```

`Session` is opaque. Holds agent reference, conversation history, and configuration.

## Usage

```
use std/agent
session = agent.dialogue agent {role: "reviewer" context: codebase_summary} ^
r1 = agent.dialogue_turn session "look at the auth module" ^
r2 = agent.dialogue_turn session "now check if that pattern repeats in payments" ^
agent.dialogue_end session
```

Turns compose with operators:
```
findings = agent.dialogue_turn session "analyze auth" ^ | (.findings)
```

## Concurrent Dialogues

```
(sec_result perf_result) = par {
  with s = agent.dialogue sec_agent {role: "security"} ^ {
    agent.dialogue_turn s "audit auth" ^
    r = agent.dialogue_turn s "now check for injection" ^
    agent.dialogue_end s
    r
  }
  with s = agent.dialogue perf_agent {role: "performance"} ^ {
    r = agent.dialogue_turn s "suggest optimizations" ^
    agent.dialogue_end s
    r
  }
}
```

## Negotiation Pattern

Dialogue subsumes negotiation via Proposal/Contract protocols:

```
Protocol Proposal = {type: Str = "proposal"  task: Any  constraints: Record = {}  acceptance_criteria: [Str] = []  budget: Record = {}}
Protocol Contract = {task: Any  constraints: Record  acceptance_criteria: [Str]  budget: Record  parties: [Str]  rounds: Int}
```

```
session = agent.dialogue contractor {role: "negotiation"} ^
offer = agent.dialogue_turn session Proposal {task: "review"  constraints: {max_files: 50}  budget: {tokens: 10000}} ^
offer ? {
  {accept: true} -> agent.dialogue_turn session {action: "execute" contract: Contract {..offer.terms parties: ["caller" "contractor"] rounds: 1}} ^
  {counter}       -> agent.dialogue_turn session Proposal counter ^
  {reject: true}  -> Err "negotiation failed: {offer.reason}"
}
agent.dialogue_end session
```

## Gotchas

- Agent disconnect mid-dialogue: next `dialogue_turn` returns `Err Disconnected`.
- Exceeding `max_turns`: returns `Err {exceeded: "max_turns"}`.
- `dialogue_end` on already-ended session is a no-op.
