use std::sync::Arc;

use crate::backends::RuntimeCtx;
use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

use super::hof::{call, get_list};

pub(super) fn bi_take_while(
    args: &[Value],
    sp: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let items = get_list(&args[1], "take_while", sp)?;
    let mut out = Vec::new();
    for v in items.iter() {
        if call(&args[0], v.clone(), sp, ctx)?.as_bool() != Some(true) {
            break;
        }
        out.push(v.clone());
    }
    Ok(Value::List(Arc::new(out)))
}

pub(super) fn bi_drop_while(
    args: &[Value],
    sp: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let items = get_list(&args[1], "drop_while", sp)?;
    let mut dropping = true;
    let mut out = Vec::new();
    for v in items.iter() {
        if dropping && call(&args[0], v.clone(), sp, ctx)?.as_bool() == Some(true) {
            continue;
        }
        dropping = false;
        out.push(v.clone());
    }
    Ok(Value::List(Arc::new(out)))
}

pub(super) fn bi_sort_by(
    args: &[Value],
    sp: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let items = get_list(&args[1], "sort_by", sp)?;
    let mut keyed: Vec<(Value, Value)> = Vec::with_capacity(items.len());
    for v in items.iter() {
        let k = call(&args[0], v.clone(), sp, ctx)?;
        keyed.push((k, v.clone()));
    }
    keyed.sort_by(|(a, _), (b, _)| super::coll::cmp_values(a, b));
    Ok(Value::List(Arc::new(
        keyed.into_iter().map(|(_, v)| v).collect(),
    )))
}

pub(super) fn bi_min_by(args: &[Value], sp: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let items = get_list(&args[1], "min_by", sp)?;
    if items.is_empty() {
        return Err(LxError::runtime("min_by: empty list", sp));
    }
    let mut best = &items[0];
    let mut best_key = call(&args[0], best.clone(), sp, ctx)?;
    for v in &items[1..] {
        let k = call(&args[0], v.clone(), sp, ctx)?;
        if super::coll::cmp_values(&k, &best_key).is_lt() {
            best = v;
            best_key = k;
        }
    }
    Ok(best.clone())
}

pub(super) fn bi_max_by(args: &[Value], sp: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let items = get_list(&args[1], "max_by", sp)?;
    if items.is_empty() {
        return Err(LxError::runtime("max_by: empty list", sp));
    }
    let mut best = &items[0];
    let mut best_key = call(&args[0], best.clone(), sp, ctx)?;
    for v in &items[1..] {
        let k = call(&args[0], v.clone(), sp, ctx)?;
        if super::coll::cmp_values(&k, &best_key).is_gt() {
            best = v;
            best_key = k;
        }
    }
    Ok(best.clone())
}

pub(super) fn bi_partition(
    args: &[Value],
    sp: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let items = get_list(&args[1], "partition", sp)?;
    let (mut yes, mut no) = (Vec::new(), Vec::new());
    for v in items.iter() {
        if call(&args[0], v.clone(), sp, ctx)?.as_bool() == Some(true) {
            yes.push(v.clone());
        } else {
            no.push(v.clone());
        }
    }
    Ok(Value::Tuple(Arc::new(vec![
        Value::List(Arc::new(yes)),
        Value::List(Arc::new(no)),
    ])))
}

pub(super) fn bi_group_by(
    args: &[Value],
    sp: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let items = get_list(&args[1], "group_by", sp)?;
    let mut groups = indexmap::IndexMap::new();
    for v in items.iter() {
        let key = call(&args[0], v.clone(), sp, ctx)?;
        groups
            .entry(crate::value::ValueKey(key))
            .or_insert_with(Vec::new)
            .push(v.clone());
    }
    let map: indexmap::IndexMap<crate::value::ValueKey, Value> = groups
        .into_iter()
        .map(|(k, vs)| (k, Value::List(Arc::new(vs))))
        .collect();
    Ok(Value::Map(Arc::new(map)))
}

pub(super) fn bi_chunks(
    args: &[Value],
    sp: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let n = args[0]
        .as_int()
        .ok_or_else(|| LxError::type_err(format!("chunks: size must be Int, got {}", args[0].type_name()), sp))?;
    let items = get_list(&args[1], "chunks", sp)?;
    let n = usize::try_from(n.clone()).map_err(|_| LxError::runtime("chunks: invalid size", sp))?;
    if n == 0 {
        return Err(LxError::runtime("chunks: size must be > 0", sp));
    }
    let out: Vec<Value> = items
        .chunks(n)
        .map(|chunk| Value::List(Arc::new(chunk.to_vec())))
        .collect();
    Ok(Value::List(Arc::new(out)))
}

pub(super) fn bi_windows(
    args: &[Value],
    sp: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let n = args[0]
        .as_int()
        .ok_or_else(|| LxError::type_err(format!("windows: size must be Int, got {}", args[0].type_name()), sp))?;
    let items = get_list(&args[1], "windows", sp)?;
    let n =
        usize::try_from(n.clone()).map_err(|_| LxError::runtime("windows: invalid size", sp))?;
    if n == 0 || items.len() < n {
        return Ok(Value::List(Arc::new(vec![])));
    }
    let out: Vec<Value> = items
        .windows(n)
        .map(|w| Value::List(Arc::new(w.to_vec())))
        .collect();
    Ok(Value::List(Arc::new(out)))
}

pub(super) fn bi_intersperse(
    args: &[Value],
    sp: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let items = get_list(&args[1], "intersperse", sp)?;
    let sep = &args[0];
    let mut out = Vec::with_capacity(items.len() * 2);
    for (i, v) in items.iter().enumerate() {
        if i > 0 {
            out.push(sep.clone());
        }
        out.push(v.clone());
    }
    Ok(Value::List(Arc::new(out)))
}

pub(super) fn bi_scan(args: &[Value], sp: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let items = get_list(&args[2], "scan", sp)?;
    let mut acc = args[0].clone();
    let f = &args[1];
    let mut out = Vec::with_capacity(items.len() + 1);
    out.push(acc.clone());
    for v in items.iter() {
        let partial = call(f, acc, sp, ctx)?;
        acc = call(&partial, v.clone(), sp, ctx)?;
        out.push(acc.clone());
    }
    Ok(Value::List(Arc::new(out)))
}

pub(super) fn bi_tap(args: &[Value], sp: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let val = &args[1];
    call(&args[0], val.clone(), sp, ctx)?;
    Ok(val.clone())
}

pub(super) fn bi_pmap(args: &[Value], sp: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let items = get_list(&args[1], "pmap", sp)?;
    let mut out = Vec::with_capacity(items.len());
    for v in items.iter() {
        out.push(call(&args[0], v.clone(), sp, ctx)?);
    }
    Ok(Value::List(Arc::new(out)))
}

pub(super) fn bi_pmap_n(args: &[Value], sp: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let items = get_list(&args[2], "pmap_n", sp)?;
    let mut out = Vec::with_capacity(items.len());
    for v in items.iter() {
        out.push(call(&args[1], v.clone(), sp, ctx)?);
    }
    Ok(Value::List(Arc::new(out)))
}

pub(super) fn bi_find_index(
    args: &[Value],
    sp: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let items = get_list(&args[1], "find_index", sp)?;
    for (i, v) in items.iter().enumerate() {
        let result = call(&args[0], v.clone(), sp, ctx)?;
        if result.as_bool() == Some(true) {
            return Ok(Value::Some(Box::new(Value::Int(i.into()))));
        }
    }
    Ok(Value::None)
}
