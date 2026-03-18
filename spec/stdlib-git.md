# Git Integration

**Status: IMPLEMENTED** (Session 43). 36 functions across 7 Rust files in `crates/lx/src/stdlib/git*.rs`. Test: `tests/64_git.lx`.

`std/git` provides structured access to git repositories. Every function returns parsed records instead of raw text, composing naturally with pipes, filter, and map.

## Problem

Coding agents live in git. Current approach is `$git status`, `$git diff`, etc. — raw text output that requires fragile regex parsing, inconsistent across git versions, and loses structure. An agent that wants "files modified in the last 3 commits by author X" must chain shell commands and parse multiple output formats.

Structured git access means:
- Status as categorized path records, not porcelain text
- Commits as records with hash, author, date, subject, body — not formatted strings
- Diffs as hunk records with line numbers and content — not unified diff text
- Blame as per-line attribution records — not column-aligned text
- Errors as typed `GitErr` variants — not exit codes

## `std/git`

### Repository Info

```
use std/git

status ()                -- GitStatus ^ GitErr
branch ()                -- Str ^ GitErr (current branch name)
branches ()              -- [{name: Str  current: Bool  remote: Maybe Str  ahead: Int  behind: Int}] ^ GitErr
remotes ()               -- [{name: Str  url: Str}] ^ GitErr
root ()                  -- Str ^ GitErr (repository root path)
is_repo ()               -- Bool (true if cwd is inside a git repo)
```

`GitStatus` record:
```
{
  branch: Str
  clean: Bool
  staged: [{path: Str  action: Str}]
  unstaged: [{path: Str  action: Str}]
  untracked: [Str]
  conflicts: [{path: Str  ours: Str  theirs: Str}]
}
```

Action values: `"added"`, `"modified"`, `"deleted"`, `"renamed"`, `"copied"`.

### History

```
log opts                 -- [Commit] ^ GitErr
show ref                 -- Commit ^ GitErr (single commit with diff)
blame path               -- [BlameLine] ^ GitErr
blame_range path from to -- [BlameLine] ^ GitErr
```

`log` options (all optional):
```
{
  n: Int                 -- max commits (default: 10)
  path: Str              -- filter to file/directory
  author: Str            -- filter by author (substring match)
  since: Str             -- date filter ("2024-01-01", "3 days ago")
  until: Str             -- date filter
  grep: Str              -- filter by commit message (substring)
  ref: Str               -- starting ref (default: HEAD)
  all: Bool              -- all branches (default: false)
}
```

`Commit` record:
```
{
  hash: Str
  short: Str
  author: Str
  email: Str
  date: Str
  subject: Str
  body: Str
  parents: [Str]
  diff: Maybe [FileDiff]
}
```

`diff` is `None` for `log`, populated for `show`.

`BlameLine` record:
```
{hash: Str  author: Str  date: Str  line: Int  content: Str}
```

### Diff

```
diff opts                -- [FileDiff] ^ GitErr
diff_stat opts           -- [DiffStat] ^ GitErr
```

`diff` options (all optional):
```
{
  staged: Bool           -- diff staged changes (default: false = unstaged)
  ref: Str               -- diff against ref ("HEAD~3", "main", "abc123")
  range: Str             -- diff between refs ("main..feature")
  path: Str              -- filter to file/directory
  context: Int           -- context lines (default: 3)
}
```

`FileDiff` record:
```
{
  path: Str
  old_path: Maybe Str
  status: Str
  hunks: [{
    old_start: Int
    old_count: Int
    new_start: Int
    new_count: Int
    header: Str
    lines: [{kind: Str  content: Str  old_line: Maybe Int  new_line: Maybe Int}]
  }]
}
```

Line `kind` values: `"add"`, `"delete"`, `"context"`.

`DiffStat` record:
```
{path: Str  additions: Int  deletions: Int}
```

### Search

```
grep pattern opts        -- [GrepHit] ^ GitErr
```

Options:
```
{
  ref: Str               -- search at ref (default: working tree)
  path: Str              -- restrict to path
  ignore_case: Bool      -- case-insensitive (default: false)
}
```

`GrepHit` record:
```
{path: Str  line: Int  content: Str}
```

### Operations

```
add paths                -- () ^ GitErr
commit msg               -- {hash: Str} ^ GitErr
commit_with opts         -- {hash: Str} ^ GitErr
tag name                 -- () ^ GitErr
tag_with name opts       -- () ^ GitErr
```

