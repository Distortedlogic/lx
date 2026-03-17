mod describe_helpers;
mod describe_render;
mod describe_visitor;

use std::sync::Arc;

use indexmap::IndexMap;

use crate::backends::RuntimeCtx;
use crate::builtins::mk;
use crate::error::LxError;
use crate::record;
use crate::span::Span;
use crate::value::Value;
use crate::visitor::AstVisitor;

use describe_visitor::{Describer, ProgramDescription};

pub fn build() -> IndexMap<String, Value> {
    let mut m = IndexMap::new();
    m.insert("extract".into(), mk("describe.extract", 1, bi_extract));
    m.insert(
        "extract_file".into(),
        mk("describe.extract_file", 1, bi_extract_file),
    );
    m.insert("render".into(), mk("describe.render", 1, bi_render));
    m
}

fn bi_extract(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let src = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("describe.extract expects Str", span))?;
    let desc = extract_description(src, span)?;
    Ok(description_to_value(&desc))
}

fn bi_extract_file(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let path = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("describe.extract_file expects Str", span))?;
    let src = std::fs::read_to_string(path)
        .map_err(|e| LxError::runtime(format!("describe.extract_file: {e}"), span))?;
    let desc = extract_description(&src, span)?;
    Ok(description_to_value(&desc))
}

fn bi_render(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let desc = value_to_description(&args[0], span)?;
    let text = describe_render::render_description(&desc);
    Ok(Value::Str(Arc::from(text.as_str())))
}

fn extract_description(src: &str, span: Span) -> Result<ProgramDescription, LxError> {
    let tokens = crate::lexer::lex(src)
        .map_err(|e| LxError::runtime(format!("describe: lex error: {e}"), span))?;
    let program = crate::parser::parse(tokens)
        .map_err(|e| LxError::runtime(format!("describe: parse error: {e}"), span))?;
    let mut describer = Describer::new();
    describer.visit_program(&program);
    Ok(describer.into_description())
}

fn description_to_value(desc: &ProgramDescription) -> Value {
    let imports: Vec<Value> = desc
        .imports
        .iter()
        .map(|i| {
            record! {
                "path" => Value::Str(Arc::from(i.path.as_str())),
                "kind" => Value::Str(Arc::from(i.kind.as_str())),
            }
        })
        .collect();

    let agents: Vec<Value> = desc
        .agents
        .iter()
        .map(|a| {
            let traits_v: Vec<Value> = a
                .traits
                .iter()
                .map(|t| Value::Str(Arc::from(t.as_str())))
                .collect();
            let methods_v: Vec<Value> = a
                .methods
                .iter()
                .map(|m| Value::Str(Arc::from(m.as_str())))
                .collect();
            record! {
                "name" => Value::Str(Arc::from(a.name.as_str())),
                "traits" => Value::List(Arc::new(traits_v)),
                "methods" => Value::List(Arc::new(methods_v)),
                "declared" => Value::Bool(a.declared),
                "spawned_by" => Value::Str(Arc::from(a.spawned_by.as_str())),
            }
        })
        .collect();

    let messages: Vec<Value> = desc
        .messages
        .iter()
        .map(|m| {
            record! {
                "from" => Value::Str(Arc::from(m.from.as_str())),
                "to" => Value::Str(Arc::from(m.to.as_str())),
                "style" => Value::Str(Arc::from(m.style.as_str())),
                "label" => Value::Str(Arc::from(m.label.as_str())),
            }
        })
        .collect();

    let control_flow: Vec<Value> = desc
        .control_flow
        .iter()
        .map(|c| {
            record! {
                "kind" => Value::Str(Arc::from(c.kind.as_str())),
                "label" => Value::Str(Arc::from(c.label.as_str())),
            }
        })
        .collect();

    let resources: Vec<Value> = desc
        .resources
        .iter()
        .map(|r| {
            record! {
                "kind" => Value::Str(Arc::from(r.kind.as_str())),
                "name" => Value::Str(Arc::from(r.name.as_str())),
                "source" => Value::Str(Arc::from(r.source.as_str())),
            }
        })
        .collect();

    let ai_calls: Vec<Value> = desc
        .ai_calls
        .iter()
        .map(|a| {
            record! {
                "context" => Value::Str(Arc::from(a.context.as_str())),
            }
        })
        .collect();

    let exports: Vec<Value> = desc
        .exports
        .iter()
        .map(|e| Value::Str(Arc::from(e.as_str())))
        .collect();

    record! {
        "imports" => Value::List(Arc::new(imports)),
        "agents" => Value::List(Arc::new(agents)),
        "messages" => Value::List(Arc::new(messages)),
        "control_flow" => Value::List(Arc::new(control_flow)),
        "resources" => Value::List(Arc::new(resources)),
        "ai_calls" => Value::List(Arc::new(ai_calls)),
        "exports" => Value::List(Arc::new(exports)),
    }
}

