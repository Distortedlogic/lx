# Standard Library — Agent Ecosystem

Agent communication, tool invocation, context management, and markdown processing modules. These are the primitives that make lx an agentic workflow language.

See [agents.md](agents.md) for patterns and design rationale. See [stdlib-modules.md](stdlib-modules.md) for core modules (fs, http, json, etc.).

## std/agent — Agent Lifecycle and Communication

```
spawn config              -- Agent ^ AgentErr
                          --   config: {name prompt model? tools? context? timeout?}
connect uri               -- Agent ^ AgentErr (connect to running agent)
kill agent                -- () ^ AgentErr (terminate agent)

send agent msg            -- () (fire-and-forget, msg is any value)
ask agent msg             -- a ^ AgentErr (request-response, blocks until reply)
ask_with opts agent msg   -- a ^ AgentErr (opts: {timeout: Duration  retries: Int})

submit agent msg          -- Task ^ AgentErr (async task, returns handle)
poll interval task        -- lazy [Status] (periodic status checks)
                          --   Status = {state: Str  progress: Maybe Float  result: Maybe a}
await_task task           -- a ^ AgentErr (block until task completes)
cancel task               -- () ^ AgentErr

channel agent             -- Channel ^ AgentErr (open persistent dialogue)
ch_send ch msg            -- () ^ AgentErr
ch_recv ch                -- a ^ AgentErr (blocks until message received)
ch_recv_timeout dur ch    -- Maybe a ^ AgentErr
ch_close ch               -- ()

name agent                -- Str
status agent              -- {state: Str  uptime: Duration}
```

`Agent`, `Task`, and `Channel` are opaque types.

`AgentErr` variants:
```
AgentErr = | SpawnFailed Str | Timeout | Disconnected | TaskFailed Str | ChannelClosed
```

### Patterns

Parallel fan-out:
```
agents | pmap (a) agent.ask a {action: "process"} ^
```

Race with fallback:
```
sel {
  agent.ask primary req   -> it
  agent.ask secondary req -> it
  timeout 30              -> Err "all agents timed out"
}
```

## std/mcp — Model Context Protocol

```
connect uri               -- McpClient ^ McpErr
                          --   uri: "stdio:///path" | "http://host:port" | "sse://host:port"
close client              -- ()

list_tools client         -- [{name: Str  description: Str  schema: a}] ^ McpErr
call client tool args     -- a ^ McpErr (invoke tool by name with args record)
call_with opts client tool args
                          -- a ^ McpErr (opts: {timeout: Duration})

list_resources client     -- [{uri: Str  name: Str  mime: Str}] ^ McpErr
read_resource client uri  -- {content: Str  mime: Str} ^ McpErr

list_prompts client       -- [{name: Str  description: Str  args: [Str]}] ^ McpErr
get_prompt client name args -- {messages: [{role: Str  content: Str}]} ^ McpErr
```

`McpClient` is an opaque type. `McpErr` variants:
```
McpErr = | ConnectionFailed Str | ToolNotFound Str | ToolError Str | Timeout | ProtocolError Str
```

### Patterns

Tool discovery and invocation:
```
client = mcp.connect "stdio:///usr/local/bin/server" ^
tools = mcp.list_tools client ^
tools | filter (t) contains? "file" t.name | each (t) $echo "{t.name}: {t.description}"
result = mcp.call client "read_file" {path: "src/main.rs"} ^
```

Multi-server orchestration:
```
(fs_client code_client) = par {
  mcp.connect "stdio:///fs-server" ^
  mcp.connect "stdio:///code-server" ^
}
files = mcp.call fs_client "list_dir" {path: "src/"} ^
files | pmap (f) mcp.call code_client "analyze" {path: f.path}
```

## std/ctx — Context and Memory

Structured key-value context for agent state persistence. Context values are immutable records — `set` returns a new context.

