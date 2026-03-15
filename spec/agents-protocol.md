# Message Contracts (Protocol)

Protocols define the expected shape of agent messages. They validate record structure at the boundary — missing fields, wrong types, and non-record values are caught immediately with clear diagnostics.

## Defining Protocols

```
Protocol ReviewRequest = {task: Str  path: Str  depth: Int = 3}
Protocol CalcRequest = {op: Str  value: Int}
```

Fields have a name and a type. Optional fields have defaults (filled in when missing). Type checking uses runtime type names: `Str`, `Int`, `Float`, `Bool`, `List`, `Record`, `Map`, `Tuple`, `Any`.

## Using Protocols

Apply a Protocol to a record to validate it. Returns the validated record on success (with defaults filled in). Runtime error on failure.

```
msg = ReviewRequest {task: "audit" path: "src/"}
-- msg == {task: "audit" path: "src/" depth: 3}

CalcRequest {op: "double" value: 5}
-- returns {op: "double" value: 5}

CalcRequest {op: "double" value: "five"}
-- RUNTIME ERROR: Protocol CalcRequest: field 'value' expected Int, got Str

CalcRequest {op: "double"}
-- RUNTIME ERROR: Protocol CalcRequest: missing required field 'value'
```

## With Agent Communication

Protocol validation happens before the message reaches the agent:

```
Protocol ReviewRequest = {task: Str  path: Str}
reviewer = {handler: (msg) analyze msg.path msg.task}

reviewer ~>? ReviewRequest {task: "audit" path: "src/"} ^
-- validates, then sends {task: "audit" path: "src/"} to reviewer
```

## Structural Subtyping

Extra fields are allowed — Protocols check that required fields exist with correct types, but don't reject additional fields:

```
Protocol Minimal = {id: Int}
Minimal {id: 1 name: "extra" tags: [1 2]}
-- returns {id: 1 name: "extra" tags: [1 2]}
```

## `Any` Type

Use `Any` for fields that accept any value:

```
Protocol Flexible = {key: Str  value: Any}
Flexible {key: "count" value: 42}       -- ok
Flexible {key: "name" value: "alice"}    -- ok
Flexible {key: "items" value: [1 2 3]}   -- ok
```

## Exports

Protocols can be exported with `+` and imported via `use`:

```
+Protocol ReviewRequest = {task: Str  path: Str}
```

## Cross-References

- Agent communication: [agents.md](agents.md)
- Design rationale: [design.md](design.md)
- Interceptors (Protocol validates before middleware): [agents-intercept.md](agents-intercept.md)
- Test suite: [../tests/14_agents.lx](../tests/14_agents.lx)
