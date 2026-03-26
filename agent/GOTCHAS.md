-- Memory: errata sheet. Non-obvious behaviors that trip up implementation or testing.
-- Only genuine traps belong here — things that look correct but silently misbehave.
-- Design decisions and syntax rules belong in LANGUAGE.md/AGENTS.md, not here.

# Gotchas

## Parser Traps

- **Multi-arg calls before `?` silently misbind.** `re.is_match pattern input ? { ... }` — `?` binds to `input`, not the full call. **Fix:** `(re.is_match pattern input) ? { ... }`.
- **`()` before `(param) { body }` breaks argument parsing.** `f name () (x) { body }` — parser doesn't see 4 args. The `()` followed by `(x)` confuses it. **Fix:** bind either to a variable first: `input = ()` then `f name input (x) { body }`.
- **`f arg () { body }` passes Unit then a separate block, not a closure.** `deadline.scope dl () { body }` — the `()` is consumed as Unit (second arg), and `{ body }` becomes a separate block expression. **Fix:** bind the closure first: `body_fn = () { ... }` then `f arg body_fn`.
- **`$cmd ^` — the `^` is consumed by the shell, not lx.** `result = $sh -c "{cmd}" ^` — the `^` becomes part of the shell command string. **Fix:** separate the unwrap: `r = $sh -c "{cmd}"` then `val = r ^` or `val = (r ^).out`.
- **`$^{cmd}` throws `LxError::propagate` on non-zero exit, NOT `Value::Err`.** `($^{cmd}) ?? ""` does NOT catch the error because `??` only catches `Value::Err` and `Value::None`, not `LxError`. rg exits 1 on no matches, so `$^rg ...` throws uncatchably when there are no results. **Fix:** use `$sh -c "{cmd}"` (returns `Ok({out, err, code})` regardless of exit code), then check `.code` or just use `.out`.
- **`${cmd}` is multi-line shell block syntax, NOT `$` with variable interpolation.** `${cmd}` opens a multi-line shell block (like `${ cd /tmp; ls }`), it does not run the value of `cmd` as a shell command. **Fix:** use `$sh -c "{cmd}"` to run a dynamically constructed command string.

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

## type_of Returns "Func" Not "Fn"

- **`type_of some_function` returns `"Func"`, not `"Fn"`.** `assert (type_of f == "Fn") "is function"` fails. **Fix:** use `"Func"`: `assert (type_of f == "Func") "is function"`.

## assert Parses Greedily

- **`assert val "msg"` calls `val` with `"msg"` as argument.** `assert foo_done "test passed"` — when `foo_done` is `true`, this tries to call `true("test passed")`. **Fix:** always parenthesize the condition: `assert (foo_done == true) "msg"` or `assert (some_expr) "msg"`.

## not Is a Function, Not an Operator

- **`not x` is curried application, not negation.** `assert (not is_foo) "msg"` — `not is_foo` returns a partially applied function, not a bool. **Fix:** use `== false` comparison: `assert (is_foo == false) "msg"`. Or fully apply: `assert ((not is_foo) == true) "msg"`.

## Module Name Collisions

- **`use module/lib/log` shadows builtin `log`.** Importing a module named `log` replaces the builtin `log.info`/`log.warn` namespace. Inside that module, `log.info` won't work. **Fix:** alias the import: `use module/lib/log : ts_log`. Or use a different module name.

## No SCREAMING_CASE Constants

- **Uppercase identifiers are TypeName tokens.** `TARGET_GRADE = 93` fails — `TARGET` is lexed as a `TypeName` (constructor), not an identifier. lx has no `const` keyword or SCREAMING_CASE convention. **Fix:** use lowercase `target_grade = 93`. Immutable bindings (no `:=`) are already non-reassignable.

## Keyword Field Names

- **`par` is a keyword — can't use as record field via dot access.** `module.par` fails to parse because `par` is consumed as the `par { }` keyword. `std/flow` uses `flow.parallel` instead. Same applies to other keywords: `sel`, `match`, `if`, `use`, `emit`, `yield`, `refine`, `receive`, `Agent`, `Trait`, `Trait`, `MCP`.

