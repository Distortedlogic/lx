use std::sync::Arc;

use num_bigint::BigInt;
use similar::{DiffOp, TextDiff};

use crate::backends::RuntimeCtx;
use crate::error::LxError;
use crate::record;
use crate::span::Span;
use crate::value::Value;

pub fn bi_merge3(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let base = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("diff.merge3 expects Str for base", span))?;
    let ours = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("diff.merge3 expects Str for ours", span))?;
    let theirs = args[2]
        .as_str()
        .ok_or_else(|| LxError::type_err("diff.merge3 expects Str for theirs", span))?;
    let base_lines: Vec<&str> = base.lines().collect();
    let ours_lines: Vec<&str> = ours.lines().collect();
    let theirs_lines: Vec<&str> = theirs.lines().collect();
    let diff_ours = TextDiff::from_slices(&base_lines, &ours_lines);
    let diff_theirs = TextDiff::from_slices(&base_lines, &theirs_lines);
    let ours_ops: Vec<_> = diff_ours.ops().to_vec();
    let theirs_ops: Vec<_> = diff_theirs.ops().to_vec();
    let mut result_lines: Vec<String> = Vec::new();
    let mut conflicts = 0usize;
    let base_len = base_lines.len();
    let mut ours_changes: Vec<Option<Vec<&str>>> = vec![None; base_len + 1];
    let mut theirs_changes: Vec<Option<Vec<&str>>> = vec![None; base_len + 1];
    let mut ours_deletes: Vec<bool> = vec![false; base_len];
    let mut theirs_deletes: Vec<bool> = vec![false; base_len];
    for op in &ours_ops {
        match op {
            DiffOp::Equal { .. } => {}
            DiffOp::Delete {
                old_index, old_len, ..
            } => {
                for slot in &mut ours_deletes[*old_index..(*old_index + *old_len)] {
                    *slot = true;
                }
            }
            DiffOp::Insert {
                old_index,
                new_index,
                new_len,
            } => {
                let lines: Vec<&str> = ours_lines[*new_index..(*new_index + *new_len)].to_vec();
                ours_changes[*old_index] = Some(lines);
            }
            DiffOp::Replace {
                old_index,
                old_len,
                new_index,
                new_len,
            } => {
                for slot in &mut ours_deletes[*old_index..(*old_index + *old_len)] {
                    *slot = true;
                }
                let lines: Vec<&str> = ours_lines[*new_index..(*new_index + *new_len)].to_vec();
                ours_changes[*old_index] = Some(lines);
            }
        }
    }
    for op in &theirs_ops {
        match op {
            DiffOp::Equal { .. } => {}
            DiffOp::Delete {
                old_index, old_len, ..
            } => {
                for slot in &mut theirs_deletes[*old_index..(*old_index + *old_len)] {
                    *slot = true;
                }
            }
            DiffOp::Insert {
                old_index,
                new_index,
                new_len,
            } => {
                let lines: Vec<&str> = theirs_lines[*new_index..(*new_index + *new_len)].to_vec();
                theirs_changes[*old_index] = Some(lines);
            }
            DiffOp::Replace {
                old_index,
                old_len,
                new_index,
                new_len,
            } => {
                for slot in &mut theirs_deletes[*old_index..(*old_index + *old_len)] {
                    *slot = true;
                }
                let lines: Vec<&str> = theirs_lines[*new_index..(*new_index + *new_len)].to_vec();
                theirs_changes[*old_index] = Some(lines);
            }
        }
    }
    for i in 0..=base_len {
        let o_ins = ours_changes[i].as_ref();
        let t_ins = theirs_changes[i].as_ref();
        match (o_ins, t_ins) {
            (Some(o), Some(t)) => {
                if o == t {
                    for l in o {
                        result_lines.push(l.to_string());
                    }
                } else {
                    conflicts += 1;
                    result_lines.push("<<<<<<<".to_string());
                    for l in o {
                        result_lines.push(l.to_string());
                    }
                    result_lines.push("=======".to_string());
                    for l in t {
                        result_lines.push(l.to_string());
                    }
                    result_lines.push(">>>>>>>".to_string());
                }
            }
            (Some(o), None) => {
                for l in o {
                    result_lines.push(l.to_string());
                }
            }
            (None, Some(t)) => {
                for l in t {
                    result_lines.push(l.to_string());
                }
            }
            (None, None) => {}
        }
        if i < base_len {
            let o_del = ours_deletes[i];
            let t_del = theirs_deletes[i];
            match (o_del, t_del) {
                (true, true) | (true, false) | (false, true) => {}
                (false, false) => {
                    result_lines.push(base_lines[i].to_string());
                }
            }
        }
    }
    let mut merged = result_lines.join("\n");
    if base.ends_with('\n') && !merged.ends_with('\n') {
        merged.push('\n');
    }
    if conflicts > 0 {
        Ok(Value::Err(Box::new(record! {
            "text" => Value::Str(Arc::from(merged.as_str())),
            "conflicts" => Value::Int(BigInt::from(conflicts)),
        })))
    } else {
        Ok(Value::Ok(Box::new(Value::Str(Arc::from(merged.as_str())))))
    }
}
