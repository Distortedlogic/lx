use std::cell::RefCell;
use std::sync::Arc;

use crate::builtins::call_value_sync;
use crate::error::LxError;
use crate::runtime::{DenyHttpBackend, HttpBackend, RuntimeCtx};
use crate::value::LxVal;
use miette::SourceSpan;

use super::sandbox::{POLICIES, Policy, policy_id};

thread_local! {
    static POLICY_STACK: RefCell<Vec<u64>> = const { RefCell::new(Vec::new()) };
}

fn build_restricted_ctx(base: &Arc<RuntimeCtx>, policy: &Policy) -> Arc<RuntimeCtx> {
  let http: Arc<dyn HttpBackend> = if policy.net_allow.is_empty() { Arc::new(DenyHttpBackend) } else { base.http.clone() };

  Arc::new(RuntimeCtx {
    emit: base.emit.clone(),
    http,
    yield_: base.yield_.clone(),
    log: base.log.clone(),
    ai: base.ai.clone(),
    source_dir: parking_lot::Mutex::new(base.source_dir.lock().clone()),
    workspace_members: base.workspace_members.clone(),
    dep_dirs: base.dep_dirs.clone(),
    tokio_runtime: base.tokio_runtime.clone(),
    test_threshold: base.test_threshold,
    test_runs: base.test_runs,
  })
}

pub fn bi_scope(args: &[LxVal], span: SourceSpan, ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let pid = policy_id(&args[0], span)?;
  let policy = POLICIES.get(&pid).ok_or_else(|| LxError::runtime("sandbox: policy not found", span))?.clone();

  let restricted_ctx = build_restricted_ctx(ctx, &policy);

  POLICY_STACK.with(|stack| stack.borrow_mut().push(pid));
  let result = call_value_sync(&args[1], LxVal::Unit, span, &restricted_ctx);
  POLICY_STACK.with(|stack| stack.borrow_mut().pop());

  result
}
