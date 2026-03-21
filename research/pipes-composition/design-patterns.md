# Pipe Operator Design Patterns

Design trade-offs, interaction patterns, and implementation considerations for
pipe operators, with specific attention to how they apply to lx's `|>` operator
and `~>` / `~>?` agent messaging.

---

## 1. First-Arg vs Last-Arg vs Placeholder

The most fundamental design decision for a pipe operator is which argument
position the piped value fills. This choice cascades through the entire API
design of a language's ecosystem.

### First-argument insertion

**Used by**: Elixir, Gleam, R, Nim (UFCS), D (UFCS), lx

The piped value becomes the first argument: `x |> f(a, b)` desugars to `f(x, a, b)`.

Advantages:
- Matches OOP intuition: the "subject" comes first, like `self`/`this`
- Works naturally with multi-argument functions without currying
- Type inference benefits: the compiler sees the data type first, then propagates
  into callbacks. `users |> List.map(u => u.age)` lets the compiler infer `u`'s
  type from `users` before analyzing the callback body
- Better IDE autocompletion: knowing the subject type enables meaningful suggestions
- Simpler error messages: type mismatches report against the known subject type
  rather than against inferred callback return types

Disadvantages:
- Partial application is less natural. `List.map(double)` doesn't work because the
  collection (first arg) is missing — you'd need `fn(xs) { List.map(xs, double) }`
- Breaks the functional programming tradition of data-last

### Last-argument insertion

**Used by**: F#, OCaml, Haskell, Elm, Ramda (JS library)

The piped value becomes the last argument (or in curried languages, the sole
argument of the partially-applied result): `x |> f a` desugars to `f a x`.

Advantages:
- Natural partial application: `List.map double` returns a function awaiting the list
- Enables pointfree composition: `process = filter isEven >> map double >> sum`
- Deeply rooted in lambda calculus and ML tradition

Disadvantages:
- Type inference flows left-to-right, but the data arrives last — the compiler
  doesn't know the subject type while analyzing callbacks
