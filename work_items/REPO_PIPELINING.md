# Goal

Add `std/repo` — a git-worktree-aware multi-agent coordination module that lets a kernel agent manage concurrent access to a shared repository. Agents read the repo freely (shared access) and mutate isolated worktree copies (exclusive access). When an agent finishes, the kernel merges its worktree and notifies all active agents so they can rebase. The kernel controls lock granularity (file, folder, or entire repo) based on the pervasiveness of each change, pausing agents that would conflict. This is Rust's borrow checker model (`&T` shared reads, `&mut T` exclusive writes) applied to a git repository at runtime.

# Why

- Multi-agent repo work is currently serial — one agent at a time, or agents work on completely separate files with no coordination. There's no way for a kernel agent to orchestrate parallel work on overlapping paths with safe isolation.
- `std/workspace` solves concurrent editing of a single in-memory text buffer, but real code tasks span multiple files, require a real filesystem (for builds, tests, linters), and need git-grade merge semantics.
- Git worktrees are the natural isolation primitive — each agent gets a cheap, independent working directory that can diverge and merge back. This is how Claude Code already works (the `isolation: "worktree"` option on subagents), and lx should expose the same pattern as a first-class coordination primitive.
- Without lock coordination, parallel agents produce merge conflicts that require manual resolution. File/folder/repo-level locks prevent conflicts at the source — the kernel prevents two agents from touching the same paths simultaneously.
- The Rust borrow checker analogy gives lx programmers an intuitive mental model: shared reads are free, exclusive writes require a lock, the kernel enforces it.

# What Changes

**New file `crates/lx/src/stdlib/git_worktree.rs` — low-level git worktree commands:**

Three functions added to `std/git`: `git.worktree_add` (create a worktree at a path on a new branch), `git.worktree_remove` (remove a worktree), `git.worktree_list` (list all worktrees with path/branch/commit info). These wrap `git worktree add/remove/list` commands using the existing `run_git` helper from `git.rs`.

**New file `crates/lx/src/stdlib/repo.rs` — module entry, data structures, core functions:**

Defines all shared data structures: `Repospace` (root path, merge strategy, locks, worktrees, watchers), `WorktreeInfo` (path, branch, agent id, status), `WorktreeStatus` (Active/Merging/Done), `Lock` (scope, mode, holder), `LockScope` (File/Folder/Repo), `LockMode` (Read/Write). Static `REPOSPACES: LazyLock<DashMap<u64, Repospace>>`. The `build()` function registers all `repo.*` functions. Implements: `repo.create` (initialize a repospace over a git repo root), `repo.status` (return current state: root, active worktrees, locks), `repo.worktrees` (list active worktrees), `repo.locks` (list current locks), `repo.on_change` (register a callback fired after each merge).

**New file `crates/lx/src/stdlib/repo_lock.rs` — lock table and conflict detection:**

Lock conflict detection logic: `scope_overlaps` checks if two lock scopes overlap (File vs File: exact match; File vs Folder: file path starts with folder; Folder vs Folder: either is prefix of other; Repo vs anything: always overlaps). `check_conflicts` takes existing locks and a requested lock, returns conflicting locks. Read-Read: no conflict. Read-Write, Write-Read, Write-Write on overlapping scopes: conflict. `acquire_locks` adds locks to a repospace's lock list. `release_locks_for_agent` removes all locks held by a given agent. Builtins: `repo.lock` (acquire explicit lock, returns Err on conflict), `repo.unlock` (release all locks for an agent), `repo.try_lock` (non-blocking lock attempt, returns Ok or Err with conflict details).

**New file `crates/lx/src/stdlib/repo_worktree.rs` — worktree lifecycle with lock integration:**

`repo.checkout`: parse agent id and lock requests, call `check_conflicts`, if no conflicts then acquire locks + run `git worktree add` + record worktree info + return handle. `repo.submit`: find worktree, set status to Merging, run `git merge --no-edit <branch>` in repo root, if merge succeeds then remove worktree + delete branch + release locks + notify watchers + return Ok with changed files list; if merge conflicts then return Err with conflict paths. `repo.rebase`: run `git -C <worktree_path> rebase <main_branch>`, return Ok or Err. `repo.abandon`: remove worktree without merging, release locks, delete branch. `notify_watchers`: call all registered on_change callbacks with a change record `{agent files branch}`.

