# Work Item 5: Agent System Refactor

Rewrite the agent system to support the handle/run model with named agents. Agents spawn as async tasks on the shared tokio runtime, identified by class name strings. The global agent registry replaces the current numeric-ID-based `AGENTS` DashMap.

## Prerequisites

- Work item 1 (event stream) must be complete — agent lifecycle events (`agent/spawn`, `agent/kill`) write to the event stream.

## Key Rules (from CLAUDE.md)

- No code comments or doc strings
- Never use `#[allow(...)]` macros
- 300 line file limit per file
- Use `just diagnose` (not raw cargo commands) to verify
- Prefer established crates over custom code

## Current State

- `crates/lx/src/builtins/agent.rs` — current agent system using `std::sync::mpsc` channels, numeric IDs (`NEXT_AGENT_ID: AtomicU64`), and a static `AGENTS: LazyLock<DashMap<u64, AgentEntry>>`. Each agent spawns via `tokio::task::spawn_blocking` with its **own** `tokio::runtime::Runtime`. Communication is blocking `mpsc::channel`.
- `crates/lx/src/builtins/register.rs` lines 165-171 — registers `agent.spawn`, `agent.kill`, `agent.ask`, `agent.tell` as sync builtins on the `agent` record.
- `crates/lx/std/agent.lx` — current Agent trait with `handle`, `run`, OODA cycle, `think`, `delegate` methods. Uses `yield` for message passing in `run`.
- `crates/lx/src/folder/desugar.rs` lines 58-65 — `Expr::Tell` desugars to `agent.tell(target, msg)`, `Expr::Ask` desugars to `agent.ask(target, msg)`.
- `crates/lx/src/ast/types.rs` lines 101-114 — `KeywordKind::Agent` exists, parsed by `crates/lx/src/parser/stmt_keyword.rs`.
- `crates/lx/src/folder/desugar.rs` lines 217-218 — `KeywordKind::Agent` desugars to `use std/agent {Agent}` + `ClassDecl` with trait `Agent`.
- `crates/lx/src/interpreter/eval.rs` lines 97-116 — `eval_par` creates concurrent tasks sharing the same tokio runtime via `futures::future::join_all`.
- `crates/lx/src/lexer/token.rs` — no `Spawn` or `Stop` token kinds exist.
- `crates/lx/src/ast/mod.rs` — no `Spawn` or `Stop` expr variants exist.
- `crates/lx/src/runtime/mod.rs` lines 20-39 — `RuntimeCtx` struct with `SmartDefault`, holds `tokio_runtime: Arc<tokio::runtime::Runtime>`.

## Files to Create

- `crates/lx/src/runtime/agent_registry.rs` — global agent registry and AgentHandle type
- `crates/lx/src/builtins/agent/mod.rs` — replaces `agent.rs`, re-exports spawn and stop
- `crates/lx/src/builtins/agent/spawn.rs` — spawn builtin implementation
- `crates/lx/src/builtins/agent/stop.rs` — stop builtin implementation

## Files to Modify

- `crates/lx/src/builtins/agent.rs` — delete (replaced by `agent/mod.rs` directory)
- `crates/lx/src/builtins/mod.rs` — no changes needed; `pub(crate) mod agent;` at line 1 resolves to `agent/mod.rs` automatically
- `crates/lx/src/builtins/register.rs` — update agent record registration to use new functions
- `crates/lx/src/runtime/mod.rs` — add `mod agent_registry; pub use agent_registry::*;` and add `agent_registry` field to `RuntimeCtx`
- `crates/lx/src/lexer/token.rs` — add `Spawn` and `Stop` token kinds
- `crates/lx/src/lexer/helpers.rs` — map `"spawn"` and `"stop"` strings to the new token kinds in `ident_or_keyword` function (line 16)
- `crates/lx/src/ast/mod.rs` — add `Spawn(ExprId)` and `Stop` variants to the `Expr` enum
- `crates/lx/src/ast/expr_types.rs` — no new types needed; `Spawn` takes a single `ExprId` (the class name ident), `Stop` takes no args
- `crates/lx/src/parser/expr.rs` — parse `spawn Foo` as `Expr::Spawn(ExprId)` and `stop` as `Expr::Stop` (prefix expressions are in `expr_parser` at line 98)
- `crates/lx/src/interpreter/mod.rs` — add eval cases for `Expr::Spawn` and `Expr::Stop`
- `crates/lx/src/folder/desugar.rs` — no changes needed; `Expr::Spawn` and `Expr::Stop` pass through the `other => other` arm at line 77 and the `_ => VisitAction::Descend` arm at `validate_core.rs` line 39
- `crates/lx/std/agent.lx` — rewrite the Agent trait to match the new model

