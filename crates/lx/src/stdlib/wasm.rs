use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use std::sync::{Arc, LazyLock, Mutex, RwLock};

use indexmap::IndexMap;
use miette::SourceSpan;

use crate::error::LxError;
use crate::interpreter::ModuleExports;
use crate::sym::{Sym, intern};
use crate::value::{DynAsyncBuiltinFn, LxVal, mk_dyn_async};

use super::wasm_marshal::{json_to_lxval, lxval_to_json};

static PLUGINS: LazyLock<RwLock<HashMap<String, Mutex<extism::Plugin>>>> = LazyLock::new(|| RwLock::new(HashMap::new()));

#[derive(serde::Deserialize)]
struct PluginToml {
  plugin: PluginMeta,
  exports: HashMap<String, ExportDef>,
  #[serde(default)]
  sandbox: Option<SandboxConfig>,
}

#[derive(serde::Deserialize)]
struct PluginMeta {
  name: String,
  wasm: String,
}

#[derive(Default, serde::Deserialize)]
#[serde(default)]
struct SandboxConfig {
  wasi: Option<bool>,
  fuel: Option<u64>,
}

#[derive(serde::Deserialize)]
struct ExportDef {
  arity: usize,
}

pub(crate) fn load_plugin(name: &str, plugin_dir: &Path, span: SourceSpan) -> Result<ModuleExports, LxError> {
  let toml_path = plugin_dir.join(crate::PLUGIN_MANIFEST);
  let toml_str = fs::read_to_string(&toml_path).map_err(|e| LxError::runtime(format!("plugin '{name}': cannot read {}: {e}", toml_path.display()), span))?;

  let manifest: PluginToml = toml::from_str(&toml_str).map_err(|e| LxError::runtime(format!("plugin '{name}': invalid plugin.toml: {e}"), span))?;

  let wasm_path = plugin_dir.join(&manifest.plugin.wasm);
  if !wasm_path.exists() {
    return Err(LxError::runtime(format!("plugin '{name}': wasm file not found: {}", wasm_path.display()), span));
  }

  let wasi_enabled = manifest.sandbox.as_ref().and_then(|s| s.wasi).unwrap_or(false);

  let extism_manifest = extism::Manifest::new([extism::Wasm::file(&wasm_path)]);

  let log_fn = extism::Function::new(
    "plugin_log",
    [extism::PTR, extism::PTR],
    [],
    extism::UserData::new(()),
    |plugin: &mut extism::CurrentPlugin, inputs: &[extism::Val], _outputs: &mut [extism::Val], _ud: extism::UserData<()>| {
      let level: u32 = plugin.memory_get_val(&inputs[0])?;
      let msg: String = plugin.memory_get_val(&inputs[1])?;
      let label = match level {
        0 => "TRACE",
        1 => "DEBUG",
        2 => "INFO",
        3 => "WARN",
        _ => "ERROR",
      };
      eprintln!("[plugin:{label}] {msg}");
      Ok(())
    },
  );

  let config_fn = extism::Function::new(
    "plugin_get_config",
    [extism::PTR],
    [extism::PTR],
    extism::UserData::new(()),
    |plugin: &mut extism::CurrentPlugin, inputs: &[extism::Val], outputs: &mut [extism::Val], _ud: extism::UserData<()>| {
      let key: String = plugin.memory_get_val(&inputs[0])?;
      let val = env::var(&key).unwrap_or_default();
      let handle = plugin.memory_new(&val)?;
      outputs[0] = plugin.memory_to_val(handle);
      Ok(())
    },
  );

  let mut builder = extism::PluginBuilder::new(&extism_manifest).with_wasi(wasi_enabled).with_functions([log_fn, config_fn]);

  if let Some(fuel) = manifest.sandbox.as_ref().and_then(|s| s.fuel) {
    builder = builder.with_fuel_limit(fuel);
  }

  let plugin = builder.build().map_err(|e| LxError::runtime(format!("plugin '{name}' ({}): failed to load: {e}", wasm_path.display()), span))?;

  {
    let mut plugins = PLUGINS.write().map_err(|e| LxError::runtime(format!("plugin lock poisoned: {e}"), span))?;
    plugins.insert(manifest.plugin.name.clone(), Mutex::new(plugin));
  }

  let mut bindings: IndexMap<Sym, LxVal> = IndexMap::new();

  for (fn_name, def) in &manifest.exports {
    let plugin_name = manifest.plugin.name.clone();
    let fn_name_owned = fn_name.clone();
    let builtin_name: &'static str = Box::leak(format!("wasm/{plugin_name}.{fn_name}").into_boxed_str());

    let closure: DynAsyncBuiltinFn = Arc::new(move |args: Vec<LxVal>, call_span: miette::SourceSpan, _ctx| {
      let p_name = plugin_name.clone();
      let f_name = fn_name_owned.clone();
      Box::pin(async move { call_plugin_fn(&p_name, &f_name, &args, call_span) })
    });

    let builtin = mk_dyn_async(builtin_name, def.arity, closure);
    bindings.insert(intern(fn_name), builtin);
  }

  Ok(ModuleExports { bindings, variant_ctors: Vec::new() })
}

fn call_plugin_fn(plugin_name: &str, fn_name: &str, args: &[LxVal], span: SourceSpan) -> Result<LxVal, LxError> {
  let json_input = if args.len() == 1 {
    lxval_to_json(&args[0])?
  } else {
    let list = LxVal::list(args.to_vec());
    lxval_to_json(&list)?
  };

  let plugins = PLUGINS.read().map_err(|e| LxError::runtime(format!("plugin lock poisoned: {e}"), span))?;

  let plugin_mutex = plugins.get(plugin_name).ok_or_else(|| LxError::runtime(format!("plugin '{plugin_name}' not loaded"), span))?;

  let mut plugin = plugin_mutex.lock().map_err(|e| LxError::runtime(format!("plugin '{plugin_name}' mutex poisoned: {e}"), span))?;

  let output: String = plugin.call::<&str, String>(fn_name, &json_input).map_err(|e| LxError::runtime(format!("wasm/{plugin_name}.{fn_name}: {e}"), span))?;

  json_to_lxval(&output)
}
