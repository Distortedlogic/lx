use std::sync::Arc;

use crate::backends::RuntimeCtx;
use crate::builtins::mk;
use crate::error::LxError;
use crate::span::Span;
use crate::value::{ProtoFieldDef, Value};

pub fn mk_handoff_protocol() -> Value {
    let fields = vec![
        ProtoFieldDef {
            name: "result".into(),
            type_name: "Any".into(),
            default: None,
            constraint: None,
        },
        proto_list_field("tried"),
        proto_list_field("assumptions"),
        proto_list_field("uncertainties"),
        proto_list_field("recommendations"),
        proto_list_field("files_read"),
        proto_list_field("tools_used"),
        ProtoFieldDef {
            name: "duration_ms".into(),
            type_name: "Int".into(),
            default: Some(Value::Int(0.into())),
            constraint: None,
        },
    ];
    Value::Protocol {
        name: Arc::from("Handoff"),
        fields: Arc::new(fields),
    }
}

fn proto_list_field(name: &str) -> ProtoFieldDef {
    ProtoFieldDef {
        name: name.into(),
        type_name: "List".into(),
        default: Some(Value::List(Arc::new(vec![]))),
        constraint: None,
    }
}

pub fn mk_as_context() -> Value {
    mk("agent.as_context", 1, bi_as_context)
}

fn bi_as_context(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let Value::Record(r) = &args[0] else {
        return Err(LxError::type_err(
            "agent.as_context: expected a Handoff Record",
            span,
        ));
    };
    let mut out = String::from("## Previous Agent Handoff\n");
    if let Some(result) = r.get("result") {
        out.push_str(&format!("**Result:** {result}\n"));
    }
    format_list_section(&mut out, r, "tried", "Tried");
    format_list_section(&mut out, r, "assumptions", "Assumptions");
    format_list_section(&mut out, r, "uncertainties", "Uncertainties");
    format_list_section(&mut out, r, "recommendations", "Recommendations");
    format_list_section(&mut out, r, "files_read", "Files examined");
    format_list_section(&mut out, r, "tools_used", "Tools used");
    Ok(Value::Str(Arc::from(out.trim_end())))
}

fn format_list_section(
    out: &mut String,
    r: &indexmap::IndexMap<String, Value>,
    field: &str,
    label: &str,
) {
    if let Some(Value::List(items)) = r.get(field)
        && !items.is_empty()
    {
        out.push_str(&format!("**{label}:**\n"));
        for item in items.as_ref() {
            out.push_str(&format!("- {item}\n"));
        }
    }
}
