use std::sync::Arc;

use indexmap::IndexMap;

use crate::value::{FieldDef, Value};

fn mk_trait(name: &str, fields: Vec<FieldDef>) -> Value {
    Value::Trait {
        name: Arc::from(name),
        fields: Arc::new(fields),
        methods: Arc::new(Vec::new()),
        defaults: Arc::new(IndexMap::new()),
        requires: Arc::new(Vec::new()),
        description: None,
        tags: Arc::new(Vec::new()),
    }
}

fn mk_field(name: &str, type_name: &str) -> FieldDef {
    FieldDef {
        name: name.into(),
        type_name: type_name.into(),
        default: None,
        constraint: None,
    }
}

fn mk_field_default(name: &str, type_name: &str, default: Value) -> FieldDef {
    FieldDef {
        name: name.into(),
        type_name: type_name.into(),
        default: Some(default),
        constraint: None,
    }
}

fn mk_yield_approval() -> Value {
    mk_trait(
        "YieldApproval",
        vec![
            mk_field_default("kind", "Str", Value::Str(Arc::from("approval"))),
            mk_field("action", "Str"),
            mk_field("details", "Any"),
            mk_field_default("timeout_policy", "Str", Value::Str(Arc::from("block"))),
        ],
    )
}

fn mk_yield_reflection() -> Value {
    mk_trait(
        "YieldReflection",
        vec![
            mk_field_default("kind", "Str", Value::Str(Arc::from("reflection"))),
            mk_field("task", "Any"),
            mk_field("attempt", "Any"),
            mk_field("question", "Str"),
            mk_field_default("context", "Any", Value::None),
        ],
    )
}

fn mk_yield_information() -> Value {
    mk_trait(
        "YieldInformation",
        vec![
            mk_field_default("kind", "Str", Value::Str(Arc::from("information"))),
            mk_field("query", "Str"),
            mk_field_default("context", "Any", Value::None),
            mk_field_default("format", "Str", Value::Str(Arc::from("text"))),
        ],
    )
}

fn mk_yield_delegation() -> Value {
    mk_trait(
        "YieldDelegation",
        vec![
            mk_field_default("kind", "Str", Value::Str(Arc::from("delegation"))),
            mk_field("task", "Any"),
            mk_field_default("constraints", "Any", Value::None),
            mk_field_default("deadline", "Any", Value::None),
        ],
    )
}

fn mk_yield_progress() -> Value {
    mk_trait(
        "YieldProgress",
        vec![
            mk_field_default("kind", "Str", Value::Str(Arc::from("progress"))),
            mk_field("stage", "Str"),
            mk_field("pct", "Float"),
            mk_field_default("message", "Str", Value::Str(Arc::from(""))),
        ],
    )
}

pub fn build() -> IndexMap<String, Value> {
    let mut m = IndexMap::new();
    m.insert("YieldApproval".into(), mk_yield_approval());
    m.insert("YieldReflection".into(), mk_yield_reflection());
    m.insert("YieldInformation".into(), mk_yield_information());
    m.insert("YieldDelegation".into(), mk_yield_delegation());
    m.insert("YieldProgress".into(), mk_yield_progress());
    m
}
