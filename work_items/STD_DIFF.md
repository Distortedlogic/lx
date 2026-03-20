# Goal

Add `std/diff` — a text diffing and patching module backed by the `similar` crate. Provides unified diff generation, hunk-level inspection, edit application, and three-way merge. No backend trait — pure computation.

# Why

- Every agentic coding tool (Claude Code, Cursor, Aider, Copilot) does structured code editing via diffs. lx has `std/fs` for raw file I/O and `std/git` for git diffs, but no programmatic diff computation on arbitrary text.
- Agents editing code need to compute diffs between versions, apply patches, merge concurrent edits, and present changes for human review — all without git involvement.
- The `similar` crate is the Rust standard for diff computation (used by insta, cargo-nextest, etc.). Zero reason to hand-roll Myers algorithm.

# What Changes

**Cargo.toml dependency:** Add `similar = "2"` to `crates/lx/Cargo.toml` dependencies.

**New file `crates/lx/src/stdlib/diff.rs`:** Module entry with `build()` registering 5 functions. `diff.unified` computes a unified diff string between two texts. `diff.hunks` returns structured hunk records. `diff.apply` applies a unified diff to text. `diff.edits` applies a list of `{line replacement}` edit records to text. `diff.merge3` performs three-way merge from base/ours/theirs.

**Registration in `crates/lx/src/stdlib/mod.rs`:** Add `mod diff;`, add `"diff"` to `get_std_module` and `std_module_exists`.

**Test file `tests/98_diff.lx`:** Covers all 5 functions.

# Files Affected

- `crates/lx/Cargo.toml` — add `similar = "2"`
- `crates/lx/src/stdlib/diff.rs` — New file
- `crates/lx/src/stdlib/mod.rs` — Register module
- `tests/98_diff.lx` — New test file

# Task List

### Task 1: Add similar dependency and create diff.rs with unified and hunks

**Subject:** Create diff.rs with unified diff generation and hunk inspection

**Description:** Add `similar = "2"` to `crates/lx/Cargo.toml` under `[dependencies]`.

Create `crates/lx/src/stdlib/diff.rs`.

Imports: `std::sync::Arc`, `similar::{ChangeTag, TextDiff}`, `indexmap::IndexMap`, `num_bigint::BigInt`, `crate::backends::RuntimeCtx`, `crate::builtins::mk`, `crate::error::LxError`, `crate::record`, `crate::span::Span`, `crate::value::Value`.

`pub fn build() -> IndexMap<String, Value>`: register:
- `"unified"` → `bi_unified` arity 2
- `"hunks"` → `bi_hunks` arity 2
- `"apply"` → `bi_apply` arity 2
- `"edits"` → `bi_edits` arity 2
- `"merge3"` → `bi_merge3` arity 3

`fn bi_unified(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError>`: args[0] is old text (Str), args[1] is new text (Str). Create `TextDiff::from_lines(old, new)`. Call `diff.unified_diff().header("old", "new").to_string()`. Return `Ok(Value::Str(Arc::from(result)))`.

`fn bi_hunks(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError>`: args[0] is old text, args[1] is new text. Create `TextDiff::from_lines(old, new)`. Iterate `diff.grouped_ops(3)` (context size 3). For each group, build a Record with:
- `"old_start"` → `Value::Int(BigInt::from(group[0].old_range().start))` (first op in group)
- `"new_start"` → `Value::Int(BigInt::from(group[0].new_range().start))`
- `"changes"` → `Value::List` of change Records

For each op in the group, iterate `diff.iter_changes(&op)`. Each change becomes a Record:
- `"tag"` → `Value::Str` — match `ChangeTag::Equal` → "equal", `Insert` → "insert", `Delete` → "delete"
- `"value"` → `Value::Str(Arc::from(change.value()))`
- `"old_line"` → change old index as `Value::Int` or `Value::None`
- `"new_line"` → change new index as `Value::Int` or `Value::None`

Collect groups into `Value::List(Arc::new(hunks))`. Return `Ok(result)`.

**ActiveForm:** Creating diff.rs with unified diff and hunks

---

### Task 2: Add apply, edits, and merge3 functions

**Subject:** Add diff application and three-way merge functions to diff.rs

**Description:** Add three functions to `crates/lx/src/stdlib/diff.rs`:

