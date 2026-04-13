use std::fs;

use lx_fmt::format;
use lx_parser::lexer::lex;
use lx_parser::parser::parse;
use lx_span::source::FileId;

fn roundtrip_check(path: &str) {
  let source = fs::read_to_string(path).expect("failed to read test file");
  let (tokens, comments) = lex(&source).expect("lex failed");
  let result = parse(tokens, FileId::new(0), comments, &source);
  let program = result.program.unwrap_or_else(|| panic!("parse failed for {path}"));
  let formatted = format(&program);

  let (tokens2, comments2) = lex(&formatted).expect("re-lex failed");
  let result2 = parse(tokens2, FileId::new(0), comments2, &formatted);
  assert!(result2.program.is_some(), "round-trip failed for {path}: formatted output does not re-parse.\nFormatted:\n{formatted}");
}

#[test]
fn formatter_roundtrips_test_files() {
  for entry in fs::read_dir("../../tests").unwrap() {
    let entry = entry.unwrap();
    let path = entry.path();
    if path.extension().map(|e| e == "lx").unwrap_or(false) {
      roundtrip_check(path.to_str().unwrap());
    }
  }
}

#[test]
fn formatter_emits_record_shorthand() {
  let cases = vec![
    ("x = 1\nr = {x: x; y: 2}\n", "{ x; y: 2 }"),
    ("a = 1\nb = 2\nr = {a: a; b: b}\n", "{ a; b }"),
    ("r = {name: \"alice\"}\n", "{ name: \"alice\" }"),
    ("a = 1\nr = {a: a}\n", "{ a }"),
  ];
  for (input, expected_fragment) in cases {
    let (tokens, comments) = lex(input).expect("lex failed");
    let result = parse(tokens, FileId::new(0), comments, input);
    let program = result.program.expect("parse failed");
    let formatted = format(&program);
    assert!(formatted.contains(expected_fragment), "Expected formatted output to contain {expected_fragment:?}, got:\n{formatted}");
  }
}
