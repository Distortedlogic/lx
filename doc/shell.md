# Shell Integration — Reference

## Three Variants

### `$` — Interpolated Shell

```
files = $ls src
name = "world"
$echo "hello {name}"
```

Everything after `$` until newline is shell. `{expr}` is evaluated by lx and substituted before shell sees it.

Returns `Result ShellResult ShellErr`:

```
ShellResult = {out: Str  err: Str  code: Int}
ShellErr = {cmd: Str  msg: Str}
```

`ShellErr` is for spawn failures (command not found). Nonzero exit is `Ok` with nonzero `code`.

### `$^` — Error-Propagating Shell

```
$^cargo build --release
dir = $^pwd | trim
```

Returns `Str` (stdout) on exit code 0. Propagates `ShellErr` on nonzero exit or spawn failure.

### `${ ... }` — Multi-Line Shell Block

```
result = ${
  cd build/
  cmake ..
  make -j8
}
```

Runs as a single shell session (commands share state). Returns `Result ShellResult ShellErr` for the last command.

## OS Pipes vs Language Pipes

`|` within a `$` line is a shell pipe. The entire line is one shell command:

```
count = $cat file.txt | grep "TODO" | wc -l
```

To transition from shell to language pipe, use `$^` (returns `Str` directly):

```
items = $^ls src | split "\n" | filter (!= "")
```

Or wrap `$` in parens to end shell mode:

```
items = ($ls src) ^ | (.out) | split "\n" | filter (!= "")
```

## Common Patterns

Capture and process:

```
branches = $^git branch | lines | map trim | filter (!= "")
```

Check exit code:

```
$git diff --quiet ? {
  Ok {code: 0 ..} -> log.info "clean"
  _               -> log.warn "dirty"
}
```

String building:

```
flags = ["--verbose" "--output" out_dir] | join " "
$cargo build {flags}
```

## Gotchas

- `$` commands are single-line only. Use `${ ... }` for multi-line.
- `{expr}` interpolation is NOT shell-safe by default. In `--sandbox` mode, values are quoted. For untrusted input without sandbox, quote explicitly: `$echo "{name}"`.
- `$cd dir` only affects that subshell. Use `${ ... }` blocks for shared `cd`.
- Commands run via `/bin/sh -c`. Shell features (`&&`, `||`, redirects) work within `$`.
