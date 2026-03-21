# Type Checking Landscape: A Comprehensive Survey

A survey of type checking implementations across programming languages, type checkers,
and academic type systems. Covers inference algorithms, gradual typing, generics,
null safety, type narrowing, performance, and error message design.

---

## Table of Contents

1. [Python Type Checkers](#1-python-type-checkers)
2. [Rust](#2-rust)
3. [TypeScript](#3-typescript)
4. [Gradually-Typed Languages](#4-gradually-typed-languages)
5. [Academic Type Systems](#5-academic-type-systems)
6. [Cross-Cutting Concerns](#6-cross-cutting-concerns)

---

## 1. Python Type Checkers

Python's type system is defined through PEPs (Python Enhancement Proposals), with
PEP 484 (2014) introducing type hints and PEP 561 defining distribution of type
information. Type annotations are available at runtime via `__annotations__` but
no type checking happens at runtime -- separate off-line type checkers run over
source code voluntarily.

### 1.1 mypy

- **Repository**: https://github.com/python/mypy
- **Language**: Python (itself)
- **Architecture**: Traditional multi-pass, top-to-bottom semantic analysis

**Type Inference Algorithm**:
- Does NOT infer parameter types (except `self`/`cls`)
- By default, skips all functions without type annotations entirely
- Uses multi-pass analysis: semantic analysis runs multiple times on a module
  from top to bottom until all types converge
- Infers return types only with `--check-untyped-defs`
- Uses "join" operations to merge types (finds common supertypes), which discards
  precision and can produce false positive errors

**Gradual Typing**:
- `Any` is the escape hatch -- compatible with every type in both directions
- Unannotated functions are treated as having `Any` return type
- Stub files (.pyi) provide type information for untyped libraries
- `py.typed` marker file opts packages into type checking

**Incremental Checking**:
- mypy daemon (`dmypy`) keeps ASTs in memory across runs
- Fine-grained dependency tracking at the function level
- If only one function signature changes, only callers of that function are rechecked
- 10x+ speedup over non-daemon incremental runs for large codebases
- Cache includes fine-grained dependency information

**Generics**: Full support for parametric polymorphism via `TypeVar`, `Generic[T]`.

**Null Safety**: `Optional[X]` is sugar for `Union[X, None]`. No special handling
beyond union narrowing.

**Type Narrowing**: Supports `isinstance()` checks, `is None` checks, basic
control flow narrowing. Does NOT narrow when variables have type annotations.

**Error Messages**: Functional but often cryptic. Error codes like `[assignment]`,
`[arg-type]`. No suggestion mechanism for fixes.

### 1.2 Pyright

- **Repository**: https://github.com/microsoft/pyright
- **Language**: TypeScript (runs on Node.js)
- **Architecture**: Lazy/just-in-time type evaluator

**Type Inference Algorithm**:
- "Lazy" or "just-in-time" evaluation: evaluates type of an arbitrary identifier
  anywhere within a module by recursively evaluating dependencies
- Infers return types from function bodies (unlike mypy)
- Type checks ALL code regardless of annotations (critical for LSP features)
- Uses union operations to merge types (preserves detailed type information,
  unlike mypy's join operations)
- Retains literal types in tuple expressions and during assignments
- For ambiguous generic solutions, calculates all possible solutions and "scores"
  them by complexity, picking the simplest

**Performance**: 3-5x faster than mypy on large codebases. The lazy evaluation
architecture avoids unnecessary work -- if a type is never queried, it's never
computed. Designed for responsive editor integration via Language Server Protocol.

**Type Narrowing**:
- Assignment-based narrowing works regardless of annotation presence
- Supports additional type guard forms: literal equality, membership tests,
  `bool()` calls
- Retains literal types during narrowing (mypy widens them)

**Key Differences from mypy**:
- Follows runtime semantics for constructors: checks `__new__` return types
- Distinguishes pure class variables, regular class variables, and pure instance
  variables with strict enforcement
- First assignment does not act as implicit type declaration (unlike mypy)

### 1.3 Pytype (Google)

- **Status**: Deprecated as of 2025 (Python 3.12 is last supported version)
- **Language**: Python

**Type Inference**: Infers types from runtime behavior and code flow analysis
rather than relying on annotations. Generates stub files automatically for
gradual adoption. Uniquely infers from ALL usages of a variable (e.g., for
empty containers, examines all append/insert calls to determine element type).

**Trade-off**: Closely mirrors Python's runtime behavior but can complicate
identifying the root cause of bugs since the "source of truth" is distributed.

### 1.4 Pyre (Facebook/Meta)

- **Language**: OCaml
- **Successor**: Pyrefly (Rust-based, under active development)

**Architecture**: Two tools in one -- Pyre (type checker) and Pysa (static
analysis for security). Handles untyped code more leniently than typed code.

### 1.5 Next-Generation Python Type Checkers (2025-2026)

| Tool | Author | Language | Key Design |
|------|--------|----------|------------|
| ty | Astral/Ruff | Rust | Uses `salsa` for incremental computation, tight ruff integration |
| Pyrefly | Meta | Rust | Successor to Pyre, aggressive inference |
| Zuban | David Halter | Rust | High mypy compatibility, by author of `jedi` LSP |

Conformance testing (Aug 2025, Python Typing Council suite):
- Zuban: 69% full passes
- Pyrefly: 58% full passes
- ty: 15% full passes (alpha stage)

### 1.6 Stub Files (.pyi)

Stub files use normal Python syntax but leave out runtime logic (bodies replaced
with `...`). If both `.py` and `.pyi` exist for a module, the `.pyi` takes
precedence. `stubgen` tool auto-generates stubs via runtime introspection and
light-weight semantic analysis.

PEP 561 standardizes packaging and distribution of type information. Packages
with a `py.typed` marker file opt into type checking; bundled types (inline or
stub) are used by type checkers.

---

## 2. Rust

- **Documentation**: https://rustc-dev-guide.rust-lang.org/type-inference.html

### 2.1 Type Inference Algorithm

Rust uses an **extended Hindley-Milner** algorithm accommodating subtyping,
region inference, and higher-ranked types. The core mechanism is unification-based
constraint solving.

**Inference Variables** (managed by `InferCtxt`):
- General type variables (unify with any type)
- Integral type variables (for integer literals -- defaults to `i32`)
- Float type variables (for float literals -- defaults to `f64`)
- Region variables (lifetimes)
- Const variables (const generics)

**Constraint Gathering**: Two primary mechanisms:
- `infcx.at(..).eq(t, u)` for type equality
- `infcx.at(..).sub(..)` for subtyping
- Operations return `InferOk<()>` carrying trait obligations

**Region Constraints**: Unlike type constraints (solved eagerly via unification),
region constraints are collected as outlives relations (`'a: 'b`) and solved
after type checking completes.

**Snapshots**: The inference context supports `snapshot()` for atomic operations
with `rollback_to()` or `confirm()` for backtracking during trait resolution.

### 2.2 Trait System

Traits provide bounded parametric polymorphism (similar to Haskell typeclasses).
Associated types allow defining types tied to trait implementations, expressing
relationships between types without additional type parameters.

**Trait objects** (`dyn Trait`) provide runtime polymorphism through vtable
dispatch. **Impl blocks** provide nominal, explicit implementation (no structural
matching).

### 2.3 Borrow Checker and Lifetime Inference

The borrow checker is a type-system extension ensuring memory safety without
garbage collection.

**Non-Lexical Lifetimes (NLL)**:
- Lifetimes computed by the borrow checker itself, not lexically determined
- Fresh inference variables (RegionVid) assigned to each lifetime
- `replace_regions_in_mir()` replaces all regions with fresh variables
- Lifetime = set of program points where the reference is live

**Polonius** (next-generation borrow checker):
- Reverses the conceptual model: lifetime = set of loans (origins) a reference
  might have come from, rather than program points where it's used
- Implemented using datalog-inspired analysis
- Accepts more programs that are actually memory-safe (e.g., lending iterators)
- Still under development, not yet the default

### 2.4 Null Safety

Rust has no null. `Option<T>` is an algebraic data type (`Some(T)` | `None`).
Pattern matching enforces exhaustive handling. The `?` operator provides
ergonomic propagation.

### 2.5 Generics

Monomorphized at compile time (each generic instantiation produces specialized
code). Trait bounds constrain type parameters: `fn foo<T: Display>(x: T)`.
Const generics allow types parameterized by values: `[T; N]`.

### 2.6 Algebraic Data Types

`enum` types are proper tagged unions with pattern matching. `struct` types
are product types. Exhaustive pattern matching is enforced by the compiler.

### 2.7 Error Messages

Rust's diagnostics are among the best in any language, designed following
Elm's philosophy:
- **Primary labels**: explain "what" went wrong
- **Secondary labels**: explain "why" (in blue)
- **Help sub-diagnostics**: suggest fixes (the ONLY place fixes appear)
- **Notes**: provide context and information
- Error codes (E0xxx) with `--explain` for detailed descriptions
- Errors are kept succinct; verbose descriptions available via `--explain`

### 2.8 Performance

Type checking is the most expensive phase of compilation. Monomorphization
of generics can cause code bloat. Incremental compilation caches results at
the query level.

---

## 3. TypeScript

- **Documentation**: https://www.typescriptlang.org/docs/handbook/
- **Architecture**: https://basarat.gitbook.io/typescript/overview

### 3.1 Architecture

Five compilation stages:
1. **Tokenize**: source code to tokens
2. **Parse**: tokens to AST
3. **Bind** (`binder.ts`): AST to Symbols and Symbol Tables
4. **Check** (`checker.ts`): type checking (50,000+ lines, largest file)
5. **Emit**: AST to JavaScript

**Key Design Decision**: Symbols (from Binder) hold declarations only, NOT type
information. Types are computed lazily by the Checker only when needed. This is
critical for performance.

**Caching**: Union/intersection/tuple types cached by structural keys. Type
relation checks (assignability, subtyping) are memoized. Control flow analysis
results cached per flow node.

### 3.2 Structural Typing

Types are compared by structure, not by name. `x` is compatible with `y` if
`y` has at least the same members as `x`. This matches JavaScript's duck-typing
idiom where anonymous objects, function expressions, and object literals are
pervasive.

Exception: Classes have nominal aspects -- a class `A` is not automatically
compatible with class `B` even if they have the same shape (though in practice,
TypeScript's structural comparison means they often are).

### 3.3 Type Inference

TypeScript infers types wherever possible:
- Variable types from initializers
- Return types from return statements
- Generic type arguments from usage context

### 3.4 Control Flow Narrowing

Implemented via **flow nodes** -- a control flow graph constructed greedily
during binding, then lazily evaluated during checking.

**Key insight**: "TypeScript narrows types by traversing BACK UP the control
flow graph from the point where symbols are referenced." This is the opposite
of how developers read code (top-down).

**Flow Node Graph**: A DAG (with cycles for loops). Each node's flow node is
the previous statement that executed. Branching creates multiple antecedents.

**Narrowing Functions**:
- `narrowTypeByTruthiness`
- `narrowTypeByBinaryExpression`
- `narrowTypeByTypeof`
- `narrowTypeByInstanceof`
- `narrowTypeByEquality`

**Performance**: Lazy evaluation avoids computing flow types for variables that
are never referenced. Often the flow type isn't needed at all.

### 3.5 Advanced Type Features

**Conditional Types**: `T extends U ? X : Y` -- type-level if/then/else.
Distributes over union types automatically.

**Mapped Types**: Transform types by iterating over keys:
`{ [K in keyof T]: T[K] }`. Combined with modifiers (`readonly`, `?`).

**Template Literal Types** (TS 4.1): String manipulation at the type level.
`type Greeting = \`Hello ${Name}\``.

**Turing Completeness**: TypeScript's type system is Turing complete via
conditional types + recursive type definitions + mapped types + index types.
This enables type-level programming but also means type checking can hang.

### 3.6 Union and Intersection Types

- **Union**: `A | B` -- value is one of the types
- **Intersection**: `A & B` -- value has all properties of both types
- **Discriminated unions**: union of objects sharing a literal-typed "tag" field
  enables exhaustive pattern matching via `switch`

### 3.7 Null Safety

`--strictNullChecks` mode. `null` and `undefined` are distinct types that must
be explicitly included in unions. Narrowing via truthiness checks or explicit
null checks.

### 3.8 Generics

Parametric polymorphism with constraints: `<T extends Constraint>`. Type
parameters are erased at runtime (no reification). Inference from usage context
is sophisticated -- bidirectional inference from both argument types and
expected return types.

### 3.9 Soundness

TypeScript is **intentionally unsound** in several places:
- Covariant array typing (`Dog[]` assignable to `Animal[]`)
- Bivariant function parameter typing (configurable via `--strictFunctionTypes`)
- Type assertions (`as`) bypass checking
- `any` type disables all checking

The design philosophy prioritizes developer productivity and JavaScript
compatibility over soundness.

### 3.10 Performance and the Go Rewrite

The TypeScript team (led by Anders Hejlsberg) announced a rewrite of the type
checker in Go for 10x performance improvement:
- 3-4x from native compilation (vs. JIT-compiled JS engine)
- 3-4x from Go's concurrency (parallel parsing and type checking)
- 50% memory reduction (Go inline structs vs. JS heap-allocated objects)
- Go's GC handles cyclic references in ASTs and symbol tables

---

## 4. Gradually-Typed Languages

### 4.1 Hack (Meta, PHP derivative)

- **Website**: https://hacklang.org/
- **Type checker**: Built into HHVM

**Type System Design**:
- Evolved from PHP with full static typing as the goal
- Sound type system with both static and runtime enforcement
- Runtime enforcement of return types and parameter types (including scalars)

**Generics**:
- Type erasure by default (generic type info unavailable at runtime)
- Opt-in **reified generics** via `reify` keyword for runtime type info
- `<<__Enforceable>>` attribute marks reified type parameters that can be
  fully enforced at runtime
- Design balances performance (erasure) with runtime needs (reification)

**Shapes**: Structural typing for dictionary-like types with known keys.
Provides type safety for associative arrays without full class definitions.

**XHP**: Type-checked XML-like output syntax. Prevents XSS and double-escaping
at the type level.

**Null Safety**: `?Type` syntax for nullable types. Null checks narrow types.

### 4.2 Sorbet (Stripe, Ruby type checker)

- **Repository**: https://github.com/sorbet/sorbet
- **Language**: C++
- **Performance**: ~100,000 lines/second/core

**Architecture** (designed for speed from day one):
- **Flat array data structures**: `GlobalState` uses large flat arrays with
  32-bit indexes (`NameRef`, `SymbolRef`) instead of heap-allocated objects
- **Cache locality**: Data structures designed to maximize CPU cache hit rates
- **String interning**: All identifiers interned; comparison via integer equality
- **Container choices**: `absl::InlinedVector`, `absl::flat_hash_map`
- **Link-time optimization** and `jemalloc` for memory allocation
- **Serialized stdlib**: Built-in library definitions pre-serialized into binary

**Type Inference**:
- **Local-only inference**: Global symbols require explicit types; inference
  only within method bodies
- **Forward-only inference**: Single-pass over control-flow graphs, no
  unification variables or backward propagation
- These choices enable trivial parallelization over read-only global state

**Strictness Levels**:
- `# typed: false` -- no type checking
- `# typed: true` -- best-effort (annotations not required)
- `# typed: strict` -- all methods must have signatures
- At Stripe: 85% of non-test files use strict mode after 4 years

**Gradual Typing**: `T.untyped` as the escape hatch. Ruby's `sig` block syntax
for type annotations: `sig {params(x: Integer).returns(String)}`

**Runtime Type Checking**: Sorbet generates runtime checks from signatures,
catching type errors even in code paths the static checker can't reach.

**Error Messages**: Lazy construction -- error text generation is guarded by
conditionals to avoid work in quiet/suppressed modes.

### 4.3 Flow (Meta, JavaScript type checker)

- **Repository**: https://github.com/facebook/flow
- **Language**: OCaml

**Design Philosophy**: Assumes most JavaScript code is "implicitly statically
typed" and infers types automatically. This contrasts with TypeScript's
assumption that code is dynamically typed by default.

**Architecture**:
- Server-based: persistent background process analyzes entire codebase at startup
- Incremental: on file change, re-analyzes affected files and dependents
- Modular analysis guided by types at module boundaries
- "Aggressively parallel and incremental" type checking

**Types-First Architecture** (performance optimization):
- Requires full annotations at module boundaries (exports)
- Eliminates need for cross-module inference
- 6x speedup at 90th percentile, 2x at 99th percentile at Facebook

**Typing Approach**:
- Structural typing for objects and functions
- Nominal typing for classes
- Null/undefined treated as distinct types requiring explicit guards
- Uses data-flow and control-flow analysis for inference and narrowing

**Current Status**: Flow has diverged from standard JavaScript typing; it's now
centered on Facebook's internal needs rather than general JS ecosystem.

### 4.4 Typed Racket

- **Documentation**: https://docs.racket-lang.org/ts-guide/

**Pioneering Features**:
- First implementation of a gradual type system (Tobin-Hochstadt & Felleisen, 2008)
- Sound gradual typing with runtime contracts at typed/untyped boundaries
- Introduced **occurrence typing** (flow-sensitive typing based on predicates)

**Occurrence Typing**:
- Predicate types carry logical propositions about what's true when they
  succeed or fail
- Example: `(-> Any Boolean : String)` means "returns boolean; if true, argument
  is String"
- Works with `if`, `cond`, `when`, and other control flow constructs
- Cannot narrow mutable variables (`set!`) -- prevents concurrent mutation issues
- Implementation uses three optimizations to avoid full propositional
  satisfiability checking in most cases

**Type System Features**:
- True recursive union types (no special enum/variant syntax needed)
- Subtyping with polymorphism
- Module-level gradual typing (entire modules are typed or untyped)

**Performance Overhead**:
- Sound gradual typing inserts runtime checks at typed/untyped boundaries
- Some benchmarks show 20x+ overhead in worst cases
- Pycket runtime eliminates 90%+ of overhead via JIT optimization
- Contract verification techniques (Corpse Reviver) reduce overhead to near zero

### 4.5 Typed Clojure (core.typed)

- **Repository**: https://github.com/clojure/core.typed

**Design**: Optional type system as a library, drawing heavily from Typed Racket.
Uses occurrence typing for type narrowing in conditional expressions.

**Key Features**:
- Absence of Null Pointer Exceptions (compile-time)
- Correctness of Java interop
- Correct usage of immutable bindings
- Gradual typing with runtime contracts at typed/untyped boundaries

**Type Inference**: Automatic for most expressions; explicit annotations needed
for vars, function parameters, and some macros.

**Status**: Active development moved to `typedclojure` fork. Supports Clojure 1.11+.

### 4.6 Dart

- **Documentation**: https://dart.dev/language/type-system

**Type System**:
- Sound type system (combination of static checking and runtime checks)
- Type annotations optional due to type inference
- Sound null safety enforced since Dart 3 (May 2023)

**Null Safety**:
- Non-nullable by default: `int` cannot be null
- Nullable via `?` suffix: `int?`
- If the type system determines non-nullable, guaranteed at runtime
- Late variables (`late`) for deferred initialization

**Generics**: Reified at runtime (unlike TypeScript/Java erasure). Supports
covariant generic types: `List<Cat>` is subtype of `List<Animal>`.

**Type Inference**: Analyzer infers types for fields, methods, local variables,
and most generic type arguments. Downward inference from context.

---

## 5. Academic Type Systems

### 5.1 Hindley-Milner (HM)

- **Key paper**: Damas & Milner, "Principal type-schemes for functional programs" (1982)
- **Wikipedia**: https://en.wikipedia.org/wiki/Hindley%E2%80%93Milner_type_system

**Core Properties**:
- Complete: always finds the most general (principal) type
- Decidable: type inference always terminates
- No programmer-supplied annotations needed
- Foundation for ML, OCaml, Haskell, Elm, Gleam, and many others

**Algorithm W**:
1. Descend AST, assign fresh type variables to every expression
2. Generate constraints from usage (e.g., if `f(x)` then `f: ?a -> ?b` and `x: ?a`)
3. Solve constraints via **unification** (find substitution making types equal)
4. If contradiction found, report type error
5. **Generalize** at `let` bindings: free type variables become universally quantified

**Unification**: Core operation. Takes two types (possibly containing unification
variables) and tries to make them equal. May fail (type error) or produce
substitutions constraining variables. Unification is the key insight -- "guess
a new type variable, and unification will steadily constrain that guess."

**Let Polymorphism**: At `let` bindings, type variables free in the binding but
not in the environment are generalized (universally quantified). This enables
polymorphic use: `let id = fn x -> x` gets type `forall a. a -> a`.

**Limitations**:
- No subtyping (extensions exist but complicate things)
- No higher-rank polymorphism (only rank-1)
- Becomes undecidable with certain extensions (e.g., GADTs without annotations)

**Languages Using HM**: OCaml, Haskell, ML, Elm, Gleam, Rust (extended), F#,
PureScript.

### 5.2 System F (Polymorphic Lambda Calculus)

- **Key papers**: Girard (1972), Reynolds (1974)
- **Wikipedia**: https://en.wikipedia.org/wiki/System_F

**Core Idea**: Extends simply typed lambda calculus with type variables and
universal quantification over types. "The essence of parametric polymorphism."

**Features**:
- Type abstraction: `\Lambda T. \lambda x:T. x` (polymorphic identity)
- Type application: `id [Int] 42` (instantiate type parameter)
- Explicit type annotations required (type inference is undecidable for System F)

**System F-omega**: Adds type-level functions (type operators). Foundation for
Haskell's Core intermediate language.

**System F-sub (F<:)**: Adds bounded quantification (subtyping). Important for
ML-family languages with record subtyping.

**Practical Relevance**: Modern Haskell compiles to an extension of System F
(GHC Core). Certain language features require type annotations and generate
intermediate code with type abstractions and applications.

### 5.3 Dependent Types

**Core Idea**: Types can depend on values. Example: `Vector n a` is a vector
of exactly `n` elements of type `a`, where `n` is a value.

**Idris**:
- General-purpose functional language with first-class dependent types
- Types are first-class values: passed to and returned from functions
- Supports "type-driven development" where types guide program construction
- Eager evaluation by default (with optional laziness)
- Practical I/O, networking, concurrency support
- Created by Edwin Brady

**Agda**:
- Extension of Martin-Lof's intuitionistic type theory
- Primarily a proof assistant; emphasis on type checking over execution
- More theoretical focus than Idris

**Key Property**: In dependently typed languages, the distinction between
types and values blurs. A type like `{v: List a | length v > 0}` is a
"non-empty list" type.

**Trade-off**: Full dependent types make type checking undecidable in general.
Languages handle this via totality checking (Agda) or accepting potential
non-termination (Idris).

### 5.4 Refinement Types (Liquid Haskell)

- **Repository**: https://github.com/ucsd-progsys/liquidhaskell
- **Documentation**: https://ucsd-progsys.github.io/liquidhaskell-tutorial/

**Core Idea**: Refine basic types with logical predicates. `{v: Int | v > 0}`
denotes positive integers. This is a "lightweight" form of dependent types.

**Verification Process**:
1. Generate logical constraints from code using Liquid Typing framework
2. Reduce type checking to validity of verification conditions (implications)
3. Use SMT solver (Z3, CVC4, MathSat) to solve constraints
4. If satisfiable, output SAFE; otherwise, report violation

**Key Features**:
- **Refinement reflection**: lifts Haskell functions into the refinement logic
  via singleton types
- **Abstract refinement types**: parameterize refinements themselves
- **Measure functions**: lift Haskell data constructors into the logic
- Compatible with standard Haskell (refinements in special comments/annotations)

**Trade-off**: Extremely expressive (can verify complex properties like sorted
output, resource usage bounds) but SMT solving can be slow and unpredictable.
Error messages from SMT failures can be opaque.

### 5.5 Bidirectional Type Checking

- **Key paper**: Dunfield & Krishnaswami, "Complete and Easy Bidirectional
  Typechecking for Higher-Rank Polymorphism" (2013)
- **Tutorial**: Christiansen, "Bidirectional Typing Rules: A Tutorial" (2013)

**Core Idea**: Two mutually recursive functions:
- **Synthesis** (`synth`): given a term, produce its type
- **Checking** (`check`): given a term and an expected type, verify the term has
  that type

**How It Works**:
- Elimination forms (function application, field access) use synthesis:
  the type can be determined from the function/object being used
- Introduction forms (lambdas, object literals) use checking: the expected
  type flows "downward" to guide inference
- This interleaving requires fewer annotations than full inference while
  remaining decidable for expressive type systems

**Advantages Over Algorithm W**:
- Remains decidable even for very expressive type systems (higher-rank
  polymorphism, dependent types)
- Produces better error messages (the expected type provides context)
- Easier to implement and extend
- Used by C#, Scala, Haskell (partially), and many modern languages

**Practical Impact**: Developers annotate major declarations (function signatures)
but not every variable. The checker "pushes down" expected types from annotations.

### 5.6 Row Polymorphism

- **Key paper**: Mitchell Wand, "Type Inference for Record Concatenation and
  Multiple Inheritance" (1989)
- **Tutorial**: https://bernsteinbear.com/blog/row-poly/

**Core Idea**: Polymorphism over record structure. A function can require certain
fields while being polymorphic over the "rest" of the record.

**Representation**:
- `TyEmptyRow`: closed row (no additional fields)
- `TyRow(fields, rest)`: open row with a tail variable
- Example: `{x: Int, ...r}` means "has field x:Int, plus whatever `r` has"

**Unification of Rows**: Four cases:
1. Identical fields: unify rest types
2. One row is subset: add missing fields to smaller, share rest
3. Mirror of case 2
4. Divergent fields: create fresh rest variable, unify with new rows

**Relies on Let Polymorphism**: `{x: Int, ...a} -> Int` generalizes to
`forall a. {x: Int, ...a} -> Int`, allowing each call site to instantiate fresh
type variables.

**Languages Using Row Polymorphism**:
- **PureScript**: extensible records and row-polymorphic effects
- **OCaml**: object types and polymorphic variants (not regular records)
- **Koka**: row-polymorphic effect types
- **Elm**: had extensible records, removed in 0.16

### 5.7 Effect Systems and Algebraic Effects

**Effect Systems** track computational effects (I/O, exceptions, mutation,
nondeterminism) in the type system.

**Koka** (Microsoft Research):
- Row-polymorphic effect types: `fun greet() : console ()` declares console effect
- Effect handlers intercept operations and can resume the computation
- Evidence passing translation: effects compiled to explicit parameter passing
- Type system ensures all effects are handled before execution
- Constant-time dispatch via static routing

**Eff**:
- Subtyping-based effect system (simpler than Koka's row-based approach)
- Multi-shot continuations (handler can invoke continuation multiple times)
- Research language, not production-oriented

**Key Design Dimensions**:
- Row-based (Koka) vs. subtyping-based (Eff) effect tracking
- One-shot vs. multi-shot continuations
- Compiler complexity trade-off: Koka's evidence passing requires monadic
  transformation; dynamic dispatch systems are simpler to implement

---

## 6. Cross-Cutting Concerns

### 6.1 Gradual Typing Theory

**Originator**: Jeremy Siek and Walid Taha (2006), "Gradual Typing for
Functional Languages"

**Formal Definition**: A type system where parts of a program can be dynamically
typed and other parts statically typed, with the programmer controlling which
parts are which through type annotations.

**Key Concepts**:
- **Unknown type (`?`)**: special type for unannotated expressions
- **Consistency relation**: `? ~ T` for any type T; replaces subtyping at
  boundaries between typed and untyped code
- **Bidirectional implicit conversion**: any type converts to `?` AND `?`
  converts to any type (unlike subtyping, which is directional)
- **Gradual guarantee**: adding or removing type annotations doesn't change
  program behavior (only whether errors are caught statically or at runtime)

**Sound vs. Unsound**:
- **Sound** (Typed Racket): runtime checks at typed/untyped boundaries ensure
  type invariants hold. Can have significant performance overhead (20x+ in
  worst cases). Blame tracking identifies which boundary violated the contract.
- **Unsound** (TypeScript, mypy): no runtime enforcement. Types are erased.
  Faster but type violations can silently corrupt data at runtime.

**The Performance Problem**: Sound gradual typing requires runtime checks at
every boundary between typed and untyped code. Research shows:
- Pycket JIT eliminates 90%+ of overhead
- Contract verification (Corpse Reviver) reduces overhead to near zero
- Rewriting hotspots in typed code eliminates boundaries

### 6.2 Structural vs. Nominal Typing

**Nominal** (Java, C#, Rust, Hack classes):
- Types compared by declared name/identity
- Subtyping must be explicitly declared (`extends`, `implements`, `impl Trait`)
- Prevents accidental type compatibility
- Enforces design intent as documentation
- Better for large teams and long-lived codebases

**Structural** (TypeScript, OCaml, Go interfaces, Elm):
- Types compared by shape/members
- No need to declare subtyping relationships ahead of time
- Supports unanticipated reuse
- More ergonomic for one-off types and prototyping
- Risk of accidental compatibility

**Hybrid Approaches**:
- Flow: structural for objects/functions, nominal for classes
- TypeScript: primarily structural, with nominal-like branded types via
  intersection with unique symbols
- Rust: nominal for traits, but structural reasoning in `where` clauses

### 6.3 Null/None/Nil Safety Approaches

| Approach | Languages | Mechanism |
|----------|-----------|-----------|
| Option/Maybe type | Rust, Haskell, OCaml, Elm, Scala | Algebraic type: `Some(T)` or `None`. Exhaustive pattern matching required. |
| Nullable types (`T?`) | Kotlin, Dart, Swift, C# 8+ | Compiler tracks nullability. `T` is non-nullable; `T?` is nullable. |
| Union with null | TypeScript, Flow | `T \| null` as explicit union. Narrowing via truthiness/equality checks. |
| Gradual/optional | Python, Ruby | `Optional[X]` = `Union[X, None]`. Depends on type checker enforcement. |

**Key Difference**: Option types support nesting (`Option<Option<T>>` is meaningful),
while nullable types do not (`T??` = `T?`).

### 6.4 Union Types, Intersection Types, and ADTs

**Union Types** (`A | B`): Value is one of the constituent types. Operations
valid on a union must be valid on ALL members. Used heavily in TypeScript, Flow,
Python (via `Union`), Typed Racket.

**Intersection Types** (`A & B`): Value has ALL properties of both types.
Used in TypeScript for mixin patterns and type narrowing.

**Algebraic Data Types** (Rust, Haskell, OCaml, Elm):
- **Sum types** (tagged unions): `enum Result<T, E> { Ok(T), Err(E) }`
- **Product types** (records/tuples): `struct Point { x: f64, y: f64 }`
- Exhaustive pattern matching enforced by compiler
- More structured than union types; each variant is explicitly named

**Trade-off**: Union types are more flexible (any types can be unioned) but less
structured. ADTs are more rigid but provide better exhaustiveness checking and
clearer semantics.

### 6.5 Type Narrowing / Flow-Sensitive Typing

Also called: occurrence typing (Typed Racket), control flow analysis (TypeScript),
type refinement, type narrowing.

**Core Idea**: The type of a variable changes based on control flow context.
After `if (x instanceof Foo)`, x has type `Foo` in the then-branch.

**Implementations**:
- **TypeScript**: Control flow graph with flow nodes, backward traversal
- **Typed Racket**: Predicate types with logical propositions
- **Rust**: Pattern matching, `if let`, `match`
- **Kotlin**: Smart casts after `is` checks
- **mypy/Pyright**: `isinstance()`, `is None`, type guard functions

### 6.6 Type Checking Performance

**Strategies for Scale**:
- **Incremental checking**: Only re-check changed files and dependents
  (TypeScript `--incremental`, mypy daemon, Flow server)
- **Lazy evaluation**: Only compute types when queried (Pyright, TypeScript checker)
- **Modular analysis**: Check at module boundaries (Flow types-first, Sorbet)
- **Parallelism**: Independent modules checked in parallel (Sorbet, Go rewrite)
- **Native code**: Rust/Go/C++ implementations vs. interpreted Python/JS
  (Sorbet 100K loc/s, Pyright 3-5x faster than mypy)

**Benchmarks**:
- Sorbet: ~100,000 lines/second/core (C++)
- Pyright: 3-5x faster than mypy (TypeScript/Node)
- TypeScript Go rewrite: 10x faster than JS implementation
- Flow types-first: 6x speedup at p90
- mypy daemon: 10x+ faster than cold mypy runs

### 6.7 Error Message Design

**Best Practices** (from Rust and Elm):

- **Primary label**: what went wrong (the error itself)
- **Secondary label**: why it went wrong (context)
- **Help/suggestion**: how to fix it
- **Notes**: additional context, related information
- Never suggest fixes in the error text itself -- use a separate "help" section
- Error messages should be concise (users see them many times)
- Verbose explanations available on demand (`--explain`, links to docs)
- Use color and formatting: red for errors, blue for context
- Point to specific locations in source code with underlines/carets

**Elm's Philosophy**: Compiler errors as a user guide. Plain English descriptions
of the exact mistake, with actionable suggestions and educational explanations.

**Anti-Patterns**:
- GHC Haskell: historically cryptic error messages from unification failures
  (improving with recent versions)
- C++ templates: error messages expose implementation details of template
  instantiation chains
- mypy: error codes without fix suggestions

### 6.8 Runtime Type Information

| Language | Runtime Types | Mechanism |
|----------|--------------|-----------|
| Rust | Partial | `TypeId` for `'static` types, no generic reification |
| TypeScript | None | Types fully erased at compile time |
| Python | Full | `__annotations__` available, but unchecked |
| Dart | Full | Reified generics (`is List<int>` works at runtime) |
| Hack | Opt-in | Type erasure by default, `reify` keyword for runtime access |
| Haskell | Partial | Typeclass dictionaries passed at runtime |
| Java | Partial | Type erasure for generics, reflection for classes |

---

## Sources

### Python
- [mypy documentation](https://mypy.readthedocs.io/)
- [Pyright mypy comparison](https://github.com/microsoft/pyright/blob/main/docs/mypy-comparison.md)
- [PEP 484 - Type Hints](https://peps.python.org/pep-0484/)
- [PEP 561 - Distributing Type Information](https://peps.python.org/pep-0561/)
- [Pytype Typing FAQ](https://google.github.io/pytype/typing_faq.html)
- [New Python Type Checkers Comparison](https://sinon.github.io/future-python-type-checkers/)
- [Pyrefly Conformance Comparison](https://pyrefly.org/blog/typing-conformance-comparison/)

### Rust
- [Rust Type Inference Dev Guide](https://rustc-dev-guide.rust-lang.org/type-inference.html)
- [Polonius Project](https://github.com/rust-lang/polonius)
- [Polonius Revisited](https://smallcultfollowing.com/babysteps/blog/2023/09/22/polonius-part-1/)
- [Rust Diagnostics Guide](https://rustc-dev-guide.rust-lang.org/diagnostics.html)
- [Rust RFC 1644 - Error Format](https://rust-lang.github.io/rfcs/1644-default-and-expanded-rustc-errors.html)

### TypeScript
- [TypeScript Handbook](https://www.typescriptlang.org/docs/handbook/)
- [Flow Nodes Implementation](https://effectivetypescript.com/2024/03/24/flownodes/)
- [TypeScript Compiler Internals](https://basarat.gitbook.io/typescript/overview)
- [TypeScript Turing Completeness](https://github.com/microsoft/TypeScript/issues/14833)
- [TypeScript Go Rewrite](https://medium.com/codex/typescripts-monumental-shift-a-10x-faster-type-checker-with-go-670d7bcab42d)
- [TypeScript Architecture](https://www.iamtk.co/a-high-level-architecture-of-the-typescript-compiler)

### Gradually-Typed Languages
- [Flow Announcement](https://engineering.fb.com/2014/11/18/web/flow-a-new-static-type-checker-for-javascript/)
- [Sorbet (Stripe)](https://stripe.dev/blog/sorbet-stripes-type-checker-for-ruby)
- [Why Sorbet Is Fast](https://blog.nelhage.com/post/why-sorbet-is-fast/)
- [Hack Language](https://hacklang.org/)
- [Hack Reified Generics](https://docs.hhvm.com/hack/reified-generics/reified-generics/)
- [Typed Racket Guide](https://docs.racket-lang.org/ts-guide/)
- [Typed Clojure](https://github.com/clojure/core.typed)
- [Dart Type System](https://dart.dev/language/type-system)
- [Dart Null Safety](https://dart.dev/null-safety/understanding-null-safety)

### Academic
- [Gradual Typing (Siek)](https://jsiek.github.io/home/WhatIsGradualTyping.html)
- [Gradual Typing for Functional Languages (PDF)](https://jsiek.github.io/home/siek06gradual.pdf)
- [Is Sound Gradual Typing Dead?](https://dl.acm.org/doi/abs/10.1145/2837614.2837630)
- [Hindley-Milner Type System (Wikipedia)](https://en.wikipedia.org/wiki/Hindley%E2%80%93Milner_type_system)
- [System F (Wikipedia)](https://en.wikipedia.org/wiki/System_F)
- [Bidirectional Typechecking (Dunfield & Krishnaswami)](https://arxiv.org/abs/1306.6032)
- [Row Polymorphism with HM](https://bernsteinbear.com/blog/row-poly/)
- [Koka Language](https://koka-lang.github.io/koka/doc/book.html)
- [Liquid Haskell](https://github.com/ucsd-progsys/liquidhaskell)
- [Retrofitting Type Systems (LWN)](https://lwn.net/Articles/1062177/)
- [Null Safety vs Option Types](https://www.ppl-lang.dev/blog/null-safety-vs-maybe-option/index.html)
- [Structural vs Nominal Typing](https://medium.com/@thejameskyle/type-systems-structural-vs-nominal-typing-explained-56511dd969f4)
