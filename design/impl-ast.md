# AST Design

The abstract syntax tree produced by the parser. One enum for expressions, one for statements, one for patterns, one for types.

Implements: [grammar.md](../spec/grammar.md)

## Core Types

```rust
struct Program {
    stmts: Vec<Stmt>,
}

struct Spanned<T> {
    node: T,
    span: Span,
}

type SExpr = Spanned<Expr>;
type SPattern = Spanned<Pattern>;
type SType = Spanned<TypeExpr>;
```

Every AST node is wrapped in `Spanned` to carry source location for diagnostics.

## Expr

```rust
enum Expr {
    Literal(Literal),
    Ident(String),
    TypeConstructor(String),

    Binary { op: BinOp, left: Box<SExpr>, right: Box<SExpr> },
    Unary { op: UnaryOp, operand: Box<SExpr> },
    Pipe { left: Box<SExpr>, right: Box<SExpr> },

    Apply { func: Box<SExpr>, arg: Box<SExpr> },
    Section(Section),
    Compose { left: Box<SExpr>, right: Box<SExpr> },

    FieldAccess { expr: Box<SExpr>, field: FieldKind },
    Slice { expr: Box<SExpr>, from: Option<Box<SExpr>>, to: Option<Box<SExpr>> },

    Block(Vec<Spanned<Stmt>>),
    Tuple(Vec<SExpr>),

    List(Vec<ListElem>),
    Record(Vec<RecordField>),
    Map(Vec<MapEntry>),
    Set(Vec<SetElem>),

    Func { params: Vec<Param>, ret_type: Option<SType>, body: Box<SExpr> },
    Match { scrutinee: Box<SExpr>, arms: Vec<MatchArm> },
    Ternary { cond: Box<SExpr>, then_: Box<SExpr>, else_: Option<Box<SExpr>> },

    Propagate(Box<SExpr>),
    Coalesce { expr: Box<SExpr>, default: Box<SExpr> },

    Shell(ShellExpr),
    Par(Vec<Spanned<Stmt>>),
    Sel(Vec<SelArm>),

    Loop(Vec<Spanned<Stmt>>),
    Break(Option<Box<SExpr>>),
    Assert { expr: Box<SExpr>, msg: Option<Box<SExpr>> },

    Dbg(Box<SExpr>),
}
```

## Supporting Types

```rust
enum Literal {
    Int(BigInt),
    Float(f64),
    Str(Vec<StrPart>),
    RawStr(String),
    Regex(String),  // pattern with flags prepended as (?flags)
    Bool(bool),
    Unit,
}

enum StrPart {
    Text(String),
    Interp(SExpr),
}

enum Section {
    Right { op: BinOp, operand: Box<SExpr> },
    Left { operand: Box<SExpr>, op: BinOp },
    Field(String),
}

enum FieldKind {
    Named(String),
    Index(i64),
    Computed(Box<SExpr>),
}

enum ListElem {
    Single(SExpr),
    Spread(SExpr),
}

struct RecordField {
    name: String,
    value: Option<SExpr>,
    is_spread: bool,
}

struct MapEntry {
    key: SExpr,
    value: Option<SExpr>,
    is_spread: bool,
}

enum SetElem {
    Single(SExpr),
    Spread(SExpr),
}

struct Param {
    pattern: SPattern,
    type_ann: Option<SType>,
    default: Option<SExpr>,
}

struct MatchArm {
    pattern: SPattern,
    guard: Option<SExpr>,
    body: SExpr,
}

struct SelArm {
    expr: SExpr,
    handler: SExpr,
}
```

## ShellExpr

```rust
struct ShellExpr {
    kind: ShellKind,
    parts: Vec<ShellPart>,
}

enum ShellKind {
    Interpolated,
    Raw,
    Propagating,
    Block,
}

enum ShellPart {
    Text(String),
    Interp(SExpr),
}
```

## Stmt

```rust
enum Stmt {
    Binding(Binding),
    TypeDef(TypeDefStmt),
    Use(UseStmt),
    Expr(SExpr),
}

struct TypeDefStmt {
    exported: bool,
    name: String,
    params: Vec<String>,
    def: TypeDef,
}

enum TypeDef {
    Record(Vec<(String, SType)>),
    Union(Vec<UnionVariant>),
}

struct UnionVariant {
    name: String,
    fields: Vec<SType>,
}

struct Binding {
    exported: bool,
    mutable: bool,
    target: BindTarget,
    type_ann: Option<SType>,
    value: SExpr,
}

enum BindTarget {
    Name(String),
    Pattern(SPattern),
    Reassign(String),
}

struct UseStmt {
    path: Vec<String>,
    relative: Option<RelativeKind>,
    alias: Option<String>,
    selective: Option<Vec<String>>,
}

enum RelativeKind { Current, Parent(usize) }
```

## Pattern

```rust
enum Pattern {
    Literal(Literal),
    Bind(String),
    Wildcard,
    Tuple(Vec<SPattern>),
    List { elems: Vec<SPattern>, rest: Option<String> },
    Record { fields: Vec<FieldPattern>, rest: Option<String> },
    Constructor { name: String, args: Vec<SPattern> },
}

struct FieldPattern {
    name: String,
    pattern: Option<SPattern>,
}
```

## TypeExpr

```rust
enum TypeExpr {
    Named(String),
    Generic { name: String, args: Vec<SType> },
    List(Box<SType>),
    Record(Vec<(String, SType)>),
    Map { key: Box<SType>, value: Box<SType> },
    Set(Box<SType>),
    Tuple(Vec<SType>),
    Func { param: Box<SType>, ret: Box<SType> },
    Fallible { ok: Box<SType>, err: Box<SType> },
    Var(String),
}
```

## BinOp / UnaryOp

```rust
enum BinOp {
    Add, Sub, Mul, Div, Mod, IntDiv,
    Concat, Range, RangeInclusive,
    Eq, NotEq, Lt, Gt, LtEq, GtEq,
    And, Or,
}

enum UnaryOp { Neg, Not }
```

## Design Notes

`Apply` is always single-argument: `f x y` parses as `Apply(Apply(f, x), y)`. Multi-arg application is represented as nested single-arg applications. This makes currying natural — partially applied functions are just `Apply` nodes with fewer arguments than the function's arity.

`Pipe` is a separate node (not `Apply`) because the pipe threading logic (data-last insertion) is different from normal application. The interpreter handles `a | f` as `Apply(f, a)` but the AST preserves the distinction for diagnostics and formatting.

`Dbg` is an AST node, not a function call, because it captures the source text of its argument at compile time for display.

## Cross-References

- Parser that produces this AST: [impl-parser.md](impl-parser.md)
- Token types consumed: [impl-lexer.md](impl-lexer.md)
- Type checker that annotates this AST: [impl-checker.md](impl-checker.md)
- Interpreter that evaluates this AST: [impl-interpreter.md](impl-interpreter.md)
