# Goal

Add `FileId` to source spans for multi-file diagnostics, and add trivia (comment) preservation during lexing so future tools (formatter, refactoring) can round-trip source text.

# Why

- `SourceSpan` from miette is (offset, length) within a single source string. For multi-file projects, a span cannot identify which file it belongs to — the file context must be threaded separately via `LxError::Sourced`. This breaks any tool that needs to display diagnostics from multiple files simultaneously or cross-reference spans across modules
- The lexer discards all comments. There is no mechanism to store or retrieve trivia. This makes it impossible to write a code formatter or comment-preserving refactoring tool

# Verified facts about lexer

- **Comment syntax:** `--` line comments only. Regex: `--[^\n]*`. No block comments exist.
- **RawToken::Comment variant** exists in `lexer/raw_token.rs` (line 92-93), recognized by the logos lexer.
- **Comment discard site:** `lexer/mod.rs` line 107: `RawToken::Comment => {},` — the empty arm silently drops comments.
- **`lex()` return type:** `Result<Vec<Token>, LxError>` (lexer/mod.rs line 23).
- **Lexer struct fields:** `source: &str`, `pos: usize`, `tokens: Vec<Token>`, `depth: i32`, `last_was_semi: bool`, `brace_stack: Vec<bool>`.

# Verified parse() call sites (8 total)

1. `crates/lx-cli/src/check.rs` line 58 — `parse(tokens)` in `recheck_source()`
2. `crates/lx-cli/src/fmt.rs` line 24 — `parse(tokens)` in `fmt_source()`
3. `crates/lx-cli/src/agent_cmd.rs` line 21 — `lx::parser::parse(tokens)`
4. `crates/lx-cli/src/run.rs` line 10 — `lx::parser::parse(tokens)` in `run()`
5. `crates/lx-cli/src/run.rs` line 43 — `lx::parser::parse(tokens)` in `read_and_parse()`
6. `crates/lx/src/interpreter/modules.rs` line 108 — `crate::parser::parse(tokens)`
7. `crates/lx/src/stdlib/diag/mod.rs` line 73 — `parse(tokens)` in `extract_graph()`
8. `crates/lx/src/stdlib/test_mod/test_invoke.rs` line 16 — `crate::parser::parse(tokens)`

All 8 call `parse(tokens)` with the same pattern: lex → parse → use program/errors.

# What changes

1. Define `FileId`, `SourceDb`, `FullSpan`, `Comment`, `CommentStore` in a new source module
2. Modify lexer to capture comments into a `Vec<Comment>` instead of discarding them, and return `(Vec<Token>, CommentStore)` from `lex()`
3. Add `comments: CommentStore` field to `Program` struct
4. Add `file: FileId` field to `Program` struct
5. Update `parse()` to accept `FileId` and `CommentStore`, pass them into `Program`
6. Update all 8 callers to construct a FileId and destructure the new lex return type

# Files affected

| File | Change |
|------|--------|
| `crates/lx/src/source.rs` | NEW — FileId, SourceDb, FullSpan, Comment, CommentStore |
| `crates/lx/src/lib.rs` | Add `pub mod source;` |
| `crates/lx/src/lexer/mod.rs` | Capture comments, change lex return to `Result<(Vec<Token>, CommentStore), LxError>` |
| `crates/lx/src/ast/mod.rs` | Add `comments: CommentStore` and `file: FileId` fields to Program |
| `crates/lx/src/parser/mod.rs` | Accept FileId and CommentStore, store in Program |
| `crates/lx/src/folder/desugar.rs` | Carry comments and file through to Core Program |
| `crates/lx-cli/src/check.rs` | Update lex/parse call (line 58) |
| `crates/lx-cli/src/fmt.rs` | Update lex/parse call (line 24) |
| `crates/lx-cli/src/agent_cmd.rs` | Update lex/parse call (line 21) |
| `crates/lx-cli/src/run.rs` | Update lex/parse calls (lines 10, 43) |
| `crates/lx/src/interpreter/modules.rs` | Update lex/parse call (line 108) |
| `crates/lx/src/stdlib/diag/mod.rs` | Update lex/parse call (line 73) |
| `crates/lx/src/stdlib/test_mod/test_invoke.rs` | Update lex/parse call (line 16) |

# Task List

### Task 1: Define source infrastructure types

Create `crates/lx/src/source.rs`:

```rust
use std::sync::Arc;
use miette::SourceSpan;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileId(u32);

impl FileId {
    pub fn new(id: u32) -> Self { Self(id) }
    pub fn index(self) -> u32 { self.0 }
}

pub struct SourceDb {
    files: Vec<SourceFile>,
}

struct SourceFile {
    path: String,
    source: Arc<str>,
}

impl SourceDb {
    pub fn new() -> Self { Self { files: Vec::new() } }

    pub fn add_file(&mut self, path: String, source: Arc<str>) -> FileId {
        let id = FileId(self.files.len() as u32);
        self.files.push(SourceFile { path, source });
        id
    }

    pub fn source(&self, id: FileId) -> &str {
        &self.files[id.0 as usize].source
    }

    pub fn path(&self, id: FileId) -> &str {
        &self.files[id.0 as usize].path
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FullSpan {
    pub file: FileId,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct Comment {
    pub span: SourceSpan,
    pub text: String,
}

#[derive(Debug, Clone, Default)]
pub struct CommentStore {
    comments: Vec<Comment>,
}

impl CommentStore {
    pub fn push(&mut self, comment: Comment) {
        self.comments.push(comment);
    }

    pub fn all(&self) -> &[Comment] {
        &self.comments
    }

    pub fn comments_in_range(&self, start: usize, end: usize) -> &[Comment] {
        let lo = self.comments.partition_point(|c| c.span.offset() < start);
        let hi = self.comments.partition_point(|c| c.span.offset() < end);
        &self.comments[lo..hi]
    }
}
```

