# Flow Testing Findings

Issues discovered while building satisfaction tests for the 14 flow specs. Each finding is a real lx language/runtime issue that should be fixed to make the language more usable for its target audience (agents writing lx programs).

Updated after completing all 12 tasks of FLOW_SATISFACTION_TESTS.md. 11 deterministic suites (35 scenarios) + 3 live-only stubs.

## Parse-Level Issues

### 1. `Protocol +Name` syntax was unsupported (FIXED)

**What:** `Protocol +GuardResult = {pass: Bool}` failed with "expected type name, found Plus". The `+` export marker was only recognized at line-start (where the lexer emits `Export` token). Mid-line `+` after a keyword emitted `Plus`.

**Fix:** Added `Plus` token check in `parse_protocol`, `parse_trait_decl`, `parse_agent_decl`, `parse_mcp_decl` in their respective `stmt_*.rs` files — all now accept `+` between keyword and name.

**Impact:** Unblocked all 15 flow lib files from parsing.

**Files changed:** `crates/lx/src/parser/stmt_protocol.rs`, `stmt_agent.rs`, `stmt_mcp.rs`

### 2. Record literal field values can't contain function calls

**What:** Inside `{key: expr}`, expressions like `get "field" record`, `audit.is_empty str`, or `list | join ", "` are misparsed. The parser terminates the field value too early — the string arg, next identifier, or pipe operator is interpreted as starting the next record field.

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

### 3. Multi-arg function call before ternary `?` — body extent

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

### 4. `!expr` inside record literals

**Same root cause as #2.** `{not_empty: !audit.is_empty output_str}` fails because the unary `!` plus function call isn't consumed as one field value.

### 5. `refine` initial expression consumed config block (FIXED)

**What:** In `grading.lx`, `refine draft { grade: ... }` — `parse_expr(0)` for the initial expression consumed the `{...}` config block as a function-call argument to `draft`.

**Fix (by subagent):** Changed `parse_expr(0)` to `parse_expr(32)` in `crates/lx/src/parser/refine.rs` line 8. This prevents juxtaposition from consuming the config block.

**Files changed:** `crates/lx/src/parser/refine.rs`

### 6. List spread `..f x y` doesn't consume function application

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

### 7. Record spread `{..f  key: val}` same issue as #6

**What:** `{..f  agent_id: e.name}` inside a `map` callback — `f` is the lambda parameter but `..f` applies spread to the raw identifier (which is a Func in the wrong context). Same pattern as #6 but for record spreads.

**Workaround:** Manually copy fields instead of using spread.

```lx
{severity: finding.severity  type: finding.type  evidence: finding.evidence  agent_id: aid}
```

## Runtime-Level Issues

### 8. Missing record field throws runtime error, not Err/None

**What:** `record.missing_field` throws `LxError` (crashes the program), not `Err` or `None` (catchable). This means `record.field ?? default` doesn't work for optional fields — the `??` never fires because the error isn't a value-level error.

```lx
opts = input.opts ?? {}
```

Crashes with "field 'opts' not found" even though `??` is supposed to coalesce errors.

**Workaround:** Use `get "field" record ?? default`.

```lx
opts = get "opts" input ?? {empty: true}
```

**Root cause:** Field access on records is implemented as a hard runtime error (`LxError`), not as a `None` or `Err` value. The `??` operator only catches `Err` and `None` values — it doesn't catch `LxError`.

**Frequency:** Hit in every flow lib that accepts optional config records: `guard.lx` `full_scan`, `defense_layers_flow.lx`, all `security_audit_flow.lx` variants.

**Should fix:** This is the second highest-impact fix. Options:
1. Make `.field` on a Record return `None` when field is absent (breaking change but consistent with `get`)
2. Make `??` catch `LxError` in addition to `Err`/`None`
3. Add `record.?field` syntax that returns `None` on missing field

### 9. `{}` is an empty block (Unit), not an empty record

**What:** `{}` evaluates to `Unit` (empty block). `%{}` is an empty Map. There's no literal syntax for an empty Record.

```lx
x = {}     -- Unit
x = %{}    -- Map (not Record)
x = {_: 0} -- parse error: Underscore token not valid as record key
```

