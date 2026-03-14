# lx Development Log

Self-continuity doc. Read this first when picking up lx work cold.

## Implementation Status

Phases 1–8 all implemented, plus agent communication, message contracts, stdlib infrastructure, and 9 stdlib modules. **16/16 PASS** via `just test`:

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
14. **14_agents.lx** — PASS ← Session 13-14 (agent communication + Protocol)
15. **15_stdlib.lx** — PASS ← Sessions 15-17 (stdlib infrastructure + 9 modules)
16. **16_edge_cases.lx** — PASS

## What Exists

- **spec/** (22 files): Complete language specification including agents.md, stdlib-agents.md. grammar.md has full EBNF.
- **impl/** (11 files): Architecture, 12-phase plan, per-component design docs.
- **suite/** (16 .lx files + 3 module files + fixtures/ + README): Golden test files for phases 1–8, agent communication, stdlib (including MCP), and edge cases (~960 assertions).
- **crates/lx/** — Rust implementation: lexer (with shell mode), parser, tree-walking interpreter with ~80 builtins, iterator protocol, shell execution, regex literals, type annotations (parse-and-skip), slicing, named args, type definitions with tagged values, error propagation, `??` sections, collection-mode application, concurrency (`par`/`sel`/`pmap`/`pmap_n` — sequential impl), module system (`use` imports, `+` exports, aliasing, selective imports, variant constructor scoping, module caching, circular import detection), agent communication (`~>` send, `~>?` ask — language-level infix operators, with subprocess agent support via `__pid`), message contracts (`Protocol`), stdlib (`std/json`, `std/ctx`, `std/math`, `std/fs`, `std/env`, `std/re`, `std/md`, `std/agent`, `std/mcp`).

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
- **`~>` (send) and `~>?` (ask) are infix operators at concat/diamond precedence (21/22)**. `agent ~>? msg ^ | process` parses as `((agent ~>? msg) ^) | process`. Agents are records with a `handler` field. `~>` calls handler and returns Unit (fire-and-forget). `~>?` calls handler and returns the result (request-response). `<-` remains exclusively reassignment — `~>` avoids any ambiguity with the reassign operator. Subprocess agents (records with `__pid` field) are handled transparently — the interpreter routes to subprocess I/O instead of calling a handler function.
- **Parens reset collection_depth**. `(md.h1 "Test")` inside `[...]` must apply `md.h1` to `"Test"`, not create a tuple. Session 16 fixed this: `parse_paren` saves/restores `collection_depth`, resetting to 0 inside parens. This means `[(f x)]` correctly applies `f` to `x` even inside a list literal.

## What Needs Doing Next

**17/17 PASS.** Phases 1–8 + agent communication + message contracts + 9 stdlib modules + MCP HTTP transport all implemented. The agentic workflow loop is closed. See `NEXT_PROMPT.md` for full breakdown.

### What's done:
- Phases 1–8 ✓ (core language, shell, modules, concurrency)
- Agent communication ✓ (`~>` send, `~>?` ask — infix operators, subprocess-transparent)
- Message contracts ✓ (`Protocol` keyword, runtime structural validation)
- 9 stdlib modules ✓ (`std/json`, `std/ctx`, `std/math`, `std/fs`, `std/env`, `std/re`, `std/md`, `std/agent`, `std/mcp`)
- `lx agent` subcommand ✓ (subprocess agent mode with JSON-line protocol)
- MCP HTTP streaming ✓ (`reqwest` blocking, SSE parsing, session management, transport abstraction)

### Priority order (what's next):
1. **Surface area reduction** (Priority S — HIGH) — Remove features that don't serve agentic workflows. See CURRENT_OPINION.md for the full list. Targets: lazy iterator protocol, currying, set literals, `$$` raw shell, type annotations (parse-and-skip), regex literals, `<>` composition, tuple semicolon rule (switch to comma syntax). Cuts ~15-20% of parser/interpreter surface area.
2. **Remaining stdlib** (Phase 9) — `std/http`, `std/time`, `std/rand`, etc.
3. **Implicit context scope** (Priority E) — eliminate manual state threading
4. **Resumable workflows** (Priority F) — checkpoint/resume for multi-step workflows
5. **Toolchain** (Phase 10) — `lx fmt`, `lx repl`, `lx check`
6. **Data ecosystem** (Phase 11) — Optional. `std/df`, `std/db`, etc.

### Surface area reduction details (Priority S):

| Feature | Why remove | Parser/interpreter savings |
|---------|-----------|--------------------------|
| Lazy iterators | Agents deal with finite data only | iterator.rs, lazy map/filter/take paths in hof.rs |
| Currying | #1 source of parser bugs, sections cover 90% | Simplifies apply.rs, removes is_func_def heuristic complexity |
| Set literals `#{}` | No agentic use case | Remove collection-mode path for sets, Value::Set variant |
| `$$` raw shell | Too niche, `${...}` covers it | Lexer shell mode simplification |
| Type annotations | Parse-and-skip is worse than nothing | Remove skip_type_expr, skip_type_at, all annotation parsing |
| Regex literals `r/.../` | `std/re` with strings is enough | Lexer regex mode removal |
| `<>` composition | `\|` covers it, direction was confusing | Remove AST node, precedence entry, interpreter path |
| Tuple semicolons | Switch to `(a, b)` comma syntax | Eliminates the #1 LLM generation ambiguity |

### Technical debt:
- Files exceeding 300-line limit: prefix.rs (773), parser/mod.rs (640+), interpreter/mod.rs (520+), hof.rs (425), value.rs (330)
- `par`/`sel`/`pmap` are sequential; real async needs `tokio`
- Named-arg parser consumes ternary `:` separator (workaround: parens around then-branch)
- Stale spec files: `examples.md`, `examples-extended.md`, `toolchain.md` still use `agent.ask`/`agent.send` library syntax

### Language Direction

See [CURRENT_OPINION.md](CURRENT_OPINION.md) — rewritten after Session 18. Priorities A–D.5 DONE. **Next: Priority S (surface area reduction) before adding more features.** Then E (implicit context scope), F (resumable workflows).

### Known Spec Tensions

Tensions marked ✂ are resolved by the Priority S surface area reduction.

- **`it` in `sel` blocks** — only implicit binding in the language. Everything else is explicit.
- **Shell line is single-line only** — no backslash continuation. Forces `${ }` blocks for anything complex.
- **Function body extent** — inline lambdas consume everything. Block bodies `(x) { body }` stop at the block. Sections cover 80% of cases.
- ✂ **Implicit Err early return scope** — only in `-> T ^ E` functions. Adding annotation changes runtime behavior. *Resolved: removing type annotations removes this tension entirely.*
- **Juxtaposition in collections** — Session 7 added `collection_depth` flag: inside `[]` and `#{}`, only TypeName constructors (not Ident) trigger application. `[x y]` = two elements, `[Ok 1 None]` = three elements. Multi-arg constructors in lists need parens: `[(Pair 1 2)]`. *Partially resolved: removing set literals `#{}` removes one collection-mode path.*
- **Minus sections** — `-` excluded from right-section detection. `(- 3)` = unary negation, not a section.
- **Match arm bodies** — Session 7 removed `no_juxtapose` from match arms. Arms are separated by semis (newlines), so application within arm bodies works: `n -> n * factorial (n - 1)`.
- ✂ **Named args + default params + currying** — `greet "bob" greeting: "hi"` fails because `greet "bob"` auto-executes with defaults before the named arg can be applied. *Resolved: removing currying eliminates the tension entirely. Functions always require all non-default args.*
- **Agent orchestration design questions** — discovery mechanism, channel backpressure, workflow resumability. See open-questions.md.
- **Named-arg parser vs ternary `:` separator** — `true ? Ok x : 0` misparses because the named-arg check sees `x :` and consumes the `:`. Workaround: use parens `(Ok x)` in the then-branch.
- ✂ **Parse-and-skip type annotations are fundamentally limited** — can't distinguish type params from body start without semantic info. *Resolved: removing parse-and-skip annotations removes this entire class of bugs.*
- **Assert parsing is greedy** — `assert (expr) "msg"` can consume the message as an application argument when `(expr)` is callable. Workaround: `assert (expr == true) "msg"`. *Partially helped by removing currying — fewer things are "callable" in ambiguous ways.*

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

### Session 10 (2026-03-14) — Iterators + Shell (Phases 4, 6)
Implemented iterator protocol (`nat`, `cycle`, record-with-`next`, lazy `map`/`filter`/`take`/`drop`) and shell integration (all four `$` variants with interpolation, depth-aware paren stopping). Test results: 12/13 PASS.

### Session 11 (2026-03-14) — Concurrency (Phase 8)
Implemented `par`, `sel`, `pmap`, `pmap_n`, `timeout` — sequential evaluation for now. Test results: **13/13 PASS**.

### Session 12 (2026-03-14) — Module System (Phase 7)
Implemented `use` imports and `+` exports. `interpreter/modules.rs` handles file loading, caching, circular import detection, variant constructor scoping. Test results: **14/14 PASS**.

### Session 13 (2026-03-14) — Agent Communication Syntax
Implemented `~>` (send) and `~>?` (ask) as language-level infix operators. `TildeArrow`/`TildeArrowQ` tokens, `Expr::AgentSend`/`AgentAsk` AST nodes, precedence (21, 22) at concat/diamond level. Agents = records with `handler` field. Test results: **15/15 PASS**.

### Session 14 (2026-03-14) — Message Contracts (Protocol)
Implemented `Protocol` keyword for structural message validation. `Protocol Name = {field: Type = default}` declares validators. Runtime type checking, default filling, structural subtyping, `Any` type, exportable/importable. `Value::Protocol` is callable. Test results: **15/15 PASS**.

### Session 15 (2026-03-14) — Stdlib Infrastructure + 6 Modules
Implemented `use std/...` routing in `interpreter/modules.rs` — stdlib modules are Rust-native builtins in `crates/lx/src/stdlib/`, bypassing file I/O/lexing/parsing. Added `serde_json` (with `preserve_order`) and `serde` workspace deps. Test results: **16/16 PASS**.

**6 modules implemented:**
- `std/json` (`parse`, `encode`, `encode_pretty`) — bidirectional serde_json conversion, JSON null↔None, object↔Record, array↔List
- `std/ctx` (`empty`, `load`, `save`, `get`, `set`, `remove`, `keys`, `merge`) — immutable context records persisted as JSON files
- `std/math` (`abs`, `ceil`, `floor`, `round`, `pow`, `sqrt`, `min`, `max`, `pi`, `e`, `inf`)
- `std/fs` (`read`, `write`, `append`, `exists`, `remove`, `mkdir`, `ls`, `stat`)
- `std/env` (`get`, `vars`, `args`, `cwd`, `home`) — `set` omitted (unsafe in Rust 2024)
- `std/re` (`match`, `find_all`, `is_match`, `replace`, `replace_all`, `split`) — accepts Str or regex literal

**Architecture:** `stdlib/mod.rs` registry + one `.rs` file per module. `stdlib/json_conv.rs` shared by `json`, `ctx`, and agent subprocess protocol. Adding a new module = one file + one match arm.

### Session 16 (2026-03-14) — Agent-Specific Stdlib (`std/md`, `std/agent`)

Implemented the two agent-differentiator stdlib modules. Test results: **16/16 PASS**.

**`std/md` (markdown processing via `pulldown-cmark`):**
- Parse: `parse` decomposes markdown into block-level node records (heading, para, code, list, ordered, blockquote, hr)
- Extract: `sections`, `code_blocks`, `headings` (from parsed nodes), `links`, `to_text` (from source string)
- Build: `h1`, `h2`, `h3`, `para`, `code`, `list`, `ordered`, `table`, `link`, `blockquote`, `hr` (constant), `raw`, `doc`
- Render: `render` converts node records back to markdown text
- Split across `md.rs` (229 lines) and `md_build.rs` (198 lines) for 300-line limit

**`std/agent` (subprocess agent lifecycle):**
- `spawn` starts a subprocess running `lx agent <script>`, returns `Ok {__pid: Int name: Str}`
- `ask`/`send` communicate via JSON-line protocol (stdin/stdout)
- `kill` terminates the subprocess, `name`/`status` inspect agent state
- Global process registry via `OnceLock<Mutex<HashMap<u32, AgentProcess>>>`
- `~>` and `~>?` transparently handle subprocess agents: interpreter checks for `__pid` field and routes to subprocess I/O instead of handler function call

**`lx agent` subcommand added to lx-cli:**
- Runs a script to get a handler function value
- Enters JSON-line message loop: read line → JSON decode → call handler → JSON encode → write line
- Error responses use `{"__err": "message"}` format

**Parser fix: collection_depth reset in parens.**
- `(md.h1 "Test")` inside `[...]` was parsed as a tuple because `collection_depth > 0` prevented FieldAccess application
- Fix: `parse_paren` now saves/restores `collection_depth`, resetting to 0 inside parens
- This correctly allows `[(f x)]` to apply `f` to `x` even inside a list

### Session 17 (2026-03-14) — MCP Tool Invocation (`std/mcp`)

Implemented `std/mcp` — the last piece needed to close the agentic workflow loop. Agents can now spawn subprocesses, communicate via `~>`/`~>?`, AND invoke MCP tools. Test results: **16/16 PASS**.

**`std/mcp` (MCP client over stdio via JSON-RPC 2.0):**
- `connect`: spawn MCP server subprocess, perform `initialize` handshake + `notifications/initialized`. Accepts URI string (`"stdio:///path"`) or config record (`{command: "cmd" args: ["a" "b"]}`)
- `close`: terminate server subprocess
- `list_tools`: `tools/list` → list of tool records
- `call`: `tools/call` → extracts text content for ergonomic single-value returns. `isError: true` → `Err`
- `list_resources`, `read_resource`: resource discovery and access
- `list_prompts`, `get_prompt`: prompt template discovery and rendering
- Global process registry via `OnceLock<Mutex<HashMap<u64, McpProcess>>>`
- MCP clients are records with `__mcp_id` field (similar to `__pid` for agents)
- Split across `mcp.rs` (111 lines) and `mcp_rpc.rs` (227 lines) for 300-line limit
- Test fixture: `asl/suite/fixtures/mcp_test_server.py` — minimal Python MCP server implementing initialize, tools/list, tools/call (echo + add + fail), resources/list, resources/read, prompts/list, prompts/get

**Design decisions:**
- No `rmcp` crate or tokio dependency — MCP over stdio is just JSON-RPC 2.0 over subprocess stdin/stdout, same pattern as `std/agent`
- `rpc()` helper skips server notifications (messages without `id`) when reading responses
- `call` text extraction: single text content blocks are returned as plain strings for ergonomic piping. Multi-content or non-text results returned as full records
- `connect` URI parsing: `stdio:///path` extracts executable path. Space-separated args supported: `stdio:///usr/bin/env npx server`

### Session 18 (2026-03-14) — MCP HTTP Streaming Transport

Implemented HTTP streaming (Streamable HTTP) transport for `std/mcp`. MCP servers can now be connected via `http://` or `https://` URIs in addition to the existing `stdio://` subprocess transport. Test results: **17/17 PASS**.

**Transport abstraction refactor:**
- `mcp_rpc.rs` now contains `McpTransport` enum (Stdio | Http), `McpConnection` struct, registry, and dispatch logic
- `mcp_stdio.rs` extracted from old `mcp_rpc.rs` — `StdioTransport` handles subprocess spawn, JSON-line I/O, shutdown
- `mcp_http.rs` new — `HttpTransport` handles `reqwest::blocking` POST, SSE response parsing, `Mcp-Session-Id` session tracking
- `mcp.rs` declares all three submodules via `#[path]`, exposes unchanged public API
- All files under 300 lines (mcp.rs: 115, mcp_rpc.rs: 210, mcp_stdio.rs: 99, mcp_http.rs: 164)

**HTTP transport features:**
- `connect` accepts `http://` or `https://` URI strings, or config record `{url: "http://..."}`
- JSON-RPC 2.0 over HTTP POST with `Content-Type: application/json`, `Accept: application/json, text/event-stream`
- SSE response parsing: handles `text/event-stream` Content-Type, extracts JSON-RPC response matching request ID
- `Mcp-Session-Id` header captured from server responses and sent with subsequent requests
- `close` sends HTTP DELETE to terminate session (if session ID exists)
- Notifications sent as POST, server 202 response expected

**Dependencies:**
- Added `reqwest` v0.12 with `blocking` + `json` features to workspace
- `tokio` pulled in as transitive dependency (reqwest's internal runtime for blocking client)

**Test fixture:**
- `asl/suite/fixtures/mcp_test_http_server.py` — HTTP server implementing full MCP protocol (initialize, tools, resources, prompts), writes port/PID to temp files for test orchestration
- `asl/suite/17_mcp_http.lx` — 17 assertions covering connect, list_tools, call, resources, prompts, close, and config record URIs
