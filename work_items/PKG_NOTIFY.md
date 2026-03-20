# Goal

Add `pkg/kit/notify` — structured notification system for agents. Notifications carry severity, optional actions, and optional persistence. Built on top of `emit` with structured Records so hosts can render them appropriately.

# Why

- Agents need to signal status to users without blocking. `emit` is fire-and-forget text. `log.*` goes to stderr. Neither carries severity, actions, or indicators.
- Agentic IDEs show notification badges, toast messages, and action buttons. A structured notification Record lets the host render appropriately while the lx API stays simple.
- No new backend needed — `emit` with a well-known Record shape is sufficient. The host checks for `__notify` in emitted values and renders accordingly.

# What Changes

**New file `pkg/kit/notify.lx`:** Functions that emit structured notification Records.

- `notify.info msg` — informational notification
- `notify.warn msg` — warning
- `notify.error msg` — error
- `notify.success msg` — success/completion
- `notify.action msg actions` — notification with action buttons
- `notify.progress label current total` — progress indicator
- `notify.clear label` — clear a named notification

# Files Affected

- `pkg/kit/notify.lx` — New file
- `tests/104_notify.lx` — New test file

# Task List

### Task 1: Create pkg/kit/notify.lx

**Subject:** Create notify.lx with structured notification functions

**Description:** Create `pkg/kit/notify.lx`:

```
-- Notifications -- structured emit for agent-to-host status signals.
-- Emits Records with __notify marker. Host inspects emitted values for this shape.

+info = (msg) {
  emit {__notify: true  level: "info"  message: msg}
}

+warn = (msg) {
  emit {__notify: true  level: "warn"  message: msg}
}

+error = (msg) {
  emit {__notify: true  level: "error"  message: msg}
}

+success = (msg) {
  emit {__notify: true  level: "success"  message: msg}
}

+action = (msg actions) {
  emit {__notify: true  level: "info"  message: msg  actions: actions}
}

+progress = (label current total) {
  emit {__notify: true  level: "progress"  label: label  current: current  total: total}
}

+clear = (label) {
  emit {__notify: true  level: "clear"  label: label}
}

+toast = (msg duration_ms) {
  emit {__notify: true  level: "info"  message: msg  duration_ms: duration_ms}
}

+badge = (label count) {
  emit {__notify: true  level: "badge"  label: label  count: count}
}
```

**ActiveForm:** Creating notify.lx with structured notifications

---

### Task 2: Write tests for pkg/kit/notify

**Subject:** Write tests verifying notify functions emit correct shapes

**Description:** Create `tests/104_notify.lx`:

```
use pkg/kit/notify

-- These calls should not error (emit is fire-and-forget)
notify.info "test info"
notify.warn "test warning"
notify.error "test error"
notify.success "test success"
notify.action "deploy?" ["approve" "reject"]
notify.progress "building" 3 10
notify.clear "building"
notify.toast "done!" 3000
notify.badge "errors" 5

-- Verify functions exist and are callable
assert (type_of notify.info == "Fn") "info is a function"
assert (type_of notify.warn == "Fn") "warn is a function"
assert (type_of notify.error == "Fn") "error is a function"
assert (type_of notify.success == "Fn") "success is a function"
assert (type_of notify.action == "Fn") "action is a function"
assert (type_of notify.progress == "Fn") "progress is a function"
assert (type_of notify.clear == "Fn") "clear is a function"

log.info "104_notify: all passed"
```

Run `just test` to verify.

**ActiveForm:** Writing tests for notify package

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
mcp__workflow__load_work_item({ path: "work_items/PKG_NOTIFY.md" })
```

Then call `next_task` to begin.