## Step 1: Add `spawn` and `stop` as keywords in the lexer

File: `crates/lx/src/lexer/token.rs`

Add two new variants to `TokenKind` in the keyword section (after `Yield` at line 83, before `With` at line 84):

```
Spawn,
Stop,
```

File: `crates/lx/src/lexer/helpers.rs`

In the `ident_or_keyword` function (line 16), add two arms to the match statement after `"as" => TokenKind::As,` at line 30, before the `_ =>` wildcard at line 31:

```
"spawn" => TokenKind::Spawn,
"stop" => TokenKind::Stop,
```

## Step 2: Add AST nodes for spawn and stop

File: `crates/lx/src/ast/mod.rs`

Add two variants to the `Expr` enum, in the concurrency section (after `Timeout` at line 106, before `Emit` at line 108):

```rust
Spawn(ExprId),
Stop,
```

No `#[walk(skip)]` needed — `Spawn(ExprId)` should walk its inner `ExprId`, and `Stop` has no children.

## Step 3: Parse spawn and stop expressions

File: `crates/lx/src/parser/expr.rs`

Prefix keyword expressions are parsed in `expr_parser` (line 98). The `emit_expr` parser is at lines 188-191, `break_expr` at lines 164-169, and `assert_expr` at lines 171-186. The atom `choice((...))` that combines them is at lines 210-229.

**Add `spawn_expr`** after `timeout_expr` (line 206), before the `atom` choice at line 210:

```rust
let spawn_expr = {
    let al = arena.clone();
    just(TokenKind::Spawn)
        .ignore_then(type_name().map_with(move |n, e| al.borrow_mut().alloc_expr(Expr::TypeConstructor(n), ss(e.span()))))
        .map_with(move |class_eid, e| arena.clone().borrow_mut().alloc_expr(Expr::Spawn(class_eid), ss(e.span())))
};
```

Note: this requires two arena clones — one for the inner `TypeConstructor` alloc and one for the outer `Spawn` alloc. Assign separate arena clones (e.g. `a12` and `a13`) following the existing pattern at lines 103-112.

**Add `stop_expr`** similarly:

```rust
let stop_expr = {
    let al = arena.clone();
    just(TokenKind::Stop).map_with(move |_, e| al.borrow_mut().alloc_expr(Expr::Stop, ss(e.span())))
};
```

**Update the `atom` choice** at lines 210-229. Add `spawn_expr` and `stop_expr` to the `choice((...))`  tuple, before `type_ctor` and `ident_expr` (since `spawn` and `stop` are keywords that should be matched before general identifiers). Add them after `assert_expr` at line 224:

```rust
let atom = choice((
    literal,
    string_lit,
    paren,
    list,
    block_or_record,
    map,
    loop_expr,
    par_expr,
    sel_expr,
    emit_expr,
    yield_expr,
    with_expr,
    break_expr,
    assert_expr,
    spawn_expr,
    stop_expr,
    type_ctor,
    ident_expr,
))
.or(timeout_expr)
.boxed();
```

**Update imports** at line 6: add `ExprSpawn` if needed — but since `Spawn(ExprId)` is a simple variant, no separate type is needed. No import changes required beyond what `Expr` already provides.

**Update `ident_or_keyword`** at lines 17-35: add `spawn` and `stop` so they can appear in field-access position:

```rust
TokenKind::Spawn => intern("spawn"),
TokenKind::Stop => intern("stop"),
```

## Step 4: Define AgentHandle and global registry

File: `crates/lx/src/runtime/agent_registry.rs`

