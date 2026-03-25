# Goal

Fix four parser bugs that force workarounds in every lx program: pipe-after-block-lambda, trait body name conflict, assert message parsing, and tuple destructuring in trait bodies.

# Why

- `filter (e) { e > 1 } | len` parses `| len` inside the lambda body instead of piping the filter result. Every program splits filter+len into two lines as a workaround.
- `{name: value}` record literals inside trait/class method bodies are misinterpreted as trait field declarations. Every program uses semicolons between record fields or avoids record literals in methods.
- `assert (condition) "message"` parses as applying the Bool result to the String. Every test omits assert messages.
- `(a b) = pair` tuple destructuring fails inside trait bodies due to the same `ident =` ambiguity. Programs use `pair.0` and `pair.1` instead.

# What Changes

**Pipe after block lambda — `crates/lx/src/parser/expr_pratt.rs`:**

The Pratt parser handles function application as a postfix operator at precedence 31. When parsing `filter (e) { e > 1 } | len`, application consumes `(e)` then `{ e > 1 } | len` as the next argument because the block expression doesn't terminate the application chain.

Fix: when parsing a postfix application argument, if the argument is a `{ }` block expression, stop the application chain. The `|` after the block starts a new pipe at the outer level. Concretely: in the postfix application parsing loop in `expr_pratt.rs`, after parsing an atom that is a Block, do not continue consuming further arguments at the application precedence level. Return the application and let the Pratt loop handle `|` at pipe precedence (19).

