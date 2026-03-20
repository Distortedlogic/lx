# Goal

Add `pkg/kit/search` — a code search package wrapping ripgrep's `--json` output. Returns structured results (file, line, column, match text, context) without parsing shell output manually. Pure lx, no Rust changes.

# Why

- Every agentic coding tool has structured code search. Agents currently shell out to `$rg ...` and parse unstructured text output. ripgrep's `--json` flag produces structured JSON-line output — this package parses it into lx Records.
- Structured results enable programmatic filtering, sorting, and aggregation of search results — essential for agents navigating large codebases.

# What Changes

**New file `pkg/kit/search.lx`:** Functions wrapping `$rg --json` with structured output parsing.

- `search.content pattern opts` — search file contents, return list of `{file line col text context}` Records
- `search.files pattern opts` — find files matching a glob, return list of `{file}` Records
- `search.count pattern opts` — count matches per file, return list of `{file count}` Records
- `search.replace pattern replacement opts` — preview replacements, return list of `{file line old new}` Records

# Files Affected

- `pkg/kit/search.lx` — New file
- `tests/103_search.lx` — New test file

# Task List

### Task 1: Create pkg/kit/search.lx

**Subject:** Create search.lx with content, files, count, and replace functions

**Description:** Create `pkg/kit/search.lx`:

```
-- Code search -- structured wrapper around ripgrep --json output.
-- Returns lx Records instead of raw text. Requires rg (ripgrep) on PATH.

use std/json

+content = (pattern opts) {
  path = opts.path ?? "."
  type_flag = opts.type ?? ""
  glob_flag = opts.glob ?? ""
  case_flag = opts.case_insensitive ?? false

  cmd = "rg --json"
  cmd = case_flag ? "{cmd} -i" : cmd
  cmd = type_flag != "" ? "{cmd} --type {type_flag}" : cmd
  cmd = glob_flag != "" ? "{cmd} --glob '{glob_flag}'" : cmd
  cmd = "{cmd} '{pattern}' {path}"

  raw = ($^{cmd}) ?? ""
  raw == "" ? [] : {
    raw | lines
      | filter (line) line | len > 0
      | map (line) json.parse line ^
      | filter (obj) obj.type == "match"
      | map (obj) {
          file: obj.data.path.text
          line: obj.data.line_number
          col: obj.data.submatches.[0].start ?? 0
          text: obj.data.lines.text | trim
          submatches: obj.data.submatches | map (sm) {
            start: sm.start
            end: sm.end
            text: sm.match.text
          }
        }
  }
}

+files = (pattern opts) {
  path = opts.path ?? "."
  type_flag = opts.type ?? ""
  glob_flag = opts.glob ?? ""

  cmd = "rg --files --json"
  cmd = type_flag != "" ? "{cmd} --type {type_flag}" : cmd
  cmd = glob_flag != "" ? "{cmd} --glob '{glob_flag}'" : cmd
  cmd = pattern != "" ? "{cmd} '{pattern}' {path}" : "{cmd} {path}"

  raw = ($^rg --files {path}) ?? ""
  raw == "" ? [] : {
    raw | lines
      | filter (line) line | len > 0
      | map (line) {file: line | trim}
  }
}

+count = (pattern opts) {
  path = opts.path ?? "."
  type_flag = opts.type ?? ""

  cmd = "rg --count-matches --json"
  cmd = type_flag != "" ? "{cmd} --type {type_flag}" : cmd
  cmd = "{cmd} '{pattern}' {path}"

  raw = ($^{cmd}) ?? ""
  raw == "" ? [] : {
    raw | lines
      | filter (line) line | len > 0
      | map (line) json.parse line ^
      | filter (obj) obj.type == "summary"
      | flat_map (obj) {
          obj.data.stats ? {
            Ok s -> [{file: "total"  count: s.matched_lines}]
            _ -> []
          }
        }
  }
}

+replace = (pattern replacement opts) {
  path = opts.path ?? "."
  type_flag = opts.type ?? ""

  cmd = "rg --json -r '{replacement}'"
  cmd = type_flag != "" ? "{cmd} --type {type_flag}" : cmd
  cmd = "{cmd} '{pattern}' {path}"

  raw = ($^{cmd}) ?? ""
  raw == "" ? [] : {
    raw | lines
      | filter (line) line | len > 0
      | map (line) json.parse line ^
      | filter (obj) obj.type == "match"
      | map (obj) {
          file: obj.data.path.text
          line: obj.data.line_number
          old: obj.data.lines.text | trim
          new: obj.data.replacement.text ?? ""
        }
  }
}
```

Adjust the rg commands based on the actual `--json` output format. Test with `rg --json "pattern" .` to verify field paths. The key JSON fields from rg's output: `type` ("match"/"begin"/"end"/"summary"), `data.path.text`, `data.line_number`, `data.lines.text`, `data.submatches[].start/end/match.text`.

**ActiveForm:** Creating search.lx with structured ripgrep wrappers

---

### Task 2: Write tests for pkg/kit/search

**Subject:** Write integration tests for search functions

**Description:** Create `tests/103_search.lx`:

```
use pkg/kit/search

-- Search for a known pattern in the test file itself
results = search.content "search.content" {path: "tests/103_search.lx"}
assert (results | len > 0) "found matches in self"
first = results.[0]
assert (first.file == "tests/103_search.lx") "file path correct"
assert (first.line > 0) "line number positive"
assert (first.text | contains? "search.content") "match text contains pattern"

-- Search with type filter
rust_results = search.content "fn build" {path: "crates/lx/src/stdlib"  type: "rust"}
assert (rust_results | len > 0) "found rust matches"

-- Files search
lx_files = search.files "" {path: "tests"  glob: "*.lx"}
assert (lx_files | len > 0) "found .lx files"

-- Replace preview (dry run)
replacements = search.replace "search\\.content" "search.find" {path: "tests/103_search.lx"}
assert (replacements | len > 0) "found replacements"
first_rep = replacements.[0]
assert (first_rep.old | contains? "search.content") "old text has pattern"

log.info "103_search: all passed"
```

Run `just test` to verify.

**ActiveForm:** Writing tests for search package

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
mcp__workflow__load_work_item({ path: "work_items/PKG_SEARCH.md" })
```

Then call `next_task` to begin.
