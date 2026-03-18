# lx Program Quality Audit

Every item below is a binary check — a violation either exists or it does not. This audit targets mistakes that Claude (and LLMs in general) make when writing lx programs, because lx is a novel language with no training data. Most violations come from bleeding through syntax/semantics of Python, Rust, JavaScript, or Haskell.

Run the **High Frequency** list first — these violations appear in nearly every first-draft lx program. Run the **Medium Frequency** list second. Run the **Low Frequency** list last.

---

## High Frequency Checks

### Syntax Bleed-Through

- **Commas in collections or parameters** — lists, records, tuples, maps, or function parameters use commas. lx is space-separated everywhere. Fix: remove all commas. `[1 2 3]` not `[1, 2, 3]`. `(x y) x + y` not `(x, y) x + y`. `{x: 1  y: 2}` not `{x: 1, y: 2}`.
  `rg ',' flows/ tests/ brain/ workgen/ --type-add 'lx:*.lx' --type lx`
  Exception: none. Commas are never valid in lx.

- **Wrong comment syntax** — uses `//`, `#`, `/* */`, or `"""` instead of `--`. lx comments are `--` only.
  `rg '^\s*(//|#|/\*)' --type-add 'lx:*.lx' --type lx`
  Fix: replace with `--`.

- **`if`/`else`/`elif` keywords** — lx has no `if`. Use `?` ternary (`cond ? then : else`), single-arm conditional (`cond ? expr`), or full match (`x ? { ... }`).
  `rg '\bif\b|\belse\b|\belif\b' --type-add 'lx:*.lx' --type lx`
  Fix: `if x > 0 then y else z` → `x > 0 ? y : z`. `if cond { body }` → `cond ? body`.

- **`let`/`var`/`const`/`mut` keywords in bindings** — lx bindings have no keyword prefix. `x = 42` (immutable), `x := 0` (mutable), `x <- x + 1` (reassign).
  `rg '\b(let|var|const|mut)\b' --type-add 'lx:*.lx' --type lx`
  Fix: delete the keyword. `let x = 42` → `x = 42`. `let mut x = 0` → `x := 0`.

- **`fn`/`def`/`func`/`lambda` keywords** — lx functions are just parenthesized params followed by a body: `double = (x) x * 2`. No keyword.
  `rg '\b(fn|def|func|lambda)\b' --type-add 'lx:*.lx' --type lx`
  Fix: `fn double(x) { x * 2 }` → `double = (x) x * 2`.

- **`return` keyword** — lx is expression-oriented. The last expression in a block is the return value. No `return`.
  `rg '\breturn\b' --type-add 'lx:*.lx' --type lx`
  Fix: delete `return`. `return x + 1` → `x + 1`.

- **`match`/`switch`/`case` keywords** — lx pattern matching uses `?`: `x ? { 0 -> "zero"; _ -> "other" }`.
  `rg '\b(match|switch|case)\b' --type-add 'lx:*.lx' --type lx`
  Fix: `match x { ... }` → `x ? { ... }`.

- **`import`/`from`/`require` keywords** — lx uses `use`: `use std/json`, `use ./util`, `use std/json {parse encode}`.
  `rg '\b(import|from|require)\b' --type-add 'lx:*.lx' --type lx`
  Fix: `import json` → `use std/json`. `from std/json import parse` → `use std/json {parse}`.

- **`async`/`await` keywords** — lx uses `par { ... }` for parallel execution, `sel { ... }` for racing, `pmap` for parallel map. No async/await.
  `rg '\b(async|await)\b' --type-add 'lx:*.lx' --type lx`
  Fix: `await fetch(url)` → `fetch url ^`. Parallel: `par { fetch url1 ^; fetch url2 ^ }`.

- **`try`/`catch`/`throw`/`raise` keywords** — lx errors are values (`Ok`/`Err`/`Some`/`None`). `^` propagates, `??` coalesces. No exceptions.
  `rg '\b(try|catch|throw|raise|except|finally)\b' --type-add 'lx:*.lx' --type lx`
  Fix: `try { f() } catch (e) { ... }` → `f ^ ?? fallback` or match on the Result.

