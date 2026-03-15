# Error System Design

Error types, diagnostic generation, propagation traces, and JSON output.

Implements: [errors.md](../spec/errors.md), [diagnostics.md](../spec/diagnostics.md)

## LxError Enum

```rust
enum LxError {
    Parse(ParseError),
    Type(TypeError),
    Runtime(RuntimeError),
    Io(IoError),
    Shell(ShellError),
    Import(ImportError),
    Assert(AssertError),
    Defer(DeferError),
    Concurrency(ConcurrencyError),
    Propagated(PropagatedError),
}

struct ParseError {
    expected: String,
    got: String,
    fix: Option<String>,
    span: Span,
}

struct TypeError {
    kind: TypeErrorKind,
    span: Span,
}

enum TypeErrorKind {
    Mismatch { expected: String, got: String, fix: Option<String> },
    Arity { expected: usize, got: usize, func_name: String },
    NonBoolCondition { got: String },
    PropagateNonResult { got: String },
    VariantConflict { name: String, existing_type: String, existing_span: Span },
}

struct RuntimeError {
    kind: RuntimeErrorKind,
    span: Span,
}

enum RuntimeErrorKind {
    DivisionByZero,
    IndexOutOfBounds { index: i64, len: usize },
    PipelineElement { stage: usize, total_stages: usize, stage_name: String,
                      element_index: usize, element_value: String, inner: Box<LxError> },
    NonExhaustiveMatch { scrutinee: String, missing: Vec<String> },
}

struct IoError {
    op: String,
    path: String,
    source: std::io::Error,
    span: Span,
}

struct ShellError {
    command: String,
    exit_code: i32,
    stderr: String,
    span: Span,
}

struct ImportError {
    kind: ImportErrorKind,
    span: Span,
}

enum ImportErrorKind {
    ModuleNotFound { path: String },
    CircularImport { chain: Vec<String> },
    NameConflict { name: String, existing_module: String, existing_span: Span },
}

struct AssertError {
    condition_src: String,
    message: Option<String>,
    sub_values: Vec<(String, String)>,
    span: Span,
}

struct DeferError {
    inner: Box<LxError>,
    defer_span: Span,
}

struct ConcurrencyError {
    binding_name: String,
    binding_span: Span,
    capture_span: Span,
}
```

Every variant carries a `Span`. The `Propagated` variant is described below.

## Diagnostic Generation with miette

`LxError` implements `miette::Diagnostic`. Each variant maps to an error category code matching the spec's bracket notation.

| Variant | Code | miette severity |
|---|---|---|
| `Parse` | `error[parse]` | Error |
| `Type` | `error[type]` | Error |
| `Runtime` | `error[runtime]` | Error |
| `Io` | `error[io]` | Error |
| `Shell` | `error[shell]` | Error |
| `Import` | `error[import]` | Error |
| `Assert` | `error[assert]` | Error |
| `Defer` | `error[defer]` | Error |
| `Concurrency` | `error[concurrency]` | Error |

Each variant builds a `miette::Report` with:
- **Source**: `miette::NamedSource::new(filename, source_text)` — the full file content, attached once per file
- **Span**: the `Span` converted to `miette::SourceSpan` via `(offset, len).into()`
- **Labels**: a `Vec<LabeledSpan>` with the primary span labeled as the error message, plus secondary labels for "expected"/"got"/"fix" where applicable
- **Help**: the `fix` field, when present, renders as miette's help text

For `TypeError::Mismatch`, three labels are attached: the primary span underlines the expression, an "expected: X" label, and a "got: Y" label. For `ShellError`, the stderr content is included as a related diagnostic (indented verbatim, not reformatted).

## Propagation Trace

```rust
struct PropagatedError {
    original: Box<LxError>,
    trace: PropagationTrace,
}

struct PropagationTrace {
    sites: Vec<PropagationSite>,
}

struct PropagationSite {
    span: Span,
    source_line: String,
}
```

Each `^` evaluation in the interpreter:
1. Evaluates the inner expression
2. On `Err(e)`, checks if `e` is already `Propagated`
3. If yes, appends the current `^` site to `trace.sites`
4. If no, wraps `e` in `PropagatedError` with a new trace containing the current site

Display format matches the spec: the original error prints first, then "propagated through:" followed by each site in reverse order (outermost `^` first, innermost last). Each site shows `file:line  source_line_text`.

When rendering as a `miette::Diagnostic`, the propagation sites are attached as `related` diagnostics — each one a `miette::Report` with its own source span pointing at the `^` token.

## JSON Output

The `--json` flag switches the diagnostic renderer from `miette::GraphicalReportHandler` to `miette::JSONReportHandler`. The CLI configures this at startup based on the flag.

