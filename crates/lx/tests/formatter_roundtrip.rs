use std::fs;

use lx::formatter::format;
use lx::lexer::lex;
use lx::parser::parse;
use lx::source::FileId;

fn roundtrip_check(path: &str) {
  let source = fs::read_to_string(path).unwrap();
  let (tokens, comments) = lex(&source).expect("lex failed");
  let result = parse(tokens, FileId::new(0), comments, &source);
  let program = result.program.expect("parse failed");
  let formatted = format(&program);

  let (tokens2, comments2) = lex(&formatted).expect("re-lex failed");
  let result2 = parse(tokens2, FileId::new(0), comments2, &formatted);
  assert!(result2.program.is_some(), "round-trip failed for {}: formatted output does not re-parse.\nFormatted:\n{}", path, formatted);
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
