# Goal

Add `pkg/data/vectors` — a vector index for semantic search using Store and `ai.embed`. Agents can index text chunks, search by similarity, and retrieve the most relevant items. Pure lx, no Rust changes.

**Depends on: EMBED_BACKEND work item must be completed first.**

# Why

- Semantic search is the backbone of context engines in Augment Code, Cursor, and Devin. Agents working on large codebases need to find semantically relevant code, not just grep matches.
- With `ai.embed` providing the embedding computation, the index/search layer is pure data structure work — Store-backed with cosine similarity scoring.
- A simple in-memory vector index is sufficient for agent workflows. Production vector databases (Pinecone, etc.) can be accessed via MCP connectors.

# What Changes

**New file `pkg/data/vectors.lx`:** Vector index Class with add, search, remove, and persistence.

- `VectorIndex {}` — Class with Store-backed entries
- `index.add id text metadata` — embed text, store vector + metadata
- `index.add_batch items` — batch embed and store
- `index.search query k` — embed query, find k nearest by cosine similarity
- `index.remove id` — remove entry
- `index.save path` / `index.load path` — persist/restore

# Files Affected

- `pkg/data/vectors.lx` — New file
- `tests/107_vectors.lx` — New test file

# Task List

### Task 1: Create pkg/data/vectors.lx

**Subject:** Create VectorIndex Class with embedding, indexing, and similarity search

**Description:** Create `pkg/data/vectors.lx`:

```
-- Vector index -- semantic search over text chunks using ai.embed + Store.
-- Cosine similarity scoring. In-memory with optional file persistence.

use std/ai
use std/json
use std/math

Class +VectorIndex = {
  entries: Store ()
  dimensions: 0

  add = (id text metadata) {
    vectors = ai.embed [text] ^
    vec = vectors.[0]
    self.dimensions <- vec | len
    self.entries.set id {
      text: text
      vector: vec
      metadata: metadata ?? {}
    }
    Ok ()
  }

  add_batch = (items) {
    texts = items | map (.text)
    vectors = ai.embed texts ^
    self.dimensions <- vectors.[0] | len
    items | enumerate | each (pair) {
      idx = pair.[0]
      item = pair.[1]
      self.entries.set item.id {
        text: item.text
        vector: vectors.(idx)
        metadata: item.metadata ?? {}
      }
    }
    Ok ()
  }

  search = (query k) {
    vectors = ai.embed [query] ^
    query_vec = vectors.[0]

    self.entries.entries ()
      | map (pair) {
          id: pair.[0]
          entry: pair.[1]
          score: cosine_similarity query_vec pair.[1].vector
        }
      | sort_by (.score)
      | rev
      | take (k ?? 5)
      | map (r) {
          id: r.id
          text: r.entry.text
          score: r.score
          metadata: r.entry.metadata
        }
  }

  remove = (id) {
    self.entries.remove id
  }

  count = () {
    self.entries.len ()
  }

  save = (path) {
    data = self.entries.entries ()
      | map (pair) {id: pair.[0]  ..pair.[1]}
    json_str = json.encode_pretty data
    std/fs.write path json_str ^
  }

  load = (path) {
    text = std/fs.read path ^
    items = json.parse text ^
    items | each (item) {
      self.entries.set item.id {
        text: item.text
        vector: item.vector
        metadata: item.metadata ?? {}
      }
    }
    self.dimensions <- (items.[0].vector | len) ?? 0
    Ok ()
  }
}

cosine_similarity = (a b) {
  a | len == 0 ? 0.0 : {
    dot = a | enumerate | fold 0.0 (acc pair) {
      idx = pair.[0]
      val = pair.[1]
      acc + val * (b.(idx) ?? 0.0)
    }
    mag_a = a | fold 0.0 (acc v) acc + v * v | math.sqrt
    mag_b = b | fold 0.0 (acc v) acc + v * v | math.sqrt
    denom = mag_a * mag_b
    denom == 0.0 ? 0.0 : dot / denom
  }
}
```

The cosine similarity computation is O(n*d) where n is index size and d is vector dimensions. For agent workflows with hundreds to low thousands of entries, this is adequate. For larger indices, use an MCP connector to a proper vector database.

**ActiveForm:** Creating VectorIndex Class with embedding and search

---

### Task 2: Write tests for pkg/data/vectors

**Subject:** Write tests for VectorIndex with graceful skip when no embed provider

**Description:** Create `tests/107_vectors.lx`:

```
use pkg/data/vectors

-- VectorIndex creation
idx = VectorIndex ()
assert (idx.count () == 0) "empty index"
assert (type_of idx.add == "Fn") "add method exists"
assert (type_of idx.search == "Fn") "search method exists"
assert (type_of idx.remove == "Fn") "remove method exists"

-- Test with actual embeddings if provider available
test_result = ai.embed ["test"]
test_result ? {
  Ok _ -> {
    idx.add "doc1" "Rust is a systems programming language" {} ^
    idx.add "doc2" "Python is great for data science" {} ^
    idx.add "doc3" "Rust borrow checker prevents memory bugs" {} ^
    assert (idx.count () == 3) "three entries"

    results = idx.search "memory safety in systems programming" 2
    assert (results | len == 2) "returns k results"
    assert (results.[0].score > 0) "scores are positive"

    -- Most relevant should be Rust-related
    top = results.[0]
    assert (top.text | contains? "Rust") "top result is Rust-related"

    -- Remove
    idx.remove "doc2"
    assert (idx.count () == 2) "count after remove"

    log.info "107_vectors: full test passed"
  }
  Err e -> {
    log.info "107_vectors: skipped embedding tests (no provider: {e})"
  }
}

log.info "107_vectors: all passed"
```

Run `just test` to verify.

**ActiveForm:** Writing tests for VectorIndex

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.

---

## Task Loading Instructions

To execute this work item, load it with the workflow MCP:

```
mcp__workflow__load_work_item({ path: "work_items/PKG_VECTORS.md" })
```

Then call `next_task` to begin.
