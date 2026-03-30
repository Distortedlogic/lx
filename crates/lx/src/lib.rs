pub use lx_ast::ast;
pub use lx_ast::visitor;
pub use lx_desugar::folder;
pub use lx_fmt::formatter;
pub use lx_parser::lexer;
pub use lx_parser::parser;
pub use lx_span::sym;

pub use lx_eval::{LX_MANIFEST, PLUGIN_MANIFEST};

pub use lx_value::{BuiltinCtx, ExternalStreamSink, ModuleExports, ToolModuleHandle, record};

pub mod builtins;
pub mod checker;
pub mod env;
pub mod error;
pub mod event_stream;
pub mod interpreter;
pub mod linter;
pub mod mcp_client;
pub mod mcp_stream_sink;
pub mod runtime;
pub mod source;
pub mod stdlib;
pub mod tool_module;
pub mod value;
