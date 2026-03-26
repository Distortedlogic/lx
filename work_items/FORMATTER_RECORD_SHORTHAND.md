# Goal

Make the lx formatter emit record shorthand notation. When a record field's value is `Expr::Ident(sym)` where `sym` matches the field name, emit just `name` instead of `name: name`. This reduces token count in formatted lx output — critical since LLM agents consume formatted lx programs and context is scarce.

# Ordering Constraint

This work item MUST execute after `RECORD_SHORTHAND_PARSER.md`. The formatter will emit `{ a; b: 1 }` which the parser must understand as a record. Without the parser change, the roundtrip test will fail because `{ a; b: 1 }` (shorthand first, no colon) parses as a block under the old `looks_like_record` disambiguator.

# Verified Facts

- `emit_record` in `crates/lx/src/formatter/emit_expr.rs` (lines 145-164) handles all record field emission
- `RecordField::Named { name, value }` is the variant for both explicit and shorthand fields — shorthand fields have `value` set to `Expr::Ident(name)` during parsing (expr_helpers.rs line 81)
- The formatter has access to the AST arena via `self.arena` (type `&AstArena`) — `self.arena.expr(id)` returns `&Expr` for any `ExprId`
- `Expr` is already imported in `emit_expr.rs` line 1: `use crate::ast::{BinOp, Expr, ExprBlock, ...}`
- `name` in `RecordField::Named` is `Sym` and `Expr::Ident` contains `Sym` — direct `==` comparison works
- Existing roundtrip test in `crates/lx/tests/formatter_roundtrip.rs` parses all `tests/*.lx` files, formats them, and verifies the output re-parses. After both parser + formatter changes, this roundtrip test will verify shorthand stability (format → re-parse → re-format produces identical output)

# Files Affected

- EDIT: `crates/lx/src/formatter/emit_expr.rs` — modify `emit_record`
- NEW: `tests/record_shorthand_fmt.lx` — test that shorthand records work end-to-end
- EDIT: `crates/lx/tests/formatter_roundtrip.rs` — add exact-output test for shorthand emission

# Task List

### Task 1: Emit shorthand for matching record fields

**Subject:** Detect shorthand fields in emit_record and emit just the name

**Description:** In `crates/lx/src/formatter/emit_expr.rs`, modify the `emit_record` method (lines 145-164). Current code for the `RecordField::Named` arm:

```rust
        RecordField::Named { name, value } => {
          self.write(name.as_str());
          self.write(": ");
          self.emit_expr(*value);
        },
```

Replace with:

```rust
        RecordField::Named { name, value } => {
          let is_shorthand = matches!(self.arena.expr(*value), Expr::Ident(sym) if *sym == *name);
          self.write(name.as_str());
          if !is_shorthand {
            self.write(": ");
            self.emit_expr(*value);
          }
        },
```

No import changes needed — `Expr` is already imported on line 1.

**ActiveForm:** Adding shorthand detection to emit_record

---

### Task 2: Write exact-output formatter test

**Subject:** Rust test that verifies formatter emits shorthand notation

**Description:** In `crates/lx/tests/formatter_roundtrip.rs`, add a new test function that verifies the formatter produces shorthand output. Add this after the existing `formatter_roundtrips_test_files` test:

```rust
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
    assert!(
      formatted.contains(expected_fragment),
      "Expected formatted output to contain {expected_fragment:?}, got:\n{formatted}"
    );
  }
}
```

This test:
- Parses programs where fields use explicit `name: name` syntax (works with both old and new parser since the first field has a colon)
- Formats the AST
- Verifies the formatted output contains the shorthand form (e.g., `{ x; y: 2 }` instead of `{ x: x; y: 2 }`)

**ActiveForm:** Writing exact-output formatter test

---

### Task 3: Write end-to-end shorthand .lx test

**Subject:** Test that shorthand records parse, format, and evaluate correctly

**Description:** Create test file `tests/record_shorthand_fmt.lx`:

```lx
-- formatter emits shorthand; verify records with shorthand work end-to-end

a = 1
b = 2
r1 = {a; b}
assert (r1.a == 1)
assert (r1.b == 2)

x = 10
r2 = {x; y: 20}
assert (r2.x == 10)
assert (r2.y == 20)

name = "alice"
r3 = {user: name}
assert (r3.user == "alice")

base = {a: 1}
extra = 2
r4 = {..base; extra}
assert (r4.a == 1)
assert (r4.extra == 2)
```

This file is also picked up by the existing `formatter_roundtrips_test_files` test, which verifies that formatting this file and re-parsing produces a valid program (roundtrip stability).

Run `just test` to verify.

**ActiveForm:** Writing end-to-end shorthand lx test

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

1. **Call `complete_task` after each task.**
2. **Call `next_task` to get the next task.**
3. **Do not add, skip, reorder, or combine tasks.**
4. **Tasks are implementation-only.**

---

## Task Loading Instructions

```
mcp__workflow__load_work_item({ path: "work_items/FORMATTER_RECORD_SHORTHAND.md" })
```