- **`for`/`while` loop keywords** — lx iteration is `collection | each (x) { ... }` or `loop { ... }` with `break`.
  `rg '\b(for|while)\b' --type-add 'lx:*.lx' --type lx`
  Fix: `for x in xs { ... }` → `xs | each (x) { ... }`.

- **`true`/`false` capitalized or stringified** — lx booleans are lowercase `true` and `false`. Not `True`, `False`, `"true"`, `TRUE`.
  `rg '\b(True|False|TRUE|FALSE)\b' --type-add 'lx:*.lx' --type lx`

### Operator & Expression Mistakes

- **`=` used for mutation** — `=` creates an immutable binding. `:=` creates a mutable one. `<-` reassigns a mutable binding.
  Manual review: look for `x = x + 1` patterns where `x` is already bound. Fix: `x <- x + 1`.

- **`&&`/`||` confused with `&` guard** — `&&` and `||` are logical operators for combining booleans. `&` is the guard operator in match arms: `n & (n > 5) -> ...`. Do not use `&&` inside match arm guards.
  Manual review: check match arms. `n && (n > 5) -> ...` ✗ → `n & (n > 5) -> ...` ✓.

- **`/` used expecting integer result** — `/` always returns Float, even for two Ints. `7 / 2` is `3.5`, not `3`. Use `//` for integer division.
  Manual review: check division operations. Fix: `7 / 2` → `7 // 2` if integer result expected.

- **Method call syntax instead of pipes** — `list.map(f)`, `str.split(",")`, `x.toString()`. lx uses pipes: `list | map f`, `str | split ","`, `to_str x`.
  `rg '\.\w+\(' --type-add 'lx:*.lx' --type lx`
  Exception: field access like `record.name` is valid. Only flag when it looks like a method call with arguments in parens.

- **Parenthesized function arguments** — `f(x, y)` instead of `f x y`. lx uses juxtaposition for application.
  `rg '\w+\(\w' --type-add 'lx:*.lx' --type lx`
  Fix: `add(1, 2)` → `add 1 2`. `map(fn, list)` → `list | map fn`.
  Exception: `(expr)` for grouping is valid. Only flag when parens wrap function arguments.

- **Bracket indexing** — `xs[0]`, `map["key"]` instead of `xs.0`, `map."key"`. lx uses dot-based access for all indexing.
  `rg '\w+\[' --type-add 'lx:*.lx' --type lx`
  Fix: `xs[0]` → `xs.0`. `xs[-1]` → `xs.-1`. `map["key"]` → `map."key"`.

- **Arrow function syntax from JS/TS** — `(x) => x + 1` or `x => x + 1`. lx functions don't use `=>`.
  `rg '=>' --type-add 'lx:*.lx' --type lx`
  Fix: `(x) => x + 1` → `(x) x + 1`.

- **Missing `^` after fallible operations** — calling a function that returns Result/Maybe and using the value directly without unwrapping. Common with `fetch`, `json.parse`, `fs.read`, shell commands.
  Manual review: trace Result-returning calls. If the next operation expects the inner value, insert `^` or `??`.
  Fix: `url | fetch | (.body)` → `url | fetch ^ | (.body)`.

- **Verbose lambda where section suffices** — writing `(x) x + 1` when `(+ 1)` works, or `(x) x.name` when `(.name)` works, or `(x) x > 0` when `(> 0)` works.
  Manual review: check lambdas with single operations.
  Fix: `map (x) x * 2` → `map (* 2)`. `filter (x) x > 0` → `filter (> 0)`. `map (x) x.name` → `map (.name)`.

### Collection & Record Mistakes

- **Single-space record field separation** — `{x: 1 y: 2}` with one space between fields will misparse. Records need two+ spaces or newlines between fields.
  Manual review: check single-line records with multiple fields. Fix: `{x: 1 y: 2}` → `{x: 1  y: 2}` (two spaces) or put fields on separate lines.

- **`{}` for empty record** — `{}` is an empty block (returns Unit), not an empty record. Use `{:}` for an empty record.
  `rg '\{\}' --type-add 'lx:*.lx' --type lx`
  Fix: `{}` → `{:}` when you mean empty record.

