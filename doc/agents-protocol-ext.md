# Protocol Extensions — Reference

## Composition (`{..Base}`)

Spread an existing Protocol's fields into a new Protocol:

```
Protocol Base = {name: Str  severity: Str  source: Str}
Protocol PerfFinding = {..Base  location: Str  issue: Str  fix: Str}
```

`PerfFinding` has 6 fields: 3 from `Base` + 3 own. Defaults carry over.

Override: fields in the extending Protocol replace same-named base fields. Multiple bases: `{..Named  ..Scored  url: Str}` — on collision, later spread wins.

## Unions (`A | B | C`)

A Protocol that accepts any of several Protocol variants:

```
Protocol ReviewRequest = {task: Str  path: Str}
Protocol AuditRequest = {severity: Str  scope: Str}
Protocol AgentMessage = ReviewRequest | AuditRequest
```

### Validation

Tries each variant in order. First variant whose required fields all match wins. If none match, runtime error listing all variants tried.

### `_variant` Injection

Matched records get an automatic `_variant` field with the Protocol name:

```
AgentMessage {task: "review" path: "src/"}
-- => {_variant: "ReviewRequest"  task: "review"  path: "src/"}
```

Enables dispatch in handlers:

```
handler = (msg: AgentMessage) {
  msg._variant ? {
    "ReviewRequest" -> do_review msg
    "AuditRequest"  -> do_audit msg
    _               -> Err "unhandled message type"
  }
}
```

## Field Constraints (`where`)

Predicates on field values, validated after type checking:

```
Protocol AuditFinding = {
  severity: Str where severity in ["low" "medium" "high" "critical"]
  score: Float where score >= 0.0 && score <= 1.0
  name: Str where len name > 0
}
```

The field name is bound as a variable within the `where` clause.

Validation order: required fields -> type check -> fill defaults -> evaluate `where` constraints.

On failure: `"Protocol AuditFinding: field 'severity' constraint violated: severity in [...]"`.
