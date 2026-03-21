# LSP Implementation Landscape

A survey of Language Server Protocol implementations, architecture patterns, server frameworks, and IDE-oriented compiler design -- informing lx's LSP server design.

## Table of Contents

1. [LSP Specification](#lsp-specification)
2. [Key LSP Features](#key-lsp-features)
3. [LSP Implementations](#lsp-implementations)
4. [LSP Server Architecture Patterns](#lsp-server-architecture-patterns)
5. [Rust LSP Frameworks](#rust-lsp-frameworks)
6. [Semantic Analysis for IDE Features](#semantic-analysis-for-ide-features)
7. [IDE-Oriented Compiler Architecture](#ide-oriented-compiler-architecture)
8. [Incremental Computation](#incremental-computation)
9. [Minimum Viable LSP for Simple Languages](#minimum-viable-lsp-for-simple-languages)

---

## LSP Specification

Source: https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/

### Transport

LSP uses JSON-RPC 2.0 over a base protocol with HTTP-like headers. Each message has a `Content-Length` header (required) followed by `\r\n\r\n` and a JSON body. Three message types:

- **RequestMessage**: has `id`, `method`, `params` -- server must respond
- **ResponseMessage**: has matching `id`, `result` on success or `error` with `code`/`message`/`data`
- **NotificationMessage**: has `method` and `params` but no `id` -- fire-and-forget

Messages prefixed with `$/` are protocol-implementation-dependent (e.g., `$/cancelRequest`, `$/progress`).

### Lifecycle

1. **`initialize` request** -- must be the first message. Client sends `ClientCapabilities`; server responds with `ServerCapabilities`. Until the server responds, the client cannot send other requests.
2. **`initialized` notification** -- client signals initialization is complete.
3. **Normal operation** -- requests and notifications flow both directions.
4. **`shutdown` request** -- client asks server to prepare for termination.
5. **`exit` notification** -- client tells server to terminate the process.

### Capabilities Negotiation

Both sides advertise what they support via capability objects in the initialize handshake:

**Client capabilities**: `workspace` (applyEdit, workspaceEdit, symbol, configuration, workspaceFolders, fileOperations), `textDocument` (synchronization, completion, hover, signatureHelp, definition, references, documentSymbol, codeAction, codeLens, formatting, rename, semanticTokens, inlayHint, diagnostic), `window` (workDoneProgress, showMessage), `general` (positionEncodings).

**Server capabilities**: mirror the client's, declaring which features the server implements. The server only needs to implement features it advertises.

Position encoding must be negotiated -- UTF-16 is the mandatory fallback, but UTF-8 and UTF-32 are available since LSP 3.17.

### Document Synchronization

`TextDocumentSyncKind` controls how file changes are transmitted:

| Kind | Value | Behavior |
|------|-------|----------|
| None | 0 | No synchronization |
| Full | 1 | Entire document text on every change |
| Incremental | 2 | Only changed ranges (start/end positions + new text) |

Full sync is simpler to implement and sufficient for small-to-medium files. Incremental sync is essential for large files where sending the full text on every keystroke is prohibitive.

Document lifecycle notifications: `didOpen`, `didChange`, `willSave`, `willSaveWaitUntil`, `didSave`, `didClose`.

### Cancellation and Progress

- `$/cancelRequest` with request `id` -- server must still respond (typically with error code -32800 `RequestCancelled`)
- Work-done progress via `$/progress`: `begin` -> `report` (with percentage) -> `end`
- Partial results via `partialResultToken` -- stream intermediate results

---

## Key LSP Features

### Tier 1: High-Impact, Implement First

| Feature | Method | Purpose |
|---------|--------|---------|
| Diagnostics | `textDocument/publishDiagnostics` (push) or pull model (3.17) | Errors, warnings, hints inline in editor |
| Completion | `textDocument/completion` | Code completion proposals with resolve support |
| Hover | `textDocument/hover` | Type info, docs on mouse hover |
| Go-to-Definition | `textDocument/definition` | Navigate to symbol definition |
| Document Symbols | `textDocument/documentSymbol` | Outline view / breadcrumbs |

### Tier 2: Productivity Features

| Feature | Method | Purpose |
|---------|--------|---------|
| Find References | `textDocument/references` | All usages of a symbol |
| Rename | `textDocument/rename` (+ `prepareRename`) | Symbol renaming across files |
| Signature Help | `textDocument/signatureHelp` | Function parameter hints while typing |
| Code Actions | `textDocument/codeAction` | Quick fixes, refactorings |
| Formatting | `textDocument/formatting` | Auto-format document |
| Workspace Symbols | `workspace/symbol` | Cross-file symbol search |

### Tier 3: Advanced Features

| Feature | Method | Purpose |
|---------|--------|---------|
| Semantic Tokens | `textDocument/semanticTokens` | Syntax-aware highlighting (full/range/delta) |
| Inlay Hints | `textDocument/inlayHint` | Inline type annotations, parameter names |
| Code Lens | `textDocument/codeLens` | Inline metadata (run counts, test status) |
| Call Hierarchy | `textDocument/prepareCallHierarchy` | Incoming/outgoing call trees |
| Folding Range | `textDocument/foldingRange` | Code folding regions |
| Selection Range | `textDocument/selectionRange` | Smart selection expansion |
| Document Link | `textDocument/documentLink` | Clickable hyperlinks in source |
| Go-to-Type-Definition | `textDocument/typeDefinition` | Navigate to a value's type |
| Go-to-Implementation | `textDocument/implementation` | Navigate to trait/interface implementations |

### Key Data Structures

- **Position**: zero-based `line` and `character` (column encoding negotiated at init)
- **Range**: `start` and `end` Position (end exclusive)
- **Location**: `uri` + `range`
- **TextEdit**: `range` + `newText`
- **WorkspaceEdit**: multi-file edits via `changes` (URI -> TextEdit[]) or `documentChanges`
- **Diagnostic**: `range`, `severity` (1=Error, 2=Warning, 3=Info, 4=Hint), `code`, `message`, `tags` (Unnecessary, Deprecated), `relatedInformation`
- **CompletionItem**: `label`, `kind`, `detail`, `documentation`, `insertText`, `textEdit`, `additionalTextEdits`
- **MarkupContent**: `kind` (plaintext | markdown) + `value`

---

## LSP Implementations

### rust-analyzer (Rust)

Sources: https://rust-analyzer.github.io/blog/2020/07/20/three-architectures-for-responsive-ide.html, rust-analyzer source

The gold standard for LSP implementation. Key architectural decisions:

**Salsa for incremental computation**: rust-analyzer uses the Salsa framework to instrument function calls, recording dependencies between computations. When an input changes, Salsa compares new results with old ones -- if identical, it stops invalidation propagation. This is a query-based incremental computation model where every analysis function is a memoized query.

**Rowan for CST**: uses a lossless concrete syntax tree (not an AST) that preserves all whitespace and comments. Nodes are cheaply cloneable (reference-counted), support parent pointers, and enable incremental reparsing. This is critical for IDE features that need exact source positions and for formatting preservation.

**Chalk for type inference**: type inference is formulated as a logic program solved by Chalk, rust-analyzer's Prolog-like trait solver.

**Cancellation**: when the user types, in-flight analysis is cancelled. The main loop receives LSP messages and VFS updates, then snapshots the database for read-only analysis on worker threads. If the input changes, worker threads detect cancellation via Salsa's revision check and abort.

**Threading model**: latency-sensitive requests (completion, semantic tokens) run on the main thread. Heavy analysis runs on worker threads with read-only database snapshots. Formatting runs on a dedicated thread pool.

**Three IDE architectures** (from matklad's blog):
1. **Map-Reduce (IntelliJ, Sorbet)**: index files independently in parallel -> merge into unified index -> lazy name resolution on queries. Fast but requires per-file indexing to be independent.
2. **Headers-Based (clangd, Merlin/OCaml)**: snapshot compiler state after processing imports -> restart from snapshot on edit. Works well when headers change rarely.
3. **Query-Based (rust-analyzer)**: fine-grained incremental computation via Salsa. Most general but most complex. Necessary for Rust because macros and crate-level scope prevent per-file indexing.

### Pyright (Python)

Source: https://github.com/microsoft/pyright/blob/main/docs/internals.md

Fast Python type checker and LSP server written in TypeScript. Architecture:

**Five analysis phases**:
1. Tokenization -- produces token stream, discards whitespace/comments
2. Parsing -- tokens to parse tree, with `parseTreeWalker` for traversal
3. Binding -- builds scopes and symbol tables, constructs reverse code flow graph for each scope (enables type narrowing at any code point)
4. Checking -- validates statements/expressions using `typeEvaluator` module
5. Selective execution -- checker runs only on files requiring full diagnostics

**Performance strategy**: prioritizes open editor files and their import dependencies. Only files needing full diagnostics get the checker phase; imported-only files skip it. Each workspace gets its own service instance.

**Incremental analysis**: file system watchers detect changes; the program invalidates affected files and their dependents. Import resolution tracks dependencies for targeted invalidation.

### TypeScript (tsserver)

TypeScript's language service is the foundational LSP that inspired the protocol. Key characteristics:

- **Project system**: `tsconfig.json` defines compilation units. The language service maintains a project graph with file membership and configuration.
- **Incremental compilation**: the compiler maintains an incremental program where changed files invalidate their dependents through the module graph.
- **Language service API**: editor features are methods on `LanguageService` -- `getCompletionsAtPosition`, `getDefinitionAtPosition`, etc. The LSP server (`tsserver`) wraps this API.
- **Script snapshots**: files are represented as immutable snapshots. On edit, a new snapshot is created, and the compiler incrementally updates its internal structures.

### gopls (Go)

Source: https://go.googlesource.com/tools/+/refs/heads/master/gopls/doc/design/implementation.md

Go's official LSP server. Architecture:

**Hierarchical workspace model**: `Session` (client communication) -> `Folder` (opened directories) -> `View` (workspace tree with build config). Supports multi-root workspaces.

**Snapshot-based analysis**: each edit produces a new `Snapshot` representing workspace state. Type checking results are memoized in `Package` objects. Dependency-aware invalidation ensures only affected packages are reprocessed.

**Persistent cache (v0.12+)**: file-based key-value storage via `filecache`. Serializable indexes (`xrefs`, `methodsets`, `typerefs`) enable fast restarts, reduced memory, and cross-process synergy.

**Go-specific advantages**: Go's simple type system (no generics until recently, fast compilation) means gopls can afford to re-typecheck packages on edit without heavy incremental infrastructure. The `go/packages` and `go/types` standard library provide the analysis backbone.

### lua-language-server (Lua)

Source: https://github.com/LuaLS/lua-language-server

Self-hosted -- written in Lua. Nearly 1 million VS Code installs.

**Dynamic typing challenge**: uses EmmyLua-style annotations for type hints, enabling static analysis on inherently dynamic code. Supports Lua 5.1 through 5.5 and LuaJIT.

**Features**: completion, hover, go-to-definition, find references, rename, formatting, diagnostics with 20+ configurable annotations.

**Parsing**: uses LPegLabel for parsing with error recovery, producing ASTs via LuaParser.

**Relevance to lx**: demonstrates that dynamic/simple languages can have comprehensive IDE support by combining annotation-based type hints with robust parsing and scope analysis.

### ElixirLS / Next LS (Elixir)

Source: https://github.com/elixir-lsp/elixir-ls

**Macro handling**: Elixir's heavy metaprogramming is handled through incremental Dialyzer analysis. The system performs background type inference to generate success typing information, enabling spec suggestions even for macro-expanded code.

**DAP integration**: ElixirLS includes a Debug Adapter Protocol implementation using `:int.ni/1` for interpreted module debugging. Supports line, function, conditional, hit-conditional, and log breakpoints. Limited to 100 breakpoints.

**Distributed debugging**: supports remote node attachment for debugging distributed BEAM systems.

### Nickel (Configuration Language)

Source: https://github.com/nickel-lang/nickel/tree/master/lsp

Most relevant reference for lx -- a simple language with a well-structured LSP server in Rust using `lsp-server` + `lsp-types`.

**Architecture**: `Server` struct manages connections, diagnostics, background jobs. Uses `crossbeam::select` for non-blocking event loop multiplexing client messages and background diagnostic results.

**Document sync**: full document sync (not incremental). On change, updates file content and queues diagnostics for the changed file and its dependents.

**Semantic state**: `World` struct holds `SourceCache` (file contents), `AnalysisRegistry` (parsed ASTs + typecheck results per file), `ImportData` (bidirectional import dependencies), `PositionLookup` (cursor -> AST node mapping).

**Position mapping**: `PositionLookup` maintains sorted, disjoint interval collections for AST ranges and identifier ranges. The `make_disjoint` algorithm splits overlapping nested ranges so smaller (more specific) ranges win.

**Feature implementations**:
- **Hover**: queries both identifier and AST lookups at cursor position, gathers type info from `analysis_reg.get_type()`, formats annotations + docs
- **Go-to-definition**: gets AST node at cursor -> `world.get_defs()` -> converts spans to LSP locations
- **Find references**: `world.get_field_refs()` + `analysis_reg.get_usages()` -> deduplicated location list
- **Completion**: context-aware -- record field completion, dot access, import paths, environment variables, parse error recovery for incomplete expressions
- **Rename**: gathers defs + usages + field refs -> deduplicates -> groups by file URI -> returns `WorkspaceEdit`
- **Document symbols**: recursive AST walk with `MAX_SYMBOL_DEPTH=32`, reports `SymbolKind::VARIABLE` with type detail
- **Formatting**: invokes built-in formatter, returns single full-document `TextEdit` (TODO: diff-based edits)
- **Code actions**: currently just "evaluate term" command
- **Diagnostics**: converts language errors via `IntoDiagnostics` trait, maps severity (Bug/Warning -> WARNING, Error -> ERROR, Note -> INFORMATION, Help -> HINT), cross-file labels go to `relatedInformation`

**Background analysis**: supervisor-worker pattern with dedicated threads. Diagnostics run in isolated subprocesses (not in-process) to prevent memory leaks and enable cancellation. Uses priority-based LIFO queuing (active edits get priority). Timeout mechanism kills long-running evaluations.

**Testing**: test harness spawns the LSP as a subprocess, communicates via JSON-RPC, uses snapshot testing against reference output. Supports request cancellation testing.

---

## LSP Server Architecture Patterns

### Document Management

Every LSP server must track open documents. The standard pattern:

1. **`didOpen`**: store document text + version in memory map
2. **`didChange`**: apply edits (full replacement or incremental patches)
3. **`didClose`**: remove from memory map, revert to filesystem version
4. **Version tracking**: each change increments version; responses can include version to detect staleness

Nickel's approach: `World` struct with `SourceCache` for file contents, `AnalysisRegistry` for cached analysis per file, bidirectional import dependency tracking for cascading invalidation.

### Incremental Analysis

What to recompute on each edit:

1. **Reparse changed file** -- always necessary
2. **Invalidate dependents** -- files that import the changed file
3. **Recompute diagnostics** -- for changed file and invalidated dependents
4. **Update symbol index** -- if top-level declarations changed

Optimization strategies:
- **Lazy recomputation**: only reanalyze files when their results are queried
- **Priority queuing**: prioritize the active editor file over background files
- **Coalescing**: batch rapid edits before triggering analysis
- **Cancellation**: abort in-flight analysis when new edits arrive

### Request Cancellation and Prioritization

From rust-analyzer's architecture:
- Register each request with a timestamp on arrival
- Latency-sensitive requests (completion, semantic tokens) get priority
- Heavy analysis runs on worker threads with read-only snapshots
- When input changes, check Salsa revision to detect cancellation
- Always respond to cancelled requests (with error code -32800)

### Threading Models

| Model | Example | Trade-offs |
|-------|---------|------------|
| Single-threaded event loop | Nickel NLS (with background subprocess) | Simple, no sync issues, but heavy requests block |
| Main thread + worker pool | rust-analyzer | Responsive UI thread, parallel analysis, complex cancellation |
| Fully async (tokio) | tower-lsp servers | Good throughput, but requires careful state management |
| Subprocess isolation | Nickel diagnostics | Prevents memory leaks, enables hard timeouts, but IPC overhead |

### Symbol Index / Workspace-Wide Analysis

For cross-file features (workspace symbols, find references across files):
- Build an index mapping symbol names to locations across all workspace files
- Update index incrementally when files change
- Use the import graph to scope searches to relevant files
- Cache resolved symbols to avoid re-resolution on every query

---

## Rust LSP Frameworks

### tower-lsp

Source: https://github.com/ebkalderon/tower-lsp

Async-first LSP framework built on Tower (the Rust service abstraction from hyper/tonic).

**Core types**:
- `LanguageServer` trait: implement async methods for each LSP feature (`initialize`, `hover`, `completion`, etc.)
- `LspService`: wraps your `LanguageServer` impl, handles JSON-RPC dispatch
- `Server`: manages I/O over stdin/stdout or TCP
- `Client`: enables server-initiated messages (publish diagnostics, log messages, show messages)

**Getting started**:
```rust
struct Backend { client: Client }

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult { capabilities: ServerCapabilities { ... }, ..Default::default() })
    }
    async fn hover(&self, _: HoverParams) -> Result<Option<Hover>> { ... }
    async fn shutdown(&self) -> Result<()> { Ok(()) }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let (service, socket) = LspService::new(|client| Backend { client });
    Server::new(stdin, stdout, socket).serve(service).await;
}
```

**Characteristics**:
- Tokio by default; `runtime-agnostic` feature for other runtimes
- `#[tower_lsp::async_trait]` for async trait methods
- Built-in JSON-RPC handling -- you only write language logic
- Tower middleware ecosystem available for request/response processing
- `proposed` feature flag for LSP 3.18 experimental features
- Boilerplate template: `tower-lsp-boilerplate` for quick start
- ~1.3k GitHub stars, actively maintained

**When to use**: when you want async-first design and minimal boilerplate. Good for new LSP servers.

### lsp-server

Used by rust-analyzer and Nickel. Lower-level than tower-lsp.

**Design philosophy**: thin wrapper around JSON-RPC transport. You manage the event loop, request dispatching, and state yourself. This gives maximum control but more boilerplate.

**Characteristics**:
- Synchronous, crossbeam-channel-based
- You write the main loop and dispatch logic
- Pairs with `lsp-types` for type definitions
- Used by rust-analyzer, Nickel NLS, and other production servers
- More control over threading, cancellation, and prioritization
- No async runtime dependency

**When to use**: when you need fine-grained control over the event loop, threading model, and cancellation. Better for complex servers with custom scheduling.

### lsp-types

Type definitions for LSP messages. Used by both `tower-lsp` and `lsp-server`. Provides Rust structs for all LSP requests, responses, notifications, and data types with serde serialization.

### Comparison

| Aspect | tower-lsp | lsp-server |
|--------|-----------|------------|
| Abstraction level | High (trait methods per LSP feature) | Low (raw JSON-RPC messages) |
| Threading | Tokio async | Synchronous + manual threading |
| Event loop | Managed by framework | You write it |
| Cancellation | Limited (trait method returns) | Full control |
| Middleware | Tower middleware ecosystem | None built-in |
| State management | Arc + Mutex patterns | Full control |
| Maturity | ~1.3k stars | Used by rust-analyzer |

**Recommendation for lx**: start with `lsp-server` + `lsp-types` (the Nickel pattern). It gives full control over the event loop, which matters for integrating with lx's interpreter. The Nickel codebase is the best reference implementation -- same problem space (simple language, Rust implementation, similar feature set).

---

## Semantic Analysis for IDE Features

How to go from AST to each IDE feature:

### Scope Analysis -> Go-to-Definition

Build a scope tree during parsing/binding:
1. Each scope (function body, block, module) maintains a symbol table mapping names to their declaration locations
2. Variable references resolve to declarations by walking up the scope chain
3. Go-to-definition = look up the reference's resolved declaration location

Nickel's approach: `UsageLookup` maintains definition table (span -> environment), usage table (definition span -> reference locations), and symbol table (all definitions). On `Var` node encounter, looks up definition in current environment.

### Type Info -> Hover

1. Run type inference/checking on the file
2. Store inferred types indexed by AST node or span
3. On hover request, find the AST node at the cursor position, look up its type
4. Format as markdown with type signature + documentation

Nickel's approach: `TypeCollector` visitor walks typechecked AST, populates `CollectedTypes` with term and identifier type mappings. Hover handler queries `analysis_reg.get_type(ast)` and `get_ident_type(&ident)`.

### Symbol Table -> Document Symbols

Walk the AST, collecting top-level and nested declarations:
- Function definitions -> `SymbolKind::Function`
- Variable bindings -> `SymbolKind::Variable`
- Type definitions -> `SymbolKind::Struct` / `SymbolKind::TypeParameter`
- Nested symbols become children in the hierarchy

### Scope + Type -> Completion

At the cursor position, determine:
1. What scope the cursor is in
2. What symbols are visible in that scope
3. What the expected type is (if inferrable from context)
4. Filter/rank completions by relevance

Context-specific completion (from Nickel):
- Record field access (`.foo`) -> resolve record type, list fields
- Import paths -> filesystem listing
- Parse error recovery -> reparse incomplete input around cursor
- Environment fallback -> all visible symbols

### Usage Tracking -> Find References / Rename

1. Build a bidirectional map: definition <-> list of references
2. Find references = look up all references for a definition
3. Rename = find references + generate TextEdits replacing each occurrence
4. Cross-file: group edits by file URI into WorkspaceEdit

### Error Recovery -> Diagnostics

The parser must continue after errors to provide diagnostics for the whole file:
1. On parse error, skip tokens until a synchronization point (statement boundary, closing delimiter)
2. Record the error with source span
3. Continue parsing from the synchronization point
4. Convert accumulated errors to LSP Diagnostic objects with severity, range, and message

---

## IDE-Oriented Compiler Architecture

Source: matklad (rust-analyzer author) -- https://matklad.github.io/2022/04/25/why-lsp.html

### Three Challenges Beyond Batch Compilers

1. **Incomplete program analysis**: unlike compilers that reject invalid code, IDE servers must analyze any invalid program as best they can. Error recovery is not optional.

2. **Stateful evolution**: servers operate on continuously-modified codebases, introducing time-dimension state management. Incremental computation is necessary.

3. **Latency optimization**: sub-100ms response times require fundamental architectural redesign. Optimization is insufficient -- the architecture must be built for low latency from the start.

### Key Insight: Rewrites Win

Most successful language servers are rewrites or alternative implementations of batch compilers: IntelliJ, Eclipse, Roslyn (C#), rust-analyzer. Trying to adapt a batch compiler for IDE use rarely works well. Design the analysis pipeline for IDE use from the beginning.

### Error-Recovering Parser Design

For IDE support, the parser must:
- Never crash or return nothing on invalid input
- Produce a partial tree covering the valid portions
- Record error nodes in the tree for diagnostics
- Preserve all source text (lossless CST) for exact position mapping and formatting

### LSP Protocol Design Critique

From matklad (https://matklad.github.io/2023/10/12/lsp-could-have-been-better.html):

1. **Transport overhead**: custom HTTP-like headers + JSON-RPC boilerplate. Simpler: newline-delimited JSON.
2. **Causality loss**: edits are notifications (no response), so the server can't know if a `workspace/applyEdit` from the server accounts for the latest client edit. Fix: make edits request-based.
3. **Request vs state model**: features like diagnostics and highlighting are queries, not synchronized state. Leads to stale data or wasteful re-querying. Fix: subscription-based model.
4. **UTF-16 position encoding**: historical artifact from VS Code. Recommendation: negotiate UTF-8.
5. **No multi-step refactoring support**: protocol can't handle interactive refactorings like "change function signature".

**Practical advice for implementors**: treat LSP as a serialization format, not an internal data model. Maintain rich internal semantic models distinct from presentation layers.

---

## Incremental Computation

### Salsa (Rust)

Source: https://salsa-rs.github.io/salsa/

Framework for writing incremental, on-demand programs. Used by rust-analyzer.

**Core concepts**:

- **Database**: central store persisting computation values across executions. All Salsa structs are newtyped integer IDs; actual data lives in the database.

- **Input structs** (`#[salsa::input]`): define program starting points. Everything else derives deterministically from inputs. Fields accessed via getters with `&db`; modified via setters with `&mut db`.

- **Tracked functions** (`#[salsa::tracked]`): memoized computations. Must accept `&db` as first parameter. Salsa records dependencies and returns cached results if inputs haven't changed (red-green algorithm).

- **Tracked structs** (`#[salsa::tracked]`): intermediate computation results, immutable, database-backed. `#[id]` annotation helps match struct instances across executions.

- **Interned structs** (`#[salsa::interned]`): canonical value copies for cheap equality (e.g., identifiers). Two interned structs with same fields get the same integer ID.

- **Accumulators** (`#[salsa::accumulator]`): side-channel for collecting diagnostics/errors during computation.

**Red-green algorithm**: when inputs change, Salsa walks the dependency graph. For each memoized function, it checks if inputs changed. If the function's output is the same as before despite input changes, it stops propagation -- downstream dependents don't need recomputation. This is the key to fine-grained incrementality.

**Relevance to lx**: Salsa is powerful but adds significant complexity. For a simple language like lx, the Nickel approach (targeted cache invalidation via import graph, re-parse changed files) may be sufficient initially. Salsa becomes worthwhile when the language has complex cross-file interactions (macros, type inference across modules).

### Roslyn (C#)

Microsoft's .NET compiler platform, designed IDE-first:
- Immutable syntax trees with incremental reparsing
- Semantic model lazily computed on demand
- Workspace abstraction tracks solution/project/document hierarchy
- Red-green trees: external immutable red nodes wrap internal shared green nodes, enabling tree reuse across edits

### Demand-Driven Computation

The general pattern behind Salsa, Roslyn, and modern IDE compilers:
1. Define computations as pure functions of their inputs
2. Memoize results keyed by inputs
3. Track dependencies between computations
4. On input change, invalidate the minimum set of cached results
5. Recompute lazily when results are queried

---

## Minimum Viable LSP for Simple Languages

### What to Implement First (ROI Order)

1. **Diagnostics** (highest ROI) -- users need to see errors. Wire up the parser's error output to `publishDiagnostics`. Minimal effort, maximum impact.

2. **Document sync** (required for everything) -- start with Full sync (send entire document on change). Incremental sync is optimization for later.

3. **Go-to-definition** -- requires scope analysis / name resolution. High-value navigation feature.

4. **Hover** -- show type info and documentation. Requires type information at cursor positions.

5. **Completion** -- in-scope symbol completion. Requires scope analysis. Start with simple scope-based completion, add context-awareness later.

6. **Document symbols** -- outline view. Walk AST for top-level declarations.

7. **Find references** -- requires building a usage table (definition -> references bidirectional map).

8. **Formatting** -- if the language has a formatter, wire it up via full-document TextEdit.

9. **Rename** -- find references + generate edits. Cross-file rename requires WorkspaceEdit.

10. **Semantic tokens** -- syntax-aware highlighting. Medium effort, nice visual improvement.

### Architecture Recommendation for lx

Based on the Nickel pattern (most similar problem space):

1. Use `lsp-server` + `lsp-types` crates
2. Main event loop with `crossbeam::select` multiplexing client messages and background results
3. `World` struct holding file contents + analysis cache + import dependencies
4. `PositionLookup` for cursor -> AST node mapping (sorted disjoint intervals)
5. `UsageLookup` for definition <-> reference tracking
6. Background diagnostics in separate thread (or subprocess for isolation)
7. Full document sync initially
8. Priority-based queuing for active file vs background analysis

### Key Dependencies (Rust)

| Crate | Purpose |
|-------|---------|
| `lsp-server` | JSON-RPC transport and connection management |
| `lsp-types` | LSP message type definitions |
| `serde` / `serde_json` | Serialization |
| `crossbeam` | Channels for event loop multiplexing |
| `codespan` / `codespan-reporting` | Diagnostic formatting with source spans |
| `log` / `env_logger` | Logging |

---

## Sources

- LSP 3.17 Specification: https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/
- rust-analyzer architecture: https://rust-analyzer.github.io/blog/2020/07/20/three-architectures-for-responsive-ide.html
- Pyright internals: https://github.com/microsoft/pyright/blob/main/docs/internals.md
- gopls design: https://go.googlesource.com/tools/+/refs/heads/master/gopls/doc/design/implementation.md
- lua-language-server: https://github.com/LuaLS/lua-language-server
- ElixirLS: https://github.com/elixir-lsp/elixir-ls
- Nickel LSP: https://github.com/nickel-lang/nickel/tree/master/lsp
- tower-lsp: https://github.com/ebkalderon/tower-lsp
- Salsa: https://salsa-rs.github.io/salsa/
- matklad on LSP: https://matklad.github.io/2022/04/25/why-lsp.html
- matklad LSP critique: https://matklad.github.io/2023/10/12/lsp-could-have-been-better.html