- Less intuitive for developers from OOP or imperative backgrounds
- Optional/default parameters become awkward (need trailing `unit` in OCaml/F#)

### Placeholder (arbitrary position)

**Used by**: JavaScript proposal (`%`), R magrittr (`.`), Clojure `as->`, Julia
`@pipe` (`_`)

The piped value goes wherever the placeholder appears:
`x |> f(a, %, b)` desugars to `f(a, x, b)`.

Advantages:
- Maximum flexibility — no argument position convention needed
- Works with any existing API regardless of parameter order
- Can appear multiple times: `x |> f(%, %)` desugars to `f(x, x)`

Disadvantages:
- Requires a new token/syntax (placeholder) in the language
- More visual noise than implicit insertion
- Debates about placeholder token choice (`%`, `_`, `.`, `#`, `^`) stall proposals
  (see: TC39 multi-year debate)
- Readability decreases with complex expressions containing multiple placeholders

### Recommendation for lx

lx uses first-argument insertion. This is the right choice for an agent-oriented
language because:

1. Agent handles, message values, and pipeline subjects are the "thing being acted
   upon" — they belong first
2. lx doesn't have pervasive currying, so last-arg gains are minimal
3. First-arg aligns with `~>` agent messaging: `agent ~> msg` reads as
   "send msg to agent," paralleling `data |> transform`
4. The Elixir/Gleam precedent on BEAM shows this convention scales to large
   ecosystems

Sources:
- [Data-first and data-last comparison](https://www.javierchavarri.com/data-first-and-data-last-a-comparison/)
- [The Right Way To Pipe](https://yangdanny97.github.io/blog/2023/12/28/pipes)
- [Pipe First (Reason)](https://reasonml.github.io/docs/en/pipe-first)

---

## 2. Pipe + Partial Application

How pipes interact with partial application and currying determines the "feel" of
a language's functional programming story.

### In curried languages (F#, Haskell, OCaml)

Pipes and partial application are deeply synergistic. Every function is curried,
so `List.map double` is already a valid expression — it returns `'a list -> 'a list`.
Piping provides the final argument:

```fsharp
[1; 2; 3] |> List.map (fun x -> x * 2) |> List.filter (fun x -> x > 2)
```

The composition operator `>>` takes this further:

```fsharp
let process = List.map double >> List.filter isEven >> List.sum
// no value mentioned — pure function composition
```

### In non-curried languages (Elixir, lx)

Without currying, partial application requires explicit syntax. Elixir uses the
capture operator: `&Enum.map(&1, double)`. lx handles this through closures:

```
-- lx: explicit closure for partial application in pipes
data |> fn(x) { map(x, double) }
```

### Sections and placeholder partial application

Some languages provide "sections" — partial application by leaving holes:

- Haskell: `(+1)`, `(*2)`, `(>3)` — operator sections
- Scala: `_ * 2` — underscore as placeholder in lambdas
- Kotlin: `{ it * 2 }` — implicit `it` parameter

These interact well with pipes because they create concise single-argument
functions without naming parameters.

### The data-first advantage

In data-first languages, you don't need partial application as often because the
pipe fills the subject position. The remaining arguments are typically the
"configuration" (the function to map, the predicate to filter) which you write
inline:

```elixir
data |> Enum.map(&double/1) |> Enum.filter(&(&1 > 3))
```

This is why Elixir, despite lacking currying, has a fully functional pipe story.

---

## 3. Pipe + Error Handling: Railway-Oriented Programming

The central question: what happens when a step in a pipe chain fails?

### The problem

```
data |> parse |> validate |> transform |> save
```

If `validate` fails, should `transform` still run? How does the error propagate?

### Railway-Oriented Programming (Scott Wlaschin)

Wlaschin's metaphor (from F# for Fun and Profit, ~2014) visualizes error handling
as a railway with two tracks:

- **Success track**: data flows through transformations normally
- **Failure track**: once an error occurs, subsequent steps are skipped

The key insight: every function is a "railway switch" that either continues on
the success track or diverts to the failure track.

### Implementation with Result/Either types

```fsharp
// F# — each function returns Result<'a, 'err>
input
|> parse       // Ok(parsed) or Error("parse failed")
|> Result.bind validate   // skipped if parse failed
|> Result.bind transform  // skipped if validate failed
|> Result.bind save       // skipped if transform failed
```

The `bind` function (also called `>>=` or `flatMap`) is the connector:
- If the input is `Ok(value)`, apply the function to `value`
- If the input is `Error(e)`, pass the error through unchanged

### Key ROP functions

| Function | Type | Purpose |
|----------|------|---------|
| `bind` | `(a -> Result<b,e>) -> Result<a,e> -> Result<b,e>` | Connect two-track functions |
| `map` | `(a -> b) -> Result<a,e> -> Result<b,e>` | Adapt one-track function for two tracks |
| `tee` | `(a -> unit) -> a -> a` | Side effect without affecting flow |
| `tryCatch` | `(a -> b) -> a -> Result<b,exn>` | Convert exception-throwing to Result |
| `doubleMap` | `(a -> c) -> (b -> d) -> Result<a,b> -> Result<c,d>` | Transform both tracks |

### Rust's `?` operator

Rust solved this at the language level. The `?` operator on `Result<T, E>`:
- If `Ok(v)`: unwrap and continue
- If `Err(e)`: return early from the function with the error

```rust
fn process(input: &str) -> Result<Output, Error> {
    let parsed = parse(input)?;      // early return on error
    let valid = validate(parsed)?;   // early return on error
    let result = transform(valid)?;  // early return on error
    save(result)
}
```

This isn't piping per se, but it achieves the same linear, top-to-bottom flow
with automatic error propagation.

### Clojure's `some->`

The `some->` macro short-circuits on `nil`:

```clojure
(some-> order :customer :address :city .toUpperCase)
;; returns nil if any step yields nil
```

This is ROP for nil instead of errors.

### lx's approach

lx has `Ok`/`Err` result types. A pipe chain with error handling could use:

```
data
|> parse
|> Result.bind(validate)
|> Result.bind(transform)
|> Result.bind(save)
```

Or a dedicated error-propagating pipe (like Elixir's `with` construct or Rust's
`?`) could be introduced.

### When NOT to use ROP

Wlaschin himself warns against overuse:
- Don't use Result for expected control flow (use pattern matching)
- Don't use Result when you need to accumulate multiple errors (use validation
  applicatives instead)
- Don't use Result for truly exceptional conditions (those should crash)

Sources:
- [Railway Oriented Programming](https://fsharpforfunandprofit.com/rop/)
- [Against Railway-Oriented Programming](https://fsharpforfunandprofit.com/posts/against-railway-oriented-programming/)
- [What is railway oriented programming?](https://blog.logrocket.com/what-is-railway-oriented-programming/)

---

## 4. Pipe + Async

Piping through asynchronous operations introduces scheduling questions: when does
each step run? How do you await intermediate results?

### Promise/Future chaining (JavaScript, Rust)

JavaScript's `.then()` is already a pipe for Promises:

```javascript
fetch(url)
  .then(response => response.json())
  .then(data => process(data))
  .then(result => save(result))
  .catch(error => handleError(error))
```

The TC39 pipe proposal integrates with `await`:

```javascript
url
  |> fetch(%)
  |> await %
  |> %.json()
  |> await %
  |> process(%)
```

### Rust async pipes

Rust's `.await` is a postfix operator, enabling natural chaining:

```rust
let result = fetch(url)
    .await?
    .json::<Data>()
    .await?
    .process();
```

The `?` and `.await` compose cleanly because both are postfix.

### Elixir — no async pipes needed

Elixir's concurrency model (processes + message passing) means async is handled
at the process level, not the expression level. Pipes are always synchronous
within a process. Async work is delegated to other processes via `Task.async` and
collected with `Task.await`.

### F# async pipes

F# uses computation expressions (`async { }`) which have their own `let!` for
awaiting:

```fsharp
async {
    let! response = fetchAsync url
    let! data = parseAsync response
    return process data
}
```

Pipes don't directly compose with `async` — you work inside the computation
expression instead.

### Design considerations for lx

lx is inherently concurrent (agents, `par`, `sel`). Options for async piping:

1. **Implicit await**: `data |> asyncOp` automatically awaits the result before
   passing to the next step. Simple but hides latency.
2. **Explicit await**: require `|> await asyncOp` or a dedicated async pipe `|!>`.
   More explicit but verbose.
3. **Agent delegation**: use `~>?` (ask) for async steps, keeping `|>` synchronous.
   This matches lx's agent-centric design.

---

## 5. Pipe + Agents: lx's `~>` and `~>?`

lx extends the pipe concept into agent messaging with two operators:

### `~>` (tell / fire-and-forget)

`agent ~> msg` sends `msg` to `agent` without waiting for a response. Analogous
to a Unix pipe where the sender doesn't wait for the receiver to finish.

### `~>?` (ask / request-response)

`agent ~>? msg` sends `msg` and waits for a response. Analogous to a synchronous
function call through a pipe — the result flows back.

### Relationship to `|>`

| Operator | Mechanism | Blocking? | Returns |
|----------|-----------|-----------|---------|
| `\|>` | Function application | yes | function result |
| `~>` | Message send | no | unit |
| `~>?` | Message send + await | yes | agent response |

The three operators form a spectrum:
- `|>` is data-through-functions (synchronous, local)
- `~>` is data-to-agent (asynchronous, distributed)
- `~>?` is data-to-agent-and-back (synchronous from caller's perspective)

### Composing pipes with agent messaging

A natural pattern in lx:

```
data
|> preprocess
|> validate
~>? worker_agent    -- ask an agent to do heavy work
|> postprocess      -- continue with the agent's response
|> format_output
```

This mixes local transformation (`|>`) with distributed computation (`~>?`).

---

## 6. Method Chaining vs Pipe Operators

Two approaches to left-to-right data flow. Neither is strictly superior.

### Method chaining

```rust
vec![1, 2, 3, 4, 5]
    .iter()
    .map(|x| x * 2)
    .filter(|x| *x > 4)
    .collect::<Vec<_>>()
```

Advantages:
- IDE-friendly: type `.` and get autocompletion of available methods
- No import needed: methods are inherently scoped to the type
- Familiar from OOP

Disadvantages:
- Methods must be defined on the type (or via traits/extensions)
- Adding new operations requires trait implementations or wrapper types
- Can't compose arbitrary free functions

### Pipe operators

```elixir
[1, 2, 3, 4, 5]
|> Enum.map(&(&1 * 2))
|> Enum.filter(&(&1 > 4))
```

Advantages:
- Works with any function, not just methods on the type
- No need to define traits or extension methods
- Free functions compose more easily than methods
- Testing: free functions with explicit inputs are trivially testable

Disadvantages:
- Less IDE support (no dot-completion)
- Requires consistent argument position conventions across the ecosystem
- Import management: must import functions to use short names

### Hybrid: UFCS (Nim, D)

UFCS eliminates the trade-off by making every free function callable as a method:
`f(a, b)` ↔ `a.f(b)`. This provides dot-completion while keeping functions free.
The cost is potential ambiguity in name resolution.

### When to use which

- **Method chaining**: when operations are tightly coupled to a type (iterator
  combinators, builder patterns, fluent APIs)
- **Pipe operators**: when composing operations across different types/modules,
  or when the operation set is open-ended

---

## 7. Type Inference with Pipes

Left-to-right data flow aligns with left-to-right type inference, creating a
synergy that improves both error messages and tooling.

### The F# / BuckleScript insight

F#'s type inferencer works left-to-right. When you write:

```fsharp
users |> List.map (fun u -> u.Name)
```

The compiler:
1. Sees `users` — knows it's `User list`
2. Sees `List.map` — knows it takes `('a -> 'b) -> 'a list -> 'b list`
3. Unifies `'a list` with `User list` — now knows `'a = User`
4. Inside the callback, knows `u : User` — can resolve `u.Name`

Without pipes (data-last, callback first):

```fsharp
List.map (fun u -> u.Name) users
```

The compiler sees the callback before it knows `u`'s type. It must either defer
resolution or require a type annotation.

### BuckleScript/ReScript's data-first move

This inference advantage motivated BuckleScript (now ReScript) to adopt data-first
APIs for its Belt standard library, breaking with OCaml's data-last tradition.
Result: fewer type annotations, better error messages, better IDE tooling.

### Practical consequences

Data-first + left-to-right inference means:
- Fewer explicit type annotations in pipe chains
- More precise error messages (errors reference known types)
- Better IDE autocompletion (the type is known before the callback)
- Faster compilation (fewer unification variables to resolve)

Sources:
- [Data-first and data-last comparison](https://www.javierchavarri.com/data-first-and-data-last-a-comparison/)
- [Pipe First (Reason)](https://reasonml.github.io/docs/en/pipe-first)

---

## 8. API Design for Pipeability

The pipe operator doesn't just affect how you write code — it shapes how you
design APIs.

### The subject-first rule

For a function to be pipeable, the "thing being transformed" must appear in the
position that the pipe fills. In data-first languages:

```
-- Good: subject is first
fn map(list, func) -> list
fn filter(list, pred) -> list
fn take(list, n) -> list

-- Bad: subject is not first (can't pipe naturally)
fn map(func, list) -> list
```

### Return type consistency

Pipeability requires that each function returns something the next function can
accept. This encourages:

- **Same-type chains**: `list |> filter |> map |> sort` — all take and return lists
- **Type-changing chains**: `string |> parse |> validate |> save` — each step may
  change the type, but the output of each is the input type of the next
- **Wrapped types**: `Result<T>` flows through `bind`/`map` — the wrapper type
  stays consistent even as the inner type changes

### Naming conventions

Pipeable APIs tend toward verb-first naming: `filter`, `map`, `sort`, `take`,
`drop`, `group_by`. The subject is implied by the pipe, so the function name
describes the action.

### Arity constraints

Functions with many parameters are awkward to pipe. Best practice:
- Primary operation: 2 args (subject + one config) — most common
- With options: 3 args (subject + config + options record)
- Complex operations: take an options record/struct as the second argument

### Builder pattern as alternative

When configuration is complex, the builder pattern avoids the arity problem:

```rust
Request::new(url)
    .method(Method::POST)
    .header("Content-Type", "application/json")
    .body(data)
    .send()
```

Each method takes `self` + one argument, keeping arity low.

---

## 9. Debugging Pipes

A common complaint: pipe chains are hard to debug because intermediate values
aren't bound to names.

### The `tap` / `inspect` pattern

Insert a side-effecting function that logs the value and returns it unchanged:

```elixir
data
|> parse()
|> tap(&IO.inspect/1)       # prints parsed value
|> validate()
|> tap(&IO.inspect/1)       # prints validated value
|> transform()
```

Languages with built-in tap:
- **Elixir**: `Kernel.tap/2` (1.12+)
- **Scala**: `.tap(f)` via `ChainingOps` (2.13+)
- **Kotlin**: `.also { }` scope function
- **Ruby**: `.tap { |x| }` (built-in since 1.9)
- **Rust**: the `tap` crate, or `.inspect()` on iterators
- **RxJS**: `tap()` operator on Observables
- **Ramda.js**: `R.tap(f)`

### Named intermediate bindings

When debugging gets serious, break the pipe into named bindings:

```elixir
# Instead of:
data |> parse() |> validate() |> transform()

# Temporarily:
parsed = parse(data)
IO.inspect(parsed, label: "parsed")
validated = validate(parsed)
IO.inspect(validated, label: "validated")
transformed = transform(validated)
```

This is the nuclear option — effective but verbose. The pipe can be restored once
the bug is found.

### IDE/debugger support

Some IDEs can show intermediate values in pipe chains:
- **IntelliJ** (Kotlin): shows `.let {}` results inline during debugging
- **VS Code** (Elixir): step-through debugging shows pipe intermediate values
- **RustRover/rust-analyzer**: hover over iterator chain to see types at each step

### lx consideration

lx could provide a built-in `tap` or `inspect` function that logs intermediate
pipe values. Since lx programs are agent workflows, logging intermediate pipeline
state is especially valuable for debugging multi-step workflows.

---

## 10. Performance

Do pipe operators have runtime overhead?

### Compile-time rewriting (zero cost)

In most languages, the pipe operator is a compile-time transformation:

- **Elixir**: `x |> f(a)` is rewritten to `f(x, a)` at compile time. Zero runtime
  overhead. The BEAM sees no difference.
- **F#**: `x |> f` compiles to `f(x)`. The pipe is erased.
- **OCaml**: `|>` is a built-in primitive, compiled away.
- **Nim/D UFCS**: `a.f(b)` is `f(a, b)` — pure syntactic sugar.
- **R base `|>`**: syntactic transformation, no function call overhead.

### Runtime overhead (non-zero but small)

Some implementations involve actual function calls:

- **R magrittr `%>%`**: implemented as a function. Each pipe step involves a
  function call, environment creation, and argument matching. Measurably slower
  than base `|>` in tight loops (microseconds per step).
- **Scala `.pipe()`**: extension method via implicit conversion. The implicit
  conversion has overhead on first use (JIT can optimize subsequent calls).
- **JavaScript (proposed)**: Hack-style pipes would be compiled to expression
  evaluation with temporary bindings. Should be zero-cost after JIT optimization.

### Rust iterator chains: the gold standard

Rust's iterator combinators are the canonical example of zero-cost abstractions:

```rust
(1..=1000)
    .map(|x| x * 3)
    .filter(|x| x % 2 != 0)
    .sum::<i32>()
```

This compiles to the same assembly as a hand-written loop. The compiler:
1. Monomorphizes all generic types
2. Inlines all closure bodies
3. Eliminates all intermediate iterator structs
4. Produces a single loop with conditional accumulation

### lx's pipe performance

lx's `|>` evaluates the left side, then applies the result as the first argument
to the right side (see `eval_pipe` in the interpreter). In interpreted mode, each
pipe step involves:
1. Evaluate left expression
2. Force defaults on the result
3. Evaluate right expression (get the function)
4. Apply the function

This is the cost of any function call — the pipe adds no overhead beyond that.
In a future compiled backend, `x |> f(a)` could compile directly to `f(x, a)`
with zero overhead.

---

## Summary: Design Space for lx

lx's current pipe design — first-argument insertion, synchronous, combined with
`~>` / `~>?` for agent messaging — occupies a well-chosen point in the design
space:

| Design dimension | lx's choice | Rationale |
|-----------------|-------------|-----------|
| Argument position | first | matches agent-oriented "subject first" pattern |
| Placeholder | none | simplicity; first-arg covers common cases |
| Error propagation | via Result types | explicit, composable with `bind`/`map` |
| Async integration | via `~>?` (ask) | agents are the concurrency primitive |
| Debugging | tap/inspect (potential) | critical for workflow debugging |
| Performance | interpreted (call overhead) | no extra cost beyond function application |

### Open design questions

1. **Error-propagating pipe**: should lx have a `|?>` that auto-propagates
   `Err` values (like Rust's `?` but for pipes)?
2. **Async pipe**: should `|>` work with async functions, or should async
   always go through agents (`~>?`)?
3. **Composition operator**: should lx have `>>` for pointfree function
   composition, or is `|>` sufficient?
4. **tap built-in**: should `tap` be a language primitive or a stdlib function?
5. **Pipe + pattern match**: could `|>` feed into `match` arms directly?
