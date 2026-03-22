# Rust Codebase Quality Audit — Round 2

## Goal

Fix violations identified by a fresh run of `rules/rust-audit.md` checks against the current codebase. This audit found actionable violations in: file size limits (2 parser files), swallowed errors in the store subsystem, backwards-compat serde attributes, a Cargo string shorthand, repeated string literals in sandbox and diag modules, string-typed dispatch where enums should exist (std modules, backend names), and a minor unused-binding pattern.

## Why

- `parser/expr.rs` (479 lines) and `parser/stmt.rs` (305 lines) exceed the 300-line hard limit from CLAUDE.md
- The store subsystem silently discards file-write, serialization, and deserialization errors — persist can corrupt data, load can silently return empty state
- Two serde attributes exist solely for backwards compatibility, violating the no-backwards-compat rule
- A Cargo dependency uses string shorthand notation instead of object form
- "sandbox: policy not found" is duplicated 6 times and "diag: context_stack underflow" 6 times — fragile and verbose
- `get_std_module` and `std_module_exists` independently maintain the same list of 12 module name strings — adding a new module requires updating two locations with no compiler help
- Backend names in BackendsSection are `Option<String>` matched against raw strings — no exhaustiveness checking, no typo prevention, manual "unknown backend" warnings instead of serde parse errors

## What changes

### File splits (300-line limit)
- `parser/expr.rs` (479 lines → ~225 + ~250): extract atom sub-parsers (`string_parser`, `list_parser`, `block_or_record_parser`, `looks_like_record`, `record_fields`, `map_parser`, `paren_parser`, `param_parser`, `section_op`, `with_parser` — lines 121-370) into new `parser/expr_atoms.rs`, keeping core framework (`ident`, `type_name`, `skip_semis`, `semi_sep`, `expr_parser`, `stmts_block`) and pratt chain (`dot_rhs`, `pratt_expr`) in `expr.rs`
- `parser/stmt.rs` (305 lines → ~160 + ~145): extract `type_def_parser`, `trait_parser`, `trait_body`, `TraitBodyItem` enum, `class_parser`, `ClassMember` enum (lines 160-305) into new `parser/stmt_decl.rs`

### Store error handling
- Change `persist()` signature to `fn persist(state: &StoreState, span: SourceSpan) -> Result<(), LxError>` — replace `unwrap_or_default()` and `let _ =` with `?` operator, propagate both serialization and IO errors
- Change `load_from_disk()` to return `Result<IndexMap<Sym, LxVal>, LxError>` — replace silent `IndexMap::new()` returns with error propagation
- Update callers: `bi_create` and `bi_load` (already return Result), and all mutation functions that call `persist()`

### Serde & Cargo cleanup
- `manifest.rs:13`: rename field `deps_table` to `deps`, remove `#[serde(rename = "deps")]`, update the one reference in `install.rs:31`
- `lockfile.rs:7`: remove `#[serde(default)]` from `LockFile.package`
- `Cargo.toml:52`: convert `smart-default = "0.7"` to `smart-default = { version = "0.7" }`

### Repeated string extraction
- Sandbox module: create `get_policy(id: u64, span: SourceSpan) -> Result<Ref<'static, u64, Policy>, LxError>` helper in `sandbox/mod.rs`, call it from all 6 sites (sandbox/mod.rs 4x, sandbox_exec.rs 1x, sandbox_scope.rs 1x)
- Diag module: extract `const CONTEXT_UNDERFLOW: &str = "diag: context_stack underflow"` at module level in `diag_walk.rs`, use it in all 6 `.expect()` calls (diag_walk.rs 5x, diag_walk_expr.rs 1x)

### StdModule enum
- Define `StdModule` enum in `stdlib/mod.rs` with variants: `Math`, `Fs`, `Env`, `Md`, `Introspect`, `Time`, `Cron`, `Diag`, `Sandbox`, `Store`, `Test`, `Trait`
- Derive `strum::EnumString` with `#[strum(serialize_all = "lowercase")]`
- Rewrite `get_std_module`: parse `path[1]` to `StdModule` via `str::parse()`, match on enum variants
- Rewrite `std_module_exists`: return `path[1].parse::<StdModule>().is_ok()`