# How it works

The coordination model mirrors Rust's borrow rules enforced at runtime:

1. **Creating a repospace**: `repo.create` validates the root is a git repo, records the merge strategy ("rebase" or "merge"), and returns a handle (Record with `__repo_id`).

2. **Acquiring work**: An agent calls `repo.checkout` with its agent ID and desired locks. The kernel checks each requested lock against all existing locks. If a Read-Read pair overlaps, no conflict. Any other mode combination on overlapping scopes is a conflict, and `checkout` returns `Err` listing the conflicting locks. On success, locks are acquired atomically, `git worktree add .lx/worktrees/<agent>-<timestamp> -b <branch>` creates the isolated copy, and the handle is returned.

3. **Working in isolation**: The agent receives the worktree path and works there — editing files, running builds, running tests. Other agents continue working in their own worktrees. Reads of the main repo are always allowed (git objects are shared).

4. **Submitting work**: `repo.submit` always merges the agent's branch into the main branch via `git merge --no-edit`. If the kernel wants linear history, it calls `repo.rebase` on the worktree before submitting (making the merge a fast-forward). The worktree is removed, locks are released, and all registered `on_change` watchers are notified with `{agent: Str  files: [Str]  branch: Str}`.

5. **Reacting to changes**: Other active agents receive the notification via their `on_change` callback. They can call `repo.rebase` to pull in the new changes, or ignore the notification if the changed files don't overlap with their work.

6. **Pervasive changes**: When the kernel determines a change is pervasive (renames a widely-used type, changes a core interface), it requests a Repo-scope Write lock. This blocks all other checkout attempts and the kernel should wait for existing worktrees to submit or abandon before proceeding. The `repo.try_lock` function enables polling for lock availability.

The `run_git_in` helper runs git commands in a specific directory via `git -C <dir>`, avoiding CWD changes in the process.

# Files affected

- `crates/lx/src/stdlib/git_worktree.rs` — New file: `bi_worktree_add`, `bi_worktree_remove`, `bi_worktree_list`
- `crates/lx/src/stdlib/git.rs` — Add 3 entries to `build()` referencing `git_worktree::*`
- `crates/lx/src/stdlib/repo.rs` — New file: data structures, statics, `build()`, `bi_create`, `bi_status`, `bi_worktrees`, `bi_locks_fn`, `bi_on_change`, helpers
- `crates/lx/src/stdlib/repo_lock.rs` — New file: `scope_overlaps`, `check_conflicts`, `acquire_locks`, `release_locks_for_agent`, `bi_lock`, `bi_unlock`, `bi_try_lock`
- `crates/lx/src/stdlib/repo_worktree.rs` — New file: `bi_checkout`, `bi_submit`, `bi_rebase`, `bi_abandon`, `notify_watchers`
- `crates/lx/src/stdlib/mod.rs` — Register `mod git_worktree`, `mod repo`, `mod repo_lock`, `mod repo_worktree`, add `"repo"` to `get_std_module` and `std_module_exists`
- `tests/98_repo.lx` — New file: integration tests

# Task List

### Task 1: Add git worktree commands to std/git

**Subject:** Add git.worktree_add, git.worktree_remove, git.worktree_list to std/git

**Description:** Create `crates/lx/src/stdlib/git_worktree.rs` with three public functions:

`bi_worktree_add(args, span, _ctx)`: args[0] is path (Str via `as_str()`), args[1] is branch name (Str via `as_str()`). Run `run_git(&["worktree", "add", path, "-b", branch])`. Return `Ok(git_ok(Value::Unit))` on success, `Ok(git_err_from(&out))` on failure — same pattern as `bi_create_branch` in `git_branch.rs`.

`bi_worktree_remove(args, span, _ctx)`: args[0] is path (Str via `as_str()`). Run `run_git(&["worktree", "remove", path])`. Return `Ok(git_ok(Value::Unit))` on success, `Ok(git_err_from(&out))` on failure.

