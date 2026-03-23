# Context Assembly & Formatting

Retrieving good chunks is only half the problem. Assembling them into a coherent, non-redundant context window for the LLM is equally critical.

## The Redundancy Problem

In production RAG systems, **30-40% of retrieved context is semantically redundant**. Multiple chunks from the same file, overlapping scope chains, and repeated imports all waste context tokens.

## Assembly Pipeline

### Step 1: Retrieve Candidates

Run hybrid search (BM25 + dense vectors), get top N candidates (typically 50-200).

### Step 2: Deduplicate

**Content-level deduplication:**
- Hash each chunk's content (after stripping whitespace/comments)
- Remove exact or near-exact duplicates
- For overlapping chunks from the same file, merge into a single larger chunk

**Semantic deduplication:**
- Compute pairwise similarity between retrieved chunks
- If two chunks have >0.95 cosine similarity, keep only the higher-ranked one
- More expensive but catches reformulations

### Step 3: Score Fusion & Reranking

- Apply RRF or weighted scoring across retrieval methods
- Optional: cross-encoder reranking on top N candidates
- Select top K after reranking

### Step 4: Context Expansion

For code, isolated chunks often lack critical context. Expand by:

**Upward expansion:**
- If a method chunk is retrieved, include its parent class/struct definition
- If a function is retrieved, include its containing module's imports

**Sibling expansion:**
- If a method is retrieved, include other methods from the same impl block that are referenced by it
- Include the function signature of siblings (not full body) for overview

**Dependency expansion:**
- If the chunk uses types/functions from other files, include their definitions
- Follow the import chain (aider's approach: follow imports, find definitions, add with high priority)

### Step 5: Group by File

Sort chunks by file path, then by line number within each file. This produces a natural reading order that matches how developers think about code.

```
// === src/auth/mod.rs (lines 1-5, 20-45) ===
use crate::db::UserStore;
use crate::crypto::hash_password;

pub struct AuthService { ... }

impl AuthService {
    pub fn login(&self, ...) -> Result<Token> { ... }
}

// === src/db/user_store.rs (lines 10-30) ===
pub struct UserStore { ... }

impl UserStore {
    pub fn find_by_email(&self, ...) -> Result<User> { ... }
}
```

### Step 6: Compress

Remove redundant content within the assembled context:
- Deduplicate identical import blocks across chunks from the same file
- For sibling functions not directly relevant, show only signatures (not full bodies)
- Remove blank lines and non-essential formatting
- If a struct definition appears in multiple chunks, include it once at the top

### Step 7: Token Budget Management

- Set a total token budget for context (e.g., 8K, 16K tokens)
- After assembly, if over budget:
  1. Remove lowest-ranked chunks first
  2. Truncate expanded siblings to signatures only
  3. As a last resort, truncate the least-relevant chunks

## Formatting Best Practices

### File Headers

Always include file path and line numbers. This helps the LLM:
- Reference specific locations in its response
- Understand the project structure
- Distinguish between same-named symbols in different files

### Metadata Annotations

Prepend minimal metadata before code blocks:
```
// File: src/auth/service.rs
// Defines: AuthService, AuthService::login, AuthService::logout
// Depends on: UserStore, hash_password
```

### Language Tags

Use fenced code blocks with language identifiers for proper syntax recognition:
````
```rust
fn example() { ... }
```
````

### Chunk Boundaries

Clearly mark where one chunk ends and another begins, especially when chunks are from the same file but non-contiguous:
```
// ... (lines 15-19 omitted) ...
```

## Code-Specific Assembly Patterns

### The "Impl Block" Pattern

When a method is retrieved, assemble:
1. Struct/enum definition
2. Trait being implemented (if impl Trait for Type)
3. The specific method
4. Signatures of other methods in the same impl block

### The "Module" Pattern

When a function is retrieved, assemble:
1. Module-level imports
2. Module-level type definitions used by the function
3. The function itself
4. Signatures of functions it calls (from same module)

### The "Cross-File" Pattern

When a type is used across files, assemble:
1. The type definition (from its defining file)
2. Key impl blocks
3. The usage sites from the queried context

## Token Budget Guidelines

| Model Context | Suggested RAG Budget | Rationale |
|---|---|---|
| 8K tokens | 2-4K for retrieved context | Leave room for user query + response |
| 32K tokens | 8-16K for retrieved context | Can include more expansion |
| 128K+ tokens | 16-32K for retrieved context | More is not always better; relevance degrades |

## References

- [The LLM Context Problem in 2026 (LogRocket)](https://blog.logrocket.com/llm-context-problem/)
- [Context Window Efficiency Guide (DEV)](https://dev.to/siddhantkcode/the-engineering-guide-to-context-window-efficiency-202b)
- [Chunking Strategies for RAG (Weaviate)](https://weaviate.io/blog/chunking-strategies-for-rag)
- [Which RAG Formatting Strategy (Tiger Data)](https://www.tigerdata.com/blog/which-rag-chunking-and-formatting-strategy-is-best)
- [Advanced Chunking Techniques (Galileo)](https://galileo.ai/blog/mastering-rag-advanced-chunking-techniques-for-llm-applications)
- [Chunking Strategies (Pinecone)](https://www.pinecone.io/learn/chunking-strategies/)
- [Practical Guide to LLM Chunking (Mindee)](https://www.mindee.com/blog/llm-chunking-strategies)
- [Best Chunking Strategies 2026 (Firecrawl)](https://www.firecrawl.dev/blog/best-chunking-strategies-rag)