Only line comments exist in lx (no block comments), so `CommentKind` is unnecessary.

`comments_in_range` uses `partition_point` (binary search) which requires comments to be sorted by offset. The lexer processes tokens in source order, so pushing comments in encounter order maintains the sort invariant.

In `crates/lx/src/lib.rs`, add `pub mod source;`.

### Task 2: Capture comments in lexer

In `crates/lx/src/lexer/mod.rs`:

Add `comments: Vec<crate::source::Comment>` field to the `Lexer` struct (after `brace_stack`).

Initialize it as `comments: Vec::new()` in the constructor.

Change the `RawToken::Comment` match arm (line 107) from:
```rust
RawToken::Comment => {},
```
to:
```rust
RawToken::Comment => {
    let text = self.source[start..self.pos].to_string();
    self.comments.push(crate::source::Comment {
        span: self.sp(start, self.pos),
        text,
    });
},
```

Where `start` is the byte offset where the current raw token began. Check how other RawToken arms access the start position — the logos lexer provides `span()` on the lexer iterator which gives the byte range. The `self.pos` tracking in the Lexer struct may need adjustment. Read the main lexing loop to determine exactly how `start` is obtained — it is the position before advancing past the token.

Change the `lex` function return type from `Result<Vec<Token>, LxError>` to `Result<(Vec<Token>, crate::source::CommentStore), LxError>`.

At the function's return site, wrap the tokens with the comment store:
```rust
Ok((lexer.tokens, crate::source::CommentStore { comments: lexer.comments }))
```

Note: `CommentStore` fields are private, so either make the `comments` field `pub(crate)` or add a `CommentStore::from_vec(comments: Vec<Comment>) -> Self` constructor.

### Task 3: Add comments and file fields to Program

In `crates/lx/src/ast/mod.rs`, change `Program` to:

```rust
pub struct Program<Phase = Surface> {
    pub stmts: Vec<StmtId>,
    pub arena: AstArena,
    pub comments: crate::source::CommentStore,
    pub file: crate::source::FileId,
    pub _phase: PhantomData<Phase>,
}
```

### Task 4: Update parse() to accept and store FileId and CommentStore

In `crates/lx/src/parser/mod.rs`, change `parse` signature to:

```rust
pub fn parse(tokens: Vec<Token>, file: FileId, comments: CommentStore) -> ParseResult
```

Pass `comments` and `file` into the `Program` struct construction at the end of the function.

### Task 5: Update desugar to carry comments and file

In `crates/lx/src/folder/desugar.rs`, the `desugar` function constructs `Program<Core>`. Update to carry over both new fields:

```rust
let core = Program {
    stmts: folded.stmts,
    arena: folded.arena,
    comments: folded.comments,
    file: folded.file,
    _phase: PhantomData,
};
```

Also update `walk_transform_program` in `visitor/walk_transform/mod.rs` — it constructs/destructures Program. Ensure `comments` and `file` are preserved through transformation.

### Task 6: Update all 8 parse callers

Each caller currently does `let tokens = lex(source)?;` then `let result = parse(tokens);`. The new pattern is:

```rust
let (tokens, comments) = lex(source)?;
let result = parse(tokens, file_id, comments);
```

For `file_id`, callers in `lx-cli` that have access to file paths should construct a SourceDb and register the file. For internal callers (interpreter/modules.rs, diag/mod.rs, test_invoke.rs) that don't track files, use `FileId::new(0)` as a sentinel value — these are single-file contexts.

Update each caller:

1. **check.rs line 58**: `let (tokens, comments) = lex(&source)?; let result = parse(tokens, file_id, comments);`
2. **fmt.rs line 24**: same pattern
3. **agent_cmd.rs line 21**: same pattern
4. **run.rs line 10**: same pattern
5. **run.rs line 43**: same pattern
6. **modules.rs line 108**: `let (tokens, comments) = crate::lexer::lex(&source)?; let result = crate::parser::parse(tokens, FileId::new(0), comments);`
7. **diag/mod.rs line 73**: `let (tokens, comments) = lex(&source)?; let result = parse(tokens, FileId::new(0), comments);`
8. **test_invoke.rs line 16**: `let (tokens, comments) = crate::lexer::lex(&source)?; let result = crate::parser::parse(tokens, FileId::new(0), comments);`

### Task 7: Format and commit

Run `just fmt` then `git add -A && git commit -m "feat: add FileId, SourceDb, CommentStore for multi-file spans and trivia preservation"`.

### Task 8: Verify

Run `just diagnose`. Fix all errors and warnings. Re-run until clean. Then run `just test`. Fix any failures.

---

## CRITICAL REMINDERS

1. **NEVER run raw cargo commands.** Use `just fmt`, `just test`, `just diagnose`.
2. **Run commands VERBATIM.** Do not append pipes, redirects, or shell operators.
3. **Do not add, skip, reorder, or combine tasks.**
4. **Comments must be sorted by offset** in the CommentStore for binary search to work. The lexer processes tokens in source order, so pushing in encounter order maintains the invariant.
5. **`walk_transform_program`** in `visitor/walk_transform/mod.rs` destructures and reconstructs Program — it must carry the new fields through.
6. **Do not change existing `SourceSpan` usages** — this task adds `FullSpan` as a new type alongside `SourceSpan`, not as a replacement.

---

## Task Loading Instructions

To execute this work item, load it with the workflow MCP:

```
mcp__workflow__load_work_item({ path: "work_items/SOURCE_INFRASTRUCTURE.md" })
```

Then call `next_task` to begin.
