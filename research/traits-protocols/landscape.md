# Trait, Protocol, and Interface Systems Across Languages

A survey of polymorphism abstractions in 10 languages, covering syntax, semantics, dispatch mechanisms, and distinguishing design choices.

---

## 1. Rust Traits

Rust traits define shared behavior through method signatures and optional default implementations. They are the primary mechanism for both static (monomorphized) and dynamic (vtable-based) polymorphism.

### Core Syntax

```rust
pub trait Summary {
    fn summarize_author(&self) -> String;

    fn summarize(&self) -> String {
        format!("(Read more from {}...)", self.summarize_author())
    }
}

impl Summary for NewsArticle {
    fn summarize_author(&self) -> String { self.author.clone() }
}
```

Default methods can call other trait methods, including abstract ones. Implementors can override any default.

### Associated Types and Constants

Associated types pin a generic to a single concrete type per implementation, unlike generic type parameters which allow multiple implementations:

```rust
pub trait Iterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
}
```

Associated constants work similarly — the trait declares `const NAME: Type;` and each implementation provides a value.

### Trait Bounds and Where Clauses

```rust
fn notify<T: Summary + Display>(item: &T) { ... }

fn complex<T, U>(t: &T, u: &U) -> i32
where
    T: Display + Clone,
    U: Clone + Debug,
{ ... }
```

`impl Trait` syntax provides shorthand: `fn notify(item: &impl Summary)`.

### Blanket Implementations

Apply a trait to all types satisfying a bound:

```rust
impl<T: Display> ToString for T { ... }
```

Any `Display` type automatically gets `to_string()`. The standard library uses this extensively.

### Orphan Rule (Coherence)

You can implement a trait on a type only if either the trait or the type is local to your crate. This prevents conflicting implementations across crates. The **newtype pattern** works around this: wrap the foreign type in a local tuple struct.

### Trait Objects and Dynamic Dispatch

`dyn Trait` creates a trait object — a fat pointer (16 bytes on 64-bit): one pointer to data, one to the vtable. The vtable contains function pointers for each method plus the type's size, alignment, and destructor.

**Dyn-safety rules** (formerly "object safety"): A trait is dyn-compatible if:
- It does not require `Self: Sized`
- No associated constants or GATs
- All methods either have `Self` only in receiver position with no type parameters, or are bounded by `where Self: Sized`
- No `async fn` or `-> impl Trait` return types

Workaround: add `where Self: Sized` to non-dyn-safe methods to exclude them from the vtable while keeping them available for concrete types.

### Auto Traits and Marker Traits

- **Auto traits** (`Send`, `Sync`): automatically implemented for types whose fields all satisfy the trait. Negative impls (`impl !Send for T`) opt out.
- **Marker traits** (`Copy`, `Sized`): carry no methods but inform the compiler about type properties. `Copy` means bitwise-copy semantics. `Sized` means compile-time-known size.

### Supertraits

```rust
trait OutlinePrint: fmt::Display {
    fn outline_print(&self) { ... }
}
```

Implementing `OutlinePrint` requires that `Display` is also implemented.

### Specialization (Unstable)

RFC 1210 proposes overlapping impls where the more specific impl wins. The `default` keyword marks specializable methods. Full specialization is unsound; `min_specialization` is a restricted subset used internally by the standard library.

### Sealed Traits

A design pattern (not a language feature) preventing external implementations. Place a public trait in a public module but require a supertrait from a private module:

```rust
mod private {
    pub trait Sealed {}
}
pub trait MyTrait: private::Sealed { ... }
```

Downstream crates cannot name `private::Sealed`, so they cannot implement `MyTrait`.

### Extension Traits

A pattern for adding methods to foreign types. Define a new trait with the desired methods and provide a blanket impl. RFC 445 establishes the `Ext` suffix convention (e.g., `FutureExt`).

