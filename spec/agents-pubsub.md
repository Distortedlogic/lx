# Agent-Level Publish/Subscribe

Extensions to `std/agent` for cross-agent publish/subscribe messaging. Agents subscribe to named topics and receive broadcasts through their message handlers — no manual forwarding by the orchestrator.

Distinct from `std/events` (in-process event bus with `(msg) -> ()` callbacks). Agent pub/sub routes messages across agent boundaries using the standard `~>` protocol. An agent subscribing to a topic receives messages the same way it receives direct `~>` messages.

## Problem

`std/events` handles in-process pub/sub: publish a topic, handler functions fire synchronously. But agents are subprocesses. To broadcast a message to multiple agents today:

```
agents | each (a) {
  a ~> {type: "build_status" status: "compiling" file: "main.rs"}
}
```

The orchestrator manually tracks who wants what and loops through sends. There's no way for an agent to declare "I care about build status updates" and have the system route automatically. This coupling means:

- Adding a new subscriber requires changing the orchestrator
- Agents can't subscribe/unsubscribe dynamically
- No filtered subscriptions (agent wants only critical build events)
- Broadcast patterns are reimplemented everywhere

## API

### Creating Topics

```
use std/agent

build_events = agent.topic "build_status"
test_events = agent.topic "test_results"
```

`agent.topic` creates or retrieves a named topic. Topics are global within the runtime — same name returns the same topic.

### Subscribing

```
agent.subscribe analyzer build_events

agent.subscribe logger build_events
agent.subscribe logger test_events
```

Subscribed agents receive all messages published to the topic through their normal handler. The message arrives with a `_topic` field injected:

```
handler = (msg) {
  msg._topic ? {
    "build_status" -> handle_build msg
    "test_results" -> handle_test msg
    _ -> handle_direct msg
  }
}
```

### Filtered Subscriptions

```
agent.subscribe security_agent build_events (msg) {
  msg.severity == :critical
}
```

The filter function runs before delivery. Only messages where the filter returns `true` are forwarded.

### Publishing

```
agent.publish build_events {status: "compiling" file: "main.rs"}

agent.publish build_events {status: "failed" file: "main.rs" error: err severity: :critical}
```

`agent.publish` broadcasts to all subscribers (after applying their filters). Fire-and-forget — does not wait for handlers to complete.

### Publish and Collect

```
responses = agent.publish_collect test_events {suite: "integration" run_id: 42} ^
// => [{agent: "analyzer" result: ...} {agent: "reporter" result: ...}]
```

`agent.publish_collect` broadcasts and waits for all subscriber responses. Returns a list of `{agent result}` records. Like `~>?` but fan-out.

### Unsubscribing

```
agent.unsubscribe analyzer build_events
```

### Listing Subscribers

```
subs = agent.subscribers build_events
// => [{agent: "analyzer" filtered: false} {agent: "logger" filtered: false} ...]
```

### Listing Topics

```
topics = agent.topics ()
// => ["build_status" "test_results"]
```

## Patterns

### Monitoring Pipeline

```
build_events = agent.topic "build"
agent.subscribe monitor build_events
agent.subscribe logger build_events
agent.subscribe metrics build_events (msg) msg.status == :failed

// builder publishes as it works
agent.publish build_events {status: :started file: f}
// ... build ...
agent.publish build_events {status: :completed file: f duration: elapsed}
```

Monitor, logger, and metrics all receive events without the builder knowing about them.

### Reactive Multi-Agent Review

```
review_topic = agent.topic "review_findings"

agent.subscribe aggregator review_topic
agent.subscribe dashboard review_topic (msg) msg.severity == :critical

par {
  security_agent ~>? {task: "review" code: diff} ^
    | (findings) agent.publish review_topic {source: "security" findings severity: :high}

  perf_agent ~>? {task: "review" code: diff} ^
    | (findings) agent.publish review_topic {source: "perf" findings severity: :normal}
}
```

### Dynamic Subscription

```
new_agent = agent.spawn "specialist" handler ^
agent.subscribe new_agent relevant_topic

// later, when agent is no longer needed
agent.unsubscribe new_agent relevant_topic
agent.kill new_agent
```

## Relationship to std/events

| | std/events | agent.topic/subscribe |
|---|---|---|
| Scope | In-process | Cross-agent (subprocess boundaries) |
| Handler | `(msg) -> ()` function | Agent message handler via `~>` |
| Delivery | Synchronous, in subscription order | Async, parallel delivery |
| Filtering | Handler-side (check in callback) | Subscription-side (filter predicate) |
| Collect | No (fire-and-forget only) | `publish_collect` waits for responses |

They complement each other. `std/events` is for in-process event wiring. Agent pub/sub is for multi-agent broadcast.

## Implementation

Extension to `std/agent`. Topics are a `DashMap<String, Topic>` in the runtime. Each `Topic` holds a `Vec<Subscription>` where each subscription is an `(AgentId, Option<FilterFn>)` pair.

`agent.publish` iterates subscribers and sends via the existing `~>` mechanism. `agent.publish_collect` uses `~>?` and collects results.

The `_topic` field is injected into the message before delivery and stripped after handler return.

### Dependencies

- `dashmap` (concurrent topic registry — already a dependency)
- Existing agent send/ask infrastructure

## Cross-References

- In-process events: stdlib (`std/events`) — complementary, different scope
- Blackboard: stdlib (`std/blackboard`) — pull-based shared state vs push-based events
- Broadcast: [agents-broadcast.md](agents-broadcast.md) — `workflow.peers`/`workflow.share` passive visibility
- Intercept: [agents-intercept.md](agents-intercept.md) — middleware can wrap topic publications
- Dispatch: [agents-dispatch.md](agents-dispatch.md) — pattern-based routing (point-to-point vs broadcast)