- **Complex values in single-line record fields** — `{x: f a  y: z}` on one line misparsed due to known parser bug. The parser terminates field value too early.
  Manual review: check single-line records where a field value is a function call or multi-token expression. Fix: use multiline records or extract to temp bindings.
  ```
  -- Wrong (misparsed):
  {result: compute x  status: "done"}
  -- Right:
  r = compute x
  {result: r  status: "done"}
  -- Also right (multiline):
  {
    result: compute x
    status: "done"
  }
  ```

- **List spread without parens around function calls** — `[..f x y]` spreads `f` (a Func), not the result of `f x y`. Known parser bug.
  `rg '\[\.\.(?![\[(])' --type-add 'lx:*.lx' --type lx`
  Fix: `[..f x y]` → `[..(f x y)]`.

- **Pipe direction misunderstanding** — `|` inserts the left value as the LAST argument, not the first. `"hello" | replace "l" "r"` means `replace "l" "r" "hello"`.
  Manual review: check pipe chains where argument order matters. If the piped value should be the first argument, restructure: `replace "l" "r" str` or use a lambda.

### Module & Export Mistakes

- **Missing `+` prefix on exports** — functions/values intended for external use must be prefixed with `+`. `exported = (x) x` is private. `+exported = (x) x` is public.
  Manual review: check entry-point files and library modules for functions that should be accessible externally.
  Fix: `main = () { ... }` → `+main = () { ... }`.

- **Dot-path module access instead of slash** — `std.json` or `std::json` instead of `std/json`.
  `rg 'use std\.' --type-add 'lx:*.lx' --type lx`
  `rg 'use std::' --type-add 'lx:*.lx' --type lx`
  Fix: `use std.json` → `use std/json`.

---

## Medium Frequency Checks

### Error Handling

- **Parenthesized constructors for Ok/Err/Some** — `Ok(42)`, `Err("fail")`, `Some("hi")`. lx uses juxtaposition: `Ok 42`, `Err "fail"`, `Some "hi"`.
  `rg '(Ok|Err|Some|None)\(' --type-add 'lx:*.lx' --type lx`
  Fix: `Ok(42)` → `Ok 42`. `Err("message")` → `Err "message"`.

- **Confusing `^` with `?` from Rust** — in Rust, `?` propagates errors. In lx, `?` is the match/ternary operator. `^` is the propagation operator.
  `rg '\w+\?' --type-add 'lx:*.lx' --type lx`
  Fix: `risky_call?` → `risky_call ^`. Exception: `ok?`, `err?`, `some?`, `empty?`, `even?`, `odd?` are predicate builtins — these are valid.

- **Not handling field-miss None** — record and agent field access returns `None` for missing fields, not an error. Code that assumes a field exists will silently get `None`.
  Manual review: check field accesses on records from external sources. Fix: use `??` for defaults or `^` with `require` to convert to Result.

### Agent Communication

