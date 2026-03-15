# Standard Library — Agent Ecosystem

Agent communication, tool invocation, context management, and markdown processing modules. These are the primitives that make lx an agentic workflow language.

See [agents.md](agents.md) for patterns and design rationale. See [stdlib-modules.md](stdlib-modules.md) for core modules (fs, http, json, etc.).

## std/agent — Agent Lifecycle and Communication

```
spawn config              -- Agent ^ AgentErr
                          --   config: {command: Str  args: [Str]}
kill agent                -- () ^ AgentErr (terminate subprocess)
```

Subprocess agents communicate via JSON-line protocol over stdin/stdout. Use `~>` (send, fire-and-forget) and `~>?` (ask, request-response) as infix operators on agents with a `__pid` field.

`Agent` is an opaque type.

`AgentErr` variants:
```
AgentErr = | SpawnFailed Str | Timeout | Disconnected
```

### Patterns

Parallel fan-out:
```
agents | pmap (a) a ~>? {action: "process"} ^
```

Race with fallback:
```
sel {
  primary ~>? req   -> it
  secondary ~>? req -> it
  timeout 30        -> Err "all agents timed out"
}
```

## std/mcp — Model Context Protocol

```
connect target            -- McpClient ^ McpErr
                          --   target: {command: Str  args: [Str]} (stdio)
                          --         | Str (HTTP URL)
close client              -- ()

list_tools client         -- [{name: Str  description: Str  schema: a}] ^ McpErr
call client tool args     -- a ^ McpErr (invoke tool by name with args record)
```

`McpClient` is an opaque type. `McpErr` variants:
```
McpErr = | ConnectionFailed Str | ToolNotFound Str | ToolError Str | Timeout | ProtocolError Str
```

### Patterns

Tool discovery and invocation:
```
client = mcp.connect {command: "server" args: []} ^
tools = mcp.list_tools client ^
tools | filter (t) contains? "file" t.name | each (t) $echo "{t.name}: {t.description}"
result = mcp.call client "read_file" {path: "src/main.rs"} ^
```

Multi-server orchestration:
```
(fs_client code_client) = par {
  mcp.connect {command: "fs-server" args: []} ^
  mcp.connect {command: "code-server" args: []} ^
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

## Cross-References

- Agent patterns and design: [agents.md](agents.md)
- Concurrency primitives: [concurrency.md](concurrency.md)
- Core stdlib modules: [stdlib-modules.md](stdlib-modules.md)
- Built-in functions: [stdlib.md](stdlib.md)
- Implementation: [implementation-phases.md](../impl/implementation-phases.md) (Phase 12)
