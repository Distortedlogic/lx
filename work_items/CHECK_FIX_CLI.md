# Goal

Add `--fix` flag to `lx check` that automatically applies all machine-applicable fixes from diagnostics and rewrites the source file.

# Why

LLM agents iterating on lx code get diagnostics with suggested fixes but currently have to interpret and apply them manually. `lx check --fix` closes the loop — the agent runs one command and all auto-fixable issues are applied. This reduces iteration cycles for mechanical fixes to zero.

Depends on SELF_CORRECTING_DIAGNOSTICS being completed first — that work item adds more `MachineApplicable` fixes which `--fix` will automatically pick up.

# Verified facts

- **Fix struct** (`diagnostics.rs`): `{ description: String, edits: Vec<TextEdit>, applicability: Applicability }`
- **TextEdit struct** (`diagnostics.rs`): `{ range: SourceSpan, replacement: String }` — note the field is `range`, not `span`
- **Applicability enum** (`diagnostics.rs`): `MachineApplicable`, `MaybeIncorrect`, `DisplayOnly`
- **Diagnostic struct** (`mod.rs`): `{ level: DiagLevel, kind: DiagnosticKind, span: SourceSpan, secondary: Vec<(SourceSpan, String)>, fix: Option<Fix> }`
- **check_file signature** (`check.rs`): `pub fn check_file(path: &str, strict: bool) -> ExitCode`
- **check_workspace signature** (`check.rs`): `pub fn check_workspace(member_filter: Option<&str>, strict: bool) -> ExitCode`
- **SourceSpan** (from miette): has `.offset()` and `.len()` methods
- **Currently only DuplicateImport has a MachineApplicable fix** — the suggest_fix method at diagnostics.rs builds a removal edit. After SELF_CORRECTING_DIAGNOSTICS, more fixes will exist.
- **The Diagnostic, Fix, TextEdit, Applicability types are all `pub`** in `checker::diagnostics`.

# What changes

**Modified `crates/lx-cli/src/main.rs`:** Add `fix: bool` field to the `Check` command variant.

**Modified `crates/lx-cli/src/check.rs`:** Add `apply_fixes` function, update `check_file` and `check_workspace` signatures to accept `fix: bool`, apply fixes when flag is set.

# Files affected

- EDIT: `crates/lx-cli/src/main.rs` — add `fix` field to Check variant, pass to check functions
- EDIT: `crates/lx-cli/src/check.rs` — add `apply_fixes`, update signatures, wire fix flow

# Task List

### Task 1: Add --fix flag to Check command

**Subject:** Add fix flag to CLI and thread through to check functions

**Description:** In `crates/lx-cli/src/main.rs`:

1. Add `#[arg(long)] fix: bool` to the `Check` variant in the `Command` enum. The current Check variant is:
   ```rust
   Check { file: Option<String>, #[arg(long)] member: Option<String>, #[arg(long)] strict: bool },
   ```
   Change to:
   ```rust
   Check { file: Option<String>, #[arg(long)] member: Option<String>, #[arg(long)] strict: bool, #[arg(long)] fix: bool },
   ```

2. In `main()`, update the Check dispatch. Find the current dispatch pattern (which destructures `file`, `member`, `strict`) and add `fix`. Pass `fix` to the check functions:
   - `check::check_file(&path, strict, fix)` (was `check::check_file(&path, strict)`)
   - `check::check_workspace(member.as_deref(), strict, fix)` (was `check::check_workspace(member.as_deref(), strict)`)

3. In `crates/lx-cli/src/check.rs`, update both function signatures:
   - `pub fn check_file(path: &str, strict: bool, fix: bool) -> ExitCode`
   - `pub fn check_workspace(member_filter: Option<&str>, strict: bool, fix: bool) -> ExitCode`
   For now, just thread `fix` through without using it — behavior is unchanged when `fix` is false.

**ActiveForm:** Adding --fix flag to Check CLI command

### Task 2: Implement fix application logic

**Subject:** Apply machine-applicable text edits from diagnostics to source files

**Description:** In `crates/lx-cli/src/check.rs`, add a function:

```rust
fn apply_fixes(source: &str, diagnostics: &[lx::checker::Diagnostic]) -> Option<String> {
    use lx::checker::diagnostics::Applicability;

    let mut edits: Vec<(usize, usize, &str)> = Vec::new();

    for diag in diagnostics {
        if let Some(ref fix) = diag.fix {
            if fix.applicability == Applicability::MachineApplicable {
                for edit in &fix.edits {
                    let start = edit.range.offset();
                    let end = start + edit.range.len();
                    edits.push((start, end, &edit.replacement));
                }
            }
        }
    }

    if edits.is_empty() {
        return None;
    }

    // Sort by start offset descending — apply from end of file backward
    // so earlier offsets don't shift
    edits.sort_by(|a, b| b.0.cmp(&a.0));

    // Check for overlapping spans — skip any edit that overlaps with the previous one
    let mut result = source.to_string();
    let mut last_start = usize::MAX;
    for (start, end, replacement) in &edits {
        if *end > last_start {
            // This edit overlaps with the previous one — skip it
            continue;
        }
        result.replace_range(*start..*end, replacement);
        last_start = *start;
    }

    Some(result)
}
```

Note: `Applicability` needs `PartialEq` derived. Check if it already has it — if not, add `#[derive(PartialEq)]` to the `Applicability` enum in `crates/lx/src/checker/diagnostics.rs`.

Also verify that `Diagnostic` is importable as `lx::checker::Diagnostic` (it's re-exported from the checker module). The `Fix` and `TextEdit` types are accessed through `diag.fix` (which is `Option<Fix>`), so they don't need separate imports.

**ActiveForm:** Implementing fix application from diagnostic text edits

### Task 3: Wire fix application into check_file and check_workspace

**Subject:** Apply fixes and rewrite files when --fix is set

**Description:** In `crates/lx-cli/src/check.rs`:

**Modify `check_file`:**

After the check produces diagnostics, if `fix` is true:
1. Call `apply_fixes(&source, &result.diagnostics)`
2. If `Some(fixed_source)` is returned:
   - Write `fixed_source` to the file with `std::fs::write(path, &fixed_source)`
   - `eprintln!("applied fixes to {}", path)`
   - Re-parse the fixed source: call `lx::lexer::lex(&fixed_source)` → `lx::parser::parse(tokens)` → `lx::folder::desugar(program)` → `lx::checker::check(&desugared, fixed_arc)`
   - Print remaining diagnostics (the ones that weren't auto-fixable)
   - Return exit code based on remaining error count
3. If `None` (no fixes applicable): proceed as normal, print all diagnostics

**Modify `check_workspace`:**

Same pattern per file:
- After checking each file, if `fix` is true, apply the fix-write-recheck cycle
- Track per member: `{fixed_files}` count alongside existing error/warning counts
- Include in summary: `"{name}: {fixed} fixed, {errors} remaining errors"`

**Edge case:** If re-parsing the fixed source fails (fix produced invalid syntax), print `"warning: fix produced invalid syntax in {path}, reverting"`, write original source back, and count it as a failure. This shouldn't happen with well-formed TextEdits but is a safety net.

**ActiveForm:** Wiring fix application into check file and workspace flows

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
mcp__workflow__load_work_item({ path: "work_items/CHECK_FIX_CLI.md" })
```

Then call `next_task` to begin.
