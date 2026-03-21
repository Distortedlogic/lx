use std::sync::Arc;

use indexmap::IndexMap;
use num_bigint::BigInt;

use crate::backends::{LogLevel, RuntimeCtx};
use crate::env::Env;
use crate::error::LxError;
use crate::span::Span;
use crate::value::LxVal;

use super::mk;

fn make_log_builtin(name: &'static str, level: LogLevel) -> LxVal {
    fn log_fn(
        args: &[LxVal],
        span: Span,
        ctx: &Arc<RuntimeCtx>,
        level: LogLevel,
        name: &str,
    ) -> Result<LxVal, LxError> {
        let s = args[0].as_str().ok_or_else(|| {
            LxError::type_err(
                format!("log.{name} expects Str, got {}", args[0].type_name()),
                span,
            )
        })?;
        ctx.log.log(level, s);
        Ok(LxVal::Unit)
    }
    match level {
        LogLevel::Info => mk(name, 1, |a, s, c| log_fn(a, s, c, LogLevel::Info, "info")),
        LogLevel::Warn => mk(name, 1, |a, s, c| log_fn(a, s, c, LogLevel::Warn, "warn")),
        LogLevel::Err => mk(name, 1, |a, s, c| log_fn(a, s, c, LogLevel::Err, "err")),
        LogLevel::Debug => mk(name, 1, |a, s, c| log_fn(a, s, c, LogLevel::Debug, "debug")),
    }
}

fn bi_not(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
    match &args[0] {
        LxVal::Bool(b) => Ok(LxVal::Bool(!b)),
        other => Err(LxError::type_err(
            format!("not expects Bool, got {}", other.type_name()),
            span,
        )),
    }
}

fn bi_len(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
    let n = match &args[0] {
        LxVal::Str(s) => s.chars().count(),
        LxVal::List(l) => l.len(),
        LxVal::Record(r) => r.len(),
        LxVal::Map(m) => m.len(),
        LxVal::Tuple(t) => t.len(),
        LxVal::Store { id } => crate::stdlib::store_len(*id),
        other => {
            return Err(LxError::type_err(
                format!("len expects collection, got {}", other.type_name()),
                span,
            ));
        }
    };
    Ok(LxVal::Int(BigInt::from(n)))
}

fn bi_empty(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
    let empty = match &args[0] {
        LxVal::Str(s) => s.is_empty(),
        LxVal::List(l) => l.is_empty(),
        LxVal::Record(r) => r.is_empty(),
        LxVal::Map(m) => m.is_empty(),
        LxVal::Tuple(t) => t.is_empty(),
        LxVal::Store { id } => crate::stdlib::store_len(*id) == 0,
        other => {
            return Err(LxError::type_err(
                format!("empty? expects collection, got {}", other.type_name()),
                span,
            ));
        }
    };
    Ok(LxVal::Bool(empty))
}

fn bi_to_str(args: &[LxVal], _span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
    Ok(LxVal::Str(Arc::from(format!("{}", args[0]).as_str())))
}

fn bi_identity(args: &[LxVal], _span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
    Ok(args[0].clone())
}

fn bi_dbg(args: &[LxVal], _span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
    eprintln!("[dbg] {}", args[0]);
    Ok(args[0].clone())
}

fn bi_ok_q(args: &[LxVal], _span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
    Ok(LxVal::Bool(matches!(&args[0], LxVal::Ok(_))))
}

fn bi_err_q(args: &[LxVal], _span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
    Ok(LxVal::Bool(matches!(&args[0], LxVal::Err(_))))
}

fn bi_some_q(args: &[LxVal], _span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
    Ok(LxVal::Bool(matches!(&args[0], LxVal::Some(_))))
}

fn bi_even(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
    match &args[0] {
        LxVal::Int(n) => Ok(LxVal::Bool(n % BigInt::from(2) == BigInt::from(0))),
        other => Err(LxError::type_err(
            format!("even? expects Int, got {}", other.type_name()),
            span,
        )),
    }
}

fn bi_odd(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
    match &args[0] {
        LxVal::Int(n) => Ok(LxVal::Bool(n % BigInt::from(2) != BigInt::from(0))),
        other => Err(LxError::type_err(
            format!("odd? expects Int, got {}", other.type_name()),
            span,
        )),
    }
}

fn bi_type_of(args: &[LxVal], _span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
    Ok(LxVal::Str(Arc::from(args[0].type_name())))
}

fn bi_print(args: &[LxVal], _span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
    println!("{}", args[0]);
    Ok(LxVal::Unit)
}

