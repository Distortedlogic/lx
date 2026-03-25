# Goal

Make newlines inside `[ ]` act as element separators, matching the `{ }` behavior. Currently `[a \n b]` parses as `[a b]` (function application) because newlines are whitespace at depth > 0.

# Why

Every list with elements on separate lines requires explicit semicolons: `["a"; "b"; "c"]`. Records inside lists on separate lines fail without semicolons between list elements: `[{record1}; {record2}]`. This is the most common parse error in lx programs.

# What Changes

**`crates/lx/src/lexer/mod.rs`:**

Apply the same depth-reset pattern used for `{ }` (already implemented this session). `[` saves `paren_bracket_depth` and resets to 0. `]` restores the saved depth.

Current code:
```rust
RawToken::LBracket => {
    self.paren_bracket_depth += 1;
    self.emit(Token::new(TokenKind::LBracket, span));
},
RawToken::RBracket => {
    self.paren_bracket_depth -= 1;
    self.emit(Token::new(TokenKind::RBracket, span));
},
```

New code:
```rust
RawToken::LBracket => {
    self.brace_stack.push(self.paren_bracket_depth);
    self.paren_bracket_depth = 0;
    self.emit(Token::new(TokenKind::LBracket, span));
},
RawToken::RBracket => {
    self.paren_bracket_depth = self.brace_stack.pop().unwrap_or(0);
    self.emit(Token::new(TokenKind::RBracket, span));
},
```

This means newlines inside `[ ]` become semicolons (element separators). Multi-line lists work without explicit semicolons.

`( )` parentheses keep the current behavior — `self.paren_bracket_depth += 1` / `-= 1`. Newlines inside `( )` remain whitespace. This is correct because parenthesized expressions can span lines: `(a + \n b)` should not insert a semicolon.

After this change, remove unnecessary semicolons between list elements in all .lx files where elements are on separate lines. These semicolons are now redundant (but harmless — they're valid syntax).

# Files Affected

- `crates/lx/src/lexer/mod.rs` — change `[` and `]` depth handling
- `tests/lexer_list_newlines.lx` — new test
- Optionally: remove workaround semicolons in programs/ and pkg/ .lx files

# Task List

### Task 1: Fix lexer bracket depth handling

**Subject:** Make [ ] save/reset depth so newlines inside become separators

**Description:** Edit `crates/lx/src/lexer/mod.rs`. Find the `RawToken::LBracket` and `RawToken::RBracket` handlers.

Change LBracket from incrementing depth to saving and resetting:
```rust
RawToken::LBracket => {
    self.brace_stack.push(self.paren_bracket_depth);
    self.paren_bracket_depth = 0;
    self.emit(Token::new(TokenKind::LBracket, span));
},
```

Change RBracket from decrementing to restoring:
```rust
RawToken::RBracket => {
    self.paren_bracket_depth = self.brace_stack.pop().unwrap_or(0);
    self.emit(Token::new(TokenKind::RBracket, span));
},
```

Write test `tests/lexer_list_newlines.lx`:
```lx
items = [
  1
  2
  3
]
assert (items | len == 3)

records = [
  {name: "a"; value: 1}
  {name: "b"; value: 2}
]
assert (records | len == 2)

nested = [
  [1; 2]
  [3; 4]
]
assert (nested | len == 2)

strings = [
  "hello"
  "world"
]
assert (strings | len == 2)
```

Run ALL existing tests to verify no regressions. This is a fundamental lexer change.

**ActiveForm:** Fixing lexer bracket depth for list newlines

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

1. **Call `complete_task` after each task.**
2. **Call `next_task` to get the next task.**
3. **Do not add, skip, reorder, or combine tasks.**
4. **Tasks are implementation-only.**

---

## Task Loading Instructions

```
mcp__workflow__load_work_item({ path: "work_items/LX_LEXER_LIST_NEWLINES.md" })
```
