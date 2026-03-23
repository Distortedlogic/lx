# Parsing & AST-Based Code Chunking

## Tree-Sitter: Incremental Parsing Library

Tree-sitter is a parser generator tool and incremental parsing library. It builds a **concrete syntax tree** (CST) for source files and efficiently updates it as the source is edited.

- **Written in**: Rust (core), outputs C parsers
- **Languages supported**: 100+ via community grammars
- **Used by**: Neovim, Helix, Zed, GitHub (for syntax highlighting and code navigation)
- **License**: MIT

### Why Tree-Sitter for Code Chunking

1. **Language-agnostic** -- same API across all supported languages
2. **Concrete syntax tree** -- preserves all tokens (whitespace, comments, punctuation). Concatenating CST nodes reproduces the original source verbatim (plug-and-play compatibility)
3. **Incremental parsing** -- updates previous tree in <1ms on edit, enabling real-time indexing
4. **Battle-tested** -- powers syntax in major editors, so grammars are well-maintained
5. **Node types** -- each node has a type (e.g., `function_definition`, `struct_item`, `impl_item`) that maps directly to semantic code boundaries

### Rust Bindings

**Crate**: [tree-sitter on crates.io](https://crates.io/crates/tree-sitter)

Each language needs its own grammar crate: `tree-sitter-rust`, `tree-sitter-python`, `tree-sitter-javascript`, etc.

Basic usage: create Parser -> set language -> parse source -> get Tree -> walk nodes to find semantic boundaries.

### Key Node Types for Rust

`source_file` (root), `function_item`, `struct_item`, `enum_item`, `impl_item`, `trait_item`, `mod_item`, `use_declaration`, `const_item`, `static_item`, `type_item`

## The Problem with Naive Chunking

Traditional text chunking (fixed-token, paragraph-based, line-based) breaks code at arbitrary boundaries:
- Splits functions/classes mid-body, destroying semantic meaning
- Produces chunks with wildly different information density
- Loses syntactic context (scope, imports, class membership)

## cAST: Key Research Paper

**Paper**: "cAST: Enhancing Code Retrieval-Augmented Generation with Structural Chunking via Abstract Syntax Tree" (EMNLP 2025 Findings, CMU)
**arXiv**: [2506.15655](https://arxiv.org/abs/2506.15655)

### Design Goals

1. **Syntactic integrity** -- chunk boundaries align with complete syntactic units
2. **High information density** -- each chunk packed up to a fixed size budget
3. **Language invariance** -- no language-specific heuristics; works via generic AST
4. **Plug-and-play compatibility** -- concatenating chunks reproduces the original file verbatim

### Algorithm: Recursive Split-Then-Merge

1. Parse source file into AST (using tree-sitter)
2. Starting from the first level of AST nodes, greedily merge sibling nodes into chunks
3. If adding a node would exceed the chunk size limit, recursively decompose it into its children
4. If a child node is still too large, recurse deeper
5. If a node fits individually but not in the current chunk, start a new chunk

**Results**: Recall@5 +4.3 points on RepoEval retrieval, Pass@1 +2.67 points on SWE-bench generation.

## Supermemory code-chunk Implementation

**Repo**: [github.com/supermemoryai/code-chunk](https://github.com/supermemoryai/code-chunk)

- Measures chunk size by **non-whitespace character count** (not lines), since blank lines/comments inflate line counts
- Same recursive split algorithm as cAST

### Metadata Enrichment Per Chunk

Each chunk includes:
- **Scope chain**: where the code lives (e.g., inside which class/module/function)
- **Entities**: what's defined in the chunk (functions, structs, enums, etc.)
- **Siblings**: what comes before/after for continuity
- **Imports**: what dependencies are referenced
- **contextualizedText**: prepends semantic context to raw code for better embedding quality

## Sizing Recommendations

From "Practical Code RAG at Scale" (arXiv 2510.20609):

| Context Budget | Optimal Chunk Size |
|---|---|
| <= 4000 tokens | 32-64 lines |
| 4000-8000 tokens | 64-128 lines |
| >= 16000 tokens | Consider whole-file retrieval |

Simple line-based chunking matches syntax-aware splitting across budgets (surprising finding), but AST chunks + metadata enrichment significantly improve embedding quality.

## References

- [cAST Paper (arXiv)](https://arxiv.org/abs/2506.15655)
- [cAST Paper (ACL Anthology)](https://aclanthology.org/2025.findings-emnlp.430/)
- [code-chunk by Supermemory](https://github.com/supermemoryai/code-chunk)
- [ASTChunk toolkit](https://github.com/yilinjz/astchunk)
- [Tree-sitter GitHub](https://github.com/tree-sitter/tree-sitter)
- [Tree-sitter Rust crate](https://crates.io/crates/tree-sitter)
- [Tree-sitter Rust grammar](https://github.com/tree-sitter/tree-sitter-rust)
- [Incremental Parsing Using Tree-sitter (Strumenta)](https://tomassetti.me/incremental-parsing-using-tree-sitter/)
- [AST Enables Code RAG (Medium)](https://medium.com/@jouryjc0409/ast-enables-code-rag-models-to-overcome-traditional-chunking-limitations-b0bc1e61bdab)
