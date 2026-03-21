# Error Message Design Across Programming Languages

A survey of how major languages design their diagnostic systems, what makes
their error messages effective (or not), and the libraries available for
building diagnostic renderers.

---

## 1. Rust -- The Gold Standard

Rust's diagnostic system is widely considered the best in the industry. The
investment is both cultural (a dedicated Diagnostics working group, RFC
processes, `--explain` documentation for every error code) and technical (the
`rustc_errors` crate, structured diagnostics, machine-applicable suggestions).

### Architecture

The core lives in `rustc_errors`. Key types:

| Type | Role |
|------|------|
| `Span` | Byte-offset pair `(lo, hi)` into the `SourceMap`; attached to every HIR/MIR node |
| `MultiSpan` | Collection of spans with optional labels |
| `Diag` | Builder for a single diagnostic; accumulates spans, children, suggestions |
| `DiagCtxt` | Central context that owns the error count and emitters |
| `ErrorGuaranteed` | Zero-size proof token that an error *has* been emitted; prevents cascading by letting later passes skip work on tainted items |

Diagnostics are emitted through two patterns:

1. **Immediate**: `dcx.span_err(span, "message")` -- fire and forget.
2. **Builder**: `dcx.struct_span_err(span, "message").span_label(...).emit()` -- accumulate labels, suggestions, notes, then emit.

The newer pattern uses **diagnostic structs** -- types that `#[derive(Diagnostic)]` and declare their spans, messages, and suggestions as struct fields. This separates diagnostic *definition* from the code path that detects the error.

### Suggestion Applicability

Every suggestion carries an `Applicability` level:

| Level | Meaning | Tooling behavior |
|-------|---------|------------------|
| `MachineApplicable` | Definitely correct | `rustfix` auto-applies |
| `HasPlaceholders` | Contains `(...)` user must fill | Show but don't auto-apply |
| `MaybeIncorrect` | Might be wrong | Show with caveat |
| `Unspecified` | Unknown confidence | Show cautiously |

### Error Codes and `--explain`

Every major error has a code like `E0308` (mismatched types). Running
`rustc --explain E0308` prints a multi-paragraph explanation with code examples.
The full index is at <https://doc.rust-lang.org/error-index.html>.

### Cascading Prevention via ErrorGuaranteed

`ErrorGuaranteed` is unconstructable outside `rustc_errors`. When a pass emits
an error, it receives an `ErrorGuaranteed` token. Downstream passes that receive
a type containing `ErrorGuaranteed` can skip further analysis, preventing
cascading errors from a single root cause. The complementary mechanism
`delay_span_bug` defers ICE-level assertions until compilation ends, so that if
an earlier error was already emitted, the delayed bug is silently discarded.

### Example of a Good Rust Error

```
error[E0308]: mismatched types
 --> src/main.rs:4:18
  |
4 |     let x: i32 = "hello";
  |            ---   ^^^^^^^ expected `i32`, found `&str`
  |            |
  |            expected due to this
  |
help: if you want to convert a string to an integer, use `parse`
  |
4 |     let x: i32 = "hello".parse().unwrap();
  |                          +++++++++++++++
```

Key qualities: the primary span points to the value, a secondary span explains
*why* `i32` was expected, and a `help` sub-diagnostic offers a concrete fix.

### Visual Format Redesign (2016)

Sophia June Turner led the 2016 redesign documented in "Shape of Errors to
Come." Core principle: **errors should focus on the code you wrote**. The
redesign introduced:

- **Primary labels** (red `^^^`) answering "what went wrong"
- **Secondary labels** (blue `---`) answering "why"
- Errors organized around source snippets, not around compiler internals

The redesign explicitly drew inspiration from Elm's approach.

### Translation Infrastructure