- **Using `~>` when `~>?` is needed** — `~>` is fire-and-forget (returns Unit). `~>?` is request-response (returns the handler's result). Using `~>` and expecting a response is a silent bug.
  `rg '~>' --type-add 'lx:*.lx' --type lx`
  Manual review: check if the result of `~>` is bound or piped. If so, it should be `~>?`.

- **Agent without handler field** — agents must have a `handler` field containing a function. `{process: (msg) msg}` is a record, not an agent. `{handler: (msg) msg}` is an agent.
  Manual review: check agent definitions. Fix: ensure the record has `handler:` as the function field.

- **Protocol fields on wrong number of lines** — Protocol definitions follow record syntax rules. Single-line multi-field protocols hit the same parser bug as records.
  Fix: put Protocol fields on separate lines for complex types.

### Iteration & Pipes

- **`fold` argument order wrong** — `fold` takes initial value first, then operation: `[1 2 3] | fold 0 (+)`. Not `fold (+) 0`.
  Manual review: check fold calls. Fix: `fold (+) 0` → `fold 0 (+)`.

- **Forgetting `collect` on lazy ranges** — `1..10 | len` fails because ranges are lazy. `1..10 | collect | len` works.
  Manual review: check operations on ranges that expect a concrete list. Fix: insert `| collect` before the consuming operation.

- **`each` vs `map` confusion** — `each` is for side effects (returns Unit). `map` transforms and returns a new list. Using `each` when you need the transformed list silently discards results.
  Manual review: check if the result of `each` is bound or piped. If so, it should be `map`.

- **Enumerate destructuring form** — `enumerate` produces `(index value)` tuples. The callback destructures as `(i x)`, not `(i, x)` and not `({i, x})`.
  Manual review: check `enumerate | each` or `enumerate | map` patterns.

### Shell Integration

- **Using `$command` and expecting stdout string** — `$echo "hi"` returns `Result {out err code}`, not a string. Use `$^echo "hi"` for direct stdout extraction.
  Manual review: check if `$command` result is used as a string. Fix: `$echo "hi"` → `$^echo "hi"` when you want the stdout string directly.

- **Shell pipe vs lx pipe ambiguity** — `|` inside `$` is a shell pipe. To pipe shell output to lx, close the shell expression first: `($^ls) | lines`.
  Manual review: check `$` expressions that also use `|`. Fix: wrap shell expression in parens before piping to lx.

### Pattern Matching

- **`->` vs `=>` in match arms** — lx uses `->` in match arms. `=>` is not valid.
  `rg '=>' --type-add 'lx:*.lx' --type lx`
  Fix: `0 => "zero"` → `0 -> "zero"`.

- **Missing semicolons between match arms on one line** — `x ? { 0 -> "zero" 1 -> "one" }` needs semicolons: `x ? { 0 -> "zero"; 1 -> "one" }`. Newlines also work as separators.
  Manual review: check single-line match expressions with multiple arms.

- **Rest pattern in records uses `..` not `...`** — `{name: "alice"  ..}` matches a record with at least a name field. Three dots is wrong.
  `rg '\.\.\.' --type-add 'lx:*.lx' --type lx`
  Fix: `{name: n  ...}` → `{name: n  ..}`.

---

## Low Frequency Checks

### Concurrency

- **Mutable capture in par/sel/pmap** — mutable bindings (`:=`) cannot be captured inside `par`, `sel`, or `pmap` bodies. This is a compile-time restriction.
  Manual review: check `par`/`sel`/`pmap` bodies for references to variables declared with `:=` outside the block. Fix: restructure to use immutable bindings or return values from par arms.

- **Assuming par/sel are actually parallel** — currently `par` and `sel` execute arms sequentially (no async runtime). `sel` doesn't actually race. Programs work but don't get real parallelism.
  No fix needed for correctness, but don't rely on timing behavior.

### Type System

- **Type annotation syntax confusion** — lx type annotations use `->` for return type and `^` for error type: `(x: Int y: Int) -> Int ^ Str`. Not `: Int` after the params.
  Manual review: check type-annotated functions. Fix: `(x: Int): Int` → `(x: Int) -> Int`.

- **Tagged union constructor as value vs call** — `Dot` (no args) is a value. `Circle 5.0` (with arg) is a constructor call. Forgetting the argument makes it a partially applied constructor.
  Manual review: check tagged union usage. `Circle` alone is a `TaggedCtor`, not a `Shape`.

- **Generic type parameter placement** — `Tree a = | Leaf a | Node (Tree a) (Tree a)`. The type parameter goes after the type name, not in angle brackets.
  `rg '<\w+>' --type-add 'lx:*.lx' --type lx`
  Fix: `Tree<a>` → `Tree a`.

### Miscellaneous

- **Named arg `:` consumed by ternary** — `f x key: val ? fallback` misparsed because `:` is ambiguous between named arg and ternary else. Known parser bug.
  Fix: parenthesize: `(f x key: val) ? fallback`.

- **Assert with parenthesized expression** — `assert (expr) "msg"` may consume `"msg"` as a function argument when `(expr)` looks callable. Known parser bug.
  Fix: `assert (expr) ; "msg"` or `assert expr "msg"` without parens.

- **Chained tuple-destructuring in HOF via test.run** — `| filter (a b) expr | map (a b) expr` fails with "undefined variable" when run through `test.run`. Works with `lx run` directly. Known runtime bug.
  Fix: avoid chained tuple-destructuring lambdas in test flows, or test via `lx run`.

- **Double-parent module imports** — `use ../../examples/foo` fails. Module resolver only handles single `..` parent.
  Fix: organize files so imports need at most one `..`, or use workspace member paths.
