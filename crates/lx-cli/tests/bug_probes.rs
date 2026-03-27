use std::process::Command;

fn run_lx(code: &str) -> (bool, String, String) {
  let output = Command::new(env!("CARGO_BIN_EXE_lx"))
    .args(["run", "/dev/stdin"])
    .stdin(std::process::Stdio::piped())
    .stdout(std::process::Stdio::piped())
    .stderr(std::process::Stdio::piped())
    .spawn()
    .and_then(|mut child| {
      use std::io::Write;
      child.stdin.take().expect("stdin handle").write_all(code.as_bytes()).expect("write to stdin");
      child.wait_with_output()
    })
    .expect("failed to run lx");
  let stdout = String::from_utf8_lossy(&output.stdout).to_string();
  let stderr = String::from_utf8_lossy(&output.stderr).to_string();
  (output.status.success(), stdout, stderr)
}

// --- FIXED: these pass and guard against regressions ---

#[test]
fn fixed_tuple_negative_index() {
  let (ok, _, _) = run_lx("t = (10; 20; 30)\nassert (t.[-1] == 30) \"negative index\"");
  assert!(ok, "tuple negative indexing works");
}

#[test]
fn fixed_coalesce_pipe_precedence() {
  let (ok, _, _) = run_lx("entry = {field: None}\nresult = entry.field ?? \"HELLO\" | lower\nassert (result == \"hello\") \"coalesce then pipe\"");
  assert!(ok, "?? binds tighter than |");
}

#[test]
fn fixed_not_returns_bool() {
  let (ok, _, _) = run_lx("result = not true\nassert (result == false) \"not true is false\"");
  assert!(ok, "not returns Bool");
}

#[test]
fn fixed_assert_greedy_callable() {
  let (ok, _, _) = run_lx("done = true\nassert done \"should pass\"");
  assert!(ok, "assert no longer consumes message as call argument");
}

#[test]
fn fixed_multiline_ternary() {
  let (ok, stdout, _) = run_lx("x = 2\nresult = x == 1 ? \"one\"\n: x == 2 ? \"two\"\n: \"other\"\nemit result");
  assert!(ok, "multi-line ternary chains parse");
  assert!(stdout.contains("two"));
}

#[test]
fn fixed_shorthand_before_keyed() {
  let (ok, stdout, _) = run_lx("steps = [1; 2; 3]\ntask = \"do it\"\nr = {steps; task; step_count: steps | len}\nemit r.step_count");
  assert!(ok, "shorthand fields before keyed fields parse");
  assert!(stdout.contains("3"));
}

#[test]
fn fixed_spread_shorthand() {
  let (ok, stdout, _) = run_lx("entry = {name: \"a\"; value: 1}\nscore = 100\nr = {..entry; score}\nemit r.score");
  assert!(ok, "spread + shorthand field works");
  assert!(stdout.contains("100"));
}

#[test]
fn fixed_unit_before_closure_param() {
  let (ok, stdout, _) = run_lx("f = (name input fn) { fn input }\nresult = f \"test\" () (x) { \"got: {x}\" }\nemit result");
  assert!(ok, "() before (param) {{ body }} parses correctly");
  assert!(stdout.contains("got:"));
}

#[test]
fn fixed_sections_equality() {
  let (ok, stdout, _) =
    run_lx("records = [{status: \"pass\"; score: 90}; {status: \"fail\"; score: 40}]\npassed = records | filter (.status == \"pass\")\nemit (passed | len)");
  assert!(ok, "sections support == operator");
  assert!(stdout.contains("1"));
}

#[test]
fn fixed_screaming_case_constants() {
  let (ok, stdout, _) = run_lx("TARGET_GRADE = 93\nemit TARGET_GRADE");
  assert!(ok, "SCREAMING_CASE identifiers work");
  assert!(stdout.contains("93"));
}

#[test]
fn fixed_uppercase_keyword_field_names() {
  let (ok, stdout, _) = run_lx("r = {Agent: 1; Tool: 2}\nemit r.Agent");
  assert!(ok, "uppercase keyword field names work");
  assert!(stdout.contains("1"));
}

#[test]
fn fixed_pipe_plus_precedence() {
  let (ok, _, _) = run_lx("result = [1; 2; 3] | len + [4; 5] | len\nassert (result == 5) \"pipe then plus\"");
  assert!(ok, "| binds tighter than +");
}

#[test]
fn fixed_find_returns_value() {
  let (ok, _, _) = run_lx("list = [1; 2; 3; 4; 5]\nfound = list | find (x) { x > 3 }\nassert (found == 4) \"find returns value\"");
  assert!(ok, "find returns value directly, not Some(value)");
}

#[test]
fn fixed_first_returns_value() {
  let (ok, _, _) = run_lx("list = [1; 2; 3]\nfirst_val = list | first\nassert (first_val == 1) \"first returns value\"");
  assert!(ok, "first returns value directly, not Some(value)");
}

#[test]
fn fixed_last_returns_value() {
  let (ok, _, _) = run_lx("list = [1; 2; 3]\nlast_val = list | last\nassert (last_val == 3) \"last returns value\"");
  assert!(ok, "last returns value directly, not Some(value)");
}

#[test]
fn fixed_parens_not_blocks() {
  let (ok, _, _) = run_lx("result = true ? ( x = 10; x + 5 ) : 0\nassert (result == 15) \"parens as block\"");
  assert!(ok, "parens work as block scope");
}
