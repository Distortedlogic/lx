#[path = "flow_compose.rs"]
mod flow_compose;
#[path = "flow_run.rs"]
mod flow_run;

use std::sync::Arc;

use indexmap::IndexMap;

use crate::backends::RuntimeCtx;
use crate::builtins::mk;
use crate::error::LxError;
use crate::record;
use crate::span::Span;
use crate::value::Value;

pub fn build() -> IndexMap<String, Value> {
    let mut m = IndexMap::new();
    m.insert("load".into(), mk("flow.load", 1, bi_load));
    m.insert("run".into(), mk("flow.run", 2, flow_run::bi_run));
    m.insert("pipe".into(), mk("flow.pipe", 1, flow_compose::bi_pipe));
    m.insert(
        "parallel".into(),
        mk("flow.parallel", 1, flow_compose::bi_par),
    );
    m.insert(
        "branch".into(),
        mk("flow.branch", 1, flow_compose::bi_branch),
    );
    m.insert(
        "with_retry".into(),
        mk("flow.with_retry", 2, flow_compose::bi_with_retry),
    );
    m.insert(
        "with_timeout".into(),
        mk("flow.with_timeout", 2, flow_compose::bi_with_timeout),
    );
    m.insert(
        "with_fallback".into(),
        mk("flow.with_fallback", 2, flow_compose::bi_with_fallback),
    );
    m
}

fn bi_load(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let path_str = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("flow.load: path must be Str", span))?;

    let path = resolve_path(path_str, ctx);

    let source = std::fs::read_to_string(&path).map_err(|e| {
        LxError::runtime(
            format!("flow.load: cannot read '{}': {e}", path.display()),
            span,
        )
    })?;

    let tokens = crate::lexer::lex(&source).map_err(|e| {
        LxError::runtime(
            format!("flow.load: lex error in '{}': {e}", path.display()),
            span,
        )
    })?;
    let program = crate::parser::parse(tokens).map_err(|e| {
        LxError::runtime(
            format!("flow.load: parse error in '{}': {e}", path.display()),
            span,
        )
    })?;

    let exports = extract_exports(&program);
    let canonical = path.canonicalize().unwrap_or(path);

    Ok(Value::Ok(Box::new(record! {
        "__flow" => Value::Str(Arc::from("single")),
        "path" => Value::Str(Arc::from(canonical.to_string_lossy().as_ref())),
        "source" => Value::Str(Arc::from(source.as_str())),
        "exports" => Value::List(Arc::new(
            exports.into_iter().map(|e| Value::Str(Arc::from(e.as_str()))).collect()
        )),
    })))
}

pub(super) fn resolve_path(path_str: &str, ctx: &Arc<RuntimeCtx>) -> std::path::PathBuf {
    if (path_str.starts_with("./") || path_str.starts_with("../"))
        && let Some(ref dir) = *ctx.source_dir.lock()
    {
        return dir.join(path_str);
    }
    std::path::PathBuf::from(path_str)
}

pub(super) fn extract_exports(program: &crate::ast::Program) -> Vec<String> {
    use crate::ast::{BindTarget, Stmt};
    let mut exports = Vec::new();
    for stmt in &program.stmts {
        if let Stmt::Binding(b) = &stmt.node
            && b.exported
            && let BindTarget::Name(ref name) = b.target
        {
            exports.push(name.clone());
        }
    }
    exports
}

pub(super) fn find_entry(program: &crate::ast::Program) -> Option<String> {
    use crate::ast::{BindTarget, Stmt};
    let mut has_run = false;
    let mut has_main = false;
    for stmt in &program.stmts {
        if let Stmt::Binding(b) = &stmt.node
            && b.exported
            && let BindTarget::Name(ref name) = b.target
        {
            if name == "run" {
                has_run = true;
            } else if name == "main" {
                has_main = true;
            }
        }
    }
    if has_run {
        Some("run".into())
    } else if has_main {
        Some("main".into())
    } else {
        None
    }
}
