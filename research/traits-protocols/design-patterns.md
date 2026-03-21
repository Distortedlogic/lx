# Design Patterns in Trait, Protocol, and Interface Systems

Cross-cutting design patterns that recur across languages, with analysis of trade-offs and implementation strategies relevant to lx's trait system design.

---

## 1. Nominal vs Structural Typing

The fundamental axis along which trait/protocol/interface systems differ.

### Nominal Systems

**Approach:** A type satisfies a trait/interface only through explicit declaration (`impl Trait for Type`, `class Foo implements Bar`).

**Languages:** Rust, Haskell, Java, Swift, Scala, Elixir

**Properties:**
- Prevents accidental conformance — two types with identical method sets are not interchangeable unless explicitly declared
- Enables the compiler to provide clear error messages referencing the trait name
- Requires boilerplate: every conformance relationship must be written out
- The orphan rule (Rust) / instance coherence (Haskell) is a consequence — nominal systems need global uniqueness guarantees for implementations

**Example (Rust):**
```rust
trait Drawable { fn draw(&self); }
struct Circle;
impl Drawable for Circle {  // explicit declaration required
    fn draw(&self) { ... }
}
```

### Structural Systems

**Approach:** A type satisfies an interface if it has all required members with compatible signatures. No declaration needed.

**Languages:** Go, TypeScript, Python (Protocol)

**Properties:**
- Zero coupling between interface definition and implementation — types from unrelated packages can satisfy the same interface without coordination
- Natural fit for duck typing and dynamic languages
- Risk of accidental conformance — a type might structurally match an interface by coincidence
- Third-party code works seamlessly: import a library type and use it where your interface is expected

**Example (Go):**
```go
type Stringer interface { String() string }
// Any type with String() string satisfies Stringer — no declaration
```

### Hybrid Approaches

Several languages blend both:
- **TypeScript**: primarily structural, but classes can use `implements` for explicit checking
- **Python**: nominal (ABC) and structural (Protocol) coexist, chosen per use case
- **Swift**: nominal conformance, but `any Protocol` existential types add runtime flexibility
- **Scala**: nominal trait implementation, but structural types exist as a lesser-used feature

### Trade-offs for Language Design

| Dimension | Nominal | Structural |
|-----------|---------|------------|
| Safety | Prevents accidental conformance | Permits accidental conformance |
| Boilerplate | Requires explicit impl blocks | Zero impl boilerplate |
| Decoupling | Coupled (must know the trait) | Decoupled (shape-only) |
| Error messages | Clear ("X doesn't implement Y") | Can be confusing ("missing method Z") |
| Retroactive conformance | Requires orphan rule workarounds | Automatic |
| Coherence | Enforceable (one impl per type) | Not enforceable |

**Design decision for lx:** lx uses nominal traits (explicit `impl` blocks) for protocol contracts between agents, which provides clear error messages and prevents accidental protocol satisfaction. This is the right choice for an agent orchestration language where accidental conformance could cause an agent to be dispatched incorrectly.

