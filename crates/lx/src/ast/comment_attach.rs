use std::cmp::Reverse;
use std::collections::HashMap;

use crate::source::{AttachedComment, CommentMap, CommentPlacement, CommentStore};

use super::{AstArena, NodeId, StmtId};

pub fn attach_comments(stmts: &[StmtId], arena: &AstArena, comments: &CommentStore, source: &str) -> CommentMap {
  let mut map: CommentMap = HashMap::new();
  if comments.all().is_empty() {
    return map;
  }

  let mut nodes: Vec<(NodeId, usize, usize)> = Vec::new();
  for (id, spanned) in arena.iter_exprs() {
    let offset = spanned.span.offset();
    let end = offset + spanned.span.len();
    nodes.push((NodeId::Expr(id), offset, end));
  }
  for (id, spanned) in arena.iter_stmts() {
    let offset = spanned.span.offset();
    let end = offset + spanned.span.len();
    nodes.push((NodeId::Stmt(id), offset, end));
  }
  for (id, spanned) in arena.iter_patterns() {
    let offset = spanned.span.offset();
    let end = offset + spanned.span.len();
    nodes.push((NodeId::Pattern(id), offset, end));
  }
  for (id, spanned) in arena.iter_type_exprs() {
    let offset = spanned.span.offset();
    let end = offset + spanned.span.len();
    nodes.push((NodeId::TypeExpr(id), offset, end));
  }
  nodes.sort_by_key(|&(_, offset, end)| (offset, Reverse(end)));

  for (comment_idx, comment) in comments.all().iter().enumerate() {
    let c_offset = comment.span.offset();
    let c_end = c_offset + comment.span.len();

    let enclosing = nodes.iter().filter(|&&(_, n_offset, n_end)| n_offset <= c_offset && c_end <= n_end).min_by_key(|&&(_, n_offset, n_end)| n_end - n_offset);

    let (node_id, placement) = if let Some(&(node_id, n_offset, n_end)) = enclosing {
      let before_comment = &source[n_offset..c_offset];
      let after_comment = &source[c_end..n_end.min(source.len())];
      let has_node_content_before = before_comment.trim().len() > before_comment.trim_start_matches(|c: char| c.is_whitespace()).len()
        || before_comment.contains(|c: char| !c.is_whitespace());
      let has_newline_after = after_comment.starts_with('\n') || after_comment.starts_with("\r\n") || after_comment.is_empty();

      if has_newline_after && !has_node_content_before {
        (node_id, CommentPlacement::Leading)
      } else if has_node_content_before {
        (node_id, CommentPlacement::Trailing)
      } else {
        (node_id, CommentPlacement::Dangling)
      }
    } else {
      let first_stmt = stmts.first().map(|s| NodeId::Stmt(*s));
      let last_stmt = stmts.last().map(|s| NodeId::Stmt(*s));

      if let Some(first) = first_stmt {
        let first_offset = first.span(arena).offset();
        if c_offset < first_offset {
          (first, CommentPlacement::Leading)
        } else if let Some(last) = last_stmt {
          (last, CommentPlacement::Trailing)
        } else {
          (first, CommentPlacement::Trailing)
        }
      } else {
        continue;
      }
    };

    map.entry(node_id).or_default().push(AttachedComment { comment_idx, placement });
  }

  map
}
