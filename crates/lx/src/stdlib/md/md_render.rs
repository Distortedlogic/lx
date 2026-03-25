use std::sync::Arc;

use crate::error::LxError;
use crate::runtime::RuntimeCtx;
use crate::value::LxVal;
use miette::SourceSpan;

use super::{field_str, get_nodes};

pub(super) fn bi_render(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let nodes = get_nodes(&args[0], span)?;
  let parts: Vec<String> = nodes.iter().filter_map(render_node).collect();
  Ok(LxVal::str(parts.join("\n\n")))
}

fn render_node(val: &LxVal) -> Option<String> {
  let LxVal::Record(r) = val else { return None };
  let t = field_str(r, "type")?;
  Some(match t.as_str() {
    "heading" => {
      let level: usize = r.get(&crate::sym::intern("level")).and_then(|v| v.as_int()).and_then(|n| n.try_into().ok()).unwrap_or(1);
      let text = field_str(r, "text").unwrap_or_default();
      format!("{} {text}", "#".repeat(level))
    },
    "para" => field_str(r, "text").unwrap_or_default(),
    "code" => {
      let lang = r
        .get(&crate::sym::intern("lang"))
        .and_then(|v| match v {
          LxVal::Some(inner) => inner.as_str().map(|s| s.to_string()),
          _ => None,
        })
        .unwrap_or_default();
      let code = field_str(r, "code").or_else(|| field_str(r, "text")).unwrap_or_default();
      format!("```{lang}\n{code}\n```")
    },
    "list" => render_items(r, "- "),
    "ordered" => render_ordered_items(r),
    "table" => render_table(r),
    "link" => {
      let text = field_str(r, "text").unwrap_or_default();
      let url = field_str(r, "url").unwrap_or_default();
      format!("[{text}]({url})")
    },
    "blockquote" => {
      let text = field_str(r, "text").unwrap_or_default();
      format!("> {text}")
    },
    "hr" => "---".to_string(),
    "raw" => field_str(r, "text").unwrap_or_default(),
    _ => field_str(r, "text").unwrap_or_default(),
  })
}

fn render_items(r: &indexmap::IndexMap<crate::sym::Sym, LxVal>, prefix: &str) -> String {
  let items = r.get(&crate::sym::intern("items")).and_then(|v| v.as_list()).cloned().unwrap_or_default();
  items.iter().map(|i| format!("{prefix}{i}")).collect::<Vec<_>>().join("\n")
}

fn render_ordered_items(r: &indexmap::IndexMap<crate::sym::Sym, LxVal>) -> String {
  let items = r.get(&crate::sym::intern("items")).and_then(|v| v.as_list()).cloned().unwrap_or_default();
  items.iter().enumerate().map(|(i, item)| format!("{}. {item}", i + 1)).collect::<Vec<_>>().join("\n")
}

fn render_table(r: &indexmap::IndexMap<crate::sym::Sym, LxVal>) -> String {
  let headers = r.get(&crate::sym::intern("headers")).and_then(|v| v.as_list()).cloned().unwrap_or_default();
  let rows = r.get(&crate::sym::intern("rows")).and_then(|v| v.as_list()).cloned().unwrap_or_default();
  let header_strs: Vec<String> = headers.iter().map(|h| format!("{h}")).collect();
  let header_row = format!("| {} |", header_strs.join(" | "));
  let sep_row = format!("| {} |", header_strs.iter().map(|_| "---").collect::<Vec<_>>().join(" | "));
  let data_rows: Vec<String> = rows
    .iter()
    .map(|row| {
      let cells: Vec<String> = row.as_list().map(|l| l.iter().map(|c| format!("{c}")).collect()).unwrap_or_default();
      format!("| {} |", cells.join(" | "))
    })
    .collect();
  format!("{header_row}\n{sep_row}\n{}", data_rows.join("\n"))
}
