-- Memory: errata sheet. Non-obvious behaviors that trip up implementation or testing.
-- Only genuine traps belong here — things that look correct but silently misbehave.
-- Design decisions and syntax rules belong in LANGUAGE.md/AGENTS.md, not here.

# Gotchas

## Parser Traps

- **Multi-arg calls before `?` silently misbind.** `re.is_match pattern input ? { ... }` — `?` binds to `input`, not the full call. `?` is postfix precedence 3, application is 31 (tighter). **Fix:** `(re.is_match pattern input) ? { ... }`.
- **`()` before `(param) { body }` breaks argument parsing.** `f name () (x) { body }` — `()` becomes Unit, then `(x) { body }` is parsed as a separate func_def. **Fix:** bind to a variable first: `input = ()` then `f name input (x) { body }`.
- **`f arg () { body }` passes Unit then a separate block, not a closure.** The `()` is consumed as Unit (second arg), and `{ body }` becomes a separate block expression. **Fix:** bind the closure first: `body_fn = () { ... }` then `f arg body_fn`.

## Multi-line Ternary

- **Multi-line ternary chains don't parse.** `cond1 ? val1\n: cond2 ? val2\n: default` — the `:` on a new line is parsed as a new statement, not the ternary else branch. **Fix:** keep on one line or nest with parens: `cond1 ? val1 : (cond2 ? val2 : default)`.

## Record Shorthand Ambiguity

- **Shorthand fields before spread or keyed fields misparse.** `{steps  task  step_count: steps | len}` — `steps task` is parsed as calling `steps` with `task` as an argument, not two shorthand fields. **Fix:** use explicit keys: `{steps: steps  task: task  step_count: steps | len}`.
- **`{..spread  shorthand}` misparses.** `{..entry  score}` is parsed as `{..(entry score)}` — calling `entry` with `score`. **Fix:** use explicit key: `{..entry  score: score}`.

## Operator Precedence

- **`??` binds looser than `|` pipe.** Precedence: `+` (25) > `|` (19) > `==` (17) > `&&` (15) > `||` (13) > `??` (11). So `entry.field ?? "" | lower | split " "` parses as `entry.field ?? ("" | lower | split " ")`. **Fix:** parenthesize: `(entry.field ?? "") | lower | split " "`.
- **`|` binds looser than `+`.** `x | len + y | len` parses as `x | (len + y) | len`. **Fix:** parenthesize each pipe expression: `(x | len) + (y | len)`.

## HOF + Enumerate

- **Tuple destructuring in HOF lambdas doesn't work.** `list | enumerate | map (i s) {id: i ...}` — `param_parser` only supports single ident params (typed or underscore), not tuple patterns. **Fix:** use single-param + field access: `map (pair) {id: pair.0  title: pair.1}`.

## type_of Returns "Func" Not "Fn"

- **`type_of some_function` returns `"Func"`, not `"Fn"`.** `bi_type_of` explicitly returns "Func" for `BuiltinFunc` and `MultiFunc`. **Fix:** use `"Func"`: `assert (type_of f == "Func")`.

## assert Parses Greedily

- **`assert val "msg"` calls `val` with `"msg"` as argument.** `assert foo_done "test passed"` — when `foo_done` is `true`, this tries to call `true("test passed")`. **Fix:** always parenthesize the condition: `assert (foo_done == true) "msg"`.

## not Is a Function, Not an Operator

- **`not x` is curried application, not negation.** `assert (not is_foo) "msg"` — `not is_foo` returns a partially applied function, not a bool. **Fix:** use `== false` comparison: `assert (is_foo == false) "msg"`.

## Module Name Collisions

- **`use module/lib/log` shadows builtin `log`.** `UseKind::Whole` binds the module name to a record containing all exports, shadowing the builtin `log` namespace. **Fix:** alias the import: `use module/lib/log : ts_log`.

## No SCREAMING_CASE Constants

- **Uppercase identifiers are TypeName tokens.** TypeName regex is `[A-Z][a-zA-Z0-9]*` — no underscores. `TARGET_GRADE` lexes as TypeName `TARGET` + Ident `_GRADE`. **Fix:** use lowercase `target_grade = 93`.

## Keyword Field Names

- **`par` is a keyword — can't use as record field via dot access.** `dot_rhs` accepts `Ident` tokens; keywords are separate token types. Same applies to: `sel`, `match`, `if`, `use`, `emit`, `yield`, `refine`, `receive`, `Agent`, `Trait`, `MCP`.

## Parens Are Not Blocks

- **`( )` is grouping, not a block scope.** `cond ? ( x = compute; use x )` fails — parens don't create blocks, only `{ }` does. But `? { }` triggers match parsing (see above). **Fix:** bind before the ternary: `val = compute; cond ? val : other`.

## find/first/last Return Some, Not the Value

- **`find`, `first`, and `last` return `Some(val)` or `None`, not the value directly.** `list | find pred | (.field)` fails — `.field` is called on `Some(record)`. **Fix:** unwrap with `??`: `(list | find pred) ?? default`. Parenthesize before `??` because `??` binds looser than `|`.

## Computed Tuple Access

