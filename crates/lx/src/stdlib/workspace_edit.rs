use std::sync::Arc;

use regex::Regex;

use crate::backends::RuntimeCtx;
use crate::builtins::call_value_sync;
use crate::error::LxError;
use crate::record;
use crate::span::Span;
use crate::value::Value;

use super::workspace::{EditEntry, Region, WORKSPACES, now_str, ws_id};

pub fn bi_claim(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = ws_id(&args[0], span)?;
    let name = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("workspace.claim: region must be Str", span))?;
    let (start, end) = match &args[2] {
        Value::Record(r) => {
            let s: usize = r
                .get("start")
                .and_then(|v| v.as_int())
                .and_then(|n| n.try_into().ok())
                .ok_or_else(|| LxError::type_err("workspace.claim: start must be Int", span))?;
            let e: usize = r
                .get("end")
                .and_then(|v| v.as_int())
                .and_then(|n| n.try_into().ok())
                .ok_or_else(|| LxError::type_err("workspace.claim: end must be Int", span))?;
            (s, e)
        }
        _ => {
            return Err(LxError::type_err(
                "workspace.claim: bounds must be Record",
                span,
            ));
        }
    };
    let mut ws = WORKSPACES
        .get_mut(&id)
        .ok_or_else(|| LxError::runtime("workspace.claim: not found", span))?;
    let line_count = ws.content.split('\n').count();
    if end >= line_count {
        return Ok(Value::Err(Box::new(Value::Str(Arc::from(
            format!("bounds exceed content: {line_count} lines, end {end}").as_str(),
        )))));
    }
    for (existing_name, existing) in &ws.regions {
        if start <= existing.end && end >= existing.start {
            return Ok(Value::Err(Box::new(Value::Str(Arc::from(
                format!("region overlaps with: {existing_name}").as_str(),
            )))));
        }
    }
    ws.regions.insert(
        name.to_string(),
        Region {
            name: name.to_string(),
            start,
            end,
        },
    );
    Ok(Value::Ok(Box::new(Value::Str(Arc::from(name)))))
}

pub fn bi_claim_pattern(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let id = ws_id(&args[0], span)?;
    let name = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("workspace.claim_pattern: region must be Str", span))?;
    let pattern = args[2]
        .as_str()
        .ok_or_else(|| LxError::type_err("workspace.claim_pattern: pattern must be Str", span))?;
    let re = Regex::new(pattern)
        .map_err(|e| LxError::runtime(format!("workspace.claim_pattern: bad regex: {e}"), span))?;
    let mut ws = WORKSPACES
        .get_mut(&id)
        .ok_or_else(|| LxError::runtime("workspace.claim_pattern: not found", span))?;
    let Some(mat) = re.find(&ws.content) else {
        return Ok(Value::Err(Box::new(Value::Str(Arc::from(
            "pattern not found",
        )))));
    };
    let start_line = ws.content[..mat.start()].matches('\n').count();
    let end_line = start_line + mat.as_str().matches('\n').count();
    for (existing_name, existing) in &ws.regions {
        if start_line <= existing.end && end_line >= existing.start {
            return Ok(Value::Err(Box::new(Value::Str(Arc::from(
                format!("region overlaps with: {existing_name}").as_str(),
            )))));
        }
    }
    ws.regions.insert(
        name.to_string(),
        Region {
            name: name.to_string(),
            start: start_line,
            end: end_line,
        },
    );
    Ok(Value::Ok(Box::new(Value::Str(Arc::from(name)))))
}

pub fn bi_edit(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = ws_id(&args[0], span)?;
    let rname = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("workspace.edit: region must be Str", span))?;
    let new_content = args[2]
        .as_str()
        .ok_or_else(|| LxError::type_err("workspace.edit: content must be Str", span))?;
    let watchers;
    {
        let mut ws = WORKSPACES
            .get_mut(&id)
            .ok_or_else(|| LxError::runtime("workspace.edit: not found", span))?;
        let region = ws.regions.get(rname).ok_or_else(|| {
            LxError::runtime(
                format!("workspace.edit: region '{rname}' not claimed"),
                span,
            )
        })?;
        let start = region.start;
        let end = region.end;
        let mut lines: Vec<String> = ws.content.split('\n').map(String::from).collect();
        let old_count = end - start + 1;
        let new_lines: Vec<String> = new_content.split('\n').map(String::from).collect();
        let new_count = new_lines.len();
        let delta = new_count as isize - old_count as isize;
        let drain_end = (end + 1).min(lines.len());
        lines.splice(start..drain_end, new_lines);
        ws.content = lines.join("\n");
        let new_end = if new_count > 0 {
            start + new_count - 1
        } else {
            start
        };
        let rname_owned = rname.to_string();
        if let Some(r) = ws.regions.get_mut(&rname_owned) {
            r.end = new_end;
        }
        if delta != 0 {
            for (k, r) in ws.regions.iter_mut() {
                if *k != rname_owned && r.start > end {
                    r.start = (r.start as isize + delta) as usize;
                    r.end = (r.end as isize + delta) as usize;
                }
            }
        }
        ws.history.push(EditEntry {
            region: rname_owned,
            at: now_str(),
        });
        watchers = ws.watchers.clone();
    }
    for w in &watchers {
        let event = record! {
            "type" => Value::Str(Arc::from("edit")),
            "region" => Value::Str(Arc::from(rname)),
        };
        let _ = call_value_sync(w, event, span, ctx);
    }
    Ok(Value::Ok(Box::new(Value::Unit)))
}

pub fn bi_append(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = ws_id(&args[0], span)?;
    let rname = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("workspace.append: region must be Str", span))?;
    let extra = args[2]
        .as_str()
        .ok_or_else(|| LxError::type_err("workspace.append: content must be Str", span))?;
    let watchers;
    {
        let mut ws = WORKSPACES
            .get_mut(&id)
            .ok_or_else(|| LxError::runtime("workspace.append: not found", span))?;
        let region = ws.regions.get(rname).ok_or_else(|| {
            LxError::runtime(
                format!("workspace.append: region '{rname}' not claimed"),
                span,
            )
        })?;
        let end = region.end;
        let mut lines: Vec<String> = ws.content.split('\n').map(String::from).collect();
        let new_lines: Vec<String> = extra.split('\n').map(String::from).collect();
        let added = new_lines.len();
        let insert_at = (end + 1).min(lines.len());
        for (i, l) in new_lines.into_iter().enumerate() {
            lines.insert(insert_at + i, l);
        }
        ws.content = lines.join("\n");
        let rname_owned = rname.to_string();
        if let Some(r) = ws.regions.get_mut(&rname_owned) {
            r.end += added;
        }
        for (k, r) in ws.regions.iter_mut() {
            if *k != rname_owned && r.start > end {
                r.start += added;
                r.end += added;
            }
        }
        ws.history.push(EditEntry {
            region: rname_owned,
            at: now_str(),
        });
        watchers = ws.watchers.clone();
    }
    for w in &watchers {
        let event = record! {
            "type" => Value::Str(Arc::from("append")),
            "region" => Value::Str(Arc::from(rname)),
        };
        let _ = call_value_sync(w, event, span, ctx);
    }
    Ok(Value::Ok(Box::new(Value::Unit)))
}

pub fn bi_release(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let id = ws_id(&args[0], span)?;
    let rname = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("workspace.release: region must be Str", span))?;
    let mut ws = WORKSPACES
        .get_mut(&id)
        .ok_or_else(|| LxError::runtime("workspace.release: not found", span))?;
    ws.regions.shift_remove(rname);
    Ok(Value::Unit)
}
