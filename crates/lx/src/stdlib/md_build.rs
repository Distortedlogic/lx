use std::sync::Arc;

use indexmap::IndexMap;
use num_bigint::BigInt;

use crate::backends::RuntimeCtx;
use crate::builtins::mk;
use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

use super::md::node_rec;

pub fn register(m: &mut IndexMap<String, Value>) {
    m.insert("h1".into(), mk("md.h1", 1, bi_h1));
    m.insert("h2".into(), mk("md.h2", 1, bi_h2));
    m.insert("h3".into(), mk("md.h3", 1, bi_h3));
    m.insert("para".into(), mk("md.para", 1, bi_para));
    m.insert("code".into(), mk("md.code", 2, bi_code));
    m.insert("list".into(), mk("md.list", 1, bi_list));
    m.insert("ordered".into(), mk("md.ordered", 1, bi_ordered));
    m.insert("table".into(), mk("md.table", 2, bi_table));
    m.insert("link".into(), mk("md.link", 2, bi_link));
    m.insert("blockquote".into(), mk("md.blockquote", 1, bi_blockquote));
    m.insert("hr".into(), node_rec("hr", vec![]));
    m.insert("raw".into(), mk("md.raw", 1, bi_raw));
    m.insert("doc".into(), mk("md.doc", 1, bi_doc));
}

fn heading(level: i64, args: &[Value], span: Span) -> Result<Value, LxError> {
    let text = args[0].as_str()
        .ok_or_else(|| LxError::type_err("md heading expects Str", span))?;
    Ok(node_rec("heading", vec![
        ("level", Value::Int(BigInt::from(level))),
        ("text", Value::Str(Arc::from(text))),
    ]))
}

fn bi_h1(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> { heading(1, args, span) }
fn bi_h2(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> { heading(2, args, span) }
fn bi_h3(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> { heading(3, args, span) }

fn bi_para(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let text = args[0].as_str()
        .ok_or_else(|| LxError::type_err("md.para expects Str", span))?;
    Ok(node_rec("para", vec![("text", Value::Str(Arc::from(text)))]))
}

fn bi_code(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let lang = args[0].as_str()
        .ok_or_else(|| LxError::type_err("md.code expects Str lang", span))?;
    let code = args[1].as_str()
        .ok_or_else(|| LxError::type_err("md.code expects Str code", span))?;
    Ok(node_rec("code", vec![
        ("lang", Value::Some(Box::new(Value::Str(Arc::from(lang))))),
        ("code", Value::Str(Arc::from(code))),
    ]))
}

fn bi_list(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let items = args[0].as_list()
        .ok_or_else(|| LxError::type_err("md.list expects List", span))?;
    Ok(node_rec("list", vec![("items", Value::List(items.clone()))]))
}

fn bi_ordered(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let items = args[0].as_list()
        .ok_or_else(|| LxError::type_err("md.ordered expects List", span))?;
    Ok(node_rec("ordered", vec![("items", Value::List(items.clone()))]))
}

fn bi_table(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let headers = args[0].as_list()
        .ok_or_else(|| LxError::type_err("md.table expects List headers", span))?;
    let rows = args[1].as_list()
        .ok_or_else(|| LxError::type_err("md.table expects List rows", span))?;
    Ok(node_rec("table", vec![
        ("headers", Value::List(headers.clone())),
        ("rows", Value::List(rows.clone())),
    ]))
}

fn bi_link(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let text = args[0].as_str()
        .ok_or_else(|| LxError::type_err("md.link expects Str text", span))?;
    let url = args[1].as_str()
        .ok_or_else(|| LxError::type_err("md.link expects Str url", span))?;
    Ok(node_rec("link", vec![
        ("text", Value::Str(Arc::from(text))),
        ("url", Value::Str(Arc::from(url))),
    ]))
}

fn bi_blockquote(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let text = args[0].as_str()
        .ok_or_else(|| LxError::type_err("md.blockquote expects Str", span))?;
    Ok(node_rec("blockquote", vec![("text", Value::Str(Arc::from(text)))]))
}

fn bi_raw(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let text = args[0].as_str()
        .ok_or_else(|| LxError::type_err("md.raw expects Str", span))?;
    Ok(node_rec("raw", vec![("text", Value::Str(Arc::from(text)))]))
}

fn bi_doc(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    args[0].as_list()
        .ok_or_else(|| LxError::type_err("md.doc expects List of nodes", span))?;
    Ok(args[0].clone())
}

fn field_str(rec: &IndexMap<String, Value>, field: &str) -> Option<String> {
    rec.get(field).and_then(|v| v.as_str()).map(|s| s.to_string())
}

pub fn bi_render(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let nodes = args[0].as_list()
        .ok_or_else(|| LxError::type_err("md.render expects List", span))?;
    let mut out = String::new();
    for node in nodes.iter() {
        let Value::Record(r) = node else { continue };
        let t = field_str(r, "type").unwrap_or_default();
        match t.as_str() {
            "heading" => {
                let lv: i64 = r.get("level").and_then(|v| v.as_int())
                    .and_then(|n| n.try_into().ok()).unwrap_or(1);
                let text = field_str(r, "text").unwrap_or_default();
                for _ in 0..lv { out.push('#'); }
                out.push(' ');
                out.push_str(&text);
                out.push_str("\n\n");
            }
            "para" => {
                out.push_str(&field_str(r, "text").unwrap_or_default());
                out.push_str("\n\n");
            }
            "code" => {
                let lang = match r.get("lang") {
                    Some(Value::Some(l)) => l.as_str().unwrap_or("").to_string(),
                    _ => String::new(),
                };
                out.push_str("```");
                out.push_str(&lang);
                out.push('\n');
                out.push_str(&field_str(r, "code").unwrap_or_default());
                out.push_str("\n```\n\n");
            }
            "list" => {
                if let Some(Value::List(items)) = r.get("items") {
                    for item in items.iter() { out.push_str(&format!("- {item}\n")); }
                    out.push('\n');
                }
            }
            "ordered" => {
                if let Some(Value::List(items)) = r.get("items") {
                    for (i, item) in items.iter().enumerate() {
                        out.push_str(&format!("{}. {item}\n", i + 1));
                    }
                    out.push('\n');
                }
            }
            "table" => render_table(r, &mut out),
            "blockquote" => {
                for line in field_str(r, "text").unwrap_or_default().lines() {
                    out.push_str(&format!("> {line}\n"));
                }
                out.push('\n');
            }
            "hr" => out.push_str("---\n\n"),
            "link" => {
                let text = field_str(r, "text").unwrap_or_default();
                let url = field_str(r, "url").unwrap_or_default();
                out.push_str(&format!("[{text}]({url})"));
            }
            "raw" => out.push_str(&field_str(r, "text").unwrap_or_default()),
            _ => {}
        }
    }
    Ok(Value::Str(Arc::from(out.trim_end())))
}

fn render_table(r: &IndexMap<String, Value>, out: &mut String) {
    let Some(Value::List(headers)) = r.get("headers") else { return };
    let Some(Value::List(rows)) = r.get("rows") else { return };
    out.push('|');
    for h in headers.iter() { out.push_str(&format!(" {h} |")); }
    out.push('\n');
    out.push('|');
    for _ in headers.iter() { out.push_str(" --- |"); }
    out.push('\n');
    for row in rows.iter() {
        if let Value::List(cells) = row {
            out.push('|');
            for c in cells.iter() { out.push_str(&format!(" {c} |")); }
            out.push('\n');
        }
    }
    out.push('\n');
}
