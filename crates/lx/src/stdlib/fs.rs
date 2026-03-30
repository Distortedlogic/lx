use std::fs;
use std::path::Path;
use std::sync::Arc;

use indexmap::IndexMap;

use crate::BuiltinCtx;
use crate::error::LxError;
use crate::record;
use crate::std_module;
use crate::stdlib::helpers::wrap_io;
use crate::sym::Sym;
use crate::value::LxVal;
use miette::SourceSpan;

pub fn build() -> IndexMap<Sym, LxVal> {
  std_module! {
    "read"   => "fs.read",   1, bi_read;
    "write"  => "fs.write",  2, bi_write;
    "append" => "fs.append", 2, bi_append;
    "exists" => "fs.exists", 1, bi_exists;
    "remove" => "fs.remove", 1, bi_remove;
    "rm"     => "fs.rm",     1, bi_rm;
    "rmdir"  => "fs.rmdir",  1, bi_rmdir;
    "mkdir"  => "fs.mkdir",  1, bi_mkdir;
    "ls"     => "fs.ls",     1, bi_ls;
    "stat"   => "fs.stat",   1, bi_stat
  }
}

fn bi_read(args: &[LxVal], span: SourceSpan, _ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  let path = args[0].require_str("fs.read", span)?;
  Ok(wrap_io(fs::read_to_string(path).map(LxVal::str)))
}

fn bi_write(args: &[LxVal], span: SourceSpan, _ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  let path = args[0].require_str("fs.write", span)?;
  let content = args[1].to_string();
  Ok(wrap_io(fs::write(path, content).map(|()| LxVal::Unit)))
}

fn bi_append(args: &[LxVal], span: SourceSpan, _ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  use std::io::Write;
  let path = args[0].require_str("fs.append", span)?;
  let content = args[1].to_string();
  let result = fs::OpenOptions::new().create(true).append(true).open(path).and_then(|mut f| f.write_all(content.as_bytes()));
  Ok(wrap_io(result.map(|()| LxVal::Unit)))
}

fn bi_exists(args: &[LxVal], span: SourceSpan, _ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  let path = args[0].require_str("fs.exists", span)?;
  Ok(LxVal::Bool(Path::new(path).exists()))
}

fn bi_remove(args: &[LxVal], span: SourceSpan, _ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  let path_str = args[0].require_str("fs.remove", span)?;
  let path = Path::new(path_str);
  let result = if path.is_dir() { fs::remove_dir_all(path) } else { fs::remove_file(path) };
  Ok(wrap_io(result.map(|()| LxVal::Unit)))
}

fn bi_rm(args: &[LxVal], span: SourceSpan, _ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  let path_str = args[0].require_str("fs.rm", span)?;
  let path = Path::new(path_str);
  if !path.exists() {
    return Ok(LxVal::ok(LxVal::Unit));
  }
  Ok(wrap_io(fs::remove_file(path).map(|()| LxVal::Unit)))
}

fn bi_rmdir(args: &[LxVal], span: SourceSpan, _ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  let path_str = args[0].require_str("fs.rmdir", span)?;
  let path = Path::new(path_str);
  if !path.exists() {
    return Ok(LxVal::ok(LxVal::Unit));
  }
  Ok(wrap_io(fs::remove_dir_all(path).map(|()| LxVal::Unit)))
}

fn bi_mkdir(args: &[LxVal], span: SourceSpan, _ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  let path = args[0].require_str("fs.mkdir", span)?;
  Ok(wrap_io(fs::create_dir_all(path).map(|()| LxVal::Unit)))
}

fn bi_ls(args: &[LxVal], span: SourceSpan, _ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  let path = args[0].require_str("fs.ls", span)?;
  let entries = match fs::read_dir(path) {
    Ok(entries) => entries,
    Err(e) => return Ok(wrap_io::<LxVal>(Err(e))),
  };
  let mut names: Vec<String> = Vec::new();
  for entry in entries {
    match entry {
      Ok(e) => names.push(e.file_name().to_string_lossy().into_owned()),
      Err(e) => return Ok(wrap_io::<LxVal>(Err(e))),
    }
  }
  names.sort();
  Ok(LxVal::ok(LxVal::list(names.into_iter().map(LxVal::str).collect())))
}

fn bi_stat(args: &[LxVal], span: SourceSpan, _ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  let path = args[0].require_str("fs.stat", span)?;
  Ok(wrap_io(fs::metadata(path).map(|meta| {
    record! {
        "size" => LxVal::int(meta.len()),
        "is_file" => LxVal::Bool(meta.is_file()),
        "is_dir" => LxVal::Bool(meta.is_dir()),
        "readonly" => LxVal::Bool(meta.permissions().readonly()),
    }
  })))
}
