use std::sync::Arc;
use std::time::Instant;

use indexmap::IndexMap;

use crate::builtins::call_value_sync;
use lx_value::BuiltinCtx;
use lx_value::LxError;
use lx_value::LxVal;
use lx_value::record;
use miette::SourceSpan;

use super::test_invoke::invoke_flow;
use super::{extract_record, score_to_f64};
use crate::stdlib::helpers::require_str_field;

fn compute_weighted_score(scores: &IndexMap<lx_span::sym::Sym, LxVal>, weights: &IndexMap<lx_span::sym::Sym, LxVal>) -> f64 {
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
    let w = weights.get(dim).and_then(|v| v.as_float().or_else(|| v.as_int().and_then(|n| i64::try_from(n).ok()).map(|n| n as f64))).unwrap_or(1.0);
    weighted_sum += s * w;
    total_weight += w;
  }
  if total_weight == 0.0 { 0.0 } else { weighted_sum / total_weight }
}

fn filter_by_tag(scenarios: &[LxVal]) -> Vec<&LxVal> {
  let tag = std::env::var("LX_TEST_TAG").unwrap_or_default();
  if tag.is_empty() {
    return scenarios.iter().collect();
  }
  scenarios
    .iter()
    .filter(|s| {
      matches!(s, LxVal::Record(r)
            if matches!(r.get(&lx_span::sym::intern("tags")), Some(LxVal::List(tags))
                if tags.iter().any(|t| t.as_str().is_some_and(|s| s == tag))))
    })
    .collect()
}

struct RunCtx<'a> {
  flow_path: &'a str,
  grader: &'a LxVal,
  weights: &'a IndexMap<lx_span::sym::Sym, LxVal>,
  threshold: f64,
  setup: Option<&'a LxVal>,
  teardown: Option<&'a LxVal>,
  span: SourceSpan,
  ctx: &'a Arc<dyn BuiltinCtx>,
}

fn run_one_scenario(scenario_val: &LxVal, rc: &RunCtx<'_>) -> Result<LxVal, LxError> {
  let span = rc.span;
  let scenario_fields = extract_record(scenario_val, "test.run", span)?;
  let s_name = require_str_field(scenario_fields, "name", "test.run", span)?;
  let input = scenario_fields.get(&lx_span::sym::intern("input")).cloned().unwrap_or(LxVal::Unit);
  let runs: i64 = scenario_fields.get(&lx_span::sym::intern("runs")).and_then(|v| v.as_int()).and_then(|n| i64::try_from(n).ok()).unwrap_or(3);

  let mut run_results = Vec::new();
  let mut run_scores = Vec::new();

  for _ in 0..runs {
    if let Some(setup_fn) = rc.setup {
      call_value_sync(setup_fn, scenario_val.clone(), span, rc.ctx)?;
    }
    let start = Instant::now();
    let output = invoke_flow(rc.flow_path, &input, rc.ctx, span)?;
    let elapsed_ms = start.elapsed().as_millis() as i64;

    let grader_input = LxVal::tuple(vec![output.clone(), scenario_val.clone()]);
    let scores_val = call_value_sync(rc.grader, grader_input, span, rc.ctx)?;
    let LxVal::Record(r) = &scores_val else {
      return Err(LxError::runtime(format!("test.run: grader must return Record, got {}", scores_val.type_name()), span));
    };
    let scores_map = r.as_ref().clone();

    let weighted = compute_weighted_score(&scores_map, rc.weights);
    run_scores.push(weighted);
    run_results.push(record! {
        "scores" => scores_val,
        "weighted" => LxVal::Float(weighted),
        "output" => output,
        "elapsed_ms" => LxVal::int(elapsed_ms),
    });
    if let Some(teardown_fn) = rc.teardown {
      call_value_sync(teardown_fn, scenario_val.clone(), span, rc.ctx)?;
    }
  }

  let mean = if run_scores.is_empty() { 0.0 } else { run_scores.iter().sum::<f64>() / run_scores.len() as f64 };
  let min = run_scores.iter().cloned().fold(f64::INFINITY, f64::min);
  let max = run_scores.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
  let passed = mean >= rc.threshold;

  Ok(record! {
      "name" => LxVal::str(s_name),
      "passed" => LxVal::Bool(passed),
      "score" => LxVal::Float(mean),
      "runs" => LxVal::list(run_results),
      "mean" => LxVal::Float(mean),
      "min" => LxVal::Float(if min.is_infinite() { 0.0 } else { min }),
      "max" => LxVal::Float(if max.is_infinite() { 0.0 } else { max }),
  })
}