`bi_worktree_list(args, span, _ctx)`: args[0] is unit (ignored). Run `run_git(&["worktree", "list", "--porcelain"])`. Parse the porcelain output — each worktree is separated by a blank line, with lines `worktree <path>`, `HEAD <hash>`, `branch refs/heads/<name>` (or `detached`). Build Records via `record!` macro with `"path"` (Str), `"head"` (Str), `"branch"` (Str — strip `refs/heads/` prefix, or `"detached"` for bare/detached). Return `Ok(git_ok(Value::List(Arc::new(entries))))`.

Import `run_git`, `git_ok`, `git_err`, `git_err_from`, `str_val` from `super::git`. Use the same error handling pattern as `git_branch.rs`.

Add `mod git_worktree;` to `crates/lx/src/stdlib/mod.rs`.

In `crates/lx/src/stdlib/git.rs` `build()` function, add three entries:
- `m.insert("worktree_add".into(), mk("git.worktree_add", 2, super::git_worktree::bi_worktree_add));`
- `m.insert("worktree_remove".into(), mk("git.worktree_remove", 1, super::git_worktree::bi_worktree_remove));`
- `m.insert("worktree_list".into(), mk("git.worktree_list", 1, super::git_worktree::bi_worktree_list));`

**ActiveForm:** Adding git worktree commands to std/git

### Task 2: Create repo.rs with data structures and core functions

**Subject:** Create repo.rs with shared data structures, statics, helpers, and core repo functions

**Description:** Create `crates/lx/src/stdlib/repo.rs`.

Imports: `std::sync::atomic::{AtomicU64, Ordering}`, `std::sync::{Arc, LazyLock}`, `std::process::{Command, Output}`, `dashmap::DashMap`, `indexmap::IndexMap`, `num_bigint::BigInt`, `crate::backends::RuntimeCtx`, `crate::builtins::mk`, `crate::error::LxError`, `crate::record`, `crate::span::Span`, `crate::value::Value`.

Define the following `pub(super)` types:

`enum LockScope` with variants `File(String)`, `Folder(String)`, `Repo`. Derive `Clone`.

`enum LockMode` with variants `Read`, `Write`. Derive `Clone, Copy, PartialEq`.

`struct Lock` with pub fields `scope: LockScope`, `mode: LockMode`, `holder: String`. Derive `Clone`.

`enum WorktreeStatus` with variants `Active`, `Merging`, `Done`. Derive `Clone, Copy, PartialEq`.

`struct WorktreeInfo` with pub fields `path: String`, `branch: String`, `agent: String`, `status: WorktreeStatus`.

`struct Repospace` with pub fields `root: String`, `merge_strategy: String`, `locks: Vec<Lock>`, `worktrees: IndexMap<String, WorktreeInfo>` (keyed by agent id), `watchers: Vec<Value>`.

Static: `pub(super) static REPOSPACES: LazyLock<DashMap<u64, Repospace>> = LazyLock::new(DashMap::new);` and `static NEXT_ID: AtomicU64 = AtomicU64::new(1);`.

Helper `pub(super) fn repo_id(v: &Value, span: Span) -> Result<u64, LxError>`: extract `__repo_id` from a Record, same pattern as `workspace::ws_id`.

Helper `fn make_handle(id: u64, name: &str) -> Value`: return a Record with `__repo_id` (Int) and `name` (Str).

Helper `pub(super) fn run_git_in(dir: &str, args: &[&str]) -> std::io::Result<std::process::Output>`: run `Command::new("git").arg("-C").arg(dir).args(args).output()`.

`pub fn build() -> IndexMap<String, Value>`: register the following functions using `mk`:
- `"create"` → `bi_create` arity 2
- `"status"` → `bi_status` arity 1
- `"worktrees"` → `bi_worktrees` arity 1
- `"locks"` → `bi_locks_fn` arity 1
- `"on_change"` → `bi_on_change` arity 2
- `"checkout"` → `super::repo_worktree::bi_checkout` arity 2
- `"submit"` → `super::repo_worktree::bi_submit` arity 2
- `"rebase"` → `super::repo_worktree::bi_rebase` arity 2
- `"abandon"` → `super::repo_worktree::bi_abandon` arity 2
- `"lock"` → `super::repo_lock::bi_lock` arity 2
- `"unlock"` → `super::repo_lock::bi_unlock` arity 2
- `"try_lock"` → `super::repo_lock::bi_try_lock` arity 2

