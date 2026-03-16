use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

use crate::span::Span;

pub type LxResult<T> = Result<T, LxError>;

#[derive(Debug, Clone, Error, Diagnostic)]
pub enum LxError {
    #[error("parse error: {msg}")]
    #[diagnostic(code(lx::parse))]
    Parse {
        msg: String,
        #[label("{msg}")]
        span: SourceSpan,
        #[help]
        help: Option<String>,
    },

    #[error("runtime error: {msg}")]
    #[diagnostic(code(lx::runtime))]
    Runtime {
        msg: String,
        #[label("{msg}")]
        span: SourceSpan,
    },

    #[error("assertion failed: {expr}")]
    #[diagnostic(code(lx::assert))]
    Assert {
        expr: String,
        message: Option<String>,
        #[label("assertion failed")]
        span: SourceSpan,
    },

    #[error("type error: {msg}")]
    #[diagnostic(code(lx::type_error))]
    Type {
        msg: String,
        #[label("{msg}")]
        span: SourceSpan,
    },

    #[error("break")]
    BreakSignal { value: Box<crate::value::Value> },

    #[error("propagated error: {value}")]
    #[diagnostic(code(lx::propagate))]
    Propagate {
        value: Box<crate::value::Value>,
        #[label("error propagated here")]
        span: SourceSpan,
    },

    #[error("{inner}")]
    Sourced {
        source_name: String,
        source_text: std::sync::Arc<str>,
        inner: Box<LxError>,
    },
}

impl LxError {
    pub fn parse(msg: impl Into<String>, span: Span, help: Option<String>) -> Self {
        Self::Parse {
            msg: msg.into(),
            span: span.into(),
            help,
        }
    }

    pub fn runtime(msg: impl Into<String>, span: Span) -> Self {
        Self::Runtime {
            msg: msg.into(),
            span: span.into(),
        }
    }

    pub fn assert_fail(expr: impl Into<String>, message: Option<String>, span: Span) -> Self {
        Self::Assert {
            expr: expr.into(),
            message,
            span: span.into(),
        }
    }

    pub fn type_err(msg: impl Into<String>, span: Span) -> Self {
        Self::Type {
            msg: msg.into(),
            span: span.into(),
        }
    }

    pub fn division_by_zero(span: Span) -> Self {
        Self::runtime("division by zero", span)
    }

    pub fn break_signal(value: crate::value::Value) -> Self {
        Self::BreakSignal {
            value: Box::new(value),
        }
    }

    pub fn propagate(value: crate::value::Value, span: Span) -> Self {
        Self::Propagate {
            value: Box::new(value),
            span: span.into(),
        }
    }

    pub fn with_source(self, name: String, text: std::sync::Arc<str>) -> Self {
        Self::Sourced {
            source_name: name,
            source_text: text,
            inner: Box::new(self),
        }
    }
}
