# Goal

Add a post-parse comment-attachment pass that assigns each comment in `CommentStore` to a specific AST `NodeId` with a placement (Leading, Trailing, or Dangling), stored in a `CommentMap` on the `Program`.

# Prerequisites

None.

# Why

- `CommentStore` is a flat sorted vec queried by byte range. There is no association between a comment and the AST node it belongs to. A formatter or code modifier that moves, deletes, or inserts nodes has no way to determine which comments travel with which nodes.

# Verified current state

- `parse()` signature (`parser/mod.rs` line 58): `pub fn parse(tokens: Vec<Token>, file: FileId, comments: CommentStore) -> ParseResult` — does NOT receive source text
- All 8 `parse()` callers pass `FileId::new(0)` and do NOT pass source text
- All callers have source text available in their local scope (they pass it to `lex()`)
- Program construction sites: `parser/mod.rs` line 84, `folder/desugar.rs` line 162
- `walk_transform_program` in `visitor/walk_transform/mod.rs` lines 35-40 destructures/reconstructs Program
- `NodeId` derives `Hash + Eq` (`ast/arena.rs` line 111)
- Comments are sorted by offset in `CommentStore` (lexer pushes in encounter order)

# What changes

1. Define `CommentPlacement`, `AttachedComment`, `CommentMap` types in `source.rs`
2. Add `comment_map: CommentMap` field to `Program`
3. Add `source: &str` parameter to `parse()` and update all 8 callers
4. Implement `attach_comments` in a new `ast/comment_attach.rs`
5. Call `attach_comments` inside `parse()` after constructing the Program

# All 8 parse() callers to update

| File | Line | Current call |
|------|------|-------------|
| `crates/lx-cli/src/run.rs` | 10 | `lx::parser::parse(tokens, lx::source::FileId::new(0), comments)` |
| `crates/lx-cli/src/run.rs` | 43 | `lx::parser::parse(tokens, lx::source::FileId::new(0), comments)` |
| `crates/lx-cli/src/check.rs` | 58 | `parse(tokens, lx::source::FileId::new(0), comments)` |
| `crates/lx-cli/src/fmt.rs` | 24 | `parse(tokens, lx::source::FileId::new(0), comments)` |
| `crates/lx-cli/src/agent_cmd.rs` | 21 | `lx::parser::parse(tokens, lx::source::FileId::new(0), comments)` |
| `crates/lx/src/stdlib/test_mod/test_invoke.rs` | 16 | `crate::parser::parse(tokens, crate::source::FileId::new(0), comments)` |
| `crates/lx/src/stdlib/diag/mod.rs` | 73 | `parse(tokens, crate::source::FileId::new(0), comments)` |
| `crates/lx/src/interpreter/modules.rs` | 108 | `crate::parser::parse(tokens, crate::source::FileId::new(0), comments)` |

Each caller has `source`/`&source` available in its local scope. Add `source` (or `&source`) as the new 4th parameter.

# Files affected

| File | Change |
|------|--------|
| `crates/lx/src/source.rs` | Add CommentPlacement, AttachedComment, CommentMap types |
| `crates/lx/src/ast/comment_attach.rs` | **New** — attach_comments function |
| `crates/lx/src/ast/mod.rs` | Add comment_attach module, add comment_map field to Program |
| `crates/lx/src/parser/mod.rs` | Add source param, call attach_comments, store comment_map in Program |
| `crates/lx/src/folder/desugar.rs` | Carry comment_map through Program construction (line 162) |
| `crates/lx/src/visitor/walk_transform/mod.rs` | Carry comment_map through walk_transform_program |
| `crates/lx-cli/src/run.rs` | Lines 10, 43: add source param |
| `crates/lx-cli/src/check.rs` | Line 58: add source param |
| `crates/lx-cli/src/fmt.rs` | Line 24: add source param |
| `crates/lx-cli/src/agent_cmd.rs` | Line 21: add source param |
| `crates/lx/src/stdlib/test_mod/test_invoke.rs` | Line 16: add source param |
| `crates/lx/src/stdlib/diag/mod.rs` | Line 73: add source param |
| `crates/lx/src/interpreter/modules.rs` | Line 108: add source param |

# Task List

### Task 1: Define comment attachment types

In `crates/lx/src/source.rs`, add after the existing `CommentStore` impl block:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommentPlacement {
    Leading,
    Trailing,
    Dangling,
}

#[derive(Debug, Clone)]
pub struct AttachedComment {
    pub comment_idx: usize,
    pub placement: CommentPlacement,
}