### Backend enums
- Define in `manifest.rs`: `EmitBackend` (Noop, Stdout), `LogBackend` (Noop, Stderr), `AiBackend` (ClaudeCode), `HttpBackend` (Reqwest), `YieldBackend` (StdinStdout)
- Each enum derives `Deserialize` with appropriate `#[serde(rename_all = "...")]` — use `"lowercase"` for simple names, per-variant `#[serde(rename = "...")]` for kebab-case names like "claude-code" and "stdin-stdout"
- Change `BackendsSection` fields from `Option<String>` to `Option<EmitBackend>`, `Option<LogBackend>`, etc.
- Update `main.rs` `configure_backends`: match on enum variants directly, remove manual `eprintln!("warning: unknown ...")` messages (serde now rejects unknown values at parse time)

### Misc
- `interpreter/mod.rs:164`: change `let _ = name;` to destructure with `name: _` in the pattern

## How it works

**Parser splits** are mechanical — move functions and their `use` dependencies into the new file, add `mod` declaration in `parser/mod.rs`, make extracted functions `pub(super)` so the parent module can reference them. `expr_parser()` in `expr.rs` calls the atom parsers, so `expr.rs` will `use super::expr_atoms::*` (or individual imports).

**Store error propagation** makes `persist()` fallible. Every store mutation (`bi_set`, `bi_update`, `bi_remove`, `bi_clear`, `bi_merge`) already returns `Result<LxVal, LxError>`, so they add `persist(&s, span)?`. The `load_from_disk` change surfaces corrupt-file errors to `bi_create` and `bi_load` instead of silently starting with empty data.

**StdModule enum** eliminates the duplicated string list. `strum::EnumString` auto-generates `FromStr` implementation. Adding a new std module requires adding a variant — the compiler enforces the match arm in `get_std_module`.

**Backend enums** shift validation from manual string matching to serde deserialization. Invalid backend names produce a serde error at manifest parse time instead of a runtime warning. The match in `configure_backends` becomes exhaustive.

## Files affected

| File | Change |
|------|--------|
| `Cargo.toml` (root) | Convert smart-default to object notation |
| `crates/lx-cli/src/manifest.rs` | Rename deps_table → deps, remove serde(rename), add backend enums, change BackendsSection field types |
| `crates/lx-cli/src/install.rs` | Update `deps_table` reference to `deps` |
| `crates/lx-cli/src/lockfile.rs` | Remove serde(default) |
| `crates/lx-cli/src/main.rs` | Refactor configure_backends to match on enum variants |
| `crates/lx/src/stdlib/store/mod.rs` | Change persist() and load_from_disk() to return Result, fix all callers within file |
| `crates/lx/src/stdlib/sandbox/mod.rs` | Add get_policy() helper, replace 4 inline lookups |
| `crates/lx/src/stdlib/sandbox/sandbox_exec.rs` | Use get_policy() helper |
| `crates/lx/src/stdlib/sandbox/sandbox_scope.rs` | Use get_policy() helper |
| `crates/lx/src/stdlib/diag/diag_walk.rs` | Add CONTEXT_UNDERFLOW const, use in 5 expect() calls |
| `crates/lx/src/stdlib/diag/diag_walk_expr.rs` | Import and use CONTEXT_UNDERFLOW const |
| `crates/lx/src/stdlib/mod.rs` | Define StdModule enum, refactor get_std_module and std_module_exists |
| `crates/lx/src/parser/expr.rs` | Keep core framework + pratt chain, remove extracted atoms |
| `crates/lx/src/parser/expr_atoms.rs` | **New** — extracted atom sub-parsers |
| `crates/lx/src/parser/stmt.rs` | Keep program/stmt/use/binding parsers, remove extracted decl parsers |
| `crates/lx/src/parser/stmt_decl.rs` | **New** — extracted trait/class declaration parsers |
| `crates/lx/src/parser/mod.rs` | Add mod declarations for expr_atoms and stmt_decl |
| `crates/lx/src/interpreter/mod.rs` | Change NamedArg destructure to use `name: _` |

## Deferred to separate work items

- **Inline import paths (231 instances)** — massive mechanical cleanup, needs its own dedicated work item
- **Prelude creation** — depends on inline import cleanup
- **StoreMethod enum** — low impact, one call site, aliases add complexity
- **ResourceModule/ResourceAction enums** — internal to diag, one call site each
- **AstVisitor single-impl trait** — architectural choice, provides useful abstraction boundary

## Task List

### Task 1: Cargo & serde cleanup

**Subtask 1a: Fix smart-default string shorthand**

In `Cargo.toml` (project root), line 52, change `smart-default = "0.7"` to `smart-default = { version = "0.7" }`.

