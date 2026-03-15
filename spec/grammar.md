# Formal Grammar

EBNF grammar, operator precedence, and keyword list.

## Keywords

Total: **13**

`true` `false` `use` `loop` `break` `par` `sel` `assert` `yield` `emit` `with` `checkpoint` `_`

Additionally, `Protocol` and `MCP` are recognized as declaration keywords when they appear at statement level followed by a type name.

`context` is recognized as a modifier keyword after `with`: `with context key: val { body }`. See [agents-ambient.md](agents-ambient.md).

`caller` is an implicit binding (like `it` in `sel`) available inside agent handler functions. See [agents-clarify.md](agents-clarify.md).

Every other construct uses sigils (`$`, `^`, `?`, `+`, `??`, `<-`, `:=`) or function application. `dbg`, `tap`, `log`, `not`, `defer`, `require`, `identity`, `collect`, `rollback` are built-in functions, not keywords — they can technically be shadowed.

## Built-in Names

These are always in scope but are not reserved words:

`dbg` `tap` `log` `not` `defer` `require` `identity` `collect` `timeout`
`retry` `retry_with` `nat` `cycle` `step`
`map` `filter` `fold` `flat_map` `each` `zip` `zip_with` `enumerate` `partition`
`sort` `sort_by` `rev` `take` `drop` `find` `find_index` `any?` `all?` `none?`
`len` `empty?` `contains?` `get` `first` `last` `sum` `product` `count`
`min` `max` `min_by` `max_by` `uniq` `uniq_by` `flatten` `chunks` `windows`
`join` `split` `scan` `intersperse` `take_while` `drop_while` `group_by`
`trim` `trim_start` `trim_end` `lines` `chars` `byte_len`
`starts?` `ends?` `upper` `lower` `replace` `replace_all` `repeat`
`pad_left` `pad_right`
`keys` `values` `entries` `has_key?` `remove` `merge`
`to_map` `to_record` `to_list` `to_str` `parse_int` `parse_float`
`ok?` `err?` `some?` `even?` `odd?` `sorted?`
`match` `test` `Ok` `Err` `Some` `None`

`log` is a record (not a function) with fields `info`, `warn`, `err`, `debug`, each a logging function. See [stdlib.md](stdlib.md) for full signatures and details.

## Not in v1

Macros, operator overloading, inheritance/class hierarchies, unstructured spawn/await, formal traits/interfaces, decorators, exceptions/try-catch, implicit conversions, duration literals, comprehensions, or-patterns, string patterns, `continue`, format-string mini-language.

## Operator Precedence (high to low)

```
 1.  .           field/index access
 2.  (juxtapose) function application
 3.  - !         unary prefix (negate, logical not)
 4.  * / % //    multiplicative
 5.  + -         additive
 6.  ..  ..=     range
 7.  ++ ~> ~>? ~>>? |>>  concat / agent comm / streaming pipe
 8.  |           pipe
 9.  == != < > <= >=   comparison
10.  &&          logical and
11.  ||          logical or
12.  ??          coalesce (postfix-like binary)
13.  ^           error propagation (postfix)
14.  &           pattern guard (inside ? arms only)
15.  ->          arm body / type arrow
16.  ?           match / ternary
17.  = := <-     binding / assignment
```

`|` is higher precedence than comparison, so `data | sort | len > 5` parses as `((data | sort) | len) > 5` — the pipeline completes before the comparison. `^` and `??` are lower precedence than `|` so they bind to the pipeline result, not to individual functions. `url | fetch ^` parses as `(url | fetch) ^`. `data | fetch ?? default` parses as `(data | fetch) ?? default`. To apply `^` before piping, use parens: `(fs.read path ^) | process`.

All binary operators are left-associative except:
- `->` is right-associative: `a -> b -> c` is `a -> (b -> c)`
- `=` / `:=` / `<-` are non-associative (cannot chain)

`^` is postfix (unary): it applies to the entire expression to its left at its precedence level. `??` is binary but placed below `|` for the same pipeline-composition reason.

