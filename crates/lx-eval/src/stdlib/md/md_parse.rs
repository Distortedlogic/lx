use pulldown_cmark::{Event, Parser, Tag, TagEnd};

use lx_value::BuiltinCtx;
use lx_value::LxError;
use lx_value::LxVal;
use miette::SourceSpan;

use super::{field_str, get_nodes, node_rec, nodes_by_type};

pub(super) fn bi_sections(args: &[LxVal], span: SourceSpan, _ctx: &dyn BuiltinCtx) -> Result<LxVal, LxError> {
  let nodes = get_nodes(&args[0], span)?;
  let mut sections = Vec::new();
  let mut cur: Option<(i64, String, String)> = None;
  for node in nodes {
    if let LxVal::Record(r) = node {
      let type_name = field_str(r, "type");
      if type_name.as_deref() == Some("heading") {
        let lv: i64 = r.get(&lx_span::sym::intern("level")).and_then(|v| v.as_int()).and_then(|n| n.try_into().ok()).unwrap_or(1);
        let heading_text = field_str(r, "text").unwrap_or_default();
        match cur.as_mut() {
          Some((cur_lv, _, content)) if lv > *cur_lv => {
            if !content.is_empty() {
              content.push_str("\n\n");
            }
            content.push_str(&"#".repeat(lv as usize));
            content.push(' ');
            content.push_str(&heading_text);
          },
          Some(_) => {
            let (old_lv, old_title, old_content) = cur.take().expect("checked Some");
            sections
              .push(node_rec("section", vec![("level", LxVal::int(old_lv)), ("title", LxVal::str(old_title)), ("content", LxVal::str(old_content.trim()))]));
            cur = Some((lv, heading_text, String::new()));
          },
          None => {
            cur = Some((lv, heading_text, String::new()));
          },
        }
      } else if let Some((_, _, ref mut content)) = cur {
        let extracted = match type_name.as_deref() {
          Some("para") | Some("blockquote") => field_str(r, "text"),
          Some("code") => field_str(r, "code"),
          Some("list") | Some("ordered") => r
            .get(&lx_span::sym::intern("items"))
            .and_then(|v| v.as_list())
            .map(|items| items.iter().map(|item| format!("- {item}")).collect::<Vec<_>>().join("\n")),
          Some("hr") => Some("---".to_string()),
          _ => field_str(r, "text"),
        };
        if let Some(text) = extracted {
          if !content.is_empty() {
            content.push_str("\n\n");
          }
          content.push_str(&text);
        }
      }
    }
  }
  if let Some((lv, title, content)) = cur {
    sections.push(node_rec("section", vec![("level", LxVal::int(lv)), ("title", LxVal::str(title)), ("content", LxVal::str(content.trim()))]));
  }
  Ok(LxVal::list(sections))
}

pub(super) fn bi_code_blocks(args: &[LxVal], span: SourceSpan, _ctx: &dyn BuiltinCtx) -> Result<LxVal, LxError> {
  Ok(LxVal::list(nodes_by_type(get_nodes(&args[0], span)?, "code")))
}

pub(super) fn bi_headings(args: &[LxVal], span: SourceSpan, _ctx: &dyn BuiltinCtx) -> Result<LxVal, LxError> {
  Ok(LxVal::list(nodes_by_type(get_nodes(&args[0], span)?, "heading")))
}

pub(super) fn bi_links(args: &[LxVal], span: SourceSpan, _ctx: &dyn BuiltinCtx) -> Result<LxVal, LxError> {
  let input = args[0].require_str("md.links", span)?;
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
      },
      Event::End(TagEnd::Link) => {
        links.push(node_rec("link", vec![("text", LxVal::str(&link_text)), ("url", LxVal::str(&link_url))]));
        in_link = false;
      },
      Event::Text(t) if in_link => link_text.push_str(&t),
      _ => {},
    }
  }
  Ok(LxVal::list(links))
}

pub(super) fn bi_to_text(args: &[LxVal], span: SourceSpan, _ctx: &dyn BuiltinCtx) -> Result<LxVal, LxError> {
  let input = args[0].require_str("md.to_text", span)?;
  let parser = Parser::new(input);
  let mut out = String::new();
  for event in parser {
    match event {
      Event::Text(t) => out.push_str(&t),
      Event::Code(c) => out.push_str(&c),
      Event::SoftBreak | Event::HardBreak => out.push('\n'),
      Event::End(TagEnd::Paragraph | TagEnd::Heading(_)) => out.push('\n'),
      Event::End(TagEnd::Item) => out.push('\n'),
      _ => {},
    }
  }
  Ok(LxVal::str(out.trim()))
}
