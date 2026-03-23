# System Architecture & Concurrency

How to structure the MCP server's async pipeline: file watching, indexing, embedding, and query serving.

## Architecture Overview

```
┌─────────────────────────────────────────────────────┐
│                   MCP Server (stdio)                │
│                                                     │
│  ┌──────────┐    ┌───────────┐    ┌──────────────┐ │
│  │  File     │───>│  Indexer   │───>│  Embedding   │ │
│  │  Watcher  │    │  (AST +   │    │  Worker      │ │
│  │  (notify) │    │  Chunker) │    │  (fastembed)  │ │
│  └──────────┘    └───────────┘    └──────┬───────┘ │
│                                          │         │
│                                          v         │
│  ┌──────────┐    ┌───────────┐    ┌──────────────┐ │
│  │  MCP      │<───│  Query    │<───│  Vector DB   │ │
│  │  Handler  │    │  Engine   │    │  + BM25      │ │
│  │  (tools)  │    │  (hybrid) │    │  Index       │ │
│  └──────────┘    └───────────┘    └──────────────┘ │
│                                                     │
│  ┌──────────────────────────────────────────────┐  │
│  │              Symbol Index (in-memory)         │  │
│  └──────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────┘
```

## Component Responsibilities

### File Watcher

- Uses `notify` crate with `notify-debouncer-mini` (500ms)
- Watches workspace directory recursively
- Filters by file extension (`.rs`, `.toml`, etc.) and respects `.gitignore`
- Sends `FileEvent { path, kind: Created | Modified | Deleted }` to indexer

### Indexer

- Receives file events from watcher
- Parses files with tree-sitter (incremental parse if cached tree exists)
- Chunks AST into semantic units
- Extracts symbols (definitions + references) in the same pass
- Computes content hashes to detect actual changes
- Sends new/modified chunks to embedding worker
- Updates symbol index directly (in-memory, fast)
- Deletes stale chunks from vector DB and BM25 index

### Embedding Worker

- Receives chunks from indexer
- Generates embeddings via fastembed (or API)
- Batches chunks for throughput (e.g., 32 at a time)
- Upserts embeddings + metadata into vector DB
- Updates BM25 index with chunk text

### Query Engine

- Receives search queries from MCP handler
- Runs hybrid search: BM25 + dense vector in parallel
- Fuses results with RRF
- Assembles context (dedup, expand, group by file)
- Returns structured results

### MCP Handler

- Implements MCP protocol over stdio
- Exposes tools: `search_code`, `get_definition`, `find_references`, `list_symbols`
- Validates input, formats output
- Stateless per-request (all state in shared indexes)

### Symbol Index

- In-memory `HashMap`-based index
- Updated synchronously by indexer (fast, no embedding needed)
- Provides O(1) lookup by name, file, or symbol ID
- Cross-reference edges for callers/callees

## Concurrency Model

### Option A: Tokio Tasks + Channels (Recommended)

Simple, well-understood, sufficient for a single-user dev tool.

```
File Watcher ──(mpsc)──> Indexer ──(mpsc)──> Embedding Worker
                              │                      │
                              v                      v
                        Symbol Index            Vector DB
                              │                      │
                              └────────┬─────────────┘
                                       │
MCP Handler ──(oneshot)──> Query Engine ──(read)──> both indexes
```

**Channel types:**
- `tokio::sync::mpsc` (bounded) between watcher -> indexer -> embedder
- `tokio::sync::oneshot` for query request/response
- `tokio::sync::RwLock` on shared indexes (symbol index, vector DB handle)

**Bounded channels provide backpressure:**
- Watcher -> Indexer channel (capacity ~100): if indexer is slow, watcher blocks
- Indexer -> Embedder channel (capacity ~50): if embedding is slow, indexer blocks
- Prevents unbounded memory growth during large operations (git checkout, initial indexing)

### Option B: Actor Model

Heavier but more structured for complex pipelines. Quickwit uses this for their indexing pipeline.

**Quickwit's actor framework pattern:**
- Each pipeline stage is an actor with its own mailbox
- Dual-queue: high-priority (control messages, timeouts) + low-priority (data)
- KillSwitch for cascading shutdown
- ActorRegistry for observability
- Sync actors run on blocking runtime, async actors on tokio runtime

**When to use:** If the pipeline grows complex (multiple embedding models, reranking, graph building). For our initial MCP server, Option A is sufficient.

### Option C: Rayon for CPU-Bound Parallelism

For initial bulk indexing, use rayon to parallelize across files:
- Parse all files in parallel (tree-sitter is thread-safe per parser instance)
- Chunk in parallel
- Batch embed and insert

Then switch to the channel-based pipeline for ongoing incremental updates.

## State Management

### Shared State

