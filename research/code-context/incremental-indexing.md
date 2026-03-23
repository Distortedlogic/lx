# Incremental Indexing & File Watching

For a code context MCP server to be useful in a dev workflow, it must keep its index up-to-date as files change -- without re-indexing the entire codebase on every edit.

## File Watching in Rust: notify crate

**Crate**: [crates.io/crates/notify](https://crates.io/crates/notify)
**Docs**: [docs.rs/notify](https://docs.rs/notify/)
**Repo**: [github.com/notify-rs/notify](https://github.com/jwilm/rsnotify)

### Overview

- Cross-platform filesystem notification library
- Born out of need for `cargo watch`
- Auto-selects best backend per platform: inotify (Linux), FSEvents (macOS), ReadDirectoryChangesW (Windows)
- Supports recursive directory watching

### Debouncing

Rapid file changes (e.g., save-on-type, git checkout) produce many events. Debouncing collapses duplicates:

**notify-debouncer-mini**: Simple debouncer, emits one event per file per timeframe.
- Crate: [docs.rs/notify-debouncer-mini](https://docs.rs/notify-debouncer-mini/latest/notify_debouncer_mini/)
- Typical: `Duration::from_millis(500)` debounce window

**notify-debouncer-full**: Full-featured debouncer with file ID tracking, handles renames.
- Crate: [docs.rs/notify-debouncer-full](https://docs.rs/notify-debouncer-full)

### Event Types

- `Create` -- new file
- `Modify` -- file content changed
- `Remove` -- file deleted
- `Rename` -- file moved/renamed

## Incremental Indexing Pipeline

### On File Create/Modify

```
1. Debounce (500ms window)
2. Filter: is this a source file we care about? (check extension, .gitignore)
3. Parse with tree-sitter (or use incremental parse if we cached the previous tree)
4. Chunk the new AST into semantic units
5. Diff chunks against previous version:
   - New chunks -> generate embeddings, insert into vector DB
   - Removed chunks -> delete from vector DB
   - Modified chunks -> re-embed, upsert
6. Update BM25 index for affected chunks
7. Update symbol index
```

### On File Delete

```
1. Remove all chunks associated with that file from vector DB
2. Remove from BM25 index
3. Remove symbols from symbol index
4. Clean up cached AST
```

### On File Rename

```
1. Update file path metadata on all chunks (no re-embedding needed)
2. Update BM25 index entries
3. Update symbol index paths
```

## Tree-Sitter Incremental Parsing

Tree-sitter supports incremental parsing: after editing a file, pass the edit range to the previous parse tree and tree-sitter will update it in <1ms. This avoids re-parsing the entire file.

```
Previous tree + edit description -> Updated tree (sub-millisecond)
```

For the incremental indexing pipeline:
1. Cache the tree-sitter `Tree` for each file
2. On file change, compute the text diff
3. Call `tree.edit()` with the edit range
4. Re-parse with the edited tree as the old tree
5. Only re-chunk the changed regions of the AST

## Chunk Diffing Strategy

Naive approach: re-chunk entire file, diff against stored chunks by content hash.

Better approach: use tree-sitter's changed ranges:
1. After incremental parse, tree-sitter can report which AST ranges changed
2. Only re-chunk the changed subtrees
3. Chunks whose AST nodes didn't change can be kept as-is

## Batch vs Real-Time

- **Initial index**: batch process all files (parallelize across CPU cores)
- **Ongoing updates**: real-time via file watcher
- **Git operations**: debounce heavily (git checkout can touch hundreds of files)

## Coordination

Use broadcast channels to coordinate:
- File watcher thread -> sends change events
- Indexer thread -> receives events, processes updates
- Query thread -> reads from vector DB (unblocked by indexing)

500ms debouncing batches rapid file changes together.

## Single-Pass Indexing

For efficiency, extract everything in one AST traversal:
- Symbols (definitions and references)
- Chunk boundaries
- Scope chains and metadata
- Import statements

This avoids walking the AST multiple times per file.

## References

- [notify crate docs](https://docs.rs/notify/)
- [notify-debouncer-mini docs](https://docs.rs/notify-debouncer-mini/latest/notify_debouncer_mini/)
- [notify-debouncer-full docs](https://docs.rs/notify-debouncer-full)
- [File Watcher with Debouncing in Rust](https://oneuptime.com/blog/post/2026-01-25-file-watcher-debouncing-rust/view)
- [Tree-sitter incremental parsing](https://tomassetti.me/incremental-parsing-using-tree-sitter/)
- [notify Rust forum discussion](https://users.rust-lang.org/t/how-to-make-good-usage-of-the-notify-crate-for-responsive-events/55891)
