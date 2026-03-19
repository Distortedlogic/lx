mod types;

pub use types::*;

use crate::span::Span;
use num_bigint::BigInt;

#[derive(Debug, Clone)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(node: T, span: Span) -> Self {
        Self { node, span }
    }
}

pub type SExpr = Spanned<Expr>;
pub type SStmt = Spanned<Stmt>;
pub type SPattern = Spanned<Pattern>;
pub type SType = Spanned<TypeExpr>;

#[derive(Debug, Clone)]
pub struct Program {
    pub stmts: Vec<SStmt>,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Binding(Binding),
    TypeDef {
        name: String,
        variants: Vec<(String, usize)>,
        exported: bool,
    },
    Protocol {
        name: String,
        entries: Vec<ProtocolEntry>,
        exported: bool,
    },
    ProtocolUnion(ProtocolUnionDef),
    McpDecl {
        name: String,
        tools: Vec<McpToolDecl>,
        exported: bool,
    },
    TraitDecl {
        name: String,
        methods: Vec<TraitMethodDecl>,
        defaults: Vec<AgentMethod>,
        requires: Vec<String>,
        description: Option<String>,
        tags: Vec<String>,
        exported: bool,
    },
    AgentDecl {
        name: String,
        traits: Vec<String>,
        uses: Vec<(String, String)>,
        init: Option<SExpr>,
        on: Option<SExpr>,
        methods: Vec<AgentMethod>,
        exported: bool,
    },
    ClassDecl {
        name: String,
        traits: Vec<String>,
        fields: Vec<ClassField>,
        methods: Vec<AgentMethod>,
        exported: bool,
    },
    FieldUpdate {
        name: String,
        fields: Vec<String>,
        value: SExpr,
    },
    Use(UseStmt),
    Expr(SExpr),
}

#[derive(Debug, Clone)]
pub struct Binding {
    pub exported: bool,
    pub mutable: bool,
    pub target: BindTarget,
    pub type_ann: Option<SType>,
    pub value: SExpr,
}

#[derive(Debug, Clone)]
pub enum BindTarget {
    Name(String),
    Reassign(String),
    Pattern(SPattern),
}

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(Literal),
    Ident(String),
    TypeConstructor(String),

    Binary {
        op: BinOp,
        left: Box<SExpr>,
        right: Box<SExpr>,
    },
    Unary {
        op: UnaryOp,
        operand: Box<SExpr>,
    },
    Pipe {
        left: Box<SExpr>,
        right: Box<SExpr>,
    },

    Apply {
        func: Box<SExpr>,
        arg: Box<SExpr>,
    },
    Section(Section),

    FieldAccess {
        expr: Box<SExpr>,
        field: FieldKind,
    },

    Block(Vec<SStmt>),
    Tuple(Vec<SExpr>),

    List(Vec<ListElem>),
    Record(Vec<RecordField>),
    Map(Vec<MapEntry>),

    Func {
        params: Vec<Param>,
        ret_type: Option<SType>,
        body: Box<SExpr>,
    },
    Match {
        scrutinee: Box<SExpr>,
        arms: Vec<MatchArm>,
    },
    Ternary {
        cond: Box<SExpr>,
        then_: Box<SExpr>,
        else_: Option<Box<SExpr>>,
    },

    Propagate(Box<SExpr>),
    Coalesce {
        expr: Box<SExpr>,
        default: Box<SExpr>,
    },

    Slice {
        expr: Box<SExpr>,
        start: Option<Box<SExpr>>,
        end: Option<Box<SExpr>>,
    },
    NamedArg {
        name: String,
        value: Box<SExpr>,
    },

    Loop(Vec<SStmt>),
    Break(Option<Box<SExpr>>),
    Assert {
        expr: Box<SExpr>,
        msg: Option<Box<SExpr>>,
    },

    Par(Vec<SStmt>),
    Sel(Vec<SelArm>),

    AgentSend {
        target: Box<SExpr>,
        msg: Box<SExpr>,
    },
    AgentAsk {
        target: Box<SExpr>,
        msg: Box<SExpr>,
    },

    Emit {
        value: Box<SExpr>,
    },
    Yield {
        value: Box<SExpr>,
    },
    With {
        name: String,
        value: Box<SExpr>,
        body: Vec<SStmt>,
        mutable: bool,
    },
    WithResource {
        resources: Vec<(SExpr, String)>,
        body: Vec<SStmt>,
    },
    Refine {
        initial: Box<SExpr>,
        grade: Box<SExpr>,
        revise: Box<SExpr>,
        threshold: Box<SExpr>,
        max_rounds: Box<SExpr>,
        on_round: Option<Box<SExpr>>,
    },
    Receive(Vec<ReceiveArm>),

    Shell {
        mode: ShellMode,
        parts: Vec<StrPart>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellMode {
    Normal,
    Propagate,
    Block,
}

#[derive(Debug, Clone)]
pub enum Literal {
    Int(BigInt),
    Float(f64),
    Str(Vec<StrPart>),
    RawStr(String),
    Regex(String),
    Bool(bool),
    Unit,
}

#[derive(Debug, Clone)]
pub enum StrPart {
    Text(String),
    Interp(SExpr),
}

#[derive(Debug, Clone)]
pub enum Section {
    Right { op: BinOp, operand: Box<SExpr> },
    Left { operand: Box<SExpr>, op: BinOp },
    Field(String),
    Index(i64),
    BinOp(BinOp),
}

#[derive(Debug, Clone)]
pub enum FieldKind {
    Named(String),
    Index(i64),
    Computed(Box<SExpr>),
}

#[derive(Debug, Clone)]
pub enum ListElem {
    Single(SExpr),
    Spread(SExpr),
}

#[derive(Debug, Clone)]
pub struct RecordField {
    pub name: Option<String>,
    pub value: SExpr,
    pub is_spread: bool,
}

#[derive(Debug, Clone)]
pub struct MapEntry {
    pub key: Option<SExpr>,
    pub value: SExpr,
    pub is_spread: bool,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub type_ann: Option<SType>,
    pub default: Option<SExpr>,
}

#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: SPattern,
    pub guard: Option<SExpr>,
    pub body: SExpr,
}

#[derive(Debug, Clone)]
pub struct SelArm {
    pub expr: SExpr,
    pub handler: SExpr,
}

#[derive(Debug, Clone)]
pub struct ReceiveArm {
    pub action: String,
    pub handler: SExpr,
}
