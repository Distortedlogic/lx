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
            child.stdin.take().unwrap().write_all(code.as_bytes()).unwrap();
            child.wait_with_output()
        })
        .expect("failed to run lx");
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    (output.status.success(), stdout, stderr)
}

#[test]
fn bug_named_arg_ternary_colon() {
    let (ok, stdout, _) = run_lx(r#"
        id = (x) x
        result = id 42 key: "v" ? "alt" : "nope"
        emit result
    "#);
    assert!(ok, "BUG-FIXED: named-arg parser no longer consumes ternary :");
    assert!(stdout.contains("42"), "should emit the id result");
}

#[test]
fn bug_assert_greedy_callable() {
    let (ok, _, _) = run_lx(r#"
        done = true
        assert done "should pass"
    "#);
    assert!(ok, "BUG-FIXED: assert no longer consumes message as call argument");
}

#[test]
fn bug_multiline_ternary() {
    let (ok, stdout, _) = run_lx(r#"
        x = 2
        result = x == 1 ? "one"
        : x == 2 ? "two"
        : "other"
        emit result
    "#);
    assert!(ok, "BUG-FIXED: multi-line ternary chains parse");
    assert!(stdout.contains("two"), "should match second branch");
}

#[test]
fn bug_shorthand_before_keyed() {
    let (ok, stdout, _) = run_lx(r#"
        steps = [1; 2; 3]
        task = "do it"
        r = {steps  task  step_count: steps | len}
        emit r.step_count
    "#);
    assert!(ok, "BUG-FIXED: shorthand fields before keyed fields parse");
    assert!(stdout.contains("3"), "step_count should be 3");
}

#[test]
fn bug_spread_shorthand() {
    let (ok, stdout, _) = run_lx(r#"
        entry = {name: "a"; value: 1}
        score = 100
        r = {..entry  score}
        emit r.score
    "#);
    assert!(ok, "BUG-FIXED: spread + shorthand field works");
    assert!(stdout.contains("100"), "score should be 100");
}

#[test]
fn bug_unit_before_closure_param() {
    let (ok, stdout, _) = run_lx(r#"
        f = (name input fn) { fn input }
        result = f "test" () (x) { "got: {x}" }
        emit result
    "#);
    assert!(ok, "BUG-FIXED: () before (param) {{ body }} parses correctly");
    assert!(stdout.contains("got: ()"), "closure should receive Unit input");
}

#[test]
fn bug_sections_equality() {
    let (ok, stdout, _) = run_lx(r#"
        records = [{status: "pass"; score: 90}; {status: "fail"; score: 40}]
        passed = records | filter (.status == "pass")
        emit (passed | len)
    "#);
    assert!(ok, "BUG-FIXED: sections support == operator");
    assert!(stdout.contains("1"), "should find 1 passing record");
}

#[test]
fn bug_screaming_case_constants() {
    let (ok, stdout, _) = run_lx(r#"
        TARGET_GRADE = 93
        emit TARGET_GRADE
    "#);
    assert!(ok, "BUG-FIXED: SCREAMING_CASE identifiers work");
    assert!(stdout.contains("93"), "should emit 93");
}

#[test]
fn bug_uppercase_keyword_field_names() {
    let (ok, stdout, _) = run_lx(r#"
        r = {Agent: 1; Tool: 2}
        emit r.Agent
    "#);
    assert!(ok, "BUG-FIXED: uppercase keyword field names work");
    assert!(stdout.contains("1"), "should emit 1");
}
