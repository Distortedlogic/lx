# AST Design Across Programming Languages

Research on how major language implementations represent abstract syntax trees, intermediate representations, and the tradeoffs between concrete and abstract syntax trees.

## Table of Contents

1. [Python: ASDL-Driven AST](#python-asdl-driven-ast)
2. [ASDL (Zephyr Abstract Syntax Description Language)](#asdl-zephyr-abstract-syntax-description-language)
3. [Rust: Multi-Level IR Pipeline](#rust-multi-level-ir-pipeline)
4. [Rust: syn Crate for Proc Macros](#rust-syn-crate-for-proc-macros)
5. [TypeScript: ts.Node and SyntaxKind](#typescript-tsnode-and-syntaxkind)
6. [Go: go/ast Package](#go-goast-package)
7. [Lua: No-AST Compiler and LuaJIT SSA IR](#lua-no-ast-compiler-and-luajit-ssa-ir)
8. [Ruby: Prism Parser](#ruby-prism-parser)
9. [CST vs AST](#cst-vs-ast)
10. [AST Design Patterns](#ast-design-patterns)

---

## Python: ASDL-Driven AST

Sources: [ast module docs](https://docs.python.org/3/library/ast.html), [CPython compiler internals](https://github.com/python/cpython/blob/main/InternalDocs/compiler.md), [PEP 339](https://peps.python.org/pep-0339/), [Eli Bendersky on ASDL](https://eli.thegreenplace.net/2014/06/04/using-asdl-to-describe-asts-in-compilers)

### ASDL Schema

CPython defines its AST using ASDL in `Parser/Python.asdl`. The schema is the single source of truth -- a code generator (`Parser/asdl_c.py`) reads this file and emits C structs in `Include/Python-ast.h` and `Python/Python-ast.c`, plus the Python-level `ast` module classes. The ASDL grammar looks like:

```
module Python {
    mod = Module(stmt* body, type_ignore* type_ignores)
        | Interactive(stmt* body)
        | Expression(expr body)
        | FunctionType(expr* argtypes, expr returns)

    stmt = FunctionDef(identifier name, arguments args, stmt* body,
                       expr* decorator_list, expr? returns, string? type_comment,
                       type_param* type_params)
         | AsyncFunctionDef(...)
         | ClassDef(...)
         | Assign(expr* targets, expr value, string? type_comment)
         | ...

    expr = BinOp(expr left, operator op, expr right)
         | UnaryOp(unaryop op, expr operand)
         | Call(expr func, expr* args, keyword* keywords)
         | Name(identifier id, expr_context ctx)
         | Constant(constant value, string? kind)
         | ...
}
```

ASDL has four builtin types: `identifier`, `int`, `string`, `constant`. Multiplicity markers: `*` = zero-or-more, `?` = optional, bare = required. Every left-hand symbol (e.g., `stmt`, `expr`) becomes an abstract base class; every right-hand constructor (e.g., `BinOp`, `Assign`) becomes a concrete class inheriting from it. Every concrete class has a `_fields` attribute listing child node names.

### Node Hierarchy

```
ast.AST (base)
  ast.mod       -- Module, Interactive, Expression, FunctionType
  ast.stmt      -- FunctionDef, ClassDef, Assign, AugAssign, Return, ...
  ast.expr      -- BinOp, UnaryOp, Call, Name, Constant, Lambda, ...
  ast.operator   -- Add, Sub, Mult, Div, ...
  ast.cmpop      -- Eq, NotEq, Lt, Gt, ...
  ast.expr_context -- Load, Store, Del
```

Every node carries `lineno`, `col_offset`, `end_lineno`, `end_col_offset` for source mapping.

### Core API

| Function | Purpose |
|----------|---------|
| `ast.parse(source, mode='exec')` | Parse source string to AST. `mode` can be `'exec'`, `'eval'`, `'single'`, `'func_type'`. Accepts `feature_version` to parse with older grammar. |
| `ast.compile()` / `compile(tree, ...)` | Compile AST to bytecode object. |
| `ast.unparse(node)` | Convert AST back to Python source (not identical to original -- whitespace/comments lost). |
| `ast.dump(node, indent=2)` | Pretty-print tree structure. |
| `ast.walk(node)` | Recursively yield all descendant nodes (breadth-first). |
| `ast.fix_missing_locations(tree)` | Copy location info to generated nodes that lack it. |
| `ast.literal_eval(source)` | Safely evaluate literals only (no code execution). |

### Version Changes

- **3.8**: `ast.Constant` unifies all literal types. `NamedExpr` added (walrus operator). `Num`, `Str`, `Bytes`, `NameConstant`, `Ellipsis` deprecated.
- **3.9**: `ast.unparse()` added. `Index` and `ExtSlice` removed (simple indexing uses value directly).
- **3.10**: Pattern matching nodes added (`Match`, `MatchAs`, `MatchOr`, `MatchSequence`, `MatchMapping`, `MatchClass`, `MatchValue`, `MatchStar`, `MatchSingleton`). `TryStar` for `except*`.
- **3.12**: Type parameter support (`TypeAlias`, `TypeVar`, `ParamSpec`, `TypeVarTuple`). `FunctionDef.type_params`, `ClassDef.type_params`.
- **3.13**: `_field_types` attribute on nodes. `optimize` parameter on `ast.parse()`.
- **3.14**: Old constant classes (`Num`, `Str`, etc.) fully removed. `ast.compare()` function added for recursive AST comparison.

---

## ASDL (Zephyr Abstract Syntax Description Language)

Source: [Eli Bendersky](https://eli.thegreenplace.net/2014/06/04/using-asdl-to-describe-asts-in-compilers)

ASDL is a domain-specific language for describing tree-like data structures used in compilers. It was created as part of the Zephyr project to enable compiler components written in different languages to interoperate via a shared AST schema.

### Syntax

```
program = Program(class* classes)
class   = Class(identifier name, identifier? parent, feature* features)
expr    = Assign(identifier name, expr value)
        | Dispatch(expr obj, identifier method, expr* args)
        | If(expr pred, expr then_branch, expr else_branch)
        | IntConst(int value)
```

- Pipe (`|`) separates variant constructors (sum types).
- `*` = sequence, `?` = optional, bare = required singleton.
- Constructors define product types (named fields).

### Benefits of Schema-Driven AST Generation

1. **Single source of truth**: The `.asdl` file is the authoritative definition. C structs, Python classes, serialization code, and visitor stubs are all generated from it.
2. **Cross-language interop**: Multiple languages can generate compatible AST representations from the same schema.
3. **Type safety**: Generated constructors enforce that nodes contain only defined attributes of specified types.
4. **Automatic boilerplate**: Constructors, accessors, serialization, pretty-printing, and visitor dispatch are generated, not hand-written.
5. **Evolvability**: Adding a new node type means editing one line in the `.asdl` file; the generator propagates the change everywhere.

CPython has used ASDL since Python 2.5. The original SPARK-based ASDL parser was later replaced with a simpler recursive-descent parser to eliminate external dependencies.

---

## Rust: Multi-Level IR Pipeline

Sources: [rustc dev guide overview](https://rustc-dev-guide.rust-lang.org/overview.html), [HIR](https://rustc-dev-guide.rust-lang.org/hir.html), [THIR](https://rustc-dev-guide.rust-lang.org/thir.html), [MIR](https://rustc-dev-guide.rust-lang.org/mir/index.html), [MIR RFC](https://github.com/nox/rust-rfcs/blob/master/text/1211-mir.md)

Rust's compiler has the most layered IR pipeline of any mainstream language. Each level exists because it desugars or restructures something the previous level could not efficiently analyze.

### Pipeline

```
Source → Tokens → AST → HIR → THIR → MIR → LLVM IR → Machine Code
```

### Stage 1: AST (`rustc_ast`)

The parser produces `rustc_ast::ast::{Crate, Expr, Pat, ...}` via recursive descent. The AST preserves full syntactic fidelity: exact token positions, macro invocations as-written, syntactic sugar, and all surface syntax. Macro expansion, AST validation, name resolution, and early linting happen at this stage. String literals are interned into `Symbol` values by the lexer for memory efficiency.

### Stage 2: HIR (High-Level IR) -- `rustc_hir`

"Lowering" from AST to HIR desugars:

- **`for` loops** become `loop` + `match` on iterator's `next()`
- **`async fn`** becomes a function returning `impl Future` wrapping a generator
- **Elided lifetimes** become explicit lifetime parameters
- **`if let`** becomes `match`
- **`?` operator** becomes `match` on `Try::branch()`

Key data structures:

| Type | Purpose |
|------|---------|
| `DefId` | Identifies a top-level definition across crates (crate number + index) |
| `LocalDefId` | `DefId` guaranteed to be from the current crate |
| `HirId` | Identifies any HIR node (owner + local id) -- expressions, patterns, etc. |
| `BodyId` | Wraps `HirId` to reference executable code blocks |
| `ItemId` | References items within a module |

Items are not nested directly inside parents. A module holds only `ItemId` references; you look up the actual item via a map. This enables incremental compilation by making data access observable through queries.

Type checking and trait resolution operate on the HIR. The central `TyCtxt` struct ("typing context") holds all queries and cached results. `ty::Ty` represents all types in the program and is interned.

### Stage 3: THIR (Typed High-Level IR)

THIR is generated after type checking succeeds. It is like HIR but:

- Fully typed (every node has a resolved type)
- Method calls become explicit function calls with resolved `DefId`
- Implicit dereferences (`*`) and autoref (`&`) are made explicit
- Overloaded operators become function calls
- Destruction scopes are explicit

THIR only represents function bodies and const initializers (no struct/trait definitions). Each body exists transiently in memory and is dropped after use, keeping peak memory low.

Used for: MIR construction, exhaustiveness checking (match arms), unsafety checking.

Debug output: `rustc -Zunpretty=thir-tree`

### Stage 4: MIR (Mid-Level IR) -- `rustc_middle::mir`

MIR is a control-flow graph. All high-level control flow (loops, matches, `?`, `if let`) has been decomposed into basic blocks connected by edges.

**Basic Block structure:**

```
bb0: {
    StorageLive(_1);                  // Statement
    _1 = const 42_i32;               // Statement
    _2 = Add(_1, const 1_i32);       // Statement
    switchInt(_2) -> [0: bb1, otherwise: bb2]; // Terminator
}
```

- **Statements**: Sequential actions with exactly one successor. Examples: `StorageLive`, `StorageDead`, `Assign`.
- **Terminators**: Actions with potentially multiple successors. Examples: `switchInt`, `Call` (which has `return` and `unwind` successors), `Return`, `Drop`.
- **Places**: Memory locations -- `_1` (local), `_1.f` (field), `*_1` (deref).
- **Rvalues**: Value-producing operations. Cannot be nested -- they reference only Places and constants.
- **Operands**: Either `copy _1` or `move _1` or a constant.

MIR is used for: borrow checking (flow-sensitive lifetime analysis), const evaluation, optimization passes, and finally monomorphization + code generation to LLVM IR.

### Stage 5: LLVM IR

After monomorphization (replacing generic type parameters with concrete types), MIR is translated to LLVM IR -- "a sort of typed assembly language with lots of annotations." LLVM then runs its own optimization passes and emits machine code.

### Why Each Level Exists

| Level | Primary Consumer | What It Removes |
|-------|-----------------|-----------------|
| AST | Macros, name resolution | Nothing (faithful to source) |
| HIR | Type checker, trait solver | Syntactic sugar (`for`, `?`, `if let`, `async`) |
| THIR | Exhaustiveness checker, unsafety checker | Implicit coercions, method dispatch |
| MIR | Borrow checker, optimizer | Structured control flow (→ CFG) |
| LLVM IR | LLVM optimizer, codegen | Generics (monomorphized) |

---

## Rust: syn Crate for Proc Macros

Sources: [syn docs](https://docs.rs/syn/latest/syn/), [syn GitHub](https://github.com/dtolnay/syn), [syn::visit](https://docs.rs/syn/latest/syn/visit/index.html)

The `syn` crate provides a complete Rust syntax tree for use in procedural macros. It parses `TokenStream` input into typed AST nodes.

### Key Types

| Type | What It Represents |
|------|-------------------|
| `syn::File` | A complete source file (top-level entry point) |
| `syn::Item` | Any item: `fn`, `struct`, `enum`, `impl`, `mod`, `use`, etc. |
| `syn::DeriveInput` | Any of the three legal derive macro inputs (struct, enum, union) |
| `syn::Expr` | Any expression (200+ variants behind feature flags) |
| `syn::Type` | Any type expression |
| `syn::Pat` | Any pattern |
| `syn::Stmt` | Statement within a block |
| `syn::ItemFn` | Function item with `attrs`, `vis`, `sig`, `block` |

### Parsing Model

Every syntax tree node implements `Parse`: `fn(ParseStream) -> Result<T>`. Nodes can be composed as building blocks for custom syntaxes. The typical proc-macro workflow:

1. `parse_macro_input!(input as DeriveInput)` -- parse `TokenStream` into typed AST
2. Inspect/transform the AST
3. `quote! { ... }` -- generate output `TokenStream`

### Visitor Traits

syn provides three traversal traits (behind feature flags):

- **`Visit<'ast>`**: Read-only traversal over `&'ast` references. 150+ `visit_*` methods.
- **`VisitMut`**: Mutable traversal over `&mut` references. Same method set.
- **`Fold`**: Ownership-taking traversal that returns transformed nodes. Slower than `VisitMut` because it takes ownership and requires returning the new value.

Each `visit_*` method has a corresponding free function (e.g., `visit::visit_expr_binary`) that provides the default recursive behavior. Override a method to customize; call the free function to continue recursion.

---

## TypeScript: ts.Node and SyntaxKind

Sources: [TypeScript Deep Dive AST](https://basarat.gitbook.io/typescript/overview/ast), [TypeScript Compiler API wiki](https://github.com/microsoft/TypeScript/wiki/Using-the-Compiler-API), [typescript-eslint AST blog](https://typescript-eslint.io/blog/asts-and-typescript-eslint/), [ts-ast-viewer.com](https://ts-ast-viewer.com/)

### Architecture

TypeScript's compiler has five key phases, all operating on or producing AST structures:

1. **Scanner** (`scanner.ts`): Source text to tokens
2. **Parser** (`parser.ts`): Tokens to AST (produces `SourceFile`)
3. **Binder** (`binder.ts`): AST to symbols (creates scope/symbol table, populates `node.symbol`)
4. **Type Checker** (`checker.ts`): Type inference and checking using symbols + AST
5. **Emitter** (`emitter.ts`): AST to JavaScript output

### Node Interface

Every AST node implements the `Node` interface:

```typescript
interface Node extends TextRange {
    kind: SyntaxKind;
    parent?: Node;
    flags: NodeFlags;
    // ... modifiers, decorators
}

interface TextRange {
    pos: number;   // start position in source
    end: number;   // end position in source
}
```

### SyntaxKind Enum

`SyntaxKind` is a `const enum` with 350+ members. It is compiled with `--preserveConstEnums` so it remains available at runtime. Categories include:

- **Tokens**: `EndOfFileToken`, `SemicolonToken`, `PlusToken`, ...
- **Trivia**: `SingleLineCommentTrivia`, `WhitespaceTrivia`, `NewLineTrivia`
- **Literals**: `NumericLiteral`, `StringLiteral`, `TemplateLiteral`, `RegularExpressionLiteral`
- **Names**: `Identifier`, `QualifiedName`, `ComputedPropertyName`
- **Declarations**: `VariableDeclaration`, `FunctionDeclaration`, `ClassDeclaration`, `InterfaceDeclaration`, `TypeAliasDeclaration`, `EnumDeclaration`
- **Statements**: `Block`, `IfStatement`, `WhileStatement`, `ForStatement`, `ReturnStatement`, `SwitchStatement`
- **Expressions**: `BinaryExpression`, `CallExpression`, `PropertyAccessExpression`, `ElementAccessExpression`, `ArrowFunction`, `ConditionalExpression`
- **Type nodes**: `TypeReference`, `UnionType`, `IntersectionType`, `MappedType`, `ConditionalType`
- **JSDoc**: `JSDocComment`, `JSDocTag`, `JSDocParameterTag`

### TypeScript AST vs ESTree

| Aspect | TypeScript AST | ESTree |
|--------|---------------|--------|
| Design goal | Parsing incomplete code + typechecking | General-purpose traversal |
| TypeScript syntax | Full support (interfaces, generics, etc.) | No knowledge of TS-specific syntax |
| Optimization | Optimized for incremental parsing | Unoptimized, general purpose |
| Trivia | Attached to nodes | Separate or absent |
| Tooling | TypeScript compiler API | ESLint, Babel, Acorn |

The `typescript-eslint` project bridges this gap by parsing TypeScript into the TS AST, then converting to `TSESTree` -- an ESTree extension with TypeScript-specific node types. It maintains tracking between equivalent nodes in both formats so ESLint rules can access type information through the TypeScript type checker.

### Traversal

TypeScript provides `ts.forEachChild(node, callback)` which visits each child of a node. Unlike a recursive `visitNode`, `forEachChild` stops early if the callback returns a truthy value (useful for search). For full traversal, you recursively call `forEachChild` within your callback.

---

## Go: go/ast Package

Sources: [go/ast package docs](https://pkg.go.dev/go/ast), [Eli Bendersky on Go AST rewriting](https://eli.thegreenplace.net/2021/rewriting-go-source-code-with-ast-tooling/), [zupzup Go AST traversal](https://www.zupzup.org/go-ast-traversal/index.html)

### Design Philosophy

Go's AST reflects the language's simplicity. The `go/ast` package is in the standard library and provides a clean, minimal representation.

### Core Interfaces

```go
type Node interface {
    Pos() token.Pos  // position of first character
    End() token.Pos  // position of first character after the node
}

type Expr interface { Node; exprNode() }   // all expressions
type Stmt interface { Node; stmtNode() }   // all statements
type Decl interface { Node; declNode() }   // all declarations
```

Every node has position information via `Pos()` and `End()`. The marker methods (`exprNode()`, `stmtNode()`, `declNode()`) are unexported, making the interfaces closed -- only types in the `go/ast` package can implement them.

### Concrete Node Types

**Expressions** (`Expr`): `Ident`, `BasicLit`, `BinaryExpr`, `UnaryExpr`, `CallExpr`, `SelectorExpr` (field/method access), `IndexExpr`, `SliceExpr`, `TypeAssertExpr`, `StarExpr`, `ParenExpr`, `FuncLit`, `CompositeLit`, `KeyValueExpr`, plus type expressions (`ChanType`, `MapType`, `ArrayType`, `FuncType`, `InterfaceType`, `StructType`).

**Statements** (`Stmt`): `AssignStmt`, `ExprStmt`, `BlockStmt`, `IfStmt`, `ForStmt`, `RangeStmt`, `SwitchStmt`, `TypeSwitchStmt`, `SelectStmt`, `ReturnStmt`, `BranchStmt` (break/continue/goto/fallthrough), `DeferStmt`, `GoStmt`, `SendStmt`, `IncDecStmt`, `LabeledStmt`.

**Declarations** (`Decl`): `FuncDecl` (function), `GenDecl` (generic declaration for import, const, type, var). `GenDecl` contains `Spec` items: `ImportSpec`, `ValueSpec` (const/var), `TypeSpec`.

**File**: `ast.File` is the top-level node with `Package`, `Name`, `Decls`, `Imports`, `Comments`.

**Error nodes**: `BadExpr`, `BadStmt`, `BadDecl` serve as placeholders when parsing encounters syntax errors.

### Comment Handling

Comments are stored separately from the AST structure in `[]*CommentGroup` on the `File` node. The `CommentMap` type (created via `NewCommentMap`) associates comments with their adjacent AST nodes. When modifying the tree, you must explicitly update comment associations via `cmap.Filter(file)`.

### Traversal API

```go
// Visitor interface -- full control
func Walk(v Visitor, node Node)
type Visitor interface { Visit(node Node) (w Visitor) }

// Inspect -- simpler closure-based API (recommended)
func Inspect(node Node, f func(Node) bool)

// Preorder -- Go 1.23+ iterator-based (preferred)
func Preorder(root Node) iter.Seq[Node]
```

`Walk` calls `v.Visit(node)`; if the returned Visitor is non-nil, it recursively walks children. `Inspect` wraps this in a closure: return `true` to continue descending, `false` to skip children. Neither `Walk` nor `Inspect` gives you access to the parent or the ability to replace the current node.

For tree rewriting, `golang.org/x/tools/go/ast/astutil` provides `Apply(root, pre, post)` with a `Cursor` that allows replacing, deleting, or inserting nodes.

---

## Lua: No-AST Compiler and LuaJIT SSA IR

Sources: [Ravi docs on Lua parser internals](https://the-ravi-programming-language.readthedocs.io/en/latest/lua-parser.html), [Implementation of Lua 5.0 (PDF)](https://www.lua.org/doc/jucs05.pdf), [LuaJIT SSA IR wiki](https://github.com/tarantool/tarantool/wiki/LuaJIT-SSA-IR)

### PUC-Rio Lua: No AST at All

Uniquely among major languages, the reference Lua implementation (PUC-Rio Lua 5.x) does not build an AST. The compiler is a single-pass recursive-descent parser that emits bytecode directly as it parses. There are no heap-allocated intermediate structures.

The key mechanism is the `expdesc` structure, which represents expressions in deferred states:

| State | Meaning |
|-------|---------|
| `VLOCAL` | References a local variable in a known register |
| `VRELOCABLE` | Instruction emitted but target register not yet assigned |
| `VNONRELOC` | Result register determined and set |
| `VCALL` | Function call (may transition to tail call) |
| `VINDEXED` | Table access operation |

When parsing a binary expression like `a + b`, the parser first produces `expdesc` values for `a` and `b`. If both are constants or locals, the `ADD` instruction can reference them directly without intermediate moves. The parser patches instruction operands retroactively as more context becomes available.

Data structures live on the C stack -- there are no heap allocations during compilation. Where linking is needed, the call stack itself forms a linked structure. This makes the Lua compiler extremely memory-efficient.

The register-based VM uses a sliding register window on a unified stack. `Callinfo` frames track activation records; registers for parameters, locals, and temporaries are addressed relative to a base pointer.

### LuaJIT: Bytecode + SSA IR

LuaJIT uses two internal representations:

1. **Stack-based bytecode** for the interpreter (different format from PUC-Rio Lua's register-based bytecode)
2. **SSA IR** for the JIT compiler's trace compiler

The SSA IR is remarkable for its compactness and cache efficiency:

**64-bit instruction format:**

```
| op1 (16 bits) | op2 (16 bits) | type (8 bits) | opcode (8 bits) | reg (8 bits) | spill (8 bits) |
```

Every instruction occupies exactly 8 bytes. The IR is stored in a linear, pointer-free array where each instruction is implicitly numbered by its position (IRRef).

**Bidirectional array growth:**

```
Constants grow downward ←  REF_BIAS  → Non-constants grow upward
     ...  K003  K002  K001  |  0001  0002  0003  ...
```

Constants and non-constants grow in opposite directions from a bias point. An IRRef greater than `REF_BIAS` is a non-constant; less than `REF_BIAS` is a constant. This enables O(1) const-vs-non-const discrimination.

**Instruction categories:**

- **Guards**: `LT`, `GE`, `LE`, `GT`, `EQ`, `NE`, `ABC` -- emit comparison branches; false outcomes trigger trace exits
- **Arithmetic**: `ADD`, `SUB`, `MUL`, `DIV`, `MOD`, `POW`, with overflow-checking variants (`ADDOV`, `SUBOV`, `MULOV`)
- **Bitwise**: `BNOT`, `BAND`, `BOR`, `BXOR`, `BSHL`, `BSHR`, `BSAR`, `BROL`, `BROR`
- **Memory**: `ALOAD`/`ASTORE` (array), `HLOAD`/`HSTORE` (hash), `FLOAD`/`FSTORE` (field), `XLOAD` (extended)
- **Control**: PHI nodes at loop ends (left operand = initial value, right = loop-iteration value), `RETF` for frame returns

**Type system**: 24 IR types including `nil`, `false`, `true`, `lightud`, `str`, `p32`/`p64` (pointers), `thread`, `proto`, `func`, `cdata`, `tab`, `udata`, `flt` (float), `num` (double), `i8`/`u8`/`i16`/`u16`/`int`/`u32`/`i64`/`u64`.

**Snapshots** capture modified stack slots and their IR references, enabling reconstruction of bytecode state when a trace exit occurs.

---

## Ruby: Prism Parser

Sources: [Prism GitHub](https://github.com/ruby/prism), [Prism in 2024](https://railsatscale.com/2024-04-16-prism-in-2024/), [Kevin Newton on Prism](https://kddnewton.com/2024/01/23/prism.html), [Evil Martians migration guide](https://evilmartians.com/chronicles/unparser-real-file-lessons-migrating-ruby-tools-from-parser-to-prism)

### Background

Ruby historically relied on several parsers:
- **Ripper**: CRuby's built-in parser (difficult API, produces S-expression-like output, only supports the running Ruby version's syntax)
- **whitequark/parser**: Third-party gem widely used by RuboCop
- **seattlerb/ruby_parser**: Another third-party parser

Shopify maintained four separate Ruby parsers (CRuby, TruffleRuby, Sorbet, and internal tooling). Prism was created to unify them all.

### Architecture

Prism is written in C99 with zero dependencies. It provides:

- **libprism**: A standalone C shared library, independent of CRuby
- A CRuby native extension
- Bindings for Rust (`ruby-prism`, `ruby-prism-sys`), Java, JavaScript/WASM

Starting in Ruby 3.3, Prism ships as a default gem. In Ruby 3.4, Prism became the parser used by CRuby itself.

### Node Design

Node specifications are defined centrally in `config.yml`, which drives code generation via ERB templates. This is conceptually similar to Python's ASDL approach -- one configuration file generates node types across all language bindings.

Prism splits nodes more granularly than competing parsers. For instance, instance variable writes have separate node types:
- `InstanceVariableWriteNode` for direct assignment (`@x = 1`)
- `InstanceVariableTargetNode` for indirect writes (e.g., in `for` loop targets)

The design principle: "you never have to consult a child node to determine how to compile the parent node." This keeps the compiler flat and maintainable.

All nodes share common interfaces:
- Named field accessors for children
- `#compact_child_nodes` (excluding nil)
- `#copy` for immutable-style node cloning
- Pattern matching via `#deconstruct` / `#deconstruct_keys`
- `#location` for source positions
- `#accept(visitor)` for double-dispatch traversal

### Error Tolerance

Prism always returns a parse result (never raises an exception on syntax errors). It parses a superset of valid Ruby, accepting expressions in positions where they are normally invalid. This keeps the syntax tree maximally informative for tools operating on incomplete code (editors, linters, type checkers).

The parse result contains:
- The syntax tree
- Lists of errors and warnings
- Comments and metadata

### Translation Layer

Prism's native AST format is not compatible with the `whitequark/parser` gem's format. To ease migration, Prism provides `Prism::Translation::Parser`, which translates Prism's AST into the structure expected by parser-based tools (like RuboCop).

---

## CST vs AST

Sources: [Eli Bendersky](https://eli.thegreenplace.net/2009/02/16/abstract-vs-concrete-syntax-trees), [Lossless Syntax Trees](https://dev.to/cad97/lossless-syntax-trees-280c), [Red-Green Trees Overview](https://willspeak.me/2021/11/24/red-green-syntax-trees-an-overview.html), [Eric Lippert on Roslyn's Red-Green Trees](https://ericlippert.com/2012/06/08/red-green-trees/), [cstree crate](https://docs.rs/cstree), [tree-sitter](https://github.com/tree-sitter/tree-sitter)

### Definitions

A **Concrete Syntax Tree** (CST / parse tree) preserves every token from the source: whitespace, comments, parentheses, semicolons, delimiters. It is a direct representation of the grammar's derivation.

An **Abstract Syntax Tree** (AST) discards syntactically redundant information (grouping parens, semicolons, whitespace) and represents semantic structure. It is easier to analyze but cannot reproduce the original source text.

### When to Use Which

| Use Case | CST | AST |
|----------|-----|-----|
| Code formatters | Required (must preserve/control whitespace) | Insufficient |
| Refactoring tools | Required (must preserve comments, formatting) | Insufficient |
| IDE features (syntax highlighting, error recovery) | Strongly preferred | Can work but lossy |
| Compilers (type checking, codegen) | Overkill | Preferred |
| Linters | Either works | Preferred (simpler) |
| Code generation from scratch | N/A | Preferred |

### Roslyn's Red-Green Trees (C#)

The Roslyn C# compiler pioneered the red-green tree architecture to support IDE scenarios requiring both immutability (for thread safety and persistence) and efficient navigation (parent pointers, absolute positions).

**Green tree** (the CST):
- Immutable and persistent
- No parent references
- Built bottom-up
- Nodes track only their width (not absolute position)
- Reference-counted / shared: identical subtrees point to the same node
- ~95% of keyword occurrences share the same green node instance

**Red tree** (the facade):
- Immutable facade built on-demand around the green tree
- Constructed top-down as you descend
- Manufactured parent references on demand
- Computes absolute positions from accumulated widths
- Thrown away and rebuilt on every edit (cheap because it's lazy)

**Incremental reuse**: When an edit occurs, only green nodes intersecting the edited span are rebuilt -- O(log n) of total nodes. The rest of the green tree is reused. This is critical for IDE responsiveness.

The name "red-green trees" comes from the whiteboard marker colors used in the original design meeting. There is no deeper meaning.

### Rowan (rust-analyzer)

Rowan adapts Roslyn's red-green architecture for Rust with one key addition: **dynamically typed nodes**.

**GreenNode**: Immutable, stores `SyntaxKind(u16)` and width. Children are either `GreenNode` or `GreenToken`. Structurally shared (Arc-based).

**SyntaxNode** (red layer): Wraps a `GreenNode` with parent pointer, absolute offset, and lazy child materialization. Only the portion of the red tree you actually traverse gets allocated.

**Typed AST layer**: On top of the untyped red tree, rowan allows defining typed wrapper types (e.g., `FnDef`, `IfExpr`) that provide semantic accessors. All accessors return `Option<T>` because the underlying CST may not match expected structure (error tolerance).

```
GreenNode (immutable, shared, no positions)
    └── SyntaxNode (lazy, parent pointers, absolute offsets)
            └── Typed AST wrappers (FnDef, IfExpr, etc. -- all return Option)
```

### tree-sitter

tree-sitter produces a concrete syntax tree and is designed for incremental parsing in editors.

Key properties:
- **Incremental**: When source changes, tree-sitter reuses unchanged portions and only re-parses the affected region. New tree creation is fast and memory-efficient.
- **Error-tolerant**: Determines start and end of every error, returning a usable tree even for malformed input.
- **Cursor-based traversal**: The `TreeCursor` API enables depth-first traversal without allocating new objects. The cursor tracks the current node and allows moving to parent, children, or siblings.
- **Language-agnostic**: Grammars are defined in JavaScript DSL, compiled to C parsers. Over 100 language grammars exist.

tree-sitter's CST includes named nodes (semantically meaningful) and anonymous nodes (punctuation, operators). Named nodes are accessible by field name on their parent.

---

## AST Design Patterns

Sources: [Flattening ASTs (Adrian Sampson)](https://www.cs.cornell.edu/~asampson/blog/flattening.html), [Super-flat ASTs](https://jhwlr.io/super-flat-ast/), [AST Typing Problem (ezyang)](https://blog.ezyang.com/2013/05/the-ast-typing-problem/), [Zig AST PR #7920](https://github.com/ziglang/zig/pull/7920), [Crafting Interpreters](https://craftinginterpreters.com/representing-code.html), [Arena allocation in compilers](https://medium.com/@inferara/arena-based-allocation-in-compilers-b96cce4dc9ac)

### Typed vs Untyped Nodes

Two fundamental approaches:

**Typed (enum-per-node)**: Each node type is a distinct type. In Rust: `enum Expr { BinOp { left: ExprId, op: Op, right: ExprId }, Lit { value: i64 }, ... }`. Provides compile-time exhaustiveness checking. Used by rustc, syn, most Rust-based parsers.

**Untyped (tag + children)**: All nodes have the same structural type with a kind tag. Rowan's `SyntaxNode` has `kind: SyntaxKind(u16)` plus a dynamic list of children. More flexible for error-tolerant parsing and IDE scenarios. Used by Roslyn, rowan, tree-sitter.

**The AST Typing Problem** (ezyang): When an AST needs to exist in multiple "phases" (untyped, then typed after type checking), you face a choice: duplicate the AST definition with different type annotations, use a type parameter to defer the choice, or use a single untyped representation with runtime checks. Each has tradeoffs in complexity, safety, and ergonomics.

### Box-Based Trees (Traditional)

```rust
enum Expr {
    BinOp { left: Box<Expr>, op: Op, right: Box<Expr> },
    Lit(i64),
}
```

Each child is heap-allocated via `Box`. Simple to write. Poor cache locality: nodes scatter across the heap. Deallocation requires recursive traversal. Lifetime management is straightforward (owned children).

### Arena Allocation

```rust
struct Arena { nodes: Vec<Expr> }
type ExprId = u32;  // index into arena

enum Expr {
    BinOp { left: ExprId, op: Op, right: ExprId },
    Lit(i64),
}
```

All nodes live in a single contiguous `Vec`. Children are referenced by 32-bit index instead of 64-bit pointer (50% memory savings on references). Allocation is a bump-pointer increment. Deallocation is a single `Vec::clear()`. Cache locality is excellent: sequential nodes share cache lines.

**Performance**: Adrian Sampson's benchmark showed 2.4x speedup for arena vs box-based AST, with 38% of the box-based version's time spent on deallocation alone.

**Ergonomic benefit in Rust**: One lifetime per arena instead of per-node, avoiding complex reference lifetime annotations.

### Struct-of-Arrays (Zig's Approach)

Zig's compiler takes arena allocation further with struct-of-arrays layout using `MultiArrayList`:

```
Traditional (AoS):  [tag|tok|lhs|rhs|pad] [tag|tok|lhs|rhs|pad] ...
Zig SoA:            tags: [tag][tag][tag]...
                    tokens: [tok][tok][tok]...
                    lhs: [lhs][lhs][lhs]...
                    rhs: [rhs][rhs][rhs]...
```

Each AST node is 13 bytes: 1 byte tag + 4 byte main token + 4 byte left child + 4 byte right child. No padding waste. Nodes needing more than 2 children use an "extra data" side array.

**Zig benchmark results** (PR #7920):
- Parsing: 22% faster wall-clock, 28% fewer instructions, 15% fewer cache misses
- `zig fmt` on stdlib: 11.4% faster, 19.3% less peak memory (84.7MB → 68.4MB)

### Super-Flat ASTs

An extreme version packs every node into exactly 8 bytes:

```
| tag (u8) | length/inline data (u24) | child index (u32) |
```

Three node categories:
- **Leaf nodes**: 7-byte inline value (e.g., interned string ID)
- **Fixed-arity** (e.g., binary ops): 3-byte inline operator + child index
- **Dynamic-arity** (e.g., blocks): child count + first-child index (siblings are contiguous)

Benchmarks show ~3x memory reduction and throughput improvements scaling linearly with input size, because reduced memory footprint means fewer page faults.

### Interned Strings

String interning stores each unique string value once in a global table, replacing occurrences with small integer IDs. Benefits:
- O(1) string equality comparison (compare IDs, not bytes)
- Significant memory reduction (20%+ in typical compilers)
- Cache-friendly: small IDs fit in node structs without pointers

rustc's `Symbol` type, Python's interned identifiers, and most production compilers use this technique.

### Span Tracking

Every node needs source location for error messages. Common approaches:

- **Inline spans**: Each node stores `(start, end)` byte offsets. Simple, slightly wasteful if many nodes share locations.
- **Separate span array**: Parallel array indexed by node ID. Better SoA cache behavior. Used by Zig.
- **Width-only (green trees)**: Nodes store only their width; absolute positions are computed by summing ancestor widths. Used by Roslyn/rowan. Enables structural sharing (position-independent nodes).
- **Span interning**: Deduplicate identical spans. Useful when many AST nodes map to the same source range (e.g., desugared code).

### Immutable vs Mutable Trees

**Immutable trees**: Every transformation produces a new tree. Enables structural sharing, safe concurrent access, and easy undo/redo. Used by Roslyn (green tree), functional language compilers.

**Mutable trees**: In-place modification. More memory-efficient for single-threaded pipelines. Used by most traditional compilers (GCC, CPython).

**Persistent/copy-on-write**: Modify a tree by copying only the path from root to changed node, sharing all other subtrees. O(log n) nodes copied per edit. Used by Roslyn's incremental reparse, Clojure's data structures.

### Visitor-Friendly Design

For enums (Rust/ML): Pattern matching provides exhaustive traversal. No visitor boilerplate needed -- `match expr { BinOp { .. } => ..., Lit(v) => ... }` handles all cases. The compiler enforces completeness.

For class hierarchies (Java/C#/Python): The visitor pattern with `accept`/`visit` double dispatch is the standard approach. Each node type must implement `accept`, which calls the appropriate `visit_*` method on the visitor.

For untyped nodes (rowan/tree-sitter): Kind-based dispatch with `match node.kind() { ... }`. Flexible but not compile-time exhaustive.
