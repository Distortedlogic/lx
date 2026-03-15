# Error System Design

Error types, diagnostic generation, propagation traces, and JSON output.

Implements: [errors.md](../spec/errors.md), [diagnostics.md](../spec/diagnostics.md)

## LxError Enum

The actual implementation uses a flat enum with miette derives:

```rust
#[derive(Debug, Clone, Error, Diagnostic)]
pub enum LxError {
    #[error("parse error: {msg}")]
    #[diagnostic(code(lx::parse))]
    Parse { msg: String, span: SourceSpan, help: Option<String> },

    #[error("runtime error: {msg}")]
    #[diagnostic(code(lx::runtime))]
    Runtime { msg: String, span: SourceSpan },

    #[error("assertion failed: {expr}")]
    #[diagnostic(code(lx::assert))]
    Assert { expr: String, message: Option<String>, span: SourceSpan },

    #[error("type error: {msg}")]
    #[diagnostic(code(lx::type_error))]
    Type { msg: String, span: SourceSpan },

    #[error("import error: {msg}")]
    #[diagnostic(code(lx::import))]
    Import { msg: String, span: SourceSpan },

    #[error("division by zero")]
    #[diagnostic(code(lx::runtime))]
    DivisionByZero { span: SourceSpan },

    BreakSignal { value: Box<Value> },
    RollbackSignal { name: String },
    PropagatedError { inner: Box<LxError>, span: SourceSpan },
}
```

Constructor helpers (`LxError::parse(...)`, `LxError::runtime(...)`, etc.) create variants with proper spans. Every variant carries a `Span` for source location.

## Diagnostic Generation with miette

`LxError` implements `miette::Diagnostic`. Each variant maps to an error category code:

| Variant | Code |
|---|---|
| `Parse` | `lx::parse` |
| `Type` | `lx::type_error` |
| `Runtime` | `lx::runtime` |
| `Assert` | `lx::assert` |
| `Import` | `lx::import` |
| `DivisionByZero` | `lx::runtime` |

Each variant builds a `miette::Report` with:
- **Source**: the full file content via `miette::NamedSource`
- **Span**: converted to `miette::SourceSpan` via `(offset, len)`
- **Labels**: primary span labeled with the error message
- **Help**: the `help` field when present

## Propagation Trace

When `^` evaluates an expression that returns `Err`, the error is wrapped in `PropagatedError` with the `^` site's span. Nested `^` operations create a chain of `PropagatedError` wrappers.

## Assert Error

When `assert` fails, the interpreter reports the failing expression's AST debug representation and optional message string. The diagnostic points at the full assert expression span.

## Error Recovery in Parser

The parser currently aborts on the first error. Error recovery (collecting multiple errors, synchronization points) is planned but not implemented.

## Cross-References

- Error handling spec: [errors.md](../spec/errors.md)
- Diagnostic format spec: [diagnostics.md](../spec/diagnostics.md)
- Interpreter error propagation: [impl-interpreter.md](impl-interpreter.md)
- Type checker diagnostics: [impl-checker.md](impl-checker.md)
- Module file: `crates/lx/src/error.rs`
