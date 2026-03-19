use strum::Display;

use super::{Literal, SExpr, SPattern, SType};

#[derive(Debug, Clone)]
pub struct UseStmt {
    pub path: Vec<String>,
    pub kind: UseKind,
}

#[derive(Debug, Clone)]
pub enum UseKind {
    Whole,
    Alias(String),
    Selective(Vec<String>),
}

#[derive(Debug, Clone)]
pub enum Pattern {
    Literal(Literal),
    Bind(String),
    Wildcard,
    Tuple(Vec<SPattern>),
    List {
        elems: Vec<SPattern>,
        rest: Option<String>,
    },
    Record {
        fields: Vec<FieldPattern>,
        rest: Option<String>,
    },
    Constructor {
        name: String,
        args: Vec<SPattern>,
    },
}

#[derive(Debug, Clone)]
pub enum ProtocolEntry {
    Field(ProtocolField),
    Spread(String),
}

#[derive(Debug, Clone)]
pub struct ProtocolField {
    pub name: String,
    pub type_name: String,
    pub default: Option<SExpr>,
    pub constraint: Option<SExpr>,
}

#[derive(Debug, Clone)]
pub struct ProtocolUnionDef {
    pub name: String,
    pub variants: Vec<String>,
    pub exported: bool,
}

#[derive(Debug, Clone)]
pub struct McpToolDecl {
    pub name: String,
    pub input: Vec<ProtocolField>,
    pub output: McpOutputType,
}

#[derive(Debug, Clone)]
pub enum McpOutputType {
    Named(String),
    List(Box<McpOutputType>),
    Record(Vec<ProtocolField>),
}

#[derive(Debug, Clone)]
pub struct TraitMethodDecl {
    pub name: String,
    pub input: Vec<ProtocolField>,
    pub output: McpOutputType,
}

#[derive(Debug, Clone)]
pub struct AgentMethod {
    pub name: String,
    pub handler: SExpr,
}

#[derive(Debug, Clone)]
pub struct ClassField {
    pub name: String,
    pub default: SExpr,
}

#[derive(Debug, Clone)]
pub struct FieldPattern {
    pub name: String,
    pub pattern: Option<SPattern>,
}

#[derive(Debug, Clone)]
pub enum TypeExpr {
    Named(String),
    Var(String),
    Applied(String, Vec<SType>),
    List(Box<SType>),
    Map { key: Box<SType>, value: Box<SType> },
    Record(Vec<TypeField>),
    Tuple(Vec<SType>),
    Func { param: Box<SType>, ret: Box<SType> },
    Fallible { ok: Box<SType>, err: Box<SType> },
}

#[derive(Debug, Clone)]
pub struct TypeField {
    pub name: String,
    pub ty: SType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
pub enum BinOp {
    #[strum(to_string = "+")]
    Add,
    #[strum(to_string = "-")]
    Sub,
    #[strum(to_string = "*")]
    Mul,
    #[strum(to_string = "/")]
    Div,
    #[strum(to_string = "%")]
    Mod,
    #[strum(to_string = "//")]
    IntDiv,
    #[strum(to_string = "++")]
    Concat,
    #[strum(to_string = "..")]
    Range,
    #[strum(to_string = "..=")]
    RangeInclusive,
    #[strum(to_string = "==")]
    Eq,
    #[strum(to_string = "!=")]
    NotEq,
    #[strum(to_string = "<")]
    Lt,
    #[strum(to_string = ">")]
    Gt,
    #[strum(to_string = "<=")]
    LtEq,
    #[strum(to_string = ">=")]
    GtEq,
    #[strum(to_string = "&&")]
    And,
    #[strum(to_string = "||")]
    Or,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
pub enum UnaryOp {
    #[strum(to_string = "-")]
    Neg,
    #[strum(to_string = "!")]
    Not,
}