```rust
use std::sync::{Arc, LazyLock};

use dashmap::DashMap;
use tokio::sync::{mpsc, oneshot};

use crate::value::LxVal;

pub struct AgentMessage {
    pub payload: LxVal,
    pub reply: Option<oneshot::Sender<LxVal>>,
}

pub struct AgentHandle {
    pub name: String,
    pub mailbox: mpsc::Sender<AgentMessage>,
    pub task: tokio::task::JoinHandle<()>,
    pub pause_flag: Arc<std::sync::atomic::AtomicBool>,
}

static AGENT_REGISTRY: LazyLock<DashMap<String, AgentHandle>> = LazyLock::new(DashMap::new);

pub fn register_agent(name: String, handle: AgentHandle) -> Result<(), String> {
    if AGENT_REGISTRY.contains_key(&name) {
        return Err(format!("agent '{}' already running", name));
    }
    AGENT_REGISTRY.insert(name, handle);
    Ok(())
}

pub fn get_agent_mailbox(name: &str) -> Option<mpsc::Sender<AgentMessage>> {
    AGENT_REGISTRY.get(name).map(|e| e.mailbox.clone())
}

pub fn remove_agent(name: &str) -> Option<(String, AgentHandle)> {
    AGENT_REGISTRY.remove(name)
}

pub fn agent_exists(name: &str) -> bool {
    AGENT_REGISTRY.contains_key(name)
}

pub fn agent_names() -> Vec<String> {
    AGENT_REGISTRY.iter().map(|e| e.key().clone()).collect()
}

pub fn get_agent_entry(name: &str) -> Option<dashmap::mapref::one::Ref<'_, String, AgentHandle>> {
    AGENT_REGISTRY.get(name)
}
```

## Step 5: Wire agent_registry into RuntimeCtx

File: `crates/lx/src/runtime/mod.rs`

Add at the top, after existing mod declarations:

```rust
pub mod agent_registry;
pub use agent_registry::*;
```

No new fields needed on `RuntimeCtx` itself — the registry is a process-global static (same pattern as the current `AGENTS` DashMap). The `pause_flag` is per-agent on `AgentHandle`.

## Step 6: Rewrite crates/lx/src/builtins/agent.rs

Convert `crates/lx/src/builtins/agent.rs` into `crates/lx/src/builtins/agent/mod.rs` with subfiles. This requires:

1. Delete `crates/lx/src/builtins/agent.rs`
2. Create `crates/lx/src/builtins/agent/mod.rs`
3. Create `crates/lx/src/builtins/agent/spawn.rs`
4. Create `crates/lx/src/builtins/agent/stop.rs`

No changes needed in `crates/lx/src/builtins/mod.rs` — `pub(crate) mod agent;` still works because `agent/mod.rs` replaces `agent.rs`.

### File: `crates/lx/src/builtins/agent/mod.rs`

```rust
mod spawn;
mod stop;

pub use spawn::bi_agent_spawn;
pub use stop::bi_agent_stop;
```

### File: `crates/lx/src/builtins/agent/spawn.rs`

Implement `bi_agent_spawn` as an **async** builtin (signature: `fn(Vec<LxVal>, SourceSpan, Arc<RuntimeCtx>) -> Pin<Box<dyn Future<Output = Result<LxVal, LxError>>>>`).