`bi_create(args, span, _ctx)`: args[0] is name (Str via `as_str()`), args[1] is opts Record. Extract `root` via `args[1].str_field("root")`, return `LxError::type_err` if missing. Extract `merge_strategy` via `args[1].str_field("merge_strategy")` defaulting to `"rebase"`. Validate root is a git repo by checking `run_git_in(root, &["rev-parse", "--is-inside-work-tree"])` succeeds; if not, return `Ok(Value::Err(...))`. Allocate ID via `NEXT_ID.fetch_add(1, Ordering::Relaxed)`, insert Repospace with empty locks/worktrees/watchers. Return `Ok(Value::Ok(Box::new(make_handle(id, name))))`.

`bi_status(args, span, _ctx)`: args[0] is handle. Get `repo_id`, look up repospace via `REPOSPACES.get(&id)`. Return `record!` with `"root"` (Str), `"worktree_count"` (`Value::Int(BigInt::from(repospace.worktrees.len()))`) , `"lock_count"` (`Value::Int(BigInt::from(repospace.locks.len()))`), `"merge_strategy"` (Str).

`bi_worktrees(args, span, _ctx)`: args[0] is handle. Get repospace. Iterate `repospace.worktrees.values()`, map each to `record!` with `"path"`, `"branch"`, `"agent"`, `"status"` (Str: match on WorktreeStatus → "active"/"merging"/"done"). Collect into `Value::List(Arc::new(vec))`.

`bi_locks_fn(args, span, _ctx)`: args[0] is handle. Get repospace. Iterate `repospace.locks`, map each to `record!` with `"scope"` (Str: match LockScope → "file"/"folder"/"repo"), `"path"` (Str — extract from `LockScope::File(p)` or `Folder(p)`, or `""` for Repo), `"mode"` (Str: match LockMode → "read"/"write"), `"holder"` (Str). Collect into `Value::List(Arc::new(vec))`.

`bi_on_change(args, span, _ctx)`: args[0] is handle, args[1] is handler (clone the Value). Get mutable repospace via `REPOSPACES.get_mut`. Push handler onto `repospace.watchers`. Return `Ok(Value::Unit)`.

**ActiveForm:** Creating repo.rs with data structures and core functions

### Task 3: Create repo_lock.rs with lock conflict detection and lock builtins

**Subject:** Create repo_lock.rs with scope overlap detection, conflict checking, and lock/unlock builtins

**Description:** Create `crates/lx/src/stdlib/repo_lock.rs`.

Imports: `std::sync::Arc`, `crate::backends::RuntimeCtx`, `crate::error::LxError`, `crate::record`, `crate::span::Span`, `crate::value::Value`. Use types from `super::repo::{Lock, LockMode, LockScope, Repospace, REPOSPACES}`.

`pub(super) fn scope_overlaps(a: &LockScope, b: &LockScope) -> bool`: Match on both scopes:
- `(File(a), File(b))` → `a == b`
- `(File(a), Folder(b))` → `a.starts_with(b)`
- `(Folder(a), File(b))` → `b.starts_with(a)`
- `(Folder(a), Folder(b))` → `a.starts_with(b) || b.starts_with(a)`
- `(Repo, _) | (_, Repo)` → `true`

`pub(super) fn check_conflicts(existing: &[Lock], requested: &Lock) -> Vec<Lock>`: Iterate existing locks. A conflict exists when `scope_overlaps` is true AND the mode pair is NOT (Read, Read). Clone and collect all conflicting locks into the result Vec.

`pub(super) fn acquire_lock(repospace: &mut Repospace, lock: Lock)`: Push the lock onto `repospace.locks`.

`pub(super) fn release_locks_for_agent(repospace: &mut Repospace, agent: &str)`: Retain only locks where `holder != agent`.

`pub(super) fn parse_lock_request(val: &Value, span: Span) -> Result<Lock, LxError>`: Parse a Record with fields `path` (Str), `scope` (Str: "file"/"folder"/"repo"), `mode` (Str: "read"/"write"), `holder` (Str). Convert scope string to `LockScope::File(path)`, `LockScope::Folder(path)`, or `LockScope::Repo`. Convert mode string to `LockMode::Read` or `LockMode::Write`. Return `LxError::type_err` on missing/invalid fields.

