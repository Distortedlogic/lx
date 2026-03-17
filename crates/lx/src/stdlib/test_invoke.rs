use std::sync::Arc;

use indexmap::IndexMap;

use crate::backends::RuntimeCtx;
use crate::builtins::call_value;
use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

pub(super) fn invoke_flow(
    flow_path: &str,
    input: &Value,
    ctx: &Arc<RuntimeCtx>,
    span: Span,
) -> Result<Value, LxError> {
    let path = std::path::Path::new(flow_path);
    let source = std::fs::read_to_string(path).map_err(|e| {
        LxError::runtime(
            format!("test.run: cannot read flow '{flow_path}': {e}"),
            span,
        )
    })?;
    let tokens = crate::lexer::lex(&source).map_err(|e| {
        LxError::runtime(format!("test.run: lex error in '{flow_path}': {e}"), span)
    })?;
    let program = crate::parser::parse(tokens).map_err(|e| {
        LxError::runtime(format!("test.run: parse error in '{flow_path}': {e}"), span)
    })?;
    let module_dir = path.parent().map(|p| p.to_path_buf());
    let mut interp = crate::interpreter::Interpreter::new(&source, module_dir, Arc::clone(ctx));
    interp.exec(&program).map_err(|e| {
        LxError::runtime(format!("test.run: exec error in '{flow_path}': {e}"), span)
    })?;

    let exports = collect_flow_exports(&program, &interp);
    let entry = exports
        .get("run")
        .or_else(|| exports.get("main"))
        .ok_or_else(|| {
            LxError::runtime(
                format!("test.run: flow '{flow_path}' must export +run or +main"),
                span,
            )
        })?;
    call_value(entry, input.clone(), span, ctx)
}

fn collect_flow_exports(
    program: &crate::ast::Program,
    interp: &crate::interpreter::Interpreter,
) -> IndexMap<String, Value> {
    use crate::ast::{BindTarget, Stmt};
    let mut bindings = IndexMap::new();
    for stmt in &program.stmts {
        if let Stmt::Binding(b) = &stmt.node
            && b.exported
            && let BindTarget::Name(ref name) = b.target
            && let Some(val) = interp.env.get(name)
        {
            bindings.insert(name.clone(), val);
        }
    }
    bindings
}