| State | Type | Access Pattern |
|---|---|---|
| Symbol Index | `Arc<RwLock<SymbolIndex>>` | Write: indexer. Read: query engine |
| Vector DB handle | `Arc<VectorDb>` | Write: embedder. Read: query engine |
| BM25 Index | `Arc<RwLock<BM25Index>>` | Write: embedder. Read: query engine |
| AST Cache | `HashMap<PathBuf, Tree>` | Write: indexer only (no sharing needed) |
| File Hashes | `HashMap<PathBuf, u64>` | Write: indexer only |

### Consistency

The system is **eventually consistent**:
- After a file edit, there's a brief window (500ms debounce + indexing + embedding time) where queries return stale results
- This is acceptable for a dev tool -- sub-second staleness is fine
- The symbol index updates faster than the vector index (no embedding needed)

## Pipeline Lifecycle

### Startup

1. Start MCP server, begin accepting connections
2. Scan workspace directory for all source files
3. Bulk index: parse, chunk, embed in parallel (rayon)
4. Build symbol index and BM25 index
5. Start file watcher for incremental updates
6. Report "ready" to client

### Steady State

1. File watcher detects changes (debounced)
2. Indexer processes changed files incrementally
3. Embedding worker updates vector DB
4. Query engine serves MCP tool calls using latest indexes

### Shutdown

1. MCP connection closes
2. Stop file watcher
3. Drain indexer and embedder channels
4. Persist index to disk (optional, for faster restart)
5. Clean up

## Error Handling

| Failure | Response |
|---|---|
| File read error | Skip file, log warning, retry on next change |
| Parse error | Skip file (tree-sitter is error-tolerant, so this is rare) |
| Embedding error | Retry with backoff; if persistent, skip chunk and log |
| Vector DB error | Queue failed operations for retry |
| MCP protocol error | Return error response, keep server running |

Key principle: **never crash the server**. Individual file/chunk failures should not take down the pipeline. Log errors, skip the problematic item, and continue.

## Performance Targets

| Metric | Target | Rationale |
|---|---|---|
| Initial indexing | <30s for 10K files | Bulk parallel with rayon |
| Incremental update | <2s from save to indexed | 500ms debounce + parse + embed |
| Query latency (p50) | <100ms | Hybrid search + assembly |
| Query latency (p99) | <500ms | Complex queries with expansion |
| Memory usage | <500MB for 50K file codebase | Disk-backed vector DB |

## Key Crates

| Crate | Purpose |
|---|---|
| `rust-mcp-sdk` | MCP protocol implementation |
| `tree-sitter` + `tree-sitter-rust` | AST parsing |
| `notify` + `notify-debouncer-mini` | File watching |
| `fastembed` | Local embedding inference |
| `tantivy` | BM25 full-text search |
| `lancedb` or `qdrant-client` | Vector storage and search |
| `tokio` | Async runtime |
| `rayon` | Parallel bulk indexing |
| `petgraph` | Optional graph index |

## Reference Architectures

### Quickwit (Search Engine)

- Actor-based indexing pipeline in Rust
- Bounded MPSC channels for backpressure
- Dual-queue mailboxes (high/low priority)
- KillSwitch for cascading shutdown
- Sync actors on blocking runtime, async on tokio
- [Blog post](https://quickwit.io/blog/quickwit-actor-framework)

### Frankensearch (Hybrid Search)

- Two-tier hybrid search: fast initial results + quality refinement
- Tantivy BM25 + vector cosine with RRF fusion
- Progressive iterator API for streaming results
- f16 SIMD vector index, memory-mapped storage
- [GitHub](https://github.com/Dicklesworthstone/frankensearch)

### Tantivy (Full-Text Search)

- Segment-based index (multiple immutable segments)
- Concurrent search by splitting across segments
- Merge policy for compacting segments
- [GitHub](https://github.com/quickwit-oss/tantivy)

### ygrep (Code Search for AI)

- Rust + Tantivy for full-text code indexing
- Optimized for AI coding assistant queries
- [GitHub](https://github.com/yetidevworks/ygrep)

## References

- [Quickwit actor framework blog](https://quickwit.io/blog/quickwit-actor-framework)
- [Quickwit actors crate](https://lib.rs/crates/quickwit-actors)
- [Tokio channels tutorial](https://tokio.rs/tokio/tutorial/channels)
- [Async Rust backpressure (Biriukov)](https://biriukov.dev/docs/async-rust-tokio-io/1-async-rust-with-tokio-io-streams-backpressure-concurrency-and-ergonomics/)
- [Backpressure with bounded channels (Sling Academy)](https://www.slingacademy.com/article/handling-backpressure-in-rust-async-systems-with-bounded-channels/)
- [Actor model in async Rust (O'Reilly)](https://www.oreilly.com/library/view/async-rust/9781098149086/ch08.html)
- [Frankensearch](https://github.com/Dicklesworthstone/frankensearch)
- [Tantivy](https://github.com/quickwit-oss/tantivy)
- [ygrep](https://github.com/yetidevworks/ygrep)