`pub fn bi_lock(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError>`: args[0] is repo handle, args[1] is lock request Record. Parse lock request via `parse_lock_request`. Get `repo_id`, get mutable reference to repospace via `REPOSPACES.get_mut(&id)`. Call `check_conflicts(&repospace.locks, &lock)`. If conflicts exist, build a conflict list — for each conflict, create a Record via `record!` with `scope` (Str), `holder` (Str), `mode` (Str) — and return `Ok(Value::Err(Box::new(record! { "conflicts" => Value::List(Arc::new(conflict_records)) })))`. If no conflicts, call `acquire_lock` and return `Ok(Value::Ok(Box::new(Value::Unit)))`.

`pub fn bi_unlock(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError>`: args[0] is repo handle, args[1] is agent id (Str). Get mutable repospace via `REPOSPACES.get_mut`. Call `release_locks_for_agent`. Return `Ok(Value::Ok(Box::new(Value::Unit)))`.

`pub fn bi_try_lock(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError>`: Same as `bi_lock` but explicitly documented as non-blocking. Implementation is identical — `bi_lock` is already non-blocking (returns immediately with Ok or Err). This is a semantic alias for clarity in lx code: `repo.lock` for "I expect this to succeed" vs `repo.try_lock` for "I'm polling for availability".

**ActiveForm:** Creating repo_lock.rs with lock conflict detection

### Task 4: Create repo_worktree.rs with checkout/submit/rebase/abandon

**Subject:** Create repo_worktree.rs with worktree lifecycle operations integrating locks and git

**Description:** Create `crates/lx/src/stdlib/repo_worktree.rs`.

Imports: `std::sync::Arc`, `std::time::{SystemTime, UNIX_EPOCH}`, `crate::backends::RuntimeCtx`, `crate::builtins::call_value_sync`, `crate::error::LxError`, `crate::record`, `crate::span::Span`, `crate::value::Value`. Use types from `super::repo::{REPOSPACES, WorktreeInfo, WorktreeStatus, repo_id, run_git_in}` and functions from `super::repo_lock::{acquire_lock, check_conflicts, parse_lock_request, release_locks_for_agent}`.

`pub fn bi_checkout(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError>`: args[0] is repo handle, args[1] is opts Record with `agent` (Str) and `locks` (List of lock request Records). Extract agent id via `args[1].str_field("agent")`. Extract locks list via `args[1].list_field("locks")`. Parse each lock request via `parse_lock_request`. Get mutable repospace via `REPOSPACES.get_mut`. For each parsed lock, call `check_conflicts(&repospace.locks, &lock)`. If any conflicts, return `Ok(Value::Err(...))` with conflict details. Generate timestamp: `SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis()`. Generate branch name: `format!("repo-{agent}-{ts}")`. Construct worktree path: `format!("{}/.lx/worktrees/{}-{}", repospace.root, agent, ts)`. Create the `.lx/worktrees/` directory if needed via `std::fs::create_dir_all`. Drop the DashMap ref before running git. Run `run_git_in(&root, &["worktree", "add", &wt_path, "-b", &branch])`. If git fails, return `Ok(Value::Err(...))` with stderr. Re-acquire mutable repospace. Acquire all locks via `acquire_lock`. Insert `WorktreeInfo { path, branch, agent, status: WorktreeStatus::Active }` into `repospace.worktrees` keyed by agent id. Return `Ok(Value::Ok(Box::new(record! { "path" => ..., "branch" => ..., "agent" => ..., "__worktree_agent" => ... })))`.

