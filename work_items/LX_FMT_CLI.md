# Goal

Wire the existing formatter (`crates/lx/src/formatter/`) to the lx CLI as `lx fmt`. Single file mode (`lx fmt <file>`) formats one file. No-arg mode (`lx fmt`) formats all `.lx` files in the workspace. A `--check` flag exits non-zero if any file would change without writing (for CI).

# Why

LLM agents write lx programs and need to normalize their output before feeding it back into subsequent turns. The formatter code exists and works — `lx::formatter::format(&program)` takes a `Program<P>` and returns a `String` — but there is no CLI entry point. Without `lx fmt`, every agent-written program has inconsistent formatting that wastes context tokens on style noise.

# Verified facts

- The formatter handles ALL Expr and Stmt variants including Surface-phase constructs (Pipe, Ternary, Section, Coalesce, With). Confirmed in `emit_expr.rs` lines 37-73 — every variant has a match arm.
- `format<P>` has no constraints on `P` — it's a phantom type. Works on both `Program<Surface>` and `Program<Core>`.
- `collect_lx_files` in `check.rs` is **private** (`fn collect_lx_files(dir: &Path) -> Vec<PathBuf>` at line 170). Must be made `pub` for `fmt.rs` to use it.
- `run::read_and_parse` desugars to `Program<Core>`. The fmt command must NOT use this — it needs `Program<Surface>` to preserve pipes, ternaries, sections, etc.
- `lx::parser::parse(tokens)` returns `ParseResult` which contains `Option<Program<Surface>>` (the parser produces Surface AST).
- The `Program` struct is `Program<Phase = Surface> { stmts: Vec<StmtId>, arena: AstArena, _phase: PhantomData<Phase> }`.

# What changes

**New file `crates/lx-cli/src/fmt.rs`:** Two public functions — `fmt_file(path, check)` and `fmt_workspace(member_filter, check)`. The file-level function reads the source, lexes, parses to `Program<Surface>` (does NOT desugar), calls `lx::formatter::format`, and writes back if changed. The workspace function follows the same pattern as `check::check_workspace` — find root, load members, collect `.lx` files, format each, print summary.

**Modified `crates/lx-cli/src/check.rs`:** Change `collect_lx_files` from `fn` to `pub fn` so `fmt.rs` can use it.

**Modified `crates/lx-cli/src/main.rs`:** Add `Fmt` command variant and dispatch.

# Files affected

- NEW: `crates/lx-cli/src/fmt.rs`
- EDIT: `crates/lx-cli/src/main.rs` — add `mod fmt;`, add `Fmt` variant to `Command` enum, add dispatch
- EDIT: `crates/lx-cli/src/check.rs` — make `collect_lx_files` pub

# Task List

### Task 1: Make collect_lx_files public

**Subject:** Change collect_lx_files visibility from private to pub

**Description:** In `crates/lx-cli/src/check.rs`, at line 170, change:
```rust
fn collect_lx_files(dir: &Path) -> Vec<PathBuf>
```
to:
```rust
pub fn collect_lx_files(dir: &Path) -> Vec<PathBuf>
```

No other changes to the function.

**ActiveForm:** Making collect_lx_files public

### Task 2: Create fmt.rs with surface-level parse and format pipeline

**Subject:** Implement lx fmt file and workspace formatting

**Description:** Create `crates/lx-cli/src/fmt.rs` with the following two functions:

**`pub fn fmt_file(path: &str, check: bool) -> ExitCode`:**
1. Read file with `std::fs::read_to_string(path)` — on error, eprintln and return `ExitCode::from(1)`
2. Lex with `lx::lexer::lex(&source)` — on `Err`, print each error via miette `Report::new(err).with_source_code(NamedSource::new(path, source.clone()))`, return `ExitCode::from(1)`
3. Parse with `lx::parser::parse(tokens)` — this returns a `ParseResult`. If `result.program` is `None`, print the parse errors from `result.errors` the same way, return `ExitCode::from(1)`
4. Extract the `Program<Surface>` (do NOT desugar)
5. Call `lx::formatter::format(&program)` → `formatted: String`
6. Compare `formatted` to `source`:
   - If `check` is true and they differ: `eprintln!("would reformat {path}")`, return `ExitCode::from(1)`
   - If `check` is true and they match: return `ExitCode::SUCCESS`
   - If `check` is false and they differ: `std::fs::write(path, &formatted)`, print `"formatted {path}"` to stderr
   - If `check` is false and they match: do nothing (silent)
7. Return `ExitCode::SUCCESS`

**`pub fn fmt_workspace(member_filter: Option<&str>, check: bool) -> ExitCode`:**
1. Get cwd with `std::env::current_dir().unwrap()`
2. Call `crate::manifest::find_workspace_root(&cwd)` — if `None`, try `crate::manifest::find_manifest_root(&cwd)` — if still `None`, eprintln `"no lx.toml found"`, return 1
3. Load workspace with `crate::manifest::load_workspace(&root)` — on Err, print and return 1
4. Filter members by `member_filter` if provided: `members.iter().filter(|m| member_filter.is_none() || member_filter == Some(m.name.as_str()))`
5. For each member:
   - Call `crate::check::collect_lx_files(&member.dir)` to find all `.lx` files
   - For each file: run the same lex→parse→format pipeline as `fmt_file`
   - Track per-member: total files, formatted count, failed count (parse errors)
   - On parse failure: print the error, increment failed, continue (don't abort)
   - Print per-member summary: `eprintln!("{name:<16} {total} checked, {formatted} formatted, {failed} failed")`
6. Print totals
7. Return 1 if any failures or (in check mode) any files would change. Return 0 otherwise.

Note on ParseResult access: Check how `run.rs` accesses the parse result. It calls `lx::parser::parse(tokens)` which returns a `ParseResult`. The program is likely in `result.program` or accessed via pattern matching. Follow the same pattern as `run.rs:read_and_parse` but stop before the desugar call.

**ActiveForm:** Creating fmt.rs with surface parse and format pipeline

### Task 3: Add Fmt command to CLI and wire dispatch

**Subject:** Add lx fmt subcommand to clap and route to fmt.rs

**Description:** In `crates/lx-cli/src/main.rs`:

1. Add `mod fmt;` to the module declarations at the top of the file (alongside existing `mod check;`, `mod run;`, etc.).

2. Add a `Fmt` variant to the `Command` enum. The existing variants follow this pattern:
   ```rust
   Check { file: Option<String>, #[arg(long)] member: Option<String>, #[arg(long)] strict: bool },
   ```
   Add:
   ```rust
   Fmt {
       file: Option<String>,
       #[arg(long)]
       member: Option<String>,
       #[arg(long)]
       check: bool,
   },
   ```

3. In the `main()` function's match on `cli.command`, add the dispatch. Follow the exact pattern of the Check command routing:
   ```rust
   Command::Fmt { file, member, check } => {
       if let Some(path) = file {
           fmt::fmt_file(&path, check)
       } else {
           fmt::fmt_workspace(member.as_deref(), check)
       }
   },
   ```

**ActiveForm:** Adding Fmt command to CLI dispatch

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
mcp__workflow__load_work_item({ path: "work_items/LX_FMT_CLI.md" })
```

Then call `next_task` to begin.
