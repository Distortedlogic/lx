# Type System Design Patterns, Trade-offs, and Lessons

Distilled from the landscape survey. Organized as design patterns and decision
points for anyone designing a type system for a new language, particularly one
that may start dynamic and add types incrementally.

---

## Table of Contents

1. [Foundational Decisions](#1-foundational-decisions)
2. [Type Inference Patterns](#2-type-inference-patterns)
3. [Gradual Typing Patterns](#3-gradual-typing-patterns)
4. [Type Checking Architecture](#4-type-checking-architecture)
5. [Specific Feature Patterns](#5-specific-feature-patterns)
6. [Error Message Patterns](#6-error-message-patterns)
7. [Lessons from Real-World Adoptions](#7-lessons-from-real-world-adoptions)
8. [Anti-Patterns](#8-anti-patterns)
9. [Decision Matrix for a New Language](#9-decision-matrix-for-a-new-language)

---

## 1. Foundational Decisions

### 1.1 Pattern: Choose Your Soundness Budget

Every type system makes a soundness/usability trade-off. The key question is:
"What percentage of programs that type-check will actually be free of type
errors at runtime?"

**Fully Sound** (Rust, Dart 3, Typed Racket):
- Every well-typed program is guaranteed free of type errors at runtime
- Requires more annotation effort and more complex type system
- May reject valid programs that are too dynamic for the type system to verify
- Performance overhead at typed/untyped boundaries (gradual systems)

**Intentionally Unsound** (TypeScript, Flow, mypy):
- Prioritizes developer productivity and migration ease
- Specific, documented escape hatches (`any`, covariant arrays, type assertions)
- Type errors can silently occur at runtime
- Much easier to adopt incrementally

**Partially Sound** (Hack, Sorbet):
- Sound core with explicit unsound escape hatches
- Runtime enforcement at boundaries (Hack enforces parameter/return types)
- Good middle ground for production systems

**Lesson**: TypeScript's intentional unsoundness was key to its adoption.
Typed Racket's full soundness limited adoption due to performance overhead.
For a new language, decide upfront which soundness bugs you'll accept, and
document them explicitly.

### 1.2 Pattern: Structural vs. Nominal -- Pick a Default, Allow the Other

No language is purely one or the other in practice.

**Structural-default** (TypeScript, Go, OCaml objects):
- Best for: scripting, prototyping, duck-typing idioms, JSON-heavy code
- Risk: accidental type compatibility, harder to enforce design intent
- Mitigation: branded types, opaque types

**Nominal-default** (Rust, Java, Hack classes):
- Best for: large teams, long-lived codebases, explicit architecture
- Risk: boilerplate, difficulty with unanticipated reuse
- Mitigation: trait objects, structural protocols

**Hybrid** (Flow, Hack):
- Structural for objects/functions, nominal for classes
- Most flexible but more complex mental model

**Lesson**: If your language targets agentic/scripting use cases with heavy
JSON and message passing, structural typing is natural. If you want to enforce
protocol contracts between agents, nominal typing (or structural typing with
explicit protocol declarations) works better.

### 1.3 Pattern: Choose Your Null Strategy Early

This decision is nearly impossible to retrofit (see Dart's multi-year migration).

| Strategy | Annotation Cost | Safety | Nesting | Runtime Cost |
|----------|----------------|--------|---------|-------------|
| Option/Maybe ADT | Higher (explicit wrapping) | Highest | Yes | Pattern match dispatch |
| Nullable `T?` | Lower (just add `?`) | High | No | Null check |
| Union with null | Medium | Medium | No | Type narrowing |
| No null (everything optional by default) | None | Low | N/A | None |

**Lesson**: For a new language, nullable types (`T?`) are the sweet spot of
ergonomics and safety. Option types are more principled but require more
syntactic overhead. If the language is dynamically typed at its core, union
with null (TypeScript style) integrates most naturally.

---

## 2. Type Inference Patterns

### 2.1 Pattern: Local Inference with Annotated Boundaries

The most successful practical pattern: infer types within function bodies,
require annotations at function boundaries (parameters and return types).

**Used by**: Sorbet, Rust, Kotlin, Swift, Dart, Flow (types-first)

**Why it works**:
- Enables parallel type checking (each function body is independent)
- Module boundaries are self-documenting
- Error messages stay local (inference failures don't cascade across modules)
- Flow (types-first) saw 6x performance improvement by requiring boundary
  annotations

**Implementation**:
- Forward-only, single-pass inference within bodies (Sorbet)
- Or bidirectional checking with expected return type flowing down (Rust, Kotlin)

### 2.2 Pattern: Full Hindley-Milner Inference

Complete inference with no annotations required anywhere.

**Used by**: OCaml, Haskell, Elm, Gleam

**Advantages**:
- Minimal annotation burden
- Finds principal (most general) type automatically
- Elegant formal properties

**Disadvantages**:
- Error messages can be terrible (error surfaces far from cause)
- Becomes undecidable with extensions (GADTs, higher-rank polymorphism)
- Whole-program analysis required (limits parallelism)
- Module boundaries are opaque without explicit signatures

**Lesson**: Full HM inference is beautiful in theory but produces poor error
messages in practice because unification failures surface at constraint
intersection points, which may be far from the actual bug. Bidirectional
checking produces better errors by having an expected type to compare against.

### 2.3 Pattern: Bidirectional Type Checking

Two mutually recursive functions: synthesis (term -> type) and checking
(term + expected type -> ok/error).

**Used by**: C#, Scala, Haskell (partially), GHC, many modern languages

**Why it's gaining ground**:
- Remains decidable even for higher-rank polymorphism and dependent types
  (unlike HM extensions)
- Better error messages: the expected type provides context for errors
- Easier to implement and extend than Algorithm W
- Requires annotations only on "top-level" declarations, not every binding
- Scales to very expressive type systems

**Implementation sketch**:
```
synth(Var(x))      = lookup(env, x)
synth(App(f, arg)) = let (A -> B) = synth(f) in check(arg, A); return B
check(Lam(x, e), A -> B) = check(e[x:A], B)
check(e, T)        = let T' = synth(e) in assert(T' <: T)
```

**Lesson**: For a new language, bidirectional checking is likely the best
starting point. It combines the benefits of inference (within expressions)
with the clarity of annotations (at declarations) and produces good error
messages from day one.

### 2.4 Pattern: Lazy/JIT Type Evaluation

Compute types on demand rather than eagerly analyzing all code.

**Used by**: Pyright, TypeScript (checker)

**How it works**: When a type is needed (e.g., for an IDE hover or error
check), recursively evaluate dependencies. Cache results aggressively.

**Advantages**:
- Responsive editor integration (compute only what's visible)
- Natural fit for Language Server Protocol
- Avoids wasted work on unreferenced code

**Lesson**: If your type checker needs to power an IDE/editor, lazy evaluation
is almost mandatory for responsive UX.

---

## 3. Gradual Typing Patterns

### 3.1 Pattern: The Escape Hatch Type

Every gradual system needs a way to say "I don't know / don't care about this
type." The design of this escape hatch shapes the entire system.

| Language | Escape Hatch | Behavior |
|----------|-------------|----------|
| TypeScript | `any` | Disables all checking; infects surrounding code |
| Python/mypy | `Any` | Compatible with all types in both directions |
| Sorbet | `T.untyped` | Equivalent to `Any` |
| Flow | `any` / `mixed` | `any` disables checking; `mixed` is safe top type |

**Lesson**: Having TWO escape hatches is better than one:
- An `unknown` type (safe -- must be narrowed before use)
- An `any` type (unsafe -- bypasses checking entirely)
TypeScript added `unknown` in 3.0 specifically because `any` was too dangerous.

### 3.2 Pattern: Strictness Levels

Allow projects/files/modules to opt into different levels of type checking.

**Sorbet**: `# typed: false` | `true` | `strict` | `strong`
**mypy**: `--strict` flag, per-module config, `# type: ignore`
**TypeScript**: `strict` flag, individual strict flags, `@ts-ignore`

**Why it works**: Enables incremental adoption. Teams can start with loose
checking and tighten over time. CI can enforce minimum strictness levels.

**Sorbet's approach is exemplary**: file-level strictness with CI-enforced
minimum levels. After 4 years, 85% of Stripe's non-test files are strict.

### 3.3 Pattern: Module-Level Gradual Typing

Typed Racket's approach: entire modules are typed or untyped. Contracts
enforced at module boundaries.

**Advantages**: Clear boundaries, easier to reason about
**Disadvantages**: All-or-nothing for each module, expensive boundary checks

### 3.4 Pattern: Stub Files for External Types

Provide type information for code you can't modify.

**Python**: `.pyi` stub files alongside `.py` source
**TypeScript**: `.d.ts` declaration files
**Flow**: `.flow` files and `flowtyped` repository

**Lesson**: A community-maintained repository of type stubs (DefinitelyTyped
for TypeScript, typeshed for Python) is essential for ecosystem adoption.
Without it, every user hits walls at library boundaries.

### 3.5 Pattern: Runtime Enforcement at Boundaries

Insert runtime checks where typed code calls untyped code (or vice versa).

**Used by**: Typed Racket (contracts), Sorbet (runtime checks from sigs),
Hack (parameter/return type enforcement)

**Trade-off**: Ensures soundness but can have 20x+ performance overhead in
pathological cases. Optimization techniques (JIT, contract verification) can
reduce this to near zero.

---

## 4. Type Checking Architecture

### 4.1 Pattern: Server Architecture

Keep the type checker running as a persistent process. Analyze the full
codebase on startup, then incrementally update on file changes.

**Used by**: Flow, mypy daemon, Pyright, TypeScript (tsc --watch), Sorbet

**Key Benefits**:
- Amortizes startup cost
- Keeps ASTs and type information in memory
- Enables fine-grained incremental updates
- Natural fit for editor integration via LSP

### 4.2 Pattern: Modular Analysis with Boundary Types

Check each module independently using only the type signatures at boundaries.

**Used by**: Flow (types-first), Sorbet, Rust (crate boundaries), Hack

**Requirements**:
- All module exports must have type annotations
- No cross-module inference (or very limited)
- Module interface files serve as "contracts"

**Benefits**:
- Aggressively parallel checking
- Changes inside a module body don't trigger rechecking of dependents
- Clear error boundaries (errors don't cascade across modules)

**Lesson**: Flow's evolution is instructive. They started with cross-module
inference, hit performance walls at scale, then migrated to types-first
(boundary annotations required) for 6x performance improvement.

### 4.3 Pattern: Flat Array Data Structures

For maximum performance, avoid pointer-heavy heap-allocated structures.

**Used by**: Sorbet

**Implementation**:
- Global state uses flat arrays of objects
- 32-bit indexes (`NameRef`, `SymbolRef`) instead of 64-bit pointers
- String interning: all identifiers stored once, compared as integers
- Maximizes CPU cache locality
- Uses `absl::InlinedVector` and `absl::flat_hash_map`

**Result**: 100,000 lines/second/core, one of the fastest production type checkers.

### 4.4 Pattern: Incremental Computation Frameworks

Use a dependency-tracking computation framework rather than ad-hoc caching.

**Used by**: Rust (queries system), ty (salsa library)

**How it works**: Functions are memoized with automatic dependency tracking.
When an input changes, only functions that transitively depend on it are
recomputed.

**Benefit**: Correctness of incrementality is handled by the framework rather
than manual cache invalidation.

### 4.5 Pattern: Implementation Language Matters

| Impl Language | Examples | Relative Speed |
|--------------|----------|----------------|
| C++ | Sorbet | Fastest (100K loc/s/core) |
| Rust | ty, Pyrefly, Zuban | Very fast |
| Go | TypeScript Go rewrite | 10x vs JS version |
| OCaml | Flow, Pyre | Fast |
| TypeScript/JS | Pyright, tsc | 3-5x slower than native |
| Python | mypy | Slowest |

**Lesson**: For a type checker that will be used in editor integration,
native-code implementation (Rust, C++, Go) provides dramatically better
responsiveness. The TypeScript team's Go rewrite achieving 10x speedup
is the most dramatic recent example.

---

## 5. Specific Feature Patterns

### 5.1 Pattern: Discriminated Unions for ADTs

Even in structurally-typed languages, a "tag" field enables exhaustive matching.

**TypeScript**: Union of objects with a literal-typed tag field:
```typescript
type Shape = { kind: "circle", radius: number } | { kind: "rect", w: number, h: number }
```
`switch` on `kind` narrows to the specific variant.

**Lesson**: If your language doesn't have native ADTs, discriminated unions
(union types + a literal-typed tag field + narrowing) can provide most of the
same functionality. TypeScript proves this works at scale.

### 5.2 Pattern: Control Flow Narrowing

Track how control flow affects variable types.

**Implementation approaches**:
- **Control flow graph** (TypeScript): Build CFG during parsing, traverse backward
  from usage point to find narrowing guards
- **Occurrence typing** (Typed Racket): Predicate types carry logical propositions
  about what's true/false
- **Pattern matching** (Rust, Elm): `match`/`case` expressions with exhaustiveness

**TypeScript's specific approach**:
- Binder greedily constructs CFG during parsing
- Checker lazily evaluates types by walking backward up the CFG
- Narrowing functions: `narrowTypeByTruthiness`, `narrowTypeByTypeof`,
  `narrowTypeByInstanceof`, `narrowTypeByEquality`
- Results cached per flow node

**Design Decision**: Mutation complicates narrowing. Typed Racket refuses to
narrow mutable variables (`set!`). TypeScript narrows mutable variables but
invalidates narrowing after function calls that might modify them.

### 5.3 Pattern: Row Polymorphism for Extensible Records

Allow functions to work with records that have "at least" certain fields.

**Implementation**:
- Rows are field-set + rest-variable: `{x: Int, ...r}`
- Unification handles four cases of field overlap
- Relies on let-polymorphism for generalization
- Compatible with HM inference (Wand 1989)

**Used by**: PureScript (records + effects), OCaml (objects, polymorphic variants),
Koka (effect types)

**Alternative**: TypeScript's structural typing provides similar capabilities
without explicit row variables, at the cost of less precise types.

### 5.4 Pattern: Effect Tracking in Types

Make side effects visible in function signatures.

**Approaches**:
- **Row-polymorphic effects** (Koka): `fun greet() : console ()` -- effects are
  part of the type, composed via row polymorphism
- **Monadic effects** (Haskell): `IO a`, `State s a` -- effects tracked via
  monad type constructors
- **Linear types** (Rust's ownership): Resource management tracked via
  affine/linear type constraints

**Trade-off**: Explicit effect tracking provides safety guarantees but adds
annotation burden. Most practical languages don't track effects in types
(TypeScript, Python, Ruby, Java, Go).

**Lesson**: For a new language, effect tracking is high cost, high reward.
Koka shows it can be ergonomic with row polymorphism and good inference, but
mainstream adoption remains limited.

### 5.5 Pattern: Reified vs. Erased Generics

**Erased** (TypeScript, Java, Hack default):
- Generic type info unavailable at runtime
- Cannot do `new T()` or `x instanceof List<Int>`
- Better performance (no runtime overhead)
- Simpler compilation

**Reified** (Dart, C#, Hack opt-in):
- Generic type info available at runtime
- Can do runtime type tests on generic types
- Slight performance overhead
- More expressive runtime behavior

**Hack's approach** is interesting: erasure by default, opt-in reification with
`reify` keyword. `<<__Enforceable>>` marks which type parameters can be fully
checked at runtime. This gives developers control over the trade-off.

### 5.6 Pattern: Refinement Types for Advanced Verification

Layer logical predicates on top of base types for property verification.

**Implementation** (Liquid Haskell):
1. Annotate types with predicates: `{v: Int | v > 0}`
2. Generate verification conditions from code
3. Discharge via SMT solver (Z3, CVC4)
4. Report SAFE or counterexample

**Trade-off**: Extremely powerful (can verify sort correctness, resource bounds)
but SMT solving is unpredictable in performance, and error messages from solver
failures are often opaque.

**Lesson**: Full refinement types are too heavy for a general-purpose language.
But lightweight refinements (non-null, non-empty, positive) could be valuable
with simpler checking strategies.

---

## 6. Error Message Patterns

### 6.1 Pattern: The Elm/Rust Error Format

```
error[E0308]: mismatched types
 --> src/main.rs:4:18
  |
4 |     let x: i32 = "hello";
  |            ---   ^^^^^^^ expected `i32`, found `&str`
  |            |
  |            expected due to this
```

**Components**:
1. **Error code** with severity level
2. **Source location** with file, line, column
3. **Primary span** (what's wrong) -- red underline
4. **Secondary span** (why it's wrong) -- blue underline
5. **Help** (how to fix) -- separate sub-diagnostic
6. **Note** (additional context)

### 6.2 Pattern: Suggest, Don't Just Complain

Good error messages include actionable suggestions:
- "Did you mean X?" for typos
- "Try adding Y" for missing annotations
- "Consider using Z" for idiomatic alternatives

**Anti-pattern**: Showing internal type representations or unification failures
in error messages (early GHC, C++ template errors).

### 6.3 Pattern: Error Recovery and Continuation

Don't stop at the first error. Report as many independent errors as possible
in a single run. This requires:
- Error recovery in the parser (skip to next statement/declaration)
- Poison/error types in the type checker (an error type is compatible with
  everything to prevent cascading errors)
- TypeScript's `never` and error nodes serve this purpose

### 6.4 Pattern: Layered Verbosity

- **Default**: concise error with location and primary message
- **Expanded**: add context, related locations, secondary spans
- **Explained**: full documentation with examples and common fixes
  (Rust's `--explain E0308`)

---

## 7. Lessons from Real-World Adoptions

### 7.1 TypeScript: Unsoundness as a Feature

TypeScript's deliberate unsoundness (covariant arrays, bivariant function
parameters, `any` type) was not a bug but a conscious design choice to
maximize adoption. JavaScript's ecosystem is deeply dynamic, and a sound
type system would reject too much existing code.

**Lesson**: When layering types onto an existing dynamic language, pragmatism
beats purity. Document your unsoundness, provide escape hatches, and focus
on catching the bugs that matter most.

### 7.2 Sorbet: Performance as Adoption Driver

Sorbet's team made performance a first-class requirement from day one, choosing
C++ and designing data structures for cache locality. The result (100K loc/s)
made the type checker feel invisible -- developers never wait for it.

**Lesson**: A slow type checker won't get adopted. If type checking takes more
than a few hundred milliseconds, developers will turn it off. Target <200ms
for incremental checks on every save.

### 7.3 Flow vs. TypeScript: Ecosystem Wins

Flow and TypeScript launched around the same time with similar goals. TypeScript
won because:
- Better IDE integration (first-class LSP support)
- DefinitelyTyped community repository of type definitions
- Wider ecosystem investment (Angular, Vue, etc. adopted TypeScript)
- More stable API and better documentation

Flow retreated to Facebook's internal needs.

**Lesson**: A type system's success depends as much on ecosystem and tooling
as on technical merits.

### 7.4 Typed Racket: The Sound Gradual Typing Performance Wall

Typed Racket proved that sound gradual typing is possible and elegant. It also
showed that runtime boundary checks can create catastrophic performance overhead
(20x+ in benchmarks). This led to years of "Is sound gradual typing dead?"
research.

**Lesson**: If you choose sound gradual typing, invest heavily in optimization
of boundary checks (JIT compilation, contract verification, static analysis to
eliminate checks).

### 7.5 Python: Too Many Type Checkers

Python has mypy, Pyright, Pyre, Pytype, ty, Pyrefly, and Zuban. They disagree
on many edge cases because the Python typing spec is underspecified.

**Lesson**: Define your type system spec precisely from the start. A conformance
test suite is essential. Multiple implementations are fine only if they agree
on semantics.

### 7.6 Dart: Migrating to Sound Null Safety

Dart 2.12 introduced null safety with a multi-year migration:
- Unsound null safety first (mixed codebases)
- Sound null safety enforced in Dart 3
- Migration tooling to automatically add `?` where needed

**Lesson**: Null safety migration is painful but achievable with good tooling,
gradual enforcement, and a clear timeline.

### 7.7 Hack: From PHP to Full Types

Hack is the most successful example of evolving a dynamic language into a
fully typed one. Key factors:
- Single organization (Meta) could mandate migration
- Runtime enforcement (not just static checking) caught real bugs
- Gradual transition preserved developer velocity

**Lesson**: When you control the entire codebase, you can be more aggressive
about type system migration. Open ecosystems require more gradual approaches.

---

## 8. Anti-Patterns

### 8.1 Anti-Pattern: Cross-Module Type Inference

Inferring types across module boundaries without annotations causes:
- Non-local error messages
- Parallelism barriers
- Performance degradation at scale
- Unstable API surfaces (internal changes break dependents)

Flow learned this the hard way and migrated to types-first.

### 8.2 Anti-Pattern: Join-Based Type Merging

Using "join" (find common supertype) instead of "union" when merging types
discards precision. mypy does this and produces false positives that Pyright
avoids by using unions.

### 8.3 Anti-Pattern: Skipping Unannotated Code

mypy's default behavior of skipping unannotated functions means type errors
in unannotated code go unreported. This surprises users who expect the type
checker to check all code.

### 8.4 Anti-Pattern: Exposing Internal Type Representations

C++ template errors expose the full instantiation chain. GHC error messages
show internal unification variables. This is hostile to users.

### 8.5 Anti-Pattern: Over-Complex Type Systems

If users need a PhD in type theory to understand error messages, the type
system is too complex. Sorbet's team explicitly avoided "super-complex type
systems," preferring simplicity that "scales better and is easier for users
to learn and understand."

### 8.6 Anti-Pattern: Swallowing Type Information at Boundaries

TypeScript's `any` type, once introduced, infects surrounding code by
suppressing all checks. The safer `unknown` type forces narrowing before use.

---

## 9. Decision Matrix for a New Language

### 9.1 If the language is dynamically typed at its core

**Recommended approach**: Gradual typing with bidirectional checking

1. Start with `Any`/`unknown` as the default type for unannotated expressions
2. Require annotations at function boundaries for checked code
3. Use structural typing (matches dynamic language idioms)
4. Implement control flow narrowing for type guards
5. Use union types for null safety (`T | null`)
6. Provide strictness levels for incremental adoption
7. Build a persistent server for editor integration

### 9.2 If the language is statically typed from the start

**Recommended approach**: Bidirectional checking with local HM inference

1. Require function parameter annotations, infer return types
2. Full HM inference within function bodies
3. Nominal typing with structural protocols/interfaces
4. ADTs with exhaustive pattern matching for null safety
5. Row polymorphism for extensible records/messages
6. Effect system if targeting safety-critical or concurrent workloads

### 9.3 Critical Success Factors

1. **Performance**: <200ms incremental checks. Native implementation if possible.
2. **IDE Integration**: LSP support from day one. Lazy type evaluation.
3. **Error Messages**: Elm/Rust style with location, context, and suggestions.
4. **Escape Hatches**: Both `any` (unsafe) and `unknown` (safe) varieties.
5. **Ecosystem**: Type stubs for common libraries. Community repository.
6. **Specification**: Precise spec with conformance test suite.
7. **Migration Path**: Strictness levels for incremental adoption.

### 9.4 Features Ranked by Impact/Cost Ratio

| Feature | Impact | Implementation Cost | Ratio |
|---------|--------|-------------------|-------|
| Control flow narrowing | Very High | Medium | Best |
| Local type inference | High | Low-Medium | Great |
| Union types | High | Medium | Great |
| Null safety (T?) | Very High | Medium | Great |
| Generics (parametric polymorphism) | High | Medium-High | Good |
| Bidirectional checking | High | Medium | Good |
| Discriminated unions | High | Medium | Good |
| Incremental checking | High | High | Good |
| Structural typing | Medium | Medium | Moderate |
| Row polymorphism | Medium | High | Moderate |
| Effect tracking | Medium | Very High | Low |
| Dependent types | High | Very High | Low |
| Refinement types | Medium | Very High | Low |

### 9.5 Recommended Reading Order for Implementers

1. **Start here**: "Bidirectional Typing Rules: A Tutorial" (Christiansen 2013)
   -- practical introduction to the most useful type checking approach
2. **Foundations**: Hindley-Milner type system (Wikipedia article is excellent)
   -- understand what HM gives you and its limits
3. **Gradual typing**: Siek & Taha 2006 paper -- understand the formal
   foundations if mixing typed and untyped code
4. **Flow-sensitive typing**: TypeScript flow nodes blog post (Effective TS 2024)
   -- practical implementation of control flow narrowing
5. **Performance**: "Why Sorbet Is Fast" (Nelhage) -- data structure design
   for performance
6. **Row polymorphism**: "Adding Row Polymorphism to Damas-Hindley-Milner"
   (Bernstein) -- if you need extensible records
7. **Effects**: Koka book and papers (Leijen) -- if considering effect types
8. **Error messages**: Rust RFC 1644 and Elm's error message philosophy

---

## Sources

- [Bidirectional Typing Tutorial (Christiansen)](https://davidchristiansen.dk/tutorials/bidirectional.pdf)
- [Complete and Easy Bidirectional Typechecking (Dunfield & Krishnaswami)](https://arxiv.org/abs/1306.6032)
- [Gradual Typing for Functional Languages (Siek & Taha)](https://jsiek.github.io/home/siek06gradual.pdf)
- [What Is Gradual Typing (Siek)](https://jsiek.github.io/home/WhatIsGradualTyping.html)
- [Is Sound Gradual Typing Dead? (Takikawa et al.)](https://dl.acm.org/doi/abs/10.1145/2837614.2837630)
- [Why Sorbet Is Fast (Nelhage)](https://blog.nelhage.com/post/why-sorbet-is-fast/)
- [Flow Nodes: How Type Inference Is Implemented (Effective TypeScript)](https://effectivetypescript.com/2024/03/24/flownodes/)
- [Pyright vs mypy Comparison](https://github.com/microsoft/pyright/blob/main/docs/mypy-comparison.md)
- [Row Polymorphism with HM (Bernstein)](https://bernsteinbear.com/blog/row-poly/)
- [Rust Type Inference Dev Guide](https://rustc-dev-guide.rust-lang.org/type-inference.html)
- [Rust Diagnostics Guide](https://rustc-dev-guide.rust-lang.org/diagnostics.html)
- [Rust Error Format RFC 1644](https://rust-lang.github.io/rfcs/1644-default-and-expanded-rustc-errors.html)
- [Koka Language Book](https://koka-lang.github.io/koka/doc/book.html)
- [Algebraic Effects (Leijen, MSR)](https://www.microsoft.com/en-us/research/wp-content/uploads/2016/08/algeff-tr-2016-v2.pdf)
- [Liquid Haskell](https://github.com/ucsd-progsys/liquidhaskell)
- [TypeScript Compiler Internals](https://basarat.gitbook.io/typescript/overview)
- [TypeScript Go Rewrite](https://medium.com/codex/typescripts-monumental-shift-a-10x-faster-type-checker-with-go-670d7bcab42d)
- [Hindley-Milner (Wikipedia)](https://en.wikipedia.org/wiki/Hindley%E2%80%93Milner_type_system)
- [System F (Wikipedia)](https://en.wikipedia.org/wiki/System_F)
- [Structural vs Nominal Typing](https://medium.com/@thejameskyle/type-systems-structural-vs-nominal-typing-explained-56511dd969f4)
- [Null Safety vs Option Types](https://www.ppl-lang.dev/blog/null-safety-vs-maybe-option/index.html)
- [Retrofitting Type Systems (LWN)](https://lwn.net/Articles/1062177/)
- [New Python Type Checkers](https://sinon.github.io/future-python-type-checkers/)
- [Typed Racket Guide](https://docs.racket-lang.org/ts-guide/)
- [Sorbet (Stripe)](https://stripe.dev/blog/sorbet-stripes-type-checker-for-ruby)
- [Flow Announcement (Meta)](https://engineering.fb.com/2014/11/18/web/flow-a-new-static-type-checker-for-javascript/)
- [Hack Language](https://hacklang.org/)
- [Dart Type System](https://dart.dev/language/type-system)
