# std/knowledge — Shared Discovery Cache

When multiple agents work on the same problem, they independently discover the same facts — reading the same files, calling the same tools, building the same understanding. `std/knowledge` provides a shared, queryable knowledge cache that agents contribute to and read from.

## Why Not `std/blackboard` or `std/ctx`

`std/blackboard` is an in-memory concurrent workspace for `par` blocks — last-write-wins key-value. It doesn't persist across sessions, doesn't support structured queries, and doesn't distinguish between "agent A read this file" and "agent A concluded X from this file."

`std/ctx` is per-agent immutable key-value persistence. It's not shared between agents and has no query capabilities beyond `get`.

`std/knowledge` fills the gap: persistent, shared, queryable, with provenance metadata (who stored it, when, with what confidence).

## API

```
use std/knowledge

knowledge.create path                 -- KB ^ IoErr (create or open at path)
knowledge.store key val meta kb       -- KB ^ IoErr
                                      --   meta: {source?: Str  confidence?: Float  tags?: [Str]}
knowledge.get key kb                  -- Maybe {val: a  meta: Record  stored_at: Str}
knowledge.query filter kb             -- [{key: Str  val: a  meta: Record  stored_at: Str}]
knowledge.keys kb                     -- [Str]
knowledge.remove key kb               -- KB
knowledge.merge kb1 kb2               -- KB (kb2 wins on key conflict)
knowledge.expire before_time kb       -- KB (remove entries older than timestamp)
```

`KB` is an opaque type backed by a JSON file (like `std/ctx`).

## Provenance Metadata

Every entry has metadata tracking its origin:

```
knowledge.store "auth_module_structure" {
  entry_point: "src/auth/mod.rs"
  exports: ["Token" "Session" "refresh"]
  patterns: ["middleware chain" "token refresh on 401"]
} {source: "reviewer-agent" confidence: 0.9 tags: ["architecture" "auth"]} kb ^
```

`source` identifies which agent stored the entry. `confidence` is 0.0–1.0 (how certain the agent was). `tags` enable filtered queries.

## Querying

`knowledge.query` takes a filter function over entries:

```
auth_facts = knowledge.query (e) contains? "auth" (e.meta.tags ?? []) kb
high_confidence = knowledge.query (e) (e.meta.confidence ?? 0) > 0.8 kb
from_reviewer = knowledge.query (e) e.meta.source == "reviewer-agent" kb
```

Returns a list of matching entries with their keys, values, metadata, and timestamps.

## Cross-Agent Sharing

Any agent with the knowledge base path can read and write:

```
use std/knowledge

kb = knowledge.create ".project-knowledge.json" ^

par {
  agent_a ~>? {task: "review auth" kb_path: ".project-knowledge.json"} ^
  agent_b ~>? {task: "review payments" kb_path: ".project-knowledge.json"} ^
  agent_c ~>? {task: "review routing" kb_path: ".project-knowledge.json"} ^
}

all_findings = knowledge.query (e) contains? "finding" (e.meta.tags ?? []) kb
```

Each agent opens the same knowledge base, stores its discoveries, and can read what others found. File-level locking prevents corruption during concurrent writes.

## Deduplication

Before making an expensive tool call, an agent checks the knowledge base:

```
cached = knowledge.get "file:src/auth/token.rs" kb
cached ? {
  Some entry -> entry.val
  None -> {
    content = fs.read "src/auth/token.rs" ^
    knowledge.store "file:src/auth/token.rs" content {source: "self" tags: ["file"]} kb ^
    content
  }
}
```

Key convention for tool results: `"file:{path}"` for file reads, `"tool:{name}:{args_hash}"` for MCP tool calls, `"fact:{description}"` for derived conclusions.

## With std/memory

`std/knowledge` is session-scoped shared facts. `std/memory` (planned) is long-term agent-internal memory with tiered promotion. They complement each other:

- Knowledge: "src/auth uses middleware chain pattern" (shared, this session)
- Memory: "middleware chain patterns often have ordering bugs" (personal, learned over time)

An agent can promote high-confidence knowledge entries to its own memory after a session.

## Implementation Status

Planned. File-backed JSON with file-level locking for concurrent access.

## Cross-References

- Per-agent context: [stdlib-agents.md](stdlib-agents.md#stdctx)
- Concurrent workspace: [stdlib-modules.md](stdlib-modules.md#stdblackboard)
- Tiered memory: [stdlib_roadmap.md](../design/stdlib_roadmap.md#stdmemory)
- Agent introspection: [stdlib-introspect.md](stdlib-introspect.md)
