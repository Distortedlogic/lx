# Flow Testing Findings

Issues discovered while building satisfaction tests for the 14 flow specs. Each finding is a real lx language/runtime issue that should be fixed to make the language more usable for its target audience (agents writing lx programs).

11 deterministic suites (35 scenarios) + 3 live-only stubs.

## Parse-Level Issues

### 1. Record literal field values can't contain function calls (single-line only)

**Status:** Partially resolved (Session 52). Multiline records now support full expressions in field values. Single-line multi-field records with complex values still require temp bindings.

**What:** Inside `{key: expr}`, expressions like `get "field" record`, `audit.is_empty str`, or `list | join ", "` are misparsed on a single line. The parser terminates the field value too early — the string arg, next identifier, or pipe operator is interpreted as starting the next record field.

```lx
{type: get "type" entry ?? "unknown"}
{not_empty: !audit.is_empty output_str}
{gaps: coverage.gaps | join ", "}
{feedback: failed | map (.feedback) | join "; "}
```

All of these fail. The parser sees the value of `type:` as just `get`, then `"type"` starts a new (invalid) field.

**Workaround:** Extract complex expressions into temp bindings before the record literal.

```lx
t = get "type" entry ?? "unknown"
ne = !(audit.is_empty output_str)
g = coverage.gaps | join ", "
fb = failed | map (.feedback) | join "; "
{type: t  not_empty: ne  gaps: g  feedback: fb}
```

**Root cause:** Record field value parsing terminates at too low a binding power. It doesn't consume multi-arg function calls, pipe chains, or unary prefix operators applied to function calls.

**Frequency:** This was the single most common issue across all flow lib files. Hit in: `catalog.lx`, `guidance.lx`, `transcript.lx`, and every grader function in spec files.

**Should fix in parser:** Record field value expressions should parse with sufficient binding power to consume function application, pipe chains, and prefix operators. This is the highest-impact fix for flow authoring ergonomics.

### 2. Multi-arg function call before ternary `?` — body extent

**What:** `re.is_match (lower p) lowered ? {...}` — the parser applies `(lower p)` as one arg to `re.is_match`, producing a partial. Then `lowered ? {...}` becomes a separate ternary on `lowered`.

```lx
re.is_match (lower p) lowered ? { true -> x; false -> y }
```

The parser sees: `re.is_match` applied to `(lower p)` = partial. Then `lowered ? { ... }` = ternary on `lowered`. The second argument to `re.is_match` is never consumed.

**Workaround:** Always use temp bindings for multi-arg calls before ternary `?`.

```lx
pat = lower p
matched = re.is_match pat lowered
matched ? { true -> x; false -> y }
```

**Root cause:** The ternary `?` has lower precedence than function application. But the parser can't determine whether `lowered` is the second arg to the preceding function call or the scrutinee of `?` without arbitrary lookahead. In practice, this means any multi-arg function call immediately before `?` is ambiguous.

**Frequency:** Hit in every `guard.lx` scan function that uses `re.is_match`.

### 3. `!expr` inside record literals

**Same root cause as #1.** `{not_empty: !audit.is_empty output_str}` fails because the unary `!` plus function call isn't consumed as one field value.

### 4. List spread `..f x y` doesn't consume function application (use parens)

**What:** `[..scan_entries entries pats]` applies the spread to `scan_entries` (a Func) instead of to the call result `scan_entries entries pats`. The `..` operator only takes the immediately following atom expression.

```lx
[..scan_entries entries pats]
```

Fails with "spread requires List, got Func". The parser sees `..scan_entries` as the spread expression, and `entries pats` as separate list elements.

**Workaround:** Bind the call result to a temp variable first.

```lx
result = scan_entries entries pats
[..result]
```

**Root cause:** Spread expression parsing in list literals uses too low a binding power — it doesn't consume function application after the `..`.

**Frequency:** Hit in guard.lx `full_scan`.

### 5. ~~Record spread `{..f  key: val}` same issue as #4~~ FIXED (Session 52)

Record spread now allows function application: `{..mk ()  key: val}` works. The spread expression is parsed at bp=31 with collection_depth=0 and record_field_depth=1, so function calls are consumed but `Ident:Colon` still terminates at the next field boundary.

## Runtime-Level Issues