`fn bi_apply(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError>`: args[0] is original text (Str), args[1] is unified diff string (Str). Parse the unified diff: split by newlines, skip header lines (starting with `---`, `+++`, `@@`). For `@@` lines, extract the old start/count via regex or manual parsing of `@@ -old_start,old_count +new_start,new_count @@`. Walk through the original text lines and diff lines simultaneously: lines starting with ` ` (context) → keep original line, advance both. Lines starting with `-` → skip original line, advance old only. Lines starting with `+` → insert new line, advance new only. Collect result lines, join with newlines. Return `Ok(Value::Ok(Box::new(Value::Str(...))))`. On parse errors, return `Ok(Value::Err(...))`.

`fn bi_edits(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError>`: args[0] is text (Str), args[1] is edits list (List of Records). Each edit Record has `"line"` (Int — 1-indexed line number) and `"text"` (Str — replacement text, empty string means delete). Split text into lines. Sort edits by line number descending (so indices don't shift). For each edit: if line is valid, replace that line with the edit text (or remove if empty). Join lines with newlines. Return `Ok(Value::Str(...))`.

`fn bi_merge3(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError>`: args[0] is base text, args[1] is ours, args[2] is theirs. Compute `TextDiff::from_lines(base, ours)` and `TextDiff::from_lines(base, theirs)`. Walk both diffs simultaneously: where only one side changed, take that change. Where both sides changed the same region, if the changes are identical, take one copy. If the changes differ, mark as conflict with `<<<<<<<`, `=======`, `>>>>>>>` markers. Return `Ok(Value::Ok(Box::new(Value::Str(...))))` on clean merge, `Ok(Value::Err(Box::new(record! { "text" => merged_with_markers, "conflicts" => Value::Int(conflict_count) })))` on conflicts.

For the merge3 implementation, the simplest correct approach: split all three texts into lines. Use `similar::capture_diff_slices` on base→ours and base→theirs. Walk the base line-by-line. For each base line, check if ours changed it and/or theirs changed it. Apply the change rules above. This doesn't need to be perfect — a correct line-level three-way merge is sufficient.

**ActiveForm:** Adding diff apply, edits, and merge3 functions

---

### Task 3: Register std/diff and write tests

**Subject:** Register diff module in mod.rs and write integration tests

**Description:** Edit `crates/lx/src/stdlib/mod.rs`:

Add `mod diff;` alongside the other module declarations (near `mod describe;`).

In `get_std_module`, add: `"diff" => diff::build(),` in the match arm.

In `std_module_exists`, add `| "diff"` to the matches! pattern.

Create `tests/98_diff.lx`:

```
use std/diff

old = "line1\nline2\nline3\n"
new = "line1\nmodified\nline3\nextra\n"

-- unified diff
ud = diff.unified old new
assert (ud | contains? "modified") "unified contains change"
assert (ud | contains? "-line2") "unified shows deletion"
assert (ud | contains? "+modified") "unified shows insertion"

-- hunks
hunks = diff.hunks old new
assert (hunks | len > 0) "hunks returns non-empty"
first_hunk = hunks.[0]
assert (first_hunk.changes | len > 0) "hunk has changes"

-- edits
text = "aaa\nbbb\nccc\n"
edited = diff.edits text [{line: 2  text: "BBB"}]
assert (edited | contains? "BBB") "edit applied"
assert (not (edited | contains? "bbb")) "old line replaced"

-- merge3 clean
base = "a\nb\nc\n"
ours = "a\nB\nc\n"
theirs = "a\nb\nC\n"
merged = diff.merge3 base ours theirs ^
assert (merged | contains? "B") "merge has ours"
assert (merged | contains? "C") "merge has theirs"

-- merge3 conflict
ours2 = "a\nX\nc\n"
theirs2 = "a\nY\nc\n"
conflict = diff.merge3 base ours2 theirs2
assert (type_of conflict == "Err") "conflicting merge returns Err"

log.info "98_diff: all passed"
```

Run `just diagnose` to verify compilation. Run `just test` to verify the test passes.

**ActiveForm:** Registering diff module and writing tests

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
mcp__workflow__load_work_item({ path: "work_items/STD_DIFF.md" })
```

Then call `next_task` to begin.
