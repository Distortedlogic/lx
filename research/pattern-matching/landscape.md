# Pattern Matching Across Programming Languages

Research survey covering pattern matching design, syntax, semantics, and pattern
types across seven language families: ML (OCaml/Haskell/F#), Rust, Python,
Elixir/Erlang, Scala, Swift, and C#/Java.

---

## Table of Contents

1. [ML Family (OCaml, Haskell, F#)](#1-ml-family-ocaml-haskell-f)
2. [Rust](#2-rust)
3. [Python](#3-python)
4. [Elixir/Erlang](#4-elixirerlang)
5. [Scala](#5-scala)
6. [Swift](#6-swift)
7. [C# and Java](#7-c-and-java)
8. [Cross-Language Comparison](#8-cross-language-comparison)

---

## 1. ML Family (OCaml, Haskell, F#)

The ML family is the origin of algebraic data types and pattern matching as a
first-class language feature. Pattern matching in ML is tightly coupled to the
type system: you define sum types (variants/unions), and the compiler can verify
that every match is exhaustive and that no clause is redundant.

### 1.1 Core Pattern Types

**Literal patterns**: Match exact values (integers, characters, strings, floats).

**Variable/identifier patterns**: Bind the matched value to a name. Always
irrefutable.

**Wildcard `_`**: Matches anything without binding. Irrefutable.

**Constructor patterns**: Match a specific variant of a sum type with nested
sub-patterns for the constructor's fields. This is the fundamental pattern type
that enables algebraic data type decomposition:

```ocaml
match expr with
| Literal n -> n
| Add (l, r) -> eval l + eval r
| Mul (l, r) -> eval l * eval r
```

**Tuple patterns**: Destructure tuples positionally. Irrefutable when all
sub-patterns are irrefutable.

**Record patterns**: Destructure records by field name. OCaml: `{ field1; field2; _ }`
where `_` ignores remaining fields.

**List patterns**: `[]` matches empty list; `x :: xs` matches head and tail
(cons cell destructuring). Lists are just algebraic types in ML.

### 1.2 Or-Patterns

Or-patterns `p1 | p2` match if either sub-pattern matches. Both sides must bind
the same set of identifiers with the same types.

```ocaml
let is_vowel = function
  | 'a' | 'e' | 'i' | 'o' | 'u' -> true
  | _ -> false
```

### 1.3 As-Patterns

As-patterns bind the entire matched value while also destructuring it:

```ocaml
match list with
| [] | [_] as l -> l
| first :: (second :: _ as tl) -> ...
```

In Haskell the syntax uses `@`: `whole@(x:xs)`.

### 1.4 Guards (when clauses)

A `when` clause adds an arbitrary boolean expression to a pattern:

```ocaml
match x with
| n when n < 0 -> "negative"
| n when n = 0 -> "zero"
| _ -> "positive"
```

Guards break exhaustiveness checking -- the compiler cannot reason about the
boolean expression, so a wildcard fallback is typically required. GHC and OCaml
differ in how strictly they warn about this.

### 1.5 Exhaustiveness Checking

OCaml emits warnings (or errors with `-warn-error`) when pattern matching is
non-exhaustive. The compiler also detects redundant (unreachable) clauses. The
algorithm is based on Maranget's "Warnings for Pattern Matching" (2007).

Haskell (GHC) historically did not warn about inexhaustive matches by default.
The `-Wincomplete-patterns` flag enables warnings. Jane Street's OCaml practice
is to turn non-exhaustive match warnings into hard errors. GHC's newer coverage
checker is based on "Lower Your Guards" (Graf, Peyton Jones, Scott, ICFP 2020).

Source: [Warnings for Pattern Matching (Maranget, 2007)](http://moscova.inria.fr/~maranget/papers/warn/warn.pdf)
Source: [What do Haskellers have against exhaustiveness? (Jane Street)](https://blog.janestreet.com/what-do-haskellers-have-against-exhaustiveness/)

### 1.6 View Patterns (Haskell GHC Extension)

View patterns apply a function before matching the result. Syntax:
`expression -> pattern`. Enabled by `{-# LANGUAGE ViewPatterns #-}`.

```haskell
size (view -> Unit) = 1
size (view -> Arrow t1 t2) = size t1 + size t2
```

The expression can be any function. Variables bound to the left in a pattern are
in scope within view pattern expressions in later arguments. GHC attempts to
collect multiple branches using the same view function into a single case
expression so the view function is only applied once.

Source: [GHC View Patterns](https://ghc.gitlab.haskell.org/ghc/doc/users_guide/exts/view_patterns.html)

### 1.7 Pattern Synonyms (Haskell GHC Extension)

Pattern synonyms define custom patterns that can be used like built-in
constructors. Three forms:

**Unidirectional**: Can only be used for matching, not construction.

**Bidirectional**: Can be used for both matching and construction. All variables
on the right must appear on the left. Wildcards and view patterns are not
allowed.

**Explicitly bidirectional**: Separate definitions for matching and construction,
allowing validation logic in the constructor direction.

Pattern synonyms cannot be defined recursively and can only appear at the top
level of a module.

Source: [GHC Pattern Synonyms](https://ghc.gitlab.haskell.org/ghc/doc/users_guide/exts/pattern_synonyms.html)

### 1.8 F# Active Patterns

F# extends pattern matching with "active patterns" -- functions used as custom
decomposers within match expressions. Four forms:

**Single-case complete**: Always succeeds, transforms the input.
```fsharp
let (|Odd|Even|) n = if n % 2 = 0 then Even else Odd
```

**Multi-case complete**: Must return one of up to seven cases. Cannot accept
additional arguments.
```fsharp
let (|A|B|C|) inp = if inp < 0 then A elif inp = 0 then B else C
```

**Partial**: Returns `option` type. Uses `_` suffix in the "banana clips."
```fsharp
let (|Integer|_|) str =
  match Int32.TryParse(str) with
  | (true, i) -> Some i
  | _ -> None
```

**Parameterized**: Accept additional arguments beyond the matched value. Only
single-case partial patterns can be parameterized.
```fsharp
let (|DivisibleBy|_|) divisor n =
  if n % divisor = 0 then Some DivisibleBy else None
```

Source: [F# Active Patterns (Microsoft)](https://learn.microsoft.com/en-us/dotnet/fsharp/language-reference/active-patterns)
Source: [Extensible Pattern Matching (Syme)](https://www.microsoft.com/en-us/research/wp-content/uploads/2016/02/p29-syme.pdf)

### 1.9 Haskell Or-Patterns (GHC 9.12+)

GHC recently added or-patterns as an extension (`OrPatterns`), bringing
first-class `(pat1; pat2)` syntax to Haskell, where previously or-patterns were
only available at the top level of case alternatives via multiple clauses sharing
the same right-hand side.

Source: [GHC Or-Patterns](https://ghc.gitlab.haskell.org/ghc/doc/users_guide/exts/or_patterns.html)

---

## 2. Rust

Rust has the most comprehensive pattern matching system among systems
programming languages, combining ML-style algebraic matching with ownership
semantics and binding modes.

### 2.1 Match Expressions and Contexts

Patterns appear in multiple contexts:
- `match expr { arms }` -- the primary match expression
- `if let PAT = EXPR { .. }` -- refutable pattern in conditional (RFC 160)
- `while let PAT = EXPR { .. }` -- refutable pattern in loop (RFC 214)
- `let PAT = EXPR;` -- irrefutable pattern in let binding
- `let PAT = EXPR else { diverge };` -- refutable pattern with early return (RFC 3137)
- Function parameters -- irrefutable patterns
- `for PAT in ITER { .. }` -- irrefutable patterns

Source: [RFC 160: if let](https://rust-lang.github.io/rfcs/0160-if-let.html)
Source: [RFC 3137: let else](https://rust-lang.github.io/rfcs/3137-let-else.html)

### 2.2 Refutable vs Irrefutable Patterns

**Irrefutable**: Guaranteed to match any value of the type. Required in `let`
bindings, function parameters, and `for` loops.

**Refutable**: May fail to match. Required in `match` arms (except the last
all-capturing arm), `if let`, `while let`.

This distinction is enforced at compile time and is fundamental to Rust's
pattern system.

### 2.3 Complete Pattern Type Catalog

| Pattern Type | Syntax | Refutability |
|---|---|---|
| Literal | `42`, `"hello"`, `true` | Refutable |
| Identifier | `x`, `ref x`, `mut x`, `x @ pat` | Irrefutable (unless `@` subpat is refutable) |
| Wildcard | `_` | Irrefutable |
| Rest | `..` | Irrefutable |
| Range | `0..=10`, `'a'..='z'` | Refutable (irrefutable if spans entire type) |
| Reference | `&pat`, `&mut pat` | Irrefutable |
| Struct | `Point { x, y, .. }` | Irrefutable for single-variant, refutable for enums |
| Tuple Struct | `Some(x)`, `Variant(a, b)` | Refutable for multi-variant enums |
| Tuple | `(a, b, .., z)` | Depends on sub-patterns |
| Grouped | `(pat)` | Depends on inner pattern |
| Slice | `[a, b, .., z]` | Irrefutable for arrays (if elements irrefutable), refutable for slices |
| Path | `None`, `CONST_NAME` | Irrefutable for single-variant, refutable otherwise |
| Or | `A \| B \| C` | Refutable if any branch is refutable |

Source: [Rust Reference: Patterns](https://doc.rust-lang.org/reference/patterns.html)

### 2.4 Binding Modes and Match Ergonomics

RFC 2005 introduced "match ergonomics" via default binding modes. When a
reference value is matched by a non-reference pattern, the compiler
auto-dereferences and adjusts the binding mode:

- Default mode starts as `move`
- Matching `&T` with a non-reference pattern sets mode to `ref`
- Matching `&mut T` sets mode to `ref mut` (unless already `ref`)
- `ref`/`ref mut` keywords override the default mode

```rust
let x: &Option<i32> = &Some(3);
if let Some(y) = x {
    // y is automatically &i32, not i32
}
```

RFC 3627 (Rust 2024 edition) further refined match ergonomics with additional
reservations to prevent confusing interactions.

Source: [RFC 2005: Match Ergonomics](https://rust-lang.github.io/rfcs/2005-match-ergonomics.html)
Source: [RFC 3627: Match Ergonomics 2024](https://rust-lang.github.io/rfcs/3627-match-ergonomics-2024.html)

### 2.5 Or-Patterns

Or-patterns `A | B | C` have the lowest precedence. All branches must bind the
same set of variables with unifiable types and binding modes. Not allowed in
`let` bindings or function parameters (only refutable contexts).

Exhaustiveness semantics: `c(p | q, ..rest)` is equivalent to
`c(p, ..rest) | c(q, ..rest)` (distributive law).

### 2.6 Constant Patterns and Structural Equality

A path pattern referencing a `const` value matches by structural equality. The
type must implement `PartialEq` via `#[derive(PartialEq)]` -- custom `PartialEq`
implementations are not sufficient. Floats with `NaN` are explicitly rejected.
Constants must be known before monomorphization.

### 2.7 Exhaustiveness Checking

Rust's exhaustiveness checking is based on Maranget's usefulness algorithm,
implemented in the `rustc_pattern_analysis` crate. Computing exhaustiveness is
NP-complete (reducible from SAT), so the implementation uses constructor
splitting and relevancy pruning for tractability. See design-patterns.md for
algorithm details.

Source: [Rust Compiler Dev Guide: Exhaustiveness](https://rustc-dev-guide.rust-lang.org/pat-exhaustive-checking.html)

---

## 3. Python

Python added structural pattern matching in 3.10 (2021) via PEPs 634/635/636.
This was the most controversial Python feature in years, with extensive debate
about syntax choices.

### 3.1 Match Statement

Python's match is a **statement**, not an expression -- consistent with Python's
statement-oriented tradition from Algol. `match` and `case` are soft keywords
(not reserved words), preserving backward compatibility for code using `match`
as a variable name.

```python
match command:
    case "quit":
        quit()
    case "go" as direction:
        go(direction)
    case _:
        unknown()
```

### 3.2 Pattern Types

**Literal patterns**: Integers, strings, booleans, `None`. Singletons (`None`,
`True`, `False`) match by identity (`is`), not equality (`==`). This prevents
surprising behavior where `case True:` would match `1.0`.

**Capture patterns**: Bare names (like `x`) always **bind** the value, never
compare. This is the central design controversy.

**Wildcard `_`**: Matches anything, binds nothing. Chosen over alternatives
(`...`, `*`, `?`) for cross-language familiarity.

**Value patterns**: Dotted names like `Color.RED` or `HttpStatus.OK` are treated
as constants and compared by equality. This was the resolution to the
capture-vs-constant controversy: only dotted names are value patterns; bare
names are always captures.

**Sequence patterns**: `[a, b, *rest]` matches sequences implementing
`collections.abc.Sequence`. Strings and bytes are explicitly excluded despite
being sequences, because "it is in fact often unintuitive and unintended that
strings pass for sequences."

**Mapping patterns**: `{"key": value, **rest}` matches dicts. Extra keys are
permitted unless `**rest` is used. This reflects dictionaries' natural structural
subtyping.

**Class patterns**: `Point(x, y)` uses `__match_args__` class attribute to
specify extraction order. Design principle: "de-construction mirrors the syntax
of construction."

**OR patterns**: `p1 | p2` -- uses `|` rather than `or` to align with regex
syntax and cross-language convention. All alternatives must bind the same set
of variables.

**AS patterns**: `pattern as name` binds the matched value while also matching
the sub-pattern. Chosen over walrus `:=` for consistent left-to-right data flow.

**Guard clauses**: `case pattern if condition:` separates structural matching
from arbitrary boolean constraints. Guards can have side effects but are
evaluated only when the structural pattern matches.

### 3.3 The Capture vs Constant Controversy

The most contentious design decision. When you write `case x:`, does `x` refer
to an existing variable (comparison) or create a new binding (capture)?

**Rejected alternatives:**
- Explicit markers (`?x`, `$x`, `=x`) -- adds syntactic clutter to the most
  common use case
- Uppercase convention -- no Python precedent
- Globals as constants -- would prevent capture patterns at module scope

**Resolution**: Bare names are always captures; dotted names are value lookups.
This means you cannot match against a local variable by name -- you must use a
guard: `case x if x == expected:`.

Source: [PEP 634: Specification](https://peps.python.org/pep-0634/)
Source: [PEP 635: Motivation and Rationale](https://peps.python.org/pep-0635/)
Source: [PEP 636: Tutorial](https://peps.python.org/pep-0636/)

### 3.4 Compilation and Semantics

Python uses **first-to-match** semantics -- patterns are tried sequentially.
There is no compilation to decision trees in CPython's implementation.
Implementations may cache information or reorder sub-pattern checks as an
optimization, but the observable semantics remain sequential.

Python does **not** perform exhaustiveness checking. There is no compiler warning
for missing cases. The `case _:` wildcard is purely optional.

---

## 4. Elixir/Erlang

In Elixir/Erlang, pattern matching is not a control flow extension -- it **is**
the primary binding mechanism. The `=` operator is the match operator, not
assignment.

### 4.1 Pattern Matching as Binding

```elixir
{:ok, result} = {:ok, 42}       # result = 42
[head | tail] = [1, 2, 3]       # head = 1, tail = [2, 3]
%{name: name} = %{name: "lx"}   # name = "lx"
```

Patterns are only allowed on the left side of `=`. The right side follows
regular evaluation semantics. If a pattern fails to match, a `MatchError` is
raised at runtime.

### 4.2 The Pin Operator `^`

Variables in Elixir patterns rebind by default. The pin operator `^` forces
comparison against an existing value rather than rebinding:

```elixir
x = 1
{^x, y} = {1, 2}   # matches, y = 2
{^x, y} = {2, 2}   # MatchError: x is pinned to 1
```

Erlang adopted a similar concept via EEP-0055, adding the `^` operator for
explicitly marking variables as already bound.

Source: [Elixir Patterns and Guards](https://hexdocs.pm/elixir/patterns-and-guards.html)
Source: [Erlang EEP-0055](https://www.erlang.org/eeps/eep-0055)

### 4.3 Pattern Types

**Literals**: Atoms, integers, floats. Integer and float do not cross-match
(`1` does not match `1.0`).

**Tuples**: Must match exact size and element patterns.

**Lists**: `[head | tail]` for cons decomposition. Prefix matching with `++`.

**Maps**: Subset matching -- the pattern needs only a subset of the map's keys.
`%{key: value}` matches any map containing `:key`.

**Structs**: `%Module{field: value}` -- keys validated at compile time.

**Binaries/bitstrings**: Erlang's most distinctive pattern matching feature.

**Strings**: Prefix matching with `<>` operator.

### 4.4 Binary Pattern Matching

Erlang/Elixir can pattern match on individual bits and bytes within binaries.
Each segment specifies type, size, unit, signedness, and endianness:

```erlang
<<Sz:8, Payload:Sz/binary-unit:8, Rest/binary>> = SomeBinary
```

This matches an 8-bit size prefix, then `Sz` bytes of payload, then the
remainder. Variables bound earlier in the pattern can be used as sizes for later
segments.

Segment type specifiers: `integer` (default), `float`, `binary`, `bitstring`,
`bytes`, `bits`, `utf8`, `utf16`, `utf32`. Default sizes: 8 for integer, 64 for
float, all remaining for binary.

Source: [Erlang Bit Syntax](https://www.erlang.org/doc/system/bit_syntax.html)

### 4.5 Function Clause Matching

Pattern matching selects which function clause to execute:

```elixir
def process({:ok, value}), do: value
def process({:error, reason}), do: raise reason
def process(_), do: raise "unexpected"
```

Multiple function clauses are tried in definition order (first-match semantics).
The compiler warns about unreachable clauses.

### 4.6 Guard Expressions

Guards extend pattern matching with boolean conditions via `when`. Guards are
restricted to a safe subset of expressions -- only BIF (built-in function) calls
are allowed. No user-defined functions, no side effects.

Allowed in guards: comparison operators, boolean operators (`and`, `or`, `not`
-- not `&&`, `||`, `!`), arithmetic, type checks (`is_list/1`, `is_number/1`,
etc.), `abs/1`, `hd/1`, `tl/1`, `map_size/1`, `in`/`not in`, `map.field` access,
and `Bitwise` module operations.

When a function raises an exception within a guard, the guard clause fails
silently (does not propagate the error) and the next clause is tried. Custom
guards can be defined with `defguard/1` and `defguardp/1` macros, composed from
the allowed primitives.

### 4.7 Exhaustiveness

Neither Elixir nor Erlang performs exhaustiveness checking. Non-exhaustive
matches raise runtime errors (`MatchError` in Elixir, `{badmatch, Value}` in
Erlang). The dynamic typing makes static exhaustiveness analysis infeasible in
the general case.

---

## 5. Scala

Scala bridges OOP and FP pattern matching through case classes, extractors, and
sealed hierarchies.

### 5.1 Case Classes

Case classes automatically generate `apply` (constructor) and `unapply`
(destructor) methods, making them directly usable in pattern matching:

```scala
case class Person(name: String, age: Int)

person match {
  case Person("Alice", _) => "Found Alice"
  case Person(name, age) if age > 18 => s"$name is an adult"
  case _ => "Someone else"
}
```

### 5.2 Extractors (unapply/unapplySeq)

Any object with an `unapply` method can be used as a pattern. The method returns
`Option[T]` (or `Boolean` for zero-arg patterns). `unapplySeq` enables variadic
patterns.

Scala 3 expanded the extractor protocol: `unapply` and `unapplySeq` can have
leading type clauses and arbitrarily many using clauses. The return type is more
flexible ("option-less pattern matching").

**Critical caveat**: In Scala 2, the presence of a custom extractor with
`Option` return type disabled exhaustiveness checking entirely, even when
concrete values were uncovered. Scala 3 fixed this so that guards and refutable
custom extractors no longer disable the exhaustivity checker.

Source: [Scala 3: Option-less Pattern Matching](https://docs.scala-lang.org/scala3/reference/changed-features/pattern-matching.html)

### 5.3 Sealed Traits/Classes

`sealed` restricts subclassing to the same file, enabling exhaustiveness
checking:

```scala
sealed trait Shape
case class Circle(r: Double) extends Shape
case class Rect(w: Double, h: Double) extends Shape

shape match {
  case Circle(r) => math.Pi * r * r
  case Rect(w, h) => w * h
  // compiler warns if a case is missing
}
```

### 5.4 Match Types (Scala 3)

Match types perform pattern matching at the type level during compilation:

```scala
type Elem[X] = X match {
  case String => Char
  case Array[t] => t
  case Iterable[t] => t
}
```

Similar to Haskell's closed type families. Enables expressing methods with
return types dependent on input types, verified at compile time.

### 5.5 Guards

Guards use `if` after the pattern: `case x if x > 0 => ...`. In Scala 3, guards
no longer disable exhaustiveness checking.

### 5.6 Type Patterns

`case x: Type => ...` matches by runtime type. Combined with sealed hierarchies,
this provides type-safe dispatch without visitor pattern boilerplate.

Source: [Scala Pattern Matching](https://docs.scala-lang.org/tour/pattern-matching.html)

---

## 6. Swift

Swift's pattern matching integrates deeply with the `switch` statement, which is
exhaustive by default for enums.

### 6.1 Pattern Types

**Wildcard `_`**: Matches any value, binds nothing.

**Identifier patterns**: Bind matched value to a constant or variable.

**Value-binding patterns**: `let x` or `var x` within a pattern to bind and
optionally mutate the matched value.

**Tuple patterns**: `(x, y, _)` matches tuples positionally.

**Enumeration case patterns**: `.some(let x)`, `.none` -- match enum variants.

**Optional patterns**: `let x?` is sugar for `.some(let x)`.

**Type-casting patterns**: `is Type` checks type without binding; `let x as Type`
checks and binds.

**Expression patterns**: Any expression implementing the `~=` operator. This
makes `switch` extensible -- you can define custom matching logic by overloading
`~=`.

### 6.2 Where Clauses

Swift uses `where` instead of `when`/`if` for guard conditions:

```swift
switch point {
  case let (x, y) where x == y:
    print("On the diagonal")
  case let (x, y) where x == -y:
    print("On the anti-diagonal")
  default:
    print("Arbitrary point")
}
```

### 6.3 Exhaustiveness

`switch` must be exhaustive for all possible values. For enums, the compiler
checks that all cases are covered. For other types, a `default` case is required.
Adding `@unknown default` handles future enum cases -- the compiler warns if a
known case is missing while still providing a fallback for unknown cases.

### 6.4 Compound Cases

Multiple patterns can share a single body: `case .a, .b, .c:`. Value bindings
in compound cases must bind the same variables with the same types.

Source: [Swift Patterns](https://docs.swift.org/swift-book/documentation/the-swift-programming-language/patterns/)

---

## 7. C# and Java

Both C# and Java have been incrementally adding pattern matching features to
their originally OOP-only designs.

### 7.1 C# Pattern Types

C# has the richest pattern matching among mainstream OOP languages, evolved
across versions 7.0 through 12:

**Declaration pattern**: `expr is Type name` -- runtime type check with binding.

**Type pattern**: `expr is Type` -- type check without binding.

**Constant pattern**: `expr is 42` -- equality check against a constant.

**Relational patterns** (C# 9): `< 0`, `>= 10`, `<= 100` -- compare against
constants.

**Logical patterns** (C# 9): `not null`, `> 0 and < 100`, `1 or 2 or 3`.
Precedence: `not` > `and` > `or`.

**Property pattern**: `expr is { Name: "Alice", Age: > 18 }` -- match against
object properties. Supports nested properties via dot syntax:
`segment is { Start.Y: 0 }`.

**Positional pattern**: `(x, y)` using `Deconstruct` method. Works with tuples,
records, and any type implementing `Deconstruct`.

**Var pattern**: `expr is var x` -- always matches, binds to variable. Matches
`null`.

**Discard pattern**: `_` -- matches anything.

**List patterns** (C# 11): `[1, 2, .., 9]` with slice patterns `..` and nested
sub-patterns within slices: `['a' or 'A', .. var s, 'a' or 'A']`.

**Parenthesized patterns**: `not (float or double)` for precedence control.

All recursive pattern types (logical, property, positional, list) can nest
arbitrarily.

Source: [C# Patterns (Microsoft)](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/operators/patterns)

### 7.2 C# Switch Expressions

```csharp
var result = shape switch
{
    Circle { Radius: > 0 and var r } => Math.PI * r * r,
    Rectangle(var w, var h) when w > 0 && h > 0 => w * h,
    _ => 0
};
```

Switch expressions require exhaustiveness (compiler warning if not). Switch
statements do not. `when` clauses provide guard functionality.

### 7.3 Java Pattern Matching

Java has been adding pattern matching incrementally via Project Amber:

**JEP 394**: Pattern matching for `instanceof` (Java 16, finalized).
```java
if (obj instanceof String s && s.length() > 5) { ... }
```

**JEP 440**: Record patterns (Java 21, finalized). Records automatically provide
deconstruction:
```java
record Point(int x, int y) {}
if (point instanceof Point(var x, var y)) { ... }
```

**JEP 441**: Pattern matching for `switch` (Java 21, finalized). Switch on
arbitrary types with pattern cases:
```java
String result = switch (shape) {
    case Circle c when c.radius() > 0 -> "circle";
    case Rectangle(var w, var h) -> "rect " + w + "x" + h;
    case null -> "null";
    default -> "unknown";
};
```

**Sealed classes**: `sealed interface Shape permits Circle, Rectangle` enables
exhaustiveness checking in switch. The compiler verifies that all permitted
subtypes are covered, removing the need for a `default` clause.

Pattern variables are flow-sensitive -- their scope extends only to code paths
where the pattern is guaranteed to have matched.

Source: [JEP 441: Pattern Matching for switch](https://openjdk.org/jeps/441)
Source: [JEP 440: Record Patterns](https://openjdk.org/jeps/440)

---

## 8. Cross-Language Comparison

### 8.1 Pattern Type Coverage Matrix

| Feature | OCaml | Haskell | F# | Rust | Python | Elixir | Scala | Swift | C# | Java |
|---|---|---|---|---|---|---|---|---|---|---|
| Literal | Y | Y | Y | Y | Y | Y | Y | Y | Y | Y |
| Variable/bind | Y | Y | Y | Y | Y | Y | Y | Y | Y | Y |
| Wildcard | Y | Y | Y | Y | Y | Y | Y | Y | Y | N |
| Constructor/variant | Y | Y | Y | Y | N* | N | Y | Y | N | N |
| Tuple | Y | Y | Y | Y | Y | Y | Y | Y | Y | N |
| Record/struct | Y | Y | Y | Y | N | Y | N | N | Y | Y |
| List/sequence | Y | Y | Y | Y | Y | Y | Y | N | Y | N |
| Map/dict | N | N | N | N | Y | Y | N | N | N | N |
| Or-pattern | Y | Y* | Y | Y | Y | N | Y | Y | Y | N |
| As-pattern | Y | Y | Y | Y | Y | N | Y | N | N | N |
| Guard | Y | Y | Y | Y | Y | Y | Y | Y | Y | Y |
| View/active | N | Y* | Y | N | N | N | Y** | Y*** | N | N |
| Range | N | N | N | Y | N | N | N | Y | Y | N |
| Binary/bit | N | N | N | N | N | Y | N | N | N | N |
| Type pattern | N | N | Y | N | N | N | Y | Y | Y | Y |
| Slice/rest | N | N | N | Y | Y | Y | Y | N | Y | N |
| Exhaustiveness | Y | Y* | Y | Y | N | N | Y | Y | W | W |

`*` = via extension. `**` = via extractors. `***` = via expression pattern `~=`.
`W` = warning only. `N*` = Python uses class patterns instead.

### 8.2 Exhaustiveness Model Comparison

| Language | Model | Error Level | Guard Impact |
|---|---|---|---|
| OCaml | Maranget usefulness | Warning (configurable to error) | Disables for guarded arms |
| Haskell | Lower Your Guards | Warning (opt-in) | Partial analysis |
| Rust | Maranget usefulness | Hard error | Guarded arms counted as covering nothing |
| Scala | Sealed hierarchy | Warning | No longer disables (Scala 3) |
| Swift | Enum + type coverage | Hard error | Guards require default |
| C# | Type + value coverage | Warning | N/A (when clause) |
| Java | Sealed + enum | Warning | Guarded cases not counted |
| Python | None | N/A | N/A |
| Elixir/Erlang | None | N/A | N/A |
| Dart | Extended Maranget (spaces) | Hard error | N/A |

### 8.3 Key Syntactic Differences

**Match keyword**: `match` (Rust, Python, Scala), `case` (Haskell, C#),
`switch` (Swift, Java, C#), `=`/`case`/`fn` (Elixir), implicit in function
heads (Erlang, OCaml, Haskell).

**Guard syntax**: `when` (OCaml, Elixir, C#, Java), `if` (Rust, Scala, Python),
`where` (Swift), `|` guards (Haskell).

**Or-pattern**: `|` (most languages), `or` (C#).

**Wildcard**: `_` (universally), with `..` for rest/slice patterns (Rust, C#,
Python).

**Pin/compare**: `^` (Elixir), dotted names (Python), `const` paths (Rust).
