#[path = "test_invoke.rs"]
mod test_invoke;
#[path = "test_report.rs"]
mod test_report;
#[path = "test_run.rs"]
mod test_run;

use std::sync::Arc;

use indexmap::IndexMap;

use crate::error::LxError;
use crate::record;
use crate::runtime::RuntimeCtx;
use crate::std_module;
use crate::value::LxVal;
use miette::SourceSpan;

pub fn build() -> IndexMap<crate::sym::Sym, LxVal> {
  std_module! {
    "spec"         => "test.spec",         2, bi_spec;
    "scenario"     => "test.scenario",     3, bi_scenario;
    "run"          => "test.run",          1, test_run::bi_run;
    "run_scenario" => "test.run_scenario", 2, test_run::bi_run_scenario;
    "report"       => "test.report",       1, test_report::bi_report
  }
}

pub(super) fn extract_record<'a>(v: &'a LxVal, name: &str, span: SourceSpan) -> Result<&'a IndexMap<crate::sym::Sym, LxVal>, LxError> {
  if let LxVal::Record(r) = v { Ok(r.as_ref()) } else { Err(LxError::type_err(format!("{name}: expected Record, got {}", v.type_name()), span, None)) }
}

pub(super) fn score_to_f64(v: &LxVal) -> Option<f64> {
  match v {
    LxVal::Float(f) => Some(*f),
    LxVal::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
    LxVal::Int(n) => i64::try_from(n).ok().map(|n| n as f64),
    _ => None,
  }
}

fn bi_spec(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let name = args[0].require_str("test.spec", span)?;
  let opts = extract_record(&args[1], "test.spec", span)?;

  let flow = opts.get(&crate::sym::intern("flow")).ok_or_else(|| LxError::type_err("test.spec: 'flow' is required", span, None))?.clone();
  if flow.as_str().is_none() {
    return Err(LxError::type_err("test.spec: 'flow' must be Str", span, None));
  }

  let grader = opts.get(&crate::sym::intern("grader")).ok_or_else(|| LxError::type_err("test.spec: 'grader' is required", span, None))?.clone();

  let threshold = opts.get(&crate::sym::intern("threshold")).and_then(|v| v.as_float()).unwrap_or(0.75);

  let weights = opts.get(&crate::sym::intern("weights")).cloned().unwrap_or_else(|| LxVal::record(IndexMap::new()));

  let setup = opts.get(&crate::sym::intern("setup")).cloned().unwrap_or(LxVal::None);
  let teardown = opts.get(&crate::sym::intern("teardown")).cloned().unwrap_or(LxVal::None);

  let timeout = opts.get(&crate::sym::intern("timeout")).and_then(|v| v.as_int()).and_then(|n| i64::try_from(n).ok()).unwrap_or(300);

  let mut m = IndexMap::new();
  m.insert(crate::sym::intern("name"), LxVal::str(name));
  m.insert(crate::sym::intern("flow"), flow);
  m.insert(crate::sym::intern("grader"), grader);
  m.insert(crate::sym::intern("threshold"), LxVal::Float(threshold));
  m.insert(crate::sym::intern("weights"), weights);
  m.insert(crate::sym::intern("setup"), setup);
  m.insert(crate::sym::intern("teardown"), teardown);
  m.insert(crate::sym::intern("timeout"), LxVal::int(timeout));
  m.insert(crate::sym::intern("scenarios"), LxVal::list(Vec::new()));
  Ok(LxVal::record(m))
}

fn bi_scenario(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let spec_fields = extract_record(&args[0], "test.scenario", span)?;
  let scenario_name = args[1].require_str("test.scenario", span)?;
  let opts = extract_record(&args[2], "test.scenario", span)?;

  let input = opts.get(&crate::sym::intern("input")).ok_or_else(|| LxError::type_err("test.scenario: 'input' is required", span, None))?.clone();

  let rubric = opts.get(&crate::sym::intern("rubric")).cloned().unwrap_or_else(|| LxVal::list(Vec::new()));
  let runs = opts.get(&crate::sym::intern("runs")).and_then(|v| v.as_int()).and_then(|n| i64::try_from(n).ok()).unwrap_or(3);
  let expect = opts.get(&crate::sym::intern("expect")).cloned().unwrap_or(LxVal::None);
  let tags = opts.get(&crate::sym::intern("tags")).cloned().unwrap_or_else(|| LxVal::list(Vec::new()));

  let scenario = record! {
      "name" => LxVal::str(scenario_name),
      "input" => input,
      "rubric" => rubric,
      "runs" => LxVal::int(runs),
      "expect" => expect,
      "tags" => tags,
  };

  let mut new_spec = spec_fields.clone();
  let scenarios = if let Some(LxVal::List(list)) = new_spec.get(&crate::sym::intern("scenarios")) {
    let mut new_list = list.as_ref().clone();
    new_list.push(scenario);
    LxVal::list(new_list)
  } else {
    LxVal::list(vec![scenario])
  };
  new_spec.insert(crate::sym::intern("scenarios"), scenarios);
  Ok(LxVal::record(new_spec))
}