## Uncatchable Errors

- **Trait conformance halts execution.** If an Agent declares a Trait but is missing a required method, it's a hard `LxError` — not `Value::Err`. `??` cannot catch it. This is by design but surprising if you expect defensive coding to work.

## Parens Are Not Blocks

- **`( )` is grouping, not a block scope.** `cond ? ( x = compute; use x )` fails with "expected RParen, found Assign" — parens don't create blocks, only `{ }` does. But `? { }` triggers match parsing (see above). **Fix:** restructure to avoid multi-statement branches. Use pipe chains: `cond ? (compute | use)`. Or bind before the ternary: `val = compute; cond ? val : other`. Or use sequential single-arm guards: `cond ? break val`.

## find/first/last Return Some, Not the Value

- **`find`, `first`, and `last` return `Some(val)` or `None`, not the value directly.** `list | find pred | (.field)` fails — `.field` is called on `Some(record)`. `list | last | trim` fails — `trim` is called on `Some(str)`. **Fix:** unwrap with `??`: `(list | find pred) ?? default`. Parenthesize the expression before `??` because `??` binds looser than `|`.

## Computed Tuple Access

- **`tuple.[0]` doesn't work.** Computed field access on tuples fails with "unsupported types Tuple / Int". **Fix:** use destructuring: `(a b) = tuple` then access `a` and `b` directly.

## time.format Argument Order

- **`time.format` takes format string first, time record second.** `time.format t "%H:%M:%S"` fails because `t` (Record) is in the format-string position. **Fix:** use pipe: `t | time.format "%H:%M:%S"`. This is pipe-last design — the data argument goes last.

## lx Package Traps

- **Self-recursive `+` exports need two-step pattern.** `+f = (n) { f (n-1) }` — `+` exports are excluded from forward declarations so builtins aren't shadowed. This means `+f` can't call itself directly. **Fix:** `f = (n) { f (n-1) }; +f = f`.
- **Closures inside `+` functions can't capture non-exported module bindings.** `helper = (x) x * 2` then `+main = () { list | each (x) { helper x } }` — `helper` is silently `None` inside the `each` closure, and the `each` body doesn't execute. **Fix:** inline the helper body, or pass the helper as a parameter, or use the two-step export: `helper = ...; +helper = helper`.
- **Adjacent string interpolation blocks fail.** `"{head}{tail}"` — the first `{head}` evaluates to a Str, then `{tail}` tries to call the Str as a function. **Fix:** use `++` concatenation: `head ++ tail`. Or use a single interpolation with the full expression.
- **String interpolation parses `{key: val}` as lx code.** `"Return JSON: {score: Int, issues: [Str]}"` — the `{score: Int}` is parsed as a record literal, causing parse errors (e.g., "unexpected token: Colon"). **Fix:** use backtick raw strings for text containing literal braces: `` `Return JSON: {score: Int}` ``.
- **Multi-line ternary chains don't parse.** `cond1 ? val1\n: cond2 ? val2\n: default` — the `:` on a new line is parsed as something else. **Fix:** keep the entire ternary chain on one line, or extract conditions into named bindings and nest: `cond1 ? val1 : (cond2 ? val2 : default)`.
- **`{}` is Unit, not empty Record.** `f x {}` passes Unit as the second arg, not an empty record. This breaks functions expecting a Record (e.g., `tasks.list store {}`). **Fix:** use `()` explicitly for Unit, or handle Unit in the function: `filter_rec == () ? defaults : filter_rec.field`.

## Module Paths

- **Only `yield` is accepted as keyword-named module path segment.** `use std/yield` works because `Yield` token is special-cased in `stmt_use.rs`. Other keyword names (e.g., `match`, `if`, `par`) are NOT accepted as module path segments. If a future module needs a keyword name, add it to the match arm in `parse_use_stmt`.

## pkg/ Import Paths

