# Work Item 6: Agent Messaging

Implement tell/ask messaging through the agent's `handle` method return value. `tell` is fire-and-forget, `ask` is request-response. Both target agents by name string. Both are language-level expressions (`Expr::Tell`, `Expr::Ask`) that the desugarer transforms into runtime calls.

## Prerequisites

- Work item 5 (agent system refactor) must be complete — named agent registry, AgentHandle with mailbox, spawn/stop infrastructure.

## Key Rules (from CLAUDE.md)

- No code comments or doc strings
- Never use `#[allow(...)]` macros
- 300 line file limit per file
- Use `just diagnose` (not raw cargo commands) to verify
- Prefer established crates over custom code

## Current State

- `crates/lx/src/ast/expr_types.rs` lines 125-135 — `ExprTell { target: ExprId, msg: ExprId }` and `ExprAsk { target: ExprId, msg: ExprId }` already exist as AST nodes.
- `crates/lx/src/lexer/token.rs` lines 54-55 — `TildeArrow` (`~>`) and `TildeArrowQ` (`~>?`) tokens exist for tell and ask syntax.
- `crates/lx/src/parser/expr_pratt.rs` lines 161-165 — `TildeArrow` parses as `Expr::Tell`, `TildeArrowQ` parses as `Expr::Ask`. Both are infix operators: `target ~> msg` (tell), `target ~>? msg` (ask).
- `crates/lx/src/folder/desugar.rs` lines 58-65 — `Expr::Tell` desugars to `agent.tell(target, msg)`, `Expr::Ask` desugars to `agent.ask(target, msg)` via `gen_field_call`.
- `crates/lx/src/builtins/agent.rs` (pre-refactor) — `bi_agent_tell` and `bi_agent_ask` use numeric IDs and blocking `mpsc` channels.
- `crates/lx/src/interpreter/mod.rs` line 146 — `Expr::Tell(_) | Expr::Ask(_) => unreachable!()` because they are desugared before evaluation.
- `crates/lx/src/runtime/agent_registry.rs` (from work item 5) — `AgentHandle` with `mailbox: mpsc::Sender<AgentMessage>`, `AgentMessage { payload: LxVal, reply: Option<oneshot::Sender<LxVal>> }`.

## Design Decision

tell/ask will **not** desugar to `agent.tell`/`agent.ask` builtins anymore. They will be evaluated directly in the interpreter as `Expr::Tell` and `Expr::Ask`, calling the agent registry directly. This eliminates the intermediate builtin layer and makes the messaging path explicit.

The desugarer must **stop** transforming `Expr::Tell` and `Expr::Ask`. They pass through to Core unchanged. The interpreter handles them directly.

## Files to Modify

- `crates/lx/src/folder/desugar.rs` — remove the tell/ask desugaring
- `crates/lx/src/interpreter/mod.rs` — replace the `unreachable!()` for Tell/Ask with actual eval logic
- `crates/lx/src/interpreter/eval.rs` — add `eval_tell` and `eval_ask` methods (if the main eval method gets too long, put them here)
- `crates/lx/src/builtins/register.rs` — remove `agent.ask` and `agent.tell` from the agent record (if not already done in work item 5)

## Step 1: Remove tell/ask desugaring

File: `crates/lx/src/folder/desugar.rs`

In the `leave_expr` method of `Desugarer` (line 55), remove the two arms that desugar Tell and Ask:

**Remove these lines (58-65):**
```rust
Expr::Tell(t) => {
    let call = super::gen_ast::gen_field_call("agent", "tell", &[t.target, t.msg], span, arena);
    arena.expr(call).clone()
},
Expr::Ask(a) => {
    let call = super::gen_ast::gen_field_call("agent", "ask", &[a.target, a.msg], span, arena);
    arena.expr(call).clone()
},
```

**Replace with pass-through:**
```rust
Expr::Tell(_) => expr,
Expr::Ask(_) => expr,
```

This means `Expr::Tell` and `Expr::Ask` survive into the Core AST and reach the interpreter.

Also check `crates/lx/src/folder/validate_core.rs` — if it has a check that rejects Tell/Ask in Core, remove that check. Tell and Ask are now valid Core nodes.

## Step 2: Implement eval_tell

File: `crates/lx/src/interpreter/eval.rs`

Add a method to `impl Interpreter`:

```rust
pub(super) async fn eval_tell(&mut self, target: ExprId, msg: ExprId, span: SourceSpan) -> EvalResult<LxVal> {
    let target_val = self.eval(target).await?;
    let target_name = match &target_val {
        LxVal::Str(s) => s.to_string(),
        other => {
            return Err(LxError::type_err(
                format!("tell target must be Str (agent name), got {}", other.type_name()),
                span,
                None,
            ).into());
        }
    };
    let msg_val = self.eval(msg).await?;

    let mailbox = crate::runtime::agent_registry::get_agent_mailbox(&target_name)
        .ok_or_else(|| LxError::runtime(
            format!("agent '{}' not running", target_name), span
        ))?;

    let message = crate::runtime::AgentMessage {
        payload: msg_val,
        reply: None,
    };

    mailbox.send(message).await.map_err(|_| {
        LxError::runtime(format!("agent '{}' mailbox closed", target_name), span)
    })?;

    Ok(LxVal::Unit)
}
```

The `send` call is `await`-ed. If the mailbox buffer is full, the sending task suspends until space is available. This is fine — tell is "fire and forget" from the lx program's perspective, but the runtime may briefly suspend if the target agent is backed up.

## Step 3: Implement eval_ask

File: `crates/lx/src/interpreter/eval.rs`

Add a method to `impl Interpreter`:

