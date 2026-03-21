# Pattern Matching: Exhaustiveness Algorithms, Compilation, and Design Trade-offs

Distilled from the landscape survey. Organized as algorithm descriptions,
implementation strategies, and design decision points for building pattern
matching into a new language.

---

## Table of Contents

1. [Exhaustiveness Checking Algorithms](#1-exhaustiveness-checking-algorithms)
2. [Pattern Compilation Strategies](#2-pattern-compilation-strategies)
3. [Destructuring and Binding](#3-destructuring-and-binding)
4. [Guard Expressions](#4-guard-expressions)
5. [Or-Patterns](#5-or-patterns)
6. [Pattern Matching on Records and Maps](#6-pattern-matching-on-records-and-maps)
7. [Design Trade-offs](#7-design-trade-offs)
8. [Decision Matrix for a New Language](#8-decision-matrix-for-a-new-language)

---

## 1. Exhaustiveness Checking Algorithms

### 1.1 Maranget's Usefulness Algorithm (2007)

The foundational algorithm for both exhaustiveness checking and redundancy
detection. Published as "Warnings for Pattern Matching" in the Journal of
Functional Programming, Vol. 17, Issue 3, pages 387-421.

**Core definition**: A pattern `q` is **useful** with respect to a set of
patterns `P = {p_1, ..., p_n}` if there exists some value `v` such that `v`
matches `q` but does not match any `p_i`.

**Two applications from one algorithm:**
1. A match is **exhaustive** if the wildcard pattern `_` is not useful w.r.t.
   all arms -- meaning every possible value is already covered.
2. A pattern `p_i` is **redundant** if it is not useful w.r.t. `{p_1, ..., p_{i-1}}`
   -- meaning everything it could match is already caught by prior arms.

**The algorithm**: Operates on a **pattern matrix** where rows are match arms
and columns correspond to positions in the matched value. The core operation is
**specialization**:

```
specialize(constructor c, pattern p):
  if p is constructor c' with fields f1..fn:
    if c == c': return f1..fn     (expose fields)
    else: return nothing          (wrong constructor)
  if p is wildcard _:
    return _.._ (n wildcards, one per field of c)
  if p is or-pattern p1|p2:
    return specialize(c, p1) union specialize(c, p2)
```

The recursive algorithm:

```
is_useful(matrix P, pattern-row q):
  base case: if P has zero columns:
    return P has zero rows  (useful iff no prior patterns exist)

  pick column i (heuristic choice)
  collect all constructors appearing in column i of P

  if constructors are complete (cover the entire type):
    for each constructor c:
      P' = specialize(c, P)
      q' = specialize(c, q)
      if q' exists and is_useful(P', q'): return true
    return false
  else (incomplete constructors):
    P_default = default_matrix(P, column i)
    q_default = default_row(q, column i)
    if q has a constructor c in column i:
      P' = specialize(c, P)
      q' = specialize(c, q)
      return is_useful(P', q')
    else:
      return is_useful(P_default, q_default)
```

The **default matrix** operation retains only rows whose column `i` is a
wildcard (or variable), dropping the column. This corresponds to "what matches
when none of the explicit constructors appear."

**Witness generation**: When `is_useful` returns true, the algorithm can
reconstruct a witness value by tracking which constructors were chosen during
specialization. These witnesses are reported in non-exhaustive match warnings
(e.g., "patterns `None` and `Some(None)` not covered").

Source: [Warnings for Pattern Matching (Maranget, JFP 2007)](http://moscova.inria.fr/~maranget/papers/warn/warn.pdf)

### 1.2 Rust's Implementation

Rust implements Maranget's algorithm in the `rustc_pattern_analysis` crate,
with significant engineering for practical performance.

**Key data structures:**

| Type | Purpose |
|---|---|
| `PatStack` | Single row of patterns (pattern-tuple) |
| `Matrix` | 2D matrix of pattern rows with usefulness tracking |
| `MatrixRow` | Row with its patterns plus per-row usefulness state |
| `WitnessStack` | Witness value under construction (built in reverse) |
| `WitnessMatrix` | Collected witnesses for non-exhaustiveness reporting |
| `Constructor` | Enum of all possible constructors (variant, literal, range, slice, etc.) |
| `DeconstructedPat` | A pattern decomposed into constructor + field patterns |
| `PlaceValidity` | Tracks whether a column (place) may contain invalid data |

**Constructor splitting**: For types with infinite or very large value spaces
(integers, slices), the algorithm cannot enumerate all constructors. Instead, it
**splits** constructors into equivalence classes:

Example: Given patterns `0..=100` and `50..=150`, the algorithm identifies five
regions: `0..50`, `50..=100`, `101..=150`, `151..=MAX`. Only these need to be
tested, not every individual integer.

For enums, splitting extracts only the constructors actually present plus a
`Missing` pseudo-constructor representing "everything else." This is implemented
in `ConstructorSet::split()`.

**Relevancy pruning**: Without this optimization, certain match expressions
cause exponential blowup. Consider:

```rust
match tuple {
    (true, _, _, _, ..) => 1,
    (_, true, _, _, ..) => 2,
    (_, _, true, _, ..) => 3,
    // ...n patterns
}
```

Naive specialization explores 2^n combinations. Relevancy pruning observes that
if a row has a wildcard in a column and there are missing constructors, then
non-`Missing` constructors are **irrelevant** for that row's usefulness. This
reduces the complexity to linear.

**Empty types and validity**: The implementation tracks whether a pattern place
might contain invalid data (e.g., through raw pointer dereference or union field
access). A wildcard pattern `_` on an empty type like `Void` is still considered
reachable because the place might contain invalid data, even though no valid
value of type `Void` exists.

**Entry point**:
```rust
pub fn compute_match_usefulness(
    cx: &MatchCtxt<'p, C>,
    arms: &[MatchArm<'p, C>],
    scrut_ty: C::Ty,
) -> UsefulnessReport<'p, C>
```

**NP-completeness**: Computing exhaustiveness is NP-complete (reducible from
SAT). A SAT formula with `n` variables can be encoded as a match with `n`
boolean columns. The optimizations above make this tractable for real-world
patterns.

Source: [Rust Compiler Dev Guide: Exhaustiveness](https://rustc-dev-guide.rust-lang.org/pat-exhaustive-checking.html)
Source: [rustc_pattern_analysis::usefulness](https://doc.rust-lang.org/beta/nightly-rustc/rustc_pattern_analysis/usefulness/index.html)

### 1.3 OCaml's Implementation

OCaml's pattern matching compilation and exhaustiveness checking are closely
related. The compiler uses two key papers:

1. **Maranget 2007** for exhaustiveness warnings
2. **Le Fessant & Maranget 2001** ("Optimizing Pattern Matching") for
   compilation to efficient automata

OCaml's exhaustiveness checker was the original implementation of Maranget's
algorithm. It runs directly on the surface-level pattern AST, providing
witnesses (example values) in warning messages. OCaml turns these warnings into
errors via `-warn-error`.

GADTs complicate exhaustiveness: when matching on a GADT, the type of the
scrutinee carries type-level information that constrains which constructors are
possible. OCaml's checker handles this by tracking type constraints during
specialization, but some edge cases remain where the checker reports false
positives.

Source: [OCaml: How does the compiler check for exhaustive pattern matching?](https://discuss.ocaml.org/t/how-does-the-compiler-check-for-exhaustive-pattern-matching/5013)

### 1.4 GHC's "Lower Your Guards" (2020)

GHC replaced its earlier coverage checker with the "Lower Your Guards" (LYG)
algorithm (Graf, Peyton Jones, Scott, ICFP 2020). The key insight:

**Problem**: GHC's pattern language is extremely complex -- view patterns,
pattern synonyms, pattern guards, overloaded literals, GADTs, etc. Building a
single checker that handles all combinations correctly proved intractable (30+
open bug reports).

**Solution**: Desugar all pattern complexity into a minimal intermediate
language of **guard trees** with only three constructs:

1. `Grd grd tree` -- check a guard, continue with tree if it succeeds
2. `Seq tree1 tree2` -- try tree1, fall through to tree2 on failure
3. `Eps` -- empty tree (no match)

Guards themselves are either `let x = e` (binding) or `x ∈ {K1, K2, ...}`
(constructor membership test).

Coverage checking on guard trees becomes remarkably simple. The algorithm
returns an **annotated tree** decorated with refinement types, indicating which
values reach each point. From this annotation, exhaustiveness witnesses and
redundancy information are extracted.

**Results**: Implementing LYG in GHC fixed over 30 bug reports related to
coverage checking. The approach is both more accurate and more performant than
the previous checker.

Source: [Lower Your Guards (ICFP 2020)](https://dl.acm.org/doi/10.1145/3408989)
Source: [Lower Your Guards (PDF)](https://www.microsoft.com/en-us/research/wp-content/uploads/2020/03/lyg.pdf)

### 1.5 Dart's Space Algebra Extension

Dart extends Maranget's algorithm for OOP with subtyping. The key abstraction
is **spaces** rather than raw patterns:

- **Type space**: Filters by static type
- **Restriction**: Constant values, arities, or open restrictions
- **Properties**: Destructured fields matched against sub-spaces

**Sealed classes** provide the closed world assumption: "No code outside of the
library where the sealed type is declared is allowed to define a new subtype."
This preserves modular compilation while enabling exhaustiveness checking.

**Space intersection** is the core operation, distributing across unions. It is
"approximate and pessimistic" -- it may fail to recognize exhaustiveness in
edge cases but never produces false negatives (never claims exhaustive when it
is not).

Source: [Dart Exhaustiveness](https://github.com/dart-lang/language/blob/main/accepted/3.0/patterns/exhaustiveness.md)

---

## 2. Pattern Compilation Strategies

### 2.1 Decision Trees

Decision trees compile pattern matching into a tree of tests. Each internal node
tests a constructor at some position, branches fan out for each possible
constructor, and leaves are match actions.

**Primary advantage**: Never tests the same sub-term more than once. Each path
from root to leaf tests each position at most once, minimizing runtime overhead.

**Primary drawback**: Potential code size explosion. In the worst case, a
decision tree can be exponentially larger than the source pattern match:

```
match (a, b, c) {
    (true, _, _) => 1,
    (_, true, _) => 2,
    (_, _, true) => 3,
    _ => 4,
}
```

A decision tree for this duplicates the fallback arm at multiple leaves.

**Mitigation**: Implement decision trees as **DAGs with maximal sharing** --
when two subtrees are identical, share a single node. This converts the tree
into a directed acyclic graph, dramatically reducing code size in practice.

### 2.2 Backtracking Automata

Backtracking automata (Augustsson, 1985) compile pattern matching to code that
tests one pattern at a time and backtracks on failure.

**Primary advantage**: Code size is linear in the size of the original match
expression. No duplication of match actions.

**Primary drawback**: May test the same sub-term multiple times during
backtracking. This creates redundant runtime checks.

**Optimization**: Le Fessant & Maranget (2001) introduced optimizations for
backtracking automata that reduce redundant tests while preserving the linear
code size guarantee. OCaml uses this approach.

### 2.3 Maranget's Decision Tree Algorithm (2008)

Published as "Compiling Pattern Matching to Good Decision Trees" (ML Workshop
2008). This algorithm produces decision trees that are practical (not
exponentially large) for real-world patterns.

**Central data structure**: Pattern matrix with three components:

1. **Pattern matrix**: Rows are patterns, columns are positions in the scrutinee
2. **Occurrence vector**: Describes the sequence of extractions needed to access
   each column's value from the scrutinee
3. **Action vector**: Right-hand side expressions for each pattern row

**Three key operations on the pattern matrix:**

**Specialization** `S(c, P)`: Filter rows admitting constructor `c`, decompose
the constructor's fields into new columns.

**Default matrix** `D(P)`: Retain only rows whose selected column is a wildcard,
drop the column.

**Column swapping**: Reorder columns so a column with refutable patterns comes
first.

**Recursive compilation algorithm:**

```
compile(P, occurrences, actions):
  if P is empty: return FAIL
  if first row of P is all wildcards: return action[0]

  pick column i with refutable patterns (heuristic)
  swap column i to first position

  for each constructor c in column 0:
    build specialized matrix S(c, P)
    recursively compile S(c, P)

  build default matrix D(P)
  recursively compile D(P) for the default branch

  emit switch node on column 0 with branches for each c + default
```

**Heuristics for column selection**: The choice of which column to test first
dramatically affects the resulting tree size. Maranget's heuristics are inspired
by **necessity** from lazy pattern matching:

- **Small branching factor**: Prefer columns where fewer distinct constructors
  appear (reduces fan-out at each node)
- **Necessity**: A column is "necessary" if every decision tree for the matrix
  must test it. Necessary columns should be tested early.
- **Scoring heuristics**: Various scoring functions combine arity, branching
  factor, and necessity to pick the best column

**Results**: The paper shows these heuristics produce decision trees competitive
with the optimizing backtracking compiler of Le Fessant & Maranget (2001).

Source: [Compiling Pattern Matching to Good Decision Trees (Maranget, 2008)](http://moscova.inria.fr/~maranget/papers/ml05e-maranget.pdf)
Source: [Colin James: Compiling Pattern Matching](https://compiler.club/compiling-pattern-matching/)

### 2.4 Comparison: Decision Trees vs Backtracking

| Property | Decision Trees | Backtracking Automata |
|---|---|---|
| Tests per sub-term | At most once | Possibly multiple |
| Code size | Potentially exponential | Linear |
| Code size (with DAG) | Usually reasonable | Linear |
| Runtime efficiency | Optimal (no redundant tests) | Near-optimal with optimization |
| Diagnostic quality | Excellent (complete, no dead code) | Poor (hard to analyze) |
| Implementation complexity | Moderate | Moderate |

**Practical convergence**: With DAG sharing for decision trees and redundancy
elimination for backtracking automata, the two approaches produce similar
results in practice. The decision tree approach has a significant edge for
diagnostics because trees are complete and contain no unreachable code.

### 2.5 How Patterns Compile to Bytecode/Native Code

OCaml compiles pattern matches to a series of test-and-branch instructions
using a "static exception" mechanism: each arm has a label, and when a test
fails, the compiler jumps to the next applicable label. The decision tree/DAG
is lowered to these jumps.

Rust compiles pattern matches during MIR lowering. The `match` expression
becomes a sequence of `SwitchInt` terminators (integer/discriminant tests),
`CheckedBinaryOp` for range checks, and `PlaceRef` for field access. Guard
expressions become separate MIR blocks that either proceed to the arm body or
fall through to the next arm.

Python's match/case compiles to sequential `COMPARE_OP` and `MATCH_CLASS`/
`MATCH_MAPPING`/`MATCH_SEQUENCE` bytecodes, following first-to-match semantics
with no decision tree optimization.

Source: [OCaml Compiler Backend](https://dev.realworldocaml.org/compiler-backend.html)
Source: [How to compile pattern matching (Jacobs, 2021)](https://julesjacobs.com/notes/patternmatching/patternmatching.pdf)

---

## 3. Destructuring and Binding

### 3.1 How Bindings Are Extracted

When a pattern matches, the runtime extracts values from the scrutinee based on
the pattern structure:

1. **Constructor selection**: The scrutinee's discriminant (tag/variant) is
   compared against the pattern's constructor.
2. **Field projection**: Once the constructor matches, each field is accessed by
   index (tuple/positional) or by name (record/struct).
3. **Recursive matching**: Sub-patterns are recursively matched against fields.
4. **Binding**: When a variable pattern is reached, the corresponding value is
   bound to the variable name in the current scope.

The order of extraction matters for ownership languages like Rust: moving a
field out of a struct must not use the same field twice across or-pattern
alternatives.

### 3.2 Nested Destructuring

All languages with pattern matching support nesting patterns arbitrarily:

```rust
match msg {
    Message::Response { status: Status::Ok, body: Some(Body { content, .. }) } =>
        process(content),
    _ => handle_error(),
}
```

Nested patterns compile to nested tests in the decision tree. Each level of
nesting adds a column to the pattern matrix during specialization.

### 3.3 Default Values in Patterns

Most ML-family languages do not support default values in patterns. Defaults are
handled at the language level through separate mechanisms:

- **Rust**: No default values in patterns. Use `Option` and handle `None`.
- **Python**: No default values in match patterns. Use guard clauses.
- **Elixir**: Map patterns ignore missing keys. `Map.get(m, :key, default)`.
- **JavaScript** (destructuring only, not match): `let { x = 10 } = obj;`.

### 3.4 Rest Patterns

Rest patterns capture remaining elements:

- **Rust**: `..` in structs/tuples (ignores rest), `rest @ ..` in slices
  (captures rest)
- **Python**: `*rest` in sequences, `**rest` in mappings
- **Elixir**: `[head | tail]` for lists
- **C#**: `..` slice pattern in list patterns, optionally with sub-pattern

---

## 4. Guard Expressions

### 4.1 What Guards Allow

Languages range from fully unrestricted to heavily restricted:

**Unrestricted guards** (Rust, Python, Scala, Swift, OCaml, Haskell): Any
boolean expression can appear in a guard, including function calls, I/O, and
side effects.

**Restricted guards** (Erlang/Elixir): Only a safe subset of expressions is
allowed -- comparison operators, arithmetic, type checks, and specific built-in
functions. No user-defined function calls. No side effects. When a guard raises
an exception, the clause fails silently and the next clause is tried.

The Erlang restriction exists for two reasons: (1) guards must be
side-effect-free to allow reordering optimizations, and (2) the BEAM VM needs
to guarantee guard evaluation terminates.

### 4.2 Guard Evaluation Order

Guards are evaluated **after** the structural pattern matches but **before**
the arm body executes. If the guard fails, matching continues to the next arm.

In languages with unrestricted guards, the evaluation order is strictly
sequential (first-match semantics). The guard for arm `i` is evaluated only
after arm `i`'s structural pattern succeeds and all prior arms have failed.

### 4.3 Guards and Exhaustiveness

Guards fundamentally break exhaustiveness checking because the compiler cannot
reason about arbitrary boolean expressions.

**Approach 1: Ignore guarded arms** (Rust). A guarded arm is treated as
matching nothing for exhaustiveness purposes. The programmer must provide a
wildcard arm after guarded arms.

**Approach 2: Disable checking** (Scala 2, older GHC). The presence of guards
disabled the exhaustiveness checker entirely. Scala 3 and modern GHC no longer
do this.

**Approach 3: Lower to guards** (GHC LYG). The "Lower Your Guards" algorithm
desugars patterns into guard trees, allowing the checker to reason about some
guard expressions (particularly `==` comparisons on literals and constructor
tests) while treating opaque expressions conservatively.

**Approach 4: No checking** (Python, Elixir). Exhaustiveness is not checked,
so guards have no impact on the checker.

---

## 5. Or-Patterns

### 5.1 Binding Consistency

All languages require that each alternative in an or-pattern binds the same set
of variable names with compatible types:

```rust
match value {
    Ok(x) | Err(x) => use(x),  // x bound in both alternatives
}
```

If an alternative binds a name that another does not, this is a compile-time
error.

### 5.2 Exhaustiveness Semantics

Or-patterns follow the **distributive law** for exhaustiveness:

```
c(p | q, ..rest) ≡ c(p, ..rest) | c(q, ..rest)
```

This means an or-pattern `A | B` covers the union of what `A` and `B` cover
individually. The exhaustiveness checker can "expand" or-patterns by
distributing them and checking each alternative.

### 5.3 Or-Pattern Redundancy

The checker can detect redundancy within or-patterns:

```rust
match x {
    Some(_) | Some(0) => {}  // Some(0) is redundant within the or-pattern
    None => {}
}
```

Rust's implementation tracks usefulness per sub-pattern alternative within
or-patterns.

### 5.4 Contexts Where Or-Patterns Are Allowed

- **Rust**: `match` arms, `if let`, `while let`. Not in `let` bindings or
  function parameters (only refutable contexts).
- **OCaml/Haskell**: In `match`/`case` arms. Haskell only recently added
  nested or-patterns via the `OrPatterns` extension.
- **Python**: In `case` clauses only.
- **C#**: Via `or` combinator in `is`/`switch` expressions.

---

## 6. Pattern Matching on Records and Maps

### 6.1 Structural (Partial) Matching

Languages differ on whether record/map patterns must match all fields:

**Exact matching by default** (Rust structs): All fields must be listed unless
`..` is used to ignore the rest.

**Partial matching by default** (Elixir maps, Python mapping patterns):
The pattern specifies a subset of keys; extra keys are ignored.

**Opt-in partial matching** (OCaml records): Use `_` or `..` to suppress
the "some fields not matched" warning.

### 6.2 Rest Patterns for Records/Maps

- **Rust**: `Point { x, .. }` ignores unmatched fields
- **Python**: `{"key": v, **rest}` captures remaining key-value pairs into `rest`
- **Elixir**: Map patterns implicitly ignore extra keys; no rest capture syntax

### 6.3 Nested Record Patterns

C# supports deep property patterns via dot syntax: `segment is { Start.Y: 0 }`,
which is sugar for `segment is { Start: { Y: 0 } }`.

Rust supports nested struct patterns with full depth:
`Struct { field: Inner { x, .. }, .. }`.

---

## 7. Design Trade-offs

### 7.1 Exhaustive vs Non-Exhaustive Matching

**Hard error** (Rust, Swift, Dart): Every match must cover all possible values.
Pros: eliminates an entire class of runtime errors. Cons: requires wildcard
arms even when the programmer "knows" only certain values appear.

**Warning** (OCaml, Scala, C#, Java): Compiler warns but allows non-exhaustive
matches. Can be configured to error. Compromise between safety and convenience.

**No checking** (Python, Elixir, Erlang): Dynamic languages typically do not
check. Exhaustiveness is the programmer's responsibility. Matches a runtime
error (MatchError) on failure.

**Recommendation for lx**: Hard error. Pattern matching is lx's primary control
flow, and agents need guaranteed handling of all message variants. The `?`
operator already implies exhaustive handling.

### 7.2 First-Match vs Best-Match Semantics

**First-match** (all mainstream languages): Arms are tried in source order. The
first matching arm wins. Programmer must order arms from most specific to most
general.

**Best-match** (theoretical, some logic programming): The most specific matching
arm wins regardless of order. More declarative but harder to implement and
reason about -- "most specific" requires a partial order on patterns, and
ambiguity (incomparable patterns) must be handled.

All practical pattern matching languages use first-match semantics. The
first-to-match rule ensures unambiguous selection and allows increasingly general
patterns to serve as fallbacks.

**Recommendation for lx**: First-match. It is simpler to implement, simpler
to reason about, and universal across languages.

### 7.3 Pattern Matching as Expression vs Statement

**Expression** (Rust, Scala, Haskell, OCaml, F#, C# switch expressions):
The match produces a value. All arms must have the same type. Fits naturally
into functional composition.

**Statement** (Python, Swift switch, Java switch statement): The match does not
produce a value. Each arm contains statements.

**Both** (C# switch statement + switch expression, Java switch statement +
switch expression): Different syntax for each use case.

**Recommendation for lx**: Expression. Everything in lx is an expression.

### 7.4 Irrefutable Patterns (let bindings) vs Refutable (match/if-let)

Rust's distinction between irrefutable and refutable patterns is unique and
valuable:

- `let` bindings require irrefutable patterns (always match)
- `match` arms allow refutable patterns
- `if let` / `while let` explicitly handle the refutable-in-conditional case
- `let ... else` handles the "expect it to match, diverge otherwise" case

This classification prevents runtime panics from pattern-match failures in
bindings while keeping the syntax lightweight.

**Recommendation for lx**: lx uses `?` as the pattern matching operator.
Distinguish between irrefutable destructuring (in `let` bindings) and refutable
matching (in `?` expressions). Require exhaustive coverage in `?` expressions.

### 7.5 Nesting Depth and Performance

Patterns can nest arbitrarily deep:

```rust
match expr {
    App(App(Var("map"), f), Cons(x, xs)) => ...
}
```

Each level of nesting adds a column during specialization. The decision tree
grows with the product of constructor arities across levels. In practice, deeply
nested patterns (5+ levels) are rare, and performance is not a concern.

The NP-completeness of exhaustiveness checking manifests only with pathological
patterns (wide tuples of booleans encoding SAT). Real-world patterns are far
from this worst case.

### 7.6 View Patterns and Active Patterns

**Problem**: How do you pattern match on abstract types that hide their
representation?

**Haskell's view patterns**: Apply an arbitrary function before matching.
`(f -> pattern)` matches if `f(value)` matches `pattern`. Fully general but
opaque to the exhaustiveness checker.

**F# active patterns**: Special syntax (`(|Name|)`) that defines custom
decomposers. Four kinds (single-case, multi-case, partial, parameterized)
with varying exhaustiveness properties. Multi-case complete active patterns
are exhaustive; partial patterns are not.

**Scala extractors**: `unapply` method returns `Option[T]`. More general than
F# active patterns but historically broke exhaustiveness checking.

**Trade-off**: Expressiveness vs. exhaustiveness. View patterns and extractors
let you match on abstract types, but the checker cannot reason about custom
decomposition logic. F#'s multi-case active patterns are the best compromise --
they provide custom matching while maintaining exhaustiveness guarantees.

### 7.7 Pattern Matching and the Expression Problem

The **expression problem**: Can you extend both data types and operations
without modifying existing code?

**Pattern matching favors adding operations**: Adding a new function that matches
on all variants is easy. Adding a new variant requires modifying every existing
match expression.

**Visitor pattern favors adding operations**: Same as pattern matching, but with
more boilerplate.

**Subtype polymorphism favors adding data types**: Adding a new subclass is
easy. Adding a new operation requires modifying every existing class.

Pattern matching with sealed types (Rust, Scala, Java) gives the compiler the
ability to flag every match that needs updating when a variant is added. This
turns the expression problem from a silent runtime failure into a compile-time
error list.

**Object algebras** and **tagless final** are approaches that solve both
directions but add implementation complexity.

Source: [The Expression Problem and its solutions (Bendersky)](https://eli.thegreenplace.net/2016/the-expression-problem-and-its-solutions/)
Source: [ADTs and Pattern Matching (okmij.org)](https://okmij.org/ftp/tagless-final/datatypes.html)

---

## 8. Decision Matrix for a New Language

### 8.1 Minimum Viable Pattern Matching

For a language to have useful pattern matching, it needs at minimum:

1. Wildcard pattern `_`
2. Variable binding patterns
3. Literal patterns (integers, strings, booleans)
4. Constructor/variant patterns with fields
5. Tuple/positional destructuring
6. First-match semantics

### 8.2 High-Value Additions

Ordered by implementation-cost to value ratio:

1. **Or-patterns** `p | q` -- low cost, high convenience
2. **Guard expressions** `when condition` -- low cost, essential for real code
3. **Exhaustiveness checking** -- moderate cost, critical for correctness
4. **Redundancy detection** -- comes free with exhaustiveness (same algorithm)
5. **Rest patterns** `..` -- low cost, needed for variadic data
6. **Record/struct patterns** -- moderate cost, needed for complex data
7. **Nested patterns** -- comes free with recursive algorithm design
8. **As-patterns** `p as name` -- low cost, occasionally needed

### 8.3 Advanced Features (Higher Cost)

1. **Slice/list patterns with rest** -- moderate cost, needed for list-heavy code
2. **Range patterns** -- moderate cost (constructor splitting), needed for
   numeric matching
3. **View patterns / active patterns** -- high cost, breaks exhaustiveness,
   needed for abstract types
4. **Pattern synonyms** -- high cost, useful for API design
5. **Binary patterns** (Erlang-style) -- high cost, domain-specific

### 8.4 Implementation Strategy

**Phase 1**: Implement pattern matching compilation using Maranget's decision
tree algorithm. This gives correct pattern matching with reasonable performance.
Start with a simple column-selection heuristic (leftmost refutable column).

**Phase 2**: Implement exhaustiveness checking using Maranget's usefulness
algorithm on the same pattern matrix representation. The specialization
operation is shared between compilation and checking.

**Phase 3**: Add constructor splitting for ranges and large enums. Add relevancy
pruning for tuple-of-booleans patterns.

**Phase 4**: Refine heuristics based on necessity scoring. Add DAG sharing for
code size.

### 8.5 Algorithm Complexity Summary

| Operation | Best Case | Worst Case | Typical |
|---|---|---|---|
| Pattern compilation | O(n) | O(2^n) | O(n * k) |
| Exhaustiveness checking | O(n * k) | O(2^n) (SAT-hard) | O(n * k) |
| Constructor splitting | O(n log n) | O(n log n) | O(n log n) |
| Relevancy pruning | O(1) per row | O(1) per row | O(1) per row |

Where `n` = number of pattern rows, `k` = number of columns (scrutinee width).
The exponential worst cases are pathological and do not arise in practice.