- **pkg/ uses subdirectory paths.** Current layout: `pkg/agent/`, `pkg/git/`, `pkg/guard/`, `pkg/schema/`, `pkg/store/`, `pkg/workflow/`, plus `pkg/log.lx` at root. Import as `use pkg/agent/auditor`, `use pkg/log`, etc.

## Regex in String Pattern Lists

- **String patterns with `(` fail when used with `re.is_match`.** `guard.scan` uses `re.is_match` which treats patterns as regex. Patterns like `"eval("` crash with "unclosed group". **Fix:** escape regex metacharacters: `"eval\\("`.

## split + flat_map Trap

- **`split` returns the string itself (not a 1-element list) when delimiter is absent.** `"hello" | split "\n"` returns `"hello"` (Str), not `["hello"]` (List). This breaks `flat_map` chains: `items | flat_map (s) { s | split "\n" }` iterates over individual characters when no delimiter is found. **Fix:** break the chain into separate steps and avoid `flat_map` with `split` on optional delimiters.

## Cron Traps

- **`cron.every` takes milliseconds, not seconds.** `cron.every 60 fn` fires every 60ms (16 times/second), not every minute. **Fix:** `cron.every 60000 fn` for once per minute.
- **Cron closures capture scope at definition time.** `cron.every 1000 () { x }` where `x` is defined AFTER the cron call → "undefined variable" error on the background thread. **Fix:** define all variables the closure needs before the `cron.every` call.

## grader / AiBackend Traps

- **`grader` used `max_turns: 1` which caused empty responses.** Claude Code uses tools by default. With `max_turns: 1`, it uses a tool on turn 1, hits the limit on turn 2, and returns empty `result` text. **Fixed:** grader Agent now declares empty `tools = () { [] }` (the trait default) and uses `json_schema` for structured output.
- **Agents with no tools leave `tools` at the default empty list.** The Agent trait default `tools = () { [] }` means no tools. The backend sees an empty tools vec and doesn't pass `--allowedTools`.
- **`--json-schema` puts output in `structured_output`, not `result`.** When `--json-schema` is used, the `result` field in Claude CLI JSON output is empty string. The structured data is in `structured_output` field. `parse_ai_response` must check `structured_output` first.

## md.sections Limitations

- **`md.sections` drops bullet list and code block content.** Sections with only `- item` bullets or fenced code return empty `content`. Only plain text paragraphs between headings are captured. **Fix:** use raw string splitting (`split "\n# Title\n"` + `take_while`) instead of `md.sections` for sections that contain bullet lists.
- **`md.sections` splits sub-headings into separate sections.** `# Parent` with `### Child` inside — the child becomes its own section, not content of the parent. `Parent` section has empty content. **Fix:** use `md.sections` for the sub-headings directly: `sections | filter (s) s.level == 3`.

## Incomplete Wiring

- **`uses` bindings are dropped.** The `Agent` keyword parses `uses` declarations but they are not stored on the Class value. MCP servers must be connected manually in method bodies or `init`.

## Fixed (kept for reference)

- **`(expr) {record}` in application context no longer misparses.** Fixed via `application_depth` tracking in the parser.
- **Lambda body with `==`/`!=` in pipe chains** — use block syntax `{ }` for lambda bodies in pipe chains. Block lambdas now correctly stop at `}` (commit 1b1f823).
- **Lambda body extends through `| pipe` in record field values** — block syntax `{ }` now terminates correctly (commit 1b1f823).
- **Sections now support `==`, `!=`, `<`, `>`, `<=`, `>=`.** `filter (.status == "pass")` works. `section_op` in `expr_pratt.rs` includes all comparison operators.
- **`Agent` keyword now supports field declarations with `:`.** `Agent Foo = { x: 0 }` works — the keyword parser uses `class_body()` which handles both `:` fields and `=` methods.
- **`Agent` keyword injects trait defaults.** `self.think`, `self.think_with` etc. are available on any `+Agent` declaration via trait desugaring.
