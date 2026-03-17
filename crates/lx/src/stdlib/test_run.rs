use std::sync::Arc;
use std::time::Instant;

use indexmap::IndexMap;
use num_bigint::BigInt;

use crate::backends::RuntimeCtx;
use crate::builtins::call_value;
use crate::error::LxError;
use crate::record;
use crate::span::Span;
use crate::value::Value;

use super::test_invoke::invoke_flow;
use super::{extract_record, extract_str, score_to_f64};

fn compute_weighted_score(
    scores: &IndexMap<String, Value>,
    weights: &IndexMap<String, Value>,
) -> f64 {
    if scores.is_empty() {
        return 0.0;
    }
    if weights.is_empty() {
        let sum: f64 = scores.values().filter_map(score_to_f64).sum();
        return sum / scores.len() as f64;
    }
    let mut weighted_sum = 0.0;
    let mut total_weight = 0.0;
    for (dim, val) in scores.iter() {
        let s = score_to_f64(val).unwrap_or(0.0);
        let w = weights
            .get(dim)
            .and_then(|v| {
                v.as_float().or_else(|| {
                    v.as_int()
                        .and_then(|n| i64::try_from(n).ok())
                        .map(|n| n as f64)
                })
            })
            .unwrap_or(1.0);
        weighted_sum += s * w;
        total_weight += w;
    }
    if total_weight == 0.0 {
        0.0
    } else {
        weighted_sum / total_weight
    }
}

fn filter_by_tag(scenarios: &[Value]) -> Vec<&Value> {
    let tag = std::env::var("LX_TEST_TAG").unwrap_or_default();
    if tag.is_empty() {
        return scenarios.iter().collect();
    }
    scenarios
        .iter()
        .filter(|s| {
            matches!(s, Value::Record(r)
            if matches!(r.get("tags"), Some(Value::List(tags))
                if tags.iter().any(|t| t.as_str().is_some_and(|s| s == tag))))
        })
        .collect()
}

struct RunCtx<'a> {
    flow_path: &'a str,
    grader: &'a Value,
    weights: &'a IndexMap<String, Value>,
    threshold: f64,
    setup: Option<&'a Value>,
    teardown: Option<&'a Value>,
    span: Span,
    ctx: &'a Arc<RuntimeCtx>,
}

fn run_one_scenario(scenario_val: &Value, rc: &RunCtx<'_>) -> Result<Value, LxError> {
    let span = rc.span;
    let scenario_fields = extract_record(scenario_val, "test.run", span)?;
    let s_name = extract_str(scenario_fields, "name", "test.run", span)?;
    let input = scenario_fields.get("input").cloned().unwrap_or(Value::Unit);
    let runs: i64 = scenario_fields
        .get("runs")
        .and_then(|v| v.as_int())
        .and_then(|n| i64::try_from(n).ok())
        .unwrap_or(3);

    let mut run_results = Vec::new();
    let mut run_scores = Vec::new();

    for _ in 0..runs {
        if let Some(setup_fn) = rc.setup {
            call_value(setup_fn, scenario_val.clone(), span, rc.ctx)?;
        }
        let start = Instant::now();
        let output = invoke_flow(rc.flow_path, &input, rc.ctx, span)?;
        let elapsed_ms = start.elapsed().as_millis() as i64;

        let grader_input = Value::Tuple(Arc::new(vec![output.clone(), scenario_val.clone()]));
        let scores_val = call_value(rc.grader, grader_input, span, rc.ctx)?;
        let scores_map = match &scores_val {
            Value::Record(r) => r.as_ref().clone(),
            _ => {
                return Err(LxError::runtime(
                    format!(
                        "test.run: grader must return Record, got {}",
                        scores_val.type_name()
                    ),
                    span,
                ));
            }
        };

        let weighted = compute_weighted_score(&scores_map, rc.weights);
        run_scores.push(weighted);
        run_results.push(record! {
            "scores" => scores_val,
            "weighted" => Value::Float(weighted),
            "output" => output,
            "elapsed_ms" => Value::Int(BigInt::from(elapsed_ms)),
        });
        if let Some(teardown_fn) = rc.teardown {
            call_value(teardown_fn, scenario_val.clone(), span, rc.ctx)?;
        }
    }

    let mean = if run_scores.is_empty() {
        0.0
    } else {
        run_scores.iter().sum::<f64>() / run_scores.len() as f64
    };
    let min = run_scores.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = run_scores.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let passed = mean >= rc.threshold;

    Ok(record! {
        "name" => Value::Str(Arc::from(s_name)),
        "passed" => Value::Bool(passed),
        "score" => Value::Float(mean),
        "runs" => Value::List(Arc::new(run_results)),
        "mean" => Value::Float(mean),
        "min" => Value::Float(if min.is_infinite() { 0.0 } else { min }),
        "max" => Value::Float(if max.is_infinite() { 0.0 } else { max }),
    })
}

