# Message Contracts (Protocol) — Reference

## Definition

```
Protocol ReviewRequest = {task: Str  path: Str  depth: Int = 3}
Protocol CalcRequest = {op: Str  value: Int}
```

Types: `Str`, `Int`, `Float`, `Bool`, `List`, `Record`, `Map`, `Tuple`, `Any`.
Optional fields have defaults (filled in when missing).

## Usage

Apply a Protocol to validate a record. Returns validated record on success, runtime error on failure.

```
msg = ReviewRequest {task: "audit" path: "src/"}
-- msg == {task: "audit" path: "src/" depth: 3}

CalcRequest {op: "double" value: "five"}
-- RUNTIME ERROR: Protocol CalcRequest: field 'value' expected Int, got Str

CalcRequest {op: "double"}
-- RUNTIME ERROR: Protocol CalcRequest: missing required field 'value'
```

## With Agent Communication

Validation happens before the message reaches the agent:

```
reviewer ~>? ReviewRequest {task: "audit" path: "src/"} ^
```

## Structural Subtyping

Extra fields are allowed — Protocols only check required fields exist with correct types:

```
Protocol Minimal = {id: Int}
Minimal {id: 1 name: "extra" tags: [1 2]}
-- returns {id: 1 name: "extra" tags: [1 2]}
```

## `Any` Type

```
Protocol Flexible = {key: Str  value: Any}
Flexible {key: "count" value: 42}       -- ok
Flexible {key: "name" value: "alice"}   -- ok
Flexible {key: "items" value: [1 2 3]}  -- ok
```

## Exports

```
+Protocol ReviewRequest = {task: Str  path: Str}
```