**Sources:**
- [Medium: Nominal vs Structural Typing Explained](https://medium.com/@thejameskyle/type-systems-structural-vs-nominal-typing-explained-56511dd969f4)
- [Wikipedia: Nominal Type System](https://en.wikipedia.org/wiki/Nominal_type_system)
- [arXiv: Why Nominal-Typing Matters](https://arxiv.org/pdf/1606.03809)
- [Leptonic: Nominal vs Structural Types in Rust](https://leptonic.solutions/blog/nominal-vs-structural-types/)

---

## 2. Default Methods

Providing concrete implementations in traits/interfaces that implementors can inherit or override.

### When to Provide Defaults

**Good defaults:**
- Derived from other trait methods (e.g., `!=` defined via `==`)
- Convenience methods that compose required methods (`Iterator::map`, `Iterator::filter` derived from `next`)
- Backwards-compatible interface evolution (Java 8 `default` methods)

**Bad defaults:**
- Defaults that most implementations will override (misleading — better to keep abstract)
- Defaults with complex logic that obscure the trait's contract
- Defaults that depend on implementation-specific invariants

### Default-Override Interaction

**Rust:**
```rust
trait Greet {
    fn name(&self) -> &str;
    fn greet(&self) -> String {
        format!("Hello, {}!", self.name())
    }
}
```
Default `greet` calls abstract `name`. Implementors must provide `name` but get `greet` for free. They can override `greet` if needed.

**Haskell — Minimal complete definition:**
```haskell
class Eq a where
    (==) :: a -> a -> Bool
    (/=) :: a -> a -> Bool
    x == y = not (x /= y)
    x /= y = not (x == y)
    {-# MINIMAL (==) | (/=) #-}
```
Both methods have defaults in terms of each other. `MINIMAL` pragma documents that implementing either one suffices.

**Swift — Protocol extensions:**
```swift
protocol Drawable {
    func draw()
}
extension Drawable {
    func draw() { print("Default drawing") }
}
```
Default implementations live in extensions, not the protocol definition itself. This creates a subtlety: if a method is declared in the extension but not in the protocol, it is statically dispatched (not polymorphic).

### The Fragile Base Class Problem

Adding or changing a default method can silently alter behavior for all implementors who rely on the default. This is less severe in trait systems than in class hierarchies because:
- Traits typically have fewer methods than base classes
- Traits cannot hold mutable state (in most languages)
- Adding a new required method is a compile error, forcing review

Java experienced this when adding `default` methods to interfaces — existing implementations suddenly gained new methods they never asked for. Sealed interfaces (Java 17) partially mitigate this by restricting who can implement.

### Diamond Problem with Defaults

When a type implements two traits that both provide a default for the same method:

| Language | Resolution |
|----------|------------|
| Rust | Compile error — must disambiguate with `<Type as Trait>::method()` |
| Java | Compile error — must override and choose via `InterfaceA.super.method()` |
| Scala | Linearization — rightmost trait wins, `super` chains through linearization order |
| Swift | Compile error if protocol declarations conflict; extension defaults don't conflict because they're not protocol requirements |
| Python | C3 linearization MRO |

**Sources:**
- [Rust Book: Default Implementations](https://doc.rust-lang.org/book/ch10-02-traits.html)
- [Sling Academy: Default Methods in Rust Traits](https://www.slingacademy.com/article/default-methods-in-rust-traits-streamlining-common-implementations/)
- [GHC User Guide: MINIMAL Pragma](https://ghc.gitlab.haskell.org/ghc/doc/users_guide/exts/deriving_strategies.html)

---

## 3. Trait Composition and Linearization

How languages resolve conflicts when multiple traits/interfaces define the same method.

### No Linearization (Explicit Disambiguation)

**Rust** and **Java** refuse to guess. If two traits define the same method name:

```rust
trait Pilot { fn fly(&self); }
trait Wizard { fn fly(&self); }
struct Human;
impl Pilot for Human { fn fly(&self) { println!("Captain speaking"); } }
impl Wizard for Human { fn fly(&self) { println!("Up!"); } }
impl Human { fn fly(&self) { println!("*waving arms*"); } }

let h = Human;
h.fly();              // inherent method: *waving arms*
Pilot::fly(&h);       // Captain speaking
Wizard::fly(&h);      // Up!
```

Fully qualified syntax: `<Human as Pilot>::fly(&h)`.

### C3 Linearization

**Python** and **Scala** use linearization algorithms to produce a single method resolution order (MRO).

**C3 Algorithm (Python):**
1. Start with the class itself
2. Merge the linearizations of parent classes with the list of parents
3. At each step, select a class that does not appear in the tail of any remaining list
4. If no such class exists, raise an error (inconsistent hierarchy)

For `class D(B, C)` where both `B` and `C` extend `A`:
```
L[D] = D + merge(L[B], L[C], [B, C])
     = D + merge([B, A, object], [C, A, object], [B, C])
     = D, B, C, A, object
```

**Key guarantees:**
- Local precedence: left-to-right parent order is preserved
- Monotonicity: if `X` precedes `Y` in `L[C]`, then `X` precedes `Y` in `L[D]` for any `D` derived from `C`

**Scala Linearization:**
Scala processes mixins right-to-left, removing duplicates (keeping the rightmost occurrence). For `class D extends B with C`:
```
L[D] = D, C, B, A, AnyRef, Any
```

The rightmost trait in the `with`/`,` list has highest priority.

### Stackable Modifications (Scala)

Scala's `abstract override` enables chaining through linearization:

```scala
trait Base { def process(s: String): String }
trait Upper extends Base {
    abstract override def process(s: String) = super.process(s.toUpperCase)
}
trait Trim extends Base {
    abstract override def process(s: String) = super.process(s.trim)
}

class Pipeline extends ConcreteBase with Upper with Trim
// Trim.process -> Upper.process -> ConcreteBase.process
```

Each `super.process()` call follows the linearization chain. This is unique to Scala — no other mainstream language supports this pattern directly.

### Interface Composition (No Conflict)

**Go** and **Swift** avoid the problem through different mechanisms:
- **Go**: embedding composes interfaces. If two embedded interfaces have the same method, the outer interface requires only one implementation.
- **Swift**: protocol composition (`A & B`) requires conformance to both. If methods conflict, the conforming type must provide a single implementation that satisfies both.

**Sources:**
- [Python.org: C3 Method Resolution Order](https://www.python.org/download/releases/2.3/mro/)
- [Rust Book: Disambiguating Methods](https://doc.rust-lang.org/book/ch20-02-advanced-traits.html)
- [GeeksforGeeks: Python MRO](https://www.geeksforgeeks.org/python/method-resolution-order-in-python-inheritance/)
- [GeeksforGeeks: Scala Trait Linearization](https://www.geeksforgeeks.org/trait-linearization-in-scala/)
- [Artima: Scala's Stackable Trait Pattern](https://www.artima.com/articles/scalas-stackable-trait-pattern)

---

## 4. Associated Types vs Type Parameters

Two approaches to connecting types within a trait/typeclass.

### Associated Types

The trait declares a type placeholder; each implementation specifies one concrete type:

```rust
trait Iterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
}
impl Iterator for Counter {
    type Item = u32;
    fn next(&mut self) -> Option<u32> { ... }
}
```

**Properties:**
- Each type can have at most one implementation (no `Iterator<u32>` and `Iterator<String>` for the same type)
- Cleaner call sites: no type annotation needed (`counter.next()` vs `counter.next::<u32>()`)
- Better for "output" types — the type is determined by the implementation, not the caller

### Type Parameters

The trait is generic over the type:

```rust
trait Add<Rhs = Self> {
    type Output;
    fn add(self, rhs: Rhs) -> Self::Output;
}
```

**Properties:**
- A type can implement the trait multiple times with different parameters
- Better for "input" types — the caller chooses which variant
- Requires annotation when ambiguous

### Decision Framework

| Criterion | Associated Type | Type Parameter |
|-----------|----------------|----------------|
| Implementations per type | Exactly one | Multiple |
| Call-site annotations | None needed | Often needed |
| Use case | "Output" types (Iterator::Item) | "Input" types (Add<Rhs>) |
| Readability | Cleaner bounds | More verbose bounds |

**Haskell parallel:** Associated types in Haskell are type families within a typeclass:
```haskell
class Container c where
    type Element c
    empty :: c
    insert :: Element c -> c -> c
```

**Swift parallel:** `associatedtype` in protocols:
```swift
protocol Container {
    associatedtype Item
    mutating func append(_ item: Item)
}
```

**Sources:**
- [Rust Book: Associated Types](https://doc.rust-lang.org/book/ch20-02-advanced-traits.html)
- [HaskellWiki: Type Families](https://wiki.haskell.org/GHC/Type_families)

---

## 5. Dynamic Dispatch

Runtime polymorphism through indirection — vtables, witness tables, existential containers.

### Vtable-Based Dispatch (Rust, C++)

A trait object (`dyn Trait`) is a fat pointer: `(data_ptr, vtable_ptr)`.

The vtable is a static struct containing:
- Function pointers for each trait method
- Type size and alignment
- Destructor pointer

```
dyn Drawable (16 bytes on 64-bit):
  ┌──────────┬──────────┐
  │ data_ptr │ vtbl_ptr │
  └──────────┴──────────┘
                    │
                    ▼
              ┌──────────┐
              │ drop_ptr │
              │ size     │
              │ align    │
              │ draw_ptr │
              │ bounds_ptr│
              └──────────┘
```

**Performance costs:**
- Indirect call: load vtable pointer, load function pointer, call through pointer (vs direct call for monomorphized generics)
- No inlining: the compiler cannot inline through a vtable call
- Cache pressure: vtable and data may be in different cache lines
- Heap allocation: `dyn Trait` is unsized, typically requiring `Box<dyn Trait>` or `&dyn Trait`

### Monomorphization (Rust, C++, Haskell with specialization)

The compiler generates a specialized copy of generic code for each concrete type used:

```rust
fn draw<T: Drawable>(item: &T) { item.draw(); }
// Generates: draw_circle, draw_rect, draw_triangle, ...
```

**Trade-offs:**
- Zero runtime overhead: direct calls, inlinable
- Binary bloat: N concrete types = N copies of the function
- Longer compile times: more code to generate and optimize
- Cannot be used when the concrete type is not known at compile time

### Existential Containers (Swift)

Swift's `any Protocol` uses an existential container — a fixed-size box (typically 3 words for inline storage + metadata pointers). Small types are stored inline; larger types are heap-allocated.

```
any Drawable:
  ┌─────────────────────┐
  │ inline storage (3w) │  ← small values stored here directly
  │ metadata_ptr        │  ← points to type metadata
  │ witness_table_ptr   │  ← points to protocol witness table
  └─────────────────────┘
```

### Dictionary Passing (Haskell)

Haskell implements typeclasses by passing "dictionaries" — records of function implementations — as implicit arguments:

```haskell
-- Conceptually:
sort :: Ord a => [a] -> [a]
-- Becomes:
sort :: OrdDict a -> [a] -> [a]
```

GHC can specialize (monomorphize) when the concrete type is known, or pass dictionaries when it is not. This is a compile-time decision.

### Enum Dispatch (Rust Pattern)

When the set of types is closed, an enum avoids vtable overhead:

```rust
enum Shape {
    Circle(Circle),
    Rect(Rect),
}
impl Shape {
    fn draw(&self) {
        match self {
            Shape::Circle(c) => c.draw(),
            Shape::Rect(r) => r.draw(),
        }
    }
}
```

Stack-allocated, branch-predicted, inlinable. The `enum_dispatch` crate automates this pattern.

**Sources:**
- [EventHelix: Rust Trait Objects VTables](https://www.eventhelix.com/rust/rust-to-assembly-tail-call-via-vtable-and-box-trait-free/)
- [SoftwareMill: Rust Static vs Dynamic Dispatch](https://softwaremill.com/rust-static-vs-dynamic-dispatch/)
- [Rust Book: Trait Objects](https://doc.rust-lang.org/book/ch18-02-trait-objects.html)

---

## 6. Extension Methods and Retroactive Conformance

Adding implementations to types you do not own.

### Rust: Orphan Rule + Workarounds

The orphan rule requires either the trait or the type to be local. Workarounds:

1. **Newtype pattern**: wrap the foreign type in a local struct
   ```rust
   struct MyVec(Vec<String>);
   impl Display for MyVec { ... }
   ```
   Downside: must manually delegate or `Deref` to expose inner methods.

2. **Extension traits**: define a new trait with the desired methods and blanket-implement it
   ```rust
   trait IteratorExt: Iterator {
       fn my_method(&self) { ... }
   }
   impl<T: Iterator> IteratorExt for T {}
   ```
   Convention: `FooExt` suffix (RFC 445).

3. **Sealed + extension**: combine sealed traits with extension traits for controlled extensibility.

### Swift: Retroactive Conformance via Extensions

```swift
extension Array: CustomStringConvertible where Element: CustomStringConvertible {
    var description: String { ... }
}
```

Swift allows adding protocol conformance to any type via extensions, including types from other modules. This is true retroactive conformance — no wrappers needed.

**Risk:** Two libraries could independently conform the same type to the same protocol with different implementations. Swift does not prevent this, and the behavior is undefined.

### Go: No Retroactive Conformance Needed

Because Go interfaces are structurally satisfied, any type that has the right methods already conforms. You cannot add methods to foreign types, but you can define new interfaces that existing types already satisfy.

### Clojure: Full Retroactive Conformance

`extend-type` and `extend-protocol` add protocol implementations to any type, including Java classes and Clojure built-in types, at any time:

```clojure
(extend-type java.lang.String
  Drawable
  (draw [this] (println this)))
```

No restrictions — but the Clojure community convention is to only extend types you own or protocols you own.

### Elixir: Full Retroactive Conformance

`defimpl` can implement any protocol for any type at any time:

```elixir
defimpl Size, for: BitString do
  def size(string), do: byte_size(string)
end
```

Protocol consolidation at compile time resolves all implementations.

**Sources:**
- [Rust Book: Newtype Pattern](https://doc.rust-lang.org/book/ch20-02-advanced-traits.html)
- [RFC 445: Extension Trait Conventions](https://rust-lang.github.io/rfcs/0445-extension-trait-conventions.html)
- [Rust API Guidelines: Future Proofing](https://rust-lang.github.io/api-guidelines/future-proofing.html)
- [Swift by Sundell: Conditional Conformances](https://www.swiftbysundell.com/articles/conditional-conformances-in-swift/)

---

## 7. Trait Objects and Object Safety

What makes a trait usable as a dynamically-dispatched type, and how languages handle the restrictions.

### Rust Dyn Safety Rules

A trait is dyn-compatible (object-safe) if:

1. **No `Self: Sized` supertrait** — `dyn Trait` is unsized, so requiring `Sized` is contradictory
2. **No associated constants** — constants need a concrete type to resolve
3. **No generic associated types** (with generics) — vtable cannot represent infinite type combinations
4. **Methods must be dispatchable** or explicitly excluded:
   - No type parameters on methods (would require infinite vtable entries)
   - `Self` appears only as the receiver (`&self`, `&mut self`, `self: Box<Self>`)
   - No `async fn` (hidden `Future` type)
   - No `-> impl Trait` return types (erased type unknown at call site)

**Workaround — exclude methods from the vtable:**
```rust
trait MyTrait {
    fn dispatch_me(&self);
    fn not_for_dyn(&self) where Self: Sized;  // excluded from dyn MyTrait
}
```

**Workaround — type erasure for return types:**
```rust
trait MyTrait {
    fn items(&self) -> Box<dyn Iterator<Item = i32>>;  // instead of -> impl Iterator
}
```

### Swift Existential Restrictions

Swift had similar restrictions on protocols with associated types — they could not be used as existential types (`any Protocol`) until Swift 5.7 introduced constrained existential types:

```swift
func process(_ items: any Collection<Int>) { ... }
```

Before this, `any Collection` was unusable because the compiler could not determine `Element`.

### Haskell Existential Types

Haskell can box typeclass-constrained values using existential quantification:

```haskell
data SomeDrawable = forall a. Drawable a => MkDrawable a
```

This erases the concrete type while preserving the typeclass dictionary. Similar to `Box<dyn Trait>` in Rust.

### Go: No Restrictions

Go interfaces have no object-safety concept because they have no associated types, no generics on methods (prior to Go 1.18), and no `Self` type. Every interface can be used as a value type.

**Sources:**
- [RFC 255: Object Safety](https://rust-lang.github.io/rfcs/0255-object-safety.html)
- [Rust Reference: Trait Object Types](https://doc.rust-lang.org/reference/types/trait-object.html)
- [Learning Rust: Dyn Safety](https://quinedot.github.io/rust-learning/dyn-safety.html)
- [Rust Error Index: E0038](https://doc.rust-lang.org/error_codes/E0038.html)

---

## 8. Protocol-Oriented Programming

Swift's paradigm of preferring protocols and protocol extensions over class inheritance.

### Core Principles

1. **Start with a protocol, not a class**: define behavior as protocol requirements
2. **Provide shared behavior via protocol extensions**: default implementations in extensions, not base classes
3. **Compose via protocol composition** (`A & B`): instead of deep inheritance hierarchies
4. **Use value types** (structs/enums) with protocol conformance instead of reference types (classes)

### Comparison to Class Hierarchies

| Aspect | Inheritance | POP |
|--------|-------------|-----|
| Reuse mechanism | Base class | Protocol extension |
| State sharing | Inherited fields | Composition |
| Type flexibility | Single superclass | Multiple protocols |
| Value/reference | Reference only | Both |
| Retroactive | Cannot re-parent | Can add conformance |

### Application to lx

lx's agent protocols are inherently protocol-oriented: agents declare which protocols they satisfy (message handling, tool invocation, state management) and compose capabilities. Protocol extensions could provide default message routing, retry logic, or logging that agents inherit without explicit implementation.

**Sources:**
- [WWDC 2015: Protocol-Oriented Programming in Swift](https://developer.apple.com/videos/play/wwdc2015/408/)
- [Swift by Sundell: Protocol-Oriented Programming](https://www.swiftbysundell.com/articles/referencing-generic-protocols-with-some-and-any-keywords/)

---

## 9. The Expression Problem

The challenge of simultaneously extending a system with new types AND new operations without modifying existing code.

### Problem Statement

Philip Wadler (1998): define a data abstraction that is extensible both in its representations and its behaviors, without recompiling existing code, while retaining static type safety.

### The Fundamental Tension

| Paradigm | Add new types | Add new operations |
|----------|--------------|-------------------|
| OOP (classes + methods) | Easy (new subclass) | Hard (modify all classes) |
| FP (ADTs + pattern match) | Hard (modify all functions) | Easy (new function) |

### How Each System Addresses It

**Traits/Typeclasses (Rust, Haskell):**
- New type: define the type, implement existing traits — existing code untouched
- New operation: define a new trait, implement it for existing types — existing code untouched (subject to orphan rules)
- Partially solved: the orphan rule restricts who can add implementations

**Protocols (Clojure, Elixir):**
- New type: define the type, implement existing protocols
- New operation: define a new protocol, extend it to existing types via `extend-type`
- Fully solved: no orphan rule, retroactive implementation unrestricted

**Interfaces (Go):**
- New type: define the type with the right methods — automatically satisfies existing interfaces
- New operation: define a new interface — existing types that already have the methods satisfy it immediately
- Fully solved via structural typing (but cannot add methods to existing types)

**Multimethods (Clojure):**
- Both types and operations are independently extensible
- Most flexible: dispatch on arbitrary properties of any/all arguments
- Performance cost: slower than protocol dispatch

**Visitor Pattern (OOP workaround):**
- Inverts the OOP trade-off: operations become easy to add, types become hard
- Does not solve the problem — just shifts which direction is difficult

### Summary Table

| Language | Add Types | Add Ops | Constraint |
|----------|-----------|---------|------------|
| Rust | Yes | Yes (local) | Orphan rule |
| Haskell | Yes | Yes (local) | Instance coherence |
| Go | Yes | Yes | Cannot add methods to foreign types |
| Clojure | Yes | Yes | Convention only |
| Elixir | Yes | Yes | Convention only |
| Swift | Yes | Yes | Duplicate conformance risk |
| Java | Yes | Partial (default methods) | Diamond ambiguity |
| TypeScript | Yes | Yes | Structural, no restrictions |
| Python | Yes | Yes | No enforcement |
| Scala | Yes | Yes (local) | Orphan-like restrictions |

**Sources:**
- [Eli Bendersky: The Expression Problem and its Solutions](https://eli.thegreenplace.net/2016/the-expression-problem-and-its-solutions/)
- [Wikipedia: Expression Problem](https://en.wikipedia.org/wiki/Expression_problem)
- [Philip Wadler: The Expression Problem](https://homepages.inf.ed.ac.uk/wadler/papers/expression/expression.txt)
- [Strange Loop: Clojure's Solutions to the Expression Problem](https://thestrangeloop.com/2010/clojures-solutions-to-the-expression-problem.html)

---

## 10. Method Resolution Order

How languages decide which method implementation to call when multiple candidates exist.

### Single-Implementation Languages (No Ambiguity)

**Rust** and **Go** avoid MRO entirely:
- Rust: at most one impl of a trait for a type. Name collisions require fully qualified syntax. Inherent methods shadow trait methods.
- Go: interfaces are satisfied once. Embedded struct methods are promoted but can be shadowed by the outer type.

### C3 Linearization (Python, Scala)

**Algorithm** (Python formulation):
```
L[C] = C + merge(L[P1], L[P2], ..., L[Pn], [P1, P2, ..., Pn])
```

**Merge procedure:**
1. Take the head of the first non-empty list
2. If that head does not appear in the tail of any other list, select it
3. Otherwise, skip to the next list's head
4. Repeat until all lists are empty, or raise an error (inconsistent hierarchy)

**Diamond example:**
```python
class A: pass
class B(A): pass
class C(A): pass
class D(B, C): pass

D.__mro__  # => (D, B, C, A, object)
```

**Guarantees:**
- **Local precedence order**: parents are visited left-to-right
- **Monotonicity**: if X precedes Y in any ancestor's MRO, X precedes Y everywhere
- **Consistency**: the algorithm rejects hierarchies that violate these properties

**Scala** uses a similar right-to-left linearization with duplicate removal. The key difference: Scala processes the rightmost mixin first, giving it the highest priority.

### Explicit Resolution (Rust, Java)

When there is no linearization and names collide:

**Rust:**
```rust
<Type as Trait>::method(&instance)
```

**Java:**
```java
InterfaceA.super.method();
```

The programmer explicitly chooses. The compiler refuses to guess.

### Priority Rules Across Languages

| Language | Priority Order |
|----------|---------------|
| Rust | Inherent method > trait method (if unambiguous) > compile error |
| Python | C3 linearization MRO |
| Scala | Linearization (rightmost mixin highest priority) |
| Java | Class method > single default > compile error if ambiguous |
| Swift | Concrete type method > protocol extension default |
| Go | Outer type method > embedded type method > ambiguity error |
| Clojure | Direct impl > metadata > external extension |

### Implications for lx

lx's trait system should follow Rust's model: no implicit resolution order, explicit disambiguation required. For an agent orchestration language, silent method resolution surprises are unacceptable — an agent must know exactly which protocol handler is being invoked.

**Sources:**
- [Python.org: The Python 2.3 Method Resolution Order](https://www.python.org/download/releases/2.3/mro/)
- [GeeksforGeeks: MRO in Python](https://www.geeksforgeeks.org/python/method-resolution-order-in-python-inheritance/)
- [Medium: C3 Linearization Algorithm](https://justanotherdev7.medium.com/understanding-pythons-c3-linearization-algorithm-c92909dacf1d)
- [Rust Book: Disambiguating Methods](https://doc.rust-lang.org/book/ch20-02-advanced-traits.html)
