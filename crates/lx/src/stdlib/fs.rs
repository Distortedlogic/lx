use std::sync::Arc;

use indexmap::IndexMap;

use crate::builtins::mk;
use crate::error::LxError;
use crate::record;
use crate::runtime::RuntimeCtx;
use crate::span::Span;
use crate::value::LxVal;

pub fn build() -> IndexMap<String, LxVal> {
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

fn bi_read(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let path = args[0].require_str("fs.read", span)?;
  match std::fs::read_to_string(path) {
    Ok(contents) => Ok(LxVal::Ok(Box::new(LxVal::str(contents)))),
    Err(e) => Ok(LxVal::Err(Box::new(LxVal::str(e.to_string())))),
  }
}

fn bi_write(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let path = args[0].require_str("fs.write", span)?;
  let content = format!("{}", args[1]);
  match std::fs::write(path, content) {
    Ok(()) => Ok(LxVal::Ok(Box::new(LxVal::Unit))),
    Err(e) => Ok(LxVal::Err(Box::new(LxVal::str(e.to_string())))),
  }
}

fn bi_append(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  use std::io::Write;
  let path = args[0].require_str("fs.append", span)?;
  let content = format!("{}", args[1]);
  let result = std::fs::OpenOptions::new().create(true).append(true).open(path).and_then(|mut f| f.write_all(content.as_bytes()));
  match result {
    Ok(()) => Ok(LxVal::Ok(Box::new(LxVal::Unit))),
    Err(e) => Ok(LxVal::Err(Box::new(LxVal::str(e.to_string())))),
  }
}

fn bi_exists(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let path = args[0].require_str("fs.exists", span)?;
  Ok(LxVal::Bool(std::path::Path::new(path).exists()))
}

fn bi_remove(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let path_str = args[0].require_str("fs.remove", span)?;
  let path = std::path::Path::new(path_str);
  let result = if path.is_dir() { std::fs::remove_dir_all(path) } else { std::fs::remove_file(path) };
  match result {
    Ok(()) => Ok(LxVal::Ok(Box::new(LxVal::Unit))),
    Err(e) => Ok(LxVal::Err(Box::new(LxVal::str(e.to_string())))),
  }
}

fn bi_mkdir(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let path = args[0].require_str("fs.mkdir", span)?;
  match std::fs::create_dir_all(path) {
    Ok(()) => Ok(LxVal::Ok(Box::new(LxVal::Unit))),
    Err(e) => Ok(LxVal::Err(Box::new(LxVal::str(e.to_string())))),
  }
}

fn bi_ls(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let path = args[0].require_str("fs.ls", span)?;
  match std::fs::read_dir(path) {
    Ok(entries) => {
      let mut items = Vec::new();
      for entry in entries {
        match entry {
          Ok(e) => items.push(LxVal::str(e.file_name().to_string_lossy())),
          Err(e) => {
            return Ok(LxVal::Err(Box::new(LxVal::str(e.to_string()))));
          },
        }
      }
      items.sort_by(|a, b| {
        let a_str = a.as_str().unwrap_or("");
        let b_str = b.as_str().unwrap_or("");
        a_str.cmp(b_str)
      });
      Ok(LxVal::Ok(Box::new(LxVal::list(items))))
    },
    Err(e) => Ok(LxVal::Err(Box::new(LxVal::str(e.to_string())))),
  }
}

fn bi_stat(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let path = args[0].require_str("fs.stat", span)?;
  match std::fs::metadata(path) {
    Ok(meta) => Ok(LxVal::Ok(Box::new(record! {
        "size" => LxVal::int(meta.len()),
        "is_file" => LxVal::Bool(meta.is_file()),
        "is_dir" => LxVal::Bool(meta.is_dir()),
        "readonly" => LxVal::Bool(meta.permissions().readonly()),
    }))),
    Err(e) => Ok(LxVal::Err(Box::new(LxVal::str(e.to_string())))),
  }
}
