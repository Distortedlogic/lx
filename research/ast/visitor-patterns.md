# Visitor Pattern and Tree Traversal

Research on visitor pattern variants, tree traversal strategies, tree rewriting patterns, and performance considerations across programming languages and compiler frameworks.

## Table of Contents

1. [The Classic Visitor Pattern](#the-classic-visitor-pattern)
2. [Python: ast.NodeVisitor and ast.NodeTransformer](#python-astnodevisitor-and-astnodetransformer)
3. [Rust: Visitor Patterns](#rust-visitor-patterns)
4. [ESLint: Selector-Based Traversal](#eslint-selector-based-traversal)
5. [ANTLR: Listener vs Visitor](#antlr-listener-vs-visitor)
6. [Alternatives to the Visitor Pattern](#alternatives-to-the-visitor-pattern)
7. [Tree Rewriting Patterns](#tree-rewriting-patterns)
8. [Performance: Allocation and Layout](#performance-allocation-and-layout)

---

## The Classic Visitor Pattern

Sources: [Wikipedia](https://en.wikipedia.org/wiki/Visitor_pattern), [Refactoring Guru](https://refactoring.guru/design-patterns/visitor-double-dispatch), [Gang of Four](https://springframework.guru/gang-of-four-design-patterns/visitor-pattern/)

### Gang of Four Formulation

The visitor pattern, introduced in *Design Patterns* (1994), separates algorithms from the object structures they operate on. The motivating example in the book is a compiler: an AST has many node types, and many operations need to traverse it (printing, type checking, code generation). Without visitors, each operation would require adding a method to every node class. With visitors, new operations are new visitor classes -- no node classes need to change.

### The Double Dispatch Mechanism

The pattern solves a language limitation: most OOP languages support single dispatch (method resolution based on the receiver's runtime type). The visitor pattern achieves double dispatch -- method selection based on both the node type and the operation type -- through two virtual calls:

```
1. node.accept(visitor)     -- dispatches on node's runtime type
2. visitor.visit_X(this)    -- dispatches on visitor's runtime type
```

Step 1: `accept` is a virtual method on the node. Its implementation is trivial but type-specific:

```java
class BinOpNode implements Node {
    void accept(Visitor v) { v.visitBinOp(this); }
}
class LitNode implements Node {
    void accept(Visitor v) { v.visitLit(this); }
}
```

Step 2: The visitor interface declares one `visit_*` method per node type:

```java
interface Visitor {
    void visitBinOp(BinOpNode node);
    void visitLit(LitNode node);
    void visitIf(IfNode node);
    // ... one per node type
}
```

The concrete visitor implements the operation for each node type. Traversal is typically handled by the visitor itself (calling `accept` on child nodes) or by a separate walker.

### Strengths and Weaknesses

**Strengths**:
- Adding new operations is easy (new visitor class, no node changes)
- Related behavior is grouped in one class (the visitor), not scattered across node classes
- Visitors can accumulate state during traversal

**Weaknesses**:
- Adding new node types is hard (every visitor interface must be updated)
- Breaks encapsulation (visitors access node internals)
- Verbose boilerplate (accept methods, visitor interface with N methods)
- In languages with pattern matching (Rust, ML, Scala), the pattern is largely unnecessary

### The Expression Problem

The visitor pattern is one side of the *expression problem*: it makes adding operations easy but adding data types hard. The opposite approach (methods on nodes) makes adding types easy but operations hard. Neither OOP nor FP solves both sides cleanly; solutions like type classes, multimethods, and object algebras address it in different ways.

---

## Python: ast.NodeVisitor and ast.NodeTransformer

Sources: [ast module docs](https://docs.python.org/3/library/ast.html), [Green Tree Snakes](https://greentreesnakes.readthedocs.io/en/latest/manipulating.html), [DeepSource tutorial](https://deepsource.com/blog/python-asts-by-building-your-own-linter)

### ast.NodeVisitor

Python's `ast.NodeVisitor` implements visitor dispatch through naming conventions rather than interfaces:

```python
class NodeVisitor:
    def visit(self, node):
        method = 'visit_' + node.__class__.__name__
        visitor = getattr(self, method, self.generic_visit)
        return visitor(node)

    def generic_visit(self, node):
        for field, value in ast.iter_fields(node):
            if isinstance(value, list):
                for item in value:
                    if isinstance(item, ast.AST):
                        self.visit(item)
            elif isinstance(value, ast.AST):
                self.visit(value)
```

**Dispatch flow**:
1. `visit(node)` constructs the method name `visit_<ClassName>` (e.g., `visit_BinOp`)
2. If the method exists on `self`, call it
3. Otherwise, fall back to `generic_visit`, which recursively visits all child nodes

**Key behavior**: Children are NOT automatically visited when you define a `visit_X` method. You must explicitly call `self.generic_visit(node)` within your method to continue descending. This gives you full control over traversal order and whether to visit children at all.

```python
class FunctionCounter(ast.NodeVisitor):
    def __init__(self):
        self.count = 0

    def visit_FunctionDef(self, node):
        self.count += 1
        self.generic_visit(node)  # continue into nested functions

    def visit_AsyncFunctionDef(self, node):
        self.count += 1
        self.generic_visit(node)
```

### ast.NodeTransformer

`NodeTransformer` extends `NodeVisitor` for tree rewriting. The key difference: the return value of each `visit_*` method determines what happens to the node:

| Return Value | Effect |
|-------------|--------|
| The node itself | No change (identity) |
| A different node | Replace the original node |
| `None` | Remove the node (only valid in list contexts like statement bodies) |
| A list of nodes | Splice multiple nodes in place of one (only in list contexts) |

```python
class DoubleAllIntegers(ast.NodeTransformer):
    def visit_Constant(self, node):
        if isinstance(node.value, int):
            return ast.Constant(value=node.value * 2)
        return node  # leave non-ints unchanged
```

**Important**: `NodeTransformer.generic_visit` handles the transformation plumbing -- it iterates over fields, visits children, and replaces them based on return values. When overriding a `visit_*` method, you must either transform children yourself or call `self.generic_visit(node)` first to let children be transformed before you act on the node.

**Post-transformation fixup**: Generated nodes lack source location info. Call `ast.fix_missing_locations(tree)` to copy location from parent nodes, and `ast.copy_location(old_node, new_node)` for explicit location transfer.

### Utility Functions for Traversal

| Function | Behavior |
|----------|----------|
| `ast.walk(node)` | Yield all descendant nodes recursively (breadth-first, no guaranteed order). Does not provide parent or replacement capability. |
| `ast.iter_fields(node)` | Yield `(fieldname, value)` pairs for a node's fields |
| `ast.iter_child_nodes(node)` | Yield direct child nodes only |
| `ast.get_source_segment(source, node)` | Extract original source text for a node using its span |

---

## Rust: Visitor Patterns

Sources: [rustc_ast Visitor trait](https://doc.rust-lang.org/stable/nightly-rustc/rustc_ast/visit/trait.Visitor.html), [MIR visitor guide](https://rustc-dev-guide.rust-lang.org/mir/visitor.html), [syn::visit](https://docs.rs/syn/latest/syn/visit/index.html), [syn::visit_mut](https://docs.rs/syn/latest/syn/visit_mut/index.html), [Rust Design Patterns](https://rust-unofficial.github.io/patterns/patterns/behavioural/visitor.html)

### Why Rust's Visitors Differ from OOP

In OOP languages, the visitor pattern exists because class hierarchies lack exhaustive pattern matching. Rust has `enum`, which gives you exhaustive `match` for free. A simple tree walk with `match` is often all you need:

```rust
fn eval(expr: &Expr) -> i64 {
    match expr {
        Expr::Lit(n) => *n,
        Expr::BinOp { left, op, right } => {
            let l = eval(left);
            let r = eval(right);
            match op {
                Op::Add => l + r,
                Op::Mul => l * r,
            }
        }
    }
}
```

The compiler ensures every variant is handled. No `accept` boilerplate needed.

However, when ASTs have dozens or hundreds of node types (as in rustc or syn), writing exhaustive matches for every operation becomes impractical. The visitor trait pattern re-emerges: define a trait with one method per node type, provide default implementations that recurse, and let users override only the methods they care about.

### rustc's AST Visitor (`rustc_ast::visit::Visitor`)

```rust
pub trait Visitor<'ast>: Sized {
    fn visit_expr(&mut self, expr: &'ast Expr) {
        walk_expr(self, expr);  // default: recurse into children
    }
    fn visit_item(&mut self, item: &'ast Item) {
        walk_item(self, item);
    }
    fn visit_pat(&mut self, pat: &'ast Pat) {
        walk_pat(self, pat);
    }
    // ... ~50 more visit_* methods
}
```

**`walk_*` functions**: Each `visit_*` method's default implementation calls a corresponding `walk_*` free function that destructures the node and visits its children. Destructuring is deliberate: if a field is added to a node type, `walk_*` will fail to compile until updated, preventing silent omissions.

**Override pattern**: Override `visit_expr` to intercept expression nodes. Call `walk_expr(self, expr)` at the end (or not) to control whether children are visited.

**Mutable variant**: `rustc_ast::mut_visit::MutVisitor` provides `&mut` access to nodes for in-place transformation. Same structure, mutable references.

### rustc's MIR Visitor (`rustc_middle::mir::visit`)

MIR visitors have a different structure due to MIR's flat CFG representation:

```rust
pub trait Visitor<'tcx> {
    fn visit_body(&mut self, body: &Body<'tcx>) { ... }
    fn visit_basic_block_data(&mut self, block: BasicBlock, data: &BasicBlockData<'tcx>) { ... }
    fn visit_statement(&mut self, statement: &Statement<'tcx>, location: Location) { ... }
    fn visit_terminator(&mut self, terminator: &Terminator<'tcx>, location: Location) { ... }
    fn visit_place(&mut self, place: &Place<'tcx>, context: PlaceContext, location: Location) { ... }
    fn visit_local(&mut self, local: Local, context: PlaceContext, location: Location) { ... }
    fn visit_rvalue(&mut self, rvalue: &Rvalue<'tcx>, location: Location) { ... }
    // ...
}
```

Key differences from AST visitors:
- **`super_*` instead of `walk_*`**: The recursive-descent methods are called `super_statement`, `super_terminator`, etc. Override `visit_*` for custom logic; call `super_*` to continue recursion. Never override `super_*` itself.
- **Location tracking**: Every `visit_*` method receives a `Location` (basic block + statement index), since MIR nodes need CFG context.
- **`MutVisitor`**: Same pattern with `&mut` references.
- **Traversal orders**: `rustc_middle::mir::traversal` provides pre-order, reverse post-order, and other CFG walking strategies.

### syn's Three Visitor Traits

The `syn` crate (for proc macros) provides three independent visitor traits, each behind a feature flag:

**`Visit<'ast>`** -- read-only traversal:

```rust
pub trait Visit<'ast> {
    fn visit_expr_binary(&mut self, node: &'ast ExprBinary) {
        visit_expr_binary(self, node);  // default: recurse via free function
    }
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        visit_item_fn(self, node);
    }
    // ... 150+ methods covering all Rust syntax
}
```

The `'ast` lifetime ensures the syntax tree outlives the traversal, allowing visitors to hold references into the tree.

**`VisitMut`** -- mutable traversal:

```rust
pub trait VisitMut {
    fn visit_expr_binary_mut(&mut self, node: &mut ExprBinary) {
        visit_expr_binary_mut(self, node);
    }
    // ... same structure, &mut references
}
```

**`Fold`** -- ownership-taking transformation:

```rust
pub trait Fold {
    fn fold_expr_binary(&mut self, node: ExprBinary) -> ExprBinary {
        fold_expr_binary(self, node)
    }
    // ... takes ownership, returns transformed value
}
```

`Fold` is the most expensive: it moves values, requiring reconstruction of the entire tree even for nodes that don't change. `VisitMut` is preferred when in-place mutation suffices.

### swc's Visitor

The swc project (JavaScript/TypeScript compiler in Rust) uses a macro-generated visitor system similar to syn but with additional features:

- `Visit`: Read-only traversal
- `VisitMut`: Mutable traversal
- `Fold`: Ownership-based transformation
- `VisitAll`: Visits all nodes regardless of overrides (useful for collecting information from the entire tree)

---

## ESLint: Selector-Based Traversal

Sources: [ESLint selectors](https://eslint.org/docs/latest/extend/selectors), [ESLint custom rules](https://eslint.org/docs/latest/extend/custom-rules), [eslint-visitor-keys](https://github.com/eslint/eslint-visitor-keys)

### Architecture

ESLint rules use a declarative, CSS-inspired selector system instead of explicit visitor interfaces. A rule exports an object mapping selectors to handler functions:

```javascript
module.exports = {
    create(context) {
        return {
            "BinaryExpression": function(node) {
                // called when entering any BinaryExpression
            },
            "FunctionDeclaration:exit": function(node) {
                // called when LEAVING any FunctionDeclaration
            },
            "IfStatement > BlockStatement": function(node) {
                // called for BlockStatements that are direct children of IfStatements
            }
        };
    }
};
```

### Selector Syntax

Modeled after CSS selectors, operating on AST node types instead of HTML elements:

**Basic selectors:**
- `ForStatement` -- matches nodes of that type
- `*` -- wildcard, matches any node

**Attribute selectors:**
- `[attr]` -- node has attribute
- `[attr="foo"]` -- exact value match
- `[attr=/^foo.*/]` -- regex match
- `[attr!=value]`, `[attr>2]`, `[attr<3]`, `[attr>=2]`, `[attr<=3]` -- comparisons
- `[attr.nested.path="value"]` -- nested attribute access

**Relationship selectors:**
- `A > B` -- B is a direct child of A
- `A B` -- B is a descendant of A (any depth)
- `A ~ B` -- B is a subsequent sibling of A
- `A + B` -- B is the immediately following sibling of A

**Pseudo-class selectors:**
- `:first-child`, `:last-child`, `:nth-child(n)`, `:nth-last-child(n)`
- `:not(selector)` -- negation
- `:matches(sel1, sel2)` / `:is(sel1, sel2)` -- matches any
- `:statement`, `:expression`, `:declaration`, `:function`, `:pattern` -- AST class selectors

**Field selection:**
- `FunctionDeclaration > Identifier.id` -- only match `Identifier` nodes that are the `id` field of their parent

### Enter and Exit Phases

Every node is visited twice during AST traversal:

1. **Enter** (default): `"FunctionDeclaration"` fires when descending into the node
2. **Exit**: `"FunctionDeclaration:exit"` fires when ascending out of the node

This enables pre-order and post-order processing in the same traversal pass.

### Specificity Rules

When multiple selectors match the same node, handlers fire in order of increasing specificity (paralleling CSS):
1. More class, attribute, and pseudo-class components rank higher
2. Equal specificity: more node type selectors rank higher
3. Still tied: alphabetical order

### Practical Examples

```javascript
// Ban require() calls
"CallExpression[callee.name='require']"

// Disallow functions with more than 3 parameters
"FunctionDeclaration[params.length>3]"

// Disallow non-block if consequents
"IfStatement > :not(BlockStatement).consequent"

// Match identifiers starting with underscore
"Identifier[name=/^_/]"
```

---

## ANTLR: Listener vs Visitor

Sources: [ANTLR listener docs](https://github.com/antlr/antlr4/blob/master/doc/listeners.md), [Jakub Dziworski comparison](https://jakubdziworski.github.io/java/2016/04/01/antlr_visitor_vs_listener.html), [DeepWiki comparison](https://deepwiki.com/jszheng/py3antlr4book/3.3-implementation-patterns:-visitor-vs-listener)

ANTLR generates two distinct traversal mechanisms from a grammar. Understanding when to use each is broadly applicable beyond ANTLR.

### Listener Pattern

ANTLR generates a listener interface with `enter*` and `exit*` methods for each grammar rule:

```java
public interface ExprListener extends ParseTreeListener {
    void enterAddExpr(ExprParser.AddExprContext ctx);
    void exitAddExpr(ExprParser.AddExprContext ctx);
    void enterMulExpr(ExprParser.MulExprContext ctx);
    void exitMulExpr(ExprParser.MulExprContext ctx);
    void enterIntExpr(ExprParser.IntExprContext ctx);
    void exitIntExpr(ExprParser.IntExprContext ctx);
}
```

**Traversal is automatic**: ANTLR's `ParseTreeWalker` walks the tree and calls `enter*`/`exit*` methods. You never recursively visit children yourself.

**No return values**: Listener methods return `void`. To pass data between nodes, you must use external state -- typically a `Stack<T>` or `Map<ParseTree, T>`.

**Heap-based traversal**: The walker uses an explicit stack on the heap, not the call stack. This means arbitrarily deep trees won't cause stack overflow.

**Can run during parsing**: Listeners can be attached to the parser itself via `addParseListener()`, receiving events as the parse tree is being constructed.

### Visitor Pattern

ANTLR also generates a visitor interface with a `visit*` method per rule:

```java
public interface ExprVisitor<T> extends ParseTreeVisitor<T> {
    T visitAddExpr(ExprParser.AddExprContext ctx);
    T visitMulExpr(ExprParser.MulExprContext ctx);
    T visitIntExpr(ExprParser.IntExprContext ctx);
}
```

**Manual traversal**: You must explicitly call `visit(child)` to descend into children. This gives you full control over traversal order, short-circuiting, and which subtrees to visit.

**Return values**: Visitor methods return a typed value `T`, enabling natural expression evaluation:

```java
@Override
public Integer visitAddExpr(ExprParser.AddExprContext ctx) {
    int left = visit(ctx.expr(0));
    int right = visit(ctx.expr(1));
    return left + right;
}

@Override
public Integer visitIntExpr(ExprParser.IntExprContext ctx) {
    return Integer.parseInt(ctx.INT().getText());
}
```

**Call-stack-based**: Uses recursive calls, meaning very deep trees can cause stack overflow.

### When to Use Which

| Criterion | Listener | Visitor |
|-----------|----------|---------|
| Traversal control | Automatic (walker) | Manual (explicit `visit()` calls) |
| Return values | None (use external state) | Typed return values |
| Computation model | Side-effect based (mutate state) | Functional (compose return values) |
| Stack safety | Heap-allocated walker stack | Call stack (overflow risk on deep trees) |
| Typical use | Symbol tables, validation, linting, metrics | Expression evaluation, tree transformation, code generation |
| Complexity | Simpler (no recursion management) | More flexible (full control) |
| During parsing | Yes (`addParseListener`) | No (requires complete tree) |

**Rule of thumb**: Use listeners for passive analysis (collecting information, validation). Use visitors for active computation (evaluation, transformation) where you need return values or controlled traversal.

---

## Alternatives to the Visitor Pattern

Sources: [Visitor Considered Pointless](https://nipafx.dev/java-visitor-pattern-pointless/), [Rust Design Patterns](https://rust-unofficial.github.io/patterns/patterns/behavioural/visitor.html), [Fold vs Visitor (Rust forum)](https://users.rust-lang.org/t/fold-pattern-compared-with-visitor-pattern/77480), [Scala and the Visitor Pattern](https://meta.plasm.us/posts/2019/09/23/scala-and-the-visitor-pattern/), [Huet Zipper](https://pavpanchekha.com/blog/zippers/huet.html), [Wikipedia: Zipper](https://en.wikipedia.org/wiki/Zipper_(data_structure))

### Pattern Matching on Enum Variants (Rust/ML Style)

In languages with algebraic data types and pattern matching, the visitor pattern is largely unnecessary:

```rust
fn transform(expr: &Expr) -> Expr {
    match expr {
        Expr::BinOp { left, op, right } => {
            let l = transform(left);
            let r = transform(right);
            // constant folding
            if let (Expr::Lit(a), Op::Add, Expr::Lit(b)) = (&l, op, &r) {
                Expr::Lit(a + b)
            } else {
                Expr::BinOp { left: Box::new(l), op: *op, right: Box::new(r) }
            }
        }
        Expr::Lit(n) => Expr::Lit(*n),
    }
}
```

The compiler enforces exhaustiveness -- adding a new variant to `Expr` produces compile errors everywhere a `match` doesn't handle it. This gives the same safety as the visitor pattern's interface without any of the boilerplate.

**When visitors are still useful in Rust**: When the AST has 100+ node types (like syn's 200+ expression variants) and most operations only care about a few. The visitor trait provides sensible defaults (recurse into children) so you override only what matters.

### Fold / Catamorphism

A fold (or catamorphism) is the functional dual of a tree constructor. Where a constructor builds up a tree bottom-to-top, a fold tears it down bottom-to-top, replacing each constructor with a function:

```haskell
data Expr = Lit Int | Add Expr Expr | Mul Expr Expr

foldExpr :: (Int -> a) -> (a -> a -> a) -> (a -> a -> a) -> Expr -> a
foldExpr lit add mul expr = case expr of
    Lit n     -> lit n
    Add l r   -> add (go l) (go r)
    Mul l r   -> mul (go l) (go r)
  where go = foldExpr lit add mul

-- Evaluate:
eval = foldExpr id (+) (*)

-- Pretty print:
pretty = foldExpr show (\l r -> "(" ++ l ++ " + " ++ r ++ ")")
                       (\l r -> "(" ++ l ++ " * " ++ r ++ ")")
```

**Catamorphism vs visitor**: A catamorphism processes children before the parent (bottom-up). A visitor can process in any order. Catamorphisms compose naturally through function composition; visitors accumulate state. Catamorphisms produce a result value; visitors can have side effects.

In Rust, the `Fold` trait in syn embodies this: it takes ownership of each node, recursively folds children, and returns the transformed result. It is more expensive than `VisitMut` because it reconstructs the entire tree, but it is conceptually cleaner.

### Tree-Walking with Match (Interpreter Pattern)

The simplest approach: a recursive function that matches on node types. No abstraction layer, no trait, no interface. The function IS the traversal:

```rust
fn eval(env: &mut Env, stmt: &Stmt) -> Result<Value> {
    match stmt {
        Stmt::Let { name, value } => {
            let v = eval_expr(env, value)?;
            env.bind(name, v);
            Ok(Value::Unit)
        }
        Stmt::If { cond, then, else_ } => {
            if eval_expr(env, cond)?.is_truthy() {
                eval_block(env, then)
            } else if let Some(e) = else_ {
                eval_block(env, e)
            } else {
                Ok(Value::Unit)
            }
        }
        // ...
    }
}
```

This works well for small-to-medium ASTs. It becomes unwieldy when the same AST needs many different traversals, which is exactly when the visitor pattern earns its keep.

### Cursor-Based Traversal (tree-sitter)

tree-sitter's `TreeCursor` provides imperative navigation without callbacks or pattern matching:

```c
TSTreeCursor cursor = ts_tree_cursor_new(root_node);

// Navigate
ts_tree_cursor_goto_first_child(&cursor);   // descend
ts_tree_cursor_goto_next_sibling(&cursor);  // move right
ts_tree_cursor_goto_parent(&cursor);        // ascend

// Inspect current node
TSNode node = ts_tree_cursor_current_node(&cursor);
const char *type = ts_node_type(node);
```

**Advantages**: No allocation per node visit (cursor is a single struct), explicit control flow, works naturally with loops instead of recursion, handles error nodes gracefully.

**Pattern**: Walk depth-first by repeatedly trying `goto_first_child`, falling back to `goto_next_sibling`, falling back to `goto_parent` + `goto_next_sibling`:

```c
bool reached_root = false;
while (!reached_root) {
    // process current node
    if (ts_tree_cursor_goto_first_child(&cursor)) continue;
    if (ts_tree_cursor_goto_next_sibling(&cursor)) continue;
    while (true) {
        if (!ts_tree_cursor_goto_parent(&cursor)) { reached_root = true; break; }
        if (ts_tree_cursor_goto_next_sibling(&cursor)) break;
    }
}
```

### Zipper Pattern

The zipper, introduced by Gerard Huet in 1997, represents a position within a tree as a pair: the subtree at the current focus and a "context" of everything else (the path from root to focus, with siblings).

```haskell
data Tree a = Leaf a | Node [Tree a]

data Ctx a = Top
           | Ctx { left :: [Tree a], parent :: Ctx a, right :: [Tree a] }

type Zipper a = (Tree a, Ctx a)
```

**Navigation is O(1)**:
- `goDown`: Focus on first child, saving siblings in context
- `goUp`: Reconstruct parent from context
- `goLeft`/`goRight`: Swap focus with adjacent sibling

**Modification is O(1)** at the focus point: Replace the focused subtree, then `goUp` reconstructs a new tree by path copying (O(depth) total to reach the root).

**Use cases**: Functional editors, cursor-based navigation in immutable trees, undo systems (save zipper states). Less common in compilers because most compiler passes visit every node anyway, making the random-access advantage of zippers less relevant.

### Go's astutil.Cursor (Post-Order Rewriting)

Go's `golang.org/x/tools/go/ast/astutil` provides `Apply(root, pre, post)` with a `Cursor`:

```go
astutil.Apply(file, func(c *astutil.Cursor) bool {
    // pre-order: called before children
    return true  // continue into children
}, func(c *astutil.Cursor) bool {
    // post-order: called after children
    if ident, ok := c.Node().(*ast.Ident); ok {
        c.Replace(newIdent(ident.Name + "_suffix"))  // replace current node
    }
    return true
})
```

The `Cursor` provides `Node()`, `Parent()`, `Replace(node)`, `Delete()`, and `InsertBefore(node)` / `InsertAfter(node)`. This solves the limitation of `ast.Walk` and `ast.Inspect`, which cannot replace the current node.

---

## Tree Rewriting Patterns

Sources: [Persistent data structures (Wikipedia)](https://en.wikipedia.org/wiki/Persistent_data_structure), [Structural sharing (Raganwald)](https://raganwald.com/2019/01/14/structural-sharing-and-copy-on-write.html), [Copy-on-write patterns](https://clojurepatterns.com/1/3/4/), [Eric Lippert on red-green trees](https://ericlippert.com/2012/06/08/red-green-trees/)

### In-Place Mutation

The simplest approach: walk the tree, mutate nodes directly.

```rust
fn desugar(stmt: &mut Stmt) {
    match stmt {
        Stmt::ForIn { var, iter, body } => {
            // rewrite for-in to loop + match
            *stmt = Stmt::Loop { body: desugar_for_in(var, iter, body) };
        }
        _ => {}
    }
    // recurse into children
    for child in stmt.children_mut() {
        desugar(child);
    }
}
```

**Advantages**: Zero allocation overhead, no tree reconstruction.
**Disadvantages**: Cannot keep the original tree. Concurrent access is unsafe. Debugging is harder (can't compare before/after).

Used by: rustc's `MutVisitor`, syn's `VisitMut`, most single-pass compiler transformations.

### Immutable Rewrite (Produce New Tree)

Every transformation produces a fresh tree. The original is preserved.

```rust
fn desugar(stmt: &Stmt) -> Stmt {
    match stmt {
        Stmt::ForIn { var, iter, body } => {
            Stmt::Loop { body: desugar_for_in(var, iter, body) }
        }
        Stmt::Block(stmts) => {
            Stmt::Block(stmts.iter().map(desugar).collect())
        }
        other => other.clone(),
    }
}
```

**Advantages**: Original tree preserved (useful for diagnostics, undo). Thread-safe. Easy to test (compare input and output).
**Disadvantages**: Allocates an entirely new tree even if only a few nodes change. O(n) time and space for every pass.

Used by: syn's `Fold`, functional language compilers, pure-functional transformations.

### Copy-on-Write / Path Copying

Only nodes on the path from root to the modified node are copied. All other subtrees are shared between old and new trees.

```
Original:          Modified (node D changed to D'):
     A                  A'
    / \                / \
   B   C              B   C'     (B is shared, C' is new copy)
  / \   \            / \   \
 D   E   F          D   E   D'  (E is shared, D' is new)
```

When modifying node D:
1. Create D' (the modified version)
2. Create C' pointing to D' and F (F is shared)
3. Create A' pointing to B (shared) and C'

Only O(depth) nodes are copied. In a balanced tree, this is O(log n).

**Advantages**: Old tree is preserved. Sharing saves memory. O(log n) per modification.
**Disadvantages**: More complex implementation. Pointer indirection for shared subtrees.

Used by: Roslyn's incremental reparse, Clojure's persistent data structures, Scala's immutable collections.

### Structural Sharing

A generalization of copy-on-write where multiple tree versions share as many nodes as possible. This is the foundation of persistent data structures.

Key insight: In an immutable tree, if a subtree hasn't changed, the new tree can point to the exact same node in memory. No copying needed for unchanged portions.

Roslyn's green tree exploits this aggressively: when a user types a character, the parser rebuilds only the green nodes whose text changed (typically O(log n) of the total tree). The remaining ~95% of green nodes are reused. Identical tokens (like the keyword `if`) may even share a single green node instance across the entire program.

### Rewrite Rules / Term Rewriting

Pattern-based rewriting where transformations are expressed as rules:

```
// Constant folding rules
Lit(a) + Lit(b)  →  Lit(a + b)
Lit(0) + x       →  x
x + Lit(0)       →  x
Lit(1) * x       →  x

// Desugaring rules
for x in iter { body }  →  { let mut __iter = iter.into_iter();
                              loop { match __iter.next() {
                                Some(x) => { body },
                                None => break } } }
```

Tools like Stratego/XT, GritQL, and comby implement generic term-rewriting systems. The rewriting engine handles traversal and fixpoint iteration; you supply only the rules.

**Advantage**: Declarative, easy to read and verify.
**Disadvantage**: Rule ordering and termination can be subtle. Performance depends on the engine.

---

## Performance: Allocation and Layout

Sources: [Flattening ASTs (Adrian Sampson)](https://www.cs.cornell.edu/~asampson/blog/flattening.html), [Super-flat ASTs](https://jhwlr.io/super-flat-ast/), [Zig AST PR #7920](https://github.com/ziglang/zig/pull/7920), [Arena allocation in compilers](https://medium.com/@inferara/arena-based-allocation-in-compilers-b96cce4dc9ac)

### Pointer-Based Trees (Baseline)

```
Heap layout: [BinOp] --ptr--> [Lit(1)]
                    \--ptr--> [Lit(2)]
```

Each node is a separate heap allocation. Cache performance is poor: visiting a node requires dereferencing a pointer to an unpredictable memory location, causing cache misses. Deallocation requires recursive traversal of every node. In Adrian Sampson's benchmark, 38% of time was spent on deallocation alone.

### Arena / Flat Array

```
Arena: | Lit(1) | Lit(2) | BinOp(idx:0, idx:1) |
        ^0       ^1       ^2
```

All nodes in one contiguous `Vec`. Children referenced by index. Benefits:

| Metric | Improvement |
|--------|------------|
| Reference size | 64-bit pointer → 32-bit index (50% smaller) |
| Allocation cost | Bump pointer (O(1), no malloc overhead) |
| Deallocation | One `Vec::clear()` (no recursive traversal) |
| Cache locality | Sequential nodes share cache lines |
| Overall speed | 2.4x faster (Sampson benchmark) |

**Extra-flat optimization**: If nodes are built bottom-up (children before parents), a forward linear scan can evaluate the tree without recursion -- essentially reinventing bytecode. This yields an additional ~8% speedup.

### Struct-of-Arrays (SoA)

Instead of `Vec<Node>` where each `Node` has tag + token + children, use separate arrays:

```
tags:     [u8;  N]   -- 1 byte each, contiguous
tokens:   [u32; N]   -- 4 bytes each, contiguous
children: [u64; N]   -- 8 bytes each (packed left+right), contiguous
```

**Why it's faster**: When a pass only needs node tags (e.g., finding all `if` statements), it reads a dense array of 1-byte values. No cache lines wasted loading token and child data that won't be used.

**Zig results** (from PR #7920):

| Operation | Improvement |
|-----------|------------|
| Parse: wall-clock | 22% faster |
| Parse: instructions | 28% fewer |
| Parse: cache misses | 15% fewer |
| zig fmt: peak memory | 19.3% less (84.7MB → 68.4MB) |
| zig fmt: wall-clock | 11.4% faster |
| Per-node storage | 13 bytes (no padding) |

### ECS-Style Approaches

Entity-Component-System architectures from game engines apply similar principles: entities are integer IDs, components are stored in type-homogeneous arrays, systems iterate over component arrays. For ASTs:

- Node ID = entity
- Node kind, span, children = components in separate arrays
- Compiler passes = systems iterating over relevant component arrays

This works best when different passes need different subsets of node data. It works worst when a pass needs all data about each node (like code generation), because it must gather from multiple arrays.

### Cache-Friendly Traversal Strategies

**Pre-order depth-first**: Good cache behavior when nodes are allocated in parse order (which is pre-order). Parent and first child are likely adjacent in memory.

**Post-order depth-first**: Natural for bottom-up evaluation (evaluate children, then parent). Works well with arena allocation if children are allocated before parents.

**Breadth-first**: Poor cache behavior with pointer-based trees (siblings may be far apart). Good with flat arrays if nodes at the same depth are contiguous.

**Linear scan (extra-flat)**: Best cache behavior. Only works when the arena order matches the desired evaluation order. Children must appear before parents. Essentially converts tree traversal into array iteration.

### Memory Layout Spectrum

From worst to best cache performance for typical compiler workloads:

```
Pointer-based tree (Box<Node>)
    ↓ better
Arena with indices (Vec<Node>, u32 indices)
    ↓ better
Arena + interned strings (Vec<Node> with Symbol IDs)
    ↓ better
Struct-of-arrays (separate Vec per field)
    ↓ better
Super-flat (8-byte uniform nodes in single array)
```

Each step trades ergonomic convenience for performance. The right choice depends on the project:
- **Prototyping / small languages**: Box-based trees are fine
- **Production interpreters**: Arena allocation is the sweet spot (large gains, modest complexity)
- **High-performance compilers**: SoA or super-flat for maximum throughput

### Benchmarks Summary

| Approach | Memory vs Boxed | Speed vs Boxed | Source |
|----------|----------------|----------------|--------|
| String interning | -20% | modest | Super-flat paper |
| Arena (indexed) | ~-30% | ~1.5x | Sampson |
| Arena + bump alloc | ~-40% | ~2x | Sampson |
| Flat arena (eval) | ~-40% | ~2.4x | Sampson |
| SoA (Zig) | -19% peak | 1.22x parse | Zig PR #7920 |
| Super-flat | -67% | ~3x throughput | Super-flat paper |
