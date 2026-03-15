# Syntax

Core lexical grammar and fundamental constructs: literals, bindings, functions, sections, and pipes.

## Lexical Grammar

```
IDENT     = [a-z_][a-z0-9_']*[?]?
TYPE      = [A-Z][a-zA-Z0-9]*
INT       = [0-9][0-9_]* | 0x[0-9a-fA-F_]+ | 0b[01_]+ | 0o[0-7_]+
FLOAT     = [0-9][0-9_]*\.[0-9][0-9_]* ([eE][+-]?[0-9]+)?
STR       = " (char | \escape | { EXPR })* "
RAW_STR   = ` char* `
SEP       = \n | ;
COMMENT   = -- to end of line
```

No commas. Whitespace separates all elements. Newlines and `;` terminate statements (interchangeable). Underscores in numeric literals are ignored (`1_000_000`). `?` appears at most once, only as the final character of an identifier. `'` can appear in identifiers for mutating variants (`sort'`). The full operator set and precedence table are in [grammar.md](grammar.md).

## Literals

```
42                       -- int
3.14                     -- float
0xff 0b1010 0o77         -- hex, binary, octal
true false               -- bool
()                       -- unit (nothing/void)
"hello {name}"           -- interpolated string
`raw {not interpolated}` -- raw string
```

Double quotes always interpolate via `{expr}`. Backticks never interpolate.

**Multi-line strings** — both `"` and `` ` `` strings can span lines. When a string literal contains newlines, leading indentation is stripped using the closing delimiter's column position:

```
msg = "
  hello
  world
  "
-- result: "hello\nworld\n"
```

The stripping algorithm: remove leading newline after opening delimiter, strip indentation matching closing delimiter's column, preserve trailing newline. Single-line strings are not affected. Backticks work the same way but with no interpolation.

## Bindings

```
x = 5          -- immutable (default)
x := 5         -- mutable
x <- 10        -- reassign mutable
x: Int = 5     -- immutable with type annotation
```

No `let`/`const`/`var`. `=` is always immutable binding. `:=` creates a mutable binding. `<-` reassigns an existing mutable. Shadowing with `=` is allowed (creates a new immutable binding that shadows the old one).

Type annotations on bindings are optional. When present, `lx check` validates the value matches the declared type.

## Functions

A function is a value. No keyword needed.

```
double = (x) x * 2
add = (x y) x + y
process = (data) {
  cleaned = data | filter (!= "")
  cleaned | sort | uniq
}
now = () $date
```

### Type annotations

Parameters and return types can be annotated. Annotations are optional — omitted types are inferred by the checker or ignored at runtime:

```
add = (x: Int y: Int) -> Int x + y
greet = (name: Str) "hello {name}"
safe_div = (a: Int b: Int) -> Int ^ Str {
  b == 0 ? Err "division by zero" : Ok (a / b)
}
```

`-> Type` after the closing `)` declares the return type. `-> Type ^ ErrType` declares a fallible return. Complex type arguments must be parenthesized: `(x: (Maybe Int))` not `(x: Maybe a)` (since lowercase `a` would be treated as the next parameter name).

### Default parameters and named arguments

Default parameters use `=`. Named arguments at call site use `:`:

```
greet = (name  greeting = "hello") "{greeting} {name}"
greet "alice"                    -- "hello alice"
greet "alice" greeting: "hi"     -- "hi alice"

