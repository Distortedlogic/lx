use std::pin::Pin;
use std::sync::Arc;

use num_traits::ToPrimitive;

use crate::backends::RuntimeCtx;
use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

pub(super) fn register(env: &mut crate::env::Env) {
    use super::{mk, mk_async};
    env.bind("map".into(), mk_async("map", 2, bi_map));
    env.bind("filter".into(), mk_async("filter", 2, bi_filter));
    env.bind("fold".into(), mk_async("fold", 3, bi_fold));
    env.bind("flat_map".into(), mk_async("flat_map", 2, bi_flat_map));
    env.bind("each".into(), mk_async("each", 2, bi_each));
    env.bind("take".into(), mk("take", 2, bi_take));
    env.bind("drop".into(), mk("drop", 2, bi_drop));
    env.bind("zip".into(), mk("zip", 2, bi_zip));
    env.bind("enumerate".into(), mk("enumerate", 1, bi_enumerate));
    env.bind("find".into(), mk_async("find", 2, bi_find));
    env.bind("any?".into(), mk_async("any?", 2, bi_any));
    env.bind("all?".into(), mk_async("all?", 2, bi_all));
    env.bind("none?".into(), mk_async("none?", 2, bi_none_q));
    env.bind(
        "count".into(),
        mk_async("count", 2, super::hof_extra::bi_count),
    );
    env.bind(
        "take_while".into(),
        mk_async("take_while", 2, super::hof_extra::bi_take_while),
    );
    env.bind(
        "drop_while".into(),
        mk_async("drop_while", 2, super::hof_extra::bi_drop_while),
    );
    env.bind(
        "sort_by".into(),
        mk_async("sort_by", 2, super::hof_extra::bi_sort_by),
    );
    env.bind(
        "min_by".into(),
        mk_async("min_by", 2, super::hof_extra::bi_min_by),
    );
    env.bind(
        "max_by".into(),
        mk_async("max_by", 2, super::hof_extra::bi_max_by),
    );
    env.bind(
        "partition".into(),
        mk_async("partition", 2, super::hof_extra::bi_partition),
    );
    env.bind(
        "group_by".into(),
        mk_async("group_by", 2, super::hof_extra::bi_group_by),
    );
    env.bind(
        "chunks".into(),
        mk("chunks", 2, super::hof_extra::bi_chunks),
    );
    env.bind(
        "windows".into(),
        mk("windows", 2, super::hof_extra::bi_windows),
    );
    env.bind(
        "intersperse".into(),
        mk("intersperse", 2, super::hof_extra::bi_intersperse),
    );
    env.bind(
        "scan".into(),
        mk_async("scan", 3, super::hof_extra::bi_scan),
    );
    env.bind("tap".into(), mk_async("tap", 2, super::hof_extra::bi_tap));
    env.bind(
        "find_index".into(),
        mk_async("find_index", 2, super::hof_extra::bi_find_index),
    );
    env.bind(
        "pmap".into(),
        mk_async("pmap", 2, super::hof_parallel::bi_pmap),
    );
    env.bind(
        "pmap_n".into(),
        mk_async("pmap_n", 3, super::hof_parallel::bi_pmap_n),
    );
}

type BoxFut = Pin<Box<dyn std::future::Future<Output = Result<Value, LxError>>>>;

pub(super) async fn call(
    f: &Value,
    arg: Value,
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    crate::builtins::call_value(f, arg, span, ctx).await
}

fn range_to_list(start: i64, end: i64, inclusive: bool) -> Vec<Value> {
    if inclusive {
        (start..=end).map(|i| Value::Int(i.into())).collect()
    } else {
        (start..end).map(|i| Value::Int(i.into())).collect()
    }
}

pub(super) enum ListRef<'a> {
    Borrowed(&'a [Value]),
    Owned(Vec<Value>),
}

impl<'a> std::ops::Deref for ListRef<'a> {
    type Target = [Value];
    fn deref(&self) -> &[Value] {
        match self {
            ListRef::Borrowed(s) => s,
            ListRef::Owned(v) => v.as_slice(),
        }
    }
}

pub(super) fn get_list<'a>(v: &'a Value, name: &str, sp: Span) -> Result<ListRef<'a>, LxError> {
    match v {
        Value::List(l) => Ok(ListRef::Borrowed(l.as_ref())),
        Value::Range {
            start,
            end,
            inclusive,
        } => Ok(ListRef::Owned(range_to_list(*start, *end, *inclusive))),
        Value::Stream { rx, .. } => {
            let items: Vec<Value> = rx.lock().try_iter().collect();
            Ok(ListRef::Owned(items))
        }
        other => Err(LxError::type_err(
            format!(
                "{name} expects List, Range, or Stream, got {}",
                other.type_name()
            ),
            sp,
        )),
    }
}

fn bi_map(args: Vec<Value>, sp: Span, ctx: Arc<RuntimeCtx>) -> BoxFut {
    Box::pin(async move {
        let items = get_list(&args[1], "map", sp)?;
        let mut out = Vec::with_capacity(items.len());
        for v in items.iter() {
            out.push(call(&args[0], v.clone(), sp, &ctx).await?);
        }
        Ok(Value::List(Arc::new(out)))
    })
}