pub type CommentMap = std::collections::HashMap<crate::ast::NodeId, Vec<AttachedComment>>;
```

`comment_idx` indexes into `CommentStore::all()`, avoiding cloning comment text.

### Task 2: Add comment_map field to Program

In `crates/lx/src/ast/mod.rs`, add field to Program:

```rust
pub struct Program<Phase = Surface> {
    pub stmts: Vec<StmtId>,
    pub arena: AstArena,
    pub comments: crate::source::CommentStore,
    pub comment_map: crate::source::CommentMap,
    pub file: crate::source::FileId,
    pub _phase: PhantomData<Phase>,
}
```

Update the Program construction in `parser/mod.rs` line 84 — add `comment_map: std::collections::HashMap::new()` (placeholder; Task 5 replaces it).

Update the Program construction in `folder/desugar.rs` line 162 — add `comment_map: folded.comment_map`.

Update `walk_transform_program` in `visitor/walk_transform/mod.rs` — the function destructures `program` and reconstructs it. Add `comment_map` to both the destructure and the reconstruct. Currently lines 36-39:

```rust
let stmts: Vec<StmtId> = program.stmts.clone();
let folded: Vec<StmtId> = stmts.into_iter().map(|s| walk_transform_stmt(t, s, &mut program.arena)).collect();
program.stmts = folded;
program
```

The comment_map field is carried through unchanged because `program` is passed by value and returned.

### Task 3: Add source param to parse() and update all callers

In `crates/lx/src/parser/mod.rs`, change parse signature:

```rust
pub fn parse(tokens: Vec<Token>, file: FileId, comments: CommentStore, source: &str) -> ParseResult {
    parse_with_recovery(tokens, file, comments, source)
}
```

Add `source: &str` param to `parse_with_recovery` as well. Thread it through to where Program is constructed (line 84).

Update all 8 callers listed in the table above. Each caller has source text in scope:
- `run.rs` line 10: the function receives `source: &str` — pass `source`
- `run.rs` line 43: `read_and_parse` reads file into `source` — pass `&source`
- `check.rs` line 58: `recheck_source` has `source: &str` — pass `source`
- `fmt.rs` line 24: `fmt_source` has `source: &str` — pass `source`
- `agent_cmd.rs` line 21: has `source` local — pass `&source`
- `test_invoke.rs` line 16: has `source` local — pass `&source`
- `diag/mod.rs` line 73: `extract_graph` has `source` param — pass `source`
- `modules.rs` line 108: has `source` local — pass `&source`

### Task 4: Implement attach_comments

Create `crates/lx/src/ast/comment_attach.rs`.

In `crates/lx/src/ast/mod.rs`, add `mod comment_attach;` and `pub use comment_attach::attach_comments;`.

Implementation:

```rust
use std::collections::HashMap;
use crate::source::{AttachedComment, CommentMap, CommentPlacement, CommentStore};
use super::{AstArena, NodeId, StmtId};

