use std::sync::Arc;

use indexmap::IndexMap;

use crate::backends::RuntimeCtx;
use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

use super::{extract_record, score_to_f64};

pub(crate) fn bi_report(
    args: &[Value],
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let results = extract_record(&args[0], "test.report", span)?;
    let spec_name = results
        .get("spec")
        .and_then(|v| v.as_str())
        .unwrap_or("unnamed");
    let threshold = results
        .get("threshold")
        .and_then(|v| v.as_float())
        .unwrap_or(0.75);
    let scenarios = match results.get("scenarios") {
        Some(Value::List(list)) => list.as_ref().clone(),
        _ => Vec::new(),
    };

    let mut out = String::new();
    out.push_str(spec_name);
    out.push('\n');
    let mut passed_count = 0usize;
    let total = scenarios.len();

    for scenario_val in &scenarios {
        let Value::Record(sr) = scenario_val else {
            continue;
        };
        format_scenario(sr, &mut out, &mut passed_count);
    }

    let spec_score = results
        .get("score")
        .and_then(|v| v.as_float())
        .unwrap_or(0.0);
    out.push_str(&format!(
        "\nOverall: {spec_score:.2} — {passed_count}/{total} scenarios passed (threshold: {threshold:.2})\n"
    ));

    ctx.emit.emit(&Value::Str(Arc::from(out.as_str())), span)?;
    Ok(Value::Unit)
}

fn format_scenario(sr: &IndexMap<String, Value>, out: &mut String, passed_count: &mut usize) {
    let s_name = sr.get("name").and_then(|v| v.as_str()).unwrap_or("?");
    let s_score = sr.get("score").and_then(|v| v.as_float()).unwrap_or(0.0);
    let s_passed = sr.get("passed").and_then(|v| v.as_bool()).unwrap_or(false);
    let s_mean = sr.get("mean").and_then(|v| v.as_float()).unwrap_or(s_score);
    let s_min = sr.get("min").and_then(|v| v.as_float()).unwrap_or(s_score);
    let s_max = sr.get("max").and_then(|v| v.as_float()).unwrap_or(s_score);
    let runs = match sr.get("runs") {
        Some(Value::List(l)) => l.len(),
        _ => 0,
    };
    if s_passed {
        *passed_count += 1;
    }
    let status = if s_passed { "PASS" } else { "FAIL" };
    let dots_len = 40usize.saturating_sub(s_name.len());
    let dots = ".".repeat(dots_len.max(2));
    out.push_str(&format!(
        "  {s_name} {dots} {s_score:.2} {status} ({runs} runs, mean {s_mean:.2}, min {s_min:.2}, max {s_max:.2})\n"
    ));

    if let Some(Value::List(run_list)) = sr.get("runs")
        && !run_list.is_empty()
    {
        format_dimensions(run_list, out);
    }
}

fn format_dimensions(run_list: &[Value], out: &mut String) {
    let mut dim_scores: IndexMap<String, Vec<f64>> = IndexMap::new();
    for run_val in run_list.iter() {
        if let Value::Record(rr) = run_val
            && let Some(Value::Record(scores)) = rr.get("scores")
        {
            for (dim, val) in scores.iter() {
                dim_scores
                    .entry(dim.clone())
                    .or_default()
                    .push(score_to_f64(val).unwrap_or(0.0));
            }
        }
    }
    for (dim, vals) in &dim_scores {
        let d_mean = vals.iter().sum::<f64>() / vals.len() as f64;
        let d_min = vals.iter().cloned().fold(f64::INFINITY, f64::min);
        let d_max = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let padding = 14usize.saturating_sub(dim.len());
        let pad = " ".repeat(padding);
        out.push_str(&format!(
            "    {dim}:{pad} {d_mean:.2} ({d_min:.2}-{d_max:.2})\n"
        ));
    }
}