**Subtask 1b: Remove serde(default) from lockfile**

In `crates/lx-cli/src/lockfile.rs`, remove line 7 (`#[serde(default)]`) — the line directly above `pub package: Vec<LockedPackage>`.

**Subtask 1c: Rename deps_table to deps in manifest**

In `crates/lx-cli/src/manifest.rs`:
- Remove line 13 (`#[serde(rename = "deps")]`)
- Rename field `deps_table` to `deps` on line 14

In `crates/lx-cli/src/install.rs`:
- Line 31: change `manifest.deps_table` to `manifest.deps`

### Task 2: Format

Run the following command verbatim, exactly as written, with no modifications: `just fmt`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way.

### Task 3: Commit

Run the following command verbatim, exactly as written, with no modifications: `git add -A && git commit -m "fix: cargo string shorthand, remove serde backwards-compat attrs, rename deps_table to deps"`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way.

### Task 4: Store error handling

**Subtask 4a: Make persist() fallible**

In `crates/lx/src/stdlib/store/mod.rs`, change the `persist` function (line 55):
- Add `span: SourceSpan` parameter: `pub(super) fn persist(state: &StoreState, span: SourceSpan) -> Result<(), LxError>`
- Replace line 59 `let pretty = serde_json::to_string_pretty(&json_val).unwrap_or_default();` with: `let pretty = serde_json::to_string_pretty(&json_val).map_err(|e| LxError::runtime(format!("store: serialize failed: {e}"), span))?;`
- Replace line 60 `let _ = std::fs::write(path, pretty);` with: `std::fs::write(path, &pretty).map_err(|e| LxError::runtime(format!("store: write failed: {e}"), span))?;`
- Add `Ok(())` as return value

**Subtask 4b: Make load_from_disk() fallible**

In the same file, change `load_from_disk` (line 63):
- Change signature to `fn load_from_disk(path: &std::path::Path, span: SourceSpan) -> Result<IndexMap<crate::sym::Sym, LxVal>, LxError>`
- Replace `let Ok(content) = std::fs::read_to_string(path) else { return IndexMap::new(); };` with: `let content = std::fs::read_to_string(path).map_err(|e| LxError::runtime(format!("store: read failed: {e}"), span))?;`
- Replace `let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&content) else { return IndexMap::new(); };` with: `let json_val: serde_json::Value = serde_json::from_str(&content).map_err(|e| LxError::runtime(format!("store: parse failed: {e}"), span))?;`
- Wrap final match result in `Ok(...)`: return `Ok(r.as_ref().clone())` for the Record case and `Ok(IndexMap::new())` for the wildcard case

**Subtask 4c: Update all callers**

In `crates/lx/src/stdlib/store/mod.rs`:
- `bi_create` line 85: change `path.as_deref().map(load_from_disk).unwrap_or_default()` to `match path.as_deref() { Some(p) => load_from_disk(p, span)?, None => IndexMap::new() }`
- `bi_set` line 96: change `persist(&s);` to `persist(&s, span)?;`
- `bi_update` line 118: change `persist(&s);` to `persist(&s, span)?;`
- `bi_remove` line 127: change `persist(&s);` to `persist(&s, span)?;`
- `bi_clear` line 184: change `persist(&s);` to `persist(&s, span)?;`
- `bi_persist` line 191: change `persist(&s);` to `persist(&s, span)?;`
- `bi_load` line 199: change `s.data = load_from_disk(path);` to `s.data = load_from_disk(path, span)?;`

### Task 5: Format

Run the following command verbatim, exactly as written, with no modifications: `just fmt`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way.

### Task 6: Commit

Run the following command verbatim, exactly as written, with no modifications: `git add -A && git commit -m "fix: propagate store persist and load errors instead of swallowing"`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way.

### Task 7: Extract repeated string literals

**Subtask 7a: Sandbox get_policy helper**

In `crates/lx/src/stdlib/sandbox/mod.rs`, add a `get_policy` helper function (place it after `policy_id`, around line 42):

```
pub(super) fn get_policy(id: u64, span: SourceSpan) -> Result<dashmap::mapref::one::Ref<'static, u64, Policy>, LxError> {
    POLICIES.get(&id).ok_or_else(|| LxError::runtime("sandbox: policy not found", span))
}
```

Then replace all 4 inline `POLICIES.get(&id).ok_or_else(...)` calls in the same file with `get_policy(id, span)?`.

