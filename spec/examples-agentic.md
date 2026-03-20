# Examples — Agentic Features

Worked examples for streaming, checkpoint/rollback, blackboard, capability attenuation, pub/sub, and negotiation. See [examples.md](examples.md) for core examples, [examples-extended.md](examples-extended.md) for general patterns, and [examples-agentic-2.md](examples-agentic-2.md) for dialogue, plan revision, interceptors, and handoff examples.

## Safe Refactor with Checkpoint

```
use std/agent
use std/fs

+main = () {
  files = fs.glob "src/**/*.lx" | collect

  result = checkpoint "refactor" {
    worker = agent.spawn {
      command: "refactor-agent"
      args: []
      capabilities: {
        fs: {read: ["./src/**"] write: ["./src/**"]}
        network: false
      }
    } ^

    worker ~>? {task: "rename" old: "fetch_data" new: "load_data" files} ^
    test_output = $^just test
    test_output | contains? "FAIL" ? {
      true  -> rollback "refactor"
      false -> Ok "refactor complete"
    }
  }

  result ? {
    Ok msg     -> $echo "{msg}"
    Err reason -> $echo "rolled back: {reason | to_str}"
  }
}
```

Uses: `checkpoint`/`rollback` for safe trial-and-error, capability attenuation on spawn, shell integration for test verification.

## Streaming Code Review

```
use std/agent

+main = () {
  reviewer = agent.spawn {command: "review-agent" args: []} ^

  reviewer ~>>? {task: "review" path: "src/" depth: 3}
    | each (chunk) {
      chunk.severity ? {
        "critical" -> {
          $echo "CRITICAL: {chunk.file}:{chunk.line} — {chunk.msg}"
          notifier ~> {alert: chunk}
        }
        "warning" -> $echo "WARN: {chunk.file} — {chunk.msg}"
        _         -> ()
      }
    }

  agent.kill reviewer ^
}
```

Uses: `~>>?` streaming for incremental review results, pattern matching on severity, fire-and-forget notification.

## Parallel Review with Shared Blackboard

```
use std/agent
use std/blackboard

+main = () {
  board = blackboard.create ()

  (sec perf style) = par {
    sec_agent ~>? {task: "security" board} ^
    perf_agent ~>? {task: "performance" board} ^
    style_agent ~>? {task: "style" board} ^
  }

  all_issues = blackboard.snapshot board
    | values
    | flatten
    | sort_by (.severity)

  all_issues | each (issue) $echo "[{issue.category}] {issue.file}: {issue.msg}"
}
```

Uses: `std/blackboard` for cross-agent awareness during parallel execution, `par` for structured concurrency.

## Agent Negotiation Pattern

```
use std/agent

Trait Offer = {task: Str  constraints: Rec  budget: Int}
Trait Accept = {commitment: Str  estimated_cost: Int}
Trait Reject = {reason: Str  counter_offer: Any}

negotiate = (agent offer) {
  response = agent ~>? Offer offer ^
  response ? {
    {commitment} -> Ok response
    {reason counter_offer} -> counter_offer ? {
      Some counter -> negotiate agent counter
      None         -> Err "rejected: {reason}"
    }
  }
}

+main = () {
  worker = agent.spawn {command: "worker" args: []} ^
  result = negotiate worker {task: "review" constraints: {max_files: 10} budget: 5000}
  result ? {
    Ok deal -> worker ~>? {action: "execute" deal} ^
    Err msg -> $echo "negotiation failed: {msg}"
  }
}
```

Uses: Trait-based negotiation pattern with recursive counter-offers, pattern matching on agent responses.

## Reactive Event-Driven Pipeline

```
use std/events
use std/agent

+main = () {
  bus = events.create ()

  analyzer = agent.spawn {command: "analyzer" args: []} ^
  notifier = agent.spawn {command: "notifier" args: []} ^

  events.subscribe bus "file_changed" (evt) {
    result = analyzer ~>? {path: evt.path action: "check"} ^
    result.issues | empty? ? () : {
      events.publish bus "issues_found" {path: evt.path issues: result.issues}
    }
  }

  events.subscribe bus "issues_found" (evt) {
    notifier ~> {alert: "Issues in {evt.path}" count: evt.issues | len}
  }

  $^inotifywait -m -r src/ --format "%w%f"
    | lines
    | each (path) events.publish bus "file_changed" {path}
}
```

Uses: `std/events` for decoupled reactive pipeline, agents as event handlers.

## Sandboxed Multi-Agent with Budget

```
use std/agent
use std/time

+main = () {
  tasks = [
    {name: "auth" path: "src/auth/"}
    {name: "api" path: "src/api/"}
    {name: "db" path: "src/db/"}
  ]

  results = tasks | pmap (task) {
    worker = agent.spawn {
      command: "reviewer"
      args: [task.name]
      capabilities: {
        tools: ["read_file" "grep" "glob"]
        fs: {read: [task.path ++ "**"] write: []}
        network: false
        budget: {tokens: 5000 wall_clock: time.min 2}
      }
    } ^
    result = worker ~>? {action: "review" path: task.path} ^
    agent.kill worker ^
    {name: task.name  ..result}
  }

  results | each (r) $echo "[{r.name}] score: {r.score} issues: {r.issues | len}"
}
```

Uses: capability attenuation with per-task filesystem scope, token budget, wall-clock limits. Each worker is sandboxed to only its assigned directory.
