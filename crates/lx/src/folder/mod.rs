pub mod desugar;
mod desugar_http;
mod desugar_mcp_cli;
mod desugar_schema;
pub(crate) mod gen_ast;
mod validate_core;

pub use desugar::desugar;
