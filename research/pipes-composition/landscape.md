# Pipe Operators Across Languages

A deep survey of pipe operators and functional composition mechanisms across 15+
languages, covering syntax, semantics, argument position conventions, and how
each approach shapes API design.

## Historical Origin

The pipe operator `|>` was first introduced by **Tobias Nipkow** in **May 1994**
for Isabelle/ML, with obvious semiotic inspiration from Unix pipes. The symbol
emerged from a mail exchange between Nipkow, Larry Paulson, and Marius Wenzel.
It was added to the F# standard library in 2003, where it became the language's
signature operator. OCaml adopted it as a built-in primitive in version 4.01
(2013), bringing the operator back to the language family that inspired F#.

Sources:
- [The origin of the pipeline operator](https://batsov.com/articles/2025/05/22/the-origin-of-the-pipeline-operator/)
- [Which language first introduced the pipe operator?](https://elixirforum.com/t/which-language-first-introduced-the-pipe-operator/16791)
- [A Brief History of Pipeline Operator](http://mamememo.blogspot.com/2019/06/a-brief-history-of-pipeline-operator.html)

---

## 1. Unix Shell — `|`

**The ancestor.** All language-level pipes descend from Douglas McIlroy's concept.

### How it works

The `|` operator connects the stdout of one process to the stdin of the next.
Implemented via the `pipe()` system call, which returns two file descriptors (read
end and write end). The shell `fork()`s child processes for each command, wires
stdout of the left process to the write end and stdin of the right process to the
read end, then `exec()`s each command.

```sh
cat access.log | grep 404 | awk '{print $7}' | sort | uniq -c | sort -rn
```

### Key characteristics

- **Concurrent execution**: processes run simultaneously. The kernel buffers data
  between them (originally 504 bytes in V3 Unix, 4096 in V4, typically 64KB on
  modern Linux via `pipe_max_size`).
- **Byte streams**: pipes carry unstructured bytes. Each program parses/emits its
  own text format. This is both a strength (universal interface) and a weakness
  (no type safety).
- **stderr is separate**: only stdout flows through `|`. stderr goes to the
  terminal unless explicitly redirected (`2>&1 |` or `|&` in bash).
- **Exit codes**: by default, the pipeline's exit code is the exit code of the
  last command. `set -o pipefail` makes it the exit code of the first failure.

### History

McIlroy proposed the concept in 1964: "We should have some ways of connecting
programs like garden hose." Ken Thompson implemented `pipe()` on January 15, 1973,
in "one feverish night." It shaped the Unix philosophy: small programs that do one
thing, connected by pipes.

Sources:
- [Pipe: How the System Call That Ties Unix Together Came About](https://thenewstack.io/pipe-how-the-system-call-that-ties-unix-together-came-about/)
- [How are Unix pipes implemented?](https://toroid.org/unix-pipe-implementation)
- [History of pipes concept](https://softpanorama.org/Scripting/Piporama/history.shtml)

---

## 2. Elixir — `|>`

**First-argument insertion, data-first by convention.**

### Semantics

`x |> f(a, b)` rewrites at compile time to `f(x, a, b)`. The left-hand value
becomes the first argument of the right-hand function call.

```elixir
# Without pipes (inside-out)
Enum.sum(Enum.filter(Enum.map(1..100_000, &(&1 * 3)), &rem(&1, 2) != 0))

# With pipes (linear, left-to-right)
1..100_000
|> Enum.map(&(&1 * 3))
|> Enum.filter(&(rem(&1, 2) != 0))
|> Enum.sum()
```

### API design consequences

The entire Elixir standard library places the "subject" as the first argument.
`Enum.map(enumerable, fun)`, `String.split(string, pattern)`, `Map.put(map, key, value)`.
This convention is pervasive and self-reinforcing: any library that breaks it
becomes awkward to pipe.

### Anonymous functions in pipes

Bare lambdas cannot appear directly on the right side of `|>`. Two solutions:

1. **`then/2`** (Elixir 1.12+): `x |> then(fn val -> transform(val) end)` — the
   lambda receives the piped value and its return value continues the pipeline.
2. **`tap/2`** (Elixir 1.12+): `x |> tap(fn val -> IO.inspect(val) end)` — the
   lambda runs as a side effect, but the original value passes through unchanged.
   Essential for debugging.

### Capture operator

The `&` capture operator creates anonymous functions: `&String.upcase/1`. With
pipes: `"hello" |> (&String.upcase/1).()` — though `then/2` is preferred.

Sources:
- [Elixir Pipe Operator](https://elixirschool.com/en/lessons/basics/pipe_operator)
- [Enumerables and Streams](https://elixir-lang.org/getting-started/enumerables-and-streams.html)
- [tap() & then()](https://blixtdev.com/two-useful-elixir-functions-you-may-not-know/)
- [elixir_express pipeline operator](https://github.com/chrismccord/elixir_express/blob/master/basics/06_pipeline_operator.md)

---

## 3. F# / OCaml — `|>`, `>>`, `<|`, `<<`

**The language that popularized pipes. Four operators covering both piping and
composition in both directions.**

### Forward pipe `|>`

```
let (|>) x f = f x
```

Type: `'a -> ('a -> 'b) -> 'b`

Places the argument before the function. For multi-parameter functions with
currying, the piped value becomes the **last** argument:

```fsharp
[1; 2; 3]
|> List.map (fun x -> x * 2)
|> List.filter (fun x -> x > 2)
|> List.sum
```

### Backward pipe `<|`

```
let (<|) f x = f x
```

Type: `('a -> 'b) -> 'a -> 'b`

Eliminates parentheses. `printf "result: %d" <| 1 + 2` instead of
`printf "result: %d" (1 + 2)`. Equivalent to Haskell's `$`.

### Forward composition `>>`

```
let (>>) f g x = g (f x)
```

Type: `('a -> 'b) -> ('b -> 'c) -> 'a -> 'c`

Composes two functions into a new function. No value needed — operates on
functions, not values:

```fsharp
let add1ThenDouble = (+) 1 >> (*) 2
add1ThenDouble 3  // 8
```

### Backward composition `<<`

```
let (<<) f g x = f (g x)
```

Equivalent to Haskell's `.` operator. Reads right-to-left.

### Type inference interaction

F#'s type inference flows left-to-right. The pipe operator aligns data flow with
inference flow, meaning the compiler already knows the type of the piped value
when it reaches the function. This is a major practical advantage: the compiler
produces better error messages and needs fewer type annotations. This same
property motivated BuckleScript/ReScript's move to data-first APIs.

### `|>` vs `>>`

- Use `|>` when you have a concrete value to transform through a series of steps
- Use `>>` when building a reusable function from smaller functions (pointfree style)
- `|>` requires a value on the left; `>>` requires functions on both sides

Sources:
- [Function associativity and composition](https://fsharpforfunandprofit.com/posts/function-composition/)
- [When to Use Pipelining vs. Composition Operators in F#](https://spin.atomicobject.com/fsharp-operators-pipeline-composition/)
- [F# Pipe Forward and Pipe Backward](https://theburningmonk.com/2011/09/fsharp-pipe-forward-and-pipe-backward/)

---

## 4. Haskell — `$`, `.`, `&`

**No single "pipe operator" — instead, a family of application and composition
operators that interact with pervasive currying.**

### `$` — low-precedence application

```haskell
($) :: (a -> b) -> a -> b    -- infixr 0
f $ x = f x
```

Exists purely to eliminate parentheses. `putStrLn $ show $ 1 + 2` instead of
`putStrLn (show (1 + 2))`. Right-associative, lowest precedence (0).

### `.` — function composition

```haskell
(.) :: (b -> c) -> (a -> b) -> a -> c    -- infixr 9
(f . g) x = f (g x)
```

The workhorse of pointfree Haskell. `map (show . negate . abs)` composes three
functions without naming the argument. Reads right-to-left.

### `&` — reverse application (the "pipe")

```haskell
(&) :: a -> (a -> b) -> b    -- infixl 1
x & f = f x
```

Added in `base-4.8.0.0` (GHC 7.10, 2015). Defined in `Data.Function`. This is
the closest thing Haskell has to Elixir/F#'s `|>`:

```haskell
42 & show & reverse & putStrLn
-- prints "24"
```

Precedence 1 (higher than `$` at 0) so `&` can nest inside `$`.

### Interaction with currying

Every Haskell function is automatically curried. `map (+1)` partially applies
`map` to `(+1)`, yielding a function `[Int] -> [Int]`. This means `.` composes
cleanly:

```haskell
processList = filter even . map (+1) . sort
```

The pipe `&` works with curried functions too:

```haskell
[3,1,2] & sort & map (+1) & filter even
```

### The `Flow` library

For those who want a full pipe toolkit, the `flow` package on Hackage provides
`|>` (forward pipe), `<|` (backward pipe), `>>` (forward compose), `<<` (backward
compose) — mirroring F# exactly.

Sources:
- [Data.Function](https://hackage.haskell.org/package/base/docs/Data-Function.html)
- [Function composition and $ operator](https://jstolarek.github.io/posts/2012-03-25-function-composition-and-dollar-operator-in-haskell.html)
- [Flow package](https://hackage.haskell.org/package/flow/docs/Flow.html)

---

## 5. R / tidyverse — `%>%` and `|>`

**Two competing pipes: the magrittr pipe that created the tidyverse revolution,
and the base R pipe added a decade later.**

### magrittr `%>%` (2014)

Created by Stefan Milton Bache. Became the backbone of the tidyverse (dplyr,
tidyr, ggplot2). Uses a dot `.` as the placeholder for the piped value.

```r
mtcars %>%
  filter(cyl == 4) %>%
  mutate(kpl = mpg * 0.425) %>%
  select(model, kpl) %>%
  arrange(desc(kpl))
```

Key features:
- **Dot placeholder**: `x %>% f(a, .)` becomes `f(a, x)`. The dot can appear
  anywhere, including nested calls and multiple times.
- **Implicit first argument**: without a dot, the value goes to the first argument.
  `x %>% f(a)` becomes `f(x, a)`.
- **Bare function names**: `x %>% sqrt` works (no parentheses needed).
- **Lambda shorthand**: `x %>% { . + 1 }` creates an inline anonymous function.

### Base R `|>` (R 4.1+, 2021)

Added to avoid the magrittr dependency. Simpler implementation.

```r
mtcars |>
  subset(cyl == 4) |>
  transform(kpl = mpg * 0.425)
```

Key differences from `%>%`:
- **Underscore placeholder** `_` (R 4.2+): `x |> f(a, b = _)` — but only in
  named argument positions, and only once per call.
- **Requires parentheses**: `x |> sqrt()` not `x |> sqrt`.
- **No multi-use placeholder**: can't use `_` multiple times in one expression.
- **Faster**: no function call overhead (syntactic transformation, not runtime).
- **No dependency**: built into the language.

### Impact on API design

The tidyverse convention is data-first: the first argument of every function is
the data frame being transformed. This convention predates the pipe but was
reinforced by it. The entire R data science ecosystem follows this pattern.

Sources:
- [Differences between the base R and magrittr pipes](https://tidyverse.org/blog/2023/04/base-vs-magrittr-pipe/)
- [Comparing pipes: Base-R vs magrittr](https://albert-rapp.de/posts/31_pipes_compared/31_pipes_compared)
- [R for Data Science: Pipes](https://r4ds.had.co.nz/pipes.html)

---

## 6. Rust — Method Chaining via `self`

**No pipe operator. Instead, method chaining through ownership/borrowing, iterator
combinators, and the builder pattern.**

### Iterator combinators

Rust's `Iterator` trait provides `.map()`, `.filter()`, `.fold()`, `.collect()`,
etc. These chain because each adapter returns a new iterator (lazy evaluation):

```rust
let result: Vec<i32> = (1..=100)
    .map(|x| x * 3)
    .filter(|x| x % 2 != 0)
    .collect();
```

### Ownership and chaining

The choice of `iter()`, `iter_mut()`, or `into_iter()` determines how elements
are accessed:

- `iter()` — borrows elements (`&T`), collection survives
- `iter_mut()` — mutable borrows (`&mut T`), collection survives
- `into_iter()` — takes ownership (`T`), collection consumed

Method chaining works because each combinator takes `self` (by value for
iterators, which are lightweight) and returns a new type implementing `Iterator`.

### Builder pattern

Rust uses method chaining extensively for builders:

```rust
let config = Config::builder()
    .host("localhost")
    .port(8080)
    .timeout(Duration::from_secs(30))
    .build()?;
```

Each method takes `self` (or `&mut self`) and returns `Self`, enabling the chain.

### Zero-cost abstraction

Rust's iterator chains compile to the same machine code as hand-written loops.
The compiler monomorphizes generic types and inlines the combinators, eliminating
all abstraction overhead. This is verified by examining assembly output —
`.map().filter().collect()` produces the same code as a `for` loop with `if` and
`push`.

### Why no `|>` in Rust?

Method syntax (`value.method()`) already provides left-to-right reading order.
UFCS would conflict with Rust's trait system and method resolution. The community
has discussed `|>` but consensus is that method chaining plus the `?` operator
for error propagation covers the use cases.

Sources:
- [Processing a Series of Items with Iterators](https://doc.rust-lang.org/book/ch13-02-iterators.html)
- [Iterator trait](https://doc.rust-lang.org/std/iter/trait.Iterator.html)
- [When will Rust adopt |>?](https://internals.rust-lang.org/t/when-will-rust-adopt-the-syntax-of-pipline-operator-in-ocaml/17191)

---

## 7. JavaScript — TC39 Proposal `|>` (Hack-style)

**Stage 2. Hack-style with `%` placeholder. Has been in committee since 2017.**

### The proposal

```javascript
// Before
console.log(chalk.dim(`$ ${Object.keys(envars).map(e => `${e}=${envars[e]}`).join(' ')}`));

// After (Hack pipes)
Object.keys(envars)
  .map(e => `${e}=${envars[e]}`)
  .join(' ')
  |> `$ ${%}`
  |> chalk.dim(%, 'node', args.join(' '))
  |> console.log(%);
```

### Hack-style vs F#-style

The proposal went through years of debate between two approaches:

**F#-style**: `value |> func` — pipes to unary functions, no placeholder needed.
Rejected by TC39 twice due to:
- Memory performance concerns from browser engine implementors
- Conflicts with `await` syntax
- Fear of ecosystem bifurcation between OOP and FP styles

**Hack-style** (current): `value |> expression(%)` — uses `%` as an explicit
placeholder (provisional token). The piped value can appear anywhere in the
expression. Advantages:
- Works with any expression: function calls, arithmetic, template literals,
  `await`, `yield`
- No special cases for `await`/`yield`
- Placeholder is immutable and lexically scoped per pipe step
- Already implemented in Babel v7.15+

### Current status (Stage 2, as of 2026)

The champion group (J.S. Choi, James DiGioia, Ron Buckton, Tab Atkins-Bittner)
continues development. The placeholder token `%` is provisional — it may change.
Stage 2 means the committee expects the feature to be developed and eventually
included, but significant design work remains.

Sources:
- [TC39 Proposal: Pipeline Operator](https://github.com/tc39/proposal-pipeline-operator)
- [proposal-pipeline-operator README](https://github.com/tc39/proposal-pipeline-operator/blob/main/README.md)

---

## 8. Clojure — Threading Macros

**Not operators but macros. The macro system enables multiple pipe variants with
different argument positions — something most languages can't do.**

### `->` (thread-first)

Inserts the threaded value as the **first** argument of each form:

```clojure
(-> person
    (assoc :hair-color :gray)
    (update :age inc))
;; expands to: (update (assoc person :hair-color :gray) :age inc)
```

Used for data structure operations (`assoc`, `update`, `dissoc`, `get`).

### `->>` (thread-last)

Inserts the threaded value as the **last** argument:

```clojure
(->> (range 10)
     (filter odd?)
     (map #(* % %))
     (reduce +))
;; expands to: (reduce + (map #(* % %) (filter odd? (range 10))))
```

Used for sequence operations (`map`, `filter`, `reduce`, `into`).

### `as->` (thread-as — arbitrary position)

Binds the threaded value to a name, allowing it anywhere:

```clojure
(as-> [:foo :bar] v
  (map name v)
  (first v)
  (.substring v 1))
```

### `some->` and `some->>` (nil short-circuit)

Thread the value through forms but **short-circuit to nil** if any step returns
nil. Essential for safe Java interop:

```clojure
(some-> order :customer :address :city .toUpperCase)
;; returns nil if any step is nil, no NullPointerException
```

### `cond->` and `cond->>` (conditional threading)

Thread through forms conditionally — each step has a test predicate:

```clojure
(cond-> val
  (string? val) clojure.string/upper-case
  (> (count val) 5) (subs 0 5))
```

### Why macros matter

Because these are macros, not operators, Clojure can offer all these variants
without language-level syntax changes. Any library can define new threading macros.
This is impossible in languages where the pipe is a fixed operator.

Sources:
- [Clojure Threading Macros Guide](https://clojure.org/guides/threading_macros)
- [Threading with Style](https://stuartsierra.com/2018/07/06/threading-with-style/)
- [Lesser known Clojure variants of threading macro](https://www.spacjer.com/blog/2015/11/09/lesser-known-clojure-variants-of-threading-macro/)

---

## 9. Nim — Uniform Function Call Syntax (UFCS)

**Any function can be called as a method. No special pipe operator needed.**

### How it works

`f(a, b)` and `a.f(b)` are interchangeable. The compiler rewrites `a.f(b)` to
`f(a, b)` when `a`'s type has no method named `f`.

```nim
proc double(x: int): int = x * 2
proc add(x, y: int): int = x + y

let result = 5.double().add(3)  # same as add(double(5), 3) = 13
```

### Property-like access

Functions with a single parameter can be called without parentheses:

```nim
proc len(s: string): int = s.length
echo "hello".len  # 5
```

### Chaining as piping

UFCS provides the same left-to-right readability as pipes:

```nim
# instead of: sort(filter(map(data, transform), predicate))
data.map(transform).filter(predicate).sort()
```

### Advantage over pipes

UFCS is more general than a pipe operator — it works for any function, at any
arity, without requiring the library to follow any argument-position convention.
The "subject" is always the first argument when using dot syntax.

Sources:
- [UFCS Wikipedia](https://en.wikipedia.org/wiki/Uniform_Function_Call_Syntax)
- [UFCS in D, Nim](https://tour.dlang.org/tour/en/gems/uniform-function-call-syntax-ufcs)

---

## 10. D — UFCS and Range Algorithms

**UFCS originated in D and is a core language feature, especially for range-based
algorithm chaining.**

### Syntax

`a.fun(b)` rewrites to `fun(a, b)` when `a` has no member `fun`. Also supports
property syntax: `"hello".toLower` without parentheses.

```d
auto result = [1, 2, 3, 4, 5]
    .map!(x => x * 2)
    .filter!(x => x > 4)
    .array;
```

### Range algorithm chaining

D's standard library (`std.algorithm`, `std.range`) is designed around UFCS:

```d
[1, 2].chain([3, 4]).retro    // yields: 4, 3, 2, 1
[1, 1, 2, 2, 2].group.dropOne.front  // returns: tuple(2, 3)
```

UFCS makes D's range-based algorithms feel like method calls while remaining free
functions — extensible without modifying types.

Sources:
- [UFCS - Dlang Tour](https://tour.dlang.org/tour/en/gems/uniform-function-call-syntax-ufcs)
- [D UFCS Tutorial](https://riptutorial.com/d/topic/4155/ufcs---uniform-function-call-syntax)

---

## 11. Kotlin — Scope Functions and Extension Functions

**No pipe operator, but scope functions (`let`, `also`, `apply`, `run`, `with`)
provide pipe-like patterns, and extension functions enable method-syntax
additions.**

### Scope functions as pipes

| Function | Access via | Returns         | Pipe equivalent          |
|----------|-----------|-----------------|--------------------------|
| `let`    | `it`      | lambda result   | transform + continue     |
| `also`   | `it`      | original object | side effect (like `tap`) |
| `apply`  | `this`    | original object | configure + continue     |
| `run`    | `this`    | lambda result   | transform + continue     |
| `with`   | `this`    | lambda result   | (not extension fn)       |

```kotlin
listOf(1, 2, 3)
    .also { println("Before: $it") }
    .map { it * 2 }
    .also { println("After: $it") }
    .filter { it > 3 }
    .let { "Result: $it" }
```

### Extension functions

Kotlin's extension functions add methods to existing types without modifying them:

```kotlin
fun String.isPalindrome(): Boolean = this == this.reversed()
"racecar".isPalindrome()  // true
```

This provides the same extensibility as UFCS — any function can become a "method"
on a type, enabling chaining.

Sources:
- [Scope functions](https://kotlinlang.org/docs/scope-functions.html)
- [Using Scoped Functions in Kotlin](https://blog.mindorks.com/using-scoped-functions-in-kotlin-let-run-with-also-apply/)

---

## 12. Swift — Custom Operators and Extensions

**No built-in pipe operator, but custom operators can define one. Extensions
provide method-syntax additions similar to Kotlin.**

### Custom pipe operator

Swift allows defining custom operators:

```swift
infix operator |> : AdditionPrecedence
func |> <T, U>(value: T, function: (T) -> U) -> U {
    return function(value)
}

2 |> { $0 * 3 } |> { $0 + 1 } |> String.init  // "7"
```

### Extensions as method chaining

```swift
extension Int {
    func doubled() -> Int { self * 2 }
    func isEven() -> Bool { self % 2 == 0 }
}

42.doubled().isEven()  // true
```

### Point-Free style

The [Point-Free](https://www.pointfree.co/) project by Brandon Williams and
Stephen Celis has advocated for pipe and compose operators in Swift, demonstrating
that function composition unlocks patterns impossible with method chaining alone.

Sources:
- [F#'s Pipe-Forward Operator in Swift](https://undefinedvalue.com/fs-pipe-forward-operator-swift.html)
- [Single Forward Pipe Operator in Swift](https://holyswift.app/single-forward-pipe-operator-in-swift/)
- [Implementing a Custom Forward Pipe Operator](https://mariusschulz.com/blog/implementing-a-custom-forward-pipe-operator-for-function-chains-in-swift)

---

## 13. Scala — `pipe` and `tap` (2.13+)

**Added pipe and tap as extension methods in Scala 2.13 via `scala.util.chaining`.**

```scala
import scala.util.chaining._

42
  .tap(x => println(s"Before: $x"))
  .pipe(_ * 2)
  .pipe(_.toString)
  .tap(x => println(s"After: $x"))
```

- `pipe` applies a function and returns the result (like `|>`)
- `tap` applies a function for side effects and returns the original value

Both are added via an implicit conversion to `ChainingOps`, available for any
type. Custom `|>` operators can be defined:

```scala
implicit class Piper[A](val a: A) {
  def |>[B](f: A => B): B = a.pipe(f)
}
```

Sources:
- [Scala 2.13's pipe and tap chaining operations](https://alvinalexander.com/scala/scala-2.13-pipe-tap-chaining-operations/)
- [scala.util.ChainingOps](https://www.scala-lang.org/api/2.13.16/scala/util/ChainingOps.html)

---

## 14. Julia — `|>` and `@pipe`

**Built-in `|>` for single-argument functions, macros for multi-argument piping.**

### Built-in `|>`

Julia's native pipe passes the left side as the sole argument to the right side:

```julia
[1, 2, 3] |> sum |> sqrt
```

Limitation: only works with single-argument functions. No multi-argument piping.

### Broadcasting `.|>`

Combines piping with Julia's dot-vectorization:

```julia
[1, 4, 9] .|> sqrt  # [1.0, 2.0, 3.0]
```

### `@pipe` macro (Pipe.jl)

Uses `_` as a placeholder for multi-argument piping:

```julia
@pipe data |> filter(_, pred) |> map(transform, _)
```

### `@chain` macro (Chain.jl)

Rewrites a block of expressions as a pipeline, inserting the previous result as
the first argument by default, with `_` for explicit placement:

```julia
@chain df begin
    filter(:age, >(30))
    select(:name, :age)
end
```

Sources:
- [Julia pipe operator](https://syl1.gitbook.io/julia-language-a-concise-tutorial/useful-packages/pipe)
- [Chain.jl](https://github.com/jkrumbiegel/Chain.jl)

---

## 15. Gleam — `|>` on the BEAM

**A statically-typed functional language on the Erlang VM. Uses `|>` with
first-argument insertion, following the BEAM ecosystem convention.**

```gleam
"Hello, world!"
|> string.split(", ")
|> list.map(string.uppercase)
|> string.join(" ")
```

`a |> b(1, 2)` becomes `b(a, 1, 2)`. Gleam's standard library is designed
data-first to support this. The choice of first-argument was deliberate: it
matches the BEAM convention (shared with Elixir and Erlang) and mirrors the
"subject as first argument" pattern from OOP languages like Go, Python, and Rust.

Sources:
- [Gleam Pipelines](https://tour.gleam.run/functions/pipelines/)
- [My Favorite Gleam Feature](https://erikarow.land/notes/gleam-favorite-feature)

---

## Summary Table

| Language    | Operator/Mechanism     | Arg Position    | Placeholder | Compile-time? |
|-------------|------------------------|-----------------|-------------|---------------|
| Unix Shell  | `\|`                   | stdin/stdout    | N/A         | N/A           |
| Elixir      | `\|>`                  | first           | no          | yes (rewrite) |
| F#          | `\|>`                  | last (curried)  | no          | yes           |
| OCaml       | `\|>`                  | last (curried)  | no          | yes           |
| Haskell     | `&`                    | sole/last       | no          | yes           |
| R (magrittr)| `%>%`                  | first (default) | `.`         | no (runtime)  |
| R (base)    | `\|>`                  | first           | `_`         | yes (syntax)  |
| Rust        | method `.`             | self            | N/A         | yes           |
| JavaScript  | `\|>` (proposed)       | placeholder     | `%`         | yes           |
| Clojure     | `->` / `->>`           | first / last    | N/A         | yes (macro)   |
| Nim         | UFCS `.`               | first           | N/A         | yes           |
| D           | UFCS `.`               | first           | N/A         | yes           |
| Kotlin      | `.let{}`/`.also{}`     | `it`/`this`     | N/A         | yes           |
| Swift       | custom `\|>` / ext     | first           | no          | yes           |
| Scala       | `.pipe()` / `.tap()`   | function arg    | no          | yes           |
| Julia       | `\|>`                  | sole            | `_` (@pipe) | yes/macro     |
| Gleam       | `\|>`                  | first           | no          | yes           |
