# Agent Capability Discovery — Reference

## Capabilities Protocol

```
Protocol Capabilities = {
  protocols: [Str]  tools: [Str]  domains: [Str]
  budget_remaining: Int = -1  accepts: [Str] = []  status: Str = "ready"
}
```

| Field | Type | Description |
|-------|------|-------------|
| `protocols` | [Str] | Protocol names this agent accepts |
| `tools` | [Str] | MCP tools this agent has access to |
| `domains` | [Str] | Semantic domains (free-form tags) |
| `budget_remaining` | Int | Remaining budget units (-1 = unlimited) |
| `accepts` | [Str] | Message types this agent handles |
| `status` | Str | `"ready"`, `"busy"`, `"draining"`, `"offline"` |

## `agent.capabilities` — Query

Sends `{type: "capabilities"}` via `~>?`, validates against `Capabilities` protocol. Returns `Result Capabilities AgentErr`.

```
use std/agent
caps = agent.capabilities worker ^
caps.status == "ready" && (caps.domains | contains? "rust") ? {
  true  -> worker ~>? task ^
  false -> find_alternative task ^
}
```

## `agent.advertise` — Self-Registration

Stores capability record in agent metadata, accessible by parent and supervisors:

```
agent.advertise {protocols: ["ReviewRequest"]  domains: ["rust" "go"]  tools: mcp.list_tools client ^ | map (.name)}
```

## Responding to Capability Queries

```
handler = (msg) {
  msg.type ? {
    "capabilities" -> Capabilities {
      protocols: ["ReviewRequest" "AuditRequest"]  tools: ["read_file" "grep" "ast_parse"]
      domains: ["rust" "security"]  budget_remaining: introspect.budget.remaining  status: "ready"
    }
    "review" -> do_review msg
    _ -> Err "unknown message type"
  }
}
```

## Dynamic Routing Example

```
route = (task agents kb) {
  candidates = agents | filter (a) {
    caps = agent.capabilities a ?? {status: "offline"}
    caps.status == "ready" && (caps.domains | any? (d) d == task.domain)
  }
  candidates | empty? ? {
    true  -> Err "no capable agent found for {task.domain}"
    false -> {
      best = candidates | min_by (a) { caps = agent.capabilities a ^  caps.budget_remaining }
      best ~>? task ^
    }
  }
}
```
