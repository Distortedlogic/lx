# lx Development Log

Self-continuity doc. Read this first when picking up lx work cold.

## Readiness Criteria

The language is ready to start implementation (Phase 1 code in `crates/lx/`) when ALL of:

1. **No spec contradictions** — every example in spec/ must actually work under the stated rules. Run a mental execution of each example against the grammar and runtime semantics. Session 5 found and fixed: `validate`/`check_positive` in 09_errors.lx used bare `Err` in non-final statement position as if it were early return — but blocks don't short-circuit (Err values are discarded). Resolved by adding "implicit Err early return" rule to spec: in `-> T ^ E` annotated functions, bare Err in statement position returns immediately. **Status: PASS** — all known contradictions resolved.

2. **Every construct has one unambiguous parse** — the grammar in grammar.md must handle every disambiguation case. Known resolved: `(` has 4 meanings (section/function/tuple/grouping), `?` has 3 modes, `? {` always starts match, function body extent in pipe chains documented. Remaining edge case: deeply nested `{expr}` interpolation inside shell mode inside string interpolation — covered by lexer mode stack design.

3. **Impl docs cover every component** — each box in the data flow (`source → lexer → parser → checker → interpreter`) has its own design doc with enough detail to write Rust without design questions. **Status: PASS** — all 11 docs written (Session 3 added impl-error.md for the error/diagnostic system). Value::Opaque added for Handle/Duration.

4. **Suite covers at least phases 1–4** — the first 4 phases are the foundation (lexer, parser, functions, collections, iteration). **Status: PASS** — Suite files exist (01–10, 12, 13, 16; ~820 assertions) covering phases 1–8 plus edge cases. Session 5 added 16_edge_cases.lx (~100 assertions) and 11_modules/ (multi-file import tests with 3 .lx files, ~30 assertions).

5. **No unresolved questions that block Phase 1** — check open-questions.md. **Status: PASS** — all v0.1 and post-v0.1 questions are resolved. v2 items are non-blocking.

## What Exists

