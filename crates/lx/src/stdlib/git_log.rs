use std::sync::Arc;

use indexmap::IndexMap;
use num_bigint::BigInt;

use crate::backends::RuntimeCtx;
use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

use super::git::{get_bool, get_int, get_str, git_err, git_err_from, git_ok, run_git, str_val};

const LOG_FMT: &str = "%H\x1f%h\x1f%an\x1f%ae\x1f%aI\x1f%s\x1f%b\x1f%P\x1e";

pub fn bi_log(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let opts = match &args[0] {
        Value::Record(r) => r.as_ref().clone(),
        Value::Unit => IndexMap::new(),
        _ => {
            return Err(LxError::type_err(
                "git.log expects Record opts or ()",
                span,
            ))
        }
    };
    let n = get_int(&opts, "n").unwrap_or(10);
    let mut cmd_args = vec![
        "log".to_string(),
        format!("--format={LOG_FMT}"),
        format!("-{n}"),
    ];
    if let Some(author) = get_str(&opts, "author") {
        cmd_args.push(format!("--author={author}"));
    }
    if let Some(since) = get_str(&opts, "since") {
        cmd_args.push(format!("--since={since}"));
    }
    if let Some(until) = get_str(&opts, "until") {
        cmd_args.push(format!("--until={until}"));
    }
    if let Some(grep) = get_str(&opts, "grep") {
        cmd_args.push(format!("--grep={grep}"));
    }
    if get_bool(&opts, "all").unwrap_or(false) {
        cmd_args.push("--all".to_string());
    }
    if let Some(r) = get_str(&opts, "ref") {
        cmd_args.push(r.to_string());
    }
    if let Some(path) = get_str(&opts, "path") {
        cmd_args.push("--".to_string());
        cmd_args.push(path.to_string());
    }
    let refs: Vec<&str> = cmd_args.iter().map(|s| s.as_str()).collect();
    match run_git(&refs) {
        Ok(out) if out.status.success() => {
            let raw = String::from_utf8_lossy(&out.stdout);
            let commits = parse_log_output(&raw, false);
            Ok(git_ok(Value::List(Arc::new(commits))))
        }
        Ok(out) => Ok(git_err_from(&out)),
        Err(e) => Ok(git_err(&format!("git: {e}"))),
    }
}

pub fn bi_show(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let ref_name = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("git.show expects Str ref", span))?;
    let fmt_arg = format!("--format={LOG_FMT}");
    match run_git(&["show", &fmt_arg, "--no-color", ref_name]) {
        Ok(out) if out.status.success() => {
            let raw = String::from_utf8_lossy(&out.stdout);
            let (commit_part, diff_part) = split_show_output(&raw);
            let commits = parse_log_output(commit_part, false);
            match commits.into_iter().next() {
                Some(Value::Record(fields)) => {
                    let mut fields = (*fields).clone();
                    let diff = super::git_diff::parse_unified_diff(diff_part);
                    fields.insert(
                        "diff".into(),
                        Value::Some(Box::new(Value::List(Arc::new(diff)))),
                    );
                    Ok(git_ok(Value::Record(Arc::new(fields))))
                }
                Some(c) => Ok(git_ok(c)),
                None => Ok(git_err("git.show: no commit found")),
            }
        }
        Ok(out) => Ok(git_err_from(&out)),
        Err(e) => Ok(git_err(&format!("git: {e}"))),
    }
}

fn split_show_output(raw: &str) -> (&str, &str) {
    if let Some(pos) = raw.find("\ndiff --git ") {
        (&raw[..pos], &raw[pos + 1..])
    } else if let Some(pos) = raw.find("\n\ndiff --git ") {
        (&raw[..pos], &raw[pos + 2..])
    } else {
        (raw, "")
    }
}

fn parse_log_output(raw: &str, _with_diff: bool) -> Vec<Value> {
    raw.split('\x1e')
        .filter(|entry| !entry.trim().is_empty())
        .filter_map(parse_commit_entry)
        .collect()
}

fn parse_commit_entry(entry: &str) -> Option<Value> {
    let entry = entry.trim();
    let parts: Vec<&str> = entry.split('\x1f').collect();
    if parts.len() < 8 {
        return None;
    }
    let parents: Vec<Value> = parts[7]
        .split_whitespace()
        .filter(|s| !s.is_empty())
        .map(str_val)
        .collect();
    let mut f = IndexMap::new();
    f.insert("hash".into(), str_val(parts[0]));
    f.insert("short".into(), str_val(parts[1]));
    f.insert("author".into(), str_val(parts[2]));
    f.insert("email".into(), str_val(parts[3]));
    f.insert("date".into(), str_val(parts[4]));
    f.insert("subject".into(), str_val(parts[5]));
    f.insert("body".into(), str_val(parts[6].trim()));
    f.insert("parents".into(), Value::List(Arc::new(parents)));
    f.insert("diff".into(), Value::None);
    Some(Value::Record(Arc::new(f)))
}

pub fn bi_blame(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let path = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("git.blame expects Str path", span))?;
    match run_git(&["blame", "--porcelain", path]) {
        Ok(out) if out.status.success() => {
            let raw = String::from_utf8_lossy(&out.stdout);
            Ok(git_ok(Value::List(Arc::new(parse_blame(&raw)))))
        }
        Ok(out) => Ok(git_err_from(&out)),
        Err(e) => Ok(git_err(&format!("git: {e}"))),
    }
}

pub fn bi_blame_range(
    args: &[Value],
    span: Span,
    _ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let path = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("git.blame_range: expects Str path", span))?;
    let from = args[1]
        .as_int()
        .ok_or_else(|| LxError::type_err("git.blame_range: expects Int from", span))?;
    let to = args[2]
        .as_int()
        .ok_or_else(|| LxError::type_err("git.blame_range: expects Int to", span))?;
    let range = format!("-L{from},{to}");
    match run_git(&["blame", "--porcelain", &range, path]) {
        Ok(out) if out.status.success() => {
            let raw = String::from_utf8_lossy(&out.stdout);
            Ok(git_ok(Value::List(Arc::new(parse_blame(&raw)))))
        }
        Ok(out) => Ok(git_err_from(&out)),
        Err(e) => Ok(git_err(&format!("git: {e}"))),
    }
}

fn parse_blame(raw: &str) -> Vec<Value> {
    let mut results = Vec::new();
    let mut current_hash = String::new();
    let mut current_author = String::new();
    let mut current_date = String::new();
    let mut current_line: i64 = 0;
    let mut in_header = true;

    for line in raw.lines() {
        if in_header && line.starts_with(|c: char| c.is_ascii_hexdigit()) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 && parts[0].len() >= 40 {
                current_hash = parts[0].to_string();
                current_line = parts[2].parse().unwrap_or(0);
                in_header = true;
            }
        } else if let Some(rest) = line.strip_prefix("author ") {
            current_author = rest.to_string();
        } else if let Some(rest) = line.strip_prefix("author-time ") {
            current_date = rest.to_string();
        } else if let Some(content) = line.strip_prefix('\t') {
            let mut f = IndexMap::new();
            f.insert("hash".into(), str_val(&current_hash));
            f.insert("author".into(), str_val(&current_author));
            f.insert("date".into(), str_val(&current_date));
            f.insert("line".into(), Value::Int(BigInt::from(current_line)));
            f.insert("content".into(), str_val(content));
            results.push(Value::Record(Arc::new(f)));
            in_header = true;
        }
    }
    results
}
