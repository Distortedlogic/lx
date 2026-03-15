# Lexer Design

The lexer is a modal state machine. It reads `&str` source and produces `Vec<Token>`. Four modes handle lx's context-sensitive lexing.

## Modes

```
Normal      -- default: identifiers, operators, literals, keywords
Shell       -- after `$`/`$^`: raw text until newline, with {expr} holes
ShellBlock  -- after `${`: raw text until `}`, with {expr} holes
StringInterp -- inside `"..."`: literal chars until `{` (re-enter Normal) or `"`
Regex       -- after `r/`: raw chars until unescaped `/`, then flag chars
```

### Mode Transitions

```
Normal:
  sees `$`  followed by `^`  → emit Token::DollarCaret, enter Shell
  sees `$`  followed by `{`  → emit Token::DollarBrace, enter ShellBlock
  sees `$`  followed by other → emit Token::Dollar, enter Shell
  sees `"`                   → emit Token::StrStart, enter StringInterp
  sees `r` followed by `/` (no space) → enter Regex
  sees `` ` ``               → scan raw string to closing `` ` ``, emit Token::RawStr

Shell:
  sees `{`         → push Shell onto mode stack, enter Normal (interpolation)
  sees newline/`;` → emit Token::ShellText, pop to Normal
  otherwise        → accumulate into shell text buffer

ShellBlock:
  sees `{`                   → if preceded by interpolation context: push, enter Normal
  sees `}` at block depth 0  → emit Token::ShellText, emit Token::BraceClose, pop to Normal
  otherwise                  → accumulate into shell text buffer

StringInterp:
  sees `{`   → emit Token::StrChunk for accumulated text, push, enter Normal
  sees `"`   → emit Token::StrChunk + Token::StrEnd, pop to Normal
  sees `\`   → process escape sequence (\n, \t, \\, \", \{, \0, \u{XXXX})
  otherwise  → accumulate into string buffer

Regex:
  sees unescaped `/` → scan trailing [gimsux]* flags, emit Token::Regex, pop to Normal
  sees `\`           → include next char literally (escaped)
  otherwise          → accumulate into regex buffer

Normal (returning from interpolation):
  when `}` is encountered and mode stack depth > 0 → emit Token::BraceClose, pop to previous mode
```

The **mode stack** handles nesting: `"hello {$^cmd | f} world"` pushes StringInterp, enters Normal for `$^cmd | f`, pushes Shell inside that, pops back to Normal, pops back to StringInterp.

## Token Types

```rust
enum TokenKind {
    // Literals
    Int(BigInt),
    Float(f64),
    StrStart,           // opening "
    StrChunk(String),   // text between interpolation holes
    StrEnd,             // closing "
    RawStr(String),     // `...` complete raw string
    Regex(String),          // pattern with flags prepended as (?flags)
    True,
    False,
    Unit,               // ()

    // Identifiers and types
    Ident(String),      // lowercase: foo, empty?, sort'
    TypeName(String),   // uppercase: Int, MyType, Ok, Err, Some, None

    // Operators
    Plus, Minus, Star, Slash, Percent, IntDiv,   // + - * / % //
    PlusPlus,                                       // ++
    Eq, NotEq, Lt, Gt, LtEq, GtEq,             // == != < > <= >=
    And, Or,                                      // && ||
    Pipe,                                         // |
    QQ,                                           // ??
    Caret,                                        // ^
    Amp,                                          // &
    Arrow,                                        // ->
    Question,                                     // ?
    Bang,                                         // !
    Dot,                                          // .
    DotDot, DotDotEq,                            // .. ..=
    Assign, DeclMut, Reassign,                   // = := <-
    Colon,                                        // :

    // Delimiters
    LParen, RParen,                              // ( )
    LBracket, RBracket,                          // [ ]
    LBrace, RBrace,                              // { }
    PercentLBrace,                               // %{

    // Shell
    Dollar,                                       // $
    DollarCaret,                                 // $^
    DollarBrace,                                 // ${
    ShellText(String),                           // raw shell text (between interpolation holes)

    // Keywords
    Use, Loop, Break, Par, Sel, Assert, Underscore, Yield, With,
    TildeArrow, TildeArrowQ,   // ~> ~>?
    Protocol, Mcp,             // declaration keywords

    // Structure
    Export,                                       // + at column 0
    Semi,                                         // ; or newline (normalized)
    Eof,
}
```

## Newline Handling

Newlines become `Semi` tokens EXCEPT:
1. Inside unmatched `(`, `[`, `{` — suppressed (track delimiter depth)
2. When the next line starts with a continuation operator (`|`, `+`, `-`, `*`, `/`, `%`, `//`, `++`, `&&`, `||`, `??`, `==`, `!=`, `<`, `>`, `<=`, `>=`, `..`, `..=`, `~>`, `~>?`) — suppress the preceding newline
3. When the current line ends with a binary operator — suppress

Implementation: the lexer tracks delimiter depth. On newline, if depth > 0, skip. Otherwise, peek at the next non-whitespace token. If it's a continuation operator, skip. Otherwise emit `Semi`.

## `+` Export Detection

`+` at column 0 (byte offset 0 on the line) followed by an identifier emits `Export`. `+` anywhere else is the addition operator. The lexer tracks column position.

## `?` in Identifiers

`empty?` is a single `Ident` token. The `?` is included when it follows `[a-z0-9_']` with no space. `expr ?` (space before `?`) emits the expression tokens then `Question`.

## Multiline String Indent Stripping

Handled at lex time. When a `StrStart` is followed by a newline, the lexer:
1. Records the column of the closing `"` when found
2. Strips that many leading whitespace chars from each `StrChunk`
3. Removes the first newline after `"` and the last newline before closing `"`

## Span Tracking

Every token carries `Span { offset: u32, len: u16 }` — byte offset into the source string and byte length. Diagnostics use spans to underline the relevant source. The lexer increments offset as it scans. Shell text and string chunks track their own spans.

## Error Recovery

On unrecognized character, the lexer emits a diagnostic (`error[parse]: unexpected character`) and skips to the next whitespace/delimiter. Lexing continues — a single bad character doesn't abort the file.
