# Shell Integration

`$` enters shell mode. This is the most-used feature — 80% of scripting is "run command, parse output, act."

## Four Variants

### `$` — Interpolated Shell

Standard shell command with lx string interpolation via `{expr}`:

```
files = $ls src
target = "src"
files = $ls {target}
name = "world"
$echo "hello {name}"
```

Everything after `$` until the newline is shell. `{expr}` sequences are evaluated by lx and substituted before the shell sees the command.

### `$$` — Raw Shell

No `{expr}` interpolation. For commands with literal braces:

```
$$find . -name "*.rs" -exec echo {} \;
$$awk '{print $1}'
```

### `$^` — Error-Propagating Shell

Propagates error on nonzero exit code (like `^` for function calls):

```
$^gcc main.c -Wall
$^cargo build --release
dir = $^pwd | trim
```

Without `$^`, a failing command returns `Err` as a value you must handle. With `$^`, a failing command immediately returns `Err` to the enclosing function.

### `${ ... }` — Multi-Line Shell Block

For multi-command shell sequences:

```
result = ${
  cd build/
  cmake ..
  make -j8
}
```

The block runs as a single shell session (commands share state like `cd`). Returns the result of the last command. Supports `{expr}` interpolation.

## Shell Results

`$` and `$^` have different return types, optimized for different use cases:

**`$cmd`** — returns `Result ShellResult ShellErr`:

```
ShellResult = {out: Str  err: Str  code: Int}
ShellErr = {cmd: Str  msg: Str}
```

`ShellErr` is for spawn failures (command not found, permission denied). A command that runs but exits nonzero is `Ok` with a nonzero `code`. The caller inspects `.code` to decide what's an error:

```
r = $gcc main.c
r ? {
  Ok {code: 0 ..} -> $./a.out
  Ok {err ..}     -> log.err err
  Err e           -> log.err "couldn't spawn: {e.cmd}: {e.msg}"
}
```

**`$^cmd`** — returns `Str ^ ShellErr`:

Extracts `.out` on exit code 0, propagates error on nonzero exit or spawn failure. The error on nonzero exit is `ShellErr {cmd  msg: err}` where `err` is the stderr output. This is the variant to use in pipelines:

```
dir = $^pwd | trim
lines = $^wc -l file.txt | trim | parse_int ^
```

**`$$cmd`** — same return type as `$` (full `ShellResult`), no interpolation.

**`${ ... }`** — returns `Result ShellResult ShellErr` for the last command in the block.

## OS Pipes vs Language Pipes

`|` within a `$` command is a shell pipe (OS-level):

```
count = $cat file.txt | grep "TODO" | wc -l
```

The `$` lexing mode extends to the end of the line. All `|` characters within a `$` line are shell pipes. To transition from shell output to a language pipe, either use `$^` (which returns `Str` directly):

```
items = $^ls src | split "\n" | filter (!= "")
```

Or wrap the `$` command in parens to end shell mode:

```
items = ($ls src) ^ | (.out) | split "\n" | filter (!= "")
```

Without parens, `|` after `$` stays in shell mode (the entire line is one shell command).

## Common Patterns

Capture and process (use `$^` for pipeline usage):

```
branches = $^git branch | lines | map trim | filter (!= "")
```

Check exit code (use `$` for the full result):

```
$git diff --quiet ? {
  Ok {code: 0 ..} -> log.info "clean"
  _               -> log.warn "dirty"
}
```

String building for complex commands:

```
flags = ["--verbose" "--output" out_dir] | join " "
$cargo build {flags}
```

Transition from shell to language pipe — wrap in parens when using `$` (not `$^`):

```
result = ($ls src)
result ^ | (.out) | split "\n" | filter (!= "")
```

With `$^`, the stdout is already extracted:

```
$^ls src | split "\n" | filter (!= "")
```

## Shell Environment

**Which shell** — `$` commands run via `/bin/sh -c`. The command string (after interpolation) is passed as a single argument to `sh`. This means shell features like `&&`, `||`, pipes, and redirects work as expected within `$` commands.

**Environment inheritance** — child processes inherit the lx process's environment. `env.set` mutations are visible to subsequent `$` commands.

**Working directory** — `$` commands inherit the lx process's working directory. `$cd dir` changes the directory only for that subshell (it exits immediately). Use `${...}` blocks for multi-command sequences that need shared `cd`:

```
result = ${
  cd build/
  make -j8
}
```

**Argument safety** — `{expr}` interpolation in `$` commands is NOT shell-safe by default. If `name` contains `"; rm -rf /"`, `$echo {name}` will execute it. In `--sandbox` mode, interpolated values are quoted. For untrusted input without sandbox mode, quote explicitly: `$echo "{name}"` (the shell sees the quotes).

## Shell Line Termination

`$` commands are single-line only — no backslash continuation. The command extends from `$` to the next newline (or `;`). For multi-line shell sequences, use `${ ... }` blocks. For complex interpolated values, assign to a variable first:

```
val = some_complex_expression
$echo {val}
```

## Cross-References

- Implementation: [impl-lexer.md](../impl/impl-lexer.md) (shell mode lexing), [impl-interpreter.md](../impl/impl-interpreter.md) (shell execution)
- Design decisions: [design.md](design.md) ($ for shell)
- Test suite: [10_shell.lx](../suite/10_shell.lx)