```rust
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::future::Future;
use std::pin::Pin;

use crate::error::LxError;
use crate::interpreter::Interpreter;
use crate::runtime::RuntimeCtx;
use crate::runtime::agent_registry::{AgentHandle, AgentMessage, register_agent};
use crate::sym::intern;
use crate::value::{LxVal, LxClass};
use miette::SourceSpan;

pub fn bi_agent_spawn(
    args: Vec<LxVal>,
    span: SourceSpan,
    ctx: Arc<RuntimeCtx>,
) -> Pin<Box<dyn Future<Output = Result<LxVal, LxError>>>> {
    Box::pin(async move {
        let class: Box<LxClass> = match args.into_iter().next() {
            Some(LxVal::Class(c)) => c,
            _ => return Err(LxError::type_err("spawn: expected a Class value", span, None)),
        };

        let name = class.name.as_str().to_string();
        let handle_method = class.methods.get(&intern("handle")).cloned();
        let run_method = class.methods.get(&intern("run")).cloned();

        if handle_method.is_none() && run_method.is_none() {
            return Err(LxError::runtime("spawn: agent class has no handle or run method", span));
        }

        let (tx, rx) = tokio::sync::mpsc::channel::<AgentMessage>(256);
        let pause_flag = Arc::new(AtomicBool::new(false));
        let rx = Arc::new(tokio::sync::Mutex::new(rx));

        let task_ctx = Arc::clone(&ctx);
        let task_name = name.clone();
        let task_pause = Arc::clone(&pause_flag);
        let task_rx = Arc::clone(&rx);

        let join_handle = tokio::spawn(async move {
            let mut interp = Interpreter::new("", None, task_ctx);
            interp.agent_name = Some(task_name.clone());

            let has_handle = handle_method.is_some();
            let has_run = run_method.is_some();

            if has_handle && !has_run {
                let handle_fn = handle_method.unwrap();
                let mut rx_guard = task_rx.lock().await;
                while let Some(msg) = rx_guard.recv().await {
                    let result = crate::builtins::call_value(&handle_fn, msg.payload.clone(), miette::SourceSpan::new(0.into(), 0), &interp.ctx)
                        .await
                        .unwrap_or_else(|e| LxVal::err_str(e.to_string()));
                    if let Some(reply) = msg.reply {
                        let _ = reply.send(result);
                    }
                }
            } else if has_run && !has_handle {
                drop(task_rx);
                if let Some(run_fn) = run_method {
                    let _ = crate::builtins::call_value(&run_fn, LxVal::Unit, miette::SourceSpan::new(0.into(), 0), &interp.ctx).await;
                }
            } else {
                interp.agent_mailbox_rx = Some(task_rx);
                interp.agent_handle_fn = handle_method;
                if let Some(run_fn) = run_method {
                    let _ = crate::builtins::call_value(&run_fn, LxVal::Unit, miette::SourceSpan::new(0.into(), 0), &interp.ctx).await;
                }
            }
        });

        let handle = AgentHandle {
            name: name.clone(),
            mailbox: tx,
            task: join_handle,
            pause_flag,
        };

        register_agent(name.clone(), handle)
            .map_err(|msg| LxError::runtime(msg, span))?;

        Ok(LxVal::ok(LxVal::str(name)))
    })
}
```

Key details:
- **LxClass extraction from LxVal**: Pattern match `LxVal::Class(c)` where `c` is `Box<LxClass>`. The `LxClass` struct is defined at `crates/lx/src/value/mod.rs` line 42 with fields: `name: Sym`, `traits: Arc<Vec<Sym>>`, `defaults: Arc<IndexMap<Sym, LxVal>>`, `methods: Arc<IndexMap<Sym, LxVal>>`.
- **Handle+run interleaving**: When both `handle` and `run` exist, the interpreter's `agent_mailbox_rx` and `agent_handle_fn` fields are set. The eval loop (Step 8) checks `rx.try_recv()` at expression boundaries, calling `handle(msg.payload)` for each pending message before continuing `run`.
- **Interpreter field additions** (see Step 8): `agent_name: Option<String>`, `agent_mailbox_rx: Option<Arc<tokio::sync::Mutex<tokio::sync::mpsc::Receiver<AgentMessage>>>>`, `agent_handle_fn: Option<LxVal>`.

### File: `crates/lx/src/builtins/agent/stop.rs`

Implement `bi_agent_stop` as a sync builtin.

This is called from within an agent's own code when evaluating `Expr::Stop`. It does NOT take an agent name argument — the interpreter knows which agent is currently executing.

For this to work, the interpreter needs to know its own agent name. Add a field to `Interpreter`:

```rust
pub(crate) agent_name: Option<String>,
```

Default: `None` (main program is not an agent). Set to `Some(name)` when creating the agent's interpreter in the spawn task.

When `Expr::Stop` is evaluated:
1. Read `self.agent_name`. If `None`, return error: `"stop: not inside an agent"`.
2. Call `crate::runtime::agent_registry::remove_agent(&name)`.
3. Write `agent/kill` event to the event stream.
4. Return a special `EvalSignal` variant. Add `EvalSignal::AgentStop` to `crates/lx/src/error.rs`:

```rust
pub enum EvalSignal {
    Error(LxError),
    Break(LxVal),
    AgentStop,
}
```

The agent's run loop in the spawned task catches `AgentStop` and exits cleanly.

## Step 7: Add interpreter eval cases

File: `crates/lx/src/interpreter/mod.rs`

In the `eval` method's match statement (starts at line 124), add two new arms after `Expr::Timeout` at line 194, before `Expr::Emit` at line 195:

```rust
Expr::Spawn(class_expr) => {
    let class_val = self.eval(class_expr).await?;
    let result = crate::builtins::agent::bi_agent_spawn(
        vec![class_val], span, Arc::clone(&self.ctx)
    ).await;
    result.map_err(EvalSignal::Error)
},
Expr::Stop => {
    let name = self.agent_name.as_ref()
        .ok_or_else(|| LxError::runtime("stop: not inside an agent", span))?;
    crate::runtime::agent_registry::remove_agent(name);
    Err(EvalSignal::AgentStop)
},
```

**Update imports** at lines 26-29: add the import for `EvalSignal::AgentStop` — it's already available via `use crate::error::{EvalResult, EvalSignal, LxError};` at line 31.

## Step 8: Add message-check to eval loop for handle+run interleaving

This is the mechanism for serializing `handle` calls between expressions in `run`. The pause flag on `AgentHandle` is reserved for external pause/resume control. Handle interleaving uses direct `try_recv` on the mailbox.

### Add fields to `Interpreter`

File: `crates/lx/src/interpreter/mod.rs`

Add three fields to the `Interpreter` struct (line 41-49), after the `arena` field at line 48:

```rust
pub(crate) agent_name: Option<String>,
pub(crate) agent_mailbox_rx: Option<Arc<tokio::sync::Mutex<tokio::sync::mpsc::Receiver<crate::runtime::agent_registry::AgentMessage>>>>,
pub(crate) agent_handle_fn: Option<LxVal>,
```

Update `Interpreter::new` (lines 52-65) to initialize the new fields:

```rust
agent_name: None,
agent_mailbox_rx: None,
agent_handle_fn: None,
```

Update `Interpreter::with_env` (lines 67-77) similarly.

### Add message drain at eval entry

File: `crates/lx/src/interpreter/mod.rs`

At the **top** of the `eval` method (line 121), after `let span = self.arena.expr_span(eid);` at line 122 and before `let expr = self.arena.expr(eid).clone();` at line 123, insert:

```rust
if let (Some(rx_arc), Some(handle_fn)) = (&self.agent_mailbox_rx, &self.agent_handle_fn) {
    let mut rx = rx_arc.lock().await;
    while let Ok(msg) = rx.try_recv() {
        let result = crate::builtins::call_value(handle_fn, msg.payload.clone(), span, &self.ctx)
            .await
            .unwrap_or_else(|e| LxVal::err_str(e.to_string()));
        if let Some(reply) = msg.reply {
            let _ = reply.send(result);
        }
    }
    drop(rx);
}
```

This checks for and processes pending messages at expression boundaries. The overhead in the non-agent case is a single `Option` check (the `if let` pattern fails immediately when `agent_mailbox_rx` is `None`).

## Step 9: Update the agent record registration

File: `crates/lx/src/builtins/register.rs`

Replace lines 165-171 (the current agent record setup):

```rust
let mut agent_fields = IndexMap::new();
agent_fields.insert(crate::sym::intern("spawn"), super::mk_async("agent.spawn", 1, |args, span, ctx| {
    Box::pin(crate::builtins::agent::bi_agent_spawn(args, span, ctx))
}));
agent_fields.insert(crate::sym::intern("kill"), mk("agent.kill", 1, |args, span, _ctx| {
    let name = args[0].require_str("agent.kill", span)?;
    match crate::runtime::agent_registry::remove_agent(name) {
        Some(_) => Ok(LxVal::ok_unit()),
        None => Ok(LxVal::err_str(format!("agent '{}' not running", name))),
    }
}));
agent_fields.insert(crate::sym::intern("exists"), mk("agent.exists", 1, |args, span, _ctx| {
    let name = args[0].require_str("agent.exists", span)?;
    Ok(LxVal::Bool(crate::runtime::agent_registry::agent_exists(name)))
}));
agent_fields.insert(crate::sym::intern("list"), mk("agent.list", 0, |_args, _span, _ctx| {
    let names = crate::runtime::agent_registry::agent_names();
    Ok(LxVal::list(names.into_iter().map(LxVal::str).collect()))
}));
agent_fields.insert(crate::sym::intern("implements"), mk("agent.implements", 2, bi_agent_implements));
env.bind_str("agent", LxVal::record(agent_fields));
```