pub fn register(env: &mut Env) {
    env.bind("true".into(), LxVal::Bool(true));
    env.bind("false".into(), LxVal::Bool(false));
    env.bind("None".into(), LxVal::None);
    env.bind(
        "Ok".into(),
        mk("Ok", 1, |a, _, _ctx| Ok(LxVal::Ok(Box::new(a[0].clone())))),
    );
    env.bind(
        "Err".into(),
        mk("Err", 1, |a, _, _ctx| {
            Ok(LxVal::Err(Box::new(a[0].clone())))
        }),
    );
    env.bind(
        "Some".into(),
        mk("Some", 1, |a, _, _ctx| {
            Ok(LxVal::Some(Box::new(a[0].clone())))
        }),
    );
    env.bind("not".into(), mk("not", 1, bi_not));
    env.bind("len".into(), mk("len", 1, bi_len));
    env.bind("empty?".into(), mk("empty?", 1, bi_empty));
    env.bind("to_str".into(), mk("to_str", 1, bi_to_str));
    env.bind("identity".into(), mk("identity", 1, bi_identity));
    env.bind("dbg".into(), mk("dbg", 1, bi_dbg));
    env.bind("ok?".into(), mk("ok?", 1, bi_ok_q));
    env.bind("err?".into(), mk("err?", 1, bi_err_q));
    env.bind("some?".into(), mk("some?", 1, bi_some_q));
    env.bind("even?".into(), mk("even?", 1, bi_even));
    env.bind("odd?".into(), mk("odd?", 1, bi_odd));
    env.bind("type_of".into(), mk("type_of", 1, bi_type_of));
    env.bind("print".into(), mk("print", 1, bi_print));
    super::convert::register(env);
    super::str::register(env);
    super::coll::register(env);
    let mut log_fields = IndexMap::new();
    log_fields.insert("info".into(), make_log_builtin("log.info", LogLevel::Info));
    log_fields.insert("warn".into(), make_log_builtin("log.warn", LogLevel::Warn));
    log_fields.insert("err".into(), make_log_builtin("log.err", LogLevel::Err));
    log_fields.insert(
        "debug".into(),
        make_log_builtin("log.debug", LogLevel::Debug),
    );
    env.bind("log".into(), LxVal::Record(Arc::new(log_fields)));
    super::hof::register(env);
    env.bind("Store".into(), crate::stdlib::build_constructor());
    env.bind("method_of".into(), mk("method_of", 2, bi_method_of));
    env.bind("methods_of".into(), mk("methods_of", 1, bi_methods_of));

    let mut json_fields = IndexMap::new();
    json_fields.insert("parse".into(), mk("json.parse", 1, bi_json_parse));
    json_fields.insert("encode".into(), mk("json.encode", 1, bi_json_encode));
    json_fields.insert(
        "encode_pretty".into(),
        mk("json.encode_pretty", 1, bi_json_encode_pretty),
    );
    env.bind("json".into(), LxVal::Record(Arc::new(json_fields)));

    super::ai_builtins::register_ai(env);

    let mut agent_fields = IndexMap::new();
    agent_fields.insert("spawn".into(), mk("agent.spawn", 1, bi_agent_spawn_stub));
    agent_fields.insert("kill".into(), mk("agent.kill", 1, bi_agent_kill_stub));
    agent_fields.insert(
        "implements".into(),
        mk("agent.implements", 2, bi_agent_implements),
    );
    env.bind("agent".into(), LxVal::Record(Arc::new(agent_fields)));

    let mut pane_fields = IndexMap::new();
    pane_fields.insert("open".into(), mk("pane.open", 2, bi_pane_open));
    pane_fields.insert("update".into(), mk("pane.update", 2, bi_pane_update));
    pane_fields.insert("close".into(), mk("pane.close", 1, bi_pane_close));
    pane_fields.insert("list".into(), mk("pane.list", 1, bi_pane_list));
    env.bind("pane".into(), LxVal::Record(Arc::new(pane_fields)));
    env.bind("try".into(), mk("try", 2, bi_try));
    env.bind(
        "resolve_handler".into(),
        mk("resolve_handler", 1, bi_resolve_handler),
    );
    let mut ctx_fields = IndexMap::new();
    ctx_fields.insert(
        "current".into(),
        mk("context.current", 1, bi_global_context_current),
    );
    ctx_fields.insert("get".into(), mk("context.get", 1, bi_global_context_get));
    env.bind("context".into(), LxVal::Record(Arc::new(ctx_fields)));
}