**Sources:**
- [The Rust Book: Traits](https://doc.rust-lang.org/book/ch10-02-traits.html)
- [The Rust Book: Advanced Traits](https://doc.rust-lang.org/book/ch20-02-advanced-traits.html)
- [RFC 255: Object Safety](https://rust-lang.github.io/rfcs/0255-object-safety.html)
- [RFC 1210: Specialization](https://rust-lang.github.io/rfcs/1210-impl-specialization.html)
- [Rust Reference: Special Types and Traits](https://doc.rust-lang.org/reference/special-types-and-traits.html)
- [Definitive Guide to Sealed Traits](https://predr.ag/blog/definitive-guide-to-sealed-traits-in-rust/)
- [RFC 445: Extension Trait Conventions](https://rust-lang.github.io/rfcs/0445-extension-trait-conventions.html)

---

## 2. Haskell Typeclasses

Haskell's typeclasses are the original formulation of ad-hoc polymorphism through constrained parametric polymorphism — they predate and inspired Rust traits.

### Class Definitions and Instances

```haskell
class Eq a where
  (==) :: a -> a -> Bool
  (/=) :: a -> a -> Bool
  x /= y = not (x == y)   -- default method
```

Instances declare how a specific type satisfies the class:

```haskell
instance Eq Bool where
  True  == True  = True
  False == False = True
  _     == _     = False
```

### Superclasses

```haskell
class Eq a => Ord a where
  compare :: a -> a -> Ordering
```

`Eq` is a superclass of `Ord` — any `Ord` instance must also have an `Eq` instance.

### Multi-Parameter Typeclasses

Generalize single-parameter classes to relationships between types:

```haskell
class Collection c e where
  insert :: e -> c -> c
  member :: e -> c -> Bool
```

Without further constraints, type inference becomes ambiguous.

### Functional Dependencies

Constrain multi-parameter typeclasses so one parameter determines another:

```haskell
class Collection c e | c -> e where ...
```

`c -> e` means the collection type uniquely determines the element type. This restores type inference by eliminating ambiguity.

### Type Families

An alternative to functional dependencies using type-level functions:

```haskell
type family Element c
type instance Element [a] = a
type instance Element (Set a) = a

class Container c where
  insert :: Element c -> c -> c
```

Type families provide a more functional style of type-level programming versus the relational style of fundeps.

### Deriving Strategies

GHC supports four deriving strategies:
- **stock**: compiler generates standard instances for built-in classes (`Eq`, `Ord`, `Show`, `Read`, `Enum`, `Bounded`, `Generic`, etc.)
- **newtype** (`GeneralizedNewtypeDeriving`): reuses the wrapped type's instance
- **anyclass** (`DeriveAnyClass`): generates an empty instance relying on default method implementations
- **via** (`DerivingVia`, GHC 8.6+): derives via a specified representationally-equal type, enabling instance reuse across types with the same runtime representation

### Differences from OOP Interfaces

Typeclasses are **not** attached to types — they live in a separate namespace and can be defined independently. A key distinction: typeclass dispatch is resolved at compile time (dictionary passing), not via vtables. Typeclasses support return-type polymorphism (`read :: Read a => String -> a`) which OOP interfaces cannot express.

**Sources:**
- [HaskellWiki: Functional Dependencies](https://wiki.haskell.org/Functional_dependencies)
- [HaskellWiki: Type Families](https://wiki.haskell.org/GHC/Type_families)
- [HaskellWiki: Multi-parameter Type Classes](https://wiki.haskell.org/Multi-parameter_type_class)
- [GHC User Guide: Deriving Strategies](https://ghc.gitlab.haskell.org/ghc/doc/users_guide/exts/deriving_strategies.html)
- [GHC User Guide: DerivingVia](https://ghc.gitlab.haskell.org/ghc/doc/users_guide/exts/deriving_via.html)
- [Kowainik: Strategic Deriving](https://kowainik.github.io/posts/deriving)

---

## 3. Go Interfaces

Go interfaces define behavior through method sets. Satisfaction is implicit — no `implements` keyword exists.

### Implicit Satisfaction

```go
type Stringer interface {
    String() string
}

type MyType struct { Name string }
func (m MyType) String() string { return m.Name }
// MyType satisfies Stringer without declaring it
```

If a type has all the methods an interface requires, it satisfies that interface. This is verified at compile time, combining duck-typing ergonomics with static safety.

### Interface Composition (Embedding)

```go
type Reader interface { Read(p []byte) (n int, err error) }
type Writer interface { Write(p []byte) (n int, err error) }
type ReadWriter interface {
    Reader
    Writer
}
```

Interfaces embed other interfaces. A type must satisfy all embedded interfaces' methods.

### Empty Interface and `any`

`interface{}` (aliased to `any` since Go 1.18) can hold any value. It carries no method requirements, making it Go's universal type container.

### Type Assertions

Extract the concrete value from an interface:

```go
var i interface{} = "hello"
s := i.(string)         // panics if wrong type
s, ok := i.(string)     // safe: ok = false if wrong type
```

### Type Switches

```go
switch v := i.(type) {
case string:  fmt.Println("string:", v)
case int:     fmt.Println("int:", v)
default:      fmt.Println("unknown")
}
```

Type switches dispatch on the dynamic type stored in an interface value.

### Design Philosophy

Go deliberately avoids inheritance. Composition through struct embedding and interface composition replaces class hierarchies. Small interfaces (1-2 methods) are idiomatic — `io.Reader`, `io.Writer`, `fmt.Stringer`. The implicit satisfaction model means types from different packages can satisfy the same interface without coordination.

**Sources:**
- [Go Blog: Interfaces](https://dev.to/arasosman/understanding-gos-type-system-a-complete-guide-to-interfaces-structs-and-composition-2025-3an)
- [FullStory: Is Go Duck-Typed?](https://www.fullstory.com/blog/is-go-duck-typed/)
- [Leapcell: Type Assertions and Type Switches in Go](https://leapcell.io/blog/understanding-type-assertion-and-type-switch-in-go)

---

## 4. Elixir Protocols

Elixir protocols provide type-based polymorphic dispatch in a functional language, solving the expression problem without inheritance.

### defprotocol and defimpl

```elixir
defprotocol Size do
  def size(data)
end

defimpl Size, for: BitString do
  def size(string), do: byte_size(string)
end

defimpl Size, for: Map do
  def size(map), do: map_size(map)
end
```

Dispatch is always based on the type of the first argument.

### Fallback to Any

```elixir
defprotocol Size do
  @fallback_to_any true
  def size(data)
end

defimpl Size, for: Any do
  def size(_), do: 0
end
```

### Deriving

```elixir
defmodule User do
  @derive [Size]
  defstruct [:name, :age]
end
```

`@derive` generates an implementation at compile time using the protocol's `__deriving__/2` callback.

### Protocol Consolidation

In development, protocol dispatch must check at runtime whether an implementation exists for a given type. **Protocol consolidation** (enabled by default in Mix projects) links protocols directly to their implementations at compile time, making dispatch equivalent to two function calls — one to find the implementation module, one to call the method.

### Protocols vs Behaviours

A behaviour is a callback specification: "give me a module, I will call these functions on it." A protocol is a behaviour plus type-based dispatch logic. Behaviours dispatch on modules; protocols dispatch on data types.

**Sources:**
- [Elixir Docs: Protocols](https://hexdocs.pm/elixir/protocols.html)
- [Elixir Docs: Protocol Module](https://hexdocs.pm/elixir/Protocol.html)
- [Elixir Getting Started: Protocols](https://elixir-lang.org/getting-started/protocols.html)

---

## 5. Swift Protocols

Swift protocols define blueprints of methods, properties, and associated types. Protocol-oriented programming (POP) is a core Swift paradigm — preferring protocol conformance and composition over class inheritance.

### Protocol Definitions

```swift
protocol Drawable {
    var color: Color { get set }
    func draw()
}
```

Protocols can require properties (get-only or get/set), methods, initializers, and subscripts.

### Default Implementations via Extensions

```swift
extension Drawable {
    func draw() {
        print("Drawing in \(color)")
    }
}
```

Conforming types inherit the default but can override it. This is the primary mechanism for protocol-oriented code reuse.

### Protocol Composition

```swift
func render(item: Drawable & Printable) { ... }
```

The `&` operator creates ad-hoc composite requirements.

### Associated Types

```swift
protocol Container {
    associatedtype Item
    mutating func append(_ item: Item)
    var count: Int { get }
    subscript(i: Int) -> Item { get }
}
```

Conforming types specify the concrete `Item` type, similar to Rust's associated types.

### Conditional Conformance

```swift
extension Array: Equatable where Element: Equatable {
    static func == (lhs: [Element], rhs: [Element]) -> Bool { ... }
}
```

`Array` conforms to `Equatable` only when its elements do. This enables recursive conformance: `[[Int]]` is `Equatable` because `[Int]` is, because `Int` is.

### Opaque Types (`some Protocol`) vs Existential Types (`any Protocol`)

- `some Protocol` (Swift 5.1+): the concrete type is fixed but hidden from the caller. Enables static dispatch and compiler optimizations. Used extensively in SwiftUI (`some View`).
- `any Protocol` (Swift 5.6+): an existential container that can hold any conforming type at runtime. Uses an existential container (similar to Rust's fat pointer) with witness tables. Dynamic dispatch, heap allocation possible.

```swift
func makeShape() -> some Shape { Circle() }  // opaque: always Circle
func anyShape() -> any Shape { ... }          // existential: could be anything
```

### Class-Only Protocols

```swift
protocol Delegate: AnyObject { ... }
```

Restricts conformance to reference types (classes), enabling weak references.

**Sources:**
- [Swift Docs: Protocols](https://docs.swift.org/swift-book/documentation/the-swift-programming-language/protocols/)
- [Swift by Sundell: Conditional Conformances](https://www.swiftbysundell.com/articles/conditional-conformances-in-swift/)
- [Swift.org: Conditional Conformance Blog](https://www.swift.org/blog/conditional-conformance/)
- [Swift by Sundell: some and any Keywords](https://www.swiftbysundell.com/articles/referencing-generic-protocols-with-some-and-any-keywords/)
- [Hacking with Swift: Existential any](https://www.hackingwithswift.com/swift/5.6/existential-any)

---

## 6. Scala Traits

Scala traits combine interface specification with mixin composition, supporting multiple inheritance through linearization.

### Trait Definitions

```scala
trait Greeter {
  def greet(name: String): Unit
  def hello(): Unit = println("Hello!")  // concrete method
}
```

Traits can have abstract methods, concrete methods, and fields.

### Mixin Composition

```scala
// Scala 2
class MyClass extends Base with TraitA with TraitB
// Scala 3
class MyClass extends Base, TraitA, TraitB
```

A class has one superclass but unlimited trait mixins.

### Linearization

When multiple traits define the same method, Scala resolves ambiguity through linearization. The algorithm:

1. Start with the class itself
2. Process mixins **right to left**
3. For each mixin, recursively compute its linearization
4. Remove duplicates, keeping only the **rightmost** occurrence

For `class D extends B with C` where `B extends A` and `C extends A`:
- Linearization: `D -> C -> B -> A -> AnyRef -> Any`
- `super` calls chain through this order

### Abstract Override (Stackable Modifications)

```scala
trait Base {
  def process(s: String): String
}

trait Logger extends Base {
  abstract override def process(s: String): String = {
    println(s"Processing: $s")
    super.process(s)
  }
}
```

`abstract override` allows a trait to call `super.process()` before the concrete implementation is known. The concrete implementation must be mixed in first. Multiple stackable traits chain via `super`:

```scala
class Pipeline extends ConcreteProcessor with Logger with Validator
// Calls: Validator.process -> Logger.process -> ConcreteProcessor.process
```

### Self-Types

```scala
trait UserRepository {
  self: DatabaseConnection =>
  def findUser(id: Int): User = query(s"SELECT * FROM users WHERE id=$id")
}
```

Self-type annotations (`self: T =>`) declare dependencies without inheritance. The trait can use `T`'s methods but does not extend `T`. Used for dependency injection (Cake Pattern).

### Sealed Traits

```scala
sealed trait Shape
case class Circle(r: Double) extends Shape
case class Rect(w: Double, h: Double) extends Shape
```

`sealed` restricts all implementations to the same file. The compiler can verify exhaustive pattern matches — the foundation for algebraic data types in Scala.

**Sources:**
- [Scala Docs: Mixin Composition](https://docs.scala-lang.org/tour/mixin-class-composition.html)
- [Scala Docs: Traits](https://docs.scala-lang.org/tour/traits.html)
- [Baeldung: Stackable Trait Pattern](https://www.baeldung.com/scala/stackable-trait-pattern)
- [Medium: Guide to Trait Linearisation](https://medium.com/@shayan1337/guide-to-trait-linearisation-in-scala-60611571e088)
- [Underscore: Sealed Traits](https://underscore.io/blog/posts/2015/06/02/everything-about-sealed.html)
- [GeeksforGeeks: Trait Linearization](https://www.geeksforgeeks.org/trait-linearization-in-scala/)

---

## 7. Clojure Protocols

Clojure protocols decouple function definitions from type implementations, providing high-performance polymorphic dispatch.

### defprotocol

```clojure
(defprotocol Drawable
  "Protocol for things that can be drawn"
  (draw [this] "Draw the thing")
  (bounds [this] "Return bounding box"))
```

Protocols are named sets of named methods. Each method must have at least one argument (the dispatch target). Dispatch is on the type of the first argument only (single dispatch).

### Implementation Approaches

**Inline** (with deftype/defrecord):
```clojure
(defrecord Circle [x y r]
  Drawable
  (draw [this] (render-circle x y r))
  (bounds [this] {:x (- x r) :y (- y r) :w (* 2 r) :h (* 2 r)}))
```

**External** (extend-type — extend an existing type with a protocol):
```clojure
(extend-type String
  Drawable
  (draw [this] (render-text this))
  (bounds [this] {:x 0 :y 0 :w (count this) :h 1}))
```

**External** (extend-protocol — extend one protocol to multiple types):
```clojure
(extend-protocol Drawable
  String
  (draw [this] (render-text this))
  java.awt.Shape
  (draw [this] (render-awt this)))
```

### Performance

Protocols dispatch via the JVM's polymorphic inline caches, making them roughly 5x faster than multimethods for type-based dispatch. `defprotocol` also generates a corresponding Java interface.

### Protocols vs Multimethods

| Feature | Protocols | Multimethods |
|---------|-----------|--------------|
| Dispatch | Type of first arg | Arbitrary function of all args |
| Grouping | Named set of methods | Individual functions |
| Performance | Fast (JVM inline caches) | Slower (arbitrary dispatch fn) |
| Flexibility | Single dispatch only | Multi-argument, value-based |

Protocols handle the "90% case" where type-based dispatch suffices. Multimethods handle the remaining cases needing dispatch on values, multiple arguments, or ad-hoc hierarchies.

### Metadata-Based Extension (Clojure 1.10+)

```clojure
(defprotocol Component
  :extend-via-metadata true
  (start [component]))

(def c (with-meta {} {`start (fn [_] "started")}))
(start c)  ;; => "started"
```

Resolution order: direct implementation > metadata > external extension.

### Solving the Expression Problem

Protocols allow adding new operations (new protocols) to existing types (via `extend-type`) and adding new types (via `defrecord`) to existing protocols — both without modifying existing code.

**Sources:**
- [Clojure Reference: Protocols](https://clojure.org/reference/protocols)
- [Brave Clojure: Multimethods, Protocols, Records](https://www.braveclojure.com/multimethods-records-protocols/)
- [Aphyr: Clojure Polymorphism](https://aphyr.com/posts/352-clojure-from-the-ground-up-polymorphism)
- [Freshcode: Clojure Protocols and the Expression Problem](https://www.freshcodeit.com/blog/clojure-protocols-and-the-expression-problem)
- [Inside Clojure: Polymorphic Performance](https://insideclojure.org/2015/04/27/poly-perf/)

---

## 8. TypeScript Interfaces

TypeScript interfaces define object shapes using structural typing — no explicit `implements` is required for type compatibility.

### Structural Typing

```typescript
interface Point {
    x: number;
    y: number;
}

function plot(p: Point) { ... }

plot({ x: 1, y: 2 });             // literal OK
const obj = { x: 1, y: 2, z: 3 };
plot(obj);                          // extra properties OK (when not literal)
```

A value satisfies an interface if it has all required members with compatible types. No `implements` keyword needed (though classes can use `implements` for explicit checking).

### Declaration Merging

```typescript
interface Box { height: number; }
interface Box { width: number; }
// Merged: interface Box { height: number; width: number; }
```

Multiple interface declarations with the same name merge automatically. Later declarations' non-function members must have identical types if they share names. For function members, later declarations take higher priority (appear earlier in the overload set).

This is exclusive to interfaces — `type` aliases cannot merge.

### Interface Extension

```typescript
interface Shape { color: string; }
interface Square extends Shape { sideLength: number; }
```

Interfaces extend other interfaces. Classes can implement multiple interfaces.

### Index Signatures

```typescript
interface StringMap {
    [key: string]: string;
}

interface NumberMap {
    [index: number]: string;  // numeric indexer return must be subtype of string indexer
}
```

Index signatures allow dynamic property access with constrained types.

### Mapped Types from Interfaces

```typescript
type Readonly<T> = { readonly [P in keyof T]: T[P] };
type Partial<T> = { [P in keyof T]?: T[P] };

interface User { name: string; age: number; }
type ReadonlyUser = Readonly<User>;
```

`keyof T` extracts property keys; mapped types iterate over them to produce new types.

### Classes vs Interfaces

`extends` creates subclass relationships. `implements` checks structural conformance against an interface without inheriting implementation. A class can `implements` multiple interfaces.

**Sources:**
- [TypeScript Handbook: Declaration Merging](https://www.typescriptlang.org/docs/handbook/declaration-merging.html)
- [TypeScript Handbook: Object Types](https://www.typescriptlang.org/docs/handbook/2/objects.html)
- [TypeScript Handbook: Mapped Types](https://www.typescriptlang.org/docs/handbook/2/mapped-types.html)
- [TypeScript Deep Dive: Index Signatures](https://basarat.gitbook.io/typescript/type-system/index-signatures)

---

## 9. Java Interfaces

Java interfaces have evolved from pure abstract contracts into a hybrid supporting concrete behavior, reflecting the language's gradual adoption of traits-like features.

### Evolution Timeline

| Version | Feature |
|---------|---------|
| Java 1.0 | Abstract method-only interfaces |
| Java 8 | `default` methods, `static` methods |
| Java 9 | `private` methods |
| Java 15-17 | `sealed` interfaces (preview to final) |

### Default Methods (Java 8+)

```java
public interface Collection<E> {
    default Stream<E> stream() {
        return StreamSupport.stream(spliterator(), false);
    }
}
```

Default methods solved the **interface evolution problem**: adding methods to `Collection` without breaking every existing implementation. Implementing classes can override defaults.

**Diamond conflict resolution**: if two interfaces provide default methods with the same signature, the implementing class must explicitly override and choose:

```java
class MyClass implements A, B {
    public void method() {
        A.super.method();  // explicit disambiguation
    }
}
```

### Static Methods (Java 8+)

```java
public interface Comparator<T> {
    static <T> Comparator<T> naturalOrder() { ... }
}
```

Utility methods on the interface itself. Not inherited by implementing classes.

### Private Methods (Java 9+)

```java
public interface Logger {
    default void logInfo(String msg) { log("INFO", msg); }
    default void logError(String msg) { log("ERROR", msg); }
    private void log(String level, String msg) {
        System.out.println(level + ": " + msg);
    }
}
```

Helper methods shared between default methods without exposing them in the public API.

### Sealed Interfaces (Java 17)

```java
public sealed interface Shape permits Circle, Rectangle, Triangle {}
public record Circle(double radius) implements Shape {}
public record Rectangle(double w, double h) implements Shape {}
public final class Triangle implements Shape { ... }
```

`sealed` restricts which classes/interfaces can implement. Permitted subtypes must be `final`, `sealed`, or `non-sealed`. Enables exhaustive pattern matching in `switch` expressions.

**Sources:**
- [Baeldung: Sealed Classes and Interfaces](https://www.baeldung.com/java-sealed-classes-interfaces)
- [JEP 409: Sealed Classes](https://openjdk.org/jeps/409)
- [Oracle: Java 17 Sealed Classes](https://docs.oracle.com/en/java/javase/17/language/sealed-classes-and-interfaces.html)
- [Medium: Java 8 to 17 Interface Evolution](https://anushasp07.medium.com/java-8-to-17-how-interfaces-have-transformed-over-time-ed4a93771039)

---

## 10. Python Protocols (PEP 544)

PEP 544 introduced structural subtyping ("static duck typing") to Python's type system via `Protocol` classes.

### Defining Protocols

```python
from typing import Protocol

class SupportsClose(Protocol):
    def close(self) -> None: ...
```

Any class with a compatible `close()` method is an implicit subtype — no inheritance needed:

```python
class Resource:
    def close(self) -> None:
        self.file.close()

def close_all(things: Iterable[SupportsClose]) -> None:
    for t in things:
        t.close()

close_all([open('f.txt'), Resource()])  # both satisfy SupportsClose
```

### Implicit vs Explicit Conformance

**Implicit** (structural): a class matches if it has all required members with compatible types. No subclassing.

**Explicit** (nominal): a class can inherit from the Protocol to get default implementations:

```python
class PColor(Protocol):
    @abstractmethod
    def draw(self) -> str: ...
    def complex_method(self) -> int:
        return 42  # default implementation

class NiceColor(PColor):  # explicit: inherits complex_method
    def draw(self) -> str: return "blue"
```

### runtime_checkable

```python
@runtime_checkable
class SupportsClose(Protocol):
    def close(self) -> None: ...

isinstance(open('f.txt'), SupportsClose)  # True at runtime
```

`@runtime_checkable` enables `isinstance()` checks, but only verifies member existence — not type signatures. It cannot catch incorrect parameter types or return types.

### Protocol Composition

```python
class SizedAndClosable(Sized, SupportsClose, Protocol):
    pass
```

Combine multiple protocols via multiple inheritance (with `Protocol` as a base).

### Generic and Recursive Protocols

```python
T_co = TypeVar('T_co', covariant=True)

class Box(Protocol[T_co]):
    def content(self) -> T_co: ...

class Traversable(Protocol):
    def leaves(self) -> Iterable['Traversable']: ...
```

### Callback Protocols

```python
class Combiner(Protocol):
    def __call__(self, *vals: bytes, maxlen: int = None) -> list[bytes]: ...
```

Protocols with `__call__` define complex callable signatures beyond what `Callable[...]` can express.

### Protocol vs ABC

| Feature | Protocol | ABC |
|---------|----------|-----|
| Subtyping | Structural | Nominal |
| Registration | Implicit | Explicit (inherit or `register()`) |
| Runtime checking | `@runtime_checkable` (limited) | `isinstance()` (full) |
| Default methods | Via explicit inheritance | Via inheritance |
| Static checking | mypy structural match | mypy nominal match |

Protocols complement ABCs rather than replacing them. ABCs are better when you want enforced registration; Protocols are better when you want duck-typing with static verification.

### Modules as Protocol Implementations

Modules can satisfy protocols if their attributes match:

```python
class Options(Protocol):
    timeout: int
    one_flag: bool

import default_config  # has timeout and one_flag attributes
setup(default_config)  # satisfies Options
```

**Sources:**
- [PEP 544: Protocols](https://peps.python.org/pep-0544/)
- [mypy: Protocols and Structural Subtyping](https://mypy.readthedocs.io/en/stable/protocols.html)
- [typing docs: Protocols](https://typing.python.org/en/latest/reference/protocols.html)
- [ABC vs Protocol Comparison](https://jellis18.github.io/post/2022-01-11-abc-vs-protocol/)
