# Scoped Resource Blocks

`with ... as` syntax for automatic resource cleanup. Extends the existing `with name = expr { body }` scoped binding to support cleanup hooks on scope exit.

## Problem

MCP connect/close ceremony appears 14+ times across flows:

```
client = mcp.connect {command: "mcp-server" args: ["--mode" "stdio"]} ^
result = mcp.call client "tool" {input: data} ^
mcp.close client
```

If `mcp.call` errors and `^` propagates, `mcp.close` never runs. Same pattern with trace sessions, knowledge stores, file handles, agent handles — anything with setup/teardown.

## Syntax

```
with mcp.connect {command: "mcp-server" args: [...]} ^ as client {
  tools = mcp.list_tools client ^
  result = mcp.call client "analyze" {path: "src/"} ^
  result.findings
}
```

On scope exit (normal return or error propagation), the resource's cleanup runs automatically.

### Desugaring

```
with <expr> as <name> { <body> }
```

Desugars to:

```
let __resource = <expr>
let __result = (() { <body> }) ()
resource.close __resource
__result
```

With the critical addition: `resource.close` runs even if `body` errors. The body's error is re-raised after cleanup.

## `Closeable` Convention

A value is closeable if it has a `close` field that is a function, OR if the resource type is registered with the runtime. The runtime checks in order:

1. **Record with `.close` field**: calls `resource.close()` (no args)
2. **Known type**: MCP clients → `mcp.close`, agents → `agent.kill`, file handles → `fs.close`, trace sessions → `trace.end`
3. **Neither**: no cleanup (equivalent to plain `with name = expr { body }`)

This means any record can participate by including a `close` field:

```
make_thing = () {
  state := setup ()
  {
    use: (x) do_something state x
    close: () teardown state
  }
}

with make_thing () as thing {
  thing.use "data"
}
```

## Multiple Resources

```
with mcp.connect server1_config ^ as s1,
     mcp.connect server2_config ^ as s2 {
  tools1 = mcp.list_tools s1 ^
  tools2 = mcp.list_tools s2 ^
  tools1 ++ tools2
}
```

Resources close in reverse order (LIFO). If `s2` setup fails, `s1` is closed before the error propagates.

## Error Handling

```
with mcp.connect config ^ as client {
  mcp.call client "risky_tool" {data} ^
}
```

If `mcp.call` returns `Err` and `^` propagates:
1. `mcp.close client` runs (cleanup)
2. The `Err` propagates to the caller of the `with` block

If cleanup itself fails, the original error takes precedence. Cleanup errors are logged via `RuntimeCtx.log`.

## Integration Examples

### MCP (most common)

```
with mcp.connect {command: "server" args: []} ^ as client {
  mcp.call client "tool" {input} ^
}
```

### Trace Sessions

```
with trace.create {name: "audit" source: "security"} as session {
  trace.record session {name: "scan" input: target output: findings}
  trace.end session
}
```

### Agent Handles

```
with agent.spawn {command: "lx" args: ["run" "worker.lx"]} ^ as worker {
  worker ~>? {task: "analyze" data: input} ^
}
```

### File Handles

```
with fs.open "data.log" as fh {
  fs.write_line fh "entry: {data}"
}
```

## Implementation

### Parser

Extend the existing `with` parsing. When the parser sees `with <expr> as <name>`, it produces a `WithResource` AST node (distinct from `WithBinding`). Multiple resources separated by `,`.

### AST

```
Expr::WithResource {
    resources: Vec<(SExpr, String)>,
    body: Box<SExpr>,
}
```

### Interpreter

1. Evaluate each resource expression in order
2. Execute body in a new scope with resource bindings
3. On exit (normal or error), close resources in reverse order
4. Return body's value or re-raise body's error

Cleanup uses the `Closeable` convention described above.

## Cross-References

- Existing `with` scoped bindings: [bindings.md](bindings.md)
- MCP connections: [agents-advanced.md](agents-advanced.md)
- Agent spawning: [agents.md](agents.md)
- File handles: [stdlib-modules.md](stdlib-modules.md) (std/fs)