To verify: `filter (e) { e > 1 } | len` on `[1; 2; 3]` should return `2`. `map (x) { x | to_str }` should still work (pipe inside lambda body is fine — it's consumed during block parsing, not during the postfix application loop).

**Trait body name conflict — `crates/lx/src/parser/stmt.rs` `trait_body()`:**

The trait body parser at line 228 tries `field_entry` first, which matches `ident : TypeName`. When a method body contains `{name: value}` record literals, the `name:` is greedily matched as a trait field.

Fix: the `field_entry` parser should require that the token after `ident :` is a TypeName (capitalized identifier or keyword-as-type). If followed by a lowercase ident, string literal, number, or other expression, it's NOT a trait field — fall through to `default_method` parsing. Add lookahead: `ident().then_ignore(just(TokenKind::Colon)).then(type_name())` already does this because `type_name()` only matches capitalized identifiers. But the issue is that `type_name()` now also matches keyword tokens via `keyword_as_type_name()`. Verify that the parser correctly rejects `name: "some string"` and `name: some_var` as field entries.

If the issue is that `field_entry` matches `name: rub.name` (where `rub` is lowercase), then the fix is correct — `type_name()` should NOT match lowercase identifiers. Verify the `type_name()` function in `expr.rs` only matches `TokenKind::TypeName` and keyword tokens, not `TokenKind::Ident`.

To verify: `+Trait T = { method = () { {name: value; other: value2} } }` should parse without errors. The record literal inside the method body should not be mistaken for trait fields.

**Assert message — `crates/lx/src/parser/expr.rs` or `expr_compound.rs`:**

Assert is parsed as `just(TokenKind::Assert).ignore_then(expr)` which parses the entire `(condition) "message"` as a single expression. The `(condition) "message"` is function application: Bool applied to String.

Fix: change the assert parser to parse the condition expression, then optionally consume a string literal as the message. The assert AST node `ExprAssert { expr, msg }` already supports an optional message. The parser should be:

```
just(TokenKind::Assert)
  .ignore_then(expr.clone())
  .then(string_literal.or_not())
  .map(|(expr, msg)| ExprAssert { expr, msg })
```

Where `string_literal` matches a `StrStart ... StrEnd` token sequence (a string literal expression). If the next token after the condition is a string start, consume it as the message. Otherwise, `msg` is None.

Read the current assert parser to find the exact location and current implementation before making changes.

To verify: `assert (x == 1) "x should be 1"` should pass with message. `assert (x == 1)` should pass without message. `assert false "expected failure"` should produce an error with "expected failure" as the help text.

**Tuple destructuring in trait body — same fix as trait body name conflict:**

`(a b) = pair` fails because after `(a b)`, the `=` is interpreted as starting a default method (`name = expr`). But `(a b)` is not an identifier — it's a parenthesized expression. The `default_method` parser expects `ident = expr`. Since `(a b)` doesn't match `ident`, it falls through to other alternatives and fails.

This is fixed by the trait body lookahead fix above — if the trait body parser correctly distinguishes field declarations from nested expressions, tuple destructuring patterns won't be misinterpreted. Verify by testing `+Trait T = { method = () { (a b) = pair; a } }`.

# Files Affected

- `crates/lx/src/parser/expr_pratt.rs` — Fix application termination after block argument
- `crates/lx/src/parser/stmt.rs` — Verify trait_body field_entry lookahead
- `crates/lx/src/parser/expr.rs` or `expr_compound.rs` — Fix assert message parsing

# Task List

### Task 1: Fix pipe after block lambda

**Subject:** Stop application chain after { } block argument in Pratt parser

**Description:** Read `crates/lx/src/parser/expr_pratt.rs`. Find the postfix application parsing — the section that handles implicit function application (when an expression is followed by another expression at application precedence).

The current behavior: `filter (e) { e > 1 } | len` — after parsing `filter`, the application loop consumes `(e)` as arg1, then `{ e > 1 } | len` as arg2 (because `{ e > 1 }` is a block expression and `| len` is parsed inside it or as continuation).

The fix: after consuming a `{ }` block as a function argument, break out of the application loop. The block is a complete argument. The next token (`|`) should be handled at the outer Pratt precedence level.

Write a test: create `tests/parser_pipe_after_block.lx`:
```lx
result = [1; 2; 3] | filter (e) { e > 1 } | len
assert (result == 2)
inner = [1; 2; 3] | map (x) { x | to_str }
assert (inner | len == 3)
```

**ActiveForm:** Fixing pipe-after-block-lambda parsing

---

### Task 2: Verify and fix trait body field lookahead

**Subject:** Ensure trait body parser doesn't match record fields as trait fields

**Description:** Read `crates/lx/src/parser/stmt.rs` around line 228 where `trait_body()` is defined. Read the `field_entry` parser.

The `field_entry` parser matches `ident : type_name`. Check whether `type_name()` in `expr.rs` matches ONLY capitalized identifiers (TypeName tokens) and keyword-as-type tokens. It should NOT match lowercase identifiers. If a method body contains `{name: rub.name}`, the `rub` after `:` is lowercase — `type_name()` should reject it, and `field_entry` should fail, letting `default_method` parse the method body.

Test by running: `echo '+Trait T = { m = () { {name: "hello"; age: 42} } }' > /tmp/test_trait.lx && cargo run -p lx-cli -- run /tmp/test_trait.lx`. If it parses, the lookahead already works. If it fails, the `field_entry` parser needs fixing.

If the lookahead IS already correct but fails in practice, the issue is that chumsky's `choice` combinator doesn't backtrack properly after `field_entry` partially matches. In that case, wrap `field_entry` in `try()` or `attempt()` to enable backtracking on partial match failure.

Write a test: create `tests/parser_trait_record.lx`:
```lx
+Trait HasMethod = {
  describe = () {
    n = "test"
    {name: n; value: 42}
  }
}
Class Impl : [HasMethod] = {}
obj = Impl {}
result = obj.describe ()
assert (result.name == "test")
assert (result.value == 42)
```

**ActiveForm:** Verifying trait body field lookahead

---

### Task 3: Fix assert message parsing

**Subject:** Assert consumes optional string message after condition

**Description:** Read the assert expression parser. Search for `Assert` in `crates/lx/src/parser/expr.rs` or `expr_compound.rs` — find where `ExprAssert` is constructed.

Currently: `assert` parses one expression as the condition, and optionally a second expression as the message. The issue is `assert (cond) "msg"` — `(cond) "msg"` is parsed as a single expression where the Bool result of `(cond)` is applied to `"msg"` as a function call.

Fix: parse assert as `assert <expr>` where the expr is the condition, then check if the NEXT token is a string literal start (`StrStart`). If so, parse the string as the message separately — not as part of the condition expression.

The key change: the condition expression should be parsed at a precedence that STOPS before function application of a string. One approach: parse the condition using a restricted expression parser that doesn't include bare string literals as continuations. Another approach: after parsing the condition, check if it's a function application where the last arg is a string — extract the string as the message.

The simplest approach: after parsing `assert expr`, check if `expr` is an `Apply` where the `arg` is a `Literal::Str`. If so, split it: the `func` part becomes the condition, the `arg` becomes the message. This is a post-parse fixup, not a parser change.

Write a test: create `tests/parser_assert_message.lx`:
```lx
assert (1 == 1) "one equals one"
assert (true)
x = 5
assert (x > 0) "x must be positive"
```

**ActiveForm:** Fixing assert message parsing

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

1. **Call `complete_task` after each task.**
2. **Call `next_task` to get the next task.**
3. **Do not add, skip, reorder, or combine tasks.**
4. **Tasks are implementation-only.**

---

## Task Loading Instructions

```
mcp__workflow__load_work_item({ path: "work_items/LX_PARSER_FIXES.md" })
```
