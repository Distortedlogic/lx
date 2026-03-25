#[path = "md_parse.rs"]
mod md_parse;
#[path = "md_render.rs"]
mod md_render;

use std::sync::Arc;

use indexmap::IndexMap;
use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};

use crate::error::LxError;
use crate::runtime::RuntimeCtx;
use crate::std_module;
use crate::value::LxVal;
use miette::SourceSpan;

pub fn build() -> IndexMap<crate::sym::Sym, LxVal> {
  std_module! {
    "parse"       => "md.parse",       1, bi_parse;
    "sections"    => "md.sections",    1, md_parse::bi_sections;
    "code_blocks" => "md.code_blocks", 1, md_parse::bi_code_blocks;
    "headings"    => "md.headings",    1, md_parse::bi_headings;
    "links"       => "md.links",       1, md_parse::bi_links;
    "to_text"     => "md.to_text",     1, md_parse::bi_to_text;
    "render"      => "md.render",      1, md_render::bi_render;
    "h1"          => "md.h1",          1, bi_h1;
    "h2"          => "md.h2",          1, bi_h2;
    "h3"          => "md.h3",          1, bi_h3;
    "para"        => "md.para",        1, bi_para;
    "code"        => "md.code",        2, bi_code;
    "list"        => "md.list",        1, bi_list;
    "ordered"     => "md.ordered",     1, bi_ordered;
    "table"       => "md.table",       2, bi_table;
    "link"        => "md.link",        2, bi_link;
    "blockquote"  => "md.blockquote",  1, bi_blockquote;
    "hr"          => "md.hr",          0, bi_hr;
    "raw"         => "md.raw",         1, bi_raw
  }
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

pub(super) fn node_rec(type_str: &str, fields: Vec<(&str, LxVal)>) -> LxVal {
  let mut rec = IndexMap::new();
  rec.insert(crate::sym::intern("type"), LxVal::str(type_str));
  for (k, v) in fields {
    rec.insert(k.into(), v);
  }
  LxVal::record(rec)
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

fn parse_to_nodes(input: &str) -> Vec<LxVal> {
  let parser = Parser::new_ext(input, Options::ENABLE_TABLES);
  let mut nodes = Vec::new();
  let mut text = String::new();
  let mut stack: Vec<Block> = Vec::new();
  let mut items: Vec<LxVal> = Vec::new();
  let mut code_lang: Option<String> = None;
  for event in parser {
    match event {
      Event::Start(tag) => match tag {
        Tag::Heading { .. } => {
          text.clear();
          stack.push(Block::Heading);
        },
        Tag::Paragraph => {
          if !stack.iter().any(|b| matches!(b, Block::Item)) {
            text.clear();
          }
          stack.push(Block::Para);
        },
        Tag::CodeBlock(kind) => {
          text.clear();
          code_lang = match kind {
            CodeBlockKind::Fenced(l) if !l.is_empty() => Some(l.to_string()),
            _ => None,
          };
          stack.push(Block::Code);
        },
        Tag::List(_) => {
          items.clear();
          stack.push(Block::List);
        },
        Tag::Item => {
          text.clear();
          stack.push(Block::Item);
        },
        Tag::BlockQuote(_) => {
          text.clear();
          stack.push(Block::Quote);
        },
        _ => stack.push(Block::Other),
      },
      Event::End(tag_end) => {
        let _ = stack.pop();
        match tag_end {
          TagEnd::Heading(level) => {
            nodes.push(node_rec("heading", vec![("level", LxVal::int(h_level(level))), ("text", LxVal::str(text.trim()))]));
          },
          TagEnd::Paragraph if !stack.iter().any(|b| matches!(b, Block::Item | Block::Quote)) => {
            nodes.push(node_rec("para", vec![("text", LxVal::str(text.trim()))]));
          },
          TagEnd::CodeBlock => {
            let lang = match code_lang.take() {
              Some(l) => LxVal::some(LxVal::str(l)),
              None => LxVal::None,
            };
            nodes.push(node_rec("code", vec![("lang", lang), ("code", LxVal::str(text.trim_end()))]));
          },
          TagEnd::Item => items.push(LxVal::str(text.trim())),
          TagEnd::List(is_ordered) => {
            let t = if is_ordered { "ordered" } else { "list" };
            nodes.push(node_rec(t, vec![("items", LxVal::list(std::mem::take(&mut items)))]));
          },
          TagEnd::BlockQuote(_) => {
            nodes.push(node_rec("blockquote", vec![("text", LxVal::str(text.trim()))]));
          },
          _ => {},
        }
      },
      Event::Text(t) => text.push_str(&t),
      Event::Code(c) => {
        text.push('`');
        text.push_str(&c);
        text.push('`');
      },
      Event::SoftBreak | Event::HardBreak => text.push('\n'),
      Event::Rule => nodes.push(node_rec("hr", vec![])),
      _ => {},
    }
  }
  nodes
}

fn bi_parse(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let input = args[0].require_str("md.parse", span)?;
  Ok(LxVal::list(parse_to_nodes(input)))
}

pub(super) fn field_str(rec: &IndexMap<crate::sym::Sym, LxVal>, field: &str) -> Option<String> {
  rec.get(&crate::sym::intern(field)).and_then(|v| v.as_str()).map(|s| s.to_string())
}

pub(super) fn get_nodes(val: &LxVal, span: SourceSpan) -> Result<&[LxVal], LxError> {
  val.as_list().map(|l| l.as_slice()).ok_or_else(|| LxError::type_err("md: expected List (parsed doc)", span, None))
}

pub(super) fn nodes_by_type(nodes: &[LxVal], type_name: &str) -> Vec<LxVal> {
  nodes.iter().filter(|n| if let LxVal::Record(r) = n { field_str(r, "type").as_deref() == Some(type_name) } else { false }).cloned().collect()
}

fn bi_h1(args: &[LxVal], span: SourceSpan, _: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  Ok(node_rec("heading", vec![("level", LxVal::int(1)), ("text", LxVal::str(args[0].require_str("md.h1", span)?))]))
}
fn bi_h2(args: &[LxVal], span: SourceSpan, _: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  Ok(node_rec("heading", vec![("level", LxVal::int(2)), ("text", LxVal::str(args[0].require_str("md.h2", span)?))]))
}
fn bi_h3(args: &[LxVal], span: SourceSpan, _: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  Ok(node_rec("heading", vec![("level", LxVal::int(3)), ("text", LxVal::str(args[0].require_str("md.h3", span)?))]))
}
fn bi_para(args: &[LxVal], span: SourceSpan, _: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  Ok(node_rec("para", vec![("text", LxVal::str(args[0].require_str("md.para", span)?))]))
}
fn bi_code(args: &[LxVal], span: SourceSpan, _: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let lang = args[0].require_str("md.code", span)?;
  let body = args[1].require_str("md.code", span)?;
  Ok(node_rec("code", vec![("lang", LxVal::some(LxVal::str(lang))), ("code", LxVal::str(body))]))
}
fn bi_list(args: &[LxVal], span: SourceSpan, _: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let items = args[0].as_list().ok_or_else(|| LxError::type_err("md.list: expected List", span, None))?;
  Ok(node_rec("list", vec![("items", LxVal::list(items.to_vec()))]))
}
fn bi_ordered(args: &[LxVal], span: SourceSpan, _: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let items = args[0].as_list().ok_or_else(|| LxError::type_err("md.ordered: expected List", span, None))?;
  Ok(node_rec("ordered", vec![("items", LxVal::list(items.to_vec()))]))
}
fn bi_table(args: &[LxVal], span: SourceSpan, _: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let headers = args[0].as_list().ok_or_else(|| LxError::type_err("md.table: expected List for headers", span, None))?;
  let rows = args[1].as_list().ok_or_else(|| LxError::type_err("md.table: expected List for rows", span, None))?;
  Ok(node_rec("table", vec![("headers", LxVal::list(headers.to_vec())), ("rows", LxVal::list(rows.to_vec()))]))
}
fn bi_link(args: &[LxVal], span: SourceSpan, _: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let text = args[0].require_str("md.link", span)?;
  let url = args[1].require_str("md.link", span)?;
  Ok(node_rec("link", vec![("text", LxVal::str(text)), ("url", LxVal::str(url))]))
}
fn bi_blockquote(args: &[LxVal], span: SourceSpan, _: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  Ok(node_rec("blockquote", vec![("text", LxVal::str(args[0].require_str("md.blockquote", span)?))]))
}
fn bi_hr(_: &[LxVal], _: SourceSpan, _: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  Ok(node_rec("hr", vec![]))
}
fn bi_raw(args: &[LxVal], span: SourceSpan, _: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  Ok(node_rec("raw", vec![("text", LxVal::str(args[0].require_str("md.raw", span)?))]))
}
