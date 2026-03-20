-- Memory: errata sheet. Non-obvious behaviors that trip up implementation or testing.
-- Only genuine traps belong here — things that look correct but silently misbehave.
-- Design decisions and syntax rules belong in LANGUAGE.md/AGENTS.md, not here.

# Gotchas

## Parser Traps

- **Multi-arg calls before `?` silently misbind.** `re.is_match pattern input ? { ... }` — `?` binds to `input`, not the full call. **Fix:** `(re.is_match pattern input) ? { ... }`.
- **`()` before `(param) { body }` breaks argument parsing.** `f name () (x) { body }` — parser doesn't see 4 args. The `()` followed by `(x)` confuses it. **Fix:** bind either to a variable first: `input = ()` then `f name input (x) { body }`.
- **`f arg () { body }` passes Unit then a separate block, not a closure.** `deadline.scope dl () { body }` — the `()` is consumed as Unit (second arg), and `{ body }` becomes a separate block expression. **Fix:** bind the closure first: `body_fn = () { ... }` then `f arg body_fn`.

## Fixed Parser Traps (Session 64)

- **`(expr) {record}` in application context no longer misparses.** Previously, `(to_str counter) {name: name}` was parsed as a 2-param function literal instead of a parenthesized expression + record. Fixed via `application_depth` tracking in the parser. In application context with 2+ bare-ident params and no strong signals, the parser rejects func-def when body is a record literal or an identifier not matching any param name.

- **Lambda body with `==`/`!=` in pipe chains breaks.** `list | keep (x) x.name == val | sort_by (.id)` — the lambda body extends through `| sort_by (.id)`, so `sort_by` receives `val` from the inner pipe, not the filtered list. **Fix:** use block syntax: `list | keep (x) { x.name == val } | sort_by (.id)`. Or break into two statements: `matched = list | keep (x) { x.name == val }` then `matched | sort_by (.id)`.

## Ternary + Record Ambiguity

- **`? {record}` is parsed as match, not ternary returning a record.** `expr ? {field: val  field2: val2} : other` — the `? {` triggers match-arm parsing, so `field:` is expected to be `pattern ->`. **Fix:** bind the record first: `result = {field: val  field2: val2}` then `expr ? result : other`.
- **`? { stmt }` where stmt uses `<-` or `:=` fails.** `cond ? { x <- [...] }` — the `? {` triggers match parsing, and reassignment/mutable bindings aren't match patterns. **Fix:** restructure functionally or reverse the condition: `not cond ? () : { x <- [...] }`. For conditional mutation, prefer functional style: `x = cond ? [...] : []`.
- **Multi-branch conditionals with `{ }` bodies all fail.** `done ? { emit "pass"; break val } : maxed ? { emit "max"; break val2 } : { emit "fix"; fix () }` — every `? {` triggers match parsing. **Fix:** use sequential single-arm ternary guards (no braces): `done ? break val` / `maxed ? break val2` / `fallthrough_code`. Or bind all branch bodies to variables before the conditional.

## Record Shorthand Ambiguity

- **Shorthand fields before spread or keyed fields misparse.** `{steps  task  step_count: steps | len}` — `steps task` is parsed as calling `steps` with `task` as an argument, not two shorthand fields. **Fix:** use explicit keys: `{steps: steps  task: task  step_count: steps | len}`.
- **`{..spread  shorthand}` misparses.** `{..entry  score}` is parsed as `{..(entry score)}` — calling `entry` with `score`. **Fix:** use explicit key: `{..entry  score: score}`.

## Operator Precedence

- **`??` binds looser than `|` pipe.** `entry.field ?? "" | lower | split " "` parses as `entry.field ?? ("" | lower | split " ")`, not `(entry.field ?? "") | lower | split " "`. If `entry.field` is not None, the pipe chain never runs. **Fix:** parenthesize: `(entry.field ?? "") | lower | split " "`.
- **`|` binds looser than `+`.** `x | len + y | len` parses as `x | (len + y) | len`, not `(x | len) + (y | len)`. **Fix:** parenthesize each pipe expression: `(x | len) + (y | len)`. Or bind to variables first: `a = x | len; b = y | len; a + b`.

## HOF + Enumerate

- **Tuple destructuring in HOF lambdas doesn't work.** `list | enumerate | map (i s) {id: i ...}` — `(i s)` is not parsed as two-parameter destructuring. `i` is undefined at call site. **Fix:** use single-param + field access: `map (pair) {id: pair.0  title: pair.1}`.

## Keyword Field Names

- **`par` is a keyword — can't use as record field via dot access.** `module.par` fails to parse because `par` is consumed as the `par { }` keyword. `std/flow` uses `flow.parallel` instead. Same applies to other keywords: `sel`, `match`, `if`, `use`, `emit`, `yield`, `refine`, `receive`, `Agent`, `Trait`, `Trait`, `MCP`.

