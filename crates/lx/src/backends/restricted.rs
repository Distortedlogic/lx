use std::sync::Arc;

use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

use super::{
    AiBackend, AiOpts, EmbedBackend, EmbedOpts, HttpBackend, HttpOpts, PaneBackend, ShellBackend,
};

pub struct DenyShellBackend;

impl ShellBackend for DenyShellBackend {
    fn exec(&self, _cmd: &str, _span: Span) -> Result<Value, LxError> {
        Ok(Value::Err(Box::new(Value::Str(Arc::from(
            "shell access denied by sandbox policy",
        )))))
    }

    fn exec_capture(&self, _cmd: &str, _span: Span) -> Result<Value, LxError> {
        Ok(Value::Err(Box::new(Value::Str(Arc::from(
            "shell access denied by sandbox policy",
        )))))
    }
}

pub struct DenyHttpBackend;

impl HttpBackend for DenyHttpBackend {
    fn request(
        &self,
        _method: &str,
        _url: &str,
        _opts: &HttpOpts,
        _span: Span,
    ) -> Result<Value, LxError> {
        Ok(Value::Err(Box::new(Value::Str(Arc::from(
            "network access denied by sandbox policy",
        )))))
    }
}

pub struct DenyAiBackend;

impl AiBackend for DenyAiBackend {
    fn prompt(&self, _text: &str, _opts: &AiOpts, _span: Span) -> Result<Value, LxError> {
        Ok(Value::Err(Box::new(Value::Str(Arc::from(
            "AI access denied by sandbox policy",
        )))))
    }
}

pub struct DenyPaneBackend;

impl PaneBackend for DenyPaneBackend {
    fn open(&self, _kind: &str, _config: &Value, _span: Span) -> Result<Value, LxError> {
        Ok(Value::Err(Box::new(Value::Str(Arc::from(
            "pane access denied by sandbox policy",
        )))))
    }

    fn update(&self, _pane_id: &str, _content: &Value, span: Span) -> Result<(), LxError> {
        Err(LxError::runtime(
            "pane access denied by sandbox policy",
            span,
        ))
    }

    fn close(&self, _pane_id: &str, span: Span) -> Result<(), LxError> {
        Err(LxError::runtime(
            "pane access denied by sandbox policy",
            span,
        ))
    }

    fn list(&self, _span: Span) -> Result<Value, LxError> {
        Ok(Value::Err(Box::new(Value::Str(Arc::from(
            "pane access denied by sandbox policy",
        )))))
    }
}

pub struct DenyEmbedBackend;

impl EmbedBackend for DenyEmbedBackend {
    fn embed(&self, _texts: &[String], _opts: &EmbedOpts, _span: Span) -> Result<Value, LxError> {
        Ok(Value::Err(Box::new(Value::Str(Arc::from(
            "embedding access denied by sandbox policy",
        )))))
    }
}

pub struct RestrictedShellBackend {
    pub inner: Arc<dyn ShellBackend>,
    pub allowed_cmds: Vec<String>,
}

impl ShellBackend for RestrictedShellBackend {
    fn exec(&self, cmd: &str, span: Span) -> Result<Value, LxError> {
        let first_word = cmd.split_whitespace().next().unwrap_or("");
        if self.allowed_cmds.iter().any(|c| c == first_word) {
            self.inner.exec(cmd, span)
        } else {
            Ok(Value::Err(Box::new(Value::Str(Arc::from(format!(
                "command '{first_word}' not allowed by sandbox policy"
            ))))))
        }
    }

    fn exec_capture(&self, cmd: &str, span: Span) -> Result<Value, LxError> {
        let first_word = cmd.split_whitespace().next().unwrap_or("");
        if self.allowed_cmds.iter().any(|c| c == first_word) {
            self.inner.exec_capture(cmd, span)
        } else {
            Ok(Value::Err(Box::new(Value::Str(Arc::from(format!(
                "command '{first_word}' not allowed by sandbox policy"
            ))))))
        }
    }
}
