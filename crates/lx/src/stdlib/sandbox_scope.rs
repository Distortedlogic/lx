use std::cell::RefCell;
use std::sync::Arc;

use crate::backends::{
    AiBackend, DenyAiBackend, DenyEmbedBackend, DenyHttpBackend, DenyPaneBackend,
    DenyShellBackend, EmbedBackend, HttpBackend, PaneBackend, RestrictedShellBackend, RuntimeCtx,
    ShellBackend,
};
use crate::builtins::call_value_sync;
use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

use super::sandbox::{POLICIES, Policy, ShellPolicy, policy_id};

thread_local! {
    static POLICY_STACK: RefCell<Vec<u64>> = const { RefCell::new(Vec::new()) };
}

fn build_restricted_ctx(base: &Arc<RuntimeCtx>, policy: &Policy) -> Arc<RuntimeCtx> {
    let ai: Arc<dyn AiBackend> = if policy.ai {
        base.ai.clone()
    } else {
        Arc::new(DenyAiBackend)
    };

    let pane: Arc<dyn PaneBackend> = if policy.pane {
        base.pane.clone()
    } else {
        Arc::new(DenyPaneBackend)
    };

    let embed: Arc<dyn EmbedBackend> = if policy.embed {
        base.embed.clone()
    } else {
        Arc::new(DenyEmbedBackend)
    };

    let http: Arc<dyn HttpBackend> = if policy.net_allow.is_empty() {
        Arc::new(DenyHttpBackend)
    } else {
        base.http.clone()
    };

    let shell: Arc<dyn ShellBackend> = match &policy.shell {
        ShellPolicy::Deny => Arc::new(DenyShellBackend),
        ShellPolicy::AllowList(cmds) => Arc::new(RestrictedShellBackend {
            inner: base.shell.clone(),
            allowed_cmds: cmds.clone(),
        }),
        ShellPolicy::Allow => base.shell.clone(),
    };

    Arc::new(RuntimeCtx {
        ai,
        emit: base.emit.clone(),
        http,
        shell,
        yield_: base.yield_.clone(),
        log: base.log.clone(),
        user: base.user.clone(),
        pane,
        embed,
        on_agent_event: base.on_agent_event.clone(),
        source_dir: parking_lot::Mutex::new(base.source_dir.lock().clone()),
        workspace_members: base.workspace_members.clone(),
        dep_dirs: base.dep_dirs.clone(),
        tokio_runtime: base.tokio_runtime.clone(),
        test_threshold: base.test_threshold,
        test_runs: base.test_runs,
    })
}

pub fn bi_scope(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let pid = policy_id(&args[0], span)?;
    let policy = POLICIES
        .get(&pid)
        .ok_or_else(|| LxError::runtime("sandbox: policy not found", span))?
        .clone();

    let restricted_ctx = build_restricted_ctx(ctx, &policy);

    POLICY_STACK.with(|stack| stack.borrow_mut().push(pid));
    let result = call_value_sync(&args[1], Value::Unit, span, &restricted_ctx);
    POLICY_STACK.with(|stack| stack.borrow_mut().pop());

    result
}
