# Syntax — Reference

## Lexical Grammar

```
IDENT     = [a-z_][a-z0-9_']*[?]?
TYPE      = [A-Z][a-zA-Z0-9]*
INT       = [0-9][0-9_]* | 0x[0-9a-fA-F_]+ | 0b[01_]+ | 0o[0-7_]+
FLOAT     = [0-9][0-9_]*\.[0-9][0-9_]* ([eE][+-]?[0-9]+)?
STR       = " (char | \escape | { EXPR })* "
RAW_STR   = ` char* `
REGEX     = r/ (char | \/)* / [imsx]*
SEP       = \n | ;
COMMENT   = -- to end of line
```

No commas. Whitespace separates. `?` only as final ident char. `'` for mutating variants (`sort'`).

## Literals

```
42  3.14  0xff  0b1010  0o77     -- numeric
true false  ()                    -- bool, unit
"hello {name}"                    -- interpolated string
`raw {not interpolated}`          -- raw string
r/\d+/i                           -- regex (flags: i m s x)
```

Multi-line strings strip indentation based on closing delimiter's column.

## Bindings

```
x = 5          -- immutable
x := 5         -- mutable
x <- 10        -- reassign mutable
x: Int = 5     -- with type annotation
```

## Functions

```
double = (x) x * 2
add = (x y) x + y
process = (data) { data | filter (!= "") | sort | uniq }
```

Type annotations: `(x: Int y: Int) -> Int x + y`. Fallible: `-> Int ^ Str`. Complex types need parens: `(x: (Maybe Int))`.

Defaults + named args: `greet = (name  greeting = "hello") "{greeting} {name}"` / `greet "alice" greeting: "hi"`.

Auto-currying for all-positional functions: `add3 = add 3`.

## Sections

```
(+ 1)  (* 2)  (> 0)  (.name)  (10 -)
```

Operator with one operand missing becomes a function. Primary inline lambda mechanism.

## Pipes

`|` passes left value as **last** argument: `data | filter (> 0) | map (* 2) | sum`

## Tuple Auto-Spread

N-param function receiving single N-tuple spreads automatically:
`add (3 4)` = 7. `[(1 2) (3 4)] | map (a b) a + b` = `[3 7]`.

## Closures

Functions capture lexical environment: `make_adder = (n) (x) x + n`.

## Recursion

Functions reference themselves by name. Tail calls optimized.

```
factorial = (n  acc = 1) n ? { 0 -> acc; n -> factorial (n - 1) (n * acc) }
```

## Concatenation

`++` concatenates strings/lists: `"a" ++ "b"`, `[1] ++ [2]`. Prefer spread for literals.

## Multiline Rules

- Inside unclosed `(` `[` `{`, newlines are whitespace
- Line starting/ending with binary operator continues previous statement

## Negation

`!` prefix logical not. `not` function form for pipelines.