fn run_scenarios(
    spec_fields: &IndexMap<String, Value>,
    scenarios: &[Value],
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let flow_path = extract_str(spec_fields, "flow", "test.run", span)?.to_string();
    let grader = spec_fields
        .get("grader")
        .ok_or_else(|| LxError::runtime("test.run: spec missing 'grader'", span))?;
    let threshold = spec_fields
        .get("threshold")
        .and_then(|v| v.as_float())
        .unwrap_or(0.75);
    let weights_map = match spec_fields.get("weights") {
        Some(Value::Record(r)) => r.as_ref().clone(),
        _ => IndexMap::new(),
    };
    let setup = spec_fields
        .get("setup")
        .filter(|v| !matches!(v, Value::None));
    let teardown = spec_fields
        .get("teardown")
        .filter(|v| !matches!(v, Value::None));
    let spec_name = spec_fields
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("unnamed");

    let rc = RunCtx {
        flow_path: &flow_path,
        grader,
        weights: &weights_map,
        threshold,
        setup,
        teardown,
        span,
        ctx,
    };
    let filtered = filter_by_tag(scenarios);
    let mut scenario_results = Vec::new();
    let mut all_passed = true;

    for scenario_val in &filtered {
        let result = run_one_scenario(scenario_val, &rc)?;
        if let Value::Record(r) = &result
            && r.get("passed").and_then(|v| v.as_bool()) != Some(true)
        {
            all_passed = false;
        }
        scenario_results.push(result);
    }

    let spec_score = if scenario_results.is_empty() {
        1.0
    } else {
        let sum: f64 = scenario_results
            .iter()
            .filter_map(|s| match s {
                Value::Record(r) => r.get("score").and_then(|v| v.as_float()),
                _ => None,
            })
            .sum();
        sum / scenario_results.len() as f64
    };

    Ok(Value::Ok(Box::new(record! {
        "spec" => Value::Str(Arc::from(spec_name)),
        "passed" => Value::Bool(all_passed),
        "score" => Value::Float(spec_score),
        "threshold" => Value::Float(threshold),
        "scenarios" => Value::List(Arc::new(scenario_results)),
    })))
}

pub(crate) fn bi_run(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let spec_fields = extract_record(&args[0], "test.run", span)?;
    let scenarios = match spec_fields.get("scenarios") {
        Some(Value::List(list)) => list.as_ref().clone(),
        _ => Vec::new(),
    };
    run_scenarios(spec_fields, &scenarios, span, ctx)
}

pub(crate) fn bi_run_scenario(
    args: &[Value],
    span: Span,
    ctx: &Arc<RuntimeCtx>,
) -> Result<Value, LxError> {
    let spec_fields = extract_record(&args[0], "test.run_scenario", span)?;
    let target_name = args[1]
        .as_str()
        .ok_or_else(|| LxError::type_err("test.run_scenario: name must be Str", span))?;
    let scenarios = match spec_fields.get("scenarios") {
        Some(Value::List(list)) => list.as_ref().clone(),
        _ => Vec::new(),
    };
    let matching: Vec<Value> = scenarios
        .into_iter()
        .filter(|s| match s {
            Value::Record(r) => r
                .get("name")
                .and_then(|v| v.as_str())
                .is_some_and(|n| n == target_name),
            _ => false,
        })
        .collect();
    if matching.is_empty() {
        return Err(LxError::runtime(
            format!("test.run_scenario: scenario not found: '{target_name}'"),
            span,
        ));
    }
    run_scenarios(spec_fields, &matching, span, ctx)
}