### 6. ~~Missing record field throws runtime error~~ FIXED (Session 52)

Record, Map, and Agent field access all return `None` on miss. `record.field ?? default` now works uniformly. Protocol validation also returns `Err` values instead of crashing.

### 7. `{}` is an empty block — use `{:}` for empty record

`{:}` is the canonical empty record literal. `{}` remains an empty block (Unit). This is by design — `{:}` is unambiguous and already implemented.

## Module System Issues

### 8. Module path resolver only handles single `..` parent

**What:** `use ../../examples/security_audit` fails because the path resolver only handles one level of `..`. The lexer tokenizes `../../` as `DotDot` + `/` + `DotDot` + `/`, but `resolve_module_path` only checks if `path[0] == ".."` for a single parent directory step.

```lx
use ../lib/guard          -- OK (one level up)
use ../../examples/foo    -- FAILS: "unexpected token: DotDot"
```

**Workaround:** Place files at the right directory depth. Test wrapper flows go in `flows/tests/` (not `flows/tests/subdir/`) so they only need single `..` imports.

**Should fix:** Support arbitrary depth `../../../path` in module path resolution. The path segments `[".", ".", ".", "lib", "guard"]` could each `..` step call `.parent()` on the base path.

### 9. `test.run` resolves flow paths relative to CWD

**What:** The `flow:` path in `test.spec` is resolved relative to the working directory (wherever `lx` was invoked from), not relative to the spec file. This means all flow paths must be CWD-relative.

```lx
-- Must use CWD-relative (assuming lx is run from project root):
test.spec "name" {flow: "flows/tests/security_audit_flow.lx" ...}

-- Can't use spec-relative:
test.spec "name" {flow: "./security_audit_flow.lx" ...}
```

**Workaround:** Always use project-root-relative paths. Convention: always run `lx` from the project root.

**Should fix:** `test.run` should resolve `flow:` relative to the calling file's `source_dir`, matching how `use ./module` works. The `source_dir` is available in the `RuntimeCtx` or from the span's source info.

### 10. Tuple-destructuring lambda params fail in `call_value` HOF chains

**What:** `| filter (tool, uses) uses | len >= 2 | map (tool, uses) {tool ...}` in `transcript.extract_patterns` fails with "undefined variable 'tool'" when the flow is invoked via `test.run`.

**Root cause:** Not fully diagnosed. Tuple auto-splatting in `call_value` may not properly bind lambda parameters when chained through multiple HOF stages (`group_by | entries | filter | map`) and called through the builtin `call_value` path rather than `Interpreter::apply_func`. The same pattern works via `lx run` but not via `test.run → invoke_flow → call_value`.

**Workaround:** Avoid `extract_patterns`/`extract_mistakes` in test flows. Use simpler inline logic: `good | map (e) { e.name }` instead of the complex group_by/entries/filter/map chain.

**Should fix:** Investigate whether `call_value`'s tuple-splatting handles all cases correctly, especially in deep HOF chains where the env chain crosses module boundaries.

## Patterns That Work

For future reference — patterns that successfully avoid the above issues:

### Safe record construction
```lx
-- Always extract complex values to temp bindings first:
t = get "type" entry ?? "unknown"
ne = !(audit.is_empty output_str)
cf = audit.references_task output_str rubric_str
{type: t  not_empty: ne  correct_findings: cf}
```

### Safe function calls before ternary
```lx
-- Always bind the call result before ?:
matched = re.is_match pattern text
matched ? { true -> x; false -> y }
```

### Safe lambda bodies in pipes
```lx
-- Always use braces for lambda bodies with operators:
items | filter (x) { x.severity == "critical" }
items | map (x) { x.name ++ ": " ++ x.value }
```

### Safe optional field access
```lx
-- Use get + ?? instead of .field + ??:
opts = get "patterns" config ?? default_patterns
name = get "name" record ?? "unnamed"
```

### Safe list/record spreads
```lx
-- Bind to temp before spreading:
injection = scan_entries entries patterns
loops = detect_loops entries threshold
[..injection ..loops]
```

### Safe function calls inside list literals
```lx
-- Function calls are NOT consumed inside [...]:
-- BAD:  [normalize entry]  → parsed as two elements [normalize, entry]
-- GOOD: bind first, then use in list:
normalized = normalize entry
[normalized]
```
