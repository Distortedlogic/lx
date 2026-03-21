# Standard Library Design Philosophy, Prelude Patterns, and Protocols

Research survey covering design trade-offs in standard libraries, auto-import patterns, type-system integration, and the protocol/trait/interface patterns that connect built-in functions to user-defined types.

---

## 1. "Batteries Included" vs Minimal Core

### 1.1 The Spectrum

Languages fall on a spectrum from "ship everything" to "ship nothing":

```
Maximal ◄──────────────────────────────────────────────► Minimal
Python    Java    Go    Ruby    Elixir    JavaScript    Lua    C
```

### 1.2 Python: Batteries Included

Python's philosophy, articulated since the 1990s: the standard library should cover enough ground that a developer can solve common problems without installing third-party packages.

**What ships with Python stdlib:**
- `json`, `csv`, `xml` -- data formats
- `http.server`, `urllib` -- HTTP client and server
- `sqlite3` -- embedded database
- `threading`, `multiprocessing`, `asyncio` -- concurrency
- `re` -- regular expressions
- `os`, `sys`, `pathlib` -- system interaction
- `unittest`, `doctest` -- testing
- `logging` -- structured logging
- `hashlib`, `hmac`, `secrets` -- cryptography
- `email`, `html`, `socketserver` -- networking
- `typing` -- type annotations
- `dataclasses`, `enum`, `abc` -- class utilities
- `functools`, `itertools`, `collections` -- functional/data structure utilities
- ~300+ modules total