```rust
pub(super) async fn eval_ask(&mut self, target: ExprId, msg: ExprId, span: SourceSpan) -> EvalResult<LxVal> {
    let target_val = self.eval(target).await?;
    let target_name = match &target_val {
        LxVal::Str(s) => s.to_string(),
        other => {
            return Err(LxError::type_err(
                format!("ask target must be Str (agent name), got {}", other.type_name()),
                span,
                None,
            ).into());
        }
    };
    let msg_val = self.eval(msg).await?;

    let mailbox = crate::runtime::agent_registry::get_agent_mailbox(&target_name)
        .ok_or_else(|| LxError::runtime(
            format!("agent '{}' not running", target_name), span
        ))?;

    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel::<LxVal>();

    let message = crate::runtime::AgentMessage {
        payload: msg_val,
        reply: Some(reply_tx),
    };

    mailbox.send(message).await.map_err(|_| {
        LxError::runtime(format!("agent '{}' mailbox closed", target_name), span)
    })?;

    let result = reply_rx.await.map_err(|_| {
        LxError::runtime(format!("agent '{}' did not reply (handle may have panicked)", target_name), span)
    })?;

    Ok(result)
}
```

Key behavior: `reply_rx.await` **suspends only the calling task**. Other agents and the main program continue executing on other tokio tasks. This is the async suspension model — no blocking threads.

## Step 4: Wire eval cases in interpreter

File: `crates/lx/src/interpreter/mod.rs`

Replace line 146:
```rust
Expr::Pipe(_) | Expr::Tell(_) | Expr::Ask(_) => unreachable!(),
```

With:
```rust
Expr::Pipe(_) => unreachable!(),
Expr::Tell(t) => self.eval_tell(t.target, t.msg, span).await,
Expr::Ask(a) => self.eval_ask(a.target, a.msg, span).await,
```

`Pipe` remains `unreachable!()` because it is still desugared to `Apply`.

## Step 5: Remove agent.tell and agent.ask from builtins

File: `crates/lx/src/builtins/register.rs`

If work item 5 already removed `agent.tell` and `agent.ask` from the agent record, this step is done. If not, remove these two lines from the `agent_fields` IndexMap construction:

```rust
agent_fields.insert(crate::sym::intern("ask"), mk("agent.ask", 2, super::agent::bi_agent_ask));
agent_fields.insert(crate::sym::intern("tell"), mk("agent.tell", 2, super::agent::bi_agent_tell));
```

The `agent` builtin record should no longer expose `ask` or `tell`. These are now language keywords, not module methods.

## Step 6: Event stream logging

Depends on work item 1 (event stream). At the appropriate points in `eval_tell` and `eval_ask`, write stream entries.

**In eval_tell**, after the `mailbox.send` succeeds:

```rust
// Write agent/tell event
// ctx.event_stream.xadd(StreamEntry {
//     kind: "agent/tell",
//     agent: self.agent_name.clone().unwrap_or_else(|| "main".to_string()),
//     fields: { "from": agent_name_or_main, "to": target_name, "msg": msg_val }
// })
```

The exact API depends on the event stream implementation from work item 1. The entry fields are:
- `kind`: `"agent/tell"`
- `from`: the sending agent's name (or `"main"` for the top-level program)
- `to`: `target_name`
- `msg`: serialized `msg_val`

**In eval_ask**, write two entries:

1. Before `reply_rx.await`: `agent/ask` entry with fields `ask_id` (monotonic counter), `from`, `to`, `msg`.
2. After `reply_rx.await` returns: `agent/reply` entry with fields `ask_id`, `from` (the target that replied), `to` (the original asker), `msg` (the reply value).

The `ask_id` is a per-interpreter monotonic counter. Add a field to `Interpreter`:

```rust
pub(crate) next_ask_id: std::sync::atomic::AtomicU64,
```

Initialize to 1 in `Interpreter::new`. Increment per ask call.

## Step 7: Update std/agent.lx

File: `crates/lx/std/agent.lx`

If work item 5 already reduced the Agent trait to the minimal form, no changes needed. If the trait still has `ask` and `tell` methods that call `agent.ask`/`agent.tell`, remove them. The Agent trait should not expose `ask`/`tell` as methods — they are language-level expressions that any code (agent or main) can use:

```lx
tell "AgentName" msg
result = ask "AgentName" msg
```

These work anywhere — inside agent code or in the main program.

## Error Cases

| Scenario | Error message |
|---|---|
| Target not a string | `"tell target must be Str (agent name), got {type}"` / `"ask target must be Str (agent name), got {type}"` |
| Agent not running | `"agent '{name}' not running"` |
| Mailbox closed (agent stopped) | `"agent '{name}' mailbox closed"` |
| ask reply channel dropped | `"agent '{name}' did not reply (handle may have panicked)"` |
| tell/ask targeting main program | Main is not in the registry, so `get_agent_mailbox` returns `None` → `"agent 'main' not running"` |

## Verification

After all changes:
1. `just diagnose` must pass with no warnings.
2. `just test` must pass all existing tests.
3. Write a test `.lx` file:

```lx
Agent Echo = {
  handle = (msg) { {echoed: msg.text, from: "Echo"} }
}

Agent Counter = {
  count := 0
  handle = (msg) {
    count <- count + 1
    {count: count, msg: msg}
  }
}

spawn Echo
spawn Counter

-- tell is fire-and-forget
tell "Counter" {inc: true}
tell "Counter" {inc: true}

-- ask blocks until reply
response = ask "Echo" {text: "hello"}
assert response.echoed == "hello"
assert response.from == "Echo"

-- counter received the tells
count_response = ask "Counter" {inc: true}
assert count_response.count == 3
```

This test verifies: tell delivers without blocking, ask suspends and returns handle's result, multiple agents coexist, agent state persists across messages.
