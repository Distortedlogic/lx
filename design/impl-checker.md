# Type Checker Design

Bidirectional type inference with structural subtyping. Operates on the AST, annotates it with type information, and produces diagnostics.

Implements: [types.md](../spec/types.md), [errors.md](../spec/errors.md), [diagnostics.md](../spec/diagnostics.md)

## Overview

The checker is optional at runtime (`lx run` can skip it) but required for `lx check`. It performs:

1. Type inference (bidirectional: propagate down from annotations, synthesize up from usage)
2. Exhaustiveness checking for pattern matches
3. Error type compatibility validation for `^` sites
4. Mutable capture detection in concurrent contexts
5. Truthiness violations (non-Bool in conditional position)
6. Variant name uniqueness within modules
7. Import conflict detection

## Type Representation

```rust
enum Type {
    Int,
    Float,
    Bool,
    Str,
    Unit,
    Bytes,

    List(Box<Type>),
    Set(Box<Type>),
    Map { key: Box<Type>, value: Box<Type> },
    Record(Vec<(String, Type)>),
    Tuple(Vec<Type>),

    Func { param: Box<Type>, ret: Box<Type> },
    Result { ok: Box<Type>, err: Box<Type> },
    Maybe(Box<Type>),

    Union { name: String, variants: Vec<Variant> },

    Var(TypeVarId),
    Unknown,
}

struct Variant {
    name: String,
    fields: Vec<Type>,
}

type TypeVarId = u32;
```

`Var(id)` represents an unresolved type variable. The inference engine fills these in via unification. `Unknown` is the error-recovery type — it unifies with anything and suppresses cascading errors.

## Bidirectional Inference

Two modes:

**Checking** (`check(expr, expected_type)`): push a known type down into an expression. Used when the context knows the type (annotation, function parameter type, match arm body).

**Synthesis** (`synth(expr) -> Type`): compute the type from the expression itself. Used when no expected type is available.

### Synthesis Rules

```
Literal:
  42          → Int
  3.14        → Float
  "hello"     → Str
  true/false  → Bool
  ()          → Unit

Binary:
  Int + Int   → Int
  Float + Float → Float
  Int + Float → Float (widening)
  Str ++ Str  → Str
  a == b      → Bool (requires a and b unify)
  a && b      → Bool (requires both Bool)

Application:
  f : (a -> b), x : a  → b
  f : (a -> b -> c), x : a  → (b -> c)  (currying)

Pipe:
  x : a, f : (... -> a -> b)  → b  (insert x as last arg)

Propagate (^):
  expr : Result a e  → a  (marks function as fallible with e)
  expr : Maybe a     → a  (marks function as fallible with NoneErr)
  expr : other       → error[type]: ^ requires Result or Maybe

Coalesce (??):
  expr : Result a e, default : a  → a
  expr : Maybe a, default : a     → a
```

### Checking Rules

```
Function body:
  if return type annotated → check body against return type
  if return type has ^     → wrap body in implicit Ok if needed

Match arms:
  all arms must produce same type
  if checking, push expected type into each arm body

List literal:
  if checking against [T], check each element against T
  if synthesizing, unify all element types
```

## Structural Subtyping

Record types use width subtyping: `{x: Int, y: Int, z: Int}` satisfies `{x: Int, y: Int}`. The checker verifies that all required fields are present with compatible types. Extra fields are ignored.

Function subtyping is contravariant in parameters, covariant in returns:
- `(Animal -> Int)` is a subtype of `(Cat -> Int)` (accepts wider input)
- `(Int -> Cat)` is a subtype of `(Int -> Animal)` (produces narrower output)

## Unification

Type variables are resolved via unification. When two types must be equal:

```
unify(Var(a), T) → bind a = T (occurs check: a must not appear in T)
unify(T, Var(a)) → bind a = T
unify(List(a), List(b)) → unify(a, b)
unify(Record(fs1), Record(fs2)) → unify each matching field
unify(Func(a1, r1), Func(a2, r2)) → unify(a1, a2), unify(r1, r2)
unify(Int, Float) → Float (widening, produces a coercion node)
unify(T, T) → ok
unify(T1, T2) → error[type]: expected T1, got T2
```

The unification table is a `Vec<Option<Type>>` indexed by `TypeVarId`. Lookups chase the table to find the concrete type.

## Exhaustiveness Checking

For match expressions on tagged unions, the checker verifies all variants are covered. Algorithm:

1. Collect all variant names from the union type
2. For each match arm, determine which variants it covers
3. `_` covers all remaining variants
4. If uncovered variants exist, emit `warning[match]: non-exhaustive`

For Bool: both `true` and `false` must be covered, or `_` present. For other types (Int, Str): `_` is required (infinitely many values).

Nested pattern exhaustiveness uses the standard algorithm from "Warnings for Pattern Matching" (Maranget 2007).

## Mutable Capture Detection

When entering a `par`, `sel`, or `pmap` body, the checker records the set of mutable bindings in scope. If any are referenced inside the concurrent body, emit `error[concurrency]`.

Mutable bindings *defined inside* the concurrent body are fine — they're local to each task.

## Implicit Err Early Return

In functions with `-> T ^ E` annotation, the checker validates that intermediate expression statements that could produce `Err` values have error types compatible with `E`. The checker marks such functions so the interpreter knows to check for Err short-circuit during evaluation.

## Error Type Propagation

Each function tracks its error type. When `^` is used:

1. If the function has a declared error type (`-> T ^ E`), the `^` site's error must be compatible with `E`. If not directly equal, check if `E` is a tagged union containing the error type as a variant, and insert automatic wrapping.

2. If the function has no error annotation, the error type is `Dynamic` — all errors propagate as-is.

## Per-Function Checking

Each function is checked independently. No global constraint solving (not Hindley-Milner). Polymorphic functions are instantiated with fresh type variables at each call site.

This keeps error messages local: "expected Int, got Str at line 15" rather than "constraint from line 15 conflicts with constraint from line 42."

## Forward References

Top-level bindings are processed in two passes:
1. **Signature pass**: collect names and any type annotations
2. **Body pass**: check function bodies with all top-level names in scope

This enables mutual recursion between top-level functions.

Within blocks, bindings are sequential: only bindings above the current line are in scope.

## Output

The checker produces:
- `TypeInfo`: a side table mapping `Span → Type` for every expression
- `Vec<Diagnostic>`: errors and warnings

The interpreter can use `TypeInfo` for runtime optimizations (e.g., skip type-checking coercions that the checker already verified). Or it can ignore `TypeInfo` entirely and run dynamically.

## Cross-References

- AST input: [impl-ast.md](impl-ast.md)
- Type spec: [types.md](../spec/types.md)
- Error handling spec: [errors.md](../spec/errors.md)
- Diagnostics format: [diagnostics.md](../spec/diagnostics.md)
- Interpreter consumer: [impl-interpreter.md](impl-interpreter.md)