**Workaround:** Use `{empty: true}` or `{x: 0}` as a dummy record with a throwaway field.

**Frequency:** Hit whenever a function expects optional `opts` and the caller wants to pass empty defaults.

**Should fix:** Add empty record literal syntax (e.g. `{:}`) or make `{}` context-sensitive (block when followed by statements, record when assigned to a binding expecting a Record).

### 10. Lambda closures from imported modules break in `invoke_flow`

**What:** This is the most serious runtime issue found. Functions from imported modules work correctly when called via `lx run` (normal `use` import path) but fail when the flow is invoked via `test.run` → `invoke_flow` → fresh `Interpreter::new` → `exec` → `call_value`.

Specifically, lambda parameters inside closures from imported modules resolve to wrong values (the variable name resolves to a Func instead of the lambda argument value).

```lx
-- guard.lx exports scan_entries which uses:
entries | flat_map (e) {
  input_text = to_str e.input    -- <-- e.input fails: "field access on Func, not Record"
  ...
}

-- Works via: lx run file.lx (with use ../lib/guard at top)
-- Fails via: test.run spec with flow: "flows/tests/defense_layers_flow.lx"
--            (where the flow does use ../lib/guard internally)
```

**Reproduction steps:**
1. Create a module (A.lx) that exports a function using `flat_map (e) { e.field }` and calls another function in the same module
2. Create a flow (B.lx) that imports A.lx via `use` and calls the exported function
3. Run B.lx directly with `lx run B.lx` — works
4. Create a spec that uses `test.run` with `flow: "B.lx"` — fails with "field access on Func, not Record"

**Root cause:** Not fully diagnosed. `invoke_flow` in `test_invoke.rs` creates a fresh `Interpreter::new`, execs the program (which loads modules into its own module cache), then calls exported functions via `call_value`. The closure env chain from the inner interpreter's module system doesn't correctly resolve lambda parameters when the closure is executed through `call_value` from the outer context.

Key observation: The issue only manifests when:
- A module (guard.lx) is imported by the flow under test
- The module's exported function uses lambdas in higher-order functions (flat_map, filter, map)
- The module's function internally calls another function from the same module

Simple module functions with lambdas work fine. The issue appears related to the interaction between module-internal function references and lambda parameter binding across interpreter boundaries.

**Workaround:** All test flows must be self-contained — inline all logic instead of importing flow lib modules. This is the primary constraint on the current testing approach.

**Impact:** This is the critical blocker for testing the actual flow code. Test flows can only test replicated patterns, not the real library code.

**Should fix (priority: critical):** Options:
1. Share module cache between outer and inner interpreter in `invoke_flow`
2. Use the existing module import system (`use ./flow`) instead of creating a fresh interpreter
3. Debug the specific env chain issue — the inner interpreter's module cache may create closures whose parent envs don't properly chain to the builtins when executed from `call_value`

## Module System Issues

### 11. Module path resolver only handles single `..` parent

**What:** `use ../../examples/security_audit` fails because the path resolver only handles one level of `..`. The lexer tokenizes `../../` as `DotDot` + `/` + `DotDot` + `/`, but `resolve_module_path` only checks if `path[0] == ".."` for a single parent directory step.

```lx
use ../lib/guard          -- OK (one level up)
use ../../examples/foo    -- FAILS: "unexpected token: DotDot"
```

**Workaround:** Place files at the right directory depth. Test wrapper flows go in `flows/tests/` (not `flows/tests/subdir/`) so they only need single `..` imports.

**Should fix:** Support arbitrary depth `../../../path` in module path resolution. The path segments `[".", ".", ".", "lib", "guard"]` could each `..` step call `.parent()` on the base path.

### 12. `test.run` resolves flow paths relative to CWD

**What:** The `flow:` path in `test.spec` is resolved relative to the working directory (wherever `lx` was invoked from), not relative to the spec file. This means all flow paths must be CWD-relative.

