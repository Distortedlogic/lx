# Hybrid Search & Code RAG

## Why Hybrid Search

Neither sparse (keyword) nor dense (semantic) retrieval alone is optimal for code:

- **BM25 (sparse)**: Excels at exact identifier matching (`HashMap`, `process_data`, `impl Trait`). Fast. No GPU needed.
- **Dense embeddings (semantic)**: Excels at NL-to-code queries ("find authentication logic"). Understands intent, not just tokens.

## Key Paper: "Practical Code RAG at Scale"

**arXiv**: [2510.20609](https://arxiv.org/abs/2510.20609)

Compares retrieval configurations across chunking strategy, similarity scoring, and splitting granularity.

### Key Findings

1. **Code-to-code retrieval**: BM25 with word-level splitting is most effective AND 100x faster than dense alternatives
2. **NL-to-code retrieval**: Dense encoders (Voyage-3 family) consistently beat sparse retrievers
3. **Optimal chunk size scales with context budget**: 32-64 lines at small budgets, whole-file at 16K tokens
4. **BPE-based splitting is needlessly slow** -- word-level splitting is equally effective
5. **Retrieval latency varies up to 200x** across configurations

## Architecture: Two-Stage Retrieval + Reranking

### Stage 1: Candidate Retrieval (parallel)

1. **BM25**: Keyword-based over tokenized code chunks. Use word-level splitting (not BPE).
2. **Dense vector search**: Cosine similarity over code embeddings.

Each returns 100-200 candidates.

### Stage 2: Score Fusion (RRF)

`RRF_score(d) = sum(1 / (k + rank_i(d)))` for each retriever i. k=60 typically. Simple, effective, no tuning required.

### Stage 3: Reranking (Optional)

Cross-encoder reranker on top N fused candidates. Much more accurate than bi-encoder but too slow for first stage. BERT-based reranker improves 7.3% MRR@10 when BM25 scores are appended as text tokens.

## Task-Aware Strategy

| Task Type | Best Retriever | Rationale |
|---|---|---|
| **Code-to-code** (completion, similar code) | BM25 + word splitting | Exact identifier matching; 100x faster |
| **NL-to-code** (search by description) | Dense (Voyage-3 family) | Semantic understanding needed |

## BM25 Implementation Notes

- Word-level tokenization, not BPE
- Code-aware: split camelCase/snake_case boundaries
- Index imports, function signatures, type definitions separately for boosted matching
- Parameters: k1=1.2, b=0.75 are reasonable defaults

## End-to-End Pipeline

### Indexing

Source files -> tree-sitter parse -> chunk at semantic boundaries -> enrich with metadata -> generate embeddings -> store in vector DB + build BM25 index

### Retrieval

Query -> BM25 (top 100-200) + dense (top 100-200) -> RRF fusion -> optional reranking -> top K results

### Hierarchical Indexing (Large Codebases)

**Tier 1 (Coarse)**: File-level summaries (exports, types, public API, dependencies)
**Tier 2 (Fine)**: Chunk-level (functions, structs, impls with metadata)

Search: query -> tier 1 to narrow files -> tier 2 for specific chunks.

## Context Preservation

When chunking, preserve critical context:
- Class/struct header with every method chunk
- Relevant import statements per chunk
- File path / module hierarchy
- Type context for method chunks (the struct/trait being implemented)

## ColBERT Late Interaction

Alternative to bi-encoder + cross-encoder: per-token embeddings, MaxSim at search time. Better quality than bi-encoder, faster than cross-encoder.

## References

- [Practical Code RAG at Scale (arXiv)](https://arxiv.org/abs/2510.20609)
- [Hybrid Search with Qdrant](https://qdrant.tech/articles/hybrid-search/)
- [Building RAG on Codebases (LanceDB)](https://lancedb.com/blog/building-rag-on-codebases-part-1/)
- [How to Build Custom Code RAG (Continue)](https://docs.continue.dev/guides/custom-code-rag)
- [RAG for Large Scale Code Repos (Qodo)](https://www.qodo.ai/blog/rag-for-large-scale-code-repos/)
- [Integrating BM25 in Hybrid Search (DEV)](https://dev.to/negitamaai/integrating-bm25-in-hybrid-search-and-reranking-pipelines-strategies-and-applications-4joi)
- [Advanced RAG: Hybrid Search and Re-ranking (DEV)](https://dev.to/kuldeep_paul/advanced-rag-from-naive-retrieval-to-hybrid-search-and-re-ranking-4km3)
