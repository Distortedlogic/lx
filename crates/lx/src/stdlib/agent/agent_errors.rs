use std::sync::Arc;

use num_bigint::BigInt;

use crate::record;
use crate::value::Value;

pub fn timeout(elapsed_ms: i64, deadline_ms: i64) -> Value {
    Value::Tagged {
        tag: Arc::from("Timeout"),
        values: Arc::new(vec![record! {
            "elapsed_ms" => Value::Int(BigInt::from(elapsed_ms)),
            "deadline_ms" => Value::Int(BigInt::from(deadline_ms)),
        }]),
    }
}

pub fn rate_limited(retry_after_ms: i64, limit: &str) -> Value {
    Value::Tagged {
        tag: Arc::from("RateLimited"),
        values: Arc::new(vec![record! {
            "retry_after_ms" => Value::Int(BigInt::from(retry_after_ms)),
            "limit" => Value::Str(Arc::from(limit)),
        }]),
    }
}

pub fn budget_exhausted(used: f64, limit: f64, resource: &str) -> Value {
    Value::Tagged {
        tag: Arc::from("BudgetExhausted"),
        values: Arc::new(vec![record! {
            "used" => Value::Float(used),
            "limit" => Value::Float(limit),
            "resource" => Value::Str(Arc::from(resource)),
        }]),
    }
}

pub fn context_overflow(size: i64, capacity: i64, content: &str) -> Value {
    Value::Tagged {
        tag: Arc::from("ContextOverflow"),
        values: Arc::new(vec![record! {
            "size" => Value::Int(BigInt::from(size)),
            "capacity" => Value::Int(BigInt::from(capacity)),
            "content" => Value::Str(Arc::from(content)),
        }]),
    }
}

pub fn upstream(service: &str, code: i64, message: &str) -> Value {
    Value::Tagged {
        tag: Arc::from("Upstream"),
        values: Arc::new(vec![record! {
            "service" => Value::Str(Arc::from(service)),
            "code" => Value::Int(BigInt::from(code)),
            "message" => Value::Str(Arc::from(message)),
        }]),
    }
}

pub fn unavailable(agent: &str, reason: &str) -> Value {
    Value::Tagged {
        tag: Arc::from("Unavailable"),
        values: Arc::new(vec![record! {
            "agent" => Value::Str(Arc::from(agent)),
            "reason" => Value::Str(Arc::from(reason)),
        }]),
    }
}

pub fn internal(message: &str) -> Value {
    Value::Tagged {
        tag: Arc::from("Internal"),
        values: Arc::new(vec![record! {
            "message" => Value::Str(Arc::from(message)),
        }]),
    }
}

fn ctor(tag: &str) -> Value {
    Value::TaggedCtor {
        tag: Arc::from(tag),
        arity: 1,
        applied: vec![],
    }
}

pub fn tagged_ctors() -> Vec<(&'static str, Value)> {
    vec![
        ("Timeout", ctor("Timeout")),
        ("RateLimited", ctor("RateLimited")),
        ("BudgetExhausted", ctor("BudgetExhausted")),
        ("ContextOverflow", ctor("ContextOverflow")),
        ("Incompetent", ctor("Incompetent")),
        ("Upstream", ctor("Upstream")),
        ("PermissionDenied", ctor("PermissionDenied")),
        ("TraitViolation", ctor("TraitViolation")),
        ("Unavailable", ctor("Unavailable")),
        ("Cancelled", ctor("Cancelled")),
        ("Internal", ctor("Internal")),
    ]
}
