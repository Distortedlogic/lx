use std::sync::Arc;

use indexmap::IndexMap;

use crate::backends::{HttpOpts, RuntimeCtx};
use crate::builtins::mk;
use crate::error::LxError;
use crate::span::Span;
use crate::stdlib::json_conv::lx_to_json;
use crate::value::LxVal;

pub fn build() -> IndexMap<String, LxVal> {
    let mut m = IndexMap::new();
    m.insert("get".into(), mk("http.get", 1, bi_get));
    m.insert("post".into(), mk("http.post", 2, bi_post));
    m.insert("put".into(), mk("http.put", 2, bi_put));
    m.insert("delete".into(), mk("http.delete", 1, bi_delete));
    m
}

fn extract_opts(val: &LxVal, span: Span) -> Result<HttpOpts, LxError> {
    let LxVal::Record(fields) = val else {
        return Ok(HttpOpts::default());
    };
    let headers = fields.get("headers").and_then(|v| {
        if let LxVal::Record(hdr_fields) = v {
            let map: IndexMap<String, String> = hdr_fields
                .iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                .collect();
            if map.is_empty() { None } else { Some(map) }
        } else {
            None
        }
    });
    let query = fields.get("query").and_then(|v| {
        if let LxVal::Record(q_fields) = v {
            let map: IndexMap<String, String> = q_fields
                .iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                .collect();
            if map.is_empty() { None } else { Some(map) }
        } else {
            None
        }
    });
    let body = fields
        .get("body")
        .map(|v| lx_to_json(v, span))
        .transpose()?;
    Ok(HttpOpts {
        headers,
        query,
        body,
    })
}

fn bi_get(args: &[LxVal], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
    let (url, opts) = match &args[0] {
        LxVal::Str(s) => (s.to_string(), HttpOpts::default()),
        LxVal::Record(fields) => {
            let url = fields
                .get("url")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    LxError::type_err("http.get: record must have Str 'url' field", span)
                })?
                .to_string();
            let opts = extract_opts(&args[0], span)?;
            (url, opts)
        }
        _ => {
            return Err(LxError::type_err(
                "http.get expects Str url or Record {url headers query}",
                span,
            ));
        }
    };
    ctx.http.request("GET", &url, &opts, span)
}

fn bi_post(args: &[LxVal], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
    let url = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("http.post expects Str url as first arg", span))?;
    let body = lx_to_json(&args[1], span)?;
    let opts = HttpOpts {
        body: Some(body),
        ..Default::default()
    };
    ctx.http.request("POST", url, &opts, span)
}

fn bi_put(args: &[LxVal], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
    let url = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("http.put expects Str url as first arg", span))?;
    let body = lx_to_json(&args[1], span)?;
    let opts = HttpOpts {
        body: Some(body),
        ..Default::default()
    };
    ctx.http.request("PUT", url, &opts, span)
}

fn bi_delete(args: &[LxVal], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
    let url = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("http.delete expects Str url", span))?;
    ctx.http.request("DELETE", url, &HttpOpts::default(), span)
}