fn bi_agent_spawn_stub(
    _args: &[LxVal],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<LxVal, LxError> {
    Err(LxError::runtime("agent.spawn: subprocess agents not yet available in this build", span))
}

fn bi_agent_kill_stub(
    _args: &[LxVal],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<LxVal, LxError> {
    Err(LxError::runtime("agent.kill: subprocess agents not yet available in this build", span))
}

fn bi_agent_implements(
    args: &[LxVal],
    _span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<LxVal, LxError> {
    let target_trait = match &args[1] {
        LxVal::Str(s) => s.to_string(),
        LxVal::Trait { name, .. } => name.to_string(),
        LxVal::Class { name, .. } => name.to_string(),
        _ => String::new(),
    };
    let has = match &args[0] {
        LxVal::Object { traits, .. } | LxVal::Class { traits, .. } => {
            traits.iter().any(|t| t.as_ref() == target_trait.as_str())
        }
        LxVal::Record(r) => {
            r.get("traits")
                .or_else(|| r.get("__traits"))
                .and_then(|v| v.as_list())
                .map(|l| l.iter().any(|v| v.as_str() == Some(target_trait.as_str())))
                .unwrap_or(false)
        }
        _ => false,
    };
    Ok(LxVal::Bool(has))
}

fn bi_pane_open(args: &[LxVal], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
    let kind = args[0].as_str().ok_or_else(|| LxError::type_err("pane.open: kind must be Str", span))?;
    ctx.pane.open(kind, &args[1], span)
}

fn bi_pane_update(args: &[LxVal], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
    let id = args[0].as_str().ok_or_else(|| LxError::type_err("pane.update: id must be Str", span))?;
    ctx.pane.update(id, &args[1], span)?;
    Ok(LxVal::Unit)
}

fn bi_pane_close(args: &[LxVal], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
    let id = args[0].as_str().ok_or_else(|| LxError::type_err("pane.close: id must be Str", span))?;
    ctx.pane.close(id, span)?;
    Ok(LxVal::Unit)
}

fn bi_pane_list(_args: &[LxVal], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
    ctx.pane.list(span)
}

fn bi_json_parse(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
    let s = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("json.parse expects Str", span))?;
    match serde_json::from_str::<serde_json::Value>(s) {
        Ok(jv) => Ok(LxVal::Ok(Box::new(LxVal::from(jv)))),
        Err(e) => Ok(LxVal::Err(Box::new(LxVal::Str(Arc::from(e.to_string().as_str()))))),
    }
}

fn bi_json_encode(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
    let s = serde_json::to_string(&args[0])
        .map_err(|e| LxError::runtime(e.to_string(), span))?;
    Ok(LxVal::Str(Arc::from(s.as_str())))
}

fn bi_json_encode_pretty(
    args: &[LxVal],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<LxVal, LxError> {
    let s = serde_json::to_string_pretty(&args[0])
        .map_err(|e| LxError::runtime(e.to_string(), span))?;
    Ok(LxVal::Str(Arc::from(s.as_str())))
}

fn bi_method_of(args: &[LxVal], _span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
    let name = match &args[1] {
        LxVal::Str(s) => s.as_ref(),
        _ => return Ok(LxVal::None),
    };
    match &args[0] {
        LxVal::Object { methods, .. } | LxVal::Class { methods, .. } => match methods.get(name) {
            Some(method) => Ok(inject_self_for_method(method, &args[0])),
            None => Ok(LxVal::None),
        },
        _ => Ok(LxVal::None),
    }
}

fn inject_self_for_method(method: &LxVal, self_val: &LxVal) -> LxVal {
    if let LxVal::Func(lf) = method {
        let mut env = lf.closure.child();
        env.bind("self".to_string(), self_val.clone());
        let mut lf = lf.clone();
        lf.closure = env.into_arc();
        LxVal::Func(lf)
    } else {
        method.clone()
    }
}

fn bi_global_context_current(
    _args: &[LxVal],
    _span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<LxVal, LxError> {
    crate::interpreter::ambient::global_context_current()
}

fn bi_global_context_get(
    args: &[LxVal],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<LxVal, LxError> {
    crate::interpreter::ambient::global_context_get(&args[0], span)
}

fn bi_resolve_handler(
    args: &[LxVal],
    _span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<LxVal, LxError> {
    let agent = &args[0];
    if let LxVal::Record(r) = agent
        && let Some(h) = r.get("handler")
    {
        return Ok(h.clone());
    }
    Ok(LxVal::None)
}

fn bi_try(args: &[LxVal], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
    let f = &args[0];
    let arg = args[1].clone();
    match crate::builtins::call_value_sync(f, arg, span, ctx) {
        Ok(v) => Ok(v),
        Err(LxError::Propagate { value, .. }) => Ok(LxVal::Err(value)),
        Err(e) => Err(e),
    }
}

fn bi_methods_of(args: &[LxVal], _span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
    let names = match &args[0] {
        LxVal::Object { methods, .. } | LxVal::Class { methods, .. } => methods
            .keys()
            .map(|k| LxVal::Str(Arc::from(k.as_str())))
            .collect(),
        _ => vec![],
    };
    Ok(LxVal::List(Arc::new(names)))
}