fn run_scenarios(spec_fields: &IndexMap<lx_span::sym::Sym, LxVal>, scenarios: &[LxVal], span: SourceSpan, ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  let flow_path = require_str_field(spec_fields, "flow", "test.run", span)?.to_string();
  let grader = spec_fields.get(&lx_span::sym::intern("grader")).ok_or_else(|| LxError::runtime("test.run: spec missing 'grader'", span))?;
  let threshold = spec_fields.get(&lx_span::sym::intern("threshold")).and_then(|v| v.as_float()).unwrap_or(0.75);
  let weights_map = if let Some(LxVal::Record(r)) = spec_fields.get(&lx_span::sym::intern("weights")) { r.as_ref().clone() } else { IndexMap::new() };
  let setup = spec_fields.get(&lx_span::sym::intern("setup")).filter(|v| !matches!(v, LxVal::None));
  let teardown = spec_fields.get(&lx_span::sym::intern("teardown")).filter(|v| !matches!(v, LxVal::None));
  let spec_name = spec_fields.get(&lx_span::sym::intern("name")).and_then(|v| v.as_str()).unwrap_or("unnamed");

  let rc = RunCtx { flow_path: &flow_path, grader, weights: &weights_map, threshold, setup, teardown, span, ctx };
  let filtered = filter_by_tag(scenarios);
  let mut scenario_results = Vec::new();
  let mut all_passed = true;

  for scenario_val in &filtered {
    let result = run_one_scenario(scenario_val, &rc)?;
    if let LxVal::Record(r) = &result
      && r.get(&lx_span::sym::intern("passed")).and_then(|v| v.as_bool()) != Some(true)
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
      .filter_map(|s| if let LxVal::Record(r) = s { r.get(&lx_span::sym::intern("score")).and_then(|v| v.as_float()) } else { None })
      .sum();
    sum / scenario_results.len() as f64
  };

  Ok(LxVal::ok(record! {
      "spec" => LxVal::str(spec_name),
      "passed" => LxVal::Bool(all_passed),
      "score" => LxVal::Float(spec_score),
      "threshold" => LxVal::Float(threshold),
      "scenarios" => LxVal::list(scenario_results),
  }))
}

pub(crate) fn bi_run(args: &[LxVal], span: SourceSpan, ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  let spec_fields = extract_record(&args[0], "test.run", span)?;
  let scenarios = if let Some(LxVal::List(list)) = spec_fields.get(&lx_span::sym::intern("scenarios")) { list.as_ref().clone() } else { Vec::new() };
  run_scenarios(spec_fields, &scenarios, span, ctx)
}

pub(crate) fn bi_run_scenario(args: &[LxVal], span: SourceSpan, ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  let spec_fields = extract_record(&args[0], "test.run_scenario", span)?;
  let target_name = args[1].require_str("test.run_scenario", span)?;
  let scenarios = if let Some(LxVal::List(list)) = spec_fields.get(&lx_span::sym::intern("scenarios")) { list.as_ref().clone() } else { Vec::new() };
  let matching: Vec<LxVal> = scenarios
    .into_iter()
    .filter(|s| match s {
      LxVal::Record(r) => r.get(&lx_span::sym::intern("name")).and_then(|v| v.as_str()).is_some_and(|n| n == target_name),
      _ => false,
    })
    .collect();
  if matching.is_empty() {
    return Err(LxError::runtime(format!("test.run_scenario: scenario not found: '{target_name}'"), span));
  }
  run_scenarios(spec_fields, &matching, span, ctx)
}
