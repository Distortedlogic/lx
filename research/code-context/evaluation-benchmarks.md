# Evaluation & Benchmarks for Code RAG

## Why Evaluation Matters

Without measuring retrieval quality, we can't tell if our chunking, embedding, or search pipeline actually works. Evaluation happens at two levels: **retrieval** (did we find the right code?) and **end-to-end** (did the LLM produce correct output with our context?).

## Retrieval Metrics

### Recall@K

Measures coverage: of all relevant chunks, how many appeared in the top K results?

`Recall@K = |relevant ∩ retrieved_top_k| / |relevant|`

Most important metric for RAG -- if relevant code isn't retrieved, the LLM can't use it. The cAST paper reports **Recall@5** as its primary retrieval metric.

### Precision@K

Measures noise: of the top K results, how many are actually relevant?

`Precision@K = |relevant ∩ retrieved_top_k| / K`

Less critical for RAG (LLMs can ignore irrelevant context) but matters for token budget efficiency.

### MRR (Mean Reciprocal Rank)

Measures where the first relevant result appears.

`MRR = mean(1 / rank_of_first_relevant_result)`

Important when the user needs one specific answer (e.g., "find the definition of X").

### NDCG@K (Normalized Discounted Cumulative Gain)

Measures ranking quality with graded relevance. Higher weight to relevant results at higher positions. Normalized so ideal ranking = 1.0.

Best when some results are more relevant than others (e.g., exact definition vs related code).

### MAP@K (Mean Average Precision)

Considers both precision and recall across all K positions. Penalizes relevant results that appear lower in the ranking.

## Code-Specific Benchmarks

### CoIR (Code Information Retrieval)

- **Paper**: [arxiv.org/abs/2407.02883](https://arxiv.org/abs/2407.02883)
- **Published**: ACL 2025 Main
- **Repo**: [github.com/CoIR-team/coir](https://github.com/CoIR-team/coir)
- **Site**: [archersama.github.io/coir](https://archersama.github.io/coir/)

**What it covers:**
- 10 curated datasets across 8 retrieval tasks and 7 domains
- 2M+ entries
- Same schema as MTEB and BEIR (cross-benchmark compatible)

**Task types:**
| Task | Description |
|---|---|
| Text-to-Code | NL query -> find relevant code |
| Code-to-Code | Code snippet -> find similar/related code |
| Code-to-Text | Code -> find documentation/description |
| Hybrid Code Retrieval | Mixed queries |

**Sub-tasks:** code contest retrieval, web query code retrieval, text-to-SQL, code summary, code context, similar code, single-turn code QA, multi-turn code QA.

**Why it matters for us:** CoIR directly measures what our MCP server does -- retrieving relevant code given a query. We can evaluate our embedding + chunking pipeline against CoIR baselines.

### RepoEval

- **Reference**: Zhang et al. (2023)
- **Focus**: Project-oriented code completion

Provides realistic coding scenarios within full repositories. The cAST paper reports Recall@5 on RepoEval (+4.3 points over baselines).

### SWE-bench

- **Site**: [swebench.com](https://www.swebench.com/original.html)
- **Paper**: ICLR 2024 ([arxiv.org/pdf/2310.06770](https://arxiv.org/pdf/2310.06770))
- **Metric**: Pass@1 (does the generated patch pass tests?)

**What it is:** 2,294 real GitHub issues across 12 Python repositories. Given a codebase + issue description, generate a patch that resolves it.

**Why it matters:** End-to-end benchmark -- measures whether better retrieval translates to better code generation. The cAST paper reports Pass@1 on SWE-bench (+2.67 points).

**Variants:**
- SWE-bench Lite: 300 representative problems
- SWE-bench Verified: human-validated subset
- SWE-bench Pro: long-horizon multi-step tasks

### Long Code Arena

Used by the "Practical Code RAG at Scale" paper (arXiv 2510.20609). Includes code completion and bug localization tasks within large repositories.

### DevEval

Used by the CodeRAG bigraph paper. The CodeRAG paper reports +40.90 Pass@1 on GPT-4o using DevEval.

## Evaluation Strategy for Our MCP Server

### Offline Evaluation (Before Deployment)

1. **Chunking quality**: compare our AST chunks vs naive line-based chunks on CoIR retrieval tasks
2. **Embedding quality**: compare different models (fastembed BGE vs Voyage-code-3 API) on the same chunks
3. **Search quality**: measure Recall@5 and MRR for hybrid search vs BM25-only vs dense-only
4. **End-to-end**: use SWE-bench Lite to see if our retrieval improves code generation

### Online Evaluation (During Use)

1. **Retrieval latency**: p50 and p99 query time
2. **Indexing throughput**: files/second during initial indexing
3. **Index freshness**: time from file save to updated index
4. **User feedback**: track which retrieved results the LLM actually uses (implicit relevance signal)

### Simple Self-Evaluation

For quick sanity checks without formal benchmarks:
- Query known functions by description, check if they appear in top 5
- Query by exact function name, check if definition ranks #1
- Query by error message, check if relevant error-handling code appears
- Modify a file, check if index updates within expected time

## References

- [CoIR Benchmark (arXiv)](https://arxiv.org/abs/2407.02883)
- [CoIR GitHub](https://github.com/CoIR-team/coir)
- [SWE-bench](https://www.swebench.com/original.html)
- [SWE-bench paper (ICLR 2024)](https://arxiv.org/pdf/2310.06770)
- [SWE-bench comprehensive review (Atoms)](https://atoms.dev/insights/swe-bench-a-comprehensive-review-of-its-fundamentals-methodology-impact-and-future-directions/6c3cb9820d3b44e69862f7b064c1fd1e)
- [RAG Evaluation Metrics Guide 2025](https://futureagi.com/blogs/rag-evaluation-metrics-2025)
- [RAG Evaluation Complete Guide (Maxim)](https://www.getmaxim.ai/articles/complete-guide-to-rag-evaluation-metrics-methods-and-best-practices-for-2025/)
- [Evaluation Metrics for Information Retrieval (Pinecone)](https://www.pinecone.io/learn/offline-evaluation/)
- [RAG Evaluation Metrics (Confident AI)](https://www.confident-ai.com/blog/rag-evaluation-metrics-answer-relevancy-faithfulness-and-more)
- [Retrieval Evaluation Metrics (Weaviate)](https://weaviate.io/blog/retrieval-evaluation-metrics)