## Lexical Grammar

```
IDENT     = [a-z_][a-z0-9_']*[?]?
TYPE      = [A-Z][a-zA-Z0-9]*
INT       = [0-9][0-9_]* | 0x[0-9a-fA-F_]+ | 0b[01_]+ | 0o[0-7_]+
FLOAT     = [0-9][0-9_]*\.[0-9][0-9_]* ([eE][+-]?[0-9]+)?
STR       = " (char | \escape | { EXPR })* "
RAW_STR   = ` char* `
REGEX     = r/ (char | \/)* / [imsx]*
SEP       = NEWLINE | ;
COMMENT   = -- to end of line
```

Underscores in numeric literals are ignored: `1_000_000`, `0xff_ff`. `?` appears at most once, only as the final character of an identifier. `'` can appear in identifiers for mutating variants (`sort'`).

## String Escapes

Inside `"..."` strings:

```
\n   newline
\t   tab
\r   carriage return
\\   backslash
\"   double quote
\{   literal brace (prevents interpolation)
\0   null byte
\u{XXXX}   unicode codepoint
```

Backtick strings (`` ` ``) have no escapes — everything is literal.

## Multiline Continuation

Newlines are statement terminators EXCEPT:

1. Inside unmatched `(`, `[`, `{` — newlines become whitespace
2. When a line starts with a binary operator — continues the previous statement
3. When a line ends with a binary operator — the next line continues

## EBNF Grammar

```
program     = (stmt SEP)*

stmt        = use_stmt | binding | expr

use_stmt    = "use" module_path (":" IDENT)? ("{" IDENT* "}")?
module_path = ("../")*  "./"? IDENT ("/" IDENT)*

binding     = "+"? IDENT "=" expr
            | "+"? IDENT ":" type "=" expr
            | "+"? TYPE IDENT* "=" type_def
            | IDENT ":=" expr
            | IDENT "<-" expr
            | pattern "=" expr
            | "Protocol" TYPE "=" "{" proto_field* "}"
            | "MCP" TYPE "=" "{" mcp_tool* "}"

type_def    = "{" field_type* "}"
            | ("|" TYPE type*)+

expr        = literal | IDENT | TYPE | section
            | expr "." (IDENT | INT | STR)
            | expr "." INT ".." INT?
            | expr "." ".." INT
            | expr expr+ named_arg*
            | expr "|" expr
            | expr "?" "{" arm* "}"
            | expr "?" expr (":" expr)?
            | expr "^"
            | expr "??" expr
            | expr binop expr
            | prefix expr
            | "{" (stmt SEP)* "}"
            | "$" shell_line
            | "$^" shell_line
            | "${" shell_block "}"
            | "par" "{" (stmt SEP)* "}"
            | "sel" "{" sel_arm* "}"
            | "assert" expr expr?
            | "loop" "{" (stmt SEP)* "}"
            | "break" expr?
            | "yield" expr
            | "emit" expr
            | "with" IDENT "=" expr "{" (stmt SEP)* "}"
            | "with" IDENT ":=" expr "{" (stmt SEP)* "}"
            | "with" "context" field* "{" (stmt SEP)* "}"
            | expr "~>" expr
            | expr "~>?" expr
            | expr "~>>?" expr
            | expr "|>>" expr
            | "checkpoint" STR "{" (stmt SEP)* "}"
            | IDENT ("." IDENT)+ "<-" expr
            | "(" params ")" ("->" type)? expr

binop       = "+" | "-" | "*" | "/" | "%" | "//"
            | "++" | "~>" | "~>?" | "~>>?" | "|>>"
            | "==" | "!=" | "<" | ">" | "<=" | ">="
            | "&&" | "||"
            | "??" | ".." | "..="

prefix      = "-" | "!"

arm         = pattern "->" expr SEP
sel_arm     = expr "->" expr SEP

pattern     = literal | IDENT | "_"
            | "{" field_pat* ("." "." IDENT)? "}"
            | "[" pat_elem* "]"
            | "(" pattern* ")"
            | TYPE pattern*
            | pattern "&" "(" expr ")"

pat_elem    = pattern | ".." IDENT

field_pat   = IDENT (":" pattern)?

section     = "(" binop expr ")"
            | "(" expr binop ")"
            | "(" "." IDENT ")"

named_arg   = IDENT ":" expr

shell_line  = raw text with {expr} interpolation until SEP
shell_block = raw text with {expr} interpolation until "}"

params      = (IDENT (":" type)? ("=" expr)?)*

type        = TYPE
            | TYPE type+
            | "[" type "]"
            | "{" field_type* "}"
            | "%{" type ":" type "}"
            | "(" type* ")"
            | type "->" type
            | type "^" type
            | IDENT

field_type  = IDENT ":" type

literal     = INT | FLOAT | STR | RAW_STR | REGEX
            | "true" | "false" | "()"
            | "[" expr* "]"
            | "{" field* "}"
            | "%{" map_entry* "}"
            | "(" expr expr+ ")"

field       = IDENT (":" expr)? | ".." expr
map_entry   = expr ":" expr | ".." expr
```

## Grammar Notes

- `;` and newline are interchangeable statement separators.
- `+` before a top-level binding marks it as exported.
- `$^` is a single token (error-propagating shell prefix).
- `?` at end of identifier (`empty?`) is part of the identifier; `expr ?` (with space) is the match operator.
- `'` in identifiers (`sort'`) marks mutating variants; part of the `IDENT` production.
- `()` is unit (the only 0-tuple). `(expr)` is grouping. `(expr expr+)` is a tuple.
- Inside `?` arms, `&` is a guard operator with the lowest arm-level precedence.
- `..` in patterns (like `[x ..rest]` and `{..p}`) is spread/rest syntax, distinct from the `..` range operator.
- `use` is a statement, not an expression — it cannot appear inside blocks or be used as a value.
- Tuple auto-spread: when a function with N params receives a single N-tuple argument, the tuple is spread. This is a runtime behavior, not a grammar production.
- Named arguments (`name: value`) at call sites are disambiguated from record fields by context: inside `{ }` they are record fields, in application position they are named arguments. Named args can only follow positional args.
- `../` in module paths is a single token (parent directory reference), not separate `.` `.` `/` tokens.
- `yield expr` suspends the current script and sends `expr` to the orchestrator. Execution resumes when the orchestrator provides a response.
- `emit expr` sends `expr` to the human/orchestrator as fire-and-forget output. Returns `()`. Does not block. Strings print directly; records are JSON-encoded.
- `with name = expr { body }` binds `name` to `expr` for the duration of `body` (immutable). `with name := expr { body }` is the mutable variant.
- `with context key: val { body }` establishes ambient context for the block. See [agents-ambient.md](agents-ambient.md).
- `|>>` is a streaming pipe operator — pushes items downstream as they complete. See [concurrency-reactive.md](concurrency-reactive.md).
- `Protocol TypeName = { ... }` defines a typed protocol (set of message/response pairs). `MCP TypeName = { ... }` defines an MCP tool declaration.
- `~>` sends a message to an agent synchronously. `~>?` sends a message and returns `Ok result` or `Err error` instead of propagating failures. `~>>?` sends a message and returns a lazy stream of partial results.
- `checkpoint "name" { body }` snapshots mutable state. `rollback "name"` (built-in function) restores the snapshot and exits the block with `Err`.
- Type annotations are optional on function parameters (`x: Int`), return types (`-> Int`), fallible returns (`-> Int ^ Str`), and bindings (`name: Int = 5`). The checker validates annotations; the interpreter ignores them.
- In type application `TYPE type+`, only uppercase `TYPE` and delimited tokens (`[`, `(`, `%{`) are consumed as arguments. Lowercase identifiers are NOT consumed to avoid ambiguity with parameter names. Use parens for type variable arguments: `(x: (Maybe a))` not `(x: Maybe a)`.
