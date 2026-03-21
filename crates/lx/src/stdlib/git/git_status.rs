use std::sync::Arc;

use indexmap::IndexMap;

use crate::backends::RuntimeCtx;
use crate::error::LxError;
use crate::record;
use crate::span::Span;
use crate::value::Value;

use super::git::{git_err, git_err_from, git_ok, int_val, run_git, str_val};

pub(super) fn parse_status(raw: &str) -> Value {
    let mut branch = String::new();
    let mut staged = Vec::new();
    let mut unstaged = Vec::new();
    let mut untracked = Vec::new();
    let mut conflicts = Vec::new();

    for entry in raw.split('\0') {
        let entry = entry.trim();
        if entry.is_empty() {
            continue;
        }
        if let Some(rest) = entry.strip_prefix("# branch.head ") {
            branch = rest.to_string();
        } else if let Some(rest) = entry.strip_prefix("1 ") {
            parse_changed_entry(rest, &mut staged, &mut unstaged);
        } else if let Some(rest) = entry.strip_prefix("2 ") {
            parse_renamed_entry(rest, &mut staged, &mut unstaged);
        } else if let Some(rest) = entry.strip_prefix("u ") {
            parse_conflict_entry(rest, &mut conflicts);
        } else if let Some(rest) = entry.strip_prefix("? ") {
            untracked.push(str_val(rest));
        }
    }

    let clean =
        staged.is_empty() && unstaged.is_empty() && untracked.is_empty() && conflicts.is_empty();
    record! {
        "branch" => str_val(&branch),
        "clean" => Value::Bool(clean),
        "staged" => Value::List(Arc::new(staged)),
        "unstaged" => Value::List(Arc::new(unstaged)),
        "untracked" => Value::List(Arc::new(untracked)),
        "conflicts" => Value::List(Arc::new(conflicts)),
    }
}

fn xy_to_action(c: char) -> &'static str {
    match c {
        'A' => "added",
        'M' => "modified",
        'D' => "deleted",
        'R' => "renamed",
        'C' => "copied",
        'T' => "modified",
        _ => "modified",
    }
}

fn action_record(path: &str, action: &str) -> Value {
    record! {
        "path" => str_val(path),
        "action" => str_val(action),
    }
}

fn parse_changed_entry(rest: &str, staged: &mut Vec<Value>, unstaged: &mut Vec<Value>) {
    let parts: Vec<&str> = rest.splitn(9, ' ').collect();
    if parts.len() < 9 {
        return;
    }
    let xy = parts[0];
    let path = parts[8];
    let chars: Vec<char> = xy.chars().collect();
    if chars.len() >= 2 {
        if chars[0] != '.' {
            staged.push(action_record(path, xy_to_action(chars[0])));
        }
        if chars[1] != '.' {
            unstaged.push(action_record(path, xy_to_action(chars[1])));
        }
    }
}

fn parse_renamed_entry(rest: &str, staged: &mut Vec<Value>, unstaged: &mut Vec<Value>) {
    let parts: Vec<&str> = rest.splitn(10, ' ').collect();
    if parts.len() < 10 {
        return;
    }
    let xy = parts[0];
    let path = parts[9];
    let chars: Vec<char> = xy.chars().collect();
    if chars.len() >= 2 {
        if chars[0] != '.' {
            staged.push(action_record(path, xy_to_action(chars[0])));
        }
        if chars[1] != '.' {
            unstaged.push(action_record(path, xy_to_action(chars[1])));
        }
    }
}

fn parse_conflict_entry(rest: &str, conflicts: &mut Vec<Value>) {
    let parts: Vec<&str> = rest.splitn(11, ' ').collect();
    if parts.len() < 11 {
        return;
    }
    let xy = parts[0];
    let path = parts[10];
    let chars: Vec<char> = xy.chars().collect();
    conflicts.push(record! {
        "path" => str_val(path),
        "ours" => str_val(&chars.first().unwrap_or(&'?').to_string()),
        "theirs" => str_val(&chars.get(1).unwrap_or(&'?').to_string()),
    });
}

pub fn bi_branches(args: &[Value], _span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let _ = &args[0];
    let fmt = "%(refname:short)\x1f%(HEAD)\x1f%(upstream:short)\x1f%(upstream:track,nobracket)";
    match run_git(&["for-each-ref", "--format", fmt, "refs/heads/"]) {
        Ok(out) if out.status.success() => {
            let raw = String::from_utf8_lossy(&out.stdout);
            let branches: Vec<Value> = raw
                .lines()
                .filter(|l| !l.is_empty())
                .map(parse_branch_line)
                .collect();
            Ok(git_ok(Value::List(Arc::new(branches))))
        }
        Ok(out) => Ok(git_err_from(&out)),
        Err(e) => Ok(git_err(&format!("git: {e}"))),
    }
}

fn parse_branch_line(line: &str) -> Value {
    let parts: Vec<&str> = line.split('\x1f').collect();
    let name = parts.first().unwrap_or(&"");
    let current = parts.get(1).unwrap_or(&"") == &"*";
    let remote_raw = parts.get(2).unwrap_or(&"");
    let track = parts.get(3).unwrap_or(&"");
    let remote = if remote_raw.is_empty() {
        Value::None
    } else {
        Value::Some(Box::new(str_val(remote_raw)))
    };
    let (ahead, behind) = parse_track(track);
    record! {
        "name" => str_val(name),
        "current" => Value::Bool(current),
        "remote" => remote,
        "ahead" => int_val(ahead),
        "behind" => int_val(behind),
    }
}

fn parse_track(track: &str) -> (i64, i64) {
    let mut ahead = 0i64;
    let mut behind = 0i64;
    for part in track.split(", ") {
        let part = part.trim();
        if let Some(rest) = part.strip_prefix("ahead ") {
            ahead = rest.parse().unwrap_or(0);
        } else if let Some(rest) = part.strip_prefix("behind ") {
            behind = rest.parse().unwrap_or(0);
        }
    }
    (ahead, behind)
}

pub fn bi_remotes(args: &[Value], _span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let _ = &args[0];
    match run_git(&["remote", "-v"]) {
        Ok(out) if out.status.success() => {
            let raw = String::from_utf8_lossy(&out.stdout);
            let mut seen = IndexMap::new();
            for line in raw.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 && !seen.contains_key(parts[0]) {
                    seen.insert(parts[0].to_string(), parts[1].to_string());
                }
            }
            let remotes: Vec<Value> = seen
                .into_iter()
                .map(|(name, url)| {
                    record! {
                        "name" => str_val(&name),
                        "url" => str_val(&url),
                    }
                })
                .collect();
            Ok(git_ok(Value::List(Arc::new(remotes))))
        }
        Ok(out) => Ok(git_err_from(&out)),
        Err(e) => Ok(git_err(&format!("git: {e}"))),
    }
}
