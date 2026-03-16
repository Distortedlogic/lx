# Agent Primitives — Reference

## Agent Values

An agent is a record with a `handler` field. Factories are closures over config.
```
echo = {handler: (msg) msg}
make_multiplier = (factor) {handler: (x) x * factor}
```

## `~>` Send (fire-and-forget, returns `()`), `~>?` Ask (request-response), `~>>?` Stream (lazy sequence, agent uses `yield`)
```
logger ~> {action: "log" data: results}
result = analyzer ~>? {task: "review" path: "src/"}
agent ~>>? {task: "analyze"} | filter (.important) | each (r) log.info r.summary
```

## Composition with `^`, `|`, `??`
```
analyzer ~>? {task: "review"} ^ | (.findings) | filter (.critical)
fallback_agent ~>? request ?? default_response
```

## Precedence

`~>`, `~>?`, `~>>?` bind at `++` level (tighter than pipe, looser than arithmetic):
`agent ~>? msg ^ | process` parses as `((agent ~>? msg) ^) | process`
## Multiline Continuation
```
result = analyzer
  ~>? {task: "review" path: "src/"}
  | (.findings)
  | filter (.critical)
```

## Multi-Agent Orchestration
```
(security perf docs) = par {
  sec_agent ~>? {task: "audit" path: "src/"} ^
  perf_agent ~>? {task: "profile" path: "src/"} ^
  docs_agent ~>? {task: "check-coverage" path: "src/"} ^
}

tasks = files | pmap (f) {
  a = agent.spawn {name: "reviewer" prompt: "Review {f}"} ^
  a ~>? {file: f action: "review"} ^
}

raw = fetcher ~>? {url: api_url} ^
parsed = parser ~>? {data: raw.body format: "json"} ^
summary = summarizer ~>? {data: parsed findings: 10} ^
```

## MCP Tool Invocation
```
use std/mcp
local = mcp.connect {command: "npx" args: ["-y" "mcp-server"]} ^
remote = mcp.connect "https://api.example.com/mcp" ^
tools = mcp.list_tools remote ^
result = mcp.call remote "read_file" {path: "src/main.rs"} ^
mcp.close remote
```

## Context and Memory
```
use std/ctx
memory = ctx.load "memory.json" ^
last_run = ctx.get "last_run" memory ?? "never"
memory = ctx.set "last_run" (time.now () | to_str) memory
ctx.save "memory.json" memory ^
```

## Protocol Validation
```
Protocol ReviewRequest = {task: Str  path: Str  depth: Int = 3}
reviewer ~>? ReviewRequest {task: "audit" path: "src/"} ^
```

## Multi-Turn Dialogue
```
session = agent.dialogue worker {role: "reviewer"} ^
r1 = agent.dialogue_turn session "look at auth module" ^
agent.dialogue_end session
```

## Message Interceptors
```
traced = agent.intercept worker (msg next) {
  log.debug "sending: {msg | to_str}"
  next msg
}
```

## Channels and `sel`
```
sel {
  agent.ch_recv ch1 -> handle_response "agent1" it
  timeout 30        -> Err "no response"
}
```
