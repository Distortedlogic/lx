#[path = "test_invoke.rs"]
mod test_invoke;
#[path = "test_report.rs"]
mod test_report;
#[path = "test_run.rs"]
mod test_run;

use std::sync::Arc;

use indexmap::IndexMap;

use crate::builtins::mk;
use crate::error::LxError;
use crate::record;
use crate::runtime::RuntimeCtx;
use crate::span::Span;
use crate::value::LxVal;

pub fn build() -> IndexMap<String, LxVal> {
  let mut m = IndexMap::new();
  m.insert("spec".into(), mk("test.spec", 2, bi_spec));
  m.insert("scenario".into(), mk("test.scenario", 3, bi_scenario));
  m.insert("run".into(), mk("test.run", 1, test_run::bi_run));
  m.insert("run_scenario".into(), mk("test.run_scenario", 2, test_run::bi_run_scenario));
  m.insert("report".into(), mk("test.report", 1, test_report::bi_report));
  m
}

pub(super) fn extract_record<'a>(v: &'a LxVal, name: &str, span: Span) -> Result<&'a IndexMap<String, LxVal>, LxError> {
  match v {
    LxVal::Record(r) => Ok(r.as_ref()),
    _ => Err(LxError::type_err(format!("{name}: expected Record, got {}", v.type_name()), span)),
  }
}

pub(super) fn extract_str<'a>(r: &'a IndexMap<String, LxVal>, key: &str, name: &str, span: Span) -> Result<&'a str, LxError> {
  r.get(key).and_then(|v| v.as_str()).ok_or_else(|| LxError::type_err(format!("{name}: '{key}' must be Str"), span))
}

pub(super) fn score_to_f64(v: &LxVal) -> Option<f64> {
  match v {
    LxVal::Float(f) => Some(*f),
    LxVal::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
    LxVal::Int(n) => i64::try_from(n).ok().map(|n| n as f64),
    _ => None,
  }
}

fn bi_spec(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let name = args[0].require_str("test.spec", span)?;
  let opts = extract_record(&args[1], "test.spec", span)?;

  let flow = opts.get("flow").ok_or_else(|| LxError::type_err("test.spec: 'flow' is required", span))?.clone();
  if flow.as_str().is_none() {
    return Err(LxError::type_err("test.spec: 'flow' must be Str", span));
  }

  let grader = opts.get("grader").ok_or_else(|| LxError::type_err("test.spec: 'grader' is required", span))?.clone();

  let threshold = opts.get("threshold").and_then(|v| v.as_float()).unwrap_or(0.75);

  let weights = opts.get("weights").cloned().unwrap_or_else(|| LxVal::record(IndexMap::new()));

  let setup = opts.get("setup").cloned().unwrap_or(LxVal::None);
  let teardown = opts.get("teardown").cloned().unwrap_or(LxVal::None);

  let timeout = opts.get("timeout").and_then(|v| v.as_int()).and_then(|n| i64::try_from(n).ok()).unwrap_or(300);

  let mut m = IndexMap::new();
  m.insert("name".into(), LxVal::str(name));
  m.insert("flow".into(), flow);
  m.insert("grader".into(), grader);
  m.insert("threshold".into(), LxVal::Float(threshold));
  m.insert("weights".into(), weights);
  m.insert("setup".into(), setup);
  m.insert("teardown".into(), teardown);
  m.insert("timeout".into(), LxVal::int(timeout));
  m.insert("scenarios".into(), LxVal::list(Vec::new()));
  Ok(LxVal::record(m))
}

fn bi_scenario(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let spec_fields = extract_record(&args[0], "test.scenario", span)?;
  let scenario_name = args[1].require_str("test.scenario", span)?;
  let opts = extract_record(&args[2], "test.scenario", span)?;

  let input = opts.get("input").ok_or_else(|| LxError::type_err("test.scenario: 'input' is required", span))?.clone();

  let rubric = opts.get("rubric").cloned().unwrap_or_else(|| LxVal::list(Vec::new()));
  let runs = opts.get("runs").and_then(|v| v.as_int()).and_then(|n| i64::try_from(n).ok()).unwrap_or(3);
  let expect = opts.get("expect").cloned().unwrap_or(LxVal::None);
  let tags = opts.get("tags").cloned().unwrap_or_else(|| LxVal::list(Vec::new()));

  let scenario = record! {
      "name" => LxVal::str(scenario_name),
      "input" => input,
      "rubric" => rubric,
      "runs" => LxVal::int(runs),
      "expect" => expect,
      "tags" => tags,
  };

  let mut new_spec = spec_fields.clone();
  let scenarios = match new_spec.get("scenarios") {
    Some(LxVal::List(list)) => {
      let mut new_list = list.as_ref().clone();
      new_list.push(scenario);
      LxVal::list(new_list)
    },
    _ => LxVal::list(vec![scenario]),
  };
  new_spec.insert("scenarios".into(), scenarios);
  Ok(LxVal::record(new_spec))
}