`commit_with` options:
```
{
  msg: Str
  author: Str            -- "Name <email>" (default: git config)
  amend: Bool            -- amend last commit (default: false)
  allow_empty: Bool      -- allow empty commit (default: false)
}
```

### Branching

```
create_branch name       -- () ^ GitErr
create_branch_at name ref -- () ^ GitErr
delete_branch name       -- () ^ GitErr
checkout ref             -- () ^ GitErr
checkout_create name     -- () ^ GitErr (checkout -b)
merge ref                -- MergeResult ^ GitErr
```

`MergeResult` record:
```
{
  fast_forward: Bool
  conflicts: [Str]
  merged: Bool
}
```

### Stash

```
stash ()                 -- () ^ GitErr
stash_with msg           -- () ^ GitErr
stash_pop ()             -- () ^ GitErr
stash_list ()            -- [{index: Int  msg: Str  branch: Str}] ^ GitErr
stash_drop index         -- () ^ GitErr
```

### Remote

```
fetch remote             -- () ^ GitErr
pull ()                  -- () ^ GitErr
push ()                  -- () ^ GitErr
push_with opts           -- () ^ GitErr
```

`push_with` options:
```
{
  remote: Str            -- default: "origin"
  branch: Str            -- default: current
  force: Bool            -- default: false
  set_upstream: Bool     -- default: false
}
```

### Error Type

```
GitErr = | NotARepo Str | RefNotFound Str | MergeConflict [Str]
       | DirtyWorkTree Str | CommandFailed Str | AuthFailed Str
```

## Patterns

### Review recent changes by an agent

```
use std/git

git.log {n: 20  author: "claude"} ^
  | filter (c) re.match? r/fix|refactor/ c.subject
  | map (c) {commit: c.short  msg: c.subject  files: git.diff_stat {ref: "{c.hash}~1..{c.hash}"} ^}
  | each (c) emit "{c.commit}: {c.msg} ({c.files | len} files)"
```

### Safe commit workflow

```
s = git.status () ^
s.clean ? true -> emit "nothing to commit"
git.add (s.unstaged | map (.path)) ^
git.commit "fix: resolve timeout in retry loop" ^
```

### Pre-merge conflict check

```
result = git.merge "feature/new-auth" ^
result.conflicts | len > 0 ? {
  true -> {
    emit "Conflicts in {result.conflicts | len} files:"
    result.conflicts | each (f) emit "  - {f}"
    git.merge_abort () ^
  }
  false -> emit "Merged cleanly"
}
```

### Blame-driven investigation

```
git.blame_range "src/auth.rs" 50 80 ^
  | group_by (.author)
  | to_list
  | sort_by (kv) kv.1 | len | rev
  | take 3
  | each (kv) emit "{kv.0}: {kv.1 | len} lines"
```

### Diff analysis for code review

```
git.diff {range: "main..HEAD"} ^
  | flat_map (.hunks)
  | flat_map (.lines)
  | filter (.kind == "add")
  | filter (l) re.match? r/unwrap\(\)|expect\(/ l.content
  | each (l) emit "potential panic: {l.content}"
```

## Implementation

`std/git` is a new stdlib module. Backend: shell out to `git` with `--porcelain`/`--format` flags for machine-parseable output, then parse into lx records.

Rationale for shell-over-libgit2: `git2-rs` (libgit2) is a large dependency that doesn't support all git features (e.g., `git grep`, some merge strategies). Shelling out to `git` with structured output flags (`--format=%(objectname)...`, `--porcelain=v2`, `-z`) is lighter and covers the full feature set.

### Key implementation details

- `status`: uses `git status --porcelain=v2 -z` for NUL-delimited, machine-parseable output
- `log`: uses `git log --format=` with custom format string, `%x00` as field separator
- `diff`: uses `git diff --no-color -U{context}` and parses unified diff format
- `blame`: uses `git blame --porcelain` which gives structured per-line output
- `grep`: uses `git grep -n` with optional `--cached` for ref searches
- All operations: capture stderr for error messages, map exit codes to `GitErr` variants

### Dependencies

- `ShellBackend` from `RuntimeCtx` (for `git` invocation)
- `std/re` (for parsing git output)

## Cross-References

- Shell commands: TICK.md (`$cmd` syntax)
- File system: [stdlib-modules.md](stdlib-modules.md) (`std/fs`)
- Agent workflows: [stdlib-agents.md](stdlib-agents.md) (agents using git for code tasks)
- Diff/patch: relates to `~>>?` streaming for incremental code review
