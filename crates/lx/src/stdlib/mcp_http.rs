use reqwest::blocking::Client;
use reqwest::header::{ACCEPT, CONTENT_TYPE, HeaderValue};

use crate::error::LxError;
use crate::span::Span;

pub(super) struct HttpTransport {
    client: Client,
    url: String,
    session_id: Option<String>,
}

impl HttpTransport {
    pub(super) fn new(url: String, span: Span) -> Result<Self, LxError> {
        let client = Client::builder()
            .build()
            .map_err(|e| LxError::runtime(format!("mcp http: client: {e}"), span))?;
        Ok(HttpTransport {
            client,
            url,
            session_id: None,
        })
    }

    pub(super) fn send(
        &mut self,
        req: &serde_json::Value,
        span: Span,
    ) -> Result<serde_json::Value, LxError> {
        let resp = self.post(req, span)?;
        self.capture_session_id(&resp);
        let ct = content_type(&resp);
        let body = resp
            .text()
            .map_err(|e| LxError::runtime(format!("mcp http: body: {e}"), span))?;
        if ct.contains("text/event-stream") {
            parse_sse(&body, req, span)
        } else {
            serde_json::from_str(&body)
                .map_err(|e| LxError::runtime(format!("mcp http: decode: {e}"), span))
        }
    }

    pub(super) fn send_notify(
        &mut self,
        req: &serde_json::Value,
        span: Span,
    ) -> Result<(), LxError> {
        let resp = self.post(req, span)?;
        self.capture_session_id(&resp);
        let status = resp.status();
        if !status.is_success() {
            return Err(LxError::runtime(
                format!("mcp http: notify status {status}"),
                span,
            ));
        }
        Ok(())
    }

    pub(super) fn shutdown(self, span: Span) -> Result<(), LxError> {
        let Some(ref sid) = self.session_id else {
            return Ok(());
        };
        self.client
            .delete(&self.url)
            .header(
                "Mcp-Session-Id",
                HeaderValue::from_str(sid)
                    .map_err(|e| LxError::runtime(format!("mcp http: header: {e}"), span))?,
            )
            .send()
            .map_err(|e| LxError::runtime(format!("mcp http: shutdown: {e}"), span))?;
        Ok(())
    }

    fn post(
        &self,
        body: &serde_json::Value,
        span: Span,
    ) -> Result<reqwest::blocking::Response, LxError> {
        let mut builder = self
            .client
            .post(&self.url)
            .header(CONTENT_TYPE, "application/json")
            .header(ACCEPT, "application/json, text/event-stream")
            .json(body);
        if let Some(ref sid) = self.session_id {
            builder = builder.header(
                "Mcp-Session-Id",
                HeaderValue::from_str(sid)
                    .map_err(|e| LxError::runtime(format!("mcp http: header: {e}"), span))?,
            );
        }
        builder
            .send()
            .map_err(|e| LxError::runtime(format!("mcp http: send: {e}"), span))
    }

    fn capture_session_id(&mut self, resp: &reqwest::blocking::Response) {
        if let Some(sid) = resp
            .headers()
            .get("mcp-session-id")
            .and_then(|v| v.to_str().ok())
        {
            self.session_id = Some(sid.to_string());
        }
    }
}

fn content_type(resp: &reqwest::blocking::Response) -> String {
    resp.headers()
        .get(CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string()
}

fn parse_sse(
    body: &str,
    req: &serde_json::Value,
    span: Span,
) -> Result<serde_json::Value, LxError> {
    let request_id = req.get("id").and_then(|v| v.as_u64());
    let mut data_buf = String::new();
    for line in body.lines() {
        if line.is_empty() {
            if let Some(jv) = try_parse_event(&data_buf, request_id, span)? {
                return Ok(jv);
            }
            data_buf.clear();
            continue;
        }
        if let Some(data) = line.strip_prefix("data:") {
            let data = data.strip_prefix(' ').unwrap_or(data);
            if !data_buf.is_empty() {
                data_buf.push('\n');
            }
            data_buf.push_str(data);
        }
    }
    if let Some(jv) = try_parse_event(&data_buf, request_id, span)? {
        return Ok(jv);
    }
    Err(LxError::runtime("mcp http: no matching SSE response", span))
}

fn try_parse_event(
    data: &str,
    request_id: Option<u64>,
    span: Span,
) -> Result<Option<serde_json::Value>, LxError> {
    if data.is_empty() {
        return Ok(None);
    }
    let jv: serde_json::Value = serde_json::from_str(data)
        .map_err(|e| LxError::runtime(format!("mcp http: sse decode: {e}"), span))?;
    if jv.get("id").and_then(|v| v.as_u64()) == request_id {
        return Ok(Some(jv));
    }
    Ok(None)
}
