use std::sync::Arc;

use indexmap::IndexMap;

use crate::error::LxError;
use crate::runtime::{HttpOpts, RuntimeCtx};
use crate::std_module;
use crate::sym::{Sym, intern};
use crate::value::LxVal;
use miette::SourceSpan;

pub fn build() -> IndexMap<Sym, LxVal> {
  std_module! {
    "get"     => "http.get",     1, bi_get;
    "post"    => "http.post",    2, bi_post;
    "put"     => "http.put",     2, bi_put;
    "delete"  => "http.delete",  1, bi_delete;
    "request" => "http.request", 1, bi_request
  }
}

fn bi_get(args: &[LxVal], span: SourceSpan, ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let url = args[0].require_str("http.get", span)?;
  ctx.http.request("GET", url, &HttpOpts::default(), span)
}

fn bi_post(args: &[LxVal], span: SourceSpan, ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let url = args[0].require_str("http.post", span)?;
  let body: serde_json::Value = (&args[1]).into();
  let opts = HttpOpts { body: Some(body), ..Default::default() };
  ctx.http.request("POST", url, &opts, span)
}

fn bi_put(args: &[LxVal], span: SourceSpan, ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let url = args[0].require_str("http.put", span)?;
  let body: serde_json::Value = (&args[1]).into();
  let opts = HttpOpts { body: Some(body), ..Default::default() };
  ctx.http.request("PUT", url, &opts, span)
}

fn bi_delete(args: &[LxVal], span: SourceSpan, ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let url = args[0].require_str("http.delete", span)?;
  ctx.http.request("DELETE", url, &HttpOpts::default(), span)
}

fn bi_request(args: &[LxVal], span: SourceSpan, ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let rec = args[0].require_record("http.request", span)?;
  let method = rec.get(&intern("method")).and_then(|v| v.as_str()).unwrap_or("GET");
  let url = rec.get(&intern("url")).and_then(|v| v.as_str()).ok_or_else(|| LxError::type_err("http.request: 'url' field required", span, None))?;
  let body = rec.get(&intern("body")).map(|v| -> serde_json::Value { v.into() });
  let headers = extract_string_map(rec, "headers");
  let query = extract_string_map(rec, "query");
  let opts = HttpOpts { headers, query, body };
  ctx.http.request(method, url, &opts, span)
}

fn extract_string_map(rec: &IndexMap<Sym, LxVal>, key: &str) -> Option<IndexMap<String, String>> {
  let LxVal::Record(inner) = rec.get(&intern(key))? else {
    return None;
  };
  let mut map = IndexMap::new();
  for (k, v) in inner.as_ref() {
    if let Some(s) = v.as_str() {
      map.insert(k.as_str().to_string(), s.to_string());
    }
  }
  Some(map)
}
