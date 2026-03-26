use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock};

use dashmap::DashMap;

use crate::error::LxError;
use crate::record;
use crate::runtime::RuntimeCtx;
use crate::value::LxVal;
use miette::SourceSpan;

struct AgentEntry {
  to_agent: Option<Arc<std::sync::mpsc::Sender<LxVal>>>,
  from_agent: Arc<std::sync::Mutex<std::sync::mpsc::Receiver<LxVal>>>,
}

static AGENTS: LazyLock<DashMap<u64, AgentEntry>> = LazyLock::new(DashMap::new);
static NEXT_AGENT_ID: AtomicU64 = AtomicU64::new(1);

fn agent_id(val: &LxVal, fn_name: &str, span: SourceSpan) -> Result<u64, LxError> {
  crate::stdlib::helpers::extract_handle_id(val, "__agent_id", fn_name, span)
}

pub fn bi_agent_spawn(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let config = match &args[0] {
    LxVal::Record(r) => r.clone(),
    _ => return Err(LxError::type_err("agent.spawn: expected config Record", span, None)),
  };

  let script_path = config
    .get(&crate::sym::intern("script"))
    .or_else(|| config.get(&crate::sym::intern("args")).and_then(|a| if let LxVal::List(l) = a { l.last() } else { None }))
    .and_then(|v| v.as_str().map(|s| s.to_string()))
    .ok_or_else(|| LxError::type_err("agent.spawn: config needs 'script' or 'args' with script path", span, None))?;

  let source = std::fs::read_to_string(&script_path).map_err(|e| LxError::runtime(format!("agent.spawn: cannot read {script_path}: {e}"), span))?;

  let (tokens, comments) = crate::lexer::lex(&source).map_err(|e| LxError::runtime(format!("agent.spawn: lex error: {e}"), span))?;
  let result = crate::parser::parse(tokens, crate::source::FileId::new(0), comments, &source);
  let surface = result.program.ok_or_else(|| LxError::runtime("agent.spawn: parse error", span))?;
  let program = crate::folder::desugar(surface);

  let (to_agent_tx, to_agent_rx) = std::sync::mpsc::channel::<LxVal>();
  let (from_agent_tx, from_agent_rx) = std::sync::mpsc::channel::<LxVal>();

  let id = NEXT_AGENT_ID.fetch_add(1, Ordering::Relaxed);
  let source_dir = std::path::Path::new(&script_path).parent().map(|p| p.to_path_buf());

  let yield_rx = Arc::new(std::sync::Mutex::new(to_agent_rx));
  let yield_tx = Arc::new(from_agent_tx);

  tokio::task::spawn_blocking(move || {
    let rt = tokio::runtime::Runtime::new().expect("agent runtime");
    let yield_backend: Arc<dyn crate::runtime::YieldBackend> = Arc::new(ChannelYieldBackend { rx: yield_rx, tx: yield_tx });
    let ctx =
      Arc::new(RuntimeCtx { source_dir: parking_lot::Mutex::new(source_dir), yield_: yield_backend, tokio_runtime: Arc::new(rt), ..RuntimeCtx::default() });
    let source_clone = source.clone();
    ctx.tokio_runtime.clone().block_on(async {
      let mut interp = crate::interpreter::Interpreter::new(&source_clone, None, ctx);
      let _ = interp.exec(&program).await;
    });
  });

  AGENTS.insert(id, AgentEntry { to_agent: Some(Arc::new(to_agent_tx)), from_agent: Arc::new(std::sync::Mutex::new(from_agent_rx)) });

  Ok(record! {
    "__agent_id" => LxVal::int(id),
    "id" => LxVal::int(id)
  })
}

pub fn bi_agent_kill(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let id = agent_id(&args[0], "agent.kill", span)?;
  if let Some((_, mut entry)) = AGENTS.remove(&id) {
    entry.to_agent = None;
  }
  Ok(LxVal::Unit)
}

pub fn bi_agent_ask(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let id = agent_id(&args[0], "agent.ask", span)?;
  let msg = args[1].clone();

  let entry = AGENTS.get(&id).ok_or_else(|| LxError::runtime("agent.ask: agent not found", span))?;
  let sender = match &entry.to_agent {
    Some(s) => Arc::clone(s),
    None => return Ok(LxVal::err_str("agent channel closed")),
  };
  let receiver = Arc::clone(&entry.from_agent);
  drop(entry);

  sender.send(msg).map_err(|_| LxError::runtime("agent.ask: agent channel closed", span))?;

  let guard = receiver.lock().map_err(|_| LxError::runtime("agent.ask: lock poisoned", span))?;
  match guard.recv() {
    Ok(value) => Ok(LxVal::ok(value)),
    Err(_) => Ok(LxVal::err_str("agent channel closed")),
  }
}

pub fn bi_agent_tell(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let id = agent_id(&args[0], "agent.tell", span)?;
  let msg = args[1].clone();

  let entry = AGENTS.get(&id).ok_or_else(|| LxError::runtime("agent.tell: agent not found", span))?;
  let sender = match &entry.to_agent {
    Some(s) => Arc::clone(s),
    None => return Ok(LxVal::err_str("agent channel closed")),
  };
  drop(entry);

  match sender.send(msg) {
    Ok(()) => Ok(LxVal::ok_unit()),
    Err(_) => Ok(LxVal::err_str("agent channel closed")),
  }
}

struct ChannelYieldBackend {
  rx: Arc<std::sync::Mutex<std::sync::mpsc::Receiver<LxVal>>>,
  tx: Arc<std::sync::mpsc::Sender<LxVal>>,
}

impl crate::runtime::YieldBackend for ChannelYieldBackend {
  fn yield_value(&self, value: LxVal, span: SourceSpan) -> Result<LxVal, LxError> {
    self.tx.send(value).map_err(|_| LxError::runtime("agent yield: channel closed", span))?;
    let guard = self.rx.lock().map_err(|_| LxError::runtime("agent yield: lock poisoned", span))?;
    guard.recv().map_err(|_| LxError::runtime("agent yield: no message received", span))
  }
}
