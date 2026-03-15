# Agent Capability Discovery

Dynamic protocol for agents to advertise what they can do and query other agents' capabilities at runtime. Enables decentralized routing — an orchestrator asks each agent "can you handle this?" instead of maintaining a centralized registry.

## Problem

The `std/agents/router` classifies prompts against a static catalog of specialists. But real multi-agent systems need dynamic capability discovery:

- Agent pools where workers have different tool access
- Heterogeneous agent networks where capabilities change at runtime
- Load-aware routing where agents report remaining budget
- Self-organizing systems where new agents join and advertise their skills

Currently, the orchestrator must know each agent's capabilities at design time. There's no standard way for an agent to say "I can do X, Y, Z" at runtime.

## Capabilities Protocol

```
Protocol Capabilities = {
  protocols: [Str]
  tools: [Str]
  domains: [Str]
  budget_remaining: Int = -1
  accepts: [Str] = []
  status: Str = "ready"
}
```

Any agent can respond to a capabilities query:

```
handler = (msg) {
  msg.type ? {
    "capabilities" -> Capabilities {
      protocols: ["ReviewRequest" "AuditRequest"]
      tools: ["read_file" "grep" "ast_parse"]
      domains: ["rust" "security"]
      budget_remaining: introspect.budget.remaining
      status: "ready"
    }
    "review" -> do_review msg
    _ -> Err "unknown message type"
  }
}
```

### Fields

| Field | Type | Description |
|-------|------|-------------|
| `protocols` | [Str] | Protocol names this agent accepts |
| `tools` | [Str] | MCP tools this agent has access to |
| `domains` | [Str] | Semantic domains (free-form tags) |
| `budget_remaining` | Int | Remaining budget units (-1 = unlimited) |
| `accepts` | [Str] | Message types this agent handles |
| `status` | Str | "ready", "busy", "draining", "offline" |

## `agent.capabilities` — Query Helper

```
use std/agent

caps = agent.capabilities worker ^
caps.status == "ready" && (caps.domains | contains? "rust") ? {
  true  -> worker ~>? task ^
  false -> find_alternative task ^
}
```

`agent.capabilities` sends a `{type: "capabilities"}` message via `~>?` and validates the response against the `Capabilities` protocol. Returns `Result Capabilities AgentErr`.

## `agent.advertise` — Self-Registration

For agents that want to proactively advertise (rather than respond to queries):

```
agent.advertise {
  protocols: ["ReviewRequest"]
  domains: ["rust" "go"]
  tools: mcp.list_tools client ^ | map (.name)
}
```

`agent.advertise` stores the capability record in the agent's metadata, accessible by the parent and any supervisor. Combined with `std/knowledge`, this enables discovery across agent boundaries:

```
knowledge.store "agent:analyzer:caps" caps {source: "self" tags: ["capabilities"]} kb
```

## Dynamic Routing Pattern

```
route = (task agents kb) {
  candidates = agents | filter (a) {
    caps = agent.capabilities a ?? {status: "offline"}
    caps.status == "ready" && (caps.domains | any? (d) d == task.domain)
  }

  candidates | empty? ? {
    true  -> Err "no capable agent found for {task.domain}"
    false -> {
      best = candidates | min_by (a) {
        caps = agent.capabilities a ^
        caps.budget_remaining
      }
      best ~>? task ^
    }
  }
}
```

## Can-You-Handle Pattern

A lighter-weight alternative to full capability queries:

```
Protocol CanHandle = {type: Str = "can_handle"  task: Any}
Protocol CanHandleResponse = {can: Bool  confidence: Float = 1.0  cost: Int = 0}

responses = agents | pmap (a) {
  r = a ~>? CanHandle {task} ?? {can: false}
  {agent: a ..r}
}
best = responses | filter (.can) | max_by (.confidence)
```

## Cross-References

- Agent spawning and capabilities: [agents.md](agents.md)
- Capability attenuation: [agents-advanced.md](agents-advanced.md)
- Router standard agent: stdlib_roadmap (`std/agents/router`)
- Introspection: [stdlib-introspect.md](stdlib-introspect.md)
- Knowledge cache: [stdlib-knowledge.md](stdlib-knowledge.md)
