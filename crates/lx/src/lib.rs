pub use lx_span::sym;

pub mod source {
  pub use lx_ast::source::*;
  pub use lx_span::source::*;
}

pub use lx_ast::ast;
pub use lx_ast::visitor;

pub use lx_parser::lexer;
pub use lx_parser::parser;

pub use lx_desugar::folder;

pub use lx_fmt::formatter;

pub mod checker {
  pub use lx_checker::*;
}

pub mod linter {
  pub use lx_linter::*;
}

pub mod value {
  pub use lx_value::*;
}

pub mod env {
  pub use lx_value::Env;
}

pub mod error {
  pub use lx_value::error::*;
}

pub mod event_stream {
  pub use lx_value::{EventStream, SpanInfo, StreamEntry, entry_to_lxval};
}

pub use lx_value::{BuiltinCtx, ExternalStreamSink, ModuleExports, ToolModuleHandle, record};

pub use lx_eval::builtins;
pub use lx_eval::interpreter;
pub use lx_eval::mcp_client;
pub use lx_eval::mcp_stream_sink;
pub use lx_eval::runtime;
pub use lx_eval::stdlib;
pub use lx_eval::tool_module;
pub use lx_eval::{LX_MANIFEST, PLUGIN_MANIFEST};
