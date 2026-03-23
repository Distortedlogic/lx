# Vector Databases for Code Search

## Requirements for Code Context MCP

- **Local/embedded** preferred (privacy, no network dependency)
- **Rust native** or has Rust bindings
- **Incremental updates** (add/remove/update chunks as files change)
- **Hybrid search** (dense vectors + sparse/keyword)
- **Metadata filtering** (filter by file path, language, symbol type)

## Comparison

### Qdrant

- **Written in**: Rust
- **Algorithm**: HNSW (Hierarchical Navigable Small World)
- **Deployment**: Server (self-hosted or cloud), also has embedded mode via `qdrant-client`
- **Performance**: 20-30ms query time, ~95% recall
- **Memory**: Medium-high footprint
- **Hybrid search**: Native support for dense + sparse vectors with RRF fusion
- **Metadata filtering**: Full payload filtering with indexes
- **Incremental updates**: Yes, via upsert/delete by point ID
- **Rust client**: [qdrant/rust-client](https://github.com/qdrant/qdrant)
- **Used by**: Roo Code for codebase indexing, Kilo Code

**Strengths**: Production-grade, best recall, native hybrid search, written in Rust
**Weaknesses**: Higher memory usage, requires running a server process (unless embedded mode)

### LanceDB

- **Written in**: Rust
- **Algorithm**: IVF_PQ (Inverted File Index + Product Quantization)
- **Deployment**: Embedded (runs in-process, no server needed)
- **Performance**: 40-60ms query time, ~88% recall
- **Memory**: Low footprint
- **Hybrid search**: Supported with BM25 full-text index
- **Storage**: Apache Lance columnar format, disk-based
- **Incremental updates**: Yes
- **Rust SDK**: Native
- **Used by**: Continue.dev for codebase indexing

**Strengths**: Zero-config embedded, low memory, disk-based (handles large codebases), native Rust
**Weaknesses**: Lower recall than Qdrant, IVF_PQ less accurate than HNSW

### Other Options

#### tinyvec / usearch (Ultra-lightweight)
- In-process, header-only style
- Good for small codebases
- No hybrid search

#### SQLite + sqlite-vss
- Familiar, embedded, but vector search is an extension
- Limited performance at scale

#### pgvector (PostgreSQL)
- Full SQL support, but heavyweight for an MCP server
- Not suitable for embedded/local use

## Recommendation

For a code context MCP server:

1. **LanceDB** for embedded/local-first approach: no server process, low memory, disk-based storage, native Rust, hybrid search support. Best for single-user dev tool.

2. **Qdrant** if higher recall is needed or serving multiple users: better accuracy, native hybrid search with RRF, but requires a server process.

Both are written in Rust and have excellent Rust APIs.

## References

- [Qdrant GitHub](https://github.com/qdrant/qdrant)
- [LanceDB](https://lancedb.com/)
- [LanceDB vs Qdrant comparison (Medium)](https://medium.com/@plaggy/lancedb-vs-qdrant-caf01c89965a)
- [LanceDB vs Qdrant for Conversational AI (Medium)](https://medium.com/@vinayak702010/lancedb-vs-qdrant-for-conversational-ai-vector-search-in-knowledge-bases-793ac51e0b81)
- [Vector databases in Rust (forum)](https://users.rust-lang.org/t/vector-databases-in-rust/96514)
- [Roo Code codebase indexing docs](https://docs.roocode.com/features/codebase-indexing)
- [Continue.dev + LanceDB](https://lancedb.com/blog/the-future-of-ai-native-development-is-local-inside-continues-lancedb-powered-evolution/)
- [Scale Up RAG with Rust + LanceDB (Medium)](https://medium.com/data-science/scale-up-your-rag-a-rust-powered-indexing-pipeline-with-lancedb-and-candle-cc681c6162e8)
