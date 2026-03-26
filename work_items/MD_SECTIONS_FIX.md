# Goal

Fix `md.sections` to accumulate all content between headings (not just nodes with `"text"` field) and to nest sub-headings inside parent sections.

# Why

- `md.sections` on markdown with bullet lists returns empty content because `bi_sections` at `md_parse.rs:25-31` only appends from nodes with a `"text"` field. Code blocks have field `"code"`, lists have field `"items"` — both are dropped.
- Every heading starts a new section regardless of level. `# Parent` followed by `### Child` produces two sections instead of one.

# How it works

Node types and their content fields (from `crates/lx/src/stdlib/md/mod.rs`, the `parse_to_nodes` function):
- `"heading"`: fields `"level"` (Int), `"text"` (Str)
- `"para"`: field `"text"` (Str)
- `"code"`: fields `"lang"` (Option Str), `"code"` (Str)
- `"list"`: field `"items"` (List of Str)
- `"ordered"`: field `"items"` (List of Str)
- `"blockquote"`: field `"text"` (Str)
- `"hr"`: no content fields
- `"table"`: fields `"headers"` (List), `"rows"` (List)

# Files affected

- `crates/lx/src/stdlib/md/md_parse.rs` lines 12-39 — Rewrite `bi_sections`

# Task List

### Task 1: Rewrite bi_sections

In `crates/lx/src/stdlib/md/md_parse.rs`, rewrite `bi_sections` (lines 12-39). Keep the same function signature and return type.

The `cur` accumulator stays as `Option<(i64, String, String)>` — `(level, title, content)`.

Replace the heading detection at line 18. When a heading node is found, read its level. If `cur` is `Some` and the new heading's level is LESS THAN OR EQUAL to the current section's level, finalize the current section (push to `sections`) and start a new one. If the new heading's level is GREATER (deeper), append the heading as content text: push `"\n\n"` then `"#".repeat(level as usize)` then `" "` then the heading text, and continue accumulating in the current section. If `cur` is `None`, always start a new section.

Replace the content accumulation at lines 25-31. For each non-heading node, extract content based on the node's `"type"` field using `field_str(r, "type")`:
- `"para"` or `"blockquote"`: use `field_str(r, "text")`
- `"code"`: use `field_str(r, "code")` — optionally prefix with the `"lang"` field value if present
- `"list"` or `"ordered"`: get the `"items"` field as a List (`r.get(&intern("items"))` then `as_list()`), iterate each item, call `to_string()` on each, prefix with `"- "`, join with `"\n"`
- `"hr"`: append `"---"`
- Any other type with a `"text"` field: use `field_str(r, "text")`
- Otherwise: skip

Append each extracted content string to `cur`'s content with `"\n\n"` separator (same as current line 29).

### Task 2: Add tests

Create `tests/md_sections_fix.lx`:

Test bullet list content: `raw = "# Title\n\n- item1\n- item2"`. Parse with `md.parse raw` then `md.sections`. Assert the section content contains "item1".

Test code block content: `raw = "# Code\n\n` ++ "`" ++ "`" ++ "`" ++ `rust\nfn main() {}\n` ++ "`" ++ "`" ++ "`"`. Parse and assert section content contains "fn main".

Test sub-heading nesting: `raw = "# Parent\n\ntext\n\n### Child\n\nchild text"`. Parse. Assert `md.sections` returns 1 section titled "Parent" whose content contains "text", "Child", and "child text".

Test same-level split: `raw = "# A\n\ntext a\n\n# B\n\ntext b"`. Assert 2 sections returned.

Regression — plain paragraph: `raw = "# Simple\n\nhello world"`. Assert section content is "hello world".

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.
