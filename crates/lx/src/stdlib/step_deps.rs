use std::collections::HashSet;

use crate::value::Value;

pub(crate) fn step_id(step: &Value) -> Option<&str> {
    match step {
        Value::Record(r) => r.get("id").and_then(|v| v.as_str()),
        _ => None,
    }
}

pub(crate) fn step_deps(step: &Value) -> Vec<String> {
    match step {
        Value::Record(r) => r
            .get("depends")
            .and_then(|v| v.as_list())
            .map(|l| {
                l.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default(),
        _ => vec![],
    }
}

pub(crate) fn next_ready(remaining: &[Value], completed: &HashSet<String>) -> Option<usize> {
    remaining
        .iter()
        .position(|step| step_deps(step).iter().all(|d| completed.contains(d)))
}