pub fn attach_comments(stmts: &[StmtId], arena: &AstArena, comments: &CommentStore, source: &str) -> CommentMap {
    let mut map: CommentMap = HashMap::new();
    if comments.all().is_empty() {
        return map;
    }

    let mut nodes: Vec<(NodeId, usize, usize)> = Vec::new();
    for (id, spanned) in arena.iter_exprs() {
        let offset = spanned.span.offset();
        let end = offset + spanned.span.len();
        nodes.push((NodeId::Expr(id), offset, end));
    }
    for (id, spanned) in arena.iter_stmts() {
        let offset = spanned.span.offset();
        let end = offset + spanned.span.len();
        nodes.push((NodeId::Stmt(id), offset, end));
    }
    for (id, spanned) in arena.iter_patterns() {
        let offset = spanned.span.offset();
        let end = offset + spanned.span.len();
        nodes.push((NodeId::Pattern(id), offset, end));
    }
    for (id, spanned) in arena.iter_type_exprs() {
        let offset = spanned.span.offset();
        let end = offset + spanned.span.len();
        nodes.push((NodeId::TypeExpr(id), offset, end));
    }
    nodes.sort_by_key(|&(_, offset, end)| (offset, std::cmp::Reverse(end)));

    for (comment_idx, comment) in comments.all().iter().enumerate() {
        let c_offset = comment.span.offset();
        let c_end = c_offset + comment.span.len();

        let enclosing = nodes
            .iter()
            .filter(|&&(_, n_offset, n_end)| n_offset <= c_offset && c_end <= n_end)
            .min_by_key(|&&(_, n_offset, n_end)| n_end - n_offset);

        let (node_id, placement) = if let Some(&(node_id, n_offset, n_end)) = enclosing {
            let before_comment = &source[n_offset..c_offset];
            let after_comment = &source[c_end..n_end.min(source.len())];
            let has_node_content_before = before_comment.trim().len() > before_comment.trim_start_matches(|c: char| c.is_whitespace()).len()
                || before_comment.contains(|c: char| !c.is_whitespace());
            let has_newline_after = after_comment.starts_with('\n') || after_comment.starts_with("\r\n") || after_comment.is_empty();

            if has_newline_after && !has_node_content_before {
                (node_id, CommentPlacement::Leading)
            } else if has_node_content_before {
                (node_id, CommentPlacement::Trailing)
            } else {
                (node_id, CommentPlacement::Dangling)
            }
        } else {
            let first_stmt = stmts.first().map(|s| NodeId::Stmt(*s));
            let last_stmt = stmts.last().map(|s| NodeId::Stmt(*s));
            let node = if let Some(first) = first_stmt {
                let first_offset = first.span(arena).offset();
                if c_offset < first_offset {
                    (first, CommentPlacement::Leading)
                } else if let Some(last) = last_stmt {
                    (last, CommentPlacement::Trailing)
                } else {
                    (first, CommentPlacement::Trailing)
                }
            } else {
                continue;
            };
            node
        };

        map.entry(node_id).or_default().push(AttachedComment { comment_idx, placement });
    }

    map
}
```

This is a first-pass algorithm. The `enclosing` search is O(n) per comment. For lx programs with typical comment counts (tens to low hundreds), this is adequate. If it exceeds 300 lines, split the node collection into a helper function in the same file.

### Task 5: Wire attachment into parse

In `crates/lx/src/parser/mod.rs`, after constructing the Program (inside `parse_with_recovery`), call attachment:

Replace the `comment_map: std::collections::HashMap::new()` placeholder from Task 2 with:

```rust
let comment_map = crate::ast::attach_comments(&stmts, &arena, &comments, source);
```

This must happen after `arena` is unwrapped from `Rc<RefCell>` and after `stmts` is finalized. Insert it between the arena unwrap and the Program construction.

### Task 6: Add accessor methods to Program

In `crates/lx/src/ast/mod.rs`, add:

```rust
impl<P> Program<P> {
    pub fn leading_comments(&self, node: NodeId) -> Vec<&crate::source::Comment> {
        self.attached_comments(node, crate::source::CommentPlacement::Leading)
    }

    pub fn trailing_comments(&self, node: NodeId) -> Vec<&crate::source::Comment> {
        self.attached_comments(node, crate::source::CommentPlacement::Trailing)
    }

    pub fn dangling_comments(&self, node: NodeId) -> Vec<&crate::source::Comment> {
        self.attached_comments(node, crate::source::CommentPlacement::Dangling)
    }

    fn attached_comments(&self, node: NodeId, placement: crate::source::CommentPlacement) -> Vec<&crate::source::Comment> {
        let all = self.comments.all();
        self.comment_map
            .get(&node)
            .map(|attached| {
                attached
                    .iter()
                    .filter(|a| a.placement == placement)
                    .map(|a| &all[a.comment_idx])
                    .collect()
            })
            .unwrap_or_default()
    }
}
```

### Task 7: Verify

Run `just fmt` then `just diagnose`. Fix all errors and warnings. Re-run until clean. Then run `just test`. Fix any failures.

### Task 8: Commit

Run `just fmt` then `git add -A && git commit -m "feat: comment attachment pass — assign comments to AST nodes with Leading/Trailing/Dangling placement"`.

---

## CRITICAL REMINDERS

1. **NEVER run raw cargo commands.** Use `just fmt`, `just test`, `just diagnose`.
2. **Run commands VERBATIM.** Do not append pipes, redirects, or shell operators.
3. **Do not add, skip, reorder, or combine tasks.**
4. **300 line file limit** — if `comment_attach.rs` exceeds 300 lines, split node collection into a helper.
5. **`comment_idx` must remain valid** — it indexes into `CommentStore::all()` which is immutable after parsing.
6. **`walk_transform_program`** carries `comment_map` through because it takes and returns `Program` by value — the field is preserved automatically. Verify no destructure-reconstruct drops it.
7. **`parse_with_recovery`** uses `Rc<RefCell<AstArena>>` internally — `attach_comments` must be called AFTER the arena is unwrapped via `Rc::try_unwrap().into_inner()`.

---

## Task Loading Instructions

To execute this work item, load it with the workflow MCP:

```
mcp__workflow__load_work_item({ path: "work_items/COMMENT_ATTACHMENT.md" })
```

Then call `next_task` to begin.