pad = (s  width = 20) s | pad_left width
```

Application by juxtaposition. Parens only for grouping:

```
double 5              -- 10
add 3 4               -- 7
double (add 3 4)      -- 14
```

Auto-currying for all-positional functions (no defaults):

```
add3 = add 3          -- (y) 3 + y
add3 7                -- 10
```

Functions with defaults are called when all required params are filled; defaults are not curried past.

## Sections

An operator with one operand missing, wrapped in parens, becomes a function:

```
(+ 1)      -- (x) x + 1
(* 2)      -- (x) x * 2
(> 0)      -- (x) x > 0
(== "a")   -- (x) x == "a"
(.name)    -- (x) x.name
```

Left sections (operand on the left):

```
(10 -)     -- (x) 10 - x
(100 /)    -- (x) 100 / x
```

Sections are the primary mechanism for inline lambdas in pipelines. `filter (> 0)` reads "filter elements greater than zero."

## Pipes

`|` passes the left value as the **last** argument to the right:

```
5 | double                          -- double 5
[1 2 3] | map (* 2)                -- map (* 2) [1 2 3]
data | filter (> 0) | map (* 2) | sum
```

Left-associative. Higher precedence than comparison and logical operators, lower than arithmetic and concatenation. This means pipeline results can be compared directly: `data | sort | len > 5` works without parens.

Pipe to an inline function for multi-step transforms:

```
data | (x) {
  tmp = x | normalize
  tmp | validate
}
```

## Statement Separators

Newlines terminate statements. `;` is an alternative separator for multiple statements on one line:

```
a = 1; b = 2; c = a + b
```

`;` and newline are interchangeable everywhere. Prefer newlines; use `;` only when compactness matters (e.g., `sel` arms on one line).

## Multiline Expressions

Two rules prevent unwanted statement breaks:

**Unclosed delimiters** — inside unmatched `(`, `[`, or `{`, newlines are whitespace:

```
xs = [
  1 2 3
  4 5 6
]

point = {
  x: 3.0
  y: 4.0
}
```

**Continuation operators** — a line starting with a binary operator (`|` `+` `-` `*` `/` `%` `//` `++` `&&` `||` `??` `==` `!=` `<` `>` `<=` `>=` `..` `..=` `~>` `~>?`) continues the previous statement:

```
data
  | filter (> 0)
  | map (* 2)
  | sum
```

A line ending with a binary operator also continues:

```
total = base_price +
  tax +
  shipping
```

## Tuple Auto-Spread

When a function with N parameters receives a single tuple of arity N as its sole argument, the tuple is spread into the parameters automatically:

```
add = (a b) a + b
add (3 4)                       -- 7 (tuple spread into a=3, b=4)

[(1 2) (3 4)] | map (a b) a + b  -- [3 7]
[10 20 30] | enumerate | each (i x) $echo "{i}: {x}"
```

This enables natural composition with `enumerate`, `entries`, `zip`, and other tuple-producing functions. Without it, `each ((i x)) body` would require double parens.

Spread only applies when the function's arity exceeds 1 and the argument is a single tuple matching that arity exactly. A 1-param function receiving a tuple gets the whole tuple as its argument.

## Closures

Functions capture their lexical environment by reference:

```
make_adder = (n) (x) x + n
add5 = make_adder 5
add5 10                     -- 15
```

A closure over a mutable binding sees mutations:

```
counter = () {
  n := 0
  {
    inc: () { n <- n + 1; n }
    get: () n
  }
}

c = counter ()
c.inc ()    -- 1
c.inc ()    -- 2
c.get ()    -- 2
```

## Recursion

Functions reference themselves by name. No special syntax:

```
factorial = (n) n ? {
  0 -> 1
  n -> n * factorial (n - 1)
}
```

Tail calls in tail position are optimized (constant stack space):

```
factorial = (n  acc = 1) n ? {
  0 -> acc
  n -> factorial (n - 1) (n * acc)
}
```

## Concatenation

`++` concatenates strings and lists at runtime:

```
"hello" ++ " world"    -- "hello world"
[1 2] ++ [3 4]         -- [1 2 3 4]
```

For literal construction, spread is preferred: `[..a ..b]`, `"prefix {middle} suffix"`. Use `++` when concatenating values computed at runtime in pipelines.

## Negation

`!` is prefix logical not. `not` is the function form for pipelines: `data | filter (x) !(empty? x)`.

## Cross-References

Impl: [impl-lexer.md](../design/impl-lexer.md), [impl-parser.md](../design/impl-parser.md), [impl-ast.md](../design/impl-ast.md). Grammar: [grammar.md](grammar.md). Tests: suite/01–05.
