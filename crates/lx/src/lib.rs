pub use lx_span::source as span_source;
pub use lx_span::sym;

pub const PLUGIN_MANIFEST: &str = lx_span::PLUGIN_MANIFEST;
pub const LX_MANIFEST: &str = lx_span::LX_MANIFEST;

pub mod ast;
pub mod builtins;
pub mod checker;
pub mod env;
pub mod error;
pub mod event_stream;
pub mod folder;
pub mod formatter;
pub mod interpreter;
pub mod lexer;
pub mod linter;
pub mod mcp_client;
pub mod parser;
pub mod runtime;
pub mod source;
pub mod stdlib;
pub mod tool_module;
pub mod value;
pub mod visitor;