The `bi_agent_implements` function already exists at `crates/lx/src/builtins/register.rs` lines 194-213. It checks `LxVal::Object`, `LxVal::Class`, and `LxVal::Record` for trait membership. No changes needed to this function.

Remove the old `agent.ask` and `agent.tell` from the agent record — tell/ask are now handled as language-level expressions (work item 6), not as `agent.tell`/`agent.ask` builtins.

## Step 10: Rewrite std/agent.lx

File: `crates/lx/std/agent.lx`

```lx
+Trait Agent = {
  handle = (msg) { msg }
  run = () { }
  subscribes = () { [] }
}
```

The trait is now minimal. `handle` receives a message, returns a response. `run` is the autonomous loop. `subscribes` returns a list of channel names for auto-registration (work item 7). The OODA cycle, think, delegate, and other methods from the current trait are removed — they were LLM-specific and belong in user-defined subtraits, not the base Agent trait.

## Step 11: Update desugar.rs for Agent keyword

File: `crates/lx/src/folder/desugar.rs`

The existing desugaring at line 218 (`KeywordKind::Agent => (vec!["std", "agent"], "Agent")`) produces a `use std/agent {Agent}` statement and a `ClassDecl`. This continues to work — the class still implements the `Agent` trait. No change needed here.

`Expr::Spawn` and `Expr::Stop` pass through the desugarer's `leave_expr` method at `crates/lx/src/folder/desugar.rs` line 55 via the `other => other` catch-all at line 77. They also pass the core validator at `crates/lx/src/folder/validate_core.rs` line 28 via the `_ => VisitAction::Descend` wildcard at line 39. No changes needed in either file.

## Step 12: Update EvalSignal

File: `crates/lx/src/error.rs`

The `EvalSignal` enum is at lines 12-15. Add a third variant after `Break(LxVal)` at line 14:

```rust
pub enum EvalSignal {
    Error(LxError),
    Break(LxVal),
    AgentStop,
}
```

File: `crates/lx/src/interpreter/mod.rs`

The `exec` method's error mapping is at lines 112-114. Add `AgentStop` arm:

```rust
result = self.eval_stmt(*sid).await.map_err(|e| match e {
    EvalSignal::Error(e) => e,
    EvalSignal::Break(_) => LxError::runtime("break outside loop", self.arena.stmt_span(*sid)),
    EvalSignal::AgentStop => LxError::runtime("agent stopped", self.arena.stmt_span(*sid)),
})?;
```

The `eval_expr` method's error mapping is at lines 85-88. Add `AgentStop` arm:

```rust
self.eval(eid).await.map_err(|e| match e {
    EvalSignal::Error(e) => e,
    EvalSignal::Break(_) => LxError::runtime("break outside loop", span),
    EvalSignal::AgentStop => LxError::runtime("agent stopped", span),
})
```

The spawned agent task (in `crates/lx/src/builtins/agent/spawn.rs`) catches `AgentStop` as a clean exit — when `call_value` returns an error, the task simply exits its loop/function without logging it as a failure.

## Verification

After all changes:
1. `just diagnose` must pass with no warnings.
2. `just test` must pass all existing tests.
3. Write a test `.lx` file:
```lx
Agent Worker = {
  handle = (msg) { {echo: msg} }
}
spawn Worker
result = ask "Worker" {hello: "world"}
assert result.echo.hello == "world"
```
This test verifies: Agent keyword desugars, spawn creates a named agent, ask targets by name string, handle returns a value that becomes the ask result.