`pub fn bi_submit(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError>`: args[0] is repo handle, args[1] is worktree handle Record (with `__worktree_agent`). Get agent id via `args[1].str_field("__worktree_agent")`. Get mutable repospace. Find worktree in `repospace.worktrees` by agent. Verify status is Active; set to Merging. Clone root, branch, and wt_path from repospace/worktree into local Strings. Drop the DashMap ref (release the lock on REPOSPACES before running git — same scoping pattern as `workspace_edit.rs` `bi_edit`). Run `run_git_in(&root, &["merge", "--no-edit", &branch])` — always merge, regardless of merge_strategy (the kernel calls `repo.rebase` on worktrees before submit if it wants linear history). If merge fails (exit code != 0), re-acquire mutable repospace, set status back to Active, return `Ok(Value::Err(...))` with stderr. On success, get changed files via `run_git_in(&root, &["diff", "--name-only", "HEAD~1"])` — split stdout by newlines, filter empty, collect into `Vec<Value>` of `Value::Str`. Run `run_git_in(&root, &["worktree", "remove", &wt_path])`. Run `run_git_in(&root, &["branch", "-d", &branch])` (ignore errors — branch may already be cleaned up). Re-acquire mutable repospace. Call `release_locks_for_agent`. Remove worktree from `repospace.worktrees`. Clone `repospace.watchers` into a local `Vec<Value>`. Drop the DashMap ref. Iterate `&watchers`: for each `w`, call `call_value_sync(w, change_record.clone(), span, ctx)` where change_record is `record! { "agent" => ..., "files" => Value::List(Arc::new(files)), "branch" => ... }` (follow the pattern from `workspace_edit.rs` lines 117-167). Return `Ok(Value::Ok(Box::new(record! { "merged" => Value::Bool(true), "files" => ..., "conflicts" => Value::List(Arc::new(vec![])) })))`.

`pub fn bi_rebase(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError>`: args[0] is repo handle, args[1] is worktree handle. Get agent id via `str_field("__worktree_agent")`. Get repospace (read-only via `.get()`), look up worktree info for its path and the root. Clone root and wt_path into local Strings. Drop the DashMap ref. Get main branch name via `run_git_in(&root, &["rev-parse", "--abbrev-ref", "HEAD"])`, trim stdout. Run `run_git_in(&wt_path, &["rebase", &main_branch])`. Return `Ok(Value::Ok(Box::new(Value::Unit)))` on success, `Ok(Value::Err(...))` with stderr on conflict/failure.

`pub fn bi_abandon(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError>`: args[0] is repo handle, args[1] is worktree handle. Get agent id via `str_field("__worktree_agent")`. Get mutable repospace. Look up worktree info, clone root/wt_path/branch into local Strings. Call `release_locks_for_agent`. Remove worktree from `repospace.worktrees`. Drop the DashMap ref. Run `run_git_in(&root, &["worktree", "remove", "--force", &wt_path])`. Run `run_git_in(&root, &["branch", "-D", &branch])`. Return `Ok(Value::Ok(Box::new(Value::Unit)))`.

**ActiveForm:** Creating repo_worktree.rs with worktree lifecycle operations

### Task 5: Register std/repo module in stdlib/mod.rs

**Subject:** Register repo, repo_lock, repo_worktree modules in stdlib/mod.rs

**Description:** Edit `crates/lx/src/stdlib/mod.rs`:

Add `mod repo;`, `mod repo_lock;`, `mod repo_worktree;` alongside the other module declarations (near `mod workspace;` and `mod workspace_edit;`).

In the `get_std_module` function's match statement, add: `"repo" => repo::build(),`.

In the `std_module_exists` function's match statement, add `| "repo"` to the pattern list.

**ActiveForm:** Registering std/repo in stdlib/mod.rs

### Task 6: Write integration tests for std/repo

**Subject:** Write integration tests covering the full repo pipelining lifecycle

**Description:** Create `tests/98_repo.lx`. The test must create a temporary git repo (not modify the real lx repo), exercise the full lifecycle, and clean up.

Structure:

