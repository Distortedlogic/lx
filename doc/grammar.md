# Grammar — Reference

## Keywords (13)

`true` `false` `use` `loop` `break` `par` `sel` `assert` `yield` `emit` `with` `checkpoint` `_`

`Protocol`/`MCP` are declaration keywords. `context` modifies `with`. `caller` is implicit in agent handlers.

## Built-in Names

`dbg` `tap` `log` `not` `defer` `require` `identity` `collect` `timeout` `retry` `retry_with` `nat` `cycle` `step` `map` `filter` `fold` `flat_map` `each` `zip` `zip_with` `enumerate` `partition` `sort` `sort_by` `rev` `take` `drop` `find` `find_index` `any?` `all?` `none?` `len` `empty?` `contains?` `get` `first` `last` `sum` `product` `count` `min` `max` `min_by` `max_by` `uniq` `uniq_by` `flatten` `chunks` `windows` `join` `split` `scan` `intersperse` `take_while` `drop_while` `group_by` `trim` `trim_start` `trim_end` `lines` `chars` `byte_len` `starts?` `ends?` `upper` `lower` `replace` `replace_all` `repeat` `pad_left` `pad_right` `keys` `values` `entries` `has_key?` `remove` `merge` `to_map` `to_record` `to_list` `to_str` `parse_int` `parse_float` `ok?` `err?` `some?` `even?` `odd?` `sorted?` `match` `test` `Ok` `Err` `Some` `None`

`log` is a record with fields `info`, `warn`, `err`, `debug`.

## Operator Precedence (high to low)

```
 1.  .             field/index access
 2.  (juxtapose)   function application
 3.  - !           unary prefix
 4.  * / % //      multiplicative
 5.  + -           additive
 6.  ..  ..=       range
 7.  ++ ~> ~>? ~>>? |>>  concat / agent / streaming
 8.  |             pipe
 9.  == != < > <= >=  comparison
10.  &&            logical and
11.  ||            logical or
12.  ??            coalesce
13.  ^             error propagation (postfix)
14.  &             pattern guard (? arms only)
15.  ->            arm body / type arrow
16.  ?             match / ternary
17.  = := <-       binding / assignment
```

All left-associative except `->` (right) and `=`/`:=`/`<-` (non-associative). `^`/`??` below `|`: `url | fetch ^` = `(url | fetch) ^`.

## EBNF Grammar

```
program     = (stmt SEP)*
stmt        = use_stmt | binding | expr
use_stmt    = "use" module_path (":" IDENT)? ("{" IDENT* "}")?
module_path = ("../")*  "./"? IDENT ("/" IDENT)*
binding     = "+"? IDENT "=" expr | "+"? IDENT ":" type "=" expr
            | "+"? TYPE IDENT* "=" type_def | IDENT ":=" expr | IDENT "<-" expr
            | pattern "=" expr | "Protocol" TYPE "=" "{" proto_field* "}"
            | "MCP" TYPE "=" "{" mcp_tool* "}"
type_def    = "{" field_type* "}" | ("|" TYPE type*)+
expr        = literal | IDENT | TYPE | section
            | expr "." (IDENT | INT | STR) | expr "." INT ".." INT? | expr "." ".." INT
            | expr expr+ named_arg* | expr "|" expr
            | expr "?" "{" arm* "}" | expr "?" expr (":" expr)?
            | expr "^" | expr "??" expr | expr binop expr | prefix expr
            | "{" (stmt SEP)* "}" | "$" shell_line | "$^" shell_line | "${" shell_block "}"
            | "par" "{" (stmt SEP)* "}" | "sel" "{" sel_arm* "}"
            | "assert" expr expr? | "loop" "{" (stmt SEP)* "}" | "break" expr?
            | "yield" expr | "emit" expr
            | "with" IDENT ("=" | ":=") expr "{" (stmt SEP)* "}"
            | "with" "context" field* "{" (stmt SEP)* "}"
            | expr ("~>" | "~>?" | "~>>?" | "|>>") expr
            | "checkpoint" STR "{" (stmt SEP)* "}"
            | IDENT ("." IDENT)+ "<-" expr
            | "(" params ")" ("->" type)? expr
binop       = "+" | "-" | "*" | "/" | "%" | "//" | "++" | "~>" | "~>?" | "~>>?" | "|>>"
            | "==" | "!=" | "<" | ">" | "<=" | ">=" | "&&" | "||" | "??" | ".." | "..="
arm         = pattern "->" expr SEP
sel_arm     = expr "->" expr SEP
pattern     = literal | IDENT | "_" | "{" field_pat* ("." "." IDENT)? "}"
            | "[" pat_elem* "]" | "(" pattern* ")" | TYPE pattern*
            | pattern "&" "(" expr ")"
section     = "(" binop expr ")" | "(" expr binop ")" | "(" "." IDENT ")"
named_arg   = IDENT ":" expr
params      = (IDENT (":" type)? ("=" expr)?)*
type        = TYPE | TYPE type+ | "[" type "]" | "{" field_type* "}"
            | "%{" type ":" type "}" | "(" type* ")" | type "->" type | type "^" type | IDENT
literal     = INT | FLOAT | STR | RAW_STR | REGEX | "true" | "false" | "()"
            | "[" expr* "]" | "{" field* "}" | "%{" map_entry* "}" | "(" expr expr+ ")"
field       = IDENT (":" expr)? | ".." expr
map_entry   = expr ":" expr | ".." expr
```

## Notes

- `+` before binding = exported. `$^` = error-propagating shell
- `()` = unit, `(expr)` = grouping, `(expr expr+)` = tuple
- `..` in patterns = spread/rest (distinct from range `..`)
- Named args follow positional args; disambiguated from record fields by context