## Uncatchable Errors

- **Trait conformance halts execution.** If an Agent declares a Trait but is missing a required method, it's a hard `LxError` — not `Value::Err`. `??` cannot catch it. This is by design but surprising if you expect defensive coding to work.

## Parens Are Not Blocks

- **`( )` is grouping, not a block scope.** `cond ? ( x = compute; use x )` fails with "expected RParen, found Assign" — parens don't create blocks, only `{ }` does. But `? { }` triggers match parsing (see above). **Fix:** restructure to avoid multi-statement branches. Use pipe chains: `cond ? (compute | use)`. Or bind before the ternary: `val = compute; cond ? val : other`. Or use sequential single-arm guards: `cond ? break val`.

## Sections Limitation

- **Sections don't support `==` or `!=`.** `filter (.status == "pass")` fails — the section parser can't handle comparison operators. **Fix:** use a lambda: `filter (r) r.status == "pass"`.

## Computed Tuple Access

- **`tuple.[0]` doesn't work.** Computed field access on tuples fails with "unsupported types Tuple / Int". **Fix:** use destructuring: `(a b) = tuple` then access `a` and `b` directly.

## time.format Argument Order

- **`time.format` takes format string first, time record second.** `time.format t "%H:%M:%S"` fails because `t` (Record) is in the format-string position. **Fix:** use pipe: `t | time.format "%H:%M:%S"`. This is pipe-last design — the data argument goes last.

## lx Package Traps

- **Self-recursive `+` exports need two-step pattern.** `+f = (n) { f (n-1) }` — `+` exports are excluded from forward declarations so builtins aren't shadowed. This means `+f` can't call itself directly. **Fix:** `f = (n) { f (n-1) }; +f = f`.
- **Adjacent string interpolation blocks fail.** `"{head}{tail}"` — the first `{head}` evaluates to a Str, then `{tail}` tries to call the Str as a function. **Fix:** use `++` concatenation: `head ++ tail`. Or use a single interpolation with the full expression.
- **String interpolation parses `{key: val}` as lx code.** `"Return JSON: {score: Int, issues: [Str]}"` — the `{score: Int}` is parsed as a record literal, causing parse errors (e.g., "unexpected token: Colon"). **Fix:** use backtick raw strings for text containing literal braces: `` `Return JSON: {score: Int}` ``.
- **Multi-line ternary chains don't parse.** `cond1 ? val1\n: cond2 ? val2\n: default` — the `:` on a new line is parsed as something else. **Fix:** keep the entire ternary chain on one line, or extract conditions into named bindings and nest: `cond1 ? val1 : (cond2 ? val2 : default)`.
- **`{}` is Unit, not empty Record.** `f x {}` passes Unit as the second arg, not an empty record. This breaks functions expecting a Record (e.g., `tasks.list store {}`). **Fix:** use `()` explicitly for Unit, or handle Unit in the function: `filter_rec == () ? defaults : filter_rec.field`.

## Module Paths

- **Only `yield` is accepted as keyword-named module path segment.** `use std/yield` works because `Yield` token is special-cased in `stmt_use.rs`. Other keyword names (e.g., `match`, `if`, `par`) are NOT accepted as module path segments. If a future module needs a keyword name, add it to the match arm in `parse_use_stmt`.

## pkg/ Import Paths

- **pkg/ uses subdirectory paths.** Packages are organized into `pkg/core/`, `pkg/ai/`, `pkg/data/`, `pkg/agents/`, `pkg/infra/`, `pkg/connectors/`, `pkg/kit/`. Import as `use pkg/core/prompt`, not `use pkg/prompt`. The old flat paths no longer resolve.

## Lambda Body Extent in Records

- **Lambda body extends through `| pipe` in record field values.** `{x: items | filter (f) f.severity == "critical" | len}` — the `| len` becomes part of the filter lambda body, not a pipe after filter. **Fix:** use block syntax: `{x: items | filter (f) { f.severity == "critical" } | len}`.

## Regex in String Pattern Lists

- **String patterns with `(` fail when used with `re.is_match`.** `guard.scan` uses `re.is_match` which treats patterns as regex. Patterns like `"eval("` crash with "unclosed group". **Fix:** escape regex metacharacters: `"eval\\("`.

## split + flat_map Trap

- **`split` returns the string itself (not a 1-element list) when delimiter is absent.** `"hello" | split "\n"` returns `"hello"` (Str), not `["hello"]` (List). This breaks `flat_map` chains: `items | flat_map (s) { s | split "\n" }` iterates over individual characters when no delimiter is found. **Fix:** break the chain into separate steps and avoid `flat_map` with `split` on optional delimiters.

## Incomplete Wiring

- **`uses` bindings are dropped.** The `Agent` keyword parses `uses` declarations but they are not stored on the Class value. MCP servers must be connected manually in method bodies or `init`.
