# Existing Code Context MCP Servers

A survey of existing MCP servers for code search and context to understand the landscape.

## Code-Sage (Rust)

**Repo**: [github.com/faxioman/code-sage](https://github.com/faxioman/code-sage)

- High-performance MCP server for semantic code search, **written in Rust**
- Hybrid search: BM25 (keyword) + vector embeddings (semantic) with RRF reranking
- Uses **tree-sitter** for intelligent code splitting into semantic units
- Most directly comparable to what we're building

## Code Context by Foad K

**Listing**: [pulsemcp.com/servers/code-context](https://www.pulsemcp.com/servers/code-context)

- Persistent intelligence infrastructure for codebase analysis
- Hybrid Rust-TypeScript architecture
- Local semantic code search using **EmbeddingGemma** embeddings
- **FAISS** vector indexing
- Intelligent AST-based chunking

## Claude Context by Zilliz

**Repo**: [github.com/zilliztech/claude-context](https://github.com/zilliztech/claude-context)

- Code search MCP specifically for Claude Code
- Indexes codebase directory for hybrid search (BM25 + dense vector)
- Natural language queries with hybrid search
- Makes entire codebase available as context

## Augment Context Engine MCP

**Docs**: [docs.augmentcode.com/context-services/mcp/overview](https://docs.augmentcode.com/context-services/mcp/overview)

- Commercial, hosted service
- Industry-leading semantic search
- 70%+ agent performance improvement
- Runs locally via Auggie CLI or connects to hosted service
- See [augment-code-context-engine.md](augment-code-context-engine.md) for deep dive

## CodeGraph MCP (Rust)

**Listing**: [pulsemcp.com/servers/jakedismo-codegraph-rust](https://www.pulsemcp.com/servers/jakedismo-codegraph-rust)

- Graph-based code understanding
- Written in Rust
- Focuses on code relationships and dependencies

## Rust Analyzer Tools MCP

**Listing**: [pulsemcp.com/servers/terhechte-rust-analyzer-tools](https://www.pulsemcp.com/servers/terhechte-rust-analyzer-tools)

- Wraps rust-analyzer as MCP tools
- Provides IDE-level code intelligence (go-to-definition, find-references, etc.)
- Rust-specific but shows how to expose code intelligence via MCP

## Seroost

- Semantic code search engine written in Rust
- TF-IDF based indexing and search
- Fast, snippet-aware, handles large directories
- Can be wrapped as MCP server

## Community Context Engine

**Repo**: [github.com/Kirachon/context-engine](https://github.com/Kirachon/context-engine)

- MCP Server for semantic code search
- Uses Augment SDK for AI-powered prompt enhancement
- Open-source implementation inspired by Augment

## Common Patterns Across Implementations

1. **Tree-sitter** is the universal choice for code parsing
2. **Hybrid search** (BM25 + dense vectors) is standard
3. **RRF** is the dominant fusion strategy
4. Most use **local embedding models** for privacy
5. MCP **stdio transport** for local integration with editors/agents
6. **Incremental indexing** on file change is expected

## Gaps We Can Fill

- Most implementations are Python or TypeScript; few are pure Rust
- Metadata enrichment (scope chain, imports, siblings) is rarely implemented
- Few implementations expose structured symbol information alongside search
- Real-time incremental re-indexing via tree-sitter's incremental parsing is underutilized

## References

- [Code-Sage](https://github.com/faxioman/code-sage)
- [Claude Context](https://github.com/zilliztech/claude-context)
- [Code Context MCP](https://www.pulsemcp.com/servers/code-context)
- [Augment Context Engine MCP](https://docs.augmentcode.com/context-services/mcp/overview)
- [CodeGraph Rust MCP](https://www.pulsemcp.com/servers/jakedismo-codegraph-rust)
- [Rust Analyzer Tools MCP](https://www.pulsemcp.com/servers/terhechte-rust-analyzer-tools)
- [Community Context Engine](https://github.com/Kirachon/context-engine)
