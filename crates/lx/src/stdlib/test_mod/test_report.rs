use std::sync::Arc;

use indexmap::IndexMap;

use crate::error::LxError;
use crate::runtime::RuntimeCtx;
use crate::value::LxVal;
use miette::SourceSpan;

use super::{extract_record, score_to_f64};

pub(crate) fn bi_report(args: &[LxVal], span: SourceSpan, ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let results = extract_record(&args[0], "test.report", span)?;
  let spec_name = results.get(&crate::sym::intern("spec")).and_then(|v| v.as_str()).unwrap_or("unnamed");
  let threshold = results.get(&crate::sym::intern("threshold")).and_then(|v| v.as_float()).unwrap_or(0.75);
  let scenarios = if let Some(LxVal::List(list)) = results.get(&crate::sym::intern("scenarios")) { list.as_ref().clone() } else { Vec::new() };

  let mut out = String::new();
  out.push_str(spec_name);
  out.push('\n');
  let mut passed_count = 0usize;
  let total = scenarios.len();

  for scenario_val in &scenarios {
    let LxVal::Record(sr) = scenario_val else {
      continue;
    };
    format_scenario(sr, &mut out, &mut passed_count);
  }

  let spec_score = results.get(&crate::sym::intern("score")).and_then(|v| v.as_float()).unwrap_or(0.0);
  out.push_str(&format!("\nOverall: {spec_score:.2} — {passed_count}/{total} scenarios passed (threshold: {threshold:.2})\n"));

  println!("{out}");
  let mut fields = indexmap::IndexMap::new();
  fields.insert(crate::sym::intern("value"), LxVal::str(&out));
  ctx.event_stream.xadd("runtime/emit", "main", None, fields);
  Ok(LxVal::Unit)
}

fn format_scenario(sr: &IndexMap<crate::sym::Sym, LxVal>, out: &mut String, passed_count: &mut usize) {
  let s_name = sr.get(&crate::sym::intern("name")).and_then(|v| v.as_str()).unwrap_or("?");
  let s_score = sr.get(&crate::sym::intern("score")).and_then(|v| v.as_float()).unwrap_or(0.0);
  let s_passed = sr.get(&crate::sym::intern("passed")).and_then(|v| v.as_bool()).unwrap_or(false);
  let s_mean = sr.get(&crate::sym::intern("mean")).and_then(|v| v.as_float()).unwrap_or(s_score);
  let s_min = sr.get(&crate::sym::intern("min")).and_then(|v| v.as_float()).unwrap_or(s_score);
  let s_max = sr.get(&crate::sym::intern("max")).and_then(|v| v.as_float()).unwrap_or(s_score);
  let runs = if let Some(LxVal::List(l)) = sr.get(&crate::sym::intern("runs")) { l.len() } else { 0 };
  if s_passed {
    *passed_count += 1;
  }
  let status = if s_passed { "PASS" } else { "FAIL" };
  let dots_len = 40usize.saturating_sub(s_name.len());
  let dots = ".".repeat(dots_len.max(2));
  out.push_str(&format!("  {s_name} {dots} {s_score:.2} {status} ({runs} runs, mean {s_mean:.2}, min {s_min:.2}, max {s_max:.2})\n"));

  if let Some(LxVal::List(run_list)) = sr.get(&crate::sym::intern("runs"))
    && !run_list.is_empty()
  {
    format_dimensions(run_list, out);
  }
}

fn format_dimensions(run_list: &[LxVal], out: &mut String) {
  let mut dim_scores: IndexMap<crate::sym::Sym, Vec<f64>> = IndexMap::new();
  for run_val in run_list.iter() {
    if let LxVal::Record(rr) = run_val
      && let Some(LxVal::Record(scores)) = rr.get(&crate::sym::intern("scores"))
    {
      for (dim, val) in scores.iter() {
        dim_scores.entry(*dim).or_default().push(score_to_f64(val).unwrap_or(0.0));
      }
    }
  }
  for (dim, vals) in &dim_scores {
    let d_mean = vals.iter().sum::<f64>() / vals.len() as f64;
    let d_min = vals.iter().cloned().fold(f64::INFINITY, f64::min);
    let d_max = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let padding = 14usize.saturating_sub(dim.as_str().len());
    let pad = " ".repeat(padding);
    out.push_str(&format!("    {dim}:{pad} {d_mean:.2} ({d_min:.2}-{d_max:.2})\n"));
  }
}
