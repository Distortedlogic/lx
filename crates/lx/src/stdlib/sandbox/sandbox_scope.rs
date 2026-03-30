use std::cell::RefCell;
use std::sync::Arc;

use crate::BuiltinCtx;
use crate::builtins::{call_value_sync, extract_runtime_ctx};
use crate::error::LxError;
use crate::runtime::RuntimeCtx;
use crate::value::LxVal;
use miette::SourceSpan;

use super::sandbox::{Policy, get_policy, policy_id};

thread_local! {
    static POLICY_STACK: RefCell<Vec<u64>> = const { RefCell::new(Vec::new()) };
}

fn build_restricted_ctx(base: &Arc<dyn BuiltinCtx>, policy: &Policy) -> Arc<dyn BuiltinCtx> {
  let network_denied = policy.net_allow.is_empty();
  let rtx = extract_runtime_ctx(base.as_ref());

  Arc::new(RuntimeCtx {
    yield_: rtx.yield_.clone(),
    source_dir: parking_lot::Mutex::new(base.source_dir()),
    workspace_members: rtx.workspace_members.clone(),
    dep_dirs: rtx.dep_dirs.clone(),
    tokio_runtime: rtx.tokio_runtime.clone(),
    test_threshold: base.test_threshold(),
    test_runs: base.test_runs(),
    event_stream: Arc::clone(base.event_stream()),
    network_denied,
    global_pause: rtx.global_pause.clone(),
    cancel_flag: rtx.cancel_flag.clone(),
    inject_tx: rtx.inject_tx.clone(),
  })
}

pub fn bi_scope(args: &[LxVal], span: SourceSpan, ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  let pid = policy_id(&args[0], span)?;
  let policy = get_policy(pid, span)?.clone();

  let restricted_ctx = build_restricted_ctx(ctx, &policy);

  POLICY_STACK.with(|stack| stack.borrow_mut().push(pid));
  let result = call_value_sync(&args[1], LxVal::Unit, span, &restricted_ctx);
  POLICY_STACK.with(|stack| stack.borrow_mut().pop());

  result
}
