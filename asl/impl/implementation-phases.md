# Implementation Phases

Each phase produces a working, testable increment. No phase depends on a later phase. Each phase ends with `just test` passing.

## Phase 1: Lexer + Literal Expressions

**Goal:** Lex and parse literal expressions, bindings, and arithmetic. Run `lx run` on trivial scripts.

**Deliverables:**
- `crates/lx/` with Cargo.toml (deps: `miette`, `num-bigint`, `num-traits`, `thiserror`)
- `crates/lx-cli/` with Cargo.toml (deps: `lx`, `tokio`, `clap` or just arg parsing)
- Lexer: integers, floats, strings (with interpolation), booleans, unit, operators, identifiers, types, comments (`--`), newlines, `;`
- Parser: literals, binary ops (+, -, *, /, %, //), unary (-, !), grouping `(expr)`, bindings (`=`, `:=`, `<-`), blocks `{ stmts }`
- Interpreter: evaluate arithmetic, bindings, print last expression
- `lx run file.lx` works for: `x = 5; y = x + 3; y * 2` → prints `16`
- Diagnostics: parse errors with source spans via miette

**Test cases:** arithmetic, precedence, integer overflow (bigint), float widening, division by zero panics, mutable binding + reassignment.

## Phase 2: Functions, Pipes, Sections

**Goal:** First-class functions, pipe operator, sections, auto-currying.

**Deliverables:**
- Lexer: `|`, `->`, function params `(x y)`
- Parser: function definitions `name = (params) body`, application by juxtaposition, pipe `|`, sections `(* 2)` `(.field)`, composition `<>`
- Interpreter: closures (Env capture), function application, pipe threading (data-last), sections as anonymous functions, currying for all-positional functions
- `[1 2 3] | map (* 2) | sum` works

**Test cases:** closures capture scope, currying, pipe left-to-right, section for each operator, composition, data-last threading.

## Phase 3: Collections + Pattern Matching

**Goal:** Lists, records, maps, sets, tuples. The `?` operator in all three modes.

**Deliverables:**
- Lexer: `[`, `]`, `{`, `}`, `%{`, `#{`, `..`, `..=`, `_`
- Parser: list/record/map/set/tuple literals, spread `..`, field access `.`, slicing, destructuring patterns, `?` (multi-arm, ternary, single-arm), guards `&`
- Interpreter: collection values, structural equality, `get`/`contains?`/`len`/`empty?`, pattern matching with destructuring, exhaustiveness checking (warnings)
- Value: implement `PartialEq` for structural equality

**Test cases:** each collection type, spread merge, negative indexing, slicing, nested destructuring, guard conditions, exhaustiveness warnings, no truthiness (non-Bool in ternary is type error).

## Phase 4: Iteration + Lazy Sequences

**Goal:** `map`, `filter`, `fold`, ranges, lazy evaluation, `loop`/`break`, iterator protocol.

**Deliverables:**
- Built-in HOFs: `map`, `filter`, `fold`, `flat_map`, `each`, `sort`, `sort_by`, `rev`, `take`, `drop`, `zip`, `enumerate`, `partition`, `group_by`, `chunks`, `windows`, `find`, `any?`, `all?`, `count`, `sum`, `product`, `uniq`, `flatten`, `intersperse`, `scan`, `take_while`, `drop_while`, `min`, `max`, `min_by`, `max_by`
- Ranges: `1..10`, `1..=10`, lazy production
- Lazy sequences: pipeline stages propagate laziness, forcing ops (`collect`, `sort`, `len`)
- Iterator protocol: any record with `next: () -> Maybe a` is iterable
- `loop`/`break` with optional value
- `nat`, `cycle` built-ins

**Test cases:** each HOF, lazy evaluation (verify infinite range doesn't materialize), iterator protocol with custom generator, Fibonacci example, loop with break value.

## Phase 5: Error Handling

**Goal:** `Result`/`Maybe`, `^` propagation, `??` coalescing, implicit Ok wrapping.

**Deliverables:**
- `Ok`, `Err`, `Some`, `None` as tagged union constructors
- `^` postfix: unwrap Ok/Some, propagate Err/None-as-Err
- `??` binary: coalesce Err/None to default
- `require` built-in: Maybe → Result
- Implicit Ok wrapping on final expression of Result-returning functions
- Propagation trace: each `^` site recorded for diagnostics
- `assert` keyword: panic on false, test runner catches

**Test cases:** `^` on Result, `^` on Maybe, `??` on both, propagation chain, pipeline error patterns (`map (x) f x ^`), implicit Ok, assert panics.

## Phase 6: Shell Integration

**Goal:** `$`, `$$`, `$^`, `${ }` — the core scripting use case.

**Deliverables:**
- Lexer: shell mode after `$`/`$$`/`$^`/`${`, `{expr}` interpolation re-entry, shell mode until newline (or `}` for blocks)
- Parser: shell expressions as AST nodes with interpolation holes
- Interpreter: execute via `tokio::process::Command` through `/bin/sh -c`, capture stdout/stderr/exit code
- `$cmd` returns `Result ShellResult ShellErr`
- `$^cmd` returns `Str ^ ShellErr` (extract stdout on exit 0)
- `$$cmd` — no interpolation
- `${ }` — multi-line block, shared shell session
- OS pipe vs language pipe disambiguation (parens to exit shell mode)

**Test cases:** simple commands, interpolation, `$^` with pipe to `trim`, exit code handling, `$$` with literal braces, multi-line block, spawn failure returns Err.

## Phase 7: Modules + Type Checker

**Goal:** `use` imports, `+` exports, structural type checking.

**Status:** Module system is **implemented**. Type checker is **not yet implemented** (type annotations are parse-and-skip).

**Implemented (Session 12):**
- Module system: file = module, `use ./...`, `use ../...`, aliasing `: name`, selective `{name1 name2}`
- Export: `+` prefix at column 0 (both lowercase and uppercase bindings/types)
- Circular import detection
- Module caching (same file loaded once)
- Variant constructor scoping (tagged union constructors imported as bare names)
- Test: `suite/11_modules/` (main.lx + lib_math.lx + lib_types.lx) — PASS

**Not yet implemented:**
- `use std/...` imports (needs stdlib infrastructure from Phase 9)
- Import conflict detection (selective imports with same name)
- Import shadowing warnings
- Bidirectional type checker: annotation propagation, type synthesis, unification
- Structural subtyping: record width subtyping, function types
- Tagged union types: nominal, variant uniqueness within module
- Generic types with instantiation
- `^` in type signatures: `-> Str ^ IoErr`
- `lx check` subcommand

**Test cases:** import resolution, circular import error, type mismatch errors, structural subtyping, generic instantiation, exhaustiveness checking on tagged unions, `^` type compatibility.

## Phase 8: Concurrency

**Goal:** `par`, `sel`, `pmap` with structured concurrency.

**Deliverables:**
- `par { stmts }` — spawn each as tokio task, collect tuple, cancel on error
- `sel { expr -> handler }` — race, cancel losers, bind `it`
- `pmap f xs` — parallel map via JoinSet, preserve order
- Cancellation: SIGTERM for shell, abort for HTTP, recursive cancel for nested par/sel
- Mutable capture restriction: compile error for `:=` bindings captured in par/sel/pmap
- `timeout n` built-in: completes after n seconds

**Test cases:** par collects results, par cancels on error, sel takes first, sel cancels others, pmap preserves order, pmap with error propagation, mutable capture error, timeout in sel.

## Phase 9: Standard Library

**Goal:** Full stdlib as specified in stdlib.md and stdlib-modules.md.

**Deliverables:**
- `std/fs` — fs ops via `tokio::fs` (read, write, walk, stat, mkdir, rm, copy, move, glob, read_lines, open/close)
- `std/net/http` — reqwest wrapper (get, post, put, delete, request)
- `std/json` — serde_json wrapper (parse, encode, encode_pretty)
- `std/csv` — csv crate wrapper (parse, parse_with, encode)
- `std/toml` — toml crate wrapper
- `std/yaml` — serde_yaml wrapper
- `std/time` — chrono/tokio::time (now, elapsed, sleep, sec, ms, min, format, parse, timeout)
- `std/fmt` — formatting functions
- `std/math` — numeric functions
- `std/env` — env vars, args, exit
- `std/io` — stdin/stdout (lazy stdin, read_line, print, println)
- `std/bit` — bitwise ops
- `std/crypto` — sha2/md5/hmac crates
- `std/os` — process info
- `std/rand` — rand crate wrapper
- `std/re` — regex crate wrapper

**Test cases:** per-module test files exercising each function.

## Phase 10: Toolchain Polish

**Goal:** `lx fmt`, `lx test`, `lx repl`, `lx notebook`, `lx watch`, `lx init`, diagnostics polish.

**Deliverables:**
- `lx fmt` — AST pretty-printer with canonical rules (2-space indent, pipe-per-line when >2 stages, record inline when ≤3 fields)
- `lx test` — run test/*.lx, collect assert failures, report counts
- `lx repl` — rustyline loop, persistent bindings, print non-unit results
- `lx notebook` — `---` separated blocks, shared env
- `lx watch` — notify-based file watcher, re-run on change
- `lx init` — create project skeleton (pkg.lx, src/, test/)
- `lx run --json` — miette JSON reporter
- `lx check --strict` — warnings as errors
- Diagnostic polish: pipeline stage/element info, `^` trace formatting, parse error suggestions

**Test cases:** formatter round-trips, test runner collects failures, REPL state persists, watch mode triggers on change.

## Phase 11: Data Ecosystem

**Goal:** `std/df`, `std/db`, `std/num`, `std/ml`, `std/plot` — the modules that make lx a Python replacement for data work.

**Deliverables:**
- `std/df` — Polars wrapper: read_csv/parquet/json, filter/select/group_by/agg/join, lazy evaluation, section-to-column-expr translation, write_csv/parquet
- `std/db` — rusqlite (SQLite) + duckdb: open/close, query/exec, prepared statements, transactions, SQL `{expr}` parameterization
- `std/num` — ndarray wrapper: from_list, element-wise ops, dot/norm/normalize, statistics (mean/median/std_dev/percentile), rolling_mean, correlation, histogram
- `std/ml` — candle-core or ort (ONNX Runtime): model loading, text embeddings, batch embedding, cosine similarity, classification, generation
- `std/plot` — charming (SVG) + custom terminal renderer: bar/line/scatter/histogram/pie/heatmap, title/labels, render to terminal or SVG file

**Phase 11 can be built incrementally:** `std/df` and `std/db` are highest value (cover 80% of data scripting). `std/num` and `std/plot` are medium. `std/ml` is most complex (model loading, tokenization). Each module is independent — implement in any order.

**Test cases:** per-module test files. df: read CSV, filter/group/agg pipeline, join, write. db: CRUD, transactions, parameterized queries. num: vectorized ops, statistics, correlation. ml: embed + similarity. plot: chart construction, SVG output.

## Phase 12: Agent Ecosystem

**Goal:** `std/agent`, `std/mcp`, `std/ctx`, `std/md`, `std/cron` — the primitives that make lx an agentic workflow language. `lx agent` subcommand.

**Deliverables:**
- `std/agent` — Agent spawning via subprocess (lx scripts or external), message passing (JSON over stdin/stdout), channels (mpsc local, Unix domain sockets cross-process), task submission and polling
- `std/mcp` — MCP client (rmcp crate): connect to stdio/HTTP/SSE servers, list tools/resources/prompts, invoke tools with structured args, read resources
- `std/ctx` — Immutable key-value context backed by serde_json. load/save to JSON files, get/set/remove/merge operations
- `std/md` — Markdown parsing (pulldown-cmark): parse to structured document, extract sections/code blocks/frontmatter/links, build documents from node list, render back to markdown string
- `std/cron` — Recurring task scheduling: `every interval f`, `at cron_expr f`, cancel handles. Requires `lx agent` mode (long-lived process)
- `lx agent script.lx` — CLI subcommand that runs scripts in agent mode (keeps process alive for cron/channels). `--daemon` flag for background execution

**Phase 12 can be built incrementally:** `std/ctx` and `std/md` are simplest (pure data processing). `std/mcp` is the highest-value agentic primitive. `std/agent` requires the most design work (process model, message format). `std/cron` requires `lx agent` mode.

**Test cases:** per-module test files. ctx: load/save/get/set round-trips. md: parse/extract/render round-trips. mcp: tool listing and invocation (mock server). agent: spawn/ask/channel (integration tests). cron: scheduling fires at correct intervals.

## Dependency Summary

```toml
[dependencies]
miette = { version = "7", features = ["fancy"] }
num-bigint = "0.4"
num-traits = "0.2"
thiserror = "2"
tokio = { version = "1", features = ["full"] }
regex = "1"
reqwest = { version = "0.13", features = ["json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"
serde_yaml = "0.9"
csv = "1"
sha2 = "0.10"
md-5 = "0.10"
hmac = "0.12"
rand = "0.9"
chrono = { version = "0.4", features = ["serde"] }
rustyline = "15"
notify = "8"
```

Most already in the workspace or in reference/. New additions: `miette`, `num-bigint`, `num-traits`, `rustyline`, `notify`, `sha2`, `md-5`, `hmac`. Everything else is already used.

### Phase 11 Dependencies (Data Ecosystem)

```toml
polars = { version = "0.46", features = ["lazy", "csv", "parquet", "json"] }
rusqlite = { version = "0.32", features = ["bundled"] }
duckdb = { version = "1.1", features = ["bundled"] }
ndarray = "0.16"
ndarray-stats = "0.6"
charming = "0.4"
candle-core = "0.8"
candle-transformers = "0.8"
candle-nn = "0.8"
tokenizers = "0.21"
```

`polars` and `charming` are in reference/. `ndarray`, `rusqlite`, `duckdb`, `candle-*`, and `tokenizers` are new additions.

### Phase 12 Dependencies (Agent Ecosystem)

```toml
rmcp = "0.1"
pulldown-cmark = "0.12"
tokio-cron-scheduler = "0.13"
```

`rmcp` for MCP client. `pulldown-cmark` for markdown parsing. `tokio-cron-scheduler` for recurring tasks. Agent spawning and channels use tokio primitives already in the workspace.
