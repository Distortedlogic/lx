use indexmap::IndexMap;
use reqwest::Client;
use reqwest::header::CONTENT_TYPE;

use crate::std_module;
use lx_span::sym::{Sym, intern};
use lx_value::BuiltinCtx;
use lx_value::LxError;
use lx_value::LxVal;
use lx_value::record;
use miette::SourceSpan;

#[derive(Debug, Clone, Default)]
struct HttpOpts {
  headers: Option<IndexMap<String, String>>,
  query: Option<IndexMap<String, String>>,
  body: Option<serde_json::Value>,
}

fn do_request(method: &str, url: &str, opts: &HttpOpts, span: SourceSpan, ctx: &dyn BuiltinCtx) -> Result<LxVal, LxError> {
  if ctx.network_denied() {
    return Ok(LxVal::err_str("network access denied by sandbox policy"));
  }
  tokio::task::block_in_place(|| {
    tokio::runtime::Handle::current().block_on(async {
      let c = Client::builder().build().map_err(|e| LxError::runtime(format!("http: client: {e}"), span))?;
      let mut builder = match method {
        "GET" => c.get(url),
        "POST" => c.post(url),
        "PUT" => c.put(url),
        "DELETE" => c.delete(url),
        _ => {
          return Err(LxError::runtime(format!("http: unknown method '{method}'"), span));
        },
      };
      if let Some(ref hdrs) = opts.headers {
        for (k, v) in hdrs {
          builder = builder.header(k.as_str(), v.as_str());
        }
      }
      if let Some(ref query) = opts.query {
        let pairs: Vec<(&str, &str)> = query.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
        builder = builder.query(&pairs);
      }
      if let Some(ref body) = opts.body {
        builder = builder.header(CONTENT_TYPE, "application/json").json(body);
      }
      match builder.send().await {
        Ok(resp) => response_to_value(resp, span).await,
        Err(e) => Ok(LxVal::err_str(e.to_string())),
      }
    })
  })
}

async fn response_to_value(resp: reqwest::Response, span: SourceSpan) -> Result<LxVal, LxError> {
  let status = resp.status().as_u16();
  let mut headers = IndexMap::new();
  for (name, value) in resp.headers() {
    let v = value.to_str().unwrap_or("").to_string();
    headers.insert(lx_span::sym::intern(name.as_str()), LxVal::str(v));
  }
  let body_str = resp.text().await.map_err(|e| LxError::runtime(format!("http: body: {e}"), span))?;
  let body = if let Ok(jv) = serde_json::from_str::<serde_json::Value>(&body_str) { LxVal::from(jv) } else { LxVal::str(body_str) };
  Ok(LxVal::ok(record! {
      "status" => LxVal::int(status),
      "body" => body,
      "headers" => LxVal::record(headers),
  }))
}

pub fn build() -> IndexMap<Sym, LxVal> {
  std_module! {
      "get"     => "http.get",     1, bi_get;
      "post"    => "http.post",    2, bi_post;
      "put"     => "http.put",     2, bi_put;
      "delete"  => "http.delete",  1, bi_delete;
      "request" => "http.request", 1, bi_request
  }
}

fn bi_get(args: &[LxVal], span: SourceSpan, ctx: &dyn BuiltinCtx) -> Result<LxVal, LxError> {
  let url = args[0].require_str("http.get", span)?;
  do_request("GET", url, &HttpOpts::default(), span, ctx)
}

fn bi_post(args: &[LxVal], span: SourceSpan, ctx: &dyn BuiltinCtx) -> Result<LxVal, LxError> {
  let url = args[0].require_str("http.post", span)?;
  let body: serde_json::Value = (&args[1]).into();
  let opts = HttpOpts { body: Some(body), ..Default::default() };
  do_request("POST", url, &opts, span, ctx)
}

fn bi_put(args: &[LxVal], span: SourceSpan, ctx: &dyn BuiltinCtx) -> Result<LxVal, LxError> {
  let url = args[0].require_str("http.put", span)?;
  let body: serde_json::Value = (&args[1]).into();
  let opts = HttpOpts { body: Some(body), ..Default::default() };
  do_request("PUT", url, &opts, span, ctx)
}

fn bi_delete(args: &[LxVal], span: SourceSpan, ctx: &dyn BuiltinCtx) -> Result<LxVal, LxError> {
  let url = args[0].require_str("http.delete", span)?;
  do_request("DELETE", url, &HttpOpts::default(), span, ctx)
}

fn bi_request(args: &[LxVal], span: SourceSpan, ctx: &dyn BuiltinCtx) -> Result<LxVal, LxError> {
  let rec = args[0].require_record("http.request", span)?;
  let method = rec.get(&intern("method")).and_then(|v| v.as_str()).unwrap_or("GET");
  let url = rec.get(&intern("url")).and_then(|v| v.as_str()).ok_or_else(|| LxError::type_err("http.request: 'url' field required", span, None))?;
  let body = rec.get(&intern("body")).map(|v| -> serde_json::Value { v.into() });
  let headers = extract_string_map(rec, "headers");
  let query = extract_string_map(rec, "query");
  let opts = HttpOpts { headers, query, body };
  do_request(method, url, &opts, span, ctx)
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
