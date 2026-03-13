# Formatter Design

`lx fmt` — one canonical format, zero configuration. Takes an AST and pretty-prints it to source text.

Implements: [toolchain.md](../spec/toolchain.md) (Formatter section)

## Architecture

The formatter operates on the AST, not on source text. This guarantees canonical output regardless of input formatting. The flow:

```
source → Lexer → Parser → AST → Formatter → canonical source
```

Round-trip stability: `lx fmt` applied twice produces identical output. This is verified by the test suite.

## Rules

All rules are hardcoded. No configuration. No options.

### Indentation

2-space indent. Tabs are never emitted.

```
x ? {
  0 -> "zero"
  n -> "{n}"
}
```

### Pipe Chains

Single-stage pipes stay on one line. Multi-stage pipes (2+) get one stage per line with leading `|`:

```
-- 1 stage: inline
data | sort

-- 2+ stages: one per line
data
  | filter (> 0)
  | map (* 2)
  | sum
```

### Records

3 or fewer fields: inline with two spaces between fields. 4+ fields: one per line.

```
-- inline (≤3 fields)
{x: 3.0  y: 4.0}

-- multiline (4+ fields)
{
  name: "alice"
  age: 30
  email: "a@b.com"
  role: "admin"
}
```

### Lists

Short lists (≤60 chars total) stay inline. Long lists wrap to one element per line.

```
-- inline
[1 2 3 4 5]

-- multiline
[
  "very long string element"
  "another long string element"
  "yet another one"
]
```

### Maps and Sets

Same rules as records/lists respectively, prefixed with `%{` or `#{`.

### Functions

Short function bodies stay inline with the definition. Long bodies get a block:

```
-- inline (≤60 chars)
double = (x) x * 2

-- block
process = (data) {
  cleaned = data | filter (!= "")
  cleaned | sort | uniq
}
```

### Match Arms

Each arm on its own line. Short arms with short bodies stay on one line:

```
x ? {
  0 -> "zero"
  1 -> "one"
  n -> "{n}"
}
```

Long arm bodies get a block:

```
cmd ? {
  "start" -> {
    init ()
    run ()
  }
  "stop" -> halt ()
}
```

### Imports

Sorted: `std/` imports first (alphabetical), then relative imports (alphabetical). One blank line between import groups and the first binding.

```
use std/fs
use std/net/http

use ./util
use ../shared/types

+main = () { ... }
```

### Spacing

- Single blank line between top-level bindings
- No blank lines inside blocks
- No trailing whitespace
- File ends with a single newline
- Spaces around binary operators: `x + y`, `a | b`, `x == y`
- No space before `:` in type annotations: `x:Int`
- Space after `:` in record fields: `{x: 1}`
- Two spaces between record fields inline: `{x: 1  y: 2}`

### Exports

`+` is immediately before the binding name, at column 0:

```
+main = () { ... }
+Point = {x: Float  y: Float}
```

## Implementation

The formatter is a recursive function `fmt(expr, indent_level) -> String`:

```rust
fn fmt_expr(expr: &Expr, indent: usize) -> String {
    match expr {
        Expr::Binary { op, left, right } => {
            format!("{} {} {}", fmt_expr(left, indent), op, fmt_expr(right, indent))
        }
        Expr::Pipe { left, right } => {
            // check pipeline length
            let stages = collect_pipe_stages(expr);
            if stages.len() <= 1 {
                format!("{} | {}", fmt_expr(left, indent), fmt_expr(right, indent))
            } else {
                let first = fmt_expr(&stages[0], indent);
                let rest: Vec<String> = stages[1..].iter()
                    .map(|s| format!("{}| {}", " ".repeat(indent + 2), fmt_expr(s, indent + 2)))
                    .collect();
                format!("{}\n{}", first, rest.join("\n"))
            }
        }
        // ...
    }
}
```

The key challenge is pipeline flattening: `a | b | c` is parsed as `Pipe(Pipe(a, b), c)`. The formatter collects all stages into a flat list before deciding on formatting.

## Width Estimation

The formatter estimates the printed width of an expression to decide inline vs multiline. This is a simple character count, not a full layout engine. If the estimate exceeds 60 chars, the expression goes multiline.

## `lx fmt --check`

In check mode, the formatter compares the canonical output to the input source. If they differ, exit nonzero and print the file paths that would change. No modifications are made.

## Cross-References

- AST input: [impl-ast.md](impl-ast.md)
- Formatting rules spec: [toolchain.md](../spec/toolchain.md)
- Test coverage: round-trip tests in the suite
