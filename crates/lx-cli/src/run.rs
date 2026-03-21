use std::path::Path;
use std::process::ExitCode;
use std::sync::Arc;

use lx::backends::RuntimeCtx;

pub fn run(
    source: &str,
    filename: &str,
    ctx: &Arc<RuntimeCtx>,
) -> Result<(), Vec<lx::error::LxError>> {
    let tokens = lx::lexer::lex(source).map_err(|e| vec![e])?;
    let program = lx::parser::parse(tokens).map_err(|e| vec![e])?;
    let source_dir = Path::new(filename).parent().map(|p| p.to_path_buf());
    let mut interp = lx::interpreter::Interpreter::new(source, source_dir, Arc::clone(ctx));
    ctx.tokio_runtime.block_on(async {
        match interp.exec(&program).await {
            Ok(val) => {
                if !matches!(val, lx::value::LxVal::Unit) {
                    println!("{val}");
                }
                Ok(())
            }
            Err(e) => Err(vec![e]),
        }
    })
}

pub fn read_and_parse(path: &str) -> Result<(String, lx::ast::Program), ExitCode> {
    let source = std::fs::read_to_string(path).map_err(|e| {
        eprintln!("error: cannot read {path}: {e}");
        ExitCode::from(1)
    })?;
    let tokens = lx::lexer::lex(&source).map_err(|e| {
        let named = miette::NamedSource::new(path, source.clone());
        eprintln!("{:?}", miette::Report::new(e).with_source_code(named));
        ExitCode::from(1)
    })?;
    let program = lx::parser::parse(tokens).map_err(|e| {
        let named = miette::NamedSource::new(path, source.clone());
        eprintln!("{:?}", miette::Report::new(e).with_source_code(named));
        ExitCode::from(1)
    })?;
    Ok((source, program))
}
