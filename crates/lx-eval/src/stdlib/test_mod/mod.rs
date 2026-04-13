#[path = "test_invoke.rs"]
mod test_invoke;
#[path = "test_report.rs"]
mod test_report;
#[path = "test_run.rs"]
mod test_run;

use indexmap::IndexMap;

use crate::std_module;
use lx_value::BuiltinCtx;
use lx_value::LxError;
use lx_value::LxVal;
use lx_value::record;
use miette::SourceSpan;

pub fn build() -> IndexMap<lx_span::sym::Sym, LxVal> {
  std_module! {
    "spec"         => "test.spec",         2, bi_spec;
    "scenario"     => "test.scenario",     3, bi_scenario;
    "run"          => "test.run",          1, test_run::bi_run;
    "run_scenario" => "test.run_scenario", 2, test_run::bi_run_scenario;
    "report"       => "test.report",       1, test_report::bi_report
  }
}

pub(super) fn extract_record<'a>(v: &'a LxVal, name: &str, span: SourceSpan) -> Result<&'a IndexMap<lx_span::sym::Sym, LxVal>, LxError> {
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

fn bi_spec(args: &[LxVal], span: SourceSpan, _ctx: &dyn BuiltinCtx) -> Result<LxVal, LxError> {
  let name = args[0].require_str("test.spec", span)?;
  let opts = extract_record(&args[1], "test.spec", span)?;

  let flow = opts.get(&lx_span::sym::intern("flow")).ok_or_else(|| LxError::type_err("test.spec: 'flow' is required", span, None))?.clone();
  if flow.as_str().is_none() {
    return Err(LxError::type_err("test.spec: 'flow' must be Str", span, None));
  }

  let grader = opts.get(&lx_span::sym::intern("grader")).ok_or_else(|| LxError::type_err("test.spec: 'grader' is required", span, None))?.clone();

  let threshold = opts.get(&lx_span::sym::intern("threshold")).and_then(|v| v.as_float()).unwrap_or(0.75);

  let weights = opts.get(&lx_span::sym::intern("weights")).cloned().unwrap_or_else(|| LxVal::record(IndexMap::new()));

  let setup = opts.get(&lx_span::sym::intern("setup")).cloned().unwrap_or(LxVal::None);
  let teardown = opts.get(&lx_span::sym::intern("teardown")).cloned().unwrap_or(LxVal::None);

  let timeout = opts.get(&lx_span::sym::intern("timeout")).and_then(|v| v.as_int()).and_then(|n| i64::try_from(n).ok()).unwrap_or(300);

  let mut m = IndexMap::new();
  m.insert(lx_span::sym::intern("name"), LxVal::str(name));
  m.insert(lx_span::sym::intern("flow"), flow);
  m.insert(lx_span::sym::intern("grader"), grader);
  m.insert(lx_span::sym::intern("threshold"), LxVal::Float(threshold));
  m.insert(lx_span::sym::intern("weights"), weights);
  m.insert(lx_span::sym::intern("setup"), setup);
  m.insert(lx_span::sym::intern("teardown"), teardown);
  m.insert(lx_span::sym::intern("timeout"), LxVal::int(timeout));
  m.insert(lx_span::sym::intern("scenarios"), LxVal::list(Vec::new()));
  Ok(LxVal::record(m))
}

fn bi_scenario(args: &[LxVal], span: SourceSpan, _ctx: &dyn BuiltinCtx) -> Result<LxVal, LxError> {
  let spec_fields = extract_record(&args[0], "test.scenario", span)?;
  let scenario_name = args[1].require_str("test.scenario", span)?;
  let opts = extract_record(&args[2], "test.scenario", span)?;

  let input = opts.get(&lx_span::sym::intern("input")).ok_or_else(|| LxError::type_err("test.scenario: 'input' is required", span, None))?.clone();

  let rubric = opts.get(&lx_span::sym::intern("rubric")).cloned().unwrap_or_else(|| LxVal::list(Vec::new()));
  let runs = opts.get(&lx_span::sym::intern("runs")).and_then(|v| v.as_int()).and_then(|n| i64::try_from(n).ok()).unwrap_or(3);
  let expect = opts.get(&lx_span::sym::intern("expect")).cloned().unwrap_or(LxVal::None);
  let tags = opts.get(&lx_span::sym::intern("tags")).cloned().unwrap_or_else(|| LxVal::list(Vec::new()));

  let scenario = record! {
      "name" => LxVal::str(scenario_name),
      "input" => input,
      "rubric" => rubric,
      "runs" => LxVal::int(runs),
      "expect" => expect,
      "tags" => tags,
  };

  let mut new_spec = spec_fields.clone();
  let scenarios = if let Some(LxVal::List(list)) = new_spec.get(&lx_span::sym::intern("scenarios")) {
    let mut new_list = list.as_ref().clone();
    new_list.push(scenario);
    LxVal::list(new_list)
  } else {
    LxVal::list(vec![scenario])
  };
  new_spec.insert(lx_span::sym::intern("scenarios"), scenarios);
  Ok(LxVal::record(new_spec))
}