**Trade-offs:**
- (+) Easy onboarding, no "which HTTP library?" decision paralysis
- (+) Standardized interfaces across the ecosystem
- (+) Portable -- same code works everywhere Python runs
- (-) Stdlib evolves slowly (annual releases, backward compatibility pressure)
- (-) Some modules become stale (PEP 594 removed "dead batteries" like `aifc`, `audioop`, `cgi`, `telnetlib`)
- (-) Stdlib quality varies (some modules have known issues but can't be removed)

Source: [PEP 594: Removing dead batteries](https://peps.python.org/pep-0594/), [Batteries included](https://szabgab.com/batteries-included)

### 1.3 Lua: Minimal Core

Lua takes the opposite approach. The entire standard library fits in ~6000 lines of C. Only what ISO C can portably provide is included.

**What ships with Lua:** string manipulation, table operations, basic math, file I/O (via C stdio), OS services (date/time, env vars, file rename/remove), coroutines, debug hooks.

**What does NOT ship:** networking, threading, GUI, regex (Lua has "patterns" -- a simpler, portable alternative), JSON, HTTP, database access, encryption, comprehensive Unicode support.

**Trade-offs:**
- (+) Tiny footprint (~150KB compiled), embeddable anywhere
- (+) Completely portable across platforms
- (+) Host application provides domain-specific libraries through C API
- (+) No dependency management headaches for the core
- (-) Every project needs external libraries for basic tasks
- (-) Package ecosystem (LuaRocks) is smaller and less standardized than pip
- (-) "Quick and dirty" scripting requires more setup than Python

**Design rationale:** Lua exists primarily as an embedded language. The host application (game engine, text editor, network device) provides the domain-specific APIs. Lua provides the glue language. Shipping a large stdlib would bloat every embedded deployment.

Source: [A Look at the Design of Lua](https://cacm.acm.org/research/a-look-at-the-design-of-lua/)

### 1.4 Go: The Middle Path

Go ships a moderately rich standard library:
- `net/http` -- production-grade HTTP client and server
- `encoding/json`, `encoding/xml` -- serialization
- `database/sql` -- database interface (drivers are third-party)
- `crypto/*` -- comprehensive cryptography
- `testing` -- test framework
- `sync`, `context` -- concurrency primitives
- `fmt`, `io`, `os`, `path`, `strings`, `strconv` -- core utilities

**What Go deliberately omits:**
- No collections library (before generics, the built-in slice/map/channel covered most needs)
- No dependency injection framework
- No ORM
- No CLI argument parser beyond basic `flag` package

**Design rationale:** Go includes what almost every server program needs (HTTP, JSON, crypto, testing) but draws the line at things that are opinionated or domain-specific. The philosophy is: the stdlib should be obviously correct for its use case, not a compromise that tries to serve all domains.

Source: [Go at Google: Language Design](https://go.dev/talks/2012/splash.article), [Golang: Some batteries not included](https://yolken.net/blog/golang-batteries-not-included)

### 1.5 Key Insight for Language Design

The "right" level of stdlib depends on the language's deployment model:

| Deployment model               | Stdlib approach  | Example     |
|--------------------------------|------------------|-------------|
| Embedded in host application   | Minimal          | Lua         |
| Standalone scripting           | Maximal          | Python      |
| Server software                | Targeted         | Go          |
| Systems programming            | Curated + ecosystem | Rust     |
| Agent/workflow orchestration   | Domain primitives | (lx?)      |

---

## 2. Prelude and Auto-Import Patterns

### 2.1 What Languages Auto-Import

| Language | Auto-imported                              | Mechanism                    | Can be disabled? |
|----------|--------------------------------------------|------------------------------|------------------|
| Rust     | `std::prelude::v1::*`                      | Compiler inserts `use`       | `#![no_std]`     |
| Python   | `builtins` module                          | LEGB scope fallback          | Override `__builtins__` |
| Haskell  | `Prelude` module                           | Implicit import              | `NoImplicitPrelude` |
| Scala    | `java.lang._`, `scala._`, `scala.Predef._` | Compiler inserts imports     | Yes (Scala 3)    |
| Ruby     | `Kernel` module mixed into `Object`        | Mixin inheritance            | Not easily       |
| Elixir   | `Kernel` module + `Kernel.SpecialForms`    | Compiler auto-imports        | `import Kernel, except: [...]` |
| Go       | Predeclared identifiers                    | Language specification       | No               |
| JavaScript | Global object properties                 | Scope chain reaches global   | No               |
| Lua      | Basic library functions                    | Registered in global table   | Remove from `_G` |

### 2.2 Rust's Prelude Design Philosophy

Rust's prelude follows strict criteria:

1. **Items must be used in nearly every Rust program.** The bar is "would most Rust programs break without this import?"

2. **Traits needed for methods on primitive types.** Primitive types like `str`, `[T]`, and `char` can't have inherent methods (they're defined by the compiler, not a crate). Extension traits in the prelude provide their methods. Example: `Iterator` in the prelude means `.map()`, `.filter()`, etc. work on all iterators without explicit import.

3. **Additions are effectively permanent.** Removing something from the prelude would be a breaking change for all Rust code. So the threshold is very high.

4. **Edition-gated additions.** New items can be added to the prelude of a new edition (2021, 2024) without breaking old code. This is how `TryFrom`/`TryInto` (2021) and `Future`/`IntoFuture` (2024) were added.

5. **No concrete types except essential ones.** `Option`, `Result`, `String`, `Vec`, `Box` are in the prelude because they're ubiquitous. Other types like `HashMap`, `BTreeMap`, `Rc`, `Arc` are common but not universal.

Source: [RFC 0503: Prelude Stabilization](https://rust-lang.github.io/rfcs/0503-prelude-stabilization.html), [RFC 3114: Prelude 2021](https://rust-lang.github.io/rfcs/3114-prelude-2021.html)

### 2.3 Haskell's Prelude

Haskell's `Prelude` is automatically imported unless `NoImplicitPrelude` is enabled. It contains:

- Core type classes: `Eq`, `Ord`, `Show`, `Read`, `Enum`, `Bounded`, `Num`, `Integral`, `Fractional`, `Floating`, `Real`, `RealFrac`, `RealFloat`
- Data types: `Bool`, `Maybe`, `Either`, `Ordering`, `Char`, `String` (= `[Char]`)
- List operations: `map`, `filter`, `foldl`, `foldr`, `zip`, `unzip`, `take`, `drop`, `head`, `tail`, `null`, `length`, `reverse`, `concat`, `sum`, `product`, `maximum`, `minimum`, etc.
- I/O: `IO` monad, `putStrLn`, `print`, `getLine`, `readFile`, `writeFile`
- Numeric: `+`, `-`, `*`, `abs`, `signum`, `fromInteger`, `div`, `mod`, `toInteger`, etc.
- Error: `error`, `undefined`

The Haskell community has long debated Prelude reform. Issues include:
- `head` and `tail` are partial functions (crash on empty lists) -- considered bad practice
- String-as-list-of-Char is inefficient
- Numeric hierarchy is overly complex
- Alternative preludes exist (`base-prelude`, `relude`, `rio`) that attempt to fix these issues

The lesson: a prelude is extremely hard to change once established. Every Haskell beginner learns `head` returns the first element, and changing it would invalidate millions of lines of code and documentation.

Source: [Haskell Wiki: Prelude](https://wiki.haskell.org/Prelude), [No import of Prelude](https://wiki.haskell.org/No_import_of_Prelude)

### 2.4 Scala's Three-Layer Auto-Import

Scala auto-imports three levels:

1. `java.lang._` -- Java's core types (String, Integer, System, etc.)
2. `scala._` -- Scala's core types (Int, List, Option, etc.)
3. `scala.Predef._` -- utility functions and implicit conversions

`Predef` contains:
- Type aliases: `String` (alias for `java.lang.String`), `Map`, `Set`
- Assertions: `assert`, `require`, `assume`
- Console I/O: `println`, `print`, `readLine`
- Implicit conversions: Java primitives to Scala equivalents (int -> Int, etc.)
- String interpolation helpers
- Arrow association: `->` for creating tuples/map entries
- `classOf[T]` -- Scala equivalent of Java's `.class`
- `identity` -- identity function

Later layers shadow earlier ones, so `scala.String` shadows `java.lang.String`. Scala 3 allows customizing the root imports.

Source: [Implicit Imports in Scala](https://www.baeldung.com/scala/implicit-imports)

### 2.5 Design Patterns for Auto-Import

**Pattern 1: Scope-chain fallback (Python, JavaScript)**
Names are resolved through a chain of scopes. Builtins are the outermost scope, always available as a fallback. No explicit import insertion -- it's just scope resolution.

**Pattern 2: Compiler-inserted import (Rust, Scala, Haskell)**
The compiler acts as if a specific `use`/`import` statement exists at the top of every file. The developer can override or disable this.

**Pattern 3: Mixin inheritance (Ruby)**
"Global" functions are methods on a module that's mixed into the root class. Every object inherits these methods. There's no separate "builtin scope" -- it's just the method lookup chain.

**Pattern 4: Predeclared identifiers (Go)**
The language specification defines certain identifiers as always available. They're not in any package -- they exist in a "universe" scope that contains all source files.

**Pattern 5: Table registration (Lua)**
Builtins are registered in the global table `_G` at startup. They can be removed, overridden, or sandboxed by manipulating this table. The ultimate in flexibility.

---

## 3. How Builtins Relate to the Type System

### 3.1 Python's Protocol System

Python uses "structural subtyping" for builtin integration -- also called "duck typing" formally specified through `typing.Protocol` (PEP 544) and informally through dunder methods.

**Key protocols and their dunder methods:**

| Protocol      | Methods required           | Enables                       |
|---------------|---------------------------|-------------------------------|
| `Iterable`    | `__iter__`                 | `for x in obj`, `list(obj)`   |
| `Iterator`    | `__iter__` + `__next__`    | `next(obj)`, lazy iteration   |
| `Sized`       | `__len__`                  | `len(obj)`                    |
| `Container`   | `__contains__`             | `x in obj`                    |
| `Callable`    | `__call__`                 | `obj(args)`                   |
| `Hashable`    | `__hash__`                 | `hash(obj)`, dict keys, sets  |
| `Reversible`  | `__reversed__`             | `reversed(obj)`               |
| `ContextManager` | `__enter__` + `__exit__` | `with obj as x:`             |
| `AsyncIterable` | `__aiter__`              | `async for x in obj:`        |
| `Sequence`    | `__getitem__` + `__len__`  | Indexing, slicing, iteration  |
| `Mapping`     | `__getitem__` + `__len__` + `__iter__` | Dict-like access |
| `Descriptor`  | `__get__` (+ `__set__`, `__delete__`) | Attribute access hooks |

The relationship between builtins and protocols is bidirectional:
- `len(x)` calls `x.__len__()` -- the builtin dispatches to the protocol method
- `for x in obj` calls `obj.__iter__()` then `__next__()` on the result -- language syntax dispatches to protocol methods
- The `collections.abc` module formally defines these protocols as abstract base classes

**Fallback chains** are an important design detail. Python tries multiple protocols for a single operation:
- `bool(x)`: try `__bool__`, fall back to `__len__` (empty = False), fall back to True
- `x in container`: try `__contains__`, fall back to iterating via `__iter__`
- `str(x)`: try `__str__`, fall back to `__repr__`

Source: [Python Data Model](https://docs.python.org/3/reference/datamodel.html), [Built-in Types](https://docs.python.org/3/library/stdtypes.html)

### 3.2 Rust's Trait System

Rust uses **nominal typing**: a type implements a trait only by an explicit `impl Trait for Type` block (or `#[derive(Trait)]`). There's no implicit structural matching.

**How traits connect to language syntax:**

| Syntax         | Trait                  | Method called          |
|----------------|------------------------|------------------------|
| `x + y`        | `Add<Rhs>`             | `x.add(y)`             |
| `x == y`       | `PartialEq<Rhs>`       | `x.eq(&y)`             |
| `x < y`        | `PartialOrd<Rhs>`      | `x.partial_cmp(&y)`    |
| `x[i]`         | `Index<Idx>`           | `x.index(i)`           |
| `*x`           | `Deref`                | `x.deref()`            |
| `for x in c`   | `IntoIterator`         | `c.into_iter()`        |
| `format!("{}", x)` | `Display`          | `x.fmt(f)`             |
| `format!("{:?}", x)` | `Debug`          | `x.fmt(f)`             |
| `drop(x)`      | `Drop`                 | `x.drop()` (auto)      |
| `x(args)`      | `Fn`/`FnMut`/`FnOnce` | `x.call(args)`         |
| `x?`           | `Try` (nightly)        | `x.branch()`           |
| `x.await`      | `Future`               | `x.poll(cx)`           |

**Auto-deriving** is a key ergonomic feature: `#[derive(Debug, Clone, PartialEq, Eq, Hash)]` generates implementations by inspecting the type's structure at compile time. This bridges the gap between nominal typing (must explicitly implement) and structural typing (the compiler figures out the implementation).

**Orphan rule**: You can only implement a trait for a type if you own either the trait or the type (or both). This prevents conflicting implementations from different crates and ensures coherence -- there's exactly one `Display` impl for any type in any program.

**Deref coercion**: The compiler automatically inserts `deref()` calls to convert types. `&String` coerces to `&str` because `String` implements `Deref<Target = str>`. This enables `&String` to be passed anywhere `&str` is expected, making APIs that take `&str` work with both owned and borrowed strings.

Source: [Rust Reference: Special types and traits](https://doc.rust-lang.org/reference/special-types-and-traits.html), [Rust by Example: Operator Overloading](https://doc.rust-lang.org/rust-by-example/trait/ops.html)

### 3.3 JavaScript's Symbol-Based Protocols

JavaScript uses well-known symbols to connect user-defined types to built-in operations. This is a metaobject protocol approach.

**How it works in practice:**

```javascript
class Range {
  constructor(start, end) {
    this.start = start;
    this.end = end;
  }

  // Make iterable with for...of
  [Symbol.iterator]() {
    let current = this.start;
    const end = this.end;
    return {
      next() {
        if (current <= end) {
          return { value: current++, done: false };
        }
        return { done: true };
      }
    };
  }

  // Customize instanceof
  static [Symbol.hasInstance](instance) {
    return typeof instance.start === 'number'
        && typeof instance.end === 'number';
  }

  // Customize type coercion
  [Symbol.toPrimitive](hint) {
    if (hint === 'number') return this.end - this.start;
    if (hint === 'string') return `${this.start}..${this.end}`;
    return this.end - this.start; // default
  }
}
```

**Why symbols instead of string-named methods:**
- Guaranteed collision-free (unlike Python's `__dunder__` convention which is just a naming convention)
- Symbols don't appear in `Object.keys()` or `JSON.stringify()` -- they're invisible to code that doesn't know about them
- Can be used as truly private interface hooks

**The trade-off:** Python's dunder methods are discoverable (you can see them in `dir(obj)`) and readable (`__len__` obviously relates to length). JavaScript's symbols are opaque by design -- `Symbol.iterator` is a global constant you must know about.

Source: [Customizing ES6 via well-known symbols](https://2ality.com/2015/09/well-known-symbols-es6.html), [MDN Symbol](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Symbol)

### 3.4 Lua's Metatable System

Lua's metatables are the most dynamic protocol system of any language surveyed:

```lua
-- Create a "class" with operator overloading
local Vector = {}
Vector.__index = Vector

function Vector.new(x, y)
    return setmetatable({x = x, y = y}, Vector)
end

function Vector:__add(other)
    return Vector.new(self.x + other.x, self.y + other.y)
end

function Vector:__tostring()
    return string.format("(%g, %g)", self.x, self.y)
end

function Vector:__len()
    return math.sqrt(self.x^2 + self.y^2)
end
```

**Key differences from other protocol systems:**
- Metatables can be **changed at runtime** (`setmetatable`). A table can switch its "class" dynamically.
- The `__index` metamethod enables prototype chains (like JavaScript's prototype chain but explicit)
- `raw*` functions (`rawget`, `rawset`, `rawlen`, `rawequal`) bypass metatables entirely -- essential for implementing the metatable system itself
- Only tables and userdata can have metatables (not strings, numbers, etc. -- though strings share a metatable set by the string library)

### 3.5 Comparison of Protocol Approaches

| Aspect                | Python (dunder)     | Rust (trait)        | JS (Symbol)         | Lua (metamethod)    |
|-----------------------|---------------------|---------------------|---------------------|---------------------|
| **Dispatch**          | Runtime, duck-typed | Compile-time, nominal | Runtime, symbol-keyed | Runtime, metatable  |
| **Discovery**         | `dir(obj)`, visible | Type signature      | Hidden from `keys()` | `getmetatable(obj)` |
| **Collision safety**  | Convention only     | Orphan rule         | Guaranteed unique   | Convention only     |
| **Composability**     | Multiple inheritance | Trait bounds        | Prototype chain     | Single metatable    |
| **Fallback chains**   | Yes (bool->len)     | No                  | Limited             | __index chain       |
| **Overridable at runtime** | Yes            | No                  | Yes                 | Yes (setmetatable)  |
| **Static checking**   | Optional (typing)   | Full                | No                  | No                  |

---

## 4. Iterator and Collection Patterns

### 4.1 The Iterator Protocol Across Languages

**Python:**
- **Iterable**: object with `__iter__()` returning an iterator
- **Iterator**: object with `__iter__()` (returns self) and `__next__()` (returns next value or raises `StopIteration`)
- **Generator**: function with `yield`, automatically creates an iterator
- Evaluation: **lazy** for generators/itertools, **eager** for list comprehensions
- Key functions: `iter()`, `next()`, `zip()`, `enumerate()`, `map()`, `filter()` -- all return lazy iterators
- Materialization: `list()`, `tuple()`, `set()`, `dict()` consume iterators

**Rust:**
- **`Iterator` trait**: one required method: `fn next(&mut self) -> Option<Self::Item>`
- **`IntoIterator` trait**: enables `for x in collection` syntax
- 70+ provided combinator methods on the `Iterator` trait itself: `map`, `filter`, `fold`, `collect`, `zip`, `chain`, `enumerate`, `take`, `skip`, `flat_map`, `inspect`, `peekable`, `scan`, `sum`, `product`, `min`, `max`, `count`, `any`, `all`, `find`, `position`, `rev`, `cloned`, `copied`, etc.
- Evaluation: **always lazy** -- combinators return new iterator types, nothing executes until a consuming method (`collect`, `for_each`, `sum`, etc.) is called
- Zero-cost abstraction: iterator chains compile to the same machine code as hand-written loops
- `collect()` is generic over the output collection type -- `iter.collect::<Vec<_>>()` vs `iter.collect::<HashSet<_>>()` is determined by the `FromIterator` trait implementation

**JavaScript:**
- **Iterable protocol**: object with `[Symbol.iterator]()` returning an iterator
- **Iterator protocol**: object with `next()` returning `{ value, done }`
- `for...of` loop consumes iterables
- Generators (`function*` / `yield`) automatically implement the iterator protocol
- Evaluation: Array methods (`map`, `filter`) are **eager** (return new arrays). Generator-based iteration is **lazy**.
- No built-in lazy combinator chain (libraries like `iter-tools` provide this)

**Elixir:**
- **`Enumerable` protocol**: types implement `reduce/3`, `count/1`, `member?/2`
- **`Enum` module**: eager operations on enumerables (`Enum.map`, `Enum.filter`, `Enum.reduce`)
- **`Stream` module**: lazy operations that return `%Stream{}` structs
- Lazy/eager split is at the module level: `Enum.map` is eager, `Stream.map` is lazy

**Ruby:**
- **`Enumerable` mixin**: include it and define `each`, get 50+ methods
- **`Enumerator` class**: lazy evaluation via `Enumerator::Lazy`
- `.lazy` method converts eager enumeration to lazy: `(1..Float::INFINITY).lazy.select(&:odd?).take(5)`

### 4.2 Lazy vs Eager Evaluation Trade-offs

| Approach      | Pros                                       | Cons                                      | Languages                |
|---------------|--------------------------------------------|--------------------------------------------|--------------------------|
| Always lazy   | Memory efficient, composable, fused loops  | Debugging harder, must explicitly consume  | Rust, Haskell            |
| Always eager  | Simple mental model, immediate results     | Memory overhead, wasteful for large data   | Ruby (default), JS arrays|
| Dual API      | Developer chooses per use case             | API surface doubles, learning curve        | Elixir (Enum/Stream), Python (list comp/generator) |

### 4.3 Collection Type Hierarchies

**Python `collections.abc`:**
```
Iterable
├── Iterator
├── Reversible
├── Collection
│   ├── Sequence (list, tuple, str, range, bytes)
│   │   └── MutableSequence (list, bytearray)
│   ├── Set (frozenset)
│   │   └── MutableSet (set)
│   └── Mapping (types.MappingProxyType)
│       └── MutableMapping (dict)
├── Generator
├── AsyncIterable
│   └── AsyncIterator
│       └── AsyncGenerator
```

**Rust collections:**
All collection types implement `IntoIterator` (for `for` loops), `FromIterator` (for `.collect()`), `Extend` (for `.extend()`). Common collections: `Vec`, `VecDeque`, `LinkedList`, `HashMap`, `BTreeMap`, `HashSet`, `BTreeSet`, `BinaryHeap`.

---

## 5. String API Design

### 5.1 What's Built-in vs Library

| Language    | String type(s)                | Built-in methods       | Library methods          |
|-------------|-------------------------------|------------------------|--------------------------|
| Python      | `str` (immutable, Unicode)    | ~45 methods on str     | `re`, `string`, `textwrap` |
| Rust        | `str` (slice), `String` (owned) | Methods on both types | `regex` (third-party)    |
| JavaScript  | `string` primitive + `String` object | ~30 prototype methods | `Intl` for locale-aware |
| Go          | `string` (immutable bytes)    | None on the type       | `strings`, `strconv`, `unicode` packages |
| Lua         | `string` (immutable bytes)    | None (no methods)      | `string` library         |
| Ruby        | `String` (mutable, encoding-aware) | ~170+ methods       | Very little in stdlib    |
| Elixir      | Binary (immutable, UTF-8)     | None on the type       | `String` module          |

### 5.2 Unicode Handling Approaches

**Python:** `str` is a sequence of Unicode code points. Indexing gives code points, not bytes. `len("cafe\u0301")` = 5 (separate accent). `unicodedata.normalize()` handles normalization.

**Rust:** `String`/`str` are always valid UTF-8. No direct indexing (because byte index != character index). `.chars()` iterates code points, `.bytes()` iterates bytes. Deliberate friction: the type system prevents casual misuse of string indexing.

**Go:** `string` is a read-only byte slice. The `range` loop iterates over runes (Unicode code points). The `rune` type is an alias for `int32`. The `unicode/utf8` package provides explicit encoding/decoding.

**JavaScript:** Strings are UTF-16 internally. Characters outside the BMP (like emoji) are represented as surrogate pairs. `string.length` counts UTF-16 code units, not code points. `Array.from(str)` or the spread operator `[...str]` gives code points.

**Lua:** Strings are arbitrary byte sequences. The `string` library operates on bytes. The `utf8` library (added in 5.3) provides code point iteration and validation but not normalization.

### 5.3 String Interpolation Mechanisms

| Language    | Syntax                           | Compile-time?  | Implementation              |
|-------------|----------------------------------|----------------|-----------------------------|
| Python      | `f"Hello {name}"`               | No (runtime)   | Compiled to format calls    |
| Rust        | `format!("Hello {}", name)`     | Yes (verified) | `format_args!` macro, stack-allocated |
| JavaScript  | `` `Hello ${name}` ``           | No (runtime)   | Template literal evaluation |
| Go          | `fmt.Sprintf("Hello %s", name)` | No (runtime)   | Printf-style format string  |
| Ruby        | `"Hello #{name}"`               | No (runtime)   | Evaluated in string context |
| Elixir      | `"Hello #{name}"`               | No (runtime)   | Compiled to binary concat   |
| Lua         | `string.format("Hello %s", name)` | No (runtime) | Printf-style               |

Rust's approach is notable: `format_args!` is a compiler built-in that validates the format string at compile time and creates a stack-allocated `fmt::Arguments` value with zero heap allocation. All other formatting macros (`println!`, `write!`, `format!`) are built on top of `format_args!`.

---

## 6. I/O Primitives Design

### 6.1 Where I/O Lives

| Language    | Stdio access              | File I/O                    | Level          |
|-------------|---------------------------|-----------------------------|----------------|
| Python      | `print()` (builtin), `sys.stdin/stdout/stderr` | `open()` (builtin) | Builtin        |
| Rust        | `println!` (macro), `std::io::stdin()/stdout()` | `std::fs::File::open()` | Stdlib     |
| Go          | `fmt.Println()` (stdlib)  | `os.Open()` (stdlib)        | Stdlib         |
| JavaScript  | `console.log()` (host)    | `fs.readFile()` (Node)      | Runtime/host   |
| Lua         | `print()` (basic lib), `io.write()` | `io.open()` (stdlib) | Stdlib         |
| Ruby        | `puts` (Kernel), `$stdout` | `File.open()` (builtin)     | Builtin        |
| Elixir      | `IO.puts()` (stdlib)      | `File.read()` (stdlib)      | Stdlib         |

### 6.2 Trait/Interface-Based I/O Design

**Rust's approach** is the most structured: the `Read` and `Write` traits abstract I/O operations. Anything implementing `Read` can be read from (files, sockets, byte buffers, stdin). Anything implementing `Write` can be written to.

```
Read trait: read(&mut self, buf: &mut [u8]) -> io::Result<usize>
  └── implemented by: File, TcpStream, Stdin, &[u8], Cursor<T>, BufReader<R>

Write trait: write(&mut self, buf: &[u8]) -> io::Result<usize>
  └── implemented by: File, TcpStream, Stdout, Vec<u8>, BufWriter<W>

BufRead trait: fill_buf, read_line, lines
  └── implemented by: BufReader<R>, Cursor<T>, StdinLock
```

`BufReader` and `BufWriter` wrap any `Read`/`Write` implementor to add buffering. This is the decorator pattern applied to I/O traits.

**Go's approach** is similar: `io.Reader` and `io.Writer` interfaces with single methods. Composition via `io.MultiReader`, `io.TeeReader`, `bufio.Scanner`, etc.

**Python's approach**: file objects implement `read()`, `write()`, `readline()`, `__iter__()`. The `io` module provides `BufferedReader`, `BufferedWriter`, `TextIOWrapper` for layered I/O.

### 6.3 Key Design Decision: Buffering

All languages must decide: is I/O buffered by default?

- **Python:** `print()` is line-buffered to terminals, block-buffered to pipes. `open()` returns buffered file objects by default.
- **Rust:** `println!` locks and flushes stdout per call. `BufWriter` is opt-in for file I/O.
- **Go:** `fmt.Println` writes directly. `bufio.Writer` is opt-in.
- **Lua:** `print()` flushes after each call. `io.write()` is buffered.

---

## 7. Concurrency Primitive Design

### 7.1 Language-Level vs Library-Level Concurrency

| Language    | Language-level                    | Stdlib                           | Third-party           |
|-------------|-----------------------------------|----------------------------------|-----------------------|
| Go          | `go`, `chan`, `select`            | `sync`, `context`                | --                    |
| Erlang/Elixir | `spawn`, `send`, `receive`    | `GenServer`, `Supervisor` (OTP) | --                    |
| Rust        | `async`/`await`, `Future` trait  | `std::thread`, `std::sync`       | `tokio`, `async-std`  |
| Python      | `async`/`await`                  | `asyncio`, `threading`, `multiprocessing` | `trio`, `anyio` |
| JavaScript  | `Promise`, `async`/`await`       | --                               | --                    |
| Lua         | `coroutine.create/resume/yield`  | --                               | `copas`, `lanes`      |
| Ruby        | `Fiber` (coroutines)             | `Thread`, `Ractor`               | `async`, `celluloid`  |

### 7.2 Design Philosophy Differences

**Go: Concurrency as a first-class citizen.** Goroutines and channels are syntax-level constructs. The runtime includes a full M:N scheduler. `go func()` is as natural as calling a function. Channels are typed, first-class values that can be passed around, stored in data structures, and composed with `select`.

**Erlang/Elixir: Everything is a process.** The BEAM VM provides preemptive scheduling, per-process heaps, and per-process garbage collection. Processes share nothing. Supervision trees handle failure. The language is designed around processes the way object-oriented languages are designed around objects.

**Rust: Syntax without runtime.** `async`/`await` and `Future` are in the language, but the event loop (executor) is a library. This separation means the same language serves embedded systems (no allocator, no threads) and network servers (tokio provides a multi-threaded work-stealing executor). The cost: you must choose and configure a runtime.

**Python: Async as a library feature.** `async`/`await` syntax was added to the language (PEP 492), but the event loop (`asyncio`) is a stdlib module. The GIL prevents true parallelism in threads. `multiprocessing` provides process-based parallelism but with serialization overhead.

**Lua: Cooperative coroutines as building blocks.** Lua provides only coroutines -- explicit, cooperative yield points. This is sufficient for implementing event loops, state machines, and cooperative multitasking but provides no parallelism. The host application handles true concurrency.

### 7.3 Message Passing vs Shared Memory

| Approach         | Languages                  | Mechanism                        | Safety model           |
|------------------|---------------------------|----------------------------------|------------------------|
| Message passing  | Erlang/Elixir, Go          | Mailboxes / channels             | No shared state        |
| Shared memory    | Rust, C, C++, Java         | Mutex/RwLock/Atomic              | Locks / ownership      |
| GIL-protected    | Python (CPython)           | Single GIL per interpreter       | Coarse lock            |
| Event loop       | JavaScript, Python asyncio | Single-threaded async            | No concurrency (cooperative) |
| Hybrid           | Go (channels + sync.Mutex) | Prefer channels, allow mutexes   | Convention             |

Rust is unique in enforcing thread safety at compile time through `Send` and `Sync` marker traits. A type that isn't `Send` can't be moved to another thread. A type that isn't `Sync` can't be shared between threads. This eliminates data races at the type system level.

---

## 8. Error Handling Primitive Design

### 8.1 Exception-Based

**Python, Ruby, JavaScript** use exceptions: `try`/`except`/`finally` (Python), `begin`/`rescue`/`ensure` (Ruby), `try`/`catch`/`finally` (JS).

Characteristics:
- Errors propagate automatically up the call stack
- Errors are objects with inheritance hierarchies
- Cost: stack unwinding on throw, try block setup
- Separation of normal flow from error flow
- Uncaught exceptions crash the program

**Python exception hierarchy (partial):**
```
BaseException
├── SystemExit
├── KeyboardInterrupt
├── GeneratorExit
└── Exception
    ├── StopIteration
    ├── ArithmeticError (ZeroDivisionError, OverflowError)
    ├── LookupError (IndexError, KeyError)
    ├── OSError (FileNotFoundError, PermissionError)
    ├── ValueError
    ├── TypeError
    └── RuntimeError
```

### 8.2 Value-Based

**Rust** uses `Result<T, E>` and `Option<T>` -- algebraic data types, not special runtime constructs.

```rust
enum Result<T, E> { Ok(T), Err(E) }
enum Option<T> { Some(T), None }
```

The `?` operator is syntax sugar for early return on `Err`/`None`:
```rust
fn read_config() -> Result<Config, io::Error> {
    let contents = fs::read_to_string("config.toml")?;  // returns Err if fails
    let config = toml::from_str(&contents)?;             // returns Err if fails
    Ok(config)
}
```

`panic!` exists for truly unrecoverable errors (like bounds violations, assertion failures). It's deliberately separate from `Result` -- the type signature tells you whether a function can fail normally (returns `Result`) or should never fail (returns a bare value).

**Go** uses multiple return values:
```go
result, err := doSomething()
if err != nil {
    return fmt.Errorf("failed to do something: %w", err)
}
```

`error` is an interface: `interface { Error() string }`. Any type implementing `Error() string` is an error. This is maximally simple but verbose.

### 8.3 Protected Calls (Lua)

Lua's `pcall` and `xpcall` are the functional equivalent of try/catch:
```lua
local ok, result = pcall(dangerous_function, arg1, arg2)
-- ok = true: result is the return value
-- ok = false: result is the error value

local ok, result = xpcall(dangerous_function, error_handler, arg1)
-- error_handler receives the error and can transform it
```

`error()` throws any Lua value (strings, tables, numbers -- anything). `pcall` catches it. This is essentially `Result` semantics implemented as a function rather than a type system feature.

### 8.4 Process-Level Error Handling (Erlang/Elixir)

Erlang's "let it crash" philosophy: individual processes are expected to fail. Supervisor processes monitor worker processes and restart them on failure.

```elixir
# Match on tagged tuples for expected errors
case File.read("config.toml") do
  {:ok, contents} -> parse(contents)
  {:error, :enoent} -> use_defaults()
  {:error, reason} -> raise "Unexpected: #{reason}"
end

# Supervisors handle unexpected crashes
children = [
  {MyWorker, arg1},
  {AnotherWorker, arg2}
]
Supervisor.start_link(children, strategy: :one_for_one)
```

### 8.5 Design Trade-offs Summary

| Approach          | Boilerplate | Composability | Static safety | Performance     |
|-------------------|-------------|---------------|---------------|-----------------|
| Exceptions        | Low         | Medium        | Low           | Zero-cost happy path, expensive throw |
| Result/Option     | Medium      | High (? operator) | High       | Zero-cost       |
| Multiple returns  | High        | Low           | Low           | Zero-cost       |
| pcall/xpcall      | Medium      | Medium        | None          | Function call overhead |
| Process isolation | Very low    | High          | Runtime only  | Process overhead |

---

## 9. Synthesis: Design Patterns for a New Language

### 9.1 What the Survey Reveals

Languages cluster around a few successful patterns:

**Pattern A: Protocol dispatch (Python, JavaScript, Lua)**
Built-in operations dispatch to user-definable methods (dunder methods, Symbols, metamethods). The language provides a fixed set of "extension points" that types can opt into. This enables open-ended extensibility -- any type can behave like a built-in type.

**Pattern B: Trait/typeclass dispatch (Rust, Haskell, Elixir)**
Operations are defined by traits/typeclasses. Types must explicitly implement them. The compiler verifies implementations at compile time. This provides static guarantees about what operations a type supports.

**Pattern C: Mixin inheritance (Ruby)**
Behaviors are composed by including modules. Define `each`, get 50 methods. Define `<=>`, get 6 comparison methods. This is high-leverage -- a small amount of user code unlocks a large surface area of derived functionality.

**Pattern D: Minimal + extension (Go, Lua)**
The language provides a small, fixed set of built-in operations. Everything else is functions operating on values. No operator overloading, no custom iteration protocols (Go now has range-over-func). Simplicity over extensibility.

### 9.2 Common Successful Patterns

1. **Separate "always available" from "import required".** Every language has this boundary. The question is where to draw it.

2. **Iterator protocols are nearly universal.** Every modern language needs a way for user types to participate in `for` loops and collection operations.

3. **String formatting is a solved problem.** All languages need string interpolation or format strings. Making this a built-in (not library) feature significantly improves ergonomics.

4. **Error handling shapes everything.** The choice between exceptions, Result types, and multiple returns influences every function signature in the language. This must be decided early and committed to.

5. **The pipe operator pattern improves API design.** Elixir's `|>` and its influence on "data-first" function signatures is a successful innovation that other languages are adopting (Hack, F#, proposed for JavaScript).

6. **Prelude size should match deployment model.** Embedded languages need minimal preludes. Application languages benefit from larger ones. The prelude should contain what nearly every program needs and nothing else.

---

## Sources

- [Python Data Model](https://docs.python.org/3/reference/datamodel.html)
- [Python Built-in Types](https://docs.python.org/3/library/stdtypes.html)
- [PEP 594: Removing dead batteries](https://peps.python.org/pep-0594/)
- [Batteries included and impact on language popularity](https://szabgab.com/batteries-included)
- [A Look at the Design of Lua](https://cacm.acm.org/research/a-look-at-the-design-of-lua/)
- [Small is Beautiful: the design of Lua](https://web.stanford.edu/class/ee380/Abstracts/100310-slides.pdf)
- [Go at Google: Language Design](https://go.dev/talks/2012/splash.article)
- [Golang: Some batteries not included](https://yolken.net/blog/golang-batteries-not-included)
- [RFC 0503: Prelude Stabilization](https://rust-lang.github.io/rfcs/0503-prelude-stabilization.html)
- [RFC 3114: Prelude 2021](https://rust-lang.github.io/rfcs/3114-prelude-2021.html)
- [Rust 2024 Prelude changes](https://doc.rust-lang.org/edition-guide/rust-2024/prelude.html)
- [Haskell Wiki: Prelude](https://wiki.haskell.org/Prelude)
- [No import of Prelude](https://wiki.haskell.org/No_import_of_Prelude)
- [Implicit Imports in Scala](https://www.baeldung.com/scala/implicit-imports)
- [Customizing ES6 via well-known symbols](https://2ality.com/2015/09/well-known-symbols-es6.html)
- [MDN: Symbol](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Symbol)
- [Rust Reference: Special types and traits](https://doc.rust-lang.org/reference/special-types-and-traits.html)
- [Tour of Rust's Standard Library Traits](https://github.com/pretzelhammer/rust-blog/blob/master/posts/tour-of-rusts-standard-library-traits.md)
- [Rust by Example: Operator Overloading](https://doc.rust-lang.org/rust-by-example/trait/ops.html)
- [std::prelude - Rust](https://doc.rust-lang.org/std/prelude/index.html)
- [std::io - Rust](https://doc.rust-lang.org/std/io/index.html)
- [Rust Book: Error Handling](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
- [Rust Book: Iterators](https://doc.rust-lang.org/book/ch13-02-iterators.html)
- [Elixir Kernel module](https://hexdocs.pm/elixir/Kernel.html)
- [Elixir: Enumerables and Streams](https://elixir-lang.org/getting-started/enumerables-and-streams.html)
- [Ruby Kernel module](https://docs.ruby-lang.org/en/master/Kernel.html)
- [Go builtin package](https://pkg.go.dev/builtin)
- [Lua 5.4 Reference Manual](https://www.lua.org/manual/5.4/manual.html)
- [Node.js CommonJS modules](https://nodejs.org/api/modules.html)
- [Python inspect module](https://docs.python.org/3/library/inspect.html)
- [Rust std::any](https://doc.rust-lang.org/std/any/index.html)
- [MDN: Meta programming](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Meta_programming)
- [PEP 492: Coroutines with async and await](https://peps.python.org/pep-0492/)
- [Concurrency in Go vs Erlang](https://dev.to/pancy/concurrency-in-go-vs-erlang-595a)
- [String interpolation - Wikipedia](https://en.wikipedia.org/wiki/String_interpolation)
- [Comparison of programming languages (strings)](https://en.wikipedia.org/wiki/Comparison_of_programming_languages_(strings))
