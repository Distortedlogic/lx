use std::sync::Arc;

use num_bigint::BigInt;
use pulldown_cmark::{Event, Parser, Tag, TagEnd};

use crate::backends::RuntimeCtx;
use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

use super::{field_str, get_nodes, node_rec, nodes_by_type};

pub(super) fn bi_sections(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let nodes = get_nodes(&args[0], span)?;
    let mut sections = Vec::new();
    let mut cur: Option<(i64, String, String)> = None;
    for node in nodes {
        if let Value::Record(r) = node {
            if field_str(r, "type").as_deref() == Some("heading") {
                if let Some((lv, title, content)) = cur.take() {
                    sections.push(node_rec(
                        "section",
                        vec![
                            ("level", Value::Int(BigInt::from(lv))),
                            ("title", Value::Str(Arc::from(title.as_str()))),
                            ("content", Value::Str(Arc::from(content.trim()))),
                        ],
                    ));
                }
                let lv: i64 = r
                    .get("level")
                    .and_then(|v| v.as_int())
                    .and_then(|n| n.try_into().ok())
                    .unwrap_or(1);
                let title = field_str(r, "text").unwrap_or_default();
                cur = Some((lv, title, String::new()));
            } else if let Some((_, _, ref mut content)) = cur
                && let Some(t) = field_str(r, "text")
            {
                if !content.is_empty() {
                    content.push_str("\n\n");
                }
                content.push_str(&t);
            }
        }
    }
    if let Some((lv, title, content)) = cur {
        sections.push(node_rec(
            "section",
            vec![
                ("level", Value::Int(BigInt::from(lv))),
                ("title", Value::Str(Arc::from(title.as_str()))),
                ("content", Value::Str(Arc::from(content.trim()))),
            ],
        ));
    }
    Ok(Value::List(Arc::new(sections)))
}

pub(super) fn bi_code_blocks(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    Ok(Value::List(Arc::new(nodes_by_type(
        get_nodes(&args[0], span)?,
        "code",
    ))))
}

pub(super) fn bi_headings(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    Ok(Value::List(Arc::new(nodes_by_type(
        get_nodes(&args[0], span)?,
        "heading",
    ))))
}

pub(super) fn bi_links(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let input = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("md.links expects Str (markdown source)", span))?;
    let parser = Parser::new(input);
    let mut links = Vec::new();
    let mut in_link = false;
    let mut link_url = String::new();
    let mut link_text = String::new();
    for event in parser {
        match event {
            Event::Start(Tag::Link { dest_url, .. }) => {
                in_link = true;
                link_url = dest_url.to_string();
                link_text.clear();
            }
            Event::End(TagEnd::Link) => {
                links.push(node_rec(
                    "link",
                    vec![
                        ("text", Value::Str(Arc::from(link_text.as_str()))),
                        ("url", Value::Str(Arc::from(link_url.as_str()))),
                    ],
                ));
                in_link = false;
            }
            Event::Text(t) if in_link => link_text.push_str(&t),
            _ => {}
        }
    }
    Ok(Value::List(Arc::new(links)))
}

pub(super) fn bi_to_text(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let input = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("md.to_text expects Str (markdown source)", span))?;
    let parser = Parser::new(input);
    let mut out = String::new();
    for event in parser {
        match event {
            Event::Text(t) => out.push_str(&t),
            Event::Code(c) => out.push_str(&c),
            Event::SoftBreak | Event::HardBreak => out.push('\n'),
            Event::End(TagEnd::Paragraph | TagEnd::Heading(_)) => out.push('\n'),
            Event::End(TagEnd::Item) => out.push('\n'),
            _ => {}
        }
    }
    Ok(Value::Str(Arc::from(out.trim())))
}
