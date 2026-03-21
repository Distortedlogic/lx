use std::sync::Arc;

use indexmap::IndexMap;
use num_bigint::BigInt;

use crate::backends::RuntimeCtx;
use crate::builtins::mk;
use crate::error::LxError;
use crate::record;
use crate::span::Span;
use crate::value::Value;

pub fn build() -> IndexMap<String, Value> {
    let mut m = IndexMap::new();
    m.insert("check".into(), mk("monitor.check", 1, bi_check));
    m.insert(
        "scan_actions".into(),
        mk("monitor.scan_actions", 1, bi_scan_actions),
    );
    m
}

const INJECTION_PATTERNS: &[&str] = &[
    "ignore previous instructions",
    "disregard all",
    "you are now",
    "new instructions:",
    "system prompt:",
    "override:",
    "<script>",
    "eval(",
    "exec(",
];

const STUCK_THRESHOLD: usize = 3;

fn detect_issues(actions: &[Value]) -> Vec<(String, String, String)> {
    let mut issues = Vec::new();
    let strs: Vec<String> = actions
        .iter()
        .filter_map(|v| match v {
            Value::Str(s) => Some(s.to_string()),
            Value::Record(r) => r
                .get("action")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            _ => None,
        })
        .collect();
    if strs.len() >= STUCK_THRESHOLD {
        let last_n = &strs[strs.len().saturating_sub(STUCK_THRESHOLD)..];
        if last_n.windows(2).all(|w| w[0] == w[1]) {
            issues.push((
                "stuck_loop".into(),
                "critical".into(),
                format!("Repeated action {} times: {}", STUCK_THRESHOLD, last_n[0]),
            ));
        }
    }
    for (i, action) in strs.iter().enumerate() {
        let lower = action.to_lowercase();
        for pattern in INJECTION_PATTERNS {
            if lower.contains(pattern) {
                issues.push((
                    "injection".into(),
                    "critical".into(),
                    format!("Suspicious pattern '{pattern}' in action {i}"),
                ));
                break;
            }
        }
    }
    if strs.len() > 50 {
        issues.push((
            "resource_abuse".into(),
            "warning".into(),
            format!("Excessive actions: {} (threshold: 50)", strs.len()),
        ));
    }
    issues
}

fn make_issue(kind: &str, severity: &str, detail: &str) -> Value {
    record! {
        "kind" => Value::Str(Arc::from(kind)),
        "severity" => Value::Str(Arc::from(severity)),
        "detail" => Value::Str(Arc::from(detail)),
    }
}

fn build_result(issues: &[(String, String, String)]) -> Value {
    let issue_vals: Vec<Value> = issues.iter().map(|(k, s, d)| make_issue(k, s, d)).collect();
    let has_critical = issues.iter().any(|(_, s, _)| s == "critical");
    record! {
        "ok" => Value::Bool(issues.is_empty()),
        "issues" => Value::List(Arc::new(issue_vals)),
        "issue_count" => Value::Int(BigInt::from(issues.len() as i64)),
        "critical" => Value::Bool(has_critical),
    }
}

fn bi_check(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let Value::Record(fields) = &args[0] else {
        return Err(LxError::type_err("monitor.check expects Record", span));
    };
    let actions = fields
        .get("actions")
        .and_then(|v| v.as_list())
        .ok_or_else(|| LxError::runtime("monitor.check: missing 'actions' (List)", span))?;
    let issues = detect_issues(actions);
    Ok(build_result(&issues))
}

fn bi_scan_actions(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let Value::List(actions) = &args[0] else {
        return Err(LxError::type_err("monitor.scan_actions expects List", span));
    };
    let issues = detect_issues(actions);
    Ok(build_result(&issues))
}
