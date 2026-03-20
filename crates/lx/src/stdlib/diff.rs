use std::sync::Arc;

use indexmap::IndexMap;
use num_bigint::BigInt;
use similar::{ChangeTag, TextDiff};

use crate::backends::RuntimeCtx;
use crate::builtins::mk;
use crate::error::LxError;
use crate::record;
use crate::span::Span;
use crate::value::Value;

pub fn build() -> IndexMap<String, Value> {
    let mut m = IndexMap::new();
    m.insert("unified".into(), mk("diff.unified", 2, bi_unified));
    m.insert("hunks".into(), mk("diff.hunks", 2, bi_hunks));
    m.insert("apply".into(), mk("diff.apply", 2, bi_apply));
    m.insert("edits".into(), mk("diff.edits", 2, bi_edits));
    m.insert(
        "merge3".into(),
        mk("diff.merge3", 3, super::diff_merge::bi_merge3),
    );
    m
}

fn bi_unified(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let old = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("diff.unified expects Str for old text", span))?;
    let new = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("diff.unified expects Str for new text", span))?;
    let diff = TextDiff::from_lines(old, new);
    let result = diff.unified_diff().header("old", "new").to_string();
    Ok(Value::Str(Arc::from(result.as_str())))
}

fn bi_hunks(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let old = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("diff.hunks expects Str for old text", span))?;
    let new = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("diff.hunks expects Str for new text", span))?;
    let diff = TextDiff::from_lines(old, new);
    let mut hunks = Vec::new();
    for group in diff.grouped_ops(3) {
        let old_start = group.first().map(|op| op.old_range().start).unwrap_or(0);
        let new_start = group.first().map(|op| op.new_range().start).unwrap_or(0);
        let mut changes = Vec::new();
        for op in &group {
            for change in diff.iter_changes(op) {
                let tag_str = match change.tag() {
                    ChangeTag::Equal => "equal",
                    ChangeTag::Insert => "insert",
                    ChangeTag::Delete => "delete",
                };
                let old_line = change
                    .old_index()
                    .map(|i| Value::Int(BigInt::from(i)))
                    .unwrap_or(Value::None);
                let new_line = change
                    .new_index()
                    .map(|i| Value::Int(BigInt::from(i)))
                    .unwrap_or(Value::None);
                changes.push(record! {
                    "tag" => Value::Str(Arc::from(tag_str)),
                    "value" => Value::Str(Arc::from(change.value())),
                    "old_line" => old_line,
                    "new_line" => new_line,
                });
            }
        }
        hunks.push(record! {
            "old_start" => Value::Int(BigInt::from(old_start)),
            "new_start" => Value::Int(BigInt::from(new_start)),
            "changes" => Value::List(Arc::new(changes)),
        });
    }
    Ok(Value::List(Arc::new(hunks)))
}

fn bi_apply(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let original = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("diff.apply expects Str for original text", span))?;
    let patch = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("diff.apply expects Str for patch", span))?;
    let orig_lines: Vec<&str> = original.lines().collect();
    let mut result_lines: Vec<String> = Vec::new();
    let mut orig_idx: usize = 0;
    let patch_lines: Vec<&str> = patch.lines().collect();
    let mut hunks: Vec<(usize, Vec<&str>)> = Vec::new();
    let mut i = 0;
    while i < patch_lines.len() {
        let line = patch_lines[i];
        if let Some(rest) = line.strip_prefix("@@ -")
            && let Some(comma_or_space) = rest.find([',', ' '])
        {
            let start_str = &rest[..comma_or_space];
            if let Ok(start) = start_str.parse::<usize>() {
                let hunk_start = if start > 0 { start - 1 } else { 0 };
                let mut hunk_lines = Vec::new();
                i += 1;
                while i < patch_lines.len() && !patch_lines[i].starts_with("@@") {
                    hunk_lines.push(patch_lines[i]);
                    i += 1;
                }
                hunks.push((hunk_start, hunk_lines));
                continue;
            }
        }
        i += 1;
    }
    for (hunk_start, hunk_lines) in &hunks {
        while orig_idx < *hunk_start && orig_idx < orig_lines.len() {
            result_lines.push(orig_lines[orig_idx].to_string());
            orig_idx += 1;
        }
        for hl in hunk_lines {
            if let Some(content) = hl.strip_prefix('+') {
                result_lines.push(content.to_string());
            } else if hl.starts_with('-') {
                orig_idx += 1;
            } else if let Some(content) = hl.strip_prefix(' ') {
                result_lines.push(content.to_string());
                orig_idx += 1;
            } else if !hl.starts_with('\\') {
                result_lines.push(hl.to_string());
                orig_idx += 1;
            }
        }
    }
    while orig_idx < orig_lines.len() {
        result_lines.push(orig_lines[orig_idx].to_string());
        orig_idx += 1;
    }
    let mut result = result_lines.join("\n");
    if original.ends_with('\n') && !result.ends_with('\n') {
        result.push('\n');
    }
    Ok(Value::Ok(Box::new(Value::Str(Arc::from(result.as_str())))))
}

fn bi_edits(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let text = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("diff.edits expects Str for text", span))?;
    let edits = args[1]
        .as_list()
        .ok_or_else(|| LxError::type_err("diff.edits expects List of edit records", span))?;
    let mut lines: Vec<String> = text.lines().map(String::from).collect();
    let mut edit_pairs: Vec<(usize, String)> = Vec::new();
    for edit in edits.iter() {
        let Value::Record(rec) = edit else {
            return Err(LxError::type_err(
                "diff.edits: each edit must be a Record",
                span,
            ));
        };
        let line_val = rec
            .get("line")
            .ok_or_else(|| LxError::runtime("diff.edits: edit missing 'line' field", span))?;
        let line_num = line_val
            .as_int()
            .ok_or_else(|| LxError::type_err("diff.edits: 'line' must be Int", span))?;
        let line_idx: usize = line_num
            .try_into()
            .map_err(|_| LxError::runtime("diff.edits: invalid line number", span))?;
        let replacement = rec
            .get("text")
            .ok_or_else(|| LxError::runtime("diff.edits: edit missing 'text' field", span))?;
        let rep_str = replacement
            .as_str()
            .ok_or_else(|| LxError::type_err("diff.edits: 'text' must be Str", span))?;
        edit_pairs.push((line_idx, rep_str.to_string()));
    }
    edit_pairs.sort_by(|a, b| b.0.cmp(&a.0));
    for (line_num, replacement) in edit_pairs {
        if line_num == 0 || line_num > lines.len() {
            continue;
        }
        let idx = line_num - 1;
        if replacement.is_empty() {
            lines.remove(idx);
        } else {
            lines[idx] = replacement;
        }
    }
    let mut result = lines.join("\n");
    if text.ends_with('\n') && !result.ends_with('\n') {
        result.push('\n');
    }
    Ok(Value::Str(Arc::from(result.as_str())))
}
