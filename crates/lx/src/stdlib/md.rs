#[path = "md_parse.rs"]
mod md_parse;

use std::sync::Arc;

use indexmap::IndexMap;
use num_bigint::BigInt;
use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};

use crate::backends::RuntimeCtx;
use crate::builtins::mk;
use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

pub fn build() -> IndexMap<String, Value> {
    let mut m = IndexMap::new();
    m.insert("parse".into(), mk("md.parse", 1, bi_parse));
    m.insert(
        "sections".into(),
        mk("md.sections", 1, md_parse::bi_sections),
    );
    m.insert(
        "code_blocks".into(),
        mk("md.code_blocks", 1, md_parse::bi_code_blocks),
    );
    m.insert(
        "headings".into(),
        mk("md.headings", 1, md_parse::bi_headings),
    );
    m.insert("links".into(), mk("md.links", 1, md_parse::bi_links));
    m.insert("to_text".into(), mk("md.to_text", 1, md_parse::bi_to_text));
    m.insert(
        "render".into(),
        mk("md.render", 1, super::md_build::bi_render),
    );
    super::md_build::register(&mut m);
    m
}

fn h_level(level: HeadingLevel) -> i64 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

pub(super) fn node_rec(type_str: &str, fields: Vec<(&str, Value)>) -> Value {
    let mut rec = IndexMap::new();
    rec.insert("type".into(), Value::Str(Arc::from(type_str)));
    for (k, v) in fields {
        rec.insert(k.into(), v);
    }
    Value::Record(Arc::new(rec))
}

enum Block {
    Heading,
    Para,
    Code,
    List,
    Item,
    Quote,
    Other,
}

fn parse_to_nodes(input: &str) -> Vec<Value> {
    let parser = Parser::new_ext(input, Options::ENABLE_TABLES);
    let mut nodes = Vec::new();
    let mut text = String::new();
    let mut stack: Vec<Block> = Vec::new();
    let mut items: Vec<Value> = Vec::new();
    let mut code_lang: Option<String> = None;
    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                Tag::Heading { .. } => {
                    text.clear();
                    stack.push(Block::Heading);
                }
                Tag::Paragraph => {
                    if !stack.iter().any(|b| matches!(b, Block::Item)) {
                        text.clear();
                    }
                    stack.push(Block::Para);
                }
                Tag::CodeBlock(kind) => {
                    text.clear();
                    code_lang = match kind {
                        CodeBlockKind::Fenced(l) if !l.is_empty() => Some(l.to_string()),
                        _ => None,
                    };
                    stack.push(Block::Code);
                }
                Tag::List(_) => {
                    items.clear();
                    stack.push(Block::List);
                }
                Tag::Item => {
                    text.clear();
                    stack.push(Block::Item);
                }
                Tag::BlockQuote(_) => {
                    text.clear();
                    stack.push(Block::Quote);
                }
                _ => stack.push(Block::Other),
            },
            Event::End(tag_end) => {
                let _ = stack.pop();
                match tag_end {
                    TagEnd::Heading(level) => {
                        nodes.push(node_rec(
                            "heading",
                            vec![
                                ("level", Value::Int(BigInt::from(h_level(level)))),
                                ("text", Value::Str(Arc::from(text.trim()))),
                            ],
                        ));
                    }
                    TagEnd::Paragraph
                        if !stack
                            .iter()
                            .any(|b| matches!(b, Block::Item | Block::Quote)) =>
                    {
                        nodes.push(node_rec(
                            "para",
                            vec![("text", Value::Str(Arc::from(text.trim())))],
                        ));
                    }
                    TagEnd::CodeBlock => {
                        let lang = match code_lang.take() {
                            Some(l) => Value::Some(Box::new(Value::Str(Arc::from(l.as_str())))),
                            None => Value::None,
                        };
                        nodes.push(node_rec(
                            "code",
                            vec![
                                ("lang", lang),
                                ("code", Value::Str(Arc::from(text.trim_end()))),
                            ],
                        ));
                    }
                    TagEnd::Item => items.push(Value::Str(Arc::from(text.trim()))),
                    TagEnd::List(is_ordered) => {
                        let t = if is_ordered { "ordered" } else { "list" };
                        nodes.push(node_rec(
                            t,
                            vec![("items", Value::List(Arc::new(std::mem::take(&mut items))))],
                        ));
                    }
                    TagEnd::BlockQuote(_) => {
                        nodes.push(node_rec(
                            "blockquote",
                            vec![("text", Value::Str(Arc::from(text.trim())))],
                        ));
                    }
                    _ => {}
                }
            }
            Event::Text(t) => text.push_str(&t),
            Event::Code(c) => {
                text.push('`');
                text.push_str(&c);
                text.push('`');
            }
            Event::SoftBreak | Event::HardBreak => text.push('\n'),
            Event::Rule => nodes.push(node_rec("hr", vec![])),
            _ => {}
        }
    }
    nodes
}

fn bi_parse(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let input = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("md.parse expects Str", span))?;
    Ok(Value::List(Arc::new(parse_to_nodes(input))))
}

pub(super) fn field_str(rec: &IndexMap<String, Value>, field: &str) -> Option<String> {
    rec.get(field)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

pub(super) fn get_nodes(val: &Value, span: Span) -> Result<&[Value], LxError> {
    val.as_list()
        .map(|l| l.as_slice())
        .ok_or_else(|| LxError::type_err("md: expected List (parsed doc)", span))
}

pub(super) fn nodes_by_type(nodes: &[Value], type_name: &str) -> Vec<Value> {
    nodes
        .iter()
        .filter(|n| {
            if let Value::Record(r) = n {
                field_str(r, "type").as_deref() == Some(type_name)
            } else {
                false
            }
        })
        .cloned()
        .collect()
}