fn value_to_description(val: &Value, span: Span) -> Result<ProgramDescription, LxError> {
    let Value::Record(rec) = val else {
        return Err(LxError::type_err("describe.render expects Record", span));
    };
    let imports = extract_list_field(rec, "imports", span, |v, s| {
        Ok(describe_visitor::ImportInfo {
            path: str_field(v, "path", s)?,
            kind: str_field(v, "kind", s)?,
        })
    })?;
    let agents = extract_list_field(rec, "agents", span, |v, s| {
        Ok(describe_visitor::AgentInfo {
            name: str_field(v, "name", s)?,
            traits: str_list_field(v, "traits", s)?,
            methods: str_list_field(v, "methods", s)?,
            declared: bool_field(v, "declared", s)?,
            spawned_by: str_field(v, "spawned_by", s)?,
        })
    })?;
    let messages = extract_list_field(rec, "messages", span, |v, s| {
        Ok(describe_visitor::MessageInfo {
            from: str_field(v, "from", s)?,
            to: str_field(v, "to", s)?,
            style: str_field(v, "style", s)?,
            label: str_field(v, "label", s)?,
        })
    })?;
    let control_flow = extract_list_field(rec, "control_flow", span, |v, s| {
        Ok(describe_visitor::ControlFlowInfo {
            kind: str_field(v, "kind", s)?,
            label: str_field(v, "label", s)?,
        })
    })?;
    let resources = extract_list_field(rec, "resources", span, |v, s| {
        Ok(describe_visitor::ResourceInfo {
            kind: str_field(v, "kind", s)?,
            name: str_field(v, "name", s)?,
            source: str_field(v, "source", s)?,
        })
    })?;
    let ai_calls = extract_list_field(rec, "ai_calls", span, |v, s| {
        Ok(describe_visitor::AiCallInfo {
            context: str_field(v, "context", s)?,
        })
    })?;
    let exports = str_list_from_val(
        rec.get("exports")
            .ok_or_else(|| LxError::type_err("missing 'exports'", span))?,
        span,
    )?;
    Ok(ProgramDescription {
        imports,
        agents,
        messages,
        control_flow,
        resources,
        ai_calls,
        exports,
    })
}

fn str_field(val: &Value, key: &str, span: Span) -> Result<String, LxError> {
    val.str_field(key)
        .map(String::from)
        .ok_or_else(|| LxError::type_err(format!("missing '{key}'"), span))
}

fn bool_field(val: &Value, key: &str, span: Span) -> Result<bool, LxError> {
    val.bool_field(key)
        .ok_or_else(|| LxError::type_err(format!("missing '{key}'"), span))
}

fn str_list_field(val: &Value, key: &str, span: Span) -> Result<Vec<String>, LxError> {
    let list = val
        .list_field(key)
        .ok_or_else(|| LxError::type_err(format!("missing '{key}'"), span))?;
    str_list_from_slice(list, span)
}

fn str_list_from_val(val: &Value, span: Span) -> Result<Vec<String>, LxError> {
    let Value::List(l) = val else {
        return Err(LxError::type_err("expected List", span));
    };
    str_list_from_slice(l, span)
}

fn str_list_from_slice(list: &[Value], span: Span) -> Result<Vec<String>, LxError> {
    list.iter()
        .map(|v| {
            v.as_str()
                .map(String::from)
                .ok_or_else(|| LxError::type_err("expected Str in list", span))
        })
        .collect()
}

fn extract_list_field<T>(
    rec: &IndexMap<String, Value>,
    key: &str,
    span: Span,
    f: impl Fn(&Value, Span) -> Result<T, LxError>,
) -> Result<Vec<T>, LxError> {
    let val = rec
        .get(key)
        .ok_or_else(|| LxError::type_err(format!("missing '{key}'"), span))?;
    let Value::List(l) = val else {
        return Err(LxError::type_err(format!("'{key}' must be List"), span));
    };
    l.iter().map(|v| f(v, span)).collect()
}
