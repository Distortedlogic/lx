# Code Formatter Landscape Survey

A comprehensive survey of code formatting tools across programming languages,
covering algorithms, architectures, design philosophies, and implementation details.

---

## Table of Contents

1. [Python](#python)
2. [Rust](#rust)
3. [JavaScript / TypeScript](#javascript--typescript)
4. [Go](#go)
5. [C / C++](#c--c)
6. [Scala](#scala)
7. [Haskell](#haskell)
8. [Elm](#elm)
9. [OCaml](#ocaml)
10. [Dart](#dart)
11. [Cross-Cutting Themes](#cross-cutting-themes)

---

## Python

### Black

**Repository**: https://github.com/psf/black

**Philosophy**: "The uncompromising code formatter." Deliberately opinionated with
minimal configuration. The core thesis is that style debates waste engineering time,
and a single deterministic output eliminates them.

**Parsing**: Uses `blib2to3`, a forked subset of Python's `lib2to3` library (taken
from CPython 3.7.0b2). This produces a Concrete Syntax Tree (CST), not an AST. The
CST preserves all syntactic information including whitespace and comments. As a safety
check, Black also parses the output to an AST and verifies equivalence with the
original AST, ensuring formatting never changes semantics.

**Performance profile**: Initial parsing with blib2to3 consumes 30-50% of formatting
runtime. The actual formatting logic spends ~75% of its time in the CST visitor.
Black has been partially compiled with mypyc for speed improvements.

**Line-length algorithm**: Defaults to 88 characters (10% over 80, chosen because it
produces significantly shorter files). The algorithm is hierarchical:

1. Try fitting the entire expression on one line
2. If it fails, examine contents of the first outer matching brackets, put that on a
   separate indented line
3. If still too long, recursively decompose using the same rule, indenting at each
   bracket level

**Magic trailing comma**: Black's primary exception to ignoring existing formatting.
A trailing comma in a collection or function signature signals intent to keep elements
on separate lines. Without it, Black collapses collections that fit within the line
limit. Controllable via `--skip-magic-trailing-comma`.

**String handling**: Standardizes on double quotes (replacing single quotes when it
doesn't increase backslash escapes). Prefix characters become lowercase (except
capital `R`), unicode markers (`u`) are removed.

**Comment handling**: Preserves comment content, enforces two spaces between code and
inline comments. Comments may relocate due to formatting changes. In blib2to3,
comments are stored in the whitespace prefix of the next token
(`pgen2/driver.py:Driver.parse_tokens()`), avoiding grammar modifications.

**Configuration**: Minimal by design. Key options: `--line-length`, `--target-version`,
`--skip-string-normalization`, `--skip-magic-trailing-comma`. No style presets.

**Idempotency**: Guaranteed. Running Black twice produces identical output.

**Sources**:
- [Black documentation](https://black.readthedocs.io/)
- [Black code style](https://black.readthedocs.io/en/stable/the_black_code_style/current_style.html)
- [Black GitHub - blib2to3](https://github.com/psf/black/tree/main/src/blib2to3)

---

### YAPF (Yet Another Python Formatter)

**Repository**: https://github.com/google/yapf

**Philosophy**: Highly configurable. Based on clang-format's approach of searching for
the "best" layout under configured rules, rather than applying fixed heuristics.

**Algorithm**: Uses a clang-format-inspired reformatting algorithm. The algorithm takes
code, calculates the best formatting that conforms to the configured style, and
searches for the lowest-cost result across formatting decisions. This is fundamentally
different from Black's approach: YAPF explores a solution space, while Black applies
deterministic rules.

**Configuration**: Extremely configurable. Includes defaults for PEP 8, Google,
Facebook, and Chromium styles. Many "knobs" for tuning formatting behavior. Accepts
predefined styles, configuration file paths, or dictionaries of key/value pairs.

**Performance**: Significantly slower than Black and Ruff due to the search-based
approach. On the Zulip codebase (~250k lines): YAPF takes ~19.56 seconds vs Black's
~3.20 seconds.

**Sources**:
- [YAPF GitHub](https://github.com/google/yapf)
- [Python formatters comparison](https://blog.frank-mich.com/python-code-formatters-comparison-black-autopep8-and-yapf/)

---

### autopep8

**Philosophy**: A "loose" formatter. Its aim is fixing PEP 8 violations, not making
code uniform. It does the least formatting of the major Python formatters.

**Approach**: Focuses on fixing specific PEP 8 errors rather than reformatting entire
files. Configurable. Does not guarantee uniform output across different inputs that
are semantically equivalent.

**Performance**: Fast (does less work), but speed comparisons should account for the
reduced scope of formatting.

---

### Blue

A fork of Black with slightly different style decisions (e.g., single quotes as
default instead of double quotes). Shares Black's architecture and algorithm but
with different default configuration choices.

---

### Ruff Formatter

**Repository**: https://github.com/astral-sh/ruff

**Philosophy**: Black-compatible but written in Rust for extreme performance.

**Architecture**: Built on a fork of Rome's `rome_formatter`, drawing on API and
implementation details from Rome, Prettier, and Black. Uses Biome's printer adapted
for Python. Shares core infrastructure (lexer, parser) with the Ruff linter.

**Performance**: Over 30x faster than Black, 100x faster than YAPF. On Zulip (~250k
lines): Ruff takes 0.10 seconds vs Black's 3.20 seconds. Leverages Rust's memory
management and Rayon for parallelism across CPU cores (no GIL limitation).

**Black compatibility**: >99.9% compatibility as measured by changed lines. Intentional
deviations are limited to cases where Ruff's behavior was deemed more consistent.
Notable: excludes pragma comments when measuring line length, preventing `# noqa`
additions from triggering reflow.

**Sources**:
- [Ruff Formatter announcement](https://astral.sh/blog/the-ruff-formatter)
- [Ruff documentation](https://docs.astral.sh/ruff/formatter/)

---

## Rust

### rustfmt

**Repository**: https://github.com/rust-lang/rustfmt

**Philosophy**: Default style aims for community consensus. Configurable but with a
strong default. Stability guarantees protect CI pipelines.

**Parsing**: Uses `syntex_syntax`, a fork of the Rust compiler's own parser. This
ensures full Rust language support including nightly syntax. The parser handles Rust's
complexity by differentiating between multiple string formats and maintaining state
for macro invocations, treating macro-internal tokens as opaque during formatting.

**Formatting algorithm**: Recursive AST traversal with rule application based on a
priority system where explicit configuration overrides defaults:

- **Shape and Indent values**: Guide alignment strategies for composite constructs
- **FormatContext**: Delivers contextual decisions about parentheses necessity and
  single-line feasibility
- **Cost modeling**: When competing rules conflict, the formatter uses a cost model
  assigning weights to whitespace modifications
- **Fixed-point iteration**: Stabilizes whitespace, comments, and alignment until no
  further changes occur

**Configuration**: Via `rustfmt.toml` or `.rustfmt.toml`. Key options:
- `max_width` (default 100)
- `edition` (must be set explicitly)
- `use_small_heuristics` (controls bracket grouping)
- `unstable_features` (nightly-only plugin support)
- Many options classified as stable or unstable

**Stability guarantees** (RFC 2437): A newer version of rustfmt cannot modify the
successfully formatted output produced by a previous version, under these conditions:
- Using default configuration options
- Formatting code that compiles with stable Rust
- Formatting produces error-free output

Major version increments for API-breaking changes. Minor versions cover formatting
changes, but major formatting changes require opt-in via `required_version`.

**Style editions** (RFC 3338): Formatting style evolves across Rust editions. Code in
Rust 2015/2018/2021 uses the existing default style. Code in Rust 2024+ may use a new
style edition. A separate `style_edition` configuration option allows decoupling
language edition from formatting style. New style editions are initially nightly-only,
stabilizing with their corresponding Rust edition.

**Macro handling challenges**: A fundamental limitation. rustfmt sees macro calls in
raw form with tokens as written in source (potentially non-valid Rust syntax). Since
rustfmt only formats valid Rust syntax, it cannot process arbitrary macro content.
Additional issues:
- Things not semantically important in normal Rust (e.g., trailing commas) can be
  semantically important in macros
- Procedural macros using Spans can generate different code based on token locations,
  meaning rustfmt can change program behavior
- Non-idempotency: indentation can increment by 8 spaces per invocation in some macro
  contexts
- Workaround: `#[rustfmt::skip::macros(target_macro_name)]`

**Comment handling**: Comments require special attention during tokenization. The
system maintains running state for macro invocations, passing comments unaltered to
later formatting routines.

**Performance**: Parsing accounts for up to 60% of execution time in single-file runs.
A million-line codebase processes within 20 seconds using 8 concurrent threads (via
Rayon). Incremental formatting provides 2-3x reduction for files with minor changes.

**Emission safety**: Output uses a buffer management system for atomic writes,
guaranteeing file modifications either complete fully or abort on error, preventing
half-formatted files.

**Sources**:
- [rustfmt GitHub](https://github.com/rust-lang/rustfmt)
- [RFC 2437: Rustfmt stability](https://rust-lang.github.io/rfcs/2437-rustfmt-stability.html)
- [RFC 3338: Style evolution](https://rust-lang.github.io/rfcs/3338-style-evolution.html)
- [Rustfmt architecture deep dive](https://moldstud.com/articles/p-a-deep-dive-into-rustfmt-architecture-understanding-how-it-works)

---

## JavaScript / TypeScript

### Prettier

**Repository**: https://github.com/prettier/prettier

**Philosophy**: Opinionated. Few configuration options. The rationale: formatters that
respect existing formatting create inconsistency; Prettier discards existing
formatting and reprints from scratch.

**Algorithm**: Based on Philip Wadler's "A prettier printer" paper. The original
codebase forked recast's printer but replaced its algorithm with Wadler's. The
algorithm explores layout possibilities by greedily resolving alternatives one line
at a time.

**Two-phase architecture**:
1. **Parse** source code to AST (supports many parsers: babel, typescript, postcss,
   markdown, etc.)
2. **Print** AST to an intermediate representation (Doc IR)
3. **Render** Doc IR to final string, making line-breaking decisions

**Doc IR commands** (the intermediate representation):

| Command | Behavior |
|---------|----------|
| `group(doc)` | Try to fit content on one line; break outermost first if it doesn't fit |
| `conditionalGroup(alternatives)` | Try alternatives from least to most expanded (exponential if nested) |
| `indent(doc)` | Increase indentation one level |
| `dedent(doc)` | Decrease indentation one level |
| `align(width, doc)` | Fixed-width indentation alignment |
| `line` | Space if flat, break+indent if broken |
| `softline` | Nothing if flat, break+indent if broken |
| `hardline` | Always break and indent |
| `literalline` | Always break, no indent (for template literals) |
| `fill(docs)` | Text-layout mode: break only when next element doesn't fit |
| `ifBreak(broken, flat)` | Conditional content based on break state |
| `breakParent` | Force all parent groups to break |
| `lineSuffix(doc)` | Buffer content to flush before next line (for trailing comments) |
| `lineSuffixBoundary` | Explicit lineSuffix flush point |
| `indentIfBreak(doc, {groupId})` | Conditional indentation tied to group break state |
| `label(label, doc)` | Metadata attachment without affecting output |
| `trim` | Remove all indentation on current line |
| `cursor` | Placeholder for cursor position tracking |

**Group mechanics**: Groups are usually nested. The printer tries to fit everything on
one line. If it doesn't fit, it breaks the outermost group first and tries again.
Breaks propagate to all parent groups. `breakParent` forces this propagation.

**Fill command**: An alternative group type that behaves like text layout: adds breaks
only when the next element doesn't fit, rather than breaking all separators. Expects
alternating content and line breaks.

**Line width handling**: Controlled by `printWidth` (default 80). The `fits()` function
measures grouped content length. If it exceeds print width, broken layout applies;
otherwise flat layout renders. This measurement drives layout selection automatically.

**Comment handling**: Parsers typically omit comments from ASTs. Prettier must
reattach them, which is nontrivial. The `lineSuffix` command handles trailing comments
by buffering content to flush before the next line break.

**Idempotency**: Mostly guaranteed but with known edge cases. Object literals that
become multiline won't collapse back. Adding then removing a property can leave
formatting different from the initial state. The team avoids non-reversible formatting
but hasn't fully solved this for object literals.

**String interpolation**: Uses a heuristic where interpolation expressions only split
across multiple lines if there was already a linebreak within the interpolation in the
original source.

**Performance**: JavaScript-based, single-threaded. Slower than native implementations
(Biome, Ruff) but fast enough for most workflows.

**Sources**:
- [Prettier technical details](https://prettier.io/docs/technical-details)
- [Prettier rationale](https://prettier.io/docs/rationale)
- [Prettier commands.md](https://github.com/prettier/prettier/blob/main/commands.md)
- [James Long - A Prettier Formatter](https://archive.jlongster.com/A-Prettier-Formatter)

---

### Biome (formerly Rome)

**Repository**: https://github.com/biomejs/biome

**Philosophy**: Prettier-compatible but implemented in Rust for performance. Part of a
unified toolchain (linter + formatter).

**Two-phase architecture**:
1. **CST to IR**: Each Concrete Syntax Tree node generates FormatElements describing
   intended formatting without committing to line breaks
2. **IR to Text**: The pretty printer consumes FormatElements and makes line-breaking
   decisions based on line width constraints

**FormatElement IR**:

| Element | Purpose |
|---------|---------|
| `Text` | Literal text content |
| `Space` | Single space character |
| `Line` | Potential break point (soft/hard/hard-or-space) |
| `Group` | Content formatted as unified block |
| `Indent` | Increase indentation level |
| `Dedent` | Decrease indentation level |
| `Fill` | Compact formatting mode |

**Line break types**:
- **Soft Line**: Breaks only if the group doesn't fit on a single line
- **Hard Line**: Always creates a new line
- **Hard-or-Space**: Conditional breaks based on context

**Pretty printing algorithm**: Implements Wadler's algorithm. Processes groups to
determine fit, checks content length against `lineWidth`, converts soft lines to hard
lines when groups overflow, tracks and applies indentation levels.

**Prettier compatibility**: 97% compatible. Extensively tested against Prettier's test
suite. Supports line width enforcement, group-based formatting, soft/hard line breaks,
comment preservation, trailing comma styles, quote styles, semicolon insertion.

**Performance advantages over Prettier**:
- Native Rust implementation (no JavaScript runtime overhead)
- Multi-threaded parallel processing
- Incremental parsing (only re-parses changed portions)
- Efficient caching of previous computation results

**Language-specific architecture**: Each language formatter follows a consistent
pattern: `biome_[lang]_formatter` depends on `biome_formatter` (core IR),
`biome_[lang]_syntax` (typed AST nodes), `biome_rowan` (syntax tree), and
`biome_suppression` (comment-based formatting suppression).

**Integration**: Supports `format_file()`, `format_range()`, and `format_on_type()`
(keystroke-triggered after `;` or `}`).

**Sources**:
- [Biome formatter implementation](https://deepwiki.com/biomejs/biome/6.2-formatter-implementation)
- [Biome documentation](https://biomejs.dev/)
- [biome_formatter crate docs](https://docs.rs/biome_formatter/latest/biome_formatter/)

---

## Go

### gofmt

**Repository**: Part of the Go standard library (`cmd/gofmt`)

**Philosophy**: "One true format." No options, no knobs. The value lies in consistency,
not aesthetic ideality. gofmt defines the de facto Go formatting standard. All
submitted Go code in `golang.org` repos must be formatted with gofmt.

**Design principles**:
- Avoid formatting options entirely
- Keep implementation simple
- Make parser/printer the foundation
- Enable source code transformation at AST level
- "Good enough" uniformity is more valuable than perfection
- Respect user intent regarding line breaks

**Processing pipeline**:

```
Source Code
  -> [Parser: go/scanner, go/parser]
  -> Abstract Syntax Tree (go/ast)
  -> [Printer: traverse AST recursively, emit tokens + tabs]
  -> Token/Position/Whitespace Stream
  -> [Merge with Comment Stream]
  -> Combined Token Stream
  -> [Text expansion through tabwriter]
  -> Formatted Source Code
```

**AST-based approach**: Uses Go's standard library parsing tools: `go/scanner`
(tokenization), `go/parser` (parsing), `go/ast` (AST generation). Each syntactic
construct has a corresponding AST node with position information.

**Printer**: Traverses the AST and prints nodes using `p.print()` which accepts
sequences of tokens with position and whitespace information. Fine-tuning heuristics
include:
- Precedence-dependent spacing between operands for expression readability
- Position information guides line break decisions
- Various context-specific heuristics

**Comment handling**: The biggest design mistake acknowledged by the Go team: comments
were not initially attached to AST nodes, making it extremely difficult to manipulate
the AST while maintaining comments in correct positions. The workaround: `ast.CommentMap`
(described as "cludgy"). Comments are grouped into `CommentGroup` structures
(consecutive comments with no intervening tokens or empty lines). The formatting
algorithm merges "token stream" with "comment stream" based on position information.

**Elastic tabstops**: Uses Go's `text/tabwriter` package (based on Nick Gravgaard's
2006 elastic tabstops proposal). A tab indicates the end of a text cell. A column
block is a run of vertically adjacent cells. Column block width equals the widest
text in cells. This enables automatic alignment without manual spacing.

**Determinism**: gofmt can pick up every source file in the Go tree, parse it into an
internal representation, and put the exact same bytes back down. Machine-formatted
code can be transformed mechanically without generating unrelated formatting noise
in diffs.

**Downstream tools enabled by gofmt's architecture**:
- `gofmt -r`: Go rewriter
- `gofmt -s`: Go simplifier
- `go fix`: API updater
- `goimports`: Import management

**Cultural impact**: Initial resistance ("gofmt doesn't match my style!") gave way to
mandatory enforcement, then paradigm shift, and current status as a major Go selling
point. Formatting is now a non-issue in the Go community.

**Sources**:
- [gofmt blog post](https://go.dev/blog/gofmt)
- [The Cultural Evolution of gofmt](https://go.dev/talks/2015/gofmt-en.slide)

---

## C / C++

### clang-format

**Repository**: Part of the LLVM project

**Philosophy**: Highly configurable. Supports predefined styles (LLVM, Google,
Chromium, Mozilla, WebKit, Microsoft) and custom configurations. Designed for C, C++,
Java, JavaScript, and Objective-C.

**Algorithm**: Penalty-based line breaking using breadth-first search (BFS) with a
priority queue. The algorithm:

1. Parses code into "unwrapped lines" (logical lines before formatting)
2. For each unwrapped line, explores different line-breaking possibilities
3. Inserts states that break the line as late as possible first (so in case of equal
   penalties, states inserted first are preferred)
4. Each potential break point has an associated penalty
5. The algorithm finds the state with the lowest total penalty

**Penalty system**: Many penalty options are configurable (e.g., penalty for breaking
after assignment, penalty for breaking before first argument). Hard-coded penalties
exist for specific situations not worth making configurable.

**Complexity management**: For long and deeply nested unwrapped lines, the algorithm
has built-in safeguards. A flag can be set to skip analyzing some combinations, though
these rarely contain the optimal solution.

**Configuration**: Via `.clang-format` or `_clang-format` files in YAML format. Can
consist of multiple sections targeting different languages. Supports hierarchical
configuration with `InheritParentConfig` for subdirectory overrides.

**Error recovery**: Can format code with syntax errors (lenient reader), unlike
formatters that require valid syntax.

**Key style options**: Column limit, indentation width, alignment of consecutive
assignments/declarations, brace wrapping, space before parentheses, and hundreds more.

**Sources**:
- [ClangFormat documentation](https://clang.llvm.org/docs/ClangFormat.html)
- [Clang-Format Style Options](https://clang.llvm.org/docs/ClangFormatStyleOptions.html)

---

## Scala

### scalafmt

**Repository**: https://github.com/scalameta/scalafmt

**Philosophy**: Opinionated but configurable. Captures popular Scala idioms and coding
styles. Supports both Scala 2 and Scala 3 syntax.

**Algorithm**: Best-first graph search for optimal line wrapping. The architecture:

1. Parse source file using `scala.meta`
2. Feed a sequence of `FormatToken` data types into a `LineWrapper`
3. The `LineWrapper` uses a `Router` to construct a weighted directed graph
4. Run best-first search to find an optimal formatting layout for the whole file

This approach is similar to clang-format's penalty-based search but uses graph
search rather than BFS with a priority queue.

**Configuration**: Via `.scalafmt.conf` in HOCON syntax. Settings include `version`,
`runner.dialect`, `align.preset`, `maxColumn`, and many more. The configuration
documentation notes that many parameters don't have ultimate authority and might be
overridden by other parameters.

**Thesis**: The algorithm and its language-agnostic components are described in
Olafur Pall Geirsson's thesis, which covers the data structures for implementing line
wrapping with a maximum line length setting and configurable vertical alignment.

**Sources**:
- [scalafmt documentation](https://scalameta.org/scalafmt/)
- [scalafmt thesis (PDF)](https://geirsson.com/assets/olafur.geirsson-scalafmt-thesis.pdf)

---

## Haskell

### Ormolu

**Repository**: https://github.com/tweag/ormolu

**Philosophy**: "One true formatting style" for Haskell. No configuration options.
Consistency across projects matters more than individual preferences.

**Parser**: Uses `ghc-exactprint` which leverages GHC's own parser (not the independent
`haskell-src-exts` library). This guarantees parsing accuracy since it mirrors the
compiler's behavior.

**Layout decisions**: A distinctive approach: programmers control whether code appears
on one line or multiple lines. Once a multi-line choice is made, the formatter
introduces additional breaks in parent nodes but not in sibling or children nodes.
This creates predictable, hierarchical formatting without aggressive automatic line
wrapping. The formatter does NOT use a line-length-based algorithm.

**Comment handling**: Comments are attached to specific syntactic entities in the AST.
Moving an entity moves its comment too, maintaining semantic associations. The
`CommentStream` is an ascending-order stream of located comments with metadata about
whether atoms preceded the comment in original input.

**Idempotency**: Guaranteed through integrated AST verification. The program tests if
the produced AST equals the one originally parsed.

**CPP limitation**: CPP directives can alter code meaning during formatting. Rather
than complex preservation logic, the developers advocate for replacing CPP with
future language extensions.

**Sources**:
- [Ormolu announcement](https://www.tweag.io/blog/2019-05-27-ormolu/)
- [Ormolu DESIGN.md](https://github.com/tweag/ormolu/blob/master/DESIGN.md)

---

### Fourmolu

**Repository**: https://github.com/fourmolu/fourmolu

**Philosophy**: A fork of Ormolu that adds configuration options. "A less-opinionated
version of Ormolu." Upstream improvements from Ormolu are continually merged.

Shares Ormolu's architecture and GHC-based parsing but allows users to configure
formatting preferences that Ormolu hardcodes.

**Sources**:
- [Fourmolu documentation](https://fourmolu.github.io/)

---

## Elm

### elm-format

**Repository**: https://github.com/avh4/elm-format

**Philosophy**: Zero-configuration, inspired by gofmt. Formats Elm source code
according to a standard set of rules based on the official Elm Style Guide. The tool
is intentionally opinionated about all formatting decisions.

**Benefits articulated by the project**:
- Makes code easier to write (no worry about formatting concerns)
- Makes code easier to read (no distracting stylistic differences)
- Allows the brain to map more efficiently from source to mental model
- Reduces code review overhead (no style debates)

**Approach**: Like gofmt, elm-format is a line-preserving formatter that respects the
user's line break decisions while normalizing whitespace and indentation.

**Sources**:
- [elm-format GitHub](https://github.com/avh4/elm-format)
- [Elm Radio Episode 23: elm-format](https://elm-radio.com/episode/elm-format/)

---

## OCaml

### OCamlformat

**Repository**: https://github.com/ocaml-ppx/ocamlformat

**Philosophy**: End-to-end parsing and printing, inspired by ReasonML's refmt.
Configurable through profiles. Has evolved from an "ocamlformat" default profile to
a "conventional" default profile.

**Historical context**: OCaml has a deep connection to pretty-printing algorithms.
The `Format` module in OCaml's standard library comes directly from Oppen's seminal
article. The module exhibits Oppen's limitations: box offsets cannot depend on future
content, but it begins printing without needing the full document.

**Profiles**:
- **Conventional (default)**: Aims to match the most commonly used style
- **OCamlformat**: Optimizes for what the formatter can do best rather than matching
  existing code styles
- **Janestreet**: Used at Jane Street

**History**: Owes its existence to Josh Berdine. Facebook fostered its inception as
part of their work on ReasonML. Guillaume Petiot and Jules Aguillon made significant
contributions.

**Sources**:
- [OCamlformat GitHub](https://github.com/ocaml-ppx/ocamlformat)
- [Format Unraveled (paper)](https://rbonichon.github.io/papers/format-unraveled.pdf)

---

## Dart

### dart format (formerly dartfmt)

**Repository**: https://github.com/dart-lang/dart_style

**Philosophy**: Opinionated, minimally configurable. Aims to end style debates in code
reviews. Supports very few tweakable settings by design.

**Algorithm**: Best-first graph search with a cost model. The most sophisticated
line-breaking algorithm among major formatters. Described by its author Bob Nystrom as
"the hardest program I've ever written."

**Core data structures**:

- **Chunks**: Atomic units of formatting representing contiguous character regions that
  won't contain line breaks. Organized hierarchically to reflect block nesting.
- **Rules**: Each potential split point is owned by a rule controlling which chunks
  break based on the rule's value. Simple rules: 0 = no splits, 1 = all split.
  Complex rules (e.g., argument lists) support multiple configurations.
- **Spans**: Mark contiguous chunks to avoid splitting, functioning like rubber bands.
  When a span breaks, the solution incurs a penalty. Nested spans teach the formatter
  to prefer higher-level splits.

**Best-first search algorithm**:

1. Each graph node = a partial solution (set of rule values, some unbound)
2. Edges connect partial solutions by binding one additional rule
3. Algorithm explores from empty solution toward complete solutions
4. Three optimization heuristics:
   - **Early termination**: Minimizing overflow dominates cost minimization. Queue
     sorted by cost; first solution fitting within column limit is optimal.
   - **Focused expansion**: Only bind rules with chunks on overflowing lines.
   - **Branch pruning**: When two partial solutions have identical unbound rules,
     lower cost dominates if bound rules don't interact with unbound rules.
5. Escape hatch: After 5,000 solutions without convergence, accept best found.

**Cost model**: Nearly every chunk and span has a cost of 1. Nestedness naturally
controls splits. Previous attempts to tune individual costs were problematic: "like a
hanging mobile where tweaking one cost would unbalance all of the others."

**Tall style (Dart 3.7+)**: The formatter now automatically manages trailing commas.
It adds them to argument/parameter lists that split across multiple lines and removes
them from ones that don't. Previous versions required manual trailing comma management.

**Sources**:
- [dart_style GitHub](https://github.com/dart-lang/dart_style)
- [The Hardest Program I've Ever Written](https://journal.stuffwithstuff.com/2015/09/08/the-hardest-program-ive-ever-written/)

---

## Cross-Cutting Themes

### Line-Preserving vs. Line-Breaking

Formatters fall into two categories:

**Line-preserving** (gofmt, elm-format, Ormolu): Maintain existing line breaks while
adjusting whitespace within lines. Simpler to implement, respect user intent, avoid
the exponential complexity of line-breaking decisions.

**Line-breaking** (Prettier, Black, clang-format, dartfmt, scalafmt): Honor page width
limits by adding or consolidating breaks. At the heart of line-breaking formatting is
pretty printing, where there can be exponentially many formatting choices.

### Opinionated vs. Configurable

| Opinionated | Configurable |
|-------------|-------------|
| Black, gofmt, Ormolu, elm-format | YAPF, clang-format, scalafmt, OCamlformat |
| Prettier (few options) | rustfmt (moderate options) |

### Editor Integration Patterns

All major formatters support:
- Format on save
- Format selection / range formatting
- CI integration (check mode that exits nonzero on unformatted code)
- LSP integration (textDocument/formatting)

### Lenient vs. Strict Parsing

- **Lenient** (clang-format): Formats code with syntax errors
- **Strict** (Prettier, Black): Require valid syntax to format

### Performance Comparison (approximate, Python formatters on ~250k lines)

| Formatter | Time | Language |
|-----------|------|----------|
| Ruff | 0.10s | Rust |
| Black | 3.20s | Python (mypyc) |
| autopep8 | 17.77s | Python |
| YAPF | 19.56s | Python |