```lx
-- Must use CWD-relative (assuming lx is run from project root):
test.spec "name" {flow: "flows/tests/security_audit_flow.lx" ...}

-- Can't use spec-relative:
test.spec "name" {flow: "./security_audit_flow.lx" ...}
```

**Workaround:** Always use project-root-relative paths. Convention: always run `lx` from the project root.

**Should fix:** `test.run` should resolve `flow:` relative to the calling file's `source_dir`, matching how `use ./module` works. The `source_dir` is available in the `RuntimeCtx` or from the span's source info.

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

### Self-contained test flows (workaround for finding #10)
```lx
-- Finding #10 was MISDIAGNOSED. The actual cause was finding #13 below.
-- Test flows CAN import flow lib modules — the invoke_flow closure chain works.
-- The real issue was [f x] inside list literals (see finding #13).
```

### 13. Function calls inside list literals silently produce wrong results (FIXED in transcript.lx)

**What:** `[normalize entry]` inside `transcript.lx:parse` was parsed as a two-element list `[normalize, entry]` instead of `[(normalize entry)]`. This caused `flat_map` to spread a `Func` value into the entry list, which later failed with "field access on Func, not Record".

**Root cause:** `is_application_candidate` in `parser/helpers.rs` disables juxtaposition (function application) when `collection_depth > 0` — only `TypeConstructor` calls are allowed inside list/record/map literals. This is by design so `[a b c]` means three elements, but it makes `[f x]` silently wrong (two elements instead of one).

**Fix applied:** Changed `transcript.lx` to extract the call to a temp binding before the list literal.

**Impact:** This was the REAL cause of Finding #10's symptoms. With this fix, `invoke_flow` works correctly with imported modules. Test flows can now import `flows/lib/` modules directly.

**Should fix in parser:** Consider a lint or warning when an `Ident` followed by another expression appears inside a list literal, since this is almost always a function call that the author expects to be applied.

### 14. Tuple-destructuring lambda params fail in `call_value` HOF chains

**What:** `| filter (tool, uses) uses | len >= 2 | map (tool, uses) {tool ...}` in `transcript.extract_patterns` fails with "undefined variable 'tool'" when the flow is invoked via `test.run`.

**Root cause:** Not fully diagnosed. Tuple auto-splatting in `call_value` may not properly bind lambda parameters when chained through multiple HOF stages (`group_by | entries | filter | map`) and called through the builtin `call_value` path rather than `Interpreter::apply_func`. The same pattern works via `lx run` but not via `test.run → invoke_flow → call_value`.

**Workaround:** Avoid `extract_patterns`/`extract_mistakes` in test flows. Use simpler inline logic: `good | map (e) { e.name }` instead of the complex group_by/entries/filter/map chain.

**Should fix:** Investigate whether `call_value`'s tuple-splatting handles all cases correctly, especially in deep HOF chains where the env chain crosses module boundaries.

### 15. `trace.record` drops Int scores — should_stop never triggers

**What:** `trace.record` uses `v.as_float()` to extract the `score` field, but `audit::build_eval_result` returns `Value::Int` for scores. Since `as_float()` only matches `Value::Float`, all progress record scores are stored as `None`. This means `progress_scores()` always returns an empty vec, `should_stop` always returns false, and verify loops run indefinitely.

**Root cause:** `trace.rs:bi_record` line 143: `fields.get("score").and_then(|v| v.as_float())` — `as_float()` doesn't coerce Int→Float. The audit module's `build_eval_result` uses `Value::Int(BigInt::from(score))` for the score field.

**Workaround:** Fixed in-place — `trace.rs:bi_record` now matches both `Value::Float` and `Value::Int` (via `num_traits::ToPrimitive::to_f64()`).

### 16. `workgen/main.lx` record shorthand `{path: out_path  name}` fails

**What:** In `workgen/main.lx:89`, `Ok {path: out_path  name}` crashes with "cannot call Str, not a function" because lx doesn't support shorthand record field syntax. `out_path  name` is parsed as a function call `out_path(name)`.

**Root cause:** lx record literals require `key: value` pairs for every field. There's no shorthand where `{name}` means `{name: name}`.

**Workaround:** Fixed in-place — changed to `{path: out_path  name: name}`.
