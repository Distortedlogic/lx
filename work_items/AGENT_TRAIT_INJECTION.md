# Goal

Fix the bug where `inject_traits` silently skips traits that are not found in the environment. When a `Class Foo : [MyTrait]` declaration references a trait that doesn't exist or isn't imported, the class is created with no trait methods and no error. Change the silent `continue` to a hard error with an actionable message.

# Why

- `inject_traits` at `traits.rs:22` has `let Some(LxVal::Trait(t)) = env.get(*tn) else { continue; }`. When a trait name is not found in the environment, it silently skips to the next trait. The class is created without the trait's defaults or required method checks. `self.think` or any other trait method is `None` with no indication of why
- The `Agent` trait is defined in `pkg/core/trait.lx` (or similar). A class declaring `Class Foo : [Agent]` must import the Agent trait explicitly. If the import is missing, the trait is silently skipped and all Agent defaults (`think`, `think_with`, `handle`, `run`) are `None`
- There is no `Agent` keyword in the current parser — the lexer has no `AgentKw` token, and the parser only has `ClassKw` (at `stmt_class.rs:31`). The `Agent` keyword referenced in BUGS.md was removed. All agent classes use `Class : [Agent]` syntax, making this silent skip the only code path
- Silent trait skipping is always wrong. If a programmer declares `Class Foo : [MyTrait]`, they expect MyTrait's methods. Silently producing a class without those methods is a bug, not a fallback

# What changes

The `continue` at `traits.rs:22` becomes an error return: `Err(LxError::runtime(...))`. Any class that declares a trait it hasn't imported will now get a clear error message naming the missing trait, instead of silently degrading.

# Files affected

| File | Change |
|------|--------|
| `crates/lx/src/interpreter/traits.rs` | Change `continue` to `Err(LxError::runtime(...))` at line 22 |

# Task List

### Task 1: Change silent skip to error in inject_traits

In `crates/lx/src/interpreter/traits.rs`, find the trait lookup at lines 20-22:

```rust
for tn in traits {
    let Some(LxVal::Trait(t)) = env.get(*tn) else {
        continue;
    };
```

Change to:

```rust
for tn in traits {
    let Some(LxVal::Trait(t)) = env.get(*tn) else {
        return Err(LxError::runtime(
            format!("{kind} '{name}' declares Trait '{tn}' but it is not defined — add `use` to import it"),
            span,
        ));
    };
```

The function already returns `Result<(), LxError>` (line 12) and all callers already use `?` (exec_stmt.rs:127 calls `Self::inject_traits(...)?.`). No signature change or caller updates needed.

### Task 2: Add a test file to verify trait default injection works

Create `tests/class_trait_defaults.lx`:

```
-- class trait default injection test
-- verifies Class : [Trait] injects trait defaults

+Trait Greeter = {
  greeting: Str = "hello"

  greet = (name) {
    self.greeting ++ " " ++ name
  }
}

+Class MyGreeter : [Greeter] = {
  greeting: "hi"
}

g = MyGreeter ()
result = g.greet "world"
assert (result == "hi world")

+Class DefaultGreeter : [Greeter] = {
  greeting: "hey"
}

g2 = DefaultGreeter ()
result2 = g2.greet "there"
assert (result2 == "hey there")
```

This tests that a Class with `: [Trait]` gets trait defaults injected when the trait is defined in the same file. If `inject_traits` finds the trait in scope (which it will since `+Trait Greeter` is defined above), it injects the `greet` default method.

### Task 3: Verify missing trait now errors

This is a manual verification, not a test file. After Task 1, run a `.lx` file that declares a class with an undefined trait:

```
Class Bad : [DoesNotExist] = {
  x: 0
}
```

Verify the output is an error like: `Class 'Bad' declares Trait 'DoesNotExist' but it is not defined — add 'use' to import it`. The test runner will show this as a FAIL with the error message.

Do NOT create a test file that expects to catch this error — `LxError::runtime` is a hard error, not catchable by `??` (which only catches `Value::Err` and `Value::None`). A test file containing this code will always fail, which is correct behavior for a programming error.

### Task 4: Fix any existing files that relied on silent trait skipping

Run `just test`. If any existing `.lx` test files or programs fail with the new "Trait not defined" error, they have a missing `use` statement. The error message names the missing trait. Add the appropriate `use` import to each failing file.

Common case: files that use `Class Foo : [Agent]` without `use pkg/core/trait` (or wherever the Agent trait is defined). Search for `Agent` in the traits list of ClassDeclData across `.lx` files and verify each has the corresponding import.

### Task 5: Compile, format, and verify

Run `just fmt` to format changed files.

Run `just test` to verify:
1. The new test file passes (trait defaults injected for same-file traits)
2. All existing tests still pass (after Task 4 fixes)
3. No regressions

---

## CRITICAL REMINDERS

1. **NEVER run raw cargo commands.** Use `just fmt`, `just test`, `just diagnose`.
2. **Do not add, skip, reorder, or combine tasks.**
3. **The fix is changing ONE line** — `continue` to `return Err(...)` at `traits.rs:22`. Do not change anything else in the injection logic.
4. **`inject_traits` is called at `exec_stmt.rs:127`** with `?` propagation. The function already returns `Result<(), LxError>`. No signature change needed.
5. **There is no `Agent` keyword in the parser.** The lexer has no `AgentKw` token. All agent classes use `Class : [Agent]` syntax. The BUGS.md references to an "Agent keyword" describe removed functionality.
6. **`LxError::runtime` is NOT catchable by `??`.** This is correct — declaring a class with a missing trait is a programming error. Do not try to make it catchable.

---

## Task Loading Instructions

To execute this work item, load it with the workflow MCP:

```
mcp__workflow__load_work_item({ path: "work_items/AGENT_TRAIT_INJECTION.md" })
```

Then call `next_task` to begin.
