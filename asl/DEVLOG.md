# lx Development Log

Self-continuity doc. Read this first when picking up lx work cold.

## Implementation Status

Phases 1–8 all implemented (including Phase 7 modules), plus agent communication syntax. **15/15 PASS** via `just test`:

1. **01_literals.lx** — PASS
2. **02_bindings.lx** — PASS
3. **03_arithmetic.lx** — PASS
4. **04_functions.lx** — PASS
5. **05_pipes.lx** — PASS
6. **06_collections.lx** — PASS
7. **07_patterns.lx** — PASS
8. **08_iteration.lx** — PASS
9. **09_errors.lx** — PASS
10. **10_shell.lx** — PASS
11. **11_modules** — PASS ← Session 12 (module system)
12. **12_types.lx** — PASS
13. **13_concurrency.lx** — PASS
14. **14_agents.lx** — PASS ← Session 13 (agent communication syntax)
15. **16_edge_cases.lx** — PASS

## What Exists

- **spec/** (22 files): Complete language specification including agents.md, stdlib-agents.md. grammar.md has full EBNF.
- **impl/** (11 files): Architecture, 12-phase plan, per-component design docs.
- **suite/** (16 .lx files + 3 module files + README): Golden test files for phases 1–8, agent communication, and edge cases (~880 assertions).
- **crates/lx/** — Rust implementation: lexer (with shell mode), parser, tree-walking interpreter with ~80 builtins, iterator protocol, shell execution, regex literals, type annotations (parse-and-skip), slicing, named args, type definitions with tagged values, error propagation, `??` sections, collection-mode application, concurrency (`par`/`sel`/`pmap`/`pmap_n` — sequential impl), module system (`use` imports, `+` exports, aliasing, selective imports, variant constructor scoping, module caching, circular import detection), agent communication (`~>` send, `~>?` ask — language-level infix operators).

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
- **Collection-mode application restriction**. Inside `[]` and `#{}`, ONLY `TypeConstructor` (not `Ident`) triggers application, and only when the next token is NOT another TypeName. This means `[x y]` = two elements (not `x(y)`), `[Ok 1 None]` = three elements (`Ok(1)`, `None`). For multi-arg constructors in lists, use parens: `[(Pair 1 2)]`. This resolves the fundamental tension between whitespace-as-separator and whitespace-as-application in collection literals.
- **`??` coalescing unwraps Ok/Some**. `Ok 42 ?? 0` = `42` (unwrapped), not `Ok 42`. `Err "x" ?? 0` = `0`. `None ?? 5` = `5`. `Some 3 ?? 0` = `3`. Non-Result/Maybe values pass through unchanged.
- **Default params reduce effective arity**. `(name greeting = "hello") body` — calling with 1 arg executes immediately using the default. The function only curries when required (non-default) params are missing.
- **Tuple creation with variables needs semicolons**. `(b; a)` creates a tuple. `(b a)` is function application `b(a)` because `b` is Ident (callable). This only matters when both elements are Idents — `(1 2)` is always a tuple because literals aren't callable.
- **Generic return types need parens**. `-> (Tree a)` not `-> Tree a`. The parse-and-skip approach can't distinguish type params from body start. Simple types (`-> Int`) work directly. Phase 7 (real type checker) will resolve this.
- **Collection-mode in maps and records**. `parse_map` and `parse_record` bump `collection_depth`, restricting application to TypeConstructors only. This prevents `{x: s  y: v}` from applying `s` to `y` across field boundaries. Use parens for function calls in field values: `{x: (f 42)}`.
- **is_func_def ambiguity rule**. `(a b c) (expr)` with all bare Ident params and body starting with `(` is NOT a func def (returns false). This prevents `Node (tree_map f l) (tree_map f r)` from misidentifying the first paren group as a function. Type annotations, defaults, underscores, or patterns make it "strong" and override this rule.
- **`~>` (send) and `~>?` (ask) are infix operators at concat/diamond precedence (21/22)**. `agent ~>? msg ^ | process` parses as `((agent ~>? msg) ^) | process`. Agents are records with a `handler` field. `~>` calls handler and returns Unit (fire-and-forget). `~>?` calls handler and returns the result (request-response). `<-` remains exclusively reassignment — `~>` avoids any ambiguity with the reassign operator.

## What Needs Doing Next

**16/16 PASS. All existing tests pass.** Phases 1–8 implemented, plus agent communication syntax (`~>` and `~>?`), message contracts (`Protocol`), stdlib infrastructure, and `std/json`. See `NEXT_PROMPT.md` for full breakdown.

### Priority order (revised):
1. ~~**Agent communication syntax**~~ ✓ — `~>` (send) and `~>?` (ask) implemented as infix operators. Sequential evaluation for now.
2. ~~**Message contracts**~~ ✓ — `Protocol` keyword with runtime structural validation. Catches malformed messages at send boundaries.
3. ~~**`std/` import infrastructure + stdlib modules**~~ ✓ — `use std/...` paths route to Rust-native modules. 6 modules: `std/json`, `std/ctx`, `std/math`, `std/fs`, `std/env`, `std/re`.
4. **Remaining core agent stdlib** — `std/md`, `std/mcp`, `std/agent`. Built ON TOP of language primitives from #1-2.
5. **Remaining stdlib** (Phase 9) — `std/http`, `std/time`, `std/rand`, etc.
6. **Toolchain** (Phase 10) — `lx fmt`, `lx repl`, `lx check`, `lx agent`.
7. **Data ecosystem** (Phase 11) — Optional. `std/df`, `std/db`, etc.

### Other remaining work:
- Real threading/async for `par`/`sel`/`pmap` (currently sequential)
- Propagation traces for `^`
- Implicit context scope, resumable workflows (see CURRENT_OPINION.md)

### Technical debt:
- Files exceeding 300-line limit: prefix.rs (773), parser/mod.rs (640+), interpreter/mod.rs (520+), hof.rs (425), value.rs (330)
- Named-arg parser consumes ternary `:` separator (workaround: parens around then-branch)

### Completed phases:
- ~~Phase 1–4~~ ✓ (literals, bindings, functions, pipes, collections, patterns, iteration)
- ~~Phase 5~~ ✓ (error handling, `^`, `??`, implicit Err return)
- ~~Phase 6~~ ✓ (shell integration, `$`/`$$`/`$^`/`${}`)
- ~~Phase 7~~ ✓ (modules — `use` imports, `+` exports, aliasing, selective imports, variant constructor scoping)
- ~~Phase 8~~ ✓ (concurrency — sequential impl)
- ~~Agent communication syntax~~ ✓ (`~>` send, `~>?` ask — language-level infix operators)
- ~~`std/` import infrastructure~~ ✓ (`use std/json` routes to Rust-native stdlib modules)
- ~~`std/json`~~ ✓ (`parse`, `encode`, `encode_pretty` via `serde_json`)
- ~~`std/ctx`~~ ✓ (`empty`, `load`, `save`, `get`, `set`, `remove`, `keys`, `merge`)
- ~~`std/math`~~ ✓ (`abs`, `ceil`, `floor`, `round`, `pow`, `sqrt`, `min`, `max`, `pi`, `e`, `inf`)
- ~~`std/fs`~~ ✓ (`read`, `write`, `append`, `exists`, `remove`, `mkdir`, `ls`, `stat`)
- ~~`std/env`~~ ✓ (`get`, `vars`, `args`, `cwd`, `home`)
- ~~`std/re`~~ ✓ (`match`, `find_all`, `is_match`, `replace`, `replace_all`, `split`)

### Language Direction

See [CURRENT_OPINION.md](CURRENT_OPINION.md) — self-critique updated after Session 14. Priorities A (agent communication syntax) and B (message contracts) are DONE. Remaining priorities: C (implicit context scope), D (resumable workflows). Agent lifecycle and tools remain library functions (`agent.spawn`, `mcp.call`).

### Known Spec Tensions

- **`it` in `sel` blocks** — only implicit binding in the language. Everything else is explicit.
- **Shell line is single-line only** — no backslash continuation. Forces `${ }` blocks for anything complex.
- **Function body extent** — inline lambdas consume everything. Block bodies `(x) { body }` stop at the block. Sections cover 80% of cases.
- **Implicit Err early return scope** — only in `-> T ^ E` functions. Adding annotation changes runtime behavior.
- **Juxtaposition in collections** — Session 7 added `collection_depth` flag: inside `[]` and `#{}`, only TypeName constructors (not Ident) trigger application. `[x y]` = two elements, `[Ok 1 None]` = three elements. Multi-arg constructors in lists need parens: `[(Pair 1 2)]`.
- **Minus sections** — `-` excluded from right-section detection. `(- 3)` = unary negation, not a section.
- **Match arm bodies** — Session 7 removed `no_juxtapose` from match arms. Arms are separated by semis (newlines), so application within arm bodies works: `n -> n * factorial (n - 1)`. Single-line inline matches without semis could theoretically have body extent issues but haven't been a problem in practice.
- **Named args + default params + currying** — `greet "bob" greeting: "hi"` fails because `greet "bob"` auto-executes with defaults before the named arg can be applied. The parser produces `(greet("bob"))(greeting: "hi")` but semantically all args belong to one call. Possible fixes: (1) defer default-execution until end of application chain, (2) pass named args as part of the first application, (3) require parens for named arg calls `greet "bob" (greeting: "hi")`. This is a fundamental tension between currying and named args.
- **Agent orchestration design questions** — agent process model (subprocess vs API), discovery mechanism, message serialization format, channel backpressure, workflow resumability. See open-questions.md.
- **Named-arg parser vs ternary `:` separator** — `true ? Ok x : 0` misparses because the named-arg check sees `x :` and consumes the `:` as a named-arg separator. Workaround: use parens `(Ok x)` in the then-branch. Fix: in the named-arg check, verify the Ident is not being used as the last token before a ternary else-separator.
- **Parse-and-skip type annotations are fundamentally limited** — can't distinguish type params from body start without semantic info. `-> Tree b  t ? {}` works via `skip_type_at` (immutable, for is_func_def) which keeps `Ident` in guard, but `skip_type_expr` (mutable, for actual parsing) removes `Ident` from guard. Generic return types need parens: `-> (Tree a)`.

## Session History

### Sessions 1–5 (2026-03-13) — Spec Audit + Completion
Systematic spec contradiction fixes (operator precedence, composition direction, log as record, division-by-zero as panic). Created impl-error.md, stdlib-data.md, and test files (09_errors, 10_shell, 12_types, 13_concurrency, 16_edge_cases, 11_modules). Added implicit Err early return rule for `-> T ^ E` functions. All key design decisions captured in "Key Design Decisions to Remember" above.

### Session 6 (2026-03-13) — First Rust Implementation
Massive bugfixing session: lexer fixes (string interpolation, brace depth tracking), 25+ parser fixes (multiline continuation, sections, patterns, default params, function body extent, tuple destructuring), interpreter features (composition, loop/break, error propagation, integer division), 27 HOF builtins. Added `lx test <dir>` CLI. Test results: 2/13 PASS. Established justfile, clean clippy.

### Session 7 (2026-03-13) — Feature Implementation
Implemented type annotations (parse-and-skip), regex literals, index sections, slicing, named args, type definitions, implicit Err early return, `(?? 0)` sections. Fixed `??` coalescing (unwraps Ok/Some), collection-mode application, default params, match arm bodies, multiline string dedent, composition callable, zip tuple order. Test results: 4/13 PASS.

### Session 8 (2026-03-14) — Agentic Identity Shift
Direction shift: lx is now an agentic workflow language. Created `agents.md` and `stdlib-agents.md` specs. All agentic features are library functions, not keywords. Updated 16 files across spec/impl/suite with agentic identity and cross-refs. Added Phase 12 (Agent Ecosystem) to implementation plan.

### Session 9 (2026-03-14) — Parser Improvements
6 new tests passing (02_bindings, 04_functions, 06_collections, 07_patterns, 12_types, 16_edge_cases). Major parser fixes: nested tuple patterns in params, type annotation skipping overhaul (arrow continuation, bracket/map matching), variant arity detection for wrapped type args, is_func_def ambiguity rule (strong/param_count), collection-mode in maps and records. Test results: 10/13 PASS.

### Session 10 (2026-03-14)
Implemented iterator protocol. Test results: 11/13 PASS (up from 10/13).

**New test passing:** 08_iteration — all 210 lines including `nat`, `cycle`, custom iterator protocol (fibonacci, counter), lazy pipeline composition.

**Iterator architecture:**
- New file `crates/lx/src/iterator.rs` (~155 lines): `LxIter` trait, `IterSource` enum (Nat/Cycle/Live), `LiveIter` type alias (`Arc<Mutex<Box<dyn LxIter + Send>>>`).
- Key design: `IterSource::Nat` and `IterSource::Cycle` are immutable descriptions, freely clonable. Each consumption creates fresh mutable state via `instantiate()`. `IterSource::Live` wraps shared mutable state for pipeline intermediates.
- Concrete iterators: `NatIter` (infinite naturals), `CycleIter` (infinite cycle), `RecordIter` (calls `next` function on records), `MappedIter` (lazy map), `FilteredIter` (lazy filter).
- Records with a `next` field are automatically detected as iterators by HOFs.

**Value changes:**
- Added `Value::Iterator(IterSource)` variant to value.rs.

**Builtin changes:**
- `nat`: bound as `Value::Iterator(IterSource::Nat)` — immutable description, fresh state per consumption.
- `cycle`: 1-arg builtin, returns `Value::Iterator(IterSource::Cycle(items))`.
- `collect`: updated to handle Iterator (pull all) and Record-with-next.
- `map`: returns lazy `MappedIter` when given Iterator/Record-with-next.
- `filter`: returns lazy `FilteredIter` when given Iterator/Record-with-next.
- `take`: pulls N items eagerly from Iterator/Record-with-next → List.
- `drop`: skips N items, returns live Iterator (still lazy).

**Remaining 2 failures (at this point):**
- 10_shell: needs `$` (Phase 6)
- 13_concurrency: needs `par` (Phase 8)

**Shell integration (Phase 6):**
Implemented all four `$` variants. Test results: 12/13 PASS.

**Lexer changes:**
- `$` and `$$` consume the rest of the line as shell text. `$` supports `{expr}` interpolation; `$$` is raw.
- `$^` consumes until `|`, `;`, or newline. First `|` transitions to language pipe. Supports `{expr}` interpolation.
- `${...}` consumes multi-line shell block until `}`. Supports `{expr}` interpolation.
- All variants emit `ShellText(String)` chunks with interpolation tokens, terminated by `ShellEnd`.
- Depth-aware stopping: when `depth > 0` (inside parens/brackets), `$` and `$^` stop at `)` so that `($cmd)` works inside expressions.

**AST changes:**
- Added `Expr::Shell { mode: ShellMode, parts: Vec<StrPart> }`.
- `ShellMode`: Normal (`$`), Raw (`$$`), Propagate (`$^`), Block (`${}`).

**Parser changes:**
- `parse_shell` method collects ShellText and interpolation expressions into `Vec<StrPart>`.
- Shell tokens added to `peek_is_expr_start` and `is_expr_start_kind`.

**Interpreter changes (new file: interpreter/shell.rs):**
- `eval_shell` builds command string from parts, executes via `sh -c`.
- Normal/Raw/Block: returns `Ok({out: Str, err: Str, code: Int})` or `Err({cmd: Str, msg: Str})`.
- Propagate: returns stdout string on exit 0, propagates `Err({cmd, msg})` on nonzero.

**Suite fixes:**
- `$true ? {` → `($true) ? {` ($ consumes full line; use parens to end shell mode).
- `{literal braces}` in assertion strings → `\{escaped}`.
- `wc -l` count fix (needs trailing `\n` for correct count).
- Lambda body extent: `map (r) r ? { ... } | sum` → `map (r) { r ? { ... } } | sum`.
- Command-not-found via `sh -c` returns `Ok` with nonzero code, not `Err`.
- `($true ? { ... })` in record field → use intermediate binding.

**Remaining 1 failure:**
- 13_concurrency: needs `par` (Phase 8)

### Session 11 (2026-03-14) — Concurrency (Phase 8)
Implemented `par`, `sel`, `pmap`, `pmap_n`, `timeout`. Test results: **13/13 PASS** (all tests passing).

**Implementation approach:** Sequential evaluation for now. `par` evaluates statements top-to-bottom and collects results as a tuple. `sel` evaluates the first arm's expression, binds `it`, evaluates the handler. `pmap`/`pmap_n` are sequential map operations. Real threading/async is future work.

**AST changes:**
- Added `Expr::Par(Vec<SStmt>)` — parallel block, collects statement results as tuple.
- Added `Expr::Sel(Vec<SelArm>)` — select/race block with `expr -> handler` arms.
- Added `SelArm { expr, handler }` struct.

**Parser changes:**
- `Par` and `Sel` tokens handled in `parse_prefix`, `peek_is_expr_start`, `is_expr_start_kind`.
- `parse_sel_arms` method: parses `expr -> handler` arms separated by semis.

**Interpreter changes:**
- `eval_par`: evaluates statements sequentially, collects expression results into `Value::Tuple`.
- `eval_sel`: evaluates first arm's expression, binds `it` in child scope, evaluates handler.

**Builtin changes:**
- `pmap` (2-arg): sequential map over list. Same as `map` for now.
- `pmap_n` (3-arg): sequential map with ignored concurrency limit.
- `timeout` (1-arg): sleeps for N seconds, returns Unit. In sequential `sel`, always loses to instant expressions.

**Suite fix:**
- `((p q) (r2 s2))` → `((p; q) (r2; s2))` — nested tuple patterns with variables need semicolons (Idents are callable).

### Session 12 (2026-03-14) — Module System (Phase 7)
Implemented `use` imports and `+` exports. Test results: **14/14 PASS** (up from 13/13).

**New test passing:** 11_modules — whole-module imports, aliased imports, selective imports, variant constructor scoping, module functions in pipelines, currying across modules.

**AST changes:**
- Added `Stmt::Use(UseStmt)` variant.
- Added `UseStmt { path: Vec<String>, kind: UseKind }` and `UseKind` enum (Whole, Alias, Selective).

**Lexer changes:**
- `+` at column 0 now also triggers `Export` before uppercase letters (was lowercase-only). Enables `+Color = | Red | Green | Blue` and `+Point = {x: Float}` exports.

**Parser changes:**
- `parse_use_stmt`: parses `use ./path`, `use ./path : alias`, `use ./path {name1 name2}`.
- Path segments parsed as Ident tokens separated by Slash. Leading `.` or `..` for relative paths.

**Interpreter changes:**
- New file: `interpreter/modules.rs` (~130 lines) — module loading, path resolution, export collection.
- `Interpreter` struct gains `source_dir`, `module_cache`, `loading` fields.
- Module loading: read file → lex → parse → execute in fresh interpreter → collect exports from AST + env.
- Module cache: `Arc<Mutex<HashMap<PathBuf, ModuleExports>>>` prevents double-loading.
- Circular import detection: `Arc<Mutex<HashSet<PathBuf>>>` tracks in-progress loads.
- Variant constructor scoping: tagged union constructors from exported TypeDefs are always brought into scope as bare names (per spec).

**CLI changes:**
- `run` function now computes `source_dir` from file path and passes to Interpreter.
- Test runner scans subdirectories for `main.lx` (enables `11_modules/main.lx` as a test entry).

**Current limitations:**
- Only relative imports (`./`, `../`) supported. `std/` imports not yet implemented (needs stdlib infrastructure).
- No import shadowing warnings.
- No circular import chain reporting (just detects and errors).

### Session 13 (2026-03-14) — Agent Communication Syntax
Implemented `~>` (send) and `~>?` (ask) as language-level infix operators. Test results: **15/15 PASS** (up from 14/14).

**New test passing:** 14_agents — send/ask syntax, propagation, piping, par composition, pmap fan-out, chained asks, multiline continuation, coalescing.

**Design decisions:**
- `~>` for fire-and-forget send (returns Unit), `~>?` for request-response ask (returns handler result)
- Infix operators at concat/diamond precedence (21/22) — tighter than pipe, looser than arithmetic
- Agent = record with `handler` field (function). Future `agent.spawn` will produce same shape backed by subprocess.
- `<-` remains exclusively reassignment — `~>` avoids ambiguity

**Token changes:**
- `TildeArrow` (`~>`) and `TildeArrowQ` (`~>?`) added to TokenKind
- `~` character: `~>` produces TildeArrow, `~>?` produces TildeArrowQ, bare `~` still produces Bang

**AST changes:**
- `Expr::AgentSend { target, msg }` — fire-and-forget
- `Expr::AgentAsk { target, msg }` — request-response

**Parser changes:**
- `TildeArrow`/`TildeArrowQ` in `infix_bp` at (21, 22)
- Handled in `parse_infix` (same pattern as Compose/Pipe)
- Added to multiline continuation token list

**Interpreter changes:**
- `eval_agent_send`: eval target, eval msg, call handler, return Unit
- `eval_agent_ask`: eval target, eval msg, call handler, return result
- `get_agent_handler`: extracts handler function from record; type error if target is not a record or has no handler

**Spec changes:**
- `agents.md` rewritten with `~>` / `~>?` syntax (was library-only `agent.ask`/`agent.send`)
- `design.md` updated: "agent communication has language-level syntax" replaces "all agentic features are library functions"

**Bug discovered (pre-existing, not fixed):**
- Named-arg parser consumes `:` from ternary `? then : else` when then-branch ends with `Ident`. `true ? Ok x : 0` parses `x:` as named arg. Workaround: `true ? (Ok x) : 0`.

### Session 14 (2026-03-14) — Message Contracts (Protocol)
Implemented `Protocol` as a language keyword for structural message validation. Test results: **15/15 PASS**.

**Design decisions:**
- `Protocol Name = {field: Type  field2: Type = default}` — declares a record shape validator
- Protocol application validates at runtime: required fields present, types match via `type_name()`, defaults filled in
- Extra fields allowed (structural subtyping)
- Validation success returns the (possibly augmented) record
- Validation failure is a runtime error (contract violation, like a type error)
- `Any` type skips type checking for a field
- Protocols are exportable (`+Protocol Name = {...}`) and importable via `use`

**Token changes:**
- `Protocol` keyword added to `TokenKind`
- Lexer `read_type_name` recognizes "Protocol" as keyword, not TypeName

**AST changes:**
- `Stmt::Protocol { name, fields, exported }` — protocol definition
- `ProtocolField { name, type_name, default }` — field specification

**Value changes:**
- `Value::Protocol { name, fields }` — runtime protocol value
- `ProtoFieldDef { name, type_name, default }` — runtime field definition
- Protocol values are callable (applied via `apply_func`)

**Parser changes:**
- `parse_protocol` method: parses `Protocol TypeName = {field: Type ...}` syntax
- Inserted before `try_parse_type_def` in `parse_stmt` flow

**Interpreter changes:**
- `eval_protocol_def`: evaluates defaults, creates `Value::Protocol`, binds in env
- `apply_protocol`: validates record against protocol fields, fills defaults, returns record or runtime error

**Module changes:**
- `collect_exports` handles `Stmt::Protocol { exported: true, .. }`

**Suite additions:**
- Protocol tests in 14_agents.lx: basic validation, defaults, structural subtyping, agent integration, Any type, pipeline usage

### Session 15 (2026-03-14) — `std/` Import Infrastructure + `std/json`
Implemented stdlib module system and 6 stdlib modules. Test results: **16/16 PASS** (up from 15/15).

**New test passing:** 15_stdlib — `std/json`, `std/ctx`, `std/math`, `std/fs`, `std/env`, `std/re`. All import styles (whole, aliased, selective). ~80 assertions.

**Architecture:**
- New directory `crates/lx/src/stdlib/` with `mod.rs` (registry) and `json.rs`
- `stdlib::get_std_module(path)` maps `["std", "json"]` → `ModuleExports` with Rust-native builtins
- `stdlib::std_module_exists(path)` for checking before resolution
- Easily extensible: add a new file, add a match arm in `mod.rs`

**Module system changes (`interpreter/modules.rs`):**
- `eval_use` now checks `std_module_exists` BEFORE attempting file-based resolution
- Stdlib modules bypass file I/O, lexing, parsing — they build `ModuleExports` directly from Rust
- Non-relative, non-std paths get a clear error message

**`std/json` module (`stdlib/json.rs`):**
- `parse`: `Str -> Result` — parses JSON string via `serde_json`, returns `Ok(value)` or `Err(msg)`
- `encode`: `Value -> Str` — encodes lx value to compact JSON string
- `encode_pretty`: `Value -> Str` — encodes with indentation
- Bidirectional conversion between `serde_json::Value` and lx `Value`
- JSON null ↔ `None`, JSON object ↔ `Record`, JSON array ↔ `List`
- `serde_json` with `preserve_order` feature to maintain record key order

**`std/ctx` module (`stdlib/ctx.rs`):**
- `empty`: `Unit -> Record` — returns empty context
- `load`: `Str -> Result` — loads JSON file as context, returns `Ok(record)` or `Err(msg)`
- `save`: `Str -> Record -> Result` — saves context to JSON file (pretty-printed)
- `get`: `Str -> Record -> Maybe` — gets field value, returns `Some(val)` or `None`
- `set`: `Str -> Value -> Record -> Record` — returns new record with key set
- `remove`: `Str -> Record -> Record` — returns new record without key
- `keys`: `Record -> List` — returns list of key strings
- `merge`: `Record -> Record -> Record` — merges two records (second overrides first)
- Context is immutable — `set`/`remove`/`merge` return new records

**`std/math` module (`stdlib/math.rs`):**
- `abs`, `ceil`, `floor`, `round`, `pow`, `sqrt`, `min`, `max`
- Constants: `pi`, `e`, `inf`
- Works with both Int and Float inputs

**Shared JSON conversion (`stdlib/json_conv.rs`):**
- Extracted `json_to_lx` and `lx_to_json` from `json.rs` for reuse by `ctx.rs`

**`std/fs` module (`stdlib/fs.rs`):**
- `read`, `write`, `append`: file I/O returning `Ok`/`Err`
- `exists`: path existence check (returns Bool)
- `remove`: removes files or directories recursively
- `mkdir`: creates directories recursively
- `ls`: lists directory entries sorted alphabetically
- `stat`: returns `{size, is_file, is_dir, readonly}` record

**`std/env` module (`stdlib/env.rs`):**
- `get`: env var lookup, returns `Some(val)` or `None`
- `vars`: returns all env vars as a Record
- `args`: returns program args as a List
- `cwd`: returns current working directory
- `home`: returns `$HOME` or `None`
- Note: `set` omitted because `std::env::set_var` is `unsafe` in Rust 2024

**`std/re` module (`stdlib/re.rs`):**
- `match`, `find_all`, `is_match`: pattern matching on strings
- `replace`, `replace_all`: substitution
- `split`: split by pattern
- Accepts both `Str` and `Regex` (lx regex literal `r/pattern/flags`) patterns

**Dependencies added:**
- `serde_json` v1 (workspace dep with `preserve_order` feature)
- `serde` v1 (workspace dep, not directly used yet but needed for future stdlib modules)