In `crates/lx/src/stdlib/sandbox/sandbox_exec.rs`:
- Add import: `use super::sandbox::get_policy;`
- Line 22: replace `POLICIES.get(&pid).ok_or_else(|| LxError::runtime("sandbox: policy not found", span))?` with `get_policy(pid, span)?`

In `crates/lx/src/stdlib/sandbox/sandbox_scope.rs`:
- Add import: `use super::sandbox::get_policy;`
- Line 35: replace `POLICIES.get(&pid).ok_or_else(|| LxError::runtime("sandbox: policy not found", span))?.clone()` with `get_policy(pid, span)?.clone()`

**Subtask 7b: Diag context_stack underflow const**

In `crates/lx/src/stdlib/diag/diag_walk.rs`, add at the top of the file (after imports):

```
const CONTEXT_UNDERFLOW: &str = "diag: context_stack underflow";
```

Replace all 5 `.expect("diag: context_stack underflow")` calls in that file with `.expect(CONTEXT_UNDERFLOW)`.

In `crates/lx/src/stdlib/diag/diag_walk_expr.rs` (which is a `#[path]` submodule of `diag_walk.rs`, so `super::` refers to `diag_walk`):
- Add import: `use super::CONTEXT_UNDERFLOW;`
- In `diag_walk.rs`, change the const visibility to `pub(super) const CONTEXT_UNDERFLOW`
- Replace the `.expect("diag: context_stack underflow")` on line 103 with `.expect(CONTEXT_UNDERFLOW)`

### Task 8: Format

Run the following command verbatim, exactly as written, with no modifications: `just fmt`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way.

### Task 9: Commit

Run the following command verbatim, exactly as written, with no modifications: `git add -A && git commit -m "refactor: extract sandbox get_policy helper and diag context underflow const"`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way.

### Task 10: StdModule enum

In `crates/lx/src/stdlib/mod.rs`:

Add `use strum::EnumString;` to imports.

Define the enum before `get_std_module`:

```
#[derive(EnumString)]
#[strum(serialize_all = "lowercase")]
enum StdModule {
    Math,
    Fs,
    Env,
    Md,
    Introspect,
    Time,
    Cron,
    Diag,
    Sandbox,
    Store,
    Test,
    Trait,
}
```

Rewrite `get_std_module`:

```
pub(crate) fn get_std_module(path: &[&str]) -> Option<ModuleExports> {
    if path.len() < 2 || path[0] != "std" {
        return None;
    }
    let module: StdModule = path[1].parse().ok()?;
    let bindings = match module {
        StdModule::Math => math::build(),
        StdModule::Fs => fs::build(),
        StdModule::Env => env::build(),
        StdModule::Md => md::build(),
        StdModule::Introspect => introspect::build(),
        StdModule::Time => time::build(),
        StdModule::Cron => cron::build(),
        StdModule::Diag => diag::build(),
        StdModule::Sandbox => sandbox::build(),
        StdModule::Store => store::build(),
        StdModule::Test => test::build(),
        StdModule::Trait => trait_ops::build(),
    };
    Some(ModuleExports { bindings, variant_ctors: Vec::new() })
}
```

Rewrite `std_module_exists`:

```
pub(crate) fn std_module_exists(path: &[&str]) -> bool {
    path.len() >= 2 && path[0] == "std" && path[1].parse::<StdModule>().is_ok()
}
```

### Task 11: Format

Run the following command verbatim, exactly as written, with no modifications: `just fmt`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way.

### Task 12: Commit

Run the following command verbatim, exactly as written, with no modifications: `git add -A && git commit -m "refactor: replace std module string dispatch with StdModule enum"`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way.

### Task 13: Backend enums

**Subtask 13a: Define backend enums in manifest.rs**

In `crates/lx-cli/src/manifest.rs`, add after the existing imports:

```
use serde::Deserialize;
```

