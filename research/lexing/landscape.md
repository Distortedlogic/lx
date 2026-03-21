# Lexer Design Landscape

A survey of lexer/tokenizer architectures across programming languages, covering implementation strategies, design tradeoffs, and common patterns.

## Table of Contents

1. [Python](#python)
2. [Rust](#rust)
3. [JavaScript](#javascript)
4. [Lua](#lua)
5. [Ruby](#ruby)
6. [Go](#go)
7. [General Patterns](#general-patterns)

---

## Python

### Architecture

CPython's tokenizer lives in two forms: a C implementation (`Parser/tokenize.c`) used by the interpreter, and a pure-Python mirror (`Lib/tokenize.py`) used for tooling. The C tokenizer is a state machine built from conditional branches and `goto` jumps, structured around the central `tok_get` function.

The `tok_get` function processes one UTF-8 byte at a time, storing the current character in an `int` to accommodate EOF as `-1`. Conditional branches are ordered by frequency of occurrence in typical source code—identifiers (NAME tokens) are tested first since they are the most common token type.

### INDENT/DEDENT Generation

Python's most distinctive lexer feature is its handling of significant whitespace. Rather than making the parser context-sensitive, CPython pushes this complexity into the tokenizer, generating synthetic INDENT and DEDENT tokens via a stack-based algorithm:

1. A stack is initialized with a single `0` (which is never popped).
2. At the beginning of each logical line, the line's indentation level is compared to the top of the stack.
3. If equal: nothing happens.
4. If greater: the new level is pushed onto the stack, and one INDENT token is emitted.
5. If less: the stack is popped until a matching level is found. For each value popped, one DEDENT token is emitted. If no matching level exists, the tokenizer reports an indentation error.

At EOF, DEDENT tokens are emitted for every remaining level above zero on the stack.

### Dual Column Counting

Python maintains two parallel column counts to handle mixed tabs and spaces:

- **`col`** (primary): treats tabs as advancing to the nearest multiple of 8 columns; spaces count as 1.
- **`altcol`** (alternate): treats tabs as 1 column, same as spaces.

When comparing indentation levels against the stack, both `col` and `altcol` must agree. This detects inconsistent mixing of tabs and spaces that would render different under different tab-stop settings.

### Bracket Nesting and Continuation Lines

Indentation tracking is disabled inside bracket nesting. A simple counter tracks unmatched `(`, `[`, and `{` characters. When the counter is nonzero, newlines are suppressed (no NEWLINE, INDENT, or DEDENT tokens are emitted), allowing expressions to span multiple lines freely. Backslash-terminated lines are handled similarly.

### Encoding Detection

The tokenizer detects source file encoding before lexing through the `detect_encoding()` function:

1. Checks for a UTF-8 BOM (byte order mark). If found, returns `'utf-8-sig'`.
2. Scans the first two lines for an encoding cookie matching PEP 263 format (e.g., `# -*- coding: utf-8 -*-`).
3. If both BOM and cookie are present but disagree, raises `SyntaxError`.
4. Default encoding is `'utf-8'`.

The first token in the stream is always an ENCODING token declaring the detected encoding.

### Async/Await Context Sensitivity

When `async` and `await` were introduced (Python 3.5), they were initially context-sensitive keywords. The tokenizer uses lookahead: `async` is only tokenized as a keyword when followed by `def`, and `await` is only recognized within async function contexts. (They became unconditional keywords in Python 3.7.)

### Sources

- [Python Lexical Analysis Reference](https://docs.python.org/3/reference/lexical_analysis.html)
- [tokenize module documentation](https://docs.python.org/3/library/tokenize.html)
- [How Python Parses White Space — Jay Conrod](https://jayconrod.com/posts/101/how-python-parses-white-space)
- [A Deep Dive into Python's Tokenizer — Benjamin Woodruff](https://benjam.info/blog/posts/2019-09-18-python-deep-dive-tokenizer/)

---

## Rust

### Architecture

The Rust compiler's lexer is split across two crates with distinct responsibilities:

1. **`rustc_lexer`**: A standalone, hand-written lexer with zero dependencies on the rest of the compiler. It takes a `&str` and produces a sequence of simple tokens, each being a `(TokenKind, length)` pair. It does not report errors as diagnostics—instead, it stores error conditions as flags on the token. It does not intern strings or track source positions.

2. **`rustc_parse::lexer`**: A wrapper around `rustc_lexer` that adds span information (source positions), interns identifiers and literals into the compiler's symbol table, and converts the raw token stream into the `rustc_ast::token::Token` type used by the parser.

This two-layer design means `rustc_lexer` can be reused outside the compiler (e.g., by rust-analyzer for IDE support) without pulling in compiler infrastructure.

### Cursor and Character Consumption

`rustc_lexer` uses a `Cursor` struct that wraps a `Chars` iterator over the input string. Key methods:

- **`bump()`**: Advances to the next character, returning it.
- **`bump_bytes(n)`**: Advances by `n` bytes.
- **`first()`** / **`second()`**: Peek at the next one or two characters without consuming.
- **`eat_while(predicate)`**: Consumes characters while a predicate returns true.
- **`is_eof()`**: Checks if the input is exhausted.

The cursor tracks how many bytes have been consumed, and the token's length is computed as the difference between cursor positions before and after lexing a token.

### TokenKind Enum

The `TokenKind` enum represents all possible token types:

- **Punctuation**: `Semi`, `Comma`, `Dot`, `OpenParen`, `CloseParen`, `OpenBrace`, `CloseBrace`, `OpenBracket`, `CloseBracket`, `At`, `Pound`, `Tilde`, `Question`, `Colon`, `Dollar`, `Eq`, `Bang`, `Lt`, `Gt`, `Minus`, `And`, `Or`, `Plus`, `Star`, `Slash`, `Caret`, `Percent`
- **Literals**: with sub-enum for `Int`, `Float`, `Char`, `Byte`, `Str`, `ByteStr`, `CStr`, `RawStr`, `RawByteStr`, `RawCStr`
- **Identifiers**: `Ident` (including keywords—`rustc_lexer` does not distinguish keywords from identifiers)
- **Lifetime**: `Lifetime` (e.g., `'a`)
- **Whitespace**, **LineComment**, **BlockComment**
- **Unknown**: for unrecognized characters

### Raw String Handling

Raw strings (`r"..."`, `r#"..."#`, etc.) are lexed by counting the number of `#` characters after `r`, then scanning until finding a `"` followed by the same count of `#` characters. The lexer supports up to 255 `#` characters. Raw strings do not process any escape sequences. The same pattern applies to raw byte strings (`br"..."`) and raw C strings (`cr"..."`).

### Unicode and Identifiers

Identifier validation follows Unicode Standard Annex #31, using the `unicode-xid` crate to check `XID_Start` and `XID_Continue` properties. The `rustc_parse` layer includes `unicode_chars.rs`, which maps visually similar Unicode characters to their ASCII equivalents for helpful error messages (e.g., suggesting `=` when the user types a fullwidth equals sign).

### Lifetime Tokens

Lifetime tokens (`'a`, `'static`) start with a single quote `'` followed by an identifier. The lexer must distinguish these from character literals (`'x'`), which also start with `'`. The disambiguation is straightforward: if the character after `'` followed by a non-`'` character sequence ends with `'`, it's a character literal; otherwise, it's a lifetime.

### Sources

- [rustc_lexer documentation](https://doc.rust-lang.org/stable/nightly-rustc/rustc_lexer/index.html)
- [Lexing and Parsing — Rust Compiler Development Guide](https://rustc-dev-guide.rust-lang.org/the-parser.html)
- [rustc_lexer/src/cursor.rs](https://github.com/rust-lang/rust/blob/master/compiler/rustc_lexer/src/cursor.rs)
- [rustc_lexer/src/lib.rs](https://github.com/rust-lang/rust/blob/main/compiler/rustc_lexer/src/lib.rs)
- [Rust Token Reference](https://doc.rust-lang.org/beta/reference/tokens.html)

---

## JavaScript

### The Regex/Division Ambiguity

JavaScript's most notorious lexing challenge is that the `/` character is ambiguous: it can begin a regex literal (`/pattern/flags`), a single-line comment (`//`), a multi-line comment (`/*`), a division operator (`/`), or a division-assignment operator (`/=`).

Pure lexical analysis cannot resolve this ambiguity. The ECMAScript specification requires the parser to determine whether a `/` should be lexed as a regex or division based on the syntactic context. Since there is no parse position where both a regex and a division operator are valid, the ambiguity is resolved by parser-lexer cooperation.

Common strategies:

1. **Previous-token heuristic**: Track the most recently emitted token. If the previous token could end an expression (identifier, number, `)`, `]`, `++`, `--`), treat `/` as division. Otherwise, treat it as regex start. This is an approximation used by lightweight tools.
2. **Two-state lexer**: Maintain explicit `DIVISION_POSSIBLE` and `REGEX_POSSIBLE` states, switched by the parser after each token.
3. **Parser-directed scanning**: The parser tells the lexer which token type to expect at each position. V8 and SpiderMonkey use this approach.

### V8 Scanner Architecture

V8's scanner (`src/parsing/scanner.cc`) converts a Unicode character stream into tokens for the parser. Key design details:

**Encoding**: The scanner operates on UTF-16 encoded input via `UTF16CharacterStream`. A `Scanner::Advance()` wrapper decodes surrogate pairs into full Unicode code points, but only when necessary (e.g., identifier scanning). During string scanning, surrogate pairs are processed as raw UTF-16 code units to avoid unnecessary combining and re-splitting.

**Lookahead**: The scanner uses a maximum lookahead of 4 characters (the longest ambiguous sequence) to determine token types. Once a scan method is selected, it consumes remaining characters and buffers the first non-matching character for the next token.

**Keyword Recognition**: V8 uses `gperf` (GNU perfect hash generator) to identify keywords. The hash function uses the length and first two characters of an identifier to find the single candidate keyword in a lookup table, then performs a full string comparison only on that candidate.

**Identifier Scanning**: Identifiers must start with `ID_Start` characters and continue with `ID_Continue` characters (per Unicode). The scanner maintains an ASCII fast-path lookup table where only `a-z`, `A-Z`, `$`, and `_` are start characters, with `0-9` added for continuation. The `AdvanceUntil` interface provides direct stream access for bulk character consumption, yielding 1.2-1.5x speedup for identifiers.

**String Interning**: All string literals and identifiers are deduplicated at the scanner-parser boundary. Single ASCII character strings use a fast lookup table, leveraging the pattern that minified code frequently uses single-character identifiers.

**Whitespace**: Handled in a separate helper method that returns immediately when the current character is not whitespace, minimizing branch overhead. Whitespace tracking is also used to detect automatic semicolon insertion triggers.

### V8 Template Literal Handling

Template literals require the lexer to handle nested expression contexts. V8's `ScanTemplateSpan()` function recognizes two token types:

- **`TEMPLATE_SPAN`**: Matches `` ` chars* ${ `` or `` } chars* ${ `` (template parts with embedded expressions)
- **`TEMPLATE_TAIL`**: Matches `` ` chars* ` `` or `` } chars* ` `` (final template segment)

The scanner maintains both `literal_chars` and `raw_literal_chars` buffers in its `TokenDesc` structure, since template literals expose both cooked (escape-processed) and raw content via the tagged template API.

Template literal scanning requires the parser's cooperation: when the scanner encounters `${`, it returns `TEMPLATE_SPAN` and the parser takes over to parse the embedded expression. When the parser finishes, it tells the scanner to resume template scanning from the `}`.

### SpiderMonkey

Mozilla's SpiderMonkey uses a similar parser-directed approach. The lexer handles compound operators with nested `if` statements and peek functionality, and coordinates with the parser for context-sensitive decisions.

### Performance Benchmarks (V8 Scanner Optimizations)

Measured improvements from V8's scanner optimization work:
- Single token scanning: **1.4x faster**
- String scanning: **1.3x faster**
- Multiline comments: **2.1x faster**
- Identifier scanning: **1.2-1.5x faster**

### Sources

- [Blazingly fast parsing, part 1: optimizing the scanner — V8](https://v8.dev/blog/scanner)
- [Blazingly fast parsing, part 2: lazy parsing — V8](https://v8.dev/blog/preparser)
- [V8 scanner.cc source](https://github.com/v8/v8/blob/main/src/parsing/scanner.cc)
- [JavaScript 2.0 Lexer — Mozilla Archive](https://www-archive.mozilla.org/js/language/js20-2000-07/core/lexer)
- [Lexer — Write a JavaScript Parser in Rust (OXC)](https://oxc-project.github.io/javascript-parser-in-rust/docs/lexer/)

---

## Lua

### Architecture

Lua's lexer (`llex.c`) is one of the simplest production-language lexers in existence, reflecting Lua's design philosophy of minimalism. The entire lexer is under 600 lines of C.

### Core Structure

The main `llex()` function is a straightforward `switch` on the current character (`ls->current`). Each case dispatches to either inline handling or a dedicated helper function. Single-character tokens (operators, delimiters) return their ASCII value directly as the token type, avoiding the need for a separate enum mapping.

Key helper macros:
- **`next(ls)`**: Advances to the next character (`ls->current = zgetc(ls->z)`)
- **`currIsNewline(ls)`**: Checks for `\n` or `\r`
- **`save_and_next(ls)`**: Saves the current character to a buffer and advances

### Token Types

The `RESERVED` enum defines token types for multi-character tokens and keywords:
- Keywords: `and`, `break`, `do`, `else`, `elseif`, `end`, `false`, `for`, `function`, `goto`, `if`, `in`, `local`, `nil`, `not`, `or`, `repeat`, `return`, `then`, `true`, `until`, `while`
- Multi-character operators: `//`, `..`, `...`, `==`, `>=`, `<=`, `~=`, `<<`, `>>`
- Literal types: `TK_INT`, `TK_FLT`, `TK_NAME`, `TK_STRING`

Single-character tokens use their ASCII value directly (e.g., `+` is token type 43).

### Keyword Recognition

During initialization (`luaX_init`), all reserved words are pre-interned into the global string table with their `extra` field set to mark them as reserved. When the lexer encounters an identifier, it interns the string and checks the `extra` field—if set, the token is a keyword rather than a name. This piggybacks on Lua's existing string interning infrastructure.

### Long Strings and Long Comments

Long strings use the bracket notation `[==[...]==]` where the number of `=` signs must match between opening and closing brackets. The `read_long_string()` function:

1. `skip_sep()` counts the `=` signs in the opening bracket, producing a separator count `sep`.
2. Characters are consumed until a matching closing bracket with the same separator count is found.
3. Embedded newlines are normalized and line numbers are tracked via `inclinenumber()`.
4. The final string value excludes the bracket delimiters.

Long comments follow the same pattern: when `--[` is encountered, `skip_sep()` checks for bracket structure. If `sep >= 2` (valid long bracket), `read_long_string()` is called with `NULL` for the semantic info parameter, causing content to be discarded rather than stored. Short comments simply scan to end-of-line.

### Number Parsing

The `read_numeral()` function takes a permissive approach:
1. If the number starts with `0x` or `0X`, it reads hexadecimal digits.
2. It accepts digits, letters (for hex), dots, and exponent characters (`e`, `E`, `p`, `P` with optional sign).
3. The accumulated string is passed to `luaO_str2num()` for actual validation and conversion.
4. Returns `TK_INT` or `TK_FLT` depending on the conversion result.

This strategy keeps the lexer simple by deferring format validation to the conversion function.

### Lookahead

The lexer supports single-token lookahead via `luaX_lookahead()`, which stores the next token in `ls->lookahead`. The main `luaX_next()` function checks for a buffered lookahead token before calling `llex()`.

### Line Number Tracking

The `inclinenumber()` function handles all newline variants: `\n`, `\r`, `\n\r`, and `\r\n` (treating two-character sequences as a single newline). Line numbers are tracked globally in the `LexState` structure.

### Sources

- [Lua 5.4 llex.c source](https://www.lua.org/source/5.4/llex.c.html)
- [Lua 5.4 llex.h source](https://www.lua.org/source/5.4/llex.h.html)
- [Lua 5.3 llex.c source](https://www.lua.org/source/5.3/llex.c.html)

---

## Ruby

### Architecture

Ruby has one of the most complex lexers among mainstream languages, a direct consequence of Ruby's context-sensitive syntax. The lexer and parser are interleaved in the `parse.y` file—the lexer function (`yylex`) is called by the Bison-generated parser as part of the parsing process, not as a separate preprocessing step.

The `parse.y` file exceeds 14,000 lines and combines grammar rules, lexer code, and semantic actions. Only 65 people have ever contributed to it in 25 years.

### Lexical State Machine (lex_state)

The central mechanism for context-sensitive lexing is the `lex_state` variable, an enum (`lex_state_e`) with the following states:

| State | Meaning |
|-------|---------|
| `EXPR_BEG` | Beginning of an expression |
| `EXPR_END` | After a complete expression |
| `EXPR_ENDARG` | End of argument list |
| `EXPR_ENDFN` | End of function definition |
| `EXPR_ARG` | Argument position |
| `EXPR_CMDARG` | First argument of a command-style call |
| `EXPR_MID` | Middle of expression (after `return`, `break`, etc.) |
| `EXPR_FNAME` | Function/method name context |
| `EXPR_DOT` | After a dot operator |
| `EXPR_CLASS` | After `class` keyword |
| `EXPR_LABEL` | Label context |
| `EXPR_LABELED` | After a label |
| `EXPR_FITEM` | Method name after `alias` |

Different states cause identical characters to produce different tokens. For example:

- **`*`**: In `EXPR_BEG`, produces `tSTAR` (splat operator). In `EXPR_ARG`, produces `'*'` (multiplication).
- **`/`**: In `EXPR_BEG`, begins a regex literal. In `EXPR_END`, produces division.
- **`+`/`-`**: In `EXPR_BEG`, produces `tUPLUS`/`tUMINUS` (unary). In `EXPR_ARG`, produces `'+'`/`'-'` (binary).
- **`::`**: After space or at expression beginning, produces `tCOLON3` (top-level constant access). In dot context, produces `tCOLON2` (namespace separator).

### Heredoc Handling

Heredocs use a sophisticated two-phase approach:

**Phase 1 — `heredoc_identifier()`**: When `<<IDENTIFIER` is encountered:
1. The identifier is captured and stored in a `NODE_HEREDOC` structure.
2. The current position in the line is saved.
3. `lex_p` is advanced to the end of the current line (skipping past any remaining tokens on the `<<` line).

**Phase 2 — `here_document()`**: On subsequent calls to the lexer:
1. Lines are accumulated until the ending identifier is found on a line by itself.
2. `heredoc_restore()` rewinds `lex_p` back to the saved position after the `<<` marker.
3. Scanning resumes from where it left off on the original line.

This mechanism allows syntax like `printf(<<EOS, n)` where the heredoc content appears on subsequent lines but the `,` and `)` are parsed from the original line.

### Regex Literal Disambiguation

Like JavaScript, Ruby must disambiguate `/` between division and regex start. Ruby uses `lex_state`: if the lexer is in a state where an operator is expected (after an expression), `/` is division. If an operand is expected (beginning of expression), `/` begins a regex literal.

An additional complication: the expression `a /b#/` parses differently depending on whether `a` is a local variable (division of `a` by `b`) or a method name (method call with regex argument `/b#/`). The lexer must track which identifiers have been declared as local variables.

### Keyword Recognition

Ruby uses `gperf` to generate a perfect hash function (`rb_reserved_word()`) for its approximately 50 reserved words. The generated function returns a `struct kwtable` containing:
- `name`: the keyword string
- `id[0]`: standard token ID (e.g., `kIF`)
- `id[1]`: modifier form token ID (e.g., `kIF_MOD` for postfix `if`)
- `state`: the `lex_state` to transition to after consuming the keyword

### Input Buffer

Ruby uses a three-pointer system (`lex_pbeg`, `lex_p`, `lex_pend`) for line-by-line input management. When `lex_p` reaches `lex_pend`, the `nextc()` function calls `lex_getline()` to load the next line, abstracting over different input sources (strings via `lex_get_str()`, IO via `rb_io_gets()`). CRLF normalization happens at this level.

### The Prism Parser (YARP)

As of Ruby 3.3+ (2023-2024), Shopify developed a new parser called Prism (originally YARP — Yet Another Ruby Parser) that replaces the Bison-generated parser with a hand-written recursive descent parser in C. Prism was motivated by:

- **Error tolerance**: The old parser stopped at the first syntax error. Prism uses missing-token insertion, missing-node insertion, and context-based recovery to continue past errors.
- **Portability**: The old parser was coupled to CRuby internals. Prism has no dependencies and can be used by JRuby, TruffleRuby, and tooling.
- **Performance**: Hand-written parsers allow compiler optimizations (inlining, branch prediction) that generated jump tables prevent.
- **Maintainability**: Prism ships with documentation and a standardized AST, replacing the opaque 14,000-line `parse.y`.

Prism supports 23 of CRuby's 90 ASCII-compatible encodings and embeds its own regex parser for named capture group variable resolution.

### Sources

- [Parser — Ruby Hacking Guide](https://ruby-hacking-guide.github.io/parser.html)
- [Rewriting the Ruby Parser — Rails at Scale](https://railsatscale.com/2023-06-12-rewriting-the-ruby-parser/)
- [Prism in 2024 — Rails at Scale](https://railsatscale.com/2024-04-16-prism-in-2024/)
- [Lexers, Parsers, and ASTs — SitePoint](https://www.sitepoint.com/lexers-parsers-and-asts-oh-my-how-ruby-executes/)
- [Fast Tokenizers with StringScanner — Tenderlove](https://tenderlovemaking.com/2023/09/02/fast-tokenizers-with-stringscanner/)

---

## Go

### Architecture

Go maintains two separate scanner implementations:

1. **`go/scanner`** (standard library): Used by tooling (`go vet`, `gofmt`, etc.). Provides a public API for tokenizing Go source.
2. **`cmd/compile/internal/syntax/scanner.go`** (compiler): The compiler's internal scanner, optimized for compilation speed.

Both are hand-written scanners using a simple character-by-character loop.

### Scanner Structure

The compiler's scanner initializes with three components:
- **Source reader**: Buffered input with optimized character access via a `buf` array and three index pointers (`b`, `r`, `e`).
- **Scanning mode**: Controls whether comments are reported as tokens.
- **Semicolon state (`nlsemi`)**: Boolean flag tracking whether automatic semicolon insertion is eligible.

### Automatic Semicolon Insertion

Go's grammar requires semicolons, but programmers almost never write them. The scanner inserts them automatically using a simple rule:

**The Rule**: When the scanner encounters a newline (or EOF), it inserts a semicolon if the last token emitted was one of:
- An identifier (including keywords like `break`, `continue`, `fallthrough`, `return`)
- A literal (integer, float, imaginary, rune, string)
- One of: `break`, `continue`, `fallthrough`, `return`
- `++` or `--`
- `)`, `]`, or `}`

The scanner tracks this via the `nlsemi` flag: after emitting a token from the above list, `nlsemi` is set to `true`. When a newline is then encountered and `nlsemi` is true, a semicolon token is emitted instead of whitespace.

When the returned token is a semicolon, the literal string is `";"` if the semicolon was present in the source, and `"\n"` if it was inserted by the scanner.

### Token Recognition

The scanner uses a multi-stage dispatch:

1. **Character classification**: Letters trigger the identifier/keyword path. Digits trigger number scanning. Symbols/operators use a switch statement with lookahead.
2. **Identifier processing**: The `ident()` method reads a complete word, then checks a `keywordMap` hash table. Non-keywords return as `_Name` tokens with the literal text preserved.
3. **Operator lookahead**: For ambiguous operators like `+` (which could be `+`, `++`, or `+=`), the scanner peeks at the next character without consuming it.

### Unicode Handling

The scanner distinguishes between ASCII and Unicode with a fast-path check:
- Characters below `utf8.RuneSelf` (0x80) take the ASCII fast path.
- Characters at or above this threshold trigger UTF-8 rune decoding.
- The `atIdentChar()` method validates Unicode identifier characters using Go's `unicode` package.

### Design Philosophy

Go's grammar was intentionally designed for simple parsing. The automatic semicolon insertion rule was chosen to be implementable as a single boolean flag check in the scanner, with no parser feedback needed. This is possible because Go's grammar ensures that the list of "semicolon-triggering" tokens is static and context-free.

### Sources

- [go/scanner package documentation](https://pkg.go.dev/go/scanner)
- [Understanding the Go Compiler: The Scanner](https://internals-for-interns.com/posts/the-go-lexer/)
- [Automatic Semicolon Insertion in Go — golangspec](https://medium.com/golangspec/automatic-semicolon-insertion-in-go-1990338f2649)
- [Go scanner source (compiler)](https://github.com/golang/go/blob/master/src/cmd/compile/internal/syntax/scanner.go)
- [Go scanner source (stdlib)](https://go.dev/src/go/scanner/scanner.go)

---

## General Patterns

### Handwritten vs. Generated Lexers

Most production compilers use hand-written lexers. Of the languages surveyed:

| Language | Lexer Type | Generator |
|----------|-----------|-----------|
| Python | Handwritten | — |
| Rust | Handwritten | — |
| JavaScript (V8) | Handwritten | — |
| Lua | Handwritten | — |
| Ruby (CRuby) | Handwritten (in parse.y) | — |
| Ruby (Prism) | Handwritten | — |
| Go | Handwritten | — |

Lexer generators remain useful for prototyping and for languages where the lexer specification changes frequently, but the trend in production compilers is overwhelmingly toward hand-written lexers for control over performance, error reporting, and context sensitivity.

**Flex** (table-driven): Generates DFA-based lexers using lookup tables. The DFA is computed at generator time from regex specifications. Each input character requires a table lookup to determine the next state. Produces relatively large tables but guarantees O(n) scanning.

**re2c** (direct-coded): Generates lexers that encode the DFA as conditional jumps and comparisons rather than table lookups. The resulting code is faster (benchmarked at ~2x faster than Flex) and often smaller, because the compiler can optimize the generated branch structures. re2c generates code for C, C++, D, Go, Haskell, Java, JavaScript, OCaml, Python, Rust, Swift, V, and Zig.

**Performance comparison**: Direct-coded lexers (re2c) consistently outperform table-driven (Flex) because:
1. Table lookups incur cache misses on large state tables.
2. Branch prediction works well on the conditional jumps that direct-coded lexers produce.
3. The compiler can further optimize direct-coded output (e.g., collapsing redundant checks).

Hand-written lexers can match or exceed re2c performance because the developer can exploit language-specific knowledge (e.g., V8's ASCII fast path for identifiers, Lua's direct ASCII-value-as-token-type optimization).

### DFA-Based vs. Ad-Hoc

Even hand-written lexers are implicitly DFA-based—they just implement the DFA as explicit code rather than as a table. The typical structure is a top-level switch on the first character, with sub-switches or if-chains for multi-character tokens.

The key difference from generator-based DFAs is that hand-written lexers can "cheat":
- They can use different strategies for different token types (e.g., a tight loop for identifiers, a state machine for strings).
- They can incorporate semantic checks mid-scan (e.g., Ruby checking `lex_state`).
- They can call helper functions that maintain their own local state.

### Lazy/On-Demand Tokenization

Most production lexers are lazy: they produce one token at a time on demand from the parser, rather than tokenizing the entire input upfront. This is the natural design for recursive descent parsers, where the parser calls a `next_token()` or `advance()` method.

Benefits:
- **Memory**: Only the current token (and possibly one lookahead) needs to be stored, not the entire token stream.
- **Performance**: Tokens that are never needed (e.g., in skipped branches during preparsing) are never produced.
- **Streaming**: The lexer can operate on input that arrives incrementally.

Exceptions: Some tools (e.g., formatters, syntax highlighters) tokenize the entire file upfront because they need random access to the token stream.

### Token Position Tracking (Spans)

All production lexers track source positions for error reporting. Common approaches:

1. **Byte offsets**: Store the start byte offset and length of each token. The cheapest representation—just two integers. Line/column information is computed on demand from a separate line-offset table. Used by Rust (`rustc_lexer`, `Span`), tree-sitter.

2. **Line/column pairs**: Store line number and column for start and end of each token. More expensive (4 integers per token) but avoids needing a line-offset table. Common in simpler implementations.

3. **Interned spans**: Rust's compiler interns spans into a global table (`SourceMap`) and uses a compact `Span` type (4 bytes: a packed byte offset + length, or an index into a side table for large spans).

4. **Single offset + token length**: `rustc_lexer` takes this approach—it reports only token lengths, and the wrapping layer accumulates byte offsets to construct spans. This keeps the low-level lexer position-agnostic.

### String Interning

String interning deduplicates identifier and literal strings, storing each unique string once and using cheap integer handles thereafter.

- **Rust**: All identifiers and keywords are interned into a `Symbol` table. Comparison becomes integer comparison.
- **V8**: Strings are deduplicated at the scanner-parser boundary. Single ASCII character strings use a fast lookup table.
- **Lua**: All strings (including keywords) go through a global string table (`TString`). Keywords are identified by checking the `extra` field on the interned string.

Benefits: Reduced memory usage, O(1) equality comparison, and cache-friendly token storage.

### Keyword Recognition Strategies

| Strategy | Description | Used By |
|----------|------------|---------|
| **Perfect hashing** (gperf) | Compile-time generated hash function with zero collisions. Maps keyword to table index in O(1) with one comparison. | V8, Ruby (CRuby) |
| **Hash table lookup** | Runtime hash table of keywords. O(1) amortized but requires full string hashing. | Go (`keywordMap`) |
| **String interning + flag** | Intern all identifiers; keywords have a flag set during initialization. O(1) after interning. | Lua |
| **Trie / switch cascade** | Walk a character-by-character decision tree. Can be faster for small keyword sets. | Some hand-written lexers |
| **Length + first chars** | Filter candidates by length and initial characters before comparing. | V8 (gperf uses this) |

For small keyword sets (< 50), the strategy barely matters. For larger sets or performance-critical paths, perfect hashing or interning-based approaches dominate.

### Sources

- [re2c — Regular Expressions to Code](https://re2c.org/)
- [Flex (lexical analyzer generator) — Wikipedia](https://en.wikipedia.org/wiki/Flex_(lexical_analyser_generator))
- [re2c — Wikipedia](https://en.wikipedia.org/wiki/Re2c)
- [Tries and Lexers — ircmaxell](https://blog.ircmaxell.com/2015/05/tries-and-lexers.html)
- [Fast Scanning: Detecting Keywords — Michal Pitr](https://medium.com/@michalpitr/fast-scanning-detecting-keywords-c58bd64befeb)
- [Lexical Analysis — Wikipedia](https://en.wikipedia.org/wiki/Lexical_analysis)