```
use std/git
use std/repo
use std/fs

-- Create a temp directory for the test repo
tmp = $^mktemp -d
tmp = tmp | trim

-- Initialize a git repo with an initial commit
$git -C {tmp} init
$echo "line1" > {tmp}/file_a.txt
$echo "line2" > {tmp}/file_b.txt
$git -C {tmp} add -A
$git -C {tmp} commit -m "initial"

-- Create repospace
rp = repo.create "test-repo" {root: tmp} ^
assert (type_of rp == "Record") "create returns Record"

-- Status check
s = repo.status rp
assert (s.root == tmp) "status has root"
assert (s.worktree_count == 0) "no worktrees initially"
assert (s.lock_count == 0) "no locks initially"

-- Checkout: creates worktree with locks
wt = repo.checkout rp {
  agent: "agent-1"
  locks: [{path: "file_a.txt" scope: "file" mode: "write" holder: "agent-1"}]
} ^
assert (wt.agent == "agent-1") "checkout returns agent"
assert (wt.path | len > 0) "checkout returns path"
assert (wt.branch | len > 0) "checkout returns branch"

-- Verify locks are acquired
locks = repo.locks rp
assert (locks | len == 1) "one lock acquired"
assert (locks.[0].holder == "agent-1") "lock holder is agent-1"

-- Verify worktree is listed
wts = repo.worktrees rp
assert (wts | len == 1) "one worktree active"
assert (wts.[0].agent == "agent-1") "worktree agent is agent-1"

-- Conflict detection: try to lock the same file with another agent
conflict_result = repo.try_lock rp {path: "file_a.txt" scope: "file" mode: "write" holder: "agent-2"}
assert (type_of conflict_result == "Err") "conflicting lock returns Err"

-- Non-conflicting lock succeeds (different file)
ok_result = repo.lock rp {path: "file_b.txt" scope: "file" mode: "write" holder: "agent-2"}
assert (type_of ok_result == "Ok") "non-conflicting lock succeeds"

-- Read lock on write-locked file conflicts
read_conflict = repo.try_lock rp {path: "file_a.txt" scope: "file" mode: "read" holder: "agent-3"}
assert (type_of read_conflict == "Err") "read on write-locked file conflicts"

-- Unlock agent-2
repo.unlock rp "agent-2"
locks2 = repo.locks rp
assert (locks2 | len == 1) "only agent-1 lock remains after unlock"

-- Make a change in the worktree and submit
$echo "modified by agent-1" > {wt.path}/file_a.txt
$git -C {wt.path} add -A
$git -C {wt.path} commit -m "agent-1 changes"

result = repo.submit rp wt ^
assert (type_of result == "Ok") "submit succeeds"

-- After submit: locks released, worktree removed
locks3 = repo.locks rp
assert (locks3 | len == 0) "all locks released after submit"
wts2 = repo.worktrees rp
assert (wts2 | len == 0) "no worktrees after submit"

-- Verify the merge happened: file_a.txt in main has new content
content = $^cat {tmp}/file_a.txt
assert (content | trim == "modified by agent-1") "merge applied to main"

-- Test abandon (checkout then abandon without merging)
wt2 = repo.checkout rp {
  agent: "agent-2"
  locks: [{path: "file_b.txt" scope: "file" mode: "write" holder: "agent-2"}]
} ^
$echo "agent-2 changes" > {wt2.path}/file_b.txt
$git -C {wt2.path} add -A
$git -C {wt2.path} commit -m "agent-2 changes"

repo.abandon rp wt2 ^
wts3 = repo.worktrees rp
assert (wts3 | len == 0) "no worktrees after abandon"
content2 = $^cat {tmp}/file_b.txt
assert (content2 | trim == "line2") "abandon did not merge changes"

-- Folder-scope lock test
repo.lock rp {path: "src/" scope: "folder" mode: "write" holder: "agent-x"}
file_in_folder = repo.try_lock rp {path: "src/main.rs" scope: "file" mode: "write" holder: "agent-y"}
assert (type_of file_in_folder == "Err") "file in locked folder conflicts"
repo.unlock rp "agent-x"

-- Repo-scope lock test
repo.lock rp {path: "" scope: "repo" mode: "write" holder: "agent-x"}
any_lock = repo.try_lock rp {path: "anything.txt" scope: "file" mode: "read" holder: "agent-y"}
assert (type_of any_lock == "Err") "repo lock blocks everything"
repo.unlock rp "agent-x"

-- Read-read does NOT conflict
repo.lock rp {path: "shared.txt" scope: "file" mode: "read" holder: "agent-a"}
read_read = repo.lock rp {path: "shared.txt" scope: "file" mode: "read" holder: "agent-b"}
assert (type_of read_read == "Ok") "read-read does not conflict"
repo.unlock rp "agent-a"
repo.unlock rp "agent-b"

-- Cleanup temp directory
$rm -rf {tmp}

log.info "98_repo: all passed"
```

Adjust the test to match the exact API as implemented. If `repo.lock` returns `Ok(Value::Unit)` wrapped in `Value::Ok`, then `type_of result` will be `"Ok"`. Use `^` to unwrap Results where needed.

**ActiveForm:** Writing integration tests for std/repo

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
mcp__workflow__load_work_item({ path: "work_items/REPO_PIPELINING.md" })
```

Then call `next_task` to begin.