```
lx run script.lx --json
```

Output goes to stderr. One JSON object per diagnostic, one per line. The JSON structure matches miette's built-in format:

```json
{"severity":"error","code":"type","message":"type mismatch",
 "labels":[{"label":"expected Int, got Str","span":{"offset":142,"length":7}}],
 "filename":"src/main.lx","help":"second argument to `add` must be Int"}
```

The CLI wires this up:

```rust
fn report_diagnostics(errors: &[LxError], json: bool) {
    let handler: Box<dyn ReportHandler> = if json {
        Box::new(JSONReportHandler::new())
    } else {
        Box::new(GraphicalReportHandler::new_themed(GraphicalTheme::unicode_nocolor()))
    };
    for err in errors {
        handler.render_report(&mut std::io::stderr(), err.as_diagnostic());
    }
}
```

The `GraphicalTheme::unicode_nocolor` theme satisfies the spec requirement of no colors and no decoration.

## Pipeline Error Context

When a pipeline stage (`map`, `filter`, `each`, etc.) catches an error from the user function, it wraps the error in `RuntimeErrorKind::PipelineElement`:

```
eval_builtin_map(func, iterator):
  for (i, elem) in iterator.enumerate():
    match apply(func, elem).await:
      Ok(val) => results.push(val)
      Err(inner) => return Err(LxError::Runtime(RuntimeError {
          kind: PipelineElement {
              stage: current_stage_index,
              total_stages: pipeline_length,
              stage_name: "map",
              element_index: i,
              element_value: elem.display_short(),
              inner: Box::new(inner),
          },
          span: pipe_span,
      }))
```

The stage index and total come from the pipeline's AST structure — the interpreter tracks position as it evaluates chained pipes. `display_short()` truncates the element's display to 120 characters to keep diagnostics readable.

The rendered output matches the spec format: `pipeline stage 3 of 4: map validate` / `element #47: {name: "bad"  age: -1}` / then the inner error with its own span.

## Assert Value Display

When `assert` fails, the interpreter captures sub-expression values for the `values:` line.

The interpreter walks the assert condition AST before reporting the error:

```
eval_assert(condition_expr, message):
  result = eval(condition_expr)
  if result != Value::Bool(true):
    sub_values = collect_sub_values(condition_expr)
    return Err(LxError::Assert(AssertError {
        condition_src: source_text_of(condition_expr.span),
        message: message,
        sub_values: sub_values,
        span: condition_expr.span,
    }))

collect_sub_values(expr) -> Vec<(String, String)>:
  match expr:
    Binary(left, op, right) =>
      let left_val = eval(left)
      let right_val = eval(right)
      let full_val = eval(expr)
      let mut subs = collect_sub_values(left)
      subs.extend(collect_sub_values(right))
      subs.push((source_text_of(expr.span), display(full_val)))
      subs
    Call(func, args) =>
      let val = eval(expr)
      vec![(source_text_of(expr.span), display(val))]
    _ => vec![]
```

Only `Binary` and `Call` nodes produce sub-value entries. Literals and identifiers are skipped (their values are obvious from the source). This produces output like `values: add 1 2 = 3, 3 == 4 = false` where each entry is `source_text = display_value`.

## Error Recovery in Parser

The parser collects up to 5 errors before aborting. On each parse error:

1. Create a `ParseError` diagnostic with the current token's span
2. Push it onto `self.diagnostics`
3. Advance to the next synchronization point
4. Resume parsing from there

Synchronization points: `;`, newline, `}`, `]`, `)`, EOF. The sync function:

```
fn synchronize(&mut self) {
    loop {
        match self.peek().kind {
            Semicolon | Newline | RBrace | RBracket | RParen | Eof => return,
            _ => self.advance(),
        }
    }
}
```

After synchronizing, the parser inserts an `Expr::Error(span)` node in the AST to mark where recovery happened. Downstream phases (checker, interpreter) skip `Error` nodes without producing additional diagnostics, preventing cascading errors.

When `self.diagnostics.len() >= 5`, the parser stops immediately and returns whatever AST it has, plus the collected diagnostics.

## Cross-References

- Error handling spec: [errors.md](../spec/errors.md)
- Diagnostic format spec: [diagnostics.md](../spec/diagnostics.md)
- Interpreter error propagation: [impl-interpreter.md](impl-interpreter.md) (eval_propagate, defer)
- Type checker diagnostics: [impl-checker.md](impl-checker.md) (type errors, exhaustiveness, mutable capture)
- Parser recovery: [impl-parser.md](impl-parser.md) (synchronization, error collection)
- Module file: `crates/lx/src/error.rs`
