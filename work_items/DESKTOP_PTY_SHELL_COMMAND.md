# Goal

Change `common_pty::PtySession::spawn` to run shell commands via `sh -c` when the `command` parameter is `Some`, instead of treating the string as a bare program name.

# Why

`portable_pty::CommandBuilder::new(cmd)` treats the entire string as a program path. Passing `"claude -p 'hello' --session-id abc"` tries to execute a binary literally named that string, which doesn't exist. The voice pipeline (WU-D) needs to run full shell commands with arguments through terminal panes. All downstream work depends on this.

# Files Affected

| File | Change |
|------|--------|
| `common-pty/src/session.rs` | Change CommandBuilder construction for Some(cmd) |

Note: this file is in the `dioxus-common` repo at `/home/entropybender/repos/dioxus-common/crates/common-pty/src/session.rs`, NOT in the lx repo.

# Task List

### Task 1: Change CommandBuilder to use sh -c for command strings

**Subject:** Run command strings through sh -c instead of as bare program names

**Description:** Edit `/home/entropybender/repos/dioxus-common/crates/common-pty/src/session.rs`. Find lines 38-41 in the `spawn` function:

```rust
let mut cmd_builder = match command {
  Some(cmd) => CommandBuilder::new(cmd),
  None => CommandBuilder::new_default_prog(),
};
```

Replace with:

```rust
let mut cmd_builder = match command {
  Some(cmd) => {
    let mut b = CommandBuilder::new("sh");
    b.arg("-c");
    b.arg(cmd);
    b
  },
  None => CommandBuilder::new_default_prog(),
};
```

This runs `sh -c "{cmd}"` which lets the shell parse the full command string including arguments, pipes, and redirections. When `command` is `None`, behavior is unchanged — the default program (user's shell) is used.

Existing callers that pass `command: None` (the vast majority) are unaffected. Existing callers that pass `command: Some("bash")` or similar single-word programs still work because `sh -c "bash"` executes `bash`.

**ActiveForm:** Changing CommandBuilder to use sh -c for command strings

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

1. **Call `complete_task` after each task.**
2. **Call `next_task` to get the next task.**
3. **Do not add, skip, reorder, or combine tasks.**
4. **Tasks are implementation-only.**

---

## Task Loading Instructions

```
mcp__workflow__load_work_item({ path: "work_items/DESKTOP_PTY_SHELL_COMMAND.md" })
```