(Already imported — just ensure it's present.)

Define the following enums before `BackendsSection`:

```
#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EmitBackend {
    Noop,
    Stdout,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogBackend {
    Noop,
    Stderr,
}

#[derive(Deserialize)]
pub enum AiBackend {
    #[serde(rename = "claude-code")]
    ClaudeCode,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HttpBackend {
    Reqwest,
}

#[derive(Deserialize)]
pub enum YieldBackend {
    #[serde(rename = "stdin-stdout")]
    StdinStdout,
}
```

**Subtask 13b: Change BackendsSection to use enums**

In the same file, change `BackendsSection` fields:
- `pub ai: Option<String>` → `pub ai: Option<AiBackend>`
- `pub http: Option<String>` → `pub http: Option<HttpBackend>`
- `pub emit: Option<String>` → `pub emit: Option<EmitBackend>`
- `pub yield_backend: Option<String>` → `pub yield_backend: Option<YieldBackend>`
- `pub log: Option<String>` → `pub log: Option<LogBackend>`

**Subtask 13c: Update main.rs backend matching**

In `crates/lx-cli/src/main.rs`, update the `apply_manifest_backends` function (starting at line 160). Replace each `if let Some(ref name) = backends.X { match name.as_str() { ... } }` block with a direct match on the enum. For example, the emit block becomes:

```
if let Some(backend) = backends.emit {
    match backend {
        manifest::EmitBackend::Noop => ctx.emit = Arc::new(NoopEmitBackend),
        manifest::EmitBackend::Stdout => {},
    }
}
```

Apply the same pattern for all 5 backends (emit, log, ai, http, yield). Remove all `eprintln!("warning: unknown ... backend")` branches — serde now rejects unknown values at parse time. The yield block uses `backends.yield_backend` and matches on `manifest::YieldBackend::StdinStdout => {}`.

### Task 14: Format

Run the following command verbatim, exactly as written, with no modifications: `just fmt`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way.

### Task 15: Commit

Run the following command verbatim, exactly as written, with no modifications: `git add -A && git commit -m "refactor: replace backend string dispatch with typed enums"`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way.

### Task 16: Split parser/expr.rs

In `crates/lx/src/parser/expr.rs`:

1. Create new file `crates/lx/src/parser/expr_atoms.rs`
2. Move the following functions from `expr.rs` into `expr_atoms.rs`: `string_parser` (line 121), `list_parser` (line 138), `block_or_record_parser` (line 156), `looks_like_record` (line 183), `record_fields` (line 190), `map_parser` (line 208), `paren_parser` (line 237), `param_parser` (line 294), `section_op` (line 310), `with_parser` (line 336)
3. In `expr_atoms.rs`, add the necessary imports at the top — copy relevant `use` statements from `expr.rs` (chumsky imports, ast types, lexer token types, super module items). Make all moved functions `pub(super)`
4. In `expr.rs`, remove the moved functions. Add `use super::expr_atoms::*;` (or individual imports) so `expr_parser()` can still call the atom parsers
5. In `crates/lx/src/parser/mod.rs`, add `mod expr_atoms;`
6. Verify both files are under 300 lines

### Task 17: Split parser/stmt.rs

In `crates/lx/src/parser/stmt.rs`:

1. Create new file `crates/lx/src/parser/stmt_decl.rs`
2. Move the following functions and helper enums from `stmt.rs` into `stmt_decl.rs`: `type_def_parser` (line 160), `trait_parser` (line 180), `trait_body` (line 209), `TraitBodyItem` enum (lines 245-250), `class_parser` (line 252), `ClassMember` enum (lines 301-305)
3. In `stmt_decl.rs`, add necessary imports — copy relevant `use` statements from `stmt.rs` (ast types like `ClassDeclData`, `TraitDeclData`, `ClassField`, `AgentMethod`, `FieldDecl`, `TraitEntry`, `TraitMethodDecl`, `TraitUnionDef`, etc.). Make all moved functions `pub(super)`
4. In `stmt.rs`, remove the moved functions. Add `use super::stmt_decl::*;` so `stmt_parser()` can still reference them
5. In `crates/lx/src/parser/mod.rs`, add `mod stmt_decl;`
6. Verify both files are under 300 lines

### Task 18: Fix NamedArg unused binding

In `crates/lx/src/interpreter/mod.rs`, line 163, change:

```
Expr::NamedArg { name, value } => {
    let _ = name;
    self.eval(value).await
},
```

to:

```
Expr::NamedArg { name: _, value } => self.eval(value).await,
```

### Task 19: Format

Run the following command verbatim, exactly as written, with no modifications: `just fmt`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way.

### Task 20: Commit

Run the following command verbatim, exactly as written, with no modifications: `git add -A && git commit -m "refactor: split parser files below 300-line limit, fix NamedArg unused binding"`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way.

### Task 21: Run tests

Run the following command verbatim, exactly as written, with no modifications: `just test`. Do NOT pipe, redirect, or append shell operators. If any tests fail, fix all failures and re-run until all tests pass.

### Task 22: Run diagnostics

Run the following command verbatim, exactly as written, with no modifications: `just diagnose`. Do NOT pipe, redirect, or append shell operators. Fix ALL reported errors AND warnings. Re-run `just diagnose` until the output is completely clean with zero errors and zero warnings.

### Task 23: Final format

Run the following command verbatim, exactly as written, with no modifications: `just fmt`. Do NOT pipe, redirect, or append shell operators.

### Task 24: Final commit

Run the following command verbatim, exactly as written, with no modifications: `git add -A && git commit -m "fix: final verification cleanup"`. Do NOT pipe, redirect, or append shell operators. CRITICAL: Do NOT use `$()`, heredocs, `cat <<EOF`, or any subshell substitution — just a plain `-m "message"` flag. Do NOT append `--trailer`, `Co-authored-by`, `Signed-off-by`, or any other trailer/metadata.

### Task 25: Remove work item file

Run the following command verbatim, exactly as written, with no modifications: `rm work_items/RUST_AUDIT_ROUND_2.md && git add -A && git commit -m "chore: remove completed work item"`. Do NOT pipe, redirect, or append shell operators. Do NOT append `--trailer`, `Co-authored-by`, `Signed-off-by`, or any other trailer/metadata.

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

These are rules you (the executor) are known to violate. Re-read before starting each task:

1. **NEVER run raw cargo commands.** No `cargo check`, `cargo test`, `cargo clippy`, `cargo fmt`. ALWAYS use `just fmt`, `just test`, `just diagnose`, `just fix`. The `just` recipes include additional steps that raw `cargo` skips.
2. **Between implementation tasks, ONLY run `just fmt` + `git commit`.** Do NOT run `just test`, `just diagnose`, or any compilation/verification command between implementation tasks. These are expensive and only belong in the FINAL VERIFICATION section at the end.
3. **Run commands VERBATIM.** Copy-paste the exact command from the task description. Do not modify, "improve," or substitute commands. Do NOT append `--trailer`, `Co-authored-by`, or any metadata to commit commands.
4. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
5. **`just fmt` is not `cargo fmt`.** `just fmt` runs `dx fmt` + `cargo fmt` + `eclint`. Running `cargo fmt` alone is wrong.
6. **Do NOT modify verbatim commands.** No piping (`| tail`, `| head`, `| grep`), no redirects (`2>&1`, `> /dev/null`), no subshells. Run the exact command string as written — nothing appended, nothing prepended.

---

## Task Loading Instructions

To load and execute this work item in a fresh session:

1. **How to load:** Read each `### Task N:` entry from the `## Task List` section above. For each task, call `TaskCreate` with:
   - `subject`: The task heading text (after `### Task N:`) — copied VERBATIM, not paraphrased
   - `description`: The full body text under that heading (including all subtasks) — copied VERBATIM, not paraphrased, summarized, or reworded. Every sentence, every command, every instruction must be transferred exactly as written. Do NOT omit lines, rephrase instructions, drop the "verbatim" language from command instructions, or inject your own wording.
   - `activeForm`: A present-continuous form of the subject (e.g., "Running tests")
2. **Dependency ordering:** After creating all tasks, use `TaskUpdate` to set `addBlockedBy` on each task N (N > 1) pointing to task N-1, enforcing strict sequential execution.
3. **Execution rules** (follow these verbatim):
   - Execute tasks strictly in order — mark each `in_progress` before starting and `completed` when done
   - Run commands EXACTLY as written in the task description — do not substitute `cargo` for `just` or vice versa
   - Do not run any command not specified in the current task
   - Do not "pre-check" compilation between implementation tasks — the task list already has verification in the correct places
   - If a task says "Run the following command verbatim" then copy-paste that exact command — do not modify it. Do NOT append `--trailer`, `Co-authored-by`, `Signed-off-by`, or any other trailer/metadata to git commit commands.
   - Do NOT paraphrase, summarize, reword, combine, split, reorder, skip, or add tasks beyond what is in the Task List section
   - When a task description says "Run the following command verbatim, exactly as written, with no modifications" — that phrase and the command after it must appear identically in the loaded task. Do not drop the "verbatim" instruction or rephrase the command.
   - Do NOT append shell operators to commands — no pipes (`|`), no redirects (`>`/`2>&1`), no subshells. The command in the task description is the complete command string.
