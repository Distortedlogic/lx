# Diagnostic Design Patterns

Principles, infrastructure, rendering techniques, and design decisions for
building a world-class diagnostic system.

---

## 1. Source Spans and Locations

### Span Representation

A span is a half-open byte range `[start, end)` into a source file. This is the
fundamental unit of location information attached to every AST node, token, and
diagnostic.

```
pub struct Span {
    pub start: u32,  // byte offset, inclusive
    pub end: u32,    // byte offset, exclusive
}
```

**Why byte offsets, not line/column?** Byte offsets are:
- O(1) to create (the parser already tracks position)
- O(1) to combine (union of two spans = `min(start), max(end)`)
- Stable across display transformations (tab width, Unicode width)
- Compact (two u32s = 8 bytes)

Line and column are derived *on demand* when rendering diagnostics, not stored.

### Line Mapping

A `LineIndex` maps byte offsets to line/column:

```
struct LineIndex {
    line_starts: Vec<u32>,  // byte offset where each line begins
}
```

Built by scanning the source for `\n`, `\r\n`, `\r`. Given a byte offset,
binary search `line_starts` to find the line number, then subtract the line's
start offset to get the column.

Clang optimizes this with SIMD scanning of newlines (see "Optimizing the Clang
compiler's line-to-offset mapping" by Red Hat).

### Multi-File Spans

For systems with multiple source files, spans need a file identifier:

```
struct FullSpan {
    file_id: FileId,
    span: Span,
}
```

Rustc uses a global `SourceMap` where all files are concatenated into a single
virtual address space. Each file occupies a range; a global byte offset uniquely
identifies both file and position. This avoids carrying file IDs on every span.

### Macro Expansion Spans

When a span originates from macro expansion, the diagnostic system needs both:
- **Call site**: where the macro was invoked
- **Definition site**: where the macro body defined the code

Rustc stores this as `expansion` metadata on spans. Clang automatically traces
through the expansion chain to show both the invocation and the relevant
definition.

**Sources:**
- [SourceMap in rustc_span](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_span/source_map/struct.SourceMap.html)
- [Optimizing Clang's line-to-offset mapping -- Red Hat](https://developers.redhat.com/blog/2021/05/04/optimizing-the-clang-compilers-line-to-offset-mapping)

---

## 2. Snippet Rendering

### Basic Layout

The standard snippet format (pioneered by Clang, refined by Rust):

```
error[E0308]: mismatched types
  --> src/main.rs:4:18
   |
 4 |     let x: i32 = "hello";
   |            ---   ^^^^^^^ expected `i32`, found `&str`
   |            |
   |            expected due to this
```

Components:
1. **Header line**: severity, error code, message
2. **Location line**: `-->` with file:line:column
3. **Gutter**: line numbers right-aligned, separated by `|`
4. **Source line**: the actual code
5. **Annotation lines**: `^` for primary, `-` for secondary, with labels

### Multi-Line Spans

When a span crosses multiple lines, use a vertical bar to connect:

```
error: unterminated string
  --> src/main.rs:3:15
   |
 3 |     let s = "hello
   |  ___________^
 4 | |     world
 5 | |     ";
   | |_____^ unterminated string
```

### Label Placement Algorithm

When multiple labels exist on the same line, they must not overlap. The general
algorithm:

1. Sort labels by span start position
2. For each label, determine if it fits inline (on the same line as the `^`
   markers) or must go on a separate line below
3. If two labels would overlap horizontally, stack them vertically
4. Use connecting lines (`|`) to associate stacked labels with their spans

ariadne uses heuristics including:
- Priority-based ordering to prevent label crossover
- Automatic attachment point selection (start vs end of span)
- Compact mode that reduces vertical space

### Handling Edge Cases

- **Zero-width spans** (e.g., missing token): render as a single `^` at the
  insertion point
- **Very long lines**: truncate with `...` but preserve the annotated region
- **Tab characters**: expand to consistent width (typically 4 spaces) for
  alignment
- **Unicode characters**: use Unicode width (not byte count) for column alignment
- **Empty lines in spans**: show `...` to indicate elided lines

### Color Assignment

When multiple labels appear, each gets a distinct color. Ariadne's
`ColorGenerator` produces visually distinct colors automatically. Typical
palette:

| Element | Color |
|---------|-------|
| Error header | Red, bold |
| Warning header | Yellow, bold |
| Primary annotation | Red |
| Secondary annotation | Blue |
| Note/help text | Cyan or green |
| Line numbers | Blue |
| Source code | Default |

---

## 3. Structured Diagnostics

### Severity Levels

Every diagnostic system uses at least these levels:

| Level | Meaning | Compilation effect |
|-------|---------|-------------------|
| Error | Definite problem | Blocks compilation |
| Warning | Likely problem or style issue | Compilation continues |
| Note | Additional context | Always attached to an error/warning |
| Help | Suggestion for fixing | Often carries a code suggestion |

Rust also has `failure-note` (for ICEs) and `error: internal compiler error`.

TypeScript adds `Suggestion` (lighter than Warning, powers IDE refactoring).

Swift avoids "remark" (purely informational) -- if something is worth saying,
it should be a note attached to a real diagnostic.

### Primary vs Secondary Spans

A diagnostic typically has one primary span (the error location) and zero or
more secondary spans (the context). In rendered output:

- Primary: `^^^` in red, with the main error label
- Secondary: `---` in blue, explaining "why" or showing related code

Example showing both:
```
error[E0308]: mismatched types
  --> src/main.rs:8:5
   |
 7 |     fn add(x: i32, y: i32) -> i32 {
   |                                --- expected `i32` because of return type
 8 |         "hello"
   |         ^^^^^^^ expected `i32`, found `&str`
```

### Sub-Diagnostics (Children)

A diagnostic can have children -- notes and helps that provide additional
context:

```
error[E0382]: use of moved value: `x`
 --> src/main.rs:5:20
  |
3 |     let x = String::from("hello");
  |         - move occurs because `x` has type `String`
4 |     takes_ownership(x);
  |                     - value moved here
5 |     println!("{}", x);
  |                    ^ value used here after move
  |
help: consider cloning the value
  |
4 |     takes_ownership(x.clone());
  |                      ++++++++
```

This has:
- One primary span (line 5)
- Two secondary spans (lines 3, 4)
- One help sub-diagnostic with a machine-applicable suggestion

### Related Information

TypeScript's `DiagnosticRelatedInformation` and miette's `#[related]` link
diagnostics to each other. Useful for "this type was defined here" or "the
conflicting implementation is here" cross-references.

---

## 4. Error Recovery and Multiple Diagnostics

### Why Error Recovery Matters

A compiler that stops at the first error forces users into an edit-compile
cycle for every mistake. Good error recovery reports multiple independent errors
in a single pass.

### Recovery Strategies

**Panic mode**: On error, skip tokens until a synchronization point (`;`, `}`,
`)`). Simple but loses information.

**Phrase-level recovery**: Insert or delete a single token to continue parsing.
Example: insert a missing `;` and continue. More precise but harder to
implement correctly.

**Error productions**: Add grammar rules for common mistakes. Example: a rule
that matches `if condition { }` without parentheses in a C-like language, emits
a diagnostic, and continues as if the parentheses were present.

### Cascading Error Prevention

The critical problem: one root error causes a cascade of dozens of follow-on
errors. Prevention strategies:

1. **Error tokens/types**: Replace the erroneous AST node with a special "error"
   node. Later passes skip error nodes instead of generating more errors.
   Rust uses `ErrorGuaranteed` + `TyKind::Error` for this.

2. **Error limit**: Stop reporting after N errors (GCC defaults to 20, Clang
   to 20). Prevents overwhelming output.

3. **Relatedness detection**: If a subsequent error is in the same expression or
   statement as a prior error, suppress it or attach it as a note.

4. **Single-error mode**: Elm's approach -- stop after the first error in many
   cases. Prioritizes quality of the single error over quantity.

5. **Delayed bugs**: Rust's `delay_span_bug` defers ICE assertions. If a real
   error was already emitted, the delayed bug is silently discarded.

### The CPCT+ Algorithm

Research by Diekmann et al. ("Don't Panic! Better, Fewer, Syntax Errors for LR
Parsers") proposes using the complete set of minimum-cost repair sequences to
reduce cascading. CPCT+ reports substantially fewer spurious error locations
compared to panic mode recovery.

**Sources:**
- [Error Recovery Strategies in Compiler Design -- GeeksforGeeks](https://www.geeksforgeeks.org/compiler-design/error-recovery-strategies-in-compiler-design/)
- [Don't Panic! -- arxiv.org](https://arxiv.org/pdf/1804.07133)
- [ErrorGuaranteed -- rustc-dev-guide](https://rustc-dev-guide.rust-lang.org/diagnostics/error-guaranteed.html)

---

## 5. Machine-Readable Output

### Rustc JSON Format

`rustc --error-format=json` emits one JSON object per line to stderr:

```json
{
  "$message_type": "diagnostic",
  "message": "unused variable: `x`",
  "code": { "code": "unused_variables", "explanation": null },
  "level": "warning",
  "spans": [{
    "file_name": "lib.rs",
    "byte_start": 21, "byte_end": 22,
    "line_start": 2, "line_end": 2,
    "column_start": 9, "column_end": 10,
    "is_primary": true,
    "text": [{ "text": "    let x = 123;", "highlight_start": 9, "highlight_end": 10 }],
    "label": null,
    "suggested_replacement": "_x",
    "suggestion_applicability": "MachineApplicable",
    "expansion": null
  }],
  "children": [...],
  "rendered": "warning: unused variable: `x`\n --> lib.rs:2:9\n..."
}
```

Key design decisions:
- `rendered` field contains the human-readable version, so tools can display
  it without re-implementing the renderer
- `children` are flat (never nested) -- simpler to parse
- `suggestion_applicability` tells tools whether to auto-apply fixes
- Character offsets are Unicode Scalar Values, not bytes
- Forward-compatible: new fields may appear; optional fields may be null

### LSP Diagnostics

The Language Server Protocol `Diagnostic` object:

```typescript
interface Diagnostic {
  range: Range;                    // start/end line/character
  severity?: DiagnosticSeverity;   // Error=1, Warning=2, Information=3, Hint=4
  code?: number | string;
  codeDescription?: { href: URI };
  source?: string;                 // e.g., "lx"
  message: string;
  tags?: DiagnosticTag[];          // Unnecessary=1, Deprecated=2
  relatedInformation?: DiagnosticRelatedInformation[];
  data?: unknown;                  // preserved for CodeAction requests
}
```

Published via `textDocument/publishDiagnostics` notification. Code actions
(quick fixes) are requested separately via `textDocument/codeAction` and linked
back to diagnostics by the `data` field.

### SARIF Format

SARIF (Static Analysis Results Interchange Format) is an OASIS standard JSON
format for static analysis tool output. Supported by GCC 12+, Clang (in
progress), and MSVC. More heavyweight than rustc's JSON but standardized across
tools.

**Sources:**
- [JSON Output -- The rustc book](https://doc.rust-lang.org/rustc/json.html)
- [LSP Specification 3.17](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/)
- [SARIF Home](https://sarifweb.azurewebsites.net/)

---

## 6. Error Message Writing Guidelines

### The Two Questions

Every error message must answer (from Google's technical writing guidelines):

1. **What went wrong?**
2. **How does the user fix it?**

### Rules

**Be specific, not vague.**

```
BAD:   invalid expression
GOOD:  expected `)` to close function call started at line 3
```

**Show the relevant code.**

Don't describe code abstractly when you can point to it. The snippet with
annotations is always clearer than a prose description.

**Suggest fixes when confident.**

```
error: unknown variable `naem`
  --> src/main.rs:4:5
   |
 4 |     naem
   |     ^^^^ help: did you mean `name`?
```

Only suggest when the confidence is high. A wrong suggestion is worse than no
suggestion.

**Don't blame the user.**

```
BAD:   you provided an invalid argument
GOOD:  expected `i32`, found `String`
```

Avoid "you", "your", "illegal", "invalid input". Describe the *state* of the
code, not the user's mistake.

**Explain the "why", not just the "what".**

```
BAD:   cannot borrow `x` as mutable
GOOD:  cannot borrow `x` as mutable because it is also borrowed
       as immutable
  --> src/main.rs:5:5
   |
 3 |     let r = &x;
   |             -- immutable borrow occurs here
 5 |     x.push(1);
   |     ^ mutable borrow occurs here
 6 |     println!("{}", r);
   |                    - immutable borrow later used here
```

**Use consistent terminology.**

Pick one term for each concept and use it everywhere. Don't alternate between
"function", "method", "procedure", "routine" for the same thing.

**Keep it short for the 80% case.**

Most of the time, developers immediately know the fix. A long message wastes
their time. Put the essential information first; use sub-diagnostics (notes,
helps) for elaboration.

**Use present tense.**

The diagnostic describes the current state of the code:

```
BAD:   found a type mismatch
GOOD:  expected `i32`, found `String`
```

**Format code with backticks.**

Distinguish code tokens from prose: `` expected `i32`, found `String` `` not
"expected i32, found String".

### Swift-Specific Guidelines Worth Adopting

- Write in "newspaper headline" style: omit articles when possible
- Use lowercase opening, no terminal period
- Phrase as rules ("cannot call X outside Y") not limitations ("unable to
  process X")
- Include information showing the compiler *understood* the code -- this builds
  trust

### Elm-Specific Techniques Worth Adopting

- Show type diffs: when types mismatch, show both types with differences
  highlighted
- Recognize common beginner mistakes and provide targeted hints
- Stop after the first error when cascading is likely
- Write hints in conversational tone

**Sources:**
- [Writing Helpful Error Messages -- Google Developers](https://developers.google.com/tech-writing/error-messages)
- [Swift Diagnostics.md](https://github.com/swiftlang/swift/blob/main/docs/Diagnostics.md)
- [Compiler Errors for Humans -- Elm](https://elm-lang.org/news/compiler-errors-for-humans)
- [Writing Good Compiler Error Messages -- Caleb Mer](https://calebmer.com/2019/07/01/writing-good-compiler-error-messages.html)

---

## 7. Color and Formatting

### ANSI Color Usage

Standard diagnostic color assignments:

| Element | ANSI code | Visual |
|---------|-----------|--------|
| `error:` | Bold red (1;31) | Immediate attention |
| `warning:` | Bold yellow (1;33) | Important but not blocking |
| `note:` | Bold (1) or cyan (1;36) | Context |
| `help:` | Bold green (1;32) or cyan | Actionable |
| Error code | Bold (1) | Clickable in some terminals |
| Line numbers | Bold blue (1;34) | Visual gutter |
| Primary span | Red (31) | The problem |
| Secondary span | Blue (34) | The context |

### When to Use Bold/Underline

- **Bold**: for labels, severity keywords, code tokens in messages
- **Underline**: sparingly; can conflict with `^` annotations. Some terminals
  use underline for hyperlinks.
- **Italic**: avoid in diagnostics; poor terminal support

### NO_COLOR Standard

The [NO_COLOR](https://no-color.org/) convention: if the `NO_COLOR` environment
variable is set and non-empty, suppress all ANSI color output. Detection order:

1. Check `NO_COLOR` -- if set, disable color
2. Check `CLICOLOR_FORCE` -- if set, force color even without TTY
3. Check if stdout/stderr is a TTY -- if not, disable color
4. Check `TERM` -- some terminals don't support color

### Accessibility

- Never use color as the *only* distinguishing feature. Always pair with text
  (`error:`, `warning:`) or symbols (`^`, `~`, `-`).
- Support high-contrast modes: the palette should work on both dark and light
  terminal backgrounds.
- Provide a screen-reader-friendly mode (miette's `NarratableReportHandler`)
  that outputs diagnostics as structured prose.

---

## 8. Suggestion Quality

### When to Suggest

Suggest a fix when:
- There is exactly one obvious correction (typo in a name, missing semicolon)
- The fix is local (same file, nearby code)
- The fix is almost certainly correct

Do NOT suggest when:
- Multiple equally valid fixes exist (suggest them as alternatives in notes)
- The fix involves a design decision the user must make
- The suggestion might mask a deeper problem

### Confidence Levels (Rust's Model)

| Level | Example | Tooling action |
|-------|---------|----------------|
| MachineApplicable | Add missing `;` | Auto-apply via `rustfix` |
| HasPlaceholders | `fn foo<#type#>(...)` | Show, require user editing |
| MaybeIncorrect | "did you mean `name`?" | Show with caveat |
| Unspecified | General restructuring | Show cautiously |

### Suggestion Rendering

Inline (for small changes):
```
help: add a semicolon
  |
4 |     let x = 5;
  |              +
```

Replacement (showing what to change):
```
help: consider using `to_string()`
  |
4 |     let x = 42.to_string();
  |                ++++++++++++
```

Multi-line (for larger suggestions):
```
help: consider restructuring
  |
4 ~ fn process(items: &[Item]) -> Result<(), Error> {
5 ~     for item in items {
6 ~         handle(item)?;
7 ~     }
  |
```

Use `+` for insertions, `~` for replacements, `-` for deletions.

---

## 9. Error Codes and Documentation

### The Rust Model

Every significant error has a stable code (e.g., `E0308`). Users can run
`rustc --explain E0308` to get a multi-paragraph explanation with examples.
The full index is at <https://doc.rust-lang.org/error-index.html>.

Benefits:
- Stable identifiers for searching Stack Overflow and docs
- Machine-parseable for tooling
- Forces the team to write thorough explanations
- Enables cross-referencing in documentation

### The Swift Model

Diagnostic *groups* (e.g., `[#InvalidSuperCall]`) link to short documentation
covering one language concept in 3-4 paragraphs. This is lighter-weight than
Rust's per-error approach but still provides a learning resource at the point
of use.

### Recommendation for lx

Use short alphanumeric codes (e.g., `LX001`) with a `--explain` command.
Keep explanations in a single indexed file or directory. Each explanation should
include:
- What the error means
- Why it occurs
- A minimal code example that triggers it
- How to fix it

---

## 10. Internationalization

### Rust's Fluent-Based Approach

Rust uses [Project Fluent](https://projectfluent.org/) rather than ICU
MessageFormat. Fluent's "asymmetric localization" lets each language use its own
grammar independently:

```fluent
# English (messages.ftl)
hir_analysis_field_not_found =
    no field `{$field_name}` on type `{$type}`

# Japanese (ja/messages.ftl)
hir_analysis_field_not_found =
    型 `{$type}` にフィールド `{$field_name}` がありません
```

Each crate has its own `messages.ftl`. The fallback (English) bundle loads
lazily. Translations are loaded as alternative bundles.

### Key Challenges

- **Code snippets in messages**: Backtick-wrapped identifiers should not be
  translated
- **Pluralization**: "1 error" vs "2 errors" -- Fluent handles this natively
  with `{ $count ->` selectors
- **Right-to-left languages**: diagnostic rendering (arrows, gutters) is
  inherently LTR; full RTL support is an unsolved problem
- **Cultural conventions**: number formatting, date formatting in messages

### Practical Advice for lx

Start with English-only but design the infrastructure to support translation
later:
- Store all diagnostic messages in a central location (not inline strings)
- Use named parameters (`{field_name}`) not positional (`{0}`)
- Keep message IDs stable across versions

---

## 11. Progressive Disclosure

### The Principle

Show the most important information first. Let users drill down for details.

### Levels of Detail

1. **One-liner**: `error[E0308]: mismatched types` -- enough to remind
   experienced users of the problem
2. **Snippet**: the source code with annotations -- enough to locate the problem
3. **Sub-diagnostics**: notes and helps explaining why and how to fix
4. **Extended explanation**: `--explain E0308` -- full documentation with examples
5. **External resources**: links to documentation, tutorials, Stack Overflow

Each level is progressively more verbose. Most users stop at level 2 or 3.

### Implementation Techniques

- Show error + snippet + primary label by default
- Add notes/helps only when they provide non-obvious information
- Support `--explain` for deep dives
- In IDEs, use hover/click to reveal additional detail
- Error codes link to searchable documentation

### The 80/20 Rule (Caleb Mer)

80% of the time, developers immediately know the fix from the one-liner + code
location. Design for this case first. The remaining 20% need extended
explanations -- serve them via `--explain`, links, or sub-diagnostics, not by
making every error verbose.

---

## 12. Performance Considerations

### Avoid Allocating on the Happy Path

Diagnostic infrastructure must not slow down successful compilations. Key
techniques:

- **Lazy span resolution**: Store byte offsets everywhere. Only compute
  line/column when rendering a diagnostic.
- **Lazy message formatting**: Use format strings or message IDs, not
  pre-formatted strings. Only format when a diagnostic is actually emitted.
  TypeScript's proposal for lazy diagnostic formatting estimates 5-15% memory
  reduction in error-heavy projects.
- **Deferred construction**: Don't build `Diagnostic` objects until an error is
  detected. Use `Result<T, ErrorGuaranteed>` to propagate the *fact* of an
  error without carrying the diagnostic data.

### Rendering Performance

- **Line index**: Build once per file, cache. Binary search for line lookup
  is O(log n).
- **Source snippets**: Read from the original source buffer (memory-mapped if
  possible). Don't copy source text into diagnostic objects.
- **Color detection**: Check TTY/NO_COLOR once at startup, not per diagnostic.

### Memory

- Spans should be small (8 bytes for two u32s).
- Don't store the rendered string in the diagnostic -- render on output.
- For JSON mode, stream diagnostics one at a time; don't accumulate them.

---

## 13. Summary: Design Checklist for lx

### Infrastructure
- [ ] `Span` type: `(u32, u32)` byte offsets
- [ ] `LineIndex` per file with cached line starts
- [ ] `SourceMap` or equivalent for multi-file span resolution
- [ ] Lazy line/column resolution (only on render)

### Diagnostic Data Model
- [ ] Severity levels: Error, Warning, Note, Help
- [ ] Primary + secondary labeled spans
- [ ] Sub-diagnostics (children)
- [ ] Error codes with `--explain` support
- [ ] Suggestion applicability levels

### Rendering
- [ ] Snippet display with annotations (`^`, `-`, `|`)
- [ ] Multi-line span support
- [ ] Label placement with overlap avoidance
- [ ] Color with NO_COLOR/CLICOLOR support
- [ ] Compact mode for high-density output

### Machine-Readable Output
- [ ] JSON diagnostic format (one object per line)
- [ ] LSP-compatible diagnostics for editor integration
- [ ] `rendered` field in JSON for tools that don't want to re-render

### Message Quality
- [ ] Central message registry (not inline strings)
- [ ] Consistent terminology guide
- [ ] Every error answers "what" and "how to fix"
- [ ] Suggestions only when confidence is high
- [ ] No jargon; explain concepts in plain English
- [ ] Backtick-wrapped code tokens in messages

### Error Recovery
- [ ] Error nodes in AST for poisoned expressions
- [ ] Error limit (stop after N errors)
- [ ] Cascading prevention (skip analysis on error-tainted nodes)

**Sources:**
- [Errors and Lints -- rustc-dev-guide](https://rustc-dev-guide.rust-lang.org/diagnostics.html)
- [Compiler Errors for Humans -- Elm](https://elm-lang.org/news/compiler-errors-for-humans)
- [Swift Diagnostics.md](https://github.com/swiftlang/swift/blob/main/docs/Diagnostics.md)
- [Writing Helpful Error Messages -- Google](https://developers.google.com/tech-writing/error-messages)
- [NO_COLOR](https://no-color.org/)
- [Error Message Guidelines -- NN/g](https://www.nngroup.com/articles/error-message-guidelines/)
