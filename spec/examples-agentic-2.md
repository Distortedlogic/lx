# Examples — Agentic Features (Part 2)

Worked examples for dialogue, plan revision, interceptors, handoff, introspection, and knowledge sharing. See [examples-agentic.md](examples-agentic.md) for streaming, checkpoint, blackboard, negotiation, and capability examples.

## Multi-Turn Dialogue: Iterative Code Review

```
use std/agent

+main = () {
  reviewer = agent.spawn {command: "reviewer" args: []} ^
  session = agent.dialogue reviewer {role: "code-reviewer" context: "lx codebase, auth module"} ^

  overview = agent.dialogue_turn session "give me a high-level overview of src/auth/" ^
  emit "Overview: {overview.summary}"

  deep = agent.dialogue_turn session "focus on the token refresh logic — any race conditions?" ^
  deep.issues | each (issue) emit "ISSUE: {issue.file}:{issue.line} — {issue.msg}"

  fix = agent.dialogue_turn session "propose a fix for the most critical issue" ^
  emit "Proposed fix:\n{fix.patch}"

  agent.dialogue_end session
  agent.kill reviewer ^
}
```

Uses: `agent.dialogue` for multi-turn conversation where each turn builds on accumulated context. The reviewer sees the full conversation history, not isolated questions.

## Dynamic Plan Revision

```
use std/plan
use std/agent

+main = () {
  steps = [
    {id: "scan" action: "scan for vulnerabilities" depends: []}
    {id: "classify" action: "classify by severity" depends: ["scan"]}
    {id: "fix" action: "apply automated fixes" depends: ["classify"]}
    {id: "verify" action: "run security tests" depends: ["fix"]}
  ]

  plan.run steps
    (step ctx) {
      worker = agent.spawn {command: "security-worker" args: [step.action]} ^
      result = worker ~>? {action: step.action context: ctx.completed} ^
      agent.kill worker ^
      result
    }
    (step result state) {
      step.id == "classify" & result.critical_count > 0 ? {
        true -> plan.replan [
          {id: "hotfix" action: "apply critical hotfixes immediately" depends: ["classify"]}
          {id: "alert" action: "notify security team" depends: ["classify"]}
          {id: "fix_rest" action: "apply remaining fixes" depends: ["hotfix"]}
          {id: "verify" action: "full security audit" depends: ["fix_rest" "alert"]}
        ]
        false -> plan.continue
      }
    }
  ^
}
```

Uses: `std/plan` for plan-as-data execution. When vulnerability scan reveals critical issues, the plan is dynamically revised to add hotfix and alert steps before continuing.

## Message Interceptor: Audit Trail

```
use std/agent
use std/time
use std/fs

+main = () {
  add_audit = (agent name) {
    agent.intercept agent (msg next) {
      start = time.now ()
      fs.append "audit.log" "[{start | time.format "%H:%M:%S"}] -> {name}: {msg | to_str}\n" ^
      result = next msg
      elapsed = time.since start
      fs.append "audit.log" "[{time.now () | time.format "%H:%M:%S"}] <- {name}: {elapsed}ms\n" ^
      result
    }
  }

  worker = agent.spawn {command: "worker" args: []} ^
  audited_worker = add_audit worker "worker"

  audited_worker ~>? {task: "process" data: [1 2 3]} ^
  audited_worker ~>? {task: "summarize"} ^

  agent.kill worker ^
}
```

Uses: `agent.intercept` for transparent audit logging. Every message sent to `audited_worker` is logged with timestamps, without modifying any call site.

## Structured Handoff: Research → Implementation

```
use std/agent

+main = () {
  researcher = agent.spawn {command: "researcher" args: []} ^
  implementer = agent.spawn {command: "implementer" args: []} ^

  research = researcher ~>? {task: "analyze auth token refresh bug"} ^

  handoff = Handoff {
    result: research
    tried: ["checked refresh interval" "checked token expiry logic" "checked retry behavior"]
    assumptions: ["single-threaded token refresh" "no concurrent requests during refresh"]
    uncertainties: ["unclear if rate limiting affects refresh timing"]
    recommendations: ["start at src/auth/token.rs:45" "check the mutex around refresh()"]
    files_read: ["src/auth/token.rs" "src/auth/session.rs" "src/auth/middleware.rs"]
  }

  fix = agent.handoff researcher implementer handoff ^
  emit "Fix applied: {fix.summary}"

  agent.kill researcher ^
  agent.kill implementer ^
}
```

Uses: `Handoff` Protocol for structured context transfer. The implementer receives not just the research results, but everything the researcher tried, assumed, and recommends.

## Introspection: Budget-Aware Processing

```
use std/introspect
use std/agent

+main = () {
  items = load_work_items ()

  results = items | map (item) {
    budget = introspect.budget ()
    budget.remaining < 500 ? {
      true -> {status: "skipped" item: item.name reason: "budget low"}
      false -> {
        introspect.mark "start_{item.name}"
        result = full_analysis item ^
        introspect.is_stuck () ? {
          true -> {
            introspect.strategy_shift "switching to lightweight for {item.name}"
            quick_analysis item ^
          }
          false -> result
        }
      }
    }
  }

  emit {type: "done" processed: results | filter (.status != "skipped") | len}
}
```

Uses: `std/introspect` for budget-aware processing and stuck detection. Agent checks remaining budget before expensive operations and switches strategy when stuck.

## Shared Knowledge: Parallel Review with Deduplication

```
use std/agent
use std/knowledge

+main = () {
  kb = knowledge.create ".review-knowledge.json" ^

  (sec_result perf_result) = par {
    sec_agent ~>? {task: "security review" kb_path: ".review-knowledge.json"} ^
    perf_agent ~>? {task: "performance review" kb_path: ".review-knowledge.json"} ^
  }

  all_findings = knowledge.query (e) contains? "finding" (e.meta.tags ?? []) kb
  shared_files = knowledge.query (e) contains? "file" (e.meta.tags ?? []) kb

  emit {
    type: "review_complete"
    total_findings: all_findings | len
    unique_files: shared_files | map (.key) | uniq | len
    deduped_reads: shared_files | len
  }
}
```

Uses: `std/knowledge` for cross-agent discovery sharing. Both agents contribute to and read from the same knowledge base, avoiding redundant file reads and surfacing all findings in one query.
