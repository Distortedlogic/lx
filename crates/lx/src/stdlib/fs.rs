use std::sync::Arc;

use indexmap::IndexMap;
use num_bigint::BigInt;

use crate::backends::RuntimeCtx;
use crate::builtins::mk;
use crate::error::LxError;
use crate::record;
use crate::span::Span;
use crate::value::Value;

pub fn build() -> IndexMap<String, Value> {
    let mut m = IndexMap::new();
    m.insert("read".into(), mk("fs.read", 1, bi_read));
    m.insert("write".into(), mk("fs.write", 2, bi_write));
    m.insert("append".into(), mk("fs.append", 2, bi_append));
    m.insert("exists".into(), mk("fs.exists", 1, bi_exists));
    m.insert("remove".into(), mk("fs.remove", 1, bi_remove));
    m.insert("mkdir".into(), mk("fs.mkdir", 1, bi_mkdir));
    m.insert("ls".into(), mk("fs.ls", 1, bi_ls));
    m.insert("stat".into(), mk("fs.stat", 1, bi_stat));
    m
}

fn bi_read(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let path = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("fs.read expects Str path", span))?;
    match std::fs::read_to_string(path) {
        Ok(contents) => Ok(Value::Ok(Box::new(Value::Str(Arc::from(
            contents.as_str(),
        ))))),
        Err(e) => Ok(Value::Err(Box::new(Value::Str(Arc::from(
            e.to_string().as_str(),
        ))))),
    }
}

fn bi_write(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let path = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("fs.write expects Str path", span))?;
    let content = format!("{}", args[1]);
    match std::fs::write(path, content) {
        Ok(()) => Ok(Value::Ok(Box::new(Value::Unit))),
        Err(e) => Ok(Value::Err(Box::new(Value::Str(Arc::from(
            e.to_string().as_str(),
        ))))),
    }
}

fn bi_append(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    use std::io::Write;
    let path = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("fs.append expects Str path", span))?;
    let content = format!("{}", args[1]);
    let result = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .and_then(|mut f| f.write_all(content.as_bytes()));
    match result {
        Ok(()) => Ok(Value::Ok(Box::new(Value::Unit))),
        Err(e) => Ok(Value::Err(Box::new(Value::Str(Arc::from(
            e.to_string().as_str(),
        ))))),
    }
}

fn bi_exists(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let path = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("fs.exists expects Str path", span))?;
    Ok(Value::Bool(std::path::Path::new(path).exists()))
}

fn bi_remove(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let path_str = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("fs.remove expects Str path", span))?;
    let path = std::path::Path::new(path_str);
    let result = if path.is_dir() {
        std::fs::remove_dir_all(path)
    } else {
        std::fs::remove_file(path)
    };
    match result {
        Ok(()) => Ok(Value::Ok(Box::new(Value::Unit))),
        Err(e) => Ok(Value::Err(Box::new(Value::Str(Arc::from(
            e.to_string().as_str(),
        ))))),
    }
}

fn bi_mkdir(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let path = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("fs.mkdir expects Str path", span))?;
    match std::fs::create_dir_all(path) {
        Ok(()) => Ok(Value::Ok(Box::new(Value::Unit))),
        Err(e) => Ok(Value::Err(Box::new(Value::Str(Arc::from(
            e.to_string().as_str(),
        ))))),
    }
}

fn bi_ls(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let path = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("fs.ls expects Str path", span))?;
    match std::fs::read_dir(path) {
        Ok(entries) => {
            let mut items = Vec::new();
            for entry in entries {
                match entry {
                    Ok(e) => items.push(Value::Str(Arc::from(
                        e.file_name().to_string_lossy().as_ref(),
                    ))),
                    Err(e) => {
                        return Ok(Value::Err(Box::new(Value::Str(Arc::from(
                            e.to_string().as_str(),
                        )))));
                    }
                }
            }
            items.sort_by(|a, b| {
                let a_str = a.as_str().unwrap_or("");
                let b_str = b.as_str().unwrap_or("");
                a_str.cmp(b_str)
            });
            Ok(Value::Ok(Box::new(Value::List(Arc::new(items)))))
        }
        Err(e) => Ok(Value::Err(Box::new(Value::Str(Arc::from(
            e.to_string().as_str(),
        ))))),
    }
}

fn bi_stat(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let path = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("fs.stat expects Str path", span))?;
    match std::fs::metadata(path) {
        Ok(meta) => Ok(Value::Ok(Box::new(record! {
            "size" => Value::Int(BigInt::from(meta.len())),
            "is_file" => Value::Bool(meta.is_file()),
            "is_dir" => Value::Bool(meta.is_dir()),
            "readonly" => Value::Bool(meta.permissions().readonly()),
        }))),
        Err(e) => Ok(Value::Err(Box::new(Value::Str(Arc::from(
            e.to_string().as_str(),
        ))))),
    }
}
