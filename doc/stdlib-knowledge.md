# std/knowledge — Reference

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

`KB` is an opaque type backed by a JSON file. File-level locking prevents corruption during concurrent writes.

## Provenance Metadata

Every entry carries: `source` (which agent stored it), `confidence` (0.0–1.0), `tags` (for filtered queries).

## Querying

`knowledge.query` takes a filter function over entries:

```
auth_facts = knowledge.query (e) contains? "auth" (e.meta.tags ?? []) kb
high_confidence = knowledge.query (e) (e.meta.confidence ?? 0) > 0.8 kb
from_reviewer = knowledge.query (e) e.meta.source == "reviewer-agent" kb
```

## Key Conventions

- `"file:{path}"` — file reads
- `"tool:{name}:{args_hash}"` — MCP tool call results
- `"fact:{description}"` — derived conclusions

## Example — Cross-Agent Sharing

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