Rust uses [Project Fluent](https://projectfluent.org/) for diagnostic
translation. Each crate has a `messages.ftl` file with translatable strings.
Fluent's "asymmetric localization" lets each target language use its own grammar
without being constrained by English sentence structure. The fallback bundle
(English) loads lazily -- only parsed when the first diagnostic is emitted.

**Sources:**
- [Errors and Lints -- Rust Compiler Development Guide](https://rustc-dev-guide.rust-lang.org/diagnostics.html)
- [ErrorGuaranteed -- rustc-dev-guide](https://rustc-dev-guide.rust-lang.org/diagnostics/error-guaranteed.html)
- [Error Codes -- rustc-dev-guide](https://rustc-dev-guide.rust-lang.org/diagnostics/error-codes.html)
- [Translation -- rustc-dev-guide](https://rustc-dev-guide.rust-lang.org/diagnostics/translation.html)
- [Shape of Errors to Come -- Rust Blog](https://blog.rust-lang.org/2016/08/10/Shape-of-errors-to-come/)
- [JSON Output -- The rustc book](https://doc.rust-lang.org/rustc/json.html)
- [Helping with the Rust Errors -- Sophia Turner](https://www.sophiajt.com/helping-out-with-rust-errors/)
- [Diagnostic Translation Effort -- Inside Rust Blog](https://blog.rust-lang.org/inside-rust/2022/08/16/diagnostic-effort/)

---

## 2. Elm -- Famously Friendly

Elm's error messages are the canonical example of "compiler as assistant."
Evan Czaplicki's 2015 blog post "Compiler Errors for Humans" and 2016 follow-up
"Compilers as Assistants" laid out a philosophy that influenced Rust, Swift,
and others.

### Philosophy

> "Compilers should be assistants, not adversaries."

The compiler is reimagined as a pair programmer that catches mistakes and
explains *why* something is wrong in plain English, with concrete suggestions.

### Structure of an Elm Error

```
-- TYPE MISMATCH ------------------------------------ src/Main.elm

The 2nd argument to `update` is not what I expect:

8|   update msg model
                ^^^^^
This `model` value is a:

    { count : Int }

But `update` needs the 2nd argument to be:

    { count : String }

Hint: I can only compare ints, floats, chars, strings, lists, and
tuples. Maybe you need to use a custom comparison function?
```

Key structural elements:

1. **Header**: error category + file path (colored blue as separator)
2. **Narrative**: plain English description of what went wrong
3. **Code snippet**: exact user code with caret pointing to the problem
4. **Type diff**: shows expected vs actual, highlighting only differences
5. **Hint**: context-sensitive suggestion for fixing the problem

### Design Techniques

- **Color**: Red for the problem, blue for section separators. Minimal palette
  to avoid overwhelming.
- **Typo detection**: When a field name is close to an existing field, suggests
  the correct spelling.
- **Cascading prevention**: Stops after the first error in many cases, preventing
  the "wall of errors" problem.
- **Beginner-friendly hints**: Recognizes common beginner mistakes (wrong string
  concatenation operator, "truthy" patterns that don't exist in Elm) and
  provides targeted guidance.
- **No jargon**: Avoids terms like "unification failure" in favor of plain
  descriptions.
- **Machine-readable output**: `--report=json` flag for editor integration.

### Impact

Elm proved that dramatically better error messages require no significant
changes to type inference algorithms and impose no noticeable performance cost.
The investment is in *presentation*, not in fundamentally different analysis.

**Sources:**
- [Compiler Errors for Humans -- elm-lang.org](https://elm-lang.org/news/compiler-errors-for-humans)
- [Compilers as Assistants -- elm-lang.org](https://elm-lang.org/news/compilers-as-assistants)
- [Writing Good Compiler Error Messages -- Caleb Mer](https://calebmer.com/2019/07/01/writing-good-compiler-error-messages.html)

---

## 3. Python -- Traceback Evolution

Python's error reporting has undergone dramatic improvement from 3.10 through
3.14, transforming from bare tracebacks to precise, caret-annotated diagnostics.

### PEP 657: Fine-Grained Error Locations (Python 3.11)

Each bytecode instruction now stores four values: start line, end line, start
column offset, end column offset. The default exception hook uses these to
render caret indicators:

```python
# Before 3.11
TypeError: 'NoneType' object is not subscriptable

# After 3.11
  File "scientists.py", line 13, in dict_to_person
    life_span=(info["birth"]["year"], info["death"]["year"]),
               ~~~~~~~~~~~~~^^^^^^^^
TypeError: 'NoneType' object is not subscriptable
```

The `~` characters show the receiver expression and `^` characters point to
the failing subscript operation. This disambiguates which part of a complex
expression failed.

Trade-off: ~25-30% increase in `.pyc` file size. Opt out via
`-X no_debug_ranges` or `PYTHONNODEBUGRANGES` env var.

### PEP 617: PEG Parser and Better SyntaxErrors (Python 3.10)

Switching from LL(1) to PEG parsing enabled context-sensitive error messages:

- **Unclosed brackets**: Points to the opening bracket instead of "unexpected
  EOF while parsing"
- **Missing commas**: "Perhaps you forgot a comma?" with highlights between the
  two items
- **SyntaxError attributes**: Added `end_lineno` and `end_offset` to
  `SyntaxError` exceptions

### Python 3.14 Enhancements

Ten new error message patterns:

- **Keyword typo suggestions**: Close misspellings of keywords get "Did you
  mean...?" suggestions
- **Incompatible string prefixes**: Explains which prefixes conflict
- **elif-after-else**: Specific message instead of generic syntax error
- **Improved `as` target errors**: Better messages for import/except/match
  misuse

Each follows a consistent pattern: identify the mistake, explain in plain
English, suggest a fix when possible.

**Sources:**
- [PEP 657 -- peps.python.org](https://peps.python.org/pep-0657/)
- [Python 3.11 Preview: Even Better Error Messages -- Real Python](https://realpython.com/python311-error-messages/)
- [What's New In Python 3.10 -- docs.python.org](https://docs.python.org/3/whatsnew/3.10.html)
- [Python 3.14: Better Syntax Error Messages -- Real Python](https://realpython.com/python314-error-messages/)

---

## 4. TypeScript -- Template Diagnostics

TypeScript's diagnostic system is optimized for IDE integration (VS Code) rather
than CLI output.

### Diagnostic Structure

```typescript
interface Diagnostic {
  category: DiagnosticCategory;  // Error=1, Warning=0, Suggestion=2, Message=3
  code: number;                  // e.g., 2322 for type mismatch
  file: SourceFile | undefined;
  start: number | undefined;     // byte offset
  length: number | undefined;
  messageText: string | DiagnosticMessageChain;
}
```

`DiagnosticMessageChain` is a linked list of messages built bottom-up, where the
head is the "main" diagnostic and children provide elaboration. This enables
TypeScript's characteristic nested error messages:

```
Type '{ name: string; age: string; }' is not assignable to type 'Person'.
  Types of property 'age' are incompatible.
    Type 'string' is not assignable to type 'number'.
```

### Related Information

`addRelatedInfo()` attaches secondary diagnostic spans, enabling "see also"
links to related code locations -- the type definition, the conflicting
assignment, etc.

### Suggestion Diagnostics

Category `Suggestion` (2) powers IDE light-bulb actions. These are
lower-severity than warnings and represent refactoring opportunities rather
than problems.

### Notable Design Choices

- Error messages are stored as templates in `diagnosticMessages.json` with
  numbered placeholders, enabling potential localization
- The `--pretty` flag enables caret-annotated output for CLI use
- No error code documentation system comparable to Rust's `--explain`

**Sources:**
- [TypeScript TSConfig diagnostics](https://www.typescriptlang.org/tsconfig/diagnostics.html)
- [ts-morph Diagnostics](https://ts-morph.com/setup/diagnostics)

---

## 5. Clang/GCC -- C/C++ Diagnostics

Clang's diagnostics historically set the bar that Rust later raised. GCC has
been catching up since version 5.0.

### Clang's Key Features

**Caret diagnostics with ranges**: Clang pins the error to an exact column and
highlights the full expression range:

```
t.c:7:39: error: invalid operands to binary expression ('int' and 'struct A')
  return y + func(y ? ((SomeA.X + 40) + SomeA) / 42 + SomeA.X : SomeA.X);
                       ~~~~~~~~~~~~~~ ^ ~~~~~
```

**Fix-it hints**: Inline suggestions for correcting problems:

```
t.c:2:1: warning: 'extern' is not needed on a function declaration
extern int foo();
^~~~~~~
```

**Typedef preservation with `aka`**: Shows user-defined type names but reveals
underlying types when helpful:

```
error: invalid operands to binary expression ('my_type' (aka 'int *') and 'float')
```

**Template type diffing**: Compares template instantiations and highlights only
differences, with optional tree-mode display (`-fdiagnostics-show-template-tree`).

**Macro expansion tracking**: Automatically shows the macro definition site,
expansion chain, and ranges within expanded macros.

**Diagnostic groups**: Every warning belongs to named groups (`-Wunused`,
`-Wformat`, etc.) controllable via `-W`, `-Wno-`, `-Werror=` flags.

### GCC's Evolution

GCC lagged behind Clang on diagnostics until version 5.0:

| Feature | GCC version | Clang |
|---------|-------------|-------|
| Column numbers in errors | 5.0 | 1.0 |
| Source snippets | 5.0 | 1.0 |
| Colored output default | 8.0 | 1.0 |
| Fix-it hints | 6.0 | 1.0 |
| `-Wmisleading-indentation` | 6.0 | later |
| SARIF output | 12.0 | in progress |

GCC's `-Wmisleading-indentation` (detecting if-without-braces problems) was
one area where GCC led.

### SARIF Support

Both GCC (12+) and Clang support SARIF (Static Analysis Results Interchange
Format) output via `-fdiagnostics-format=sarif-stderr`. SARIF is a JSON-based
OASIS standard designed for machine-readable static analysis results.

**Sources:**
- [Clang Expressive Diagnostics](https://clang.llvm.org/diagnostics.html)
- [GCC vs Clang vs MSVC Diagnostics](https://easyaspi314.github.io/gcc-vs-clang.html)
- [GCC Diagnostic Message Formatting Options](https://gcc.gnu.org/onlinedocs/gcc/Diagnostic-Message-Formatting-Options.html)
- [Structured SARIF Diagnostics -- Microsoft Learn](https://learn.microsoft.com/en-us/cpp/build/reference/sarif-output)

---

## 6. Swift -- FixIts and Educational Notes

Swift's diagnostic system inherits from Clang but adds two distinctive features:
educational notes and diagnostic groups.

### FixIt Criteria

Swift FixIts must represent "the single, obvious, and very likely correct way to
fix the issue." Multiple alternatives are presented as separate notes, ordered by
safety. FixIts must stay within the same file and can use Xcode placeholders
(`<#placeholder#>`) when exact replacements are unknown.

### Educational Notes (Diagnostic Groups)

Diagnostic groups bundle related diagnostics under a named category with short
documentation:

```
error: cannot call 'super.init' outside of an initializer [#InvalidSuperCall]
```

The `[#GroupName]` links to 3-4 paragraph documentation explaining the language
concept. This serves as a "teachable moment" -- learning resources at the point
of use, following Swift's philosophy of progressive disclosure.

Documentation requirements:
- Unabbreviated English, accessible to beginners
- References to *The Swift Programming Language* book
- Covers one concept per group

### Writing Guidelines

- Single phrase/sentence, no terminal period
- Newspaper-headline style: omit articles ("the", "a")
- Phrase as rules, not compiler limitations: prefer "cannot call `super.init`
  outside initializers" over "unable to process `super.init` call"
- Use backticks for code tokens
- Include information that demonstrates the compiler *understood* the code

### Key Design Decision: No Warning Suppression

Unlike Clang/GCC, Swift intentionally prevents disabling compiler warnings to
prevent language fragmentation into dialects.

**Sources:**
- [Swift Diagnostics.md](https://github.com/swiftlang/swift/blob/main/docs/Diagnostics.md)
- [RFC: Educational Notes for Data-Race Safety Errors](https://forums.swift.org/t/rfc-educational-notes-for-data-race-safety-errors/78003)
- [SE-0196: Diagnostic Directives](https://github.com/swiftlang/swift-evolution/blob/main/proposals/0196-diagnostic-directives.md)

---

## 7. Go -- Minimal but Clear

Go takes a deliberately minimalist approach to compiler diagnostics, consistent
with the language's overall philosophy of simplicity.

### Error Message Style

Go compiler errors are terse, single-line messages:

```
./main.go:5:6: x declared and not used
./main.go:3:8: "fmt" imported and not used
```

No multi-line snippets, no color, no suggestions. The philosophy is that the
compiler enforces strict rules (unused imports/variables are *errors*, not
warnings) and the messages are clear enough that no further explanation is needed.

### go vet as Supplementary Diagnostics

`go vet` provides supplementary analysis beyond compilation errors. Key design
criteria:

- Checks must be about **correctness**, not style
- Must find real bugs often enough to justify the overhead
- Both false positive and false negative rates must be very low
- JSON output via `-json` flag for machine consumption

The checks are heuristic: `go vet` examines constructs like `Printf` calls
whose arguments don't align with the format string.

### Design Philosophy

> "The more magic we introduce at the language level, the higher the cost for
> users to debug, read, and track down problems."

Go's error messages reflect this by being predictable and uniform. There are no
"smart" suggestions that might mislead. The trade-off: newcomers find messages
like "multiple-value in single-value context" confusing because Go expects
developers to build compiler literacy over time.

**Sources:**
- [Learn to love your compiler -- YourBasic Go](https://yourbasic.org/golang/compiler-error-messages/)
- [go vet -- Go Packages](https://pkg.go.dev/cmd/vet)

---

## 8. Diagnostic Rendering Libraries (Rust Ecosystem)

Four crates dominate the Rust ecosystem for diagnostic rendering.

### ariadne

**Repository**: <https://github.com/zesterer/ariadne>

The most visually sophisticated option. Produces colorful, Unicode-decorated
output with multi-line label handling.

API:
```rust
Report::build(ReportKind::Error, ("file.lx", 12..12))
    .with_code(3)
    .with_message("Incompatible types")
    .with_label(Label::new(("file.lx", 32..33))
        .with_message(format!("This is of type {}", "Nat".fg(a)))
        .with_color(a))
    .with_label(Label::new(("file.lx", 42..45))
        .with_message(format!("This is of type {}", "Str".fg(b)))
        .with_color(b))
    .finish()
    .print(("file.lx", Source::from(src)))
    .unwrap();
```

Features:
- Multi-file error reporting
- 8-bit and 24-bit color
- Built-in overlap/crossover avoidance heuristics
- `ColorGenerator` for automatic distinct colors
- Compact mode
- Variable-width character handling (tabs, Unicode)
- Configurable character sets for terminal compatibility

Caveats:
- Layout heuristics can change between versions (visual output not guaranteed
  stable)
- Sister project of `chumsky` parser combinator

### miette

**Repository**: <https://github.com/zkat/miette>

The most feature-complete option, designed as a drop-in for applications that
want compiler-quality diagnostics in their error handling.

API (derive macro):
```rust
#[derive(Error, Debug, Diagnostic)]
#[error("expected {expected}, found {found}")]
#[diagnostic(code(lx::type_mismatch), help("try converting with `to_str()`"))]
struct TypeMismatch {
    expected: String,
    found: String,
    #[source_code]
    src: NamedSource<String>,
    #[label("this expression")]
    span: SourceSpan,
}
```

Unique features:
- `#[derive(Diagnostic)]` on error types -- diagnostics *are* errors
- Multiple report handlers: `GraphicalReportHandler` (fancy),
  `NarratableReportHandler` (screen-reader friendly),
  `JSONReportHandler` (machine-readable)
- Automatic `NO_COLOR` / `CLICOLOR` / CI detection
- Optional syntax highlighting via `syntect`
- URL support (`#[diagnostic(url(...))]`) for linking to docs
- `#[related]` for chaining multiple errors
- Primary vs collection labels

### codespan-reporting

**Repository**: <https://github.com/brendanzab/codespan>

The most mature and battle-tested option. Powers error reporting in Gleam,
Gluon, CXX, and many other production compilers.

API:
```rust
let diagnostic = Diagnostic::error()
    .with_message("`case` expression has incompatible arms")
    .with_code("E0308")
    .with_labels(vec![
        Label::primary(file_id, 328..331)
            .with_message("expected `String`, found `Nat`"),
        Label::secondary(file_id, 211..331)
            .with_message("`case` arms have incompatible types"),
    ])
    .with_notes(vec![
        "expected type `String`\n   found type `Nat`".to_string(),
    ]);

term::emit(&mut writer, &config, &files, &diagnostic)?;
```

Features:
- `SimpleFile` / `SimpleFiles` for source management
- Primary and secondary labels
- Configurable `Config` struct for rendering
- Clean separation between diagnostic data and rendering

### annotate-snippets

**Repository**: <https://github.com/rust-lang/annotate-snippets-rs>

The official Rust project library, used inside `rustc` itself. Designed to
produce output identical to rustc's native renderer.

API:
```rust
let report = Level::ERROR.title("mismatched types")
    .snippet(Snippet::source(source)
        .origin("src/main.rs")
        .line_start(1)
        .annotation(AnnotationKind::Primary.span(14..22).label("expected i32"))
        .annotation(AnnotationKind::Context.span(7..10).label("expected due to this")));

let renderer = Renderer::styled();
println!("{}", renderer.render(report));
```

Features:
- Matches rustc output exactly (regression-tested)
- Unicode decoration style
- Used by both `rustc` and `cargo`
- Minimal API surface

### Comparison Matrix

| Feature | ariadne | miette | codespan-reporting | annotate-snippets |
|---------|---------|--------|--------------------|-------------------|
| Multi-file | Yes | Yes | Yes | Yes |
| Color | 8/24-bit | 8/24-bit | Via termcolor | Styled |
| Derive macro | No | Yes | No | No |
| JSON output | No | Yes | No | No |
| Screen reader mode | No | Yes | No | No |
| Syntax highlighting | No | Optional | No | No |
| Used in production | Many | Many | Gleam, Gluon, CXX | rustc, cargo |
| Overlap avoidance | Yes | Yes | Basic | Yes |
| URL/error code links | No | Yes | No | No |

### Recommendation for lx

**ariadne** or **miette** are the strongest choices. ariadne produces the most
visually striking output. miette integrates most naturally with Rust's error
handling ecosystem and provides JSON output, screen reader support, and derive
macros that reduce boilerplate. codespan-reporting is the safe, proven choice.
annotate-snippets is best if exact rustc-style output is desired.

**Sources:**
- [ariadne -- GitHub](https://github.com/zesterer/ariadne)
- [miette -- docs.rs](https://docs.rs/miette/latest/miette/)
- [codespan -- GitHub](https://github.com/brendanzab/codespan)
- [annotate-snippets -- docs.rs](https://docs.rs/annotate-snippets/)
- [Use annotate-snippets for rustc -- Rust Project Goals](https://rust-lang.github.io/rust-project-goals/2024h2/annotate-snippets.html)

---

## 9. Academic Research

### "Do Developers Read Compiler Error Messages?" (Barik et al., ICSE 2017)

Eye-tracking study with 56 participants. Key findings:
- Developers *do* read error messages (13-25% of total task time)
- Difficulty reading error messages significantly predicts task performance
- Reading difficulty is comparable to reading source code

### "Compiler Error Messages Considered Unhelpful" (Becker et al., ITiCSE 2019)

Comprehensive landscape survey of 50+ years of research on diagnostic messages.
Found that messages present "substantial difficulty" and could be more effective,
particularly for novices.

### "On Compiler Error Messages: What They Say and What They Mean" (Traver, 2010)

HCI-focused study finding that most compilers have not paid attention to error
message usability despite it being a critical aspect of the developer experience.
Users prefer messages that are informative, human-centric, and provide accurate
information about error resolution.

### "On Designing Programming Error Messages for Novices" (CHI 2021)

Studied readability factors in error messages, finding that constituent factors
like vocabulary complexity, sentence structure, and information density all
affect how quickly developers can understand and act on diagnostics.

**Sources:**
- [Do Developers Read Compiler Error Messages? -- IEEE](https://ieeexplore.ieee.org/document/7985695/)
- [Compiler Error Messages Considered Unhelpful -- ACM](https://dl.acm.org/doi/10.1145/3344429.3372508)
- [On Compiler Error Messages -- Wiley](https://www.hindawi.com/journals/ahci/2010/602570/)
- [On Designing Error Messages for Novices -- ACM CHI](https://dl.acm.org/doi/10.1145/3411764.3445696)