fn bi_filter(args: Vec<Value>, sp: Span, ctx: Arc<RuntimeCtx>) -> BoxFut {
    Box::pin(async move {
        let items = get_list(&args[1], "filter", sp)?;
        let mut out = Vec::new();
        for v in items.iter() {
            let result = call(&args[0], v.clone(), sp, &ctx).await?;
            match result.as_bool() {
                Some(true) => out.push(v.clone()),
                Some(false) => {}
                _ => {
                    return Err(LxError::type_err(
                        format!(
                            "filter predicate must return Bool, got {}",
                            result.type_name()
                        ),
                        sp,
                    ));
                }
            }
        }
        Ok(Value::List(Arc::new(out)))
    })
}

fn bi_fold(args: Vec<Value>, sp: Span, ctx: Arc<RuntimeCtx>) -> BoxFut {
    Box::pin(async move {
        let items = get_list(&args[2], "fold", sp)?;
        let mut acc = args[0].clone();
        let f = &args[1];
        for v in items.iter() {
            let partial = call(f, acc, sp, &ctx).await?;
            acc = call(&partial, v.clone(), sp, &ctx).await?;
        }
        Ok(acc)
    })
}

fn bi_flat_map(args: Vec<Value>, sp: Span, ctx: Arc<RuntimeCtx>) -> BoxFut {
    Box::pin(async move {
        let items = get_list(&args[1], "flat_map", sp)?;
        let mut out = Vec::new();
        for v in items.iter() {
            let result = call(&args[0], v.clone(), sp, &ctx).await?;
            match result {
                Value::List(l) => out.extend(l.as_ref().iter().cloned()),
                other => out.push(other),
            }
        }
        Ok(Value::List(Arc::new(out)))
    })
}

fn bi_each(args: Vec<Value>, sp: Span, ctx: Arc<RuntimeCtx>) -> BoxFut {
    Box::pin(async move {
        let items = get_list(&args[1], "each", sp)?;
        for v in items.iter() {
            call(&args[0], v.clone(), sp, &ctx).await?;
        }
        Ok(Value::Unit)
    })
}

fn bi_take(args: &[Value], sp: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let n = args[0].as_int().ok_or_else(|| {
        LxError::type_err(
            format!("take: first arg must be Int, got {}", args[0].type_name()),
            sp,
        )
    })?;
    let n = n
        .to_usize()
        .ok_or_else(|| LxError::runtime("take: count out of range", sp))?;
    let items = get_list(&args[1], "take", sp)?;
    Ok(Value::List(Arc::new(
        items.iter().take(n).cloned().collect(),
    )))
}

fn bi_drop(args: &[Value], sp: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let n = args[0].as_int().ok_or_else(|| {
        LxError::type_err(
            format!("drop: first arg must be Int, got {}", args[0].type_name()),
            sp,
        )
    })?;
    let n = n
        .to_usize()
        .ok_or_else(|| LxError::runtime("drop: count out of range", sp))?;
    let items = get_list(&args[1], "drop", sp)?;
    Ok(Value::List(Arc::new(
        items.iter().skip(n).cloned().collect(),
    )))
}

fn bi_zip(args: &[Value], sp: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let a = get_list(&args[0], "zip", sp)?;
    let b = get_list(&args[1], "zip", sp)?;
    let out: Vec<Value> = a
        .iter()
        .zip(b.iter())
        .map(|(x, y)| Value::Tuple(Arc::new(vec![y.clone(), x.clone()])))
        .collect();
    Ok(Value::List(Arc::new(out)))
}

fn bi_enumerate(args: &[Value], sp: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let items = get_list(&args[0], "enumerate", sp)?;
    let out: Vec<Value> = items
        .iter()
        .enumerate()
        .map(|(i, v)| Value::Tuple(Arc::new(vec![Value::Int(i.into()), v.clone()])))
        .collect();
    Ok(Value::List(Arc::new(out)))
}

fn bi_find(args: Vec<Value>, sp: Span, ctx: Arc<RuntimeCtx>) -> BoxFut {
    Box::pin(async move {
        let items = get_list(&args[1], "find", sp)?;
        for v in items.iter() {
            let result = call(&args[0], v.clone(), sp, &ctx).await?;
            if result.as_bool() == Some(true) {
                return Ok(Value::Some(Box::new(v.clone())));
            }
        }
        Ok(Value::None)
    })
}

fn bi_any(args: Vec<Value>, sp: Span, ctx: Arc<RuntimeCtx>) -> BoxFut {
    Box::pin(async move {
        let items = get_list(&args[1], "any?", sp)?;
        for v in items.iter() {
            if call(&args[0], v.clone(), sp, &ctx).await?.as_bool() == Some(true) {
                return Ok(Value::Bool(true));
            }
        }
        Ok(Value::Bool(false))
    })
}

fn bi_all(args: Vec<Value>, sp: Span, ctx: Arc<RuntimeCtx>) -> BoxFut {
    Box::pin(async move {
        let items = get_list(&args[1], "all?", sp)?;
        for v in items.iter() {
            if call(&args[0], v.clone(), sp, &ctx).await?.as_bool() != Some(true) {
                return Ok(Value::Bool(false));
            }
        }
        Ok(Value::Bool(true))
    })
}

fn bi_none_q(args: Vec<Value>, sp: Span, ctx: Arc<RuntimeCtx>) -> BoxFut {
    Box::pin(async move {
        let items = get_list(&args[1], "none?", sp)?;
        for v in items.iter() {
            if call(&args[0], v.clone(), sp, &ctx).await?.as_bool() == Some(true) {
                return Ok(Value::Bool(false));
            }
        }
        Ok(Value::Bool(true))
    })
}