```
empty ()                  -- Ctx (empty context)
load path                 -- Ctx ^ IoErr (load from JSON file)
save path ctx             -- () ^ IoErr (save to JSON file)

get key ctx               -- Maybe a
set key val ctx           -- Ctx (returns new context with key set)
remove key ctx            -- Ctx
keys ctx                  -- [Str]
has? key ctx              -- Bool

merge ctx1 ctx2           -- Ctx (ctx2 wins on conflict)
from_record rec           -- Ctx
to_record ctx             -- {..}
```

`Ctx` is an opaque type backed by a persistent map.

### Patterns

Load-modify-save:
```
ctx.load "state.json" ^
  | ctx.set "last_run" (time.now () | to_str)
  | ctx.set "status" "running"
  | (c) ctx.save "state.json" c ^
```

Cross-session continuity:
```
state = ctx.load ".agent-state.json" ?? ctx.empty ()
prev_findings = ctx.get "findings" state ?? []
new_findings = analyze "src/"
all_findings = [..prev_findings ..new_findings] | uniq_by (.id)
ctx.set "findings" all_findings state | (c) ctx.save ".agent-state.json" c ^
```

## std/md — Markdown Processing

Parse, transform, and generate markdown. Agents use markdown extensively for memory, reports, and structured communication.

```
parse str                 -- MdDoc
sections doc              -- [{level: Int  title: Str  content: Str  children: [Section]}]
code_blocks doc           -- [{lang: Maybe Str  code: Str}]
frontmatter doc           -- Maybe %{Str: a} (YAML frontmatter)
headings doc              -- [{level: Int  text: Str}]
links doc                 -- [{text: Str  url: Str}]
to_text doc               -- Str (strip all formatting, plain text)
render doc                -- Str (back to markdown string)

doc nodes                 -- MdDoc (build from node list)
h1 text                   -- MdNode
h2 text                   -- MdNode
h3 text                   -- MdNode
para text                 -- MdNode
code lang text            -- MdNode (fenced code block)
list items                -- MdNode (bullet list from [Str])
ordered items             -- MdNode (numbered list from [Str])
table headers rows        -- MdNode (table from [Str] header + [[Str]] rows)
link text url             -- MdNode (inline link)
blockquote text           -- MdNode
hr                        -- MdNode (horizontal rule)
raw text                  -- MdNode (raw markdown string, no escaping)
```

`MdDoc` and `MdNode` are opaque types.

### Patterns

Extract knowledge from agent memory:
```
doc = md.parse (fs.read "MEMORY.md" ^)
tasks = md.sections doc | filter (s) s.title == "Tasks"
code = md.code_blocks doc | filter (b) b.lang == Some "lx"
```

Generate structured reports:
```
md.doc [
  md.h1 "Deploy Report — {time.now () | time.format "%Y-%m-%d"}"
  md.para "Deployed {artifact.name} to {env}"
  md.h2 "Health Checks"
  md.table ["Service" "Status" "Latency"]
    (results | map (r) [r.name r.status "{r.ms}ms"])
  md.h2 "Logs"
  md.code "text" (logs | join "\n")
] | md.render | (out) fs.write "deploy-report.md" out ^
```

## std/cron — Scheduling

Recurring task execution for long-running agent processes.

```
every interval f          -- Handle (run f every interval)
at cron_expr f            -- Handle (cron expression: "0 */5 * * *")
cancel handle             -- ()
```

```
use std/cron
health_check = cron.every (time.min 5) () {
  status = agent.ask monitor {action: "check"} ^
  status.healthy ? () : agent.send alerter {severity: "warn" msg: status.msg}
}
-- later: cron.cancel health_check
```

## Cross-References

- Agent patterns and design: [agents.md](agents.md)
- Concurrency primitives: [concurrency.md](concurrency.md)
- Core stdlib modules: [stdlib-modules.md](stdlib-modules.md)
- Data ecosystem: [stdlib-data.md](stdlib-data.md)
- Built-in functions: [stdlib.md](stdlib.md)
- Implementation: [implementation-phases.md](../impl/implementation-phases.md) (Phase 12)
