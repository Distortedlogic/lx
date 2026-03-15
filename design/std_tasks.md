# std/tasks Design — PLANNED, NOT IMPLEMENTED

In-process task state machine with hierarchical subtasks and JSON persistence.

## Why Stdlib

Every multi-step agentic flow needs task tracking. The flow_full_pipeline uses workflow MCP tools (create_task, update_task, start_work_item). The agentic loop tracks progress. The agent lifecycle tracks developmental stages. The discovery system tracks candidates. Right now each flow reinvents this. A stdlib module standardizes the pattern.

## API

```
use std/tasks

store = tasks.load "tasks.json"
store = tasks.empty ()

id = tasks.create store {title: "fix auth bug"  tags: ["security"]}
sub_id = tasks.create store {title: "read token flow"  parent: id}

tasks.start store id
tasks.update store id {notes: "found the issue in validate.rs"}
tasks.submit store id {output: "patched validate.rs, added test"}
tasks.pass store id
tasks.fail store id {feedback: "missed the refresh token path"  failed_categories: ["completeness"]}
tasks.complete store id {result: "fixed in commit abc123"}

task = tasks.get store id
children = tasks.children store id
by_status = tasks.list store {status: "in_progress"}
all = tasks.list store {}

tasks.save store "tasks.json"
```

## Task Record Shape

```
Protocol Task = {
  id: Str
  title: Str
  status: Str
  parent: Str = ""
  tags: List = []
  notes: Str = ""
  output: Str = ""
  feedback: Str = ""
  result: Str = ""
  created_at: Str
  updated_at: Str
}
```

## Status Lifecycle

```
todo → in_progress → submitted → pending_audit → passed → complete
                                               → failed → revision → submitted → ...
```

Transitions:
- `tasks.start` — todo → in_progress
- `tasks.submit` — in_progress or revision → submitted
- `tasks.audit` — submitted → pending_audit (called by audit module)
- `tasks.pass` — pending_audit → passed
- `tasks.fail` — pending_audit → failed (with feedback)
- `tasks.revise` — failed → revision
- `tasks.complete` — passed → complete (with result)

Invalid transitions return `Err`. The state machine prevents skipping steps.

## Persistence Model

Auto-persist on every mutation. Every `create`, `start`, `update`, `submit`, `pass`, `fail`, `complete` writes to disk immediately. The store holds the file path and flushes after each operation.

Rationale: long-running agents crash. Losing task state means redoing work. The I/O cost of writing a JSON file per operation is negligible compared to the cost of an LLM re-running a task.

## Hierarchical Tasks

Tasks have an optional `parent` field. `tasks.children store id` returns all direct children. No depth limit. A parent task's status is informational — completing children does not auto-complete the parent. The orchestrating agent decides when the parent is done.

## Interaction with std/audit

`std/audit` evaluates a task's output and produces a structured result. The flow:

```
tasks.submit store id {output: draft}
result = audit.evaluate rubric {response: draft  context: ctx  task: task}
result.passed ? {
  true -> tasks.pass store id
  false -> tasks.fail store id {feedback: result.feedback  failed_categories: result.failed}
}
```

The task module doesn't call the audit module. They compose in user code. This keeps them independent — you can use tasks without auditing, or audit without the task state machine.

## Implementation

Backed by a `DashMap<String, Task>` with a file path for auto-persist. Each mutation locks the entry, updates it, serializes the full map to JSON, writes to disk. UUID generation for task IDs via timestamp + counter (no external crate needed).

Estimated: ~150 lines of Rust. One stdlib file.
