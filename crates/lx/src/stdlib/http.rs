use std::sync::Arc;

use indexmap::IndexMap;
use num_bigint::BigInt;
use reqwest::blocking::Client;
use reqwest::header::CONTENT_TYPE;

use crate::builtins::mk;
use crate::error::LxError;
use crate::span::Span;
use crate::stdlib::json_conv::{json_to_lx, lx_to_json};
use crate::value::Value;

pub fn build() -> IndexMap<String, Value> {
    let mut m = IndexMap::new();
    m.insert("get".into(), mk("http.get", 1, bi_get));
    m.insert("post".into(), mk("http.post", 2, bi_post));
    m.insert("put".into(), mk("http.put", 2, bi_put));
    m.insert("delete".into(), mk("http.delete", 1, bi_delete));
    m
}

fn client(span: Span) -> Result<Client, LxError> {
    Client::builder()
        .build()
        .map_err(|e| LxError::runtime(format!("http: client: {e}"), span))
}

fn response_to_value(
    resp: reqwest::blocking::Response,
    span: Span,
) -> Result<Value, LxError> {
    let status = resp.status().as_u16();
    let mut headers = IndexMap::new();
    for (name, value) in resp.headers() {
        let v = value.to_str().unwrap_or("").to_string();
        headers.insert(name.to_string(), Value::Str(Arc::from(v.as_str())));
    }
    let body_str = resp.text()
        .map_err(|e| LxError::runtime(format!("http: body: {e}"), span))?;
    let body = if let Ok(jv) = serde_json::from_str::<serde_json::Value>(&body_str) {
        json_to_lx(jv)
    } else {
        Value::Str(Arc::from(body_str.as_str()))
    };
    let mut fields = IndexMap::new();
    fields.insert("status".into(), Value::Int(BigInt::from(status)));
    fields.insert("body".into(), body);
    fields.insert("headers".into(), Value::Record(Arc::new(headers)));
    Ok(Value::Ok(Box::new(Value::Record(Arc::new(fields)))))
}

fn extract_opts(val: &Value) -> (Option<&Value>, Option<&Value>) {
    match val {
        Value::Record(fields) => {
            let headers = fields.get("headers");
            let query = fields.get("query");
            (headers, query)
        }
        _ => (None, None),
    }
}

fn apply_headers(
    mut builder: reqwest::blocking::RequestBuilder,
    headers_val: Option<&Value>,
    span: Span,
) -> Result<reqwest::blocking::RequestBuilder, LxError> {
    if let Some(Value::Record(hdr_fields)) = headers_val {
        for (k, v) in hdr_fields.iter() {
            let val_str = v.as_str()
                .ok_or_else(|| LxError::type_err("http: header value must be Str", span))?;
            builder = builder.header(k.as_str(), val_str);
        }
    }
    Ok(builder)
}

fn apply_query(
    mut builder: reqwest::blocking::RequestBuilder,
    query_val: Option<&Value>,
    span: Span,
) -> Result<reqwest::blocking::RequestBuilder, LxError> {
    if let Some(Value::Record(q_fields)) = query_val {
        let pairs: Vec<(String, String)> = q_fields.iter()
            .map(|(k, v)| {
                let val_str = v.as_str()
                    .ok_or_else(|| LxError::type_err("http: query value must be Str", span));
                val_str.map(|s| (k.clone(), s.to_string()))
            })
            .collect::<Result<_, _>>()?;
        builder = builder.query(&pairs);
    }
    Ok(builder)
}

fn bi_get(args: &[Value], span: Span) -> Result<Value, LxError> {
    let (url, opts) = match &args[0] {
        Value::Str(s) => (s.to_string(), None),
        Value::Record(fields) => {
            let url = fields.get("url")
                .and_then(|v| v.as_str())
                .ok_or_else(|| LxError::type_err("http.get: record must have Str 'url' field", span))?
                .to_string();
            (url, Some(&args[0]))
        }
        _ => return Err(LxError::type_err("http.get expects Str url or Record {url headers query}", span)),
    };
    let c = client(span)?;
    let mut builder = c.get(&url);
    if let Some(o) = opts {
        let (hdrs, query) = extract_opts(o);
        builder = apply_headers(builder, hdrs, span)?;
        builder = apply_query(builder, query, span)?;
    }
    match builder.send() {
        Ok(resp) => response_to_value(resp, span),
        Err(e) => Ok(Value::Err(Box::new(Value::Str(Arc::from(e.to_string().as_str()))))),
    }
}

fn send_with_body(
    method: &str,
    args: &[Value],
    span: Span,
) -> Result<Value, LxError> {
    let url = args[0].as_str()
        .ok_or_else(|| LxError::type_err(
            format!("http.{method} expects Str url as first arg"), span,
        ))?;
    let json_body = lx_to_json(&args[1], span)?;
    let c = client(span)?;
    let builder = if method == "post" {
        c.post(url)
    } else {
        c.put(url)
    };
    let builder = builder
        .header(CONTENT_TYPE, "application/json")
        .json(&json_body);
    match builder.send() {
        Ok(resp) => response_to_value(resp, span),
        Err(e) => Ok(Value::Err(Box::new(Value::Str(Arc::from(e.to_string().as_str()))))),
    }
}

fn bi_post(args: &[Value], span: Span) -> Result<Value, LxError> {
    send_with_body("post", args, span)
}

fn bi_put(args: &[Value], span: Span) -> Result<Value, LxError> {
    send_with_body("put", args, span)
}

fn bi_delete(args: &[Value], span: Span) -> Result<Value, LxError> {
    let url = args[0].as_str()
        .ok_or_else(|| LxError::type_err("http.delete expects Str url", span))?;
    let c = client(span)?;
    match c.delete(url).send() {
        Ok(resp) => response_to_value(resp, span),
        Err(e) => Ok(Value::Err(Box::new(Value::Str(Arc::from(e.to_string().as_str()))))),
    }
}
