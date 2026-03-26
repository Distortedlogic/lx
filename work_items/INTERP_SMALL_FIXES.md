# Goal

Fix two bugs: adjacent string interpolation `"{a}{b}"` failing with "cannot call Str", and computed tuple index `t.[i]` failing with "unsupported types Tuple / Int".

# Why

- `"{a}{b}"` crashes because the lexer at `strings.rs:47-51` advances past `{` without emitting `LBrace`, and `lex_interpolation` returns at line 81 without emitting `RBrace`. The parser at `expr_pratt.rs:25` expects `LBrace expr RBrace` per interpolation (`interp_braced`) but gets bare tokens instead. The fallback `interp_bare` at line 26 parses `Ident(a) Ident(b)` as `Apply(a, b)` — function application. At runtime, `to_str(a)` returns a Str, then the Apply tries to call it with `to_str(b)`, causing "cannot call Str".
- `t.[i]` crashes because the computed access match at `apply_helpers.rs:50-61` handles Record+Str, Map+Str, and List+Int but has no Tuple+Int arm.

# Files affected

- `crates/lx/src/lexer/strings.rs` lines 47-51 and 78-81 — Emit LBrace/RBrace tokens
- `crates/lx/src/interpreter/apply_helpers.rs` lines 56-59 — Add Tuple+Int arm

# Task List

### Task 1: Emit LBrace before interpolation in read_string

In `crates/lx/src/lexer/strings.rs`, the `Some('{')` arm at lines 47-52 currently does:
```
flush_buf → advance past { → lex_interpolation → chunk_start = self.pos
```

Change to: capture brace position before advance, then emit LBrace between advance and lex_interpolation. The new sequence: `flush_buf`, capture `let brace_start = self.pos`, `self.advance()`, `self.push(TokenKind::LBrace, brace_start, brace_start + 1)`, `self.lex_interpolation(start)?`, `chunk_start = self.pos`.

### Task 2: Emit RBrace at end of lex_interpolation

In `crates/lx/src/lexer/strings.rs`, the `lex_interpolation` function at line 64. At lines 78-82, when `RBrace` is found and `brace_depth == 0`, the function returns without emitting. Change: before `return Ok(())` at line 81, add `self.push(TokenKind::RBrace, start, end)` where `start` and `end` are from line 71 (`let (start, end) = (base + rel.start, base + rel.end)`) — these are the positions of the `}` character.

After this fix, `"{a}{b}"` tokenizes as: `StrStart, LBrace, Ident(a), RBrace, LBrace, Ident(b), RBrace, StrEnd`. The parser's `interp_braced` at `expr_pratt.rs:25` matches each `LBrace expr RBrace` pair. Single interpolation like `"hello {name}"` tokenizes as: `StrStart, StrChunk("hello "), LBrace, Ident(name), RBrace, StrEnd` — also correct. Nested braces like `"{f {x: 1}}"` work because `lex_interpolation` tracks brace depth: inner `{` increments to 2, inner `}` decrements to 1 (dispatched normally at line 87), outer `}` decrements to 0 (emitted as RBrace).

### Task 3: Add Tuple arm in computed field access

In `crates/lx/src/interpreter/apply_helpers.rs`, add a new match arm after the `LxVal::List` arm at line 59. The arm matches `(LxVal::Tuple(items), LxVal::Int(n))`. Implementation identical to the List arm at lines 56-59: call `n.to_i64()` (returns `Option<i64>`), convert to usize with negative index support (`if i < 0 { items.len() as i64 + i } else { i } as usize`), index into `items.get(i)`, return cloned value or error "index {i} out of bounds (tuple length {})". `Tuple` stores data as `Arc<Vec<LxVal>>` same as `List`.

### Task 4: Add tests

Create `tests/interp_small_fixes.lx`:

Adjacent interpolation: `a = "hello"`, `b = "world"`, `assert ("{a}{b}" == "helloworld") "adjacent interp"`.

Three adjacent: `x = "a"`, `y = "b"`, `z = "c"`, `assert ("{x}{y}{z}" == "abc") "three adjacent"`.

Mixed text and interp (regression): `name = "lx"`, `assert ("hello {name}!" == "hello lx!") "mixed interp"`.

Tuple computed access: `t = (10; 20; 30)`, `assert (t.[0] == 10) "tuple idx 0"`, `assert (t.[2] == 30) "tuple idx 2"`.

Tuple computed with variable: `i = 1`, `t = ("a"; "b"; "c")`, `assert (t.[i] == "b") "tuple var idx"`.

Positional access still works (regression): `t = (1; 2)`, `assert (t.0 == 1) "positional"`.

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.
