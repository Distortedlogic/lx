# Goal

Wire `StdinStdoutUserBackend` into the CLI when stdin is a TTY, so `std/user` interactive functions (`confirm`, `choose`, `ask`) work in terminal sessions. Then replace the workgen justfile's `just --choose` audit picker with a native lx chooser program that uses `user.choose`.

# Why

- `RuntimeCtx::default()` hardcodes `NoopUserBackend` (backends/mod.rs:41). Every `lx run` invocation silently auto-approves confirms, picks the first option on choose, and returns empty on ask — regardless of whether a human is at the terminal. This makes `std/user` unusable for interactive workflows.
- The workgen justfile's `audit` recipe delegates to `just --choose` for audit selection (workgen/justfile:8). This works but means the chooser lives outside lx. With TTY detection landed, an lx program can do this natively using `user.choose`, eliminating the dependency on just's chooser and making the interaction composable with other lx logic.

# What changes

**crates/lx-cli/src/main.rs — parameterize `run` and add TTY detection in `run_file`:**

The `run` function (line 184) is called by both `run_file` (line 59) and `run_tests` (line 145). It currently constructs `RuntimeCtx::default()` internally (line 188). Change `run` to accept `ctx: Arc<RuntimeCtx>` as a third parameter instead of constructing it. Move TTY detection to `run_file`: check `std::io::stdin().is_terminal()` (from `std::io::IsTerminal`, stabilized in Rust 1.70), build `RuntimeCtx` with `user: Arc::new(StdinStdoutUserBackend)` when true, otherwise `RuntimeCtx::default()`. Pass the constructed ctx to `run`. In `run_tests`, pass `Arc::new(RuntimeCtx::default())` to `run` — tests must never block on interactive input. Do not change `run_agent` — it owns stdin for JSON-line protocol via `BufReader::new(stdin.lock())` (line 270), and `StdinStdoutUserBackend` reads stdin independently via `stdin().read_line()`, creating two competing consumers of the same fd with different buffering.

**workgen/audit.lx — native audit chooser:**

New lx program that discovers audit files, presents them via `user.choose`, then calls `workgen.run` with the selected audit. The program reads `RULES_DIR` and `WORK_ITEM_RULES` from env vars (passed by the justfile recipe using its absolute-path variables). It lists the rules directory with `fs.ls`, filters for filenames containing "audit" and ending in ".md", sorts, builds display labels by stripping the `.md` suffix, calls `user.choose` to pick one, reconstructs the full path by joining `rules_dir / chosen_filename`, then calls `workgen.run` with the audit path, work-item rules path, and root `.`.

**workgen/justfile — replace `just --choose` with lx:**

Change the `audit` recipe from `@just --justfile {{justfile()}} --choose` to pass `RULES_DIR` and `WORK_ITEM_RULES` as env vars and invoke `lx run {{LX}}/workgen/audit.lx`. Keep all the individual `audit-*` recipes unchanged — they're useful for scripted/CI invocations that don't need interactive selection.

# How it works

`std::io::IsTerminal` is a zero-cost check (single `isatty` syscall). `run_file` checks it and builds a `RuntimeCtx` with `StdinStdoutUserBackend` when true, which reads from stdin and writes prompts to stderr. When false (piped input, CI), `NoopUserBackend` stays in place so non-interactive invocations never block. `run_tests` always passes `RuntimeCtx::default()` (NoopUserBackend) since tests must be deterministic. The `run` function itself is backend-agnostic — it accepts whatever `ctx` the caller provides.

The audit chooser is a thin lx program — `fs.ls` + `filter`, map to labels, `user.choose`, call `workgen.run`. It replaces shell-level orchestration with language-level orchestration, which is the point of lx. Note: `std/fs` has no glob function; the program uses `fs.ls` to list the directory and filters with `contains?` and `ends?` builtins. `fs.ls` returns filenames only (not full paths), so the program reconstructs full paths by joining the directory prefix.

# Files affected

- `crates/lx-cli/src/main.rs` — Parameterize `run` to accept `ctx`, TTY detection in `run_file`, default ctx in `run_tests`
- `workgen/audit.lx` — New file: interactive audit chooser using `std/user`
- `workgen/justfile` — Change `audit` recipe to invoke `workgen/audit.lx`

# Task List

## Task 1: Add TTY detection to CLI run path

**Subject:** Use StdinStdoutUserBackend when stdin is a TTY
**ActiveForm:** Adding TTY detection to CLI run path

Edit crates/lx-cli/src/main.rs. Add `use std::io::IsTerminal;` to the imports. Change `fn run` (line 184) to accept a third parameter `ctx: Arc<RuntimeCtx>` and remove the internal `let ctx = Arc::new(lx::backends::RuntimeCtx::default());` line. In `run_file`, before calling `run`, build the ctx: if `std::io::stdin().is_terminal()`, construct `Arc::new(RuntimeCtx { user: Arc::new(StdinStdoutUserBackend), ..RuntimeCtx::default() })`, otherwise `Arc::new(RuntimeCtx::default())`. Pass it to `run`. In `run_tests`, pass `Arc::new(RuntimeCtx::default())` to each `run` call — tests must never block on interactive input. Do not change `run_agent` — it owns stdin for JSON-line protocol via `BufReader`, and `StdinStdoutUserBackend` would create a competing stdin reader causing data corruption.

Verify: run `just diagnose` and confirm it passes.

## Task 2: Create workgen audit chooser program

**Subject:** Native lx audit chooser using user.choose
**ActiveForm:** Creating workgen/audit.lx

Create workgen/audit.lx. The program: uses std/fs, std/user, std/env, and ./main : workgen. In +main: read `RULES_DIR` and `WORK_ITEM_RULES` from env vars via `env.get` (these are passed by the justfile recipe using its absolute-path variables). List the rules directory with `fs.ls rules_dir ^`, filter for filenames containing "audit" and ending in ".md" using `filter (f) { contains? "audit" f && ends? ".md" f }`, sort the list. If the filtered list is empty, emit `"no audit files found in {rules_dir}"` and return early. Build display labels by stripping the ".md" suffix. Call `user.choose "Select audit:" labels`. Map the chosen label back to the full path by joining `"{rules_dir}/{chosen}.md"`. Call `workgen.run` with the audit path, work-item rules path, and root ".".

Verify: run `just diagnose` and confirm it passes.

## Task 3: Update workgen justfile audit recipe

**Subject:** Replace just --choose with lx audit chooser
**ActiveForm:** Updating workgen justfile audit recipe

Edit workgen/justfile. Change the `audit` recipe body from `@just --justfile {{justfile()}} --choose` to `RULES_DIR={{RULES}} WORK_ITEM_RULES={{WORK_ITEM_RULES}} lx run {{LX}}/workgen/audit.lx`.

Verify: confirm the justfile parses by running `just --justfile workgen/justfile --list`.

## Task 4: Remove workaround entry

**Subject:** ~~Remove UserBackend workaround from WORKAROUNDS.md~~
**Status:** RESOLVED — agent/WORKAROUNDS.md was deleted (file no longer exists). No action needed.

Verify: run `just diagnose` and confirm it passes.

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.