- **`tuple.[0]` doesn't work.** Computed field access on tuples fails. **Fix:** use destructuring: `(a b) = tuple` then access `a` and `b` directly.

## time.format Argument Order

- **`time.format` takes format string first, time record second.** Pipe-last design — the data argument goes last. **Fix:** use pipe: `t | time.format "%H:%M:%S"`.

## lx Package Traps

- **Self-recursive `+` exports need two-step pattern.** `+f = (n) { f (n-1) }` — `+` exports are excluded from forward declarations so builtins aren't shadowed. **Fix:** `f = (n) { f (n-1) }; +f = f`.
- **Closures inside `+` functions can't capture non-exported module bindings.** `helper = (x) x * 2` then `+main = () { list | each (x) { helper x } }` — `helper` is silently `None` inside the closure. **Fix:** use the two-step export: `helper = ...; +helper = helper`.
- **Adjacent string interpolation blocks fail.** `"{head}{tail}"` — the first `{head}` evaluates to a Str, then `{tail}` tries to call the Str as a function. **Fix:** use `++` concatenation: `head ++ tail`.
- **String interpolation parses `{key: val}` as lx code.** `"Return JSON: {score: Int, issues: [Str]}"` — the `{score: Int}` is parsed as a record literal. **Fix:** use backtick raw strings: `` `Return JSON: {score: Int}` ``.
- **`{}` is Unit, not empty Record.** Empty block `{}` evaluates to Unit. Empty record requires `{:}` syntax. **Fix:** use `()` explicitly for Unit.

## Module Paths

- **Only `yield` is accepted as keyword-named module path segment.** `use std/yield` works because `Yield` token is special-cased in `stmt_use.rs`. Other keywords are NOT accepted.

## pkg/ Import Paths

- **pkg/ uses subdirectory paths.** Current layout: `pkg/agent/`, `pkg/git/`, `pkg/guard/`, `pkg/schema/`, `pkg/store/`, `pkg/workflow/`, plus `pkg/log.lx` at root. Import as `use pkg/agent/auditor`, `use pkg/log`, etc.

## Cron Traps

- **`cron.every` takes milliseconds, not seconds.** `bi_every` uses `Duration::from_millis(ms)`. **Fix:** `cron.every 60000 fn` for once per minute.
- **Cron closures capture scope at definition time.** The context is `Arc::clone`'d before `tokio::task::spawn_blocking`. **Fix:** define all variables the closure needs before the `cron.every` call.

## grader / LlmBackend Traps

- **Agents with no tools leave `tools` at the default empty list.** The Agent trait default `tools = () { [] }` means no tools. The backend sees an empty tools vec and doesn't pass `--allowedTools`.
- **`--json-schema` puts output in `structured_output`, not `result`.** When `--json-schema` is used, the `result` field in Claude CLI JSON output is empty string. The structured data is in `structured_output` field.

## md.sections Limitations

- **`md.sections` drops bullet list and code block content.** Only extracts the `"text"` field from each node. **Fix:** use raw string splitting (`split "\n# Title\n"` + `take_while`) for sections with bullet lists.
- **`md.sections` splits sub-headings into separate sections.** Every heading starts a new section regardless of level. **Fix:** filter by level: `sections | filter (s) s.level == 3`.

## Incomplete Wiring

- **`uses` bindings are dropped.** `KeywordDeclData` has no `uses` field. The desugar ignores them. MCP servers must be connected manually.

## Fixed (kept for reference)

- **`(expr) {record}` in application context no longer misparses.** Fixed via `application_depth` tracking in the parser.
- **Lambda body with `==`/`!=` in pipe chains** — block lambdas now correctly stop at `}` (commit 1b1f823).
- **Lambda body extends through `| pipe` in record field values** — block syntax `{ }` now terminates correctly (commit 1b1f823).
- **Sections now support `==`, `!=`, `<`, `>`, `<=`, `>=`.** `section_op` in `expr_pratt.rs` includes all comparison operators.
- **`Agent` keyword now supports field declarations with `:`.** The keyword parser uses `class_body()` which handles both `:` fields and `=` methods.
- **`Agent` keyword injects trait defaults.** `self.think`, `self.think_with` etc. are available via trait desugaring.
- **`grader` max_turns trap** — grader Agent now declares empty `tools = () { [] }` (trait default) and uses `json_schema` for structured output.
- **`split` always returns a List.** `s.split(sep)` collects into `Vec<LxVal>`. When delimiter is absent, returns a 1-element list, not the string itself.
- **`? {block}` works for ternary blocks.** Chumsky backtracks from match-arm parsing when no `->` is found. `true ? { x <- 5 }`, `true ? { a = 1; b = 2; a + b }`, and multi-branch `? { }` all work. Verified by testing.
- **`? {record}` works inline.** `true ? {name: "hello"} : {name: "world"}` parses correctly — no need to bind records first.
- **Trait errors are consistently catchable.** `apply_trait_fields` and `apply_trait_union` both return `Ok(LxVal::err_str(...))`. `try_match_variant` uses `LxError` internally but `apply_trait_union` catches it via `.is_ok()` at line 66 — no `LxError` escapes to user code.
