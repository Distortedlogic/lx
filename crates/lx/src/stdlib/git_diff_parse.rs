use std::sync::Arc;

use indexmap::IndexMap;
use num_bigint::BigInt;

use crate::value::Value;

use super::git::str_val;

pub(super) fn parse_unified_diff(raw: &str) -> Vec<Value> {
    let mut files = Vec::new();
    let mut current_path = String::new();
    let mut old_path: Option<String> = None;
    let mut status = "modified";
    let mut hunks: Vec<Value> = Vec::new();
    let mut current_hunk: Option<HunkState> = None;

    for line in raw.lines() {
        if line.starts_with("diff --git ") {
            flush_file(
                &mut files,
                &current_path,
                &old_path,
                status,
                &mut hunks,
                &mut current_hunk,
            );
            let (a, b) = parse_diff_header(line);
            current_path = b;
            old_path = if a != current_path { Some(a) } else { None };
            status = "modified";
        } else if line.starts_with("new file") {
            status = "added";
        } else if line.starts_with("deleted file") {
            status = "deleted";
        } else if line.starts_with("rename from") {
            status = "renamed";
        } else if line.starts_with("@@ ") {
            if let Some(ref mut h) = current_hunk {
                hunks.push(h.to_value());
            }
            current_hunk = Some(parse_hunk_header(line));
        } else if let Some(ref mut h) = current_hunk {
            match line.as_bytes().first() {
                Some(b'+') => h.add_line("add", &line[1..]),
                Some(b'-') => h.add_line("delete", &line[1..]),
                Some(b' ') => h.add_line("context", &line[1..]),
                _ => {}
            }
        }
    }
    flush_file(
        &mut files,
        &current_path,
        &old_path,
        status,
        &mut hunks,
        &mut current_hunk,
    );
    files
}

fn flush_file(
    files: &mut Vec<Value>,
    path: &str,
    old_path: &Option<String>,
    status: &str,
    hunks: &mut Vec<Value>,
    current_hunk: &mut Option<HunkState>,
) {
    if let Some(h) = current_hunk.as_ref() {
        hunks.push(h.to_value());
    }
    *current_hunk = None;
    if !path.is_empty() {
        let mut f = IndexMap::new();
        f.insert("path".into(), str_val(path));
        f.insert(
            "old_path".into(),
            match old_path {
                Some(p) => Value::Some(Box::new(str_val(p))),
                None => Value::None,
            },
        );
        f.insert("status".into(), str_val(status));
        f.insert("hunks".into(), Value::List(Arc::new(std::mem::take(hunks))));
        files.push(Value::Record(Arc::new(f)));
    }
}

fn parse_diff_header(line: &str) -> (String, String) {
    let rest = line.strip_prefix("diff --git ").unwrap_or(line);
    let parts: Vec<&str> = rest.split(' ').collect();
    let a = parts
        .first()
        .unwrap_or(&"")
        .strip_prefix("a/")
        .unwrap_or(parts.first().unwrap_or(&""));
    let b = parts
        .get(1)
        .unwrap_or(&"")
        .strip_prefix("b/")
        .unwrap_or(parts.get(1).unwrap_or(&""));
    (a.to_string(), b.to_string())
}

struct HunkState {
    old_start: i64,
    old_count: i64,
    new_start: i64,
    new_count: i64,
    header: String,
    lines: Vec<Value>,
    old_line: i64,
    new_line: i64,
}

impl HunkState {
    fn add_line(&mut self, kind: &str, content: &str) {
        let mut f = IndexMap::new();
        f.insert("kind".into(), str_val(kind));
        f.insert("content".into(), str_val(content));
        match kind {
            "add" => {
                f.insert("old_line".into(), Value::None);
                f.insert(
                    "new_line".into(),
                    Value::Some(Box::new(Value::Int(BigInt::from(self.new_line)))),
                );
                self.new_line += 1;
            }
            "delete" => {
                f.insert(
                    "old_line".into(),
                    Value::Some(Box::new(Value::Int(BigInt::from(self.old_line)))),
                );
                f.insert("new_line".into(), Value::None);
                self.old_line += 1;
            }
            _ => {
                f.insert(
                    "old_line".into(),
                    Value::Some(Box::new(Value::Int(BigInt::from(self.old_line)))),
                );
                f.insert(
                    "new_line".into(),
                    Value::Some(Box::new(Value::Int(BigInt::from(self.new_line)))),
                );
                self.old_line += 1;
                self.new_line += 1;
            }
        }
        self.lines.push(Value::Record(Arc::new(f)));
    }

    fn to_value(&self) -> Value {
        let mut f = IndexMap::new();
        f.insert("old_start".into(), Value::Int(BigInt::from(self.old_start)));
        f.insert("old_count".into(), Value::Int(BigInt::from(self.old_count)));
        f.insert("new_start".into(), Value::Int(BigInt::from(self.new_start)));
        f.insert("new_count".into(), Value::Int(BigInt::from(self.new_count)));
        f.insert("header".into(), str_val(&self.header));
        f.insert("lines".into(), Value::List(Arc::new(self.lines.clone())));
        Value::Record(Arc::new(f))
    }
}

fn parse_hunk_header(line: &str) -> HunkState {
    let mut old_start = 1i64;
    let mut old_count = 1i64;
    let mut new_start = 1i64;
    let mut new_count = 1i64;
    if let Some(rest) = line.strip_prefix("@@ -").and_then(|r| r.split_once(" +")) {
        parse_range(rest.0, &mut old_start, &mut old_count);
        if let Some((new_part, _)) = rest.1.split_once(" @@") {
            parse_range(new_part, &mut new_start, &mut new_count);
        }
    }
    HunkState {
        old_start,
        old_count,
        new_start,
        new_count,
        header: line.to_string(),
        lines: Vec::new(),
        old_line: old_start,
        new_line: new_start,
    }
}

fn parse_range(s: &str, start: &mut i64, count: &mut i64) {
    if let Some((a, b)) = s.split_once(',') {
        *start = a.parse().unwrap_or(1);
        *count = b.parse().unwrap_or(1);
    } else {
        *start = s.parse().unwrap_or(1);
        *count = 1;
    }
}