- **spec/** (20 files): Complete language specification. grammar.md has full EBNF and corrected precedence table. stdlib-data.md covers data ecosystem (df, db, num, ml, plot). Implicit Err early return rule added in Session 5.
- **impl/** (11 files): Architecture, 11-phase plan, and per-component design docs (lexer, parser, AST, checker, interpreter, builtins, formatter, stdlib, error). TypeDef AST node added in Session 5.
- **suite/** (15 .lx files in 14 files + 3 module files + README): Golden test files for phases 1–8 plus edge cases (~820 assertions).

## Key Design Decisions to Remember

These are the non-obvious choices that are easy to forget and would cause confusion mid-implementation:

- **Pipe has HIGHER precedence than comparison**. `data | sort | len > 5` parses as `((data | sort) | len) > 5`. This was changed in Session 2 — the original table had pipe below comparison, which broke every `assert (pipeline == expected)` test. The new table puts pipe at position 8, comparison at 9.
- **`^` and `??` are LOWER precedence than `|`**. `url | fetch ^` = `(url | fetch) ^`. This is counterintuitive but essential. Because `??` is below comparison, `Ok 42 ?? 0 == 42` parses as `Ok 42 ?? (0 == 42)` — always wrap `??` expressions in parens when comparing: `(Ok 42 ?? 0) == 42`.
- **Function body extent in pipe chains** — `map (x) x * 2 | sum` gives map a function whose body is `x * 2 | sum`. Use blocks for multi-expression bodies: `map (x) { x * 2 } | sum`. Sections (`(* 2)`) have no ambiguity.
- **Division by zero is a panic**, not `Err`. Same category as `assert` and out-of-bounds indexing. Use `math.safe_div` for recoverable.
- **Tuple auto-spread**: function with N params receiving one N-tuple → auto-destructure. This is what makes `enumerate | each (i x) body` work.
- **`none?` is 2-arg only** (collection predicate). No 1-arg Maybe form — use `!some?` instead. Resolves currying ambiguity.
- **`pmap_n limit f xs`** exists in v1 (not deferred to v2).
- **Implicit Err early return in Result-annotated functions**. In `-> T ^ E` functions, bare `Err e` in statement position returns immediately. `^` handles errors from called functions, implicit Err return handles locally-constructed errors. No `return` keyword needed.
- **`$echo "hello {name}"`** — the `"` are shell quotes, not lx string delimiters. `{name}` is lx interpolation inside shell mode. The lexer handles this via mode stack.
- **`+` at column 0** is export. `+` anywhere else is addition. Lexer tracks column.
- **`<>` composition is left-to-right**: `f <> g` = `(x) f x | g` = `(x) g(f(x))`. To negate a predicate, write `pred <> not` (apply pred, then negate), NOT `not <> pred`. This has caused bugs in 3 separate places across Sessions 2-3. Read it as "apply f, then pipe result to g."
- **Record equality is order-independent**. `{x: 1 y: 2} == {y: 2 x: 1}` is `true`. Records compare by field names and values, not insertion order. This matters for the `IndexMap`-based implementation — equality must sort or ignore key order.
- **`log` is a record, not a function**. `log.info "msg"`, `log.warn "msg"`, `log.err "msg"`, `log.debug "msg"`. No bare `log "msg"` shorthand. This resolves a Session 1-3 ambiguity where `log` was used both ways.
- **Application requires callable left-side**. `f x` only parses as application when `f` is Ident, TypeName, Apply, FieldAccess, Section, or Func. Literals and binary expressions do NOT trigger application. This ensures `[1 2 3]` is three elements, not `Apply(Apply(1,2),3)`.

## What Needs Doing Next

### Phase 1 implementation (`crates/lx/`):
- Cargo.toml with deps from impl/implementation-phases.md
- span.rs, token.rs, lexer.rs (impl-lexer.md)
- ast.rs (impl-ast.md)
- parser.rs + parser_expr.rs (impl-parser.md)
- value.rs, env.rs (impl-interpreter.md)
- interpreter.rs (basic arithmetic, bindings, blocks)
- error.rs (miette diagnostics)
- `crates/lx-cli/` with `lx run`

### Phase 11 (Data Ecosystem, post-Phase 10):
- `std/df` (Polars) — highest priority, covers 80% of data scripting
- `std/db` (SQLite + DuckDB) — persistence and analytical SQL
- `std/num` (ndarray) — vectorized numerical computation
- `std/ml` (candle/ONNX) — embeddings, classification, local inference
- `std/plot` (charming) — terminal/SVG charts for observe-iterate workflow
- Suite files: 17_dataframes.lx through 21_plot.lx

### Known Spec Tensions

Things that are decided but feel slightly off. Worth revisiting if they cause implementation friction:

- **`it` in `sel` blocks** — only implicit binding in the language. Everything else is explicit. Could change to `sel { expr -> (result) handler }` with explicit binding, but that's more tokens.
- **Shell line is single-line only** — no backslash continuation. Forces `${ }` blocks for anything complex. This is probably fine but worth watching for friction.
- **Function body extent** — inline lambdas in pipe arguments consume `|` operators. This is natural for binding context (`f = (x) x | g`) but surprising in pipe chains (`map (x) x | g`). Blocks resolve it, but it's one more thing to remember. Sections cover 80% of cases.
- **Implicit Err early return scope** — the rule only applies to `-> T ^ E` annotated functions. Unannotated functions don't short-circuit on Err. This means adding/removing a type annotation can change runtime behavior. In practice, validation functions should always be annotated, so this is fine. But it's a subtlety to watch.

## Session History

### Session 1 (2026-03-13)
Read entire spec (19 files), impl (3 files), suite (1 README). Found and fixed:
- `fetch_users` example had dead `Err` expression (no early return)
- CSV example tuple destructuring was underspecified → added tuple auto-spread to spec
- Division-by-zero as `Err` contradicted "fewest tokens" axiom → changed to panic
- `none?` had ambiguous 1-arg/2-arg overloading with currying → made 2-arg only
- Grammar was missing `use` statement entirely → added `use_stmt` production
- `Handle` and `Duration` types were referenced but never defined → marked as opaque
- Added `pmap_n`, `math.safe_div`, `math.safe_mod` to stdlib
- Wrote 7 impl design docs (parser, AST, checker, interpreter, builtins, formatter, stdlib)
- Wrote 8 suite test files covering phases 1–4 (~350 assertions)
- Added cross-references between all three directories

### Session 2 (2026-03-13)
Full re-read of all 37 files (19 spec, 10 impl, 8 suite). Systematic audit for contradictions, ambiguities, and gaps.

**Critical fix: Operator precedence restructured.** The original table had `|` (pipe) at precedence 11, below comparison at 8. This meant `[1 2 3] | map (* 2) == [2 4 6]` would parse as `[1 2 3] | (map (* 2) == [2 4 6])` — comparing a curried function to a list. Every `assert (pipeline == expected)` test was broken under the stated rules. Moved pipe to position 8 (above comparison at 9). `^` and `??` remain below pipe. Updated grammar.md, syntax.md, design.md, impl-parser.md.

**Spec fixes:**
- syntax.md: `filter (not <> empty?)` was backwards → `filter (empty? <> not)`. `<>` is left-to-right, so `not <> empty?` = `(x) empty? (not x)`, not the intended `(x) not (empty? x)`.
- syntax.md: pipe described as "lowest precedence except bindings" → updated to "higher than comparison and logical operators"
- examples.md: retry example had 5-arg call to 3-param function (unit `()` parsed separately from function body). Fixed to assign function to variable first.
- examples.md: retry `break val` vs `break (Err e)` inconsistency → now always wraps in Ok/Err.
- examples.md: inline function bodies in pipe chains consumed `|` operators. Added `{ }` blocks to `flat_map` and `filter` bodies.
- examples-extended.md: `elapsed | time.ms` used creation function as extractor → changed to `time.to_ms`.
- examples-extended.md: CSV report `map (r) r."amount" | parse_int ^` body extent issue → added blocks.
- stdlib-modules.md: added `to_ms`/`to_sec`/`to_min` Duration conversion functions to std/time.

**Impl fixes:**
- impl-parser.md: added function body extent documentation (bodies consume pipes in argument position; use blocks)
- impl-parser.md: updated binding power table to match new precedence
- impl-interpreter.md: added `Value::Opaque` variant for Handle and Duration types

**Suite fixes:**
- 03_arithmetic.lx: short-circuit tests used wrong precedence grouping. `false && true == false` relied on `==` binding before `&&` by accident. Added explicit parens.
- 04_functions.lx: removed broken `parse_int <> (* 2)` composition (parse_int returns Result, not Int).
- 05_pipes.lx: moved function definitions before first assert (top-level statements are sequential). Fixed `tap ()` to `tap (_)`. Simplified nested pipe test. Removed broken parse_int composition.
- 08_iteration.lx: `map (x) x * x == [1 4 9]` body consumed `==`. Added block: `map (x) { x * x }`.
- 09_errors.lx: **NEW** — 270 lines, ~80 assertions covering Result/Maybe construction, matching, `^` propagation, `??` coalescing, `require`, implicit Ok wrapping, predicates, pipeline error patterns, sections with `??`.
- 10_shell.lx: **NEW** — 245 lines, ~60 assertions covering all four `$` variants, string/expression interpolation, OS pipe vs language pipe, exit code handling, multiline blocks, error propagation.
- All `??` tests wrapped in parens due to `??` being below comparison in precedence.

**Grammar updates:**
- grammar.md: expanded built-in names list with ~30 missing functions (predicates, string ops, conversions). Added reference to stdlib.md.
- suite/README.md: updated to reflect 09 and 10 are no longer TODO.
- errors.md: cross-reference updated to point to 09_errors.lx.

### Session 3 (2026-03-13)
Full re-read of all 41 files. Focus: fill spec gaps, write missing suite files, add missing impl doc, fix bugs.

**Composition order bug (same class as Session 2):**
- 04_functions.lx: `not <> is_positive` was backwards → fixed to `is_positive <> not`. `<>` is left-to-right: `f <> g` = `(x) g(f(x))`. `not <> is_positive` would call `not` on an Int (type error). `is_positive <> not` correctly gives `(x) not(is_positive(x))`. This is the same pattern as Session 2's `filter (not <> empty?)` fix. **Takeaway: always read `f <> g` as "apply f, then pipe to g."**

**Spec gaps fixed:**
- grammar.md: type definitions lacked generic parameters. `Tree a = | Leaf a | Node (Tree a) (Tree a)` was valid in examples/types.md but not in the EBNF. Added `IDENT*` to the binding production: `"+"? TYPE IDENT* "=" type_def`.
- grammar.md: added `encode`/`decode` to built-in names list.
- runtime.md: clarified record equality is order-independent (`{x: 1 y: 2} == {y: 2 x: 1}` is true).
- stdlib-modules.md: added `math.min a b` and `math.max a b` for 2-value comparison (the existing `min xs` is list-only).
- stdlib.md: added `encode`/`decode` to conversion functions (were referenced in runtime.md but homeless). Clarified `min xs`/`max xs` on empty list is a runtime panic.
- modules.md: clarified that `env.args` requires `use std/env` (was ambiguous about whether it's always in scope).
- types.md: added `Bytes` primitive type mention (was in runtime.md but absent from types.md).

**New impl doc:**
- impl-error.md: **NEW** — 282 lines covering the full error system: LxError enum (10 variants), miette Diagnostic integration, propagation trace mechanics, JSON output, pipeline error context, assert value display, parser error recovery. This was the missing piece — every other data flow component had a design doc except the error system.

**New suite files:**
- 12_types.lx: **NEW** — 183 lines, ~50 assertions. Covers type annotations (params, returns, standalone bindings), record types, structural subtyping (extra fields OK), tagged unions (with record payloads), generic types (Pair, Tree), recursive types (Json), type alias interchangeability (Point/Velocity), Result/Maybe annotations with `^`, implicit Ok wrapping, function types in annotations, empty collection annotations.
- 13_concurrency.lx: **NEW** — 188 lines, ~55 assertions. Covers par blocks (basic, nested, heterogeneous results), pmap (basic, block body, empty list), pmap_n (rate-limited), sel blocks (race with timeout, `it` binding, handler transformation), error propagation via `^` inside par, par without `^` returning raw values, immutable captures in concurrent bodies, local mutable bindings inside concurrent bodies, mutable capture restriction (compile error, noted in comments).

**Suite additions to existing files:**
- 06_collections.lx: added record equality order-independence tests and `sorted?` predicate tests (+7 assertions).
- 13_concurrency.lx: fixed par error propagation test that incorrectly used `??` on a par-block tuple. Changed to use `^` inside the par with a function wrapper, and added a test showing par without `^` returns raw values.

**Cross-reference improvements:**
- README.md: added impl-error.md to impl table, added 09/10/12/13 to suite table, updated status from phases 1-4 to phases 1-8.
- diagnostics.md: added Cross-References section (to impl-error.md, errors.md, toolchain.md).
- concurrency.md: added Cross-References section (to impl-interpreter.md, impl-builtins.md, design.md, 13_concurrency.lx).
- types.md: updated cross-reference from `12_types.lx (TODO)` to active link.

**Readiness assessment: All 5 criteria now met.** The language is ready for Phase 1 implementation. The only remaining pre-implementation task is 11_modules/ (multi-file import tests), which is Phase 7 — not blocking Phase 1.

### Session 4 (2026-03-13)
Full re-read of all 44 files. Focus: resolve contradictions, fix broken tests, tighten spec precision.

**`log` namespace resolution (spec contradiction):**
`log` was used both as a bare function (`log "info message"`) and as a record namespace (`log.warn "msg"`). A value in lx cannot be both a function and a record — this was a genuine contradiction. Resolved: `log` is a record with fields `info`, `warn`, `err`, `debug`. Use `log.info "msg"` for info level. No bare `log "msg"` shorthand. Updated runtime.md, grammar.md, stdlib.md, design.md, errors.md, pattern-matching.md, shell.md, examples.md, iteration.md.

**Shell test POSIX fix (suite bug):**
10_shell.lx used `echo "a\nb\nc"` expecting actual newlines, but `/bin/sh` `echo` does NOT interpret `\n` escape sequences (POSIX-undefined behavior). All multi-line shell tests would fail on real execution. Fixed by replacing `echo` with `printf` for escape sequence tests throughout 10_shell.lx.

**Suite fixes:**
- 12_types.lx: used `| sqrt` without `use std/math` import. Replaced `dist` (requiring sqrt) with `dist_sq` (squared distance, no import needed).
- 10_shell.lx: 6 instances of `echo` with `\n` replaced with `printf`.

**Spec fixes:**
- concurrency.md: `.value` and `.error` field access on `Ok`/`Err` variants — these are tagged unions, not records. Fixed to use `(?? ())` for unwrapping Ok values and pattern matching for extracting Err values. Same bug in two places.
- examples-extended.md: 3 examples had missing module imports. Health Checker: added `use std/env` and `use std/fmt`. Log File Analyzer: added `use std/math`. Config Merger: added `use std/env`.
- modules.md: simple script example used `env.get` without `use std/env`. Added import.
- examples.md: `slow_url` variable used but never defined in Concurrent API example. Added definition. Retry example renamed `retry` → `with_retry` to avoid shadowing the built-in `retry` function.
- runtime.md: assert must require Bool (no-truthiness rule applies). Made shadowing warning definitive ("warns" not "may warn").
- collections.md: clarified record ordering (insertion order preserved for iteration, equality is order-independent). Added map key comparability note (functions cannot be map keys).

**Impl fixes:**
- implementation-phases.md: Phase 1 test cases said "division by zero returns Err" — contradicts the spec (it's a panic). Fixed.
- impl-parser.md: added explicit list of callable vs non-callable AST forms for juxtaposition application. Without this, `[1 2 3]` could parse as `Apply(Apply(1, 2), 3)` instead of a 3-element list. Callable forms: Ident, TypeName, Apply, FieldAccess, Section, Func. Non-callable: Literal, Binary, List, Record, etc.
- iteration.md: renamed user-defined `repeat` to `forever` to avoid shadowing the built-in string `repeat` function.

**Cross-references added:**
- shell.md: added Cross-References section (impl-lexer.md, impl-interpreter.md, design.md, 10_shell.lx).
- toolchain.md: added Cross-References section (implementation.md, implementation-phases.md, impl-formatter.md, diagnostics.md, impl-error.md).

**New key design decision:**
- **`log` is a record namespace**, not a function. `log.info`, `log.warn`, `log.err`, `log.debug`. This is the only built-in name that is a record rather than a function. It follows the same access pattern as stdlib modules (`fs.read`, `time.sec`).
- **Application requires callable left-hand side** — the Pratt parser only attempts juxtaposition application when the left-side AST node is syntactically callable (Ident, TypeName, Apply, FieldAccess, Section, Func). Literals and other expression forms do NOT trigger application. This is what makes `[1 2 3]` parse as three elements.

**Data ecosystem design (Phase 11):**
lx replaces Python for LLM scripting — it needs data processing capabilities, not just shell automation. Added 5 new stdlib modules as Phase 11, all requiring zero language changes (pipes + sections + data-last provide the ergonomics):
- `std/df` — Polars dataframes. Columnar, lazy, section-to-column-expr translation. `df.read_csv "data.csv" ^ | df.filter (.amount > 1000) | df.group_by [(.region)] | df.agg {total: df.sum (.amount)}`.
- `std/db` — SQLite (transactional) + DuckDB (analytical). SQL with `{expr}` parameterization. DuckDB reads CSV/Parquet directly.
- `std/num` — ndarray-backed typed numerical arrays. Vectorized math, statistics (mean/median/percentile/correlation), rolling operations.
- `std/ml` — candle/ONNX inference. Embeddings (text → vector), cosine similarity, classification, generation. Enables semantic scripting: "find similar files," "cluster error logs."
- `std/plot` — charming SVG + terminal Unicode charts. bar/line/scatter/histogram. Enables the observe-iterate workflow.

Created stdlib-data.md (spec), updated implementation-phases.md (Phase 11 + deps), implementation.md (crate choices), impl-stdlib.md (module loader), stdlib.md (module table), open-questions.md (resolved), suite/README.md (test file plan), README.md (spec table + status).

### Session 5 (2026-03-13)
Full re-read of all 47 files. Focus: find remaining spec contradictions, fill impl gaps, write missing suite files, tighten grammar precision.

**Critical fix: Implicit Err early return in Result-annotated functions.**
09_errors.lx `validate` and `check_positive` used bare `Err` in non-final statement position:
```
validate = (age: Int) -> Int ^ Str {
  age < 0 ? Err "too young"       -- bare Err in statement position
  age > 150 ? Err "too old"       -- bare Err in statement position
  age                              -- final expr: implicit Ok wrapping
}
```
Without early return, `validate(-1)` would: evaluate `age < 0 ? Err "too young"` → `Err "too young"` (discarded), evaluate `age > 150 ?` → `()` (discarded), return `Ok(-1)`. The test `assert (validate (-1) == Err "too young")` would FAIL. This was a genuine contradiction present since Session 2.

Resolved by adding a new spec rule: in functions with `-> T ^ E` annotation, any bare expression statement that evaluates to `Err e` immediately returns `Err e`. This is the lx equivalent of `if err != nil { return err }` in Go, but without the boilerplate. The rule applies only to annotated functions — unannotated functions treat `Err` as an ordinary value.

Updated: errors.md (new section), design.md (new decision), runtime.md (block evaluation note), impl-interpreter.md (evaluation logic), impl-checker.md (validation note). Removed "No early return" from Known Spec Tensions (resolved). Added new tension: "Implicit Err early return scope" (annotation changes runtime behavior).

**Grammar fixes:**
- grammar.md: `module_path` was `("." ".")* "./"?` — missing `/` after `..`. Fixed to `("../")*  "./"?`. Without this, `../shared/types` wouldn't parse.
- grammar.md: added `named_arg = IDENT ":" expr` production and `named_arg*` to application. Call-site named arguments (`f x greeting: "hi"`) had no grammar production. Added grammar note about disambiguation from record fields (context-dependent: inside `{}` = record field, in application = named arg).

**Impl gap fixed: TypeDef AST node.**
impl-ast.md had no representation for type definitions (`Shape = | Circle Float | Rect Float Float`). The grammar has `"+"? TYPE IDENT* "=" type_def` but the AST's `Stmt` enum only had `Binding`, `Use`, and `Expr`. Added `Stmt::TypeDef(TypeDefStmt)` with `name`, `params`, `def` (Record or Union), and `exported` flag. Added type definition parsing section to impl-parser.md.

**Parser notes added:**
- impl-parser.md: documented `dbg` special-casing (parser emits `Expr::Dbg` instead of `Expr::Apply` when it sees `Apply(Ident("dbg"), inner)`, to capture source text at parse time).
- impl-parser.md: added type definition parsing section covering how `TYPE IDENT* "="` dispatches to record or union type parsing.

**Spec clarifications:**
- iteration.md: added `scan` to HOF examples showing that the initial value IS included in output (`[1 2 3] | scan 0 (+) == [0 1 3 6]`). Previously only documented as "fold returning all intermediate values" which was ambiguous.
- stdlib.md: removed misleading reference to `repeat` as a sequence constructor (it's the string repeat function `repeat n s`). The cross-reference now says "for custom sequence constructors via the iterator protocol."
- runtime.md: added section on block evaluation semantics and `break` returning unit when no value is provided.

**New suite files:**
- 16_edge_cases.lx: ~100 assertions covering disambiguation rules (grouping vs tuple vs unit), operator precedence interactions (pipe > comparison, `^`/`??` below pipe), all section forms (right/left/field for every operator), composition direction (`<>`), function body extent with blocks, ternary record disambiguation, range edge cases, negative indexing, string functions, multiline continuation, and the new implicit Err early return feature.
- 11_modules/main.lx: ~30 assertions testing whole-module import, aliased import, selective import, type imports, module functions in pipelines, currying with module functions.
- 11_modules/lib_math.lx: exported math utilities (add, mul, square, clamp_to, doubled_square, Point type, origin record).
- 11_modules/lib_types.lx: exported type definitions (Color tagged union, color_name, generic Pair, make_pair, swap_pair).

**Cross-references updated:**
- errors.md: added refs to impl-interpreter.md (Err early return), impl-checker.md (Err early return validation), 16_edge_cases.lx.
- README.md: added 16_edge_cases.lx and 11_modules/ to suite table, updated status line.
- suite/README.md: updated 11_modules and 16_edge_cases from TODO to active entries.

**Readiness assessment: All 5 criteria still met.** The implicit Err early return rule resolved the last spec contradiction. 11_modules/ tests are written (no longer a pre-implementation blocker). Implementation can proceed to Phase 1.
