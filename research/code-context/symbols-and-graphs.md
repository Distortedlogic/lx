# Symbol Extraction, Cross-References & Code Graphs

A structured symbol index enables precise lookups ("find the definition of `AuthService`") alongside fuzzy semantic search. Graph-based approaches complement vector search by capturing structural relationships.

## Tree-Sitter Tags & Queries

### Query Files (`.scm`)

Each language grammar includes query files:

| File | Purpose |
|---|---|
| `highlights.scm` | Syntax highlighting |
| `locals.scm` | Local scope definitions, references, and scope boundaries |
| `tags.scm` | Symbol definitions and references for code navigation |
| `injections.scm` | Embedded language detection |

### tags.scm for Symbol Extraction

Identifies definitions (where symbols are defined) and references (where symbols are used). Each tag has: role (definition/reference), kind (function, class, method, module), name, and byte range.

### locals.scm for Scope Resolution

Defines scopes, local definitions, and references within scopes. Enables scope-aware resolution: distinguishing between a local variable `x` and a module-level `x`.

## Symbol ID Format

```
{file_path}::{qualified_name}#{kind}
```

Examples: `src/auth/service.rs::AuthService#struct`, `src/auth/service.rs::AuthService::login#method`

Enables O(1) lookup by stable symbol ID.

## Cross-Reference Index

### Building the Index

1. **Extract definitions** from all files using `tags.scm` -> `HashMap<SymbolName, Vec<Definition>>`
2. **Extract references** from all files -> `HashMap<SymbolName, Vec<Reference>>`
3. **Resolve references to definitions** -- scope-aware, import-aware, build edges

### Data Structures

```
SymbolIndex {
    definitions: HashMap<SymbolId, SymbolDef>,
    by_name: HashMap<String, Vec<SymbolId>>,
    by_file: HashMap<PathBuf, Vec<SymbolId>>,
    references: HashMap<SymbolId, Vec<ReferenceInfo>>,
    reverse_refs: HashMap<SymbolId, Vec<SymbolId>>,
}
```

## What Code Graphs Capture

| Relationship | Example |
|---|---|
| **Calls** | `fn a()` calls `fn b()` |
| **Imports** | `mod a` uses `use crate::b::Type` |
| **Implements** | `impl Trait for Struct` |
| **Extends** | `struct B` contains `A` as a field |
| **Defines** | `mod a` defines `struct X` |
| **Type references** | `fn foo(x: Bar)` references `Bar` |

Tree-sitter extracts imports and definitions directly, but call relationships must be inferred by matching identifiers across files.

## Aider's Repo Map (PageRank Approach)

1. Use tree-sitter to extract definitions and references from all files
2. Build a `NetworkX MultiDiGraph`: nodes = files, edges = cross-file references
3. Run **PageRank** with personalization (boost files the user is editing)
4. Select top-ranked definitions that fit the token budget
5. Format as a condensed "repo map" showing key symbols per file

Files with many incoming references (widely-used utilities, core types) rank higher. Following imports from target files and adding those definitions with high priority dramatically improves context quality.

**Blog**: [aider.chat/2023/10/22/repomap.html](https://aider.chat/2023/10/22/repomap.html)

## Code Graph RAG

**Repo**: [github.com/vitali87/code-graph-rag](https://github.com/vitali87/code-graph-rag)

1. Parse code with tree-sitter (Python, JS, TS, C++, Rust, Java, Lua)
2. Extract symbols (functions, classes, imports, call sites)
3. Build knowledge graph in Memgraph
4. Query with Cypher to traverse relationships
5. Feed graph context + vector search results to LLM

## CodeRAG: Bigraph Approach

**Paper**: "CodeRAG: Supportive Code Retrieval on Bigraph for Real-World Code Generation" ([arXiv 2504.10046](https://arxiv.org/abs/2504.10046))

Uses a dual graph (requirement graph + DS-code graph). Maps requirements to code nodes, uses LLM reasoning to traverse. Results: +40.90 Pass@1 on GPT-4o (DevEval).

## Practical MCP Tools

| Tool | Input | Output |
|---|---|---|
| `get_definition` | symbol name or ID | definition location + source code |
| `find_references` | symbol name or ID | all reference locations |
| `list_symbols` | file path, kind filter | symbols in file/codebase |
| `get_callers` | function/method name | functions that call it |
| `get_callees` | function/method name | functions it calls |
| `get_type_hierarchy` | struct/trait name | impl blocks, trait implementations |
| `get_imports` | file path | all imports and what they resolve to |

## Single-Pass Extraction

Extract symbols during the same AST walk used for chunking: check chunk boundaries, extract symbol definitions, extract references, record scope chains, accumulate chunk content. Avoids walking the tree multiple times per file.

## Implementation Recommendations

**Start lightweight**: skip a full graph database. Use in-memory `HashMap<SymbolId, SymbolInfo>` with cross-reference edges built by matching reference names to definition names.

**Add a full graph** (e.g., petgraph crate) if users need multi-hop traversal, impact analysis, or architectural visualization.

## SCIP vs Tree-Sitter

SCIP provides precise cross-references with full type resolution. Tree-sitter is faster (no compilation), language-agnostic, but less precise (no type resolution). For an MCP server, tree-sitter is the right trade-off.

## References

- [Tree-sitter Code Navigation docs](https://tree-sitter.github.io/tree-sitter/4-code-navigation.html)
- [Aider repo map blog](https://aider.chat/2023/10/22/repomap.html)
- [Aider repo map docs](https://aider.chat/docs/repomap.html)
- [jCodeMunch MCP](https://github.com/SY-MEDIA/jCodeMunch-MCP)
- [Semantic Code Indexing (Medium)](https://medium.com/@email2dineshkuppan/semantic-code-indexing-with-ast-and-tree-sitter-for-ai-agents-part-1-of-3-eb5237ba687a)
- [SCIP to Tree-sitter RFC](https://github.com/orgs/sheeptechnologies/discussions/4)
- [Code-Graph-RAG](https://github.com/vitali87/code-graph-rag)
- [GraphRAG for Devs (Memgraph)](https://memgraph.com/blog/graphrag-for-devs-coding-assistant)
- [CodeRAG Bigraph paper](https://arxiv.org/abs/2504.10046)
- [GitLab Knowledge Graph](https://docs.gitlab.com/user/project/repository/knowledge_graph/)
- [FalkorDB Code Graph](https://www.falkordb.com/blog/code-graph/)
