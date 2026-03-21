# Code Formatting: Algorithms, IR Design, and Architectural Lessons

A technical deep-dive into the foundational algorithms, intermediate representation
design, and engineering patterns that underpin modern code formatters.

---

## Table of Contents

1. [The Pretty-Printing Problem](#the-pretty-printing-problem)
2. [Foundational Algorithms](#foundational-algorithms)
3. [Intermediate Representation Design](#intermediate-representation-design)
4. [Line-Breaking Strategies](#line-breaking-strategies)
5. [Comment Handling](#comment-handling)
6. [Macros, DSLs, and String Interpolation](#macros-dsls-and-string-interpolation)
7. [Idempotency and Stability](#idempotency-and-stability)
8. [Performance Engineering](#performance-engineering)
9. [Architectural Patterns](#architectural-patterns)
10. [Design Tradeoffs](#design-tradeoffs)
11. [Lessons for New Formatter Design](#lessons-for-new-formatter-design)

---

## The Pretty-Printing Problem

The core problem: given a tree-structured representation of code, produce a
human-readable string that respects a maximum line width. This requires deciding
where to insert line breaks, and each break decision can affect downstream decisions,
creating a combinatorial explosion.

**Why it's hard**: There can be exponentially many formatting choices. A naive approach
that tries all combinations is intractable. The challenge is finding algorithms that
are efficient (ideally linear time), produce good output (ideally optimal), and can
begin producing output without seeing the entire input (bounded lookahead).

**Three desirable properties** (only two achievable simultaneously):

1. **Optimality**: Choose the layout that minimizes some cost function
2. **Boundedness**: Make decisions after seeing at most `w` characters (where `w` is
   line width)
3. **Efficiency**: O(n) time and O(w) space

Hughes noted that "it would be unreasonably inefficient for a pretty-printer to decide
whether or not to split the first line of a document on the basis of the content of
the last." He therefore chose a greedy algorithm. Wadler showed his algorithm achieves
all three properties simultaneously.

**Sources**:
- [UW PLSE: The Science of Code Formatting](https://uwplse.org/2024/03/04/code-formatting.html)
- [Wikipedia: Pretty-printing](https://en.wikipedia.org/wiki/Prettyprint)

---

## Foundational Algorithms

### Derek Oppen's Algorithm (1980)

**Paper**: "Prettyprinting" - ACM TOPLAS, Vol. 2, No. 4, October 1980, pp. 465-483
([ACM Digital Library](https://dl.acm.org/doi/pdf/10.1145/357114.357115))

**Architecture**: Two parallel processes communicating through a buffer of size O(m):
1. **Scanner**: Adds tokens to the right end of a buffer, computing the space required
   to print each logical block
2. **Printer**: Uses scanner information to decide where to break lines

**Performance**: O(n) time, O(m) space for input length n and margin width m. Crucially,
it does not wait for the entire stream -- it begins printing as soon as it has received
a lineful of input. This streaming property is unique among the foundational algorithms.

**Key data structures**:
- `scan()` function that adds tokens and computes associated integers
- `leftotal` variable tracking total spaces needed for buffer contents
- `stream` and `size` arrays implemented as ring buffers

**Properties**: If the input contains enough actual text (not just administrative
tokens), pretty-printing uses constant space and time linear in input size.

**Influence**: OCaml's `Format` module is directly descended from Oppen's algorithm.
The imperative, streaming nature suits languages where memory is constrained or output
must begin before input is complete.

**Limitation**: Box offsets cannot depend on future content. The algorithm makes
irrevocable decisions as tokens stream through.

**Source**: [Stanford CS-TR-79-770](http://i.stanford.edu/pub/cstr/reports/cs/tr/79/770/CS-TR-79-770.pdf)

---

### John Hughes's Pretty-Printing Library (1995)

**Paper**: "The Design of a Pretty-printing Library" in Advanced Functional
Programming, 1995
([Chalmers](https://www.cse.chalmers.se/~rjmh/Papers/pretty.html))

**Approach**: Combinator-based. Uses algebraic properties of combinators to guide both
design and implementation. Requires higher-order functions and lazy evaluation
(Haskell).

**Key insight**: Studying the algebra led to correction of a subtle error in
combinator behavior and to much more efficient implementations. The library provides
combinators for horizontal and vertical composition, nesting, and grouping.

**Algorithm**: Greedy. Tries to fit as much as possible on a single line without
regard for what comes next. Hughes acknowledged this tradeoff explicitly.

**Limitation**: No algorithm for Hughes's combinators is both optimal and bounded.
Spurious line breaks can appear, making output longer than necessary. A document laid
out after another cannot always be properly indented.

**Legacy**: Versions used in both Chalmers and Glasgow Haskell compilers, program
transformation tools, and proof assistants. The Hughes-Peyton Jones library was
distributed with GHC for years, making it the de facto standard Haskell pretty printer.

---

### Philip Wadler's "A Prettier Printer" (1998/2003)

**Paper**: "A prettier printer" ([PDF](https://homepages.inf.ed.ac.uk/wadler/papers/prettier/prettier.pdf))

**Key contribution**: Reconstructed a pretty-printing library from scratch using
equational laws and derived implementation. Uses an algorithm equivalent to Oppen's
but presented in functional style.

**Critical achievement**: The algorithm is both **optimal** and **bounded**. It
chooses line breaks to avoid overflow whenever possible, and makes this choice after
looking at no more than the next `w` characters. Hughes explicitly noted that no
algorithm for his combinators has both properties.

**Core operations**:

| Operation | Semantics |
|-----------|-----------|
| `nil` | Empty document |
| `text s` | Literal string |
| `line` | Line break or space |
| `nest i x` | Indent by `i` |
| `x <> y` | Concatenation |
| `group x` | Replace line breaks with spaces if content fits |
| `flatten x` | Force single-line layout (replace all `line` with space) |

**Two layout modes**:
- **Flat**: Content on a single line (within a flattened group)
- **Broken**: Content split across multiple lines

**The `fits()` function**: Determines whether content fits on the current line by
calculating remaining space (`max_width - current_column`), scanning ahead through the
document. When encountering a `Newline`, returns `true`. When text exceeds remaining
space, returns `false`.

**Rendering**: The renderer measures grouped content length. If it exceeds print width,
broken layout applies. Otherwise, flat layout renders. The measurement approach drives
layout selection automatically.

**Wadler-Leijen extension**: Daan Leijen took Wadler's implementation and modified it
to increase expressivity, producing the `wl-pprint` package. Seven of the 19 pretty
printing libraries on Hackage are derived from Leijen's implementation.

**Influence**: Direct ancestor of Prettier's algorithm, Biome's formatter, Ruff's
formatter, Elixir's formatter (in the standard library), and dozens of
language-specific formatting tools.

**Sources**:
- [Wadler's paper (PDF)](https://homepages.inf.ed.ac.uk/wadler/papers/prettier/prettier.pdf)
- [Paiges: Scala implementation](https://github.com/typelevel/paiges)
- [prettier4j: Java implementation](https://github.com/opencastsoftware/prettier4j)
- [wadler_lindig: Python implementation](https://github.com/patrick-kidger/wadler_lindig)

---

### A Twist on Wadler's Printer (Pombrio, 2024)

**Post**: [justinpombrio.net](https://justinpombrio.net/2024/02/23/a-twist-on-Wadlers-printer.html)

**Innovation**: Replaces Wadler's `group`/`flatten` with `choice`/`flat`:

- `x | y` (Choice): Arbitrary alternatives. Printer selects left if it fits, right
  otherwise. Enables choices beyond whitespace (e.g., trailing commas in multi-line).
- `flat(x)`: Forces every choice to select its leftmost option.

**The Critical Rule**: For every choice `x | y`, the shortest possible first line of
`y` must be at least as short as every possible first line of `x`. This constraint
enables greedy decisions without exponential exploration.

**`fits()` modification**: When encountering a nested choice during lookahead, it only
checks the right option (`y`), relying on The Rule for safety.

**Implementation**: Maintains a chunk stack with notations paired with accumulated
indentation and flat-mode status. Processing is iterative (no recursion), maintaining
linear performance.

---

### Yelland's Optimal Code Formatting (2016)

**Paper**: "A New Approach to Optimal Code Formatting" by Phillip M. Yelland, Google
([Google Research](https://research.google/pubs/pub44667/))

**Innovation**: Uses dynamic programming directly to optimize layout cost, rather than
the indirect approach of clang-format (Dijkstra's algorithm, itself a form of dynamic
programming).

**Key concept**: Associates each layout expression with a "minimum cost function" that
maps a column to the minimum cost incurred by that layout when started at that column.
This enables composing layout costs algebraically.

**Advantage over clang-format**: More principled optimization. Clang-format uses
penalty-based BFS which can fail to find optimal solutions for deeply nested code.
Yelland's approach guarantees optimality through the dynamic programming formulation.

**Adopted by**: YAPF (Google's Python formatter) draws on similar ideas from
clang-format's algorithm.

---

### Swierstra-Chitil Linear Functional Pretty Printing

**Paper**: "Linear, Online, Functional Pretty Printing" (Chitil, 2005)
([Kent CS](https://www.cs.kent.ac.uk/people/staff/oc/pretty.html))

**Achievement**: A combinator-based functional pretty-printing algorithm that retains
the linear space and time complexities of Oppen's algorithm. Like Oppen's, it does
not need the full document to start printing.

**Significance**: Bridges the gap between Oppen's streaming efficiency and Wadler's
functional elegance. Proves that functional style does not require sacrificing Oppen's
performance properties.

---

### "A Pretty But Not Greedy Printer" (Bernardy, 2017)

**Paper**: [PDF](https://jyp.github.io/pdf/Prettiest.pdf)

**Approach**: Respects both legibility and frugality but gives up greediness. The
algorithm is fast enough for common pretty-printing tasks despite not being greedy.
Explores the tradeoff space differently than Wadler (who gives up nothing but
expressiveness of Hughes's combinators).

---

### Algorithm Comparison Summary

| Algorithm | Time | Space | Optimal | Bounded | Streaming | Style |
|-----------|------|-------|---------|---------|-----------|-------|
| Oppen (1980) | O(n) | O(m) | Yes | Yes | Yes | Imperative |
| Hughes (1995) | O(n) | O(n) | No | No | No | Functional |
| Wadler (1998) | O(n*m) | O(n) | Yes | Yes | No | Functional |
| Chitil (2005) | O(n) | O(m) | Yes | Yes | Yes | Functional |
| Yelland (2016) | O(n*m) | O(n*m) | Yes | No | No | DP-based |
| Bernardy (2017) | varies | varies | Yes | No | No | Functional |

---

## Intermediate Representation Design

### The Doc IR Pattern

The dominant pattern across modern formatters: parse source to a tree, convert the
tree to an intermediate "document" representation, then render the document to a
string. The IR abstracts over concrete formatting decisions, allowing the renderer to
make line-breaking choices.

### Prettier's Doc Commands (the reference implementation)

Prettier's IR is the most widely adopted and cloned. Its commands fall into categories:

**Layout primitives**:
- `line`: Space when flat, break when broken
- `softline`: Nothing when flat, break when broken
- `hardline`: Always break (propagates to parents)
- `literalline`: Always break, no indent change

**Grouping**:
- `group(doc)`: Try flat first, break if doesn't fit
- `conditionalGroup(alternatives)`: Try alternatives in order (expensive)
- `fill(docs)`: Text-layout mode (break only as needed)

**Indentation**:
- `indent(doc)`: +1 level
- `dedent(doc)`: -1 level
- `align(width, doc)`: Fixed-width alignment

**Conditional**:
- `ifBreak(broken, flat)`: Different content based on break state
- `indentIfBreak(doc, {groupId})`: Conditional indent tied to group
- `breakParent`: Force parent groups to break

**Metadata**:
- `lineSuffix(doc)`: Buffer for trailing comments
- `label(label, doc)`: Metadata attachment
- `cursor`: Position tracking
- `trim`: Remove current-line indentation

### Biome's FormatElement (Prettier-compatible, Rust-native)

Biome's IR closely mirrors Prettier's but uses Rust enums:

- `Text`: Literal content
- `Space`: Single space
- `Line(SoftLine | HardLine | HardOrSpace)`: Break points
- `Group`: Unified formatting block
- `Indent` / `Dedent`: Indentation control
- `Fill`: Compact layout mode

### Black's CST-Based Approach (no explicit IR)

Black does not use a separate IR. Instead, it operates directly on the Concrete Syntax
Tree from blib2to3. The CST preserves all syntactic information, and Black modifies
whitespace tokens in-place. The formatting algorithm walks the CST, decides on
line breaks based on line-length calculations, and mutates the tree.

This works because Black's algorithm is simpler (hierarchical bracket decomposition)
and doesn't need the expressive power of a Wadler-style IR.

### dartfmt's Chunks/Rules/Spans (custom IR)

dart format uses a unique IR designed for its graph-search algorithm:

- **Chunks**: Atomic text regions (no internal breaks)
- **Rules**: Control split decisions at potential break points (with multiple possible
  values, not just binary)
- **Spans**: Grouping mechanism with penalty costs for breaking

This IR is optimized for the best-first search: it naturally represents the graph of
partial solutions that the algorithm explores.

### Key IR Design Lessons

1. **Flat/broken duality is fundamental**: Every IR needs a way to express "try flat,
   fall back to broken." This is the core insight from Wadler.

2. **Group nesting determines break order**: Outermost groups break first. This is
   counterintuitive but produces better output than breaking innermost first.

3. **Fill mode is essential for prose-like content**: Regular groups are all-or-nothing
   (every break in the group triggers). Fill breaks only as needed, crucial for
   argument lists and markdown.

4. **Break propagation needs control**: `breakParent` is powerful but can cause
   unwanted cascading. Variants like `hardlineWithoutBreakParent` provide escape
   hatches.

5. **LineSuffix solves comment placement**: Trailing comments need special handling.
   Buffering them until the next line break is the cleanest solution.

6. **Conditional content enables polish**: `ifBreak` allows different formatting in
   flat vs broken mode (e.g., trailing commas only in multi-line).

---

## Line-Breaking Strategies

### Strategy 1: Greedy (Wadler/Prettier)

**How it works**: Try to fit content on the current line. If it doesn't fit, break at
the outermost group and try again.

**`fits()` function**: Scan ahead through the document, consuming remaining line width.
If a newline is encountered before width is exhausted, it fits. If width goes negative,
it doesn't.

**Advantages**: Fast (linear time), simple implementation, bounded lookahead.

**Disadvantages**: Not globally optimal. A greedy decision on line 1 might force
suboptimal formatting on line 2. In practice, this is rarely noticeable for code.

**Used by**: Prettier, Biome, Ruff, Elixir formatter, most Wadler-derived formatters.

### Strategy 2: Penalty-Based Search (clang-format)

**How it works**: Each potential break point has a penalty. The algorithm uses BFS with
a priority queue to find the line-breaking configuration with minimum total penalty.

**Advantages**: Can find better solutions than greedy for complex cases.

**Disadvantages**: Potentially exponential time for deeply nested code. clang-format
has safeguards (skip some combinations after a threshold).

**Used by**: clang-format, YAPF (inspired by).

### Strategy 3: Best-First Graph Search (dartfmt)

**How it works**: Represent the space of all possible formatting configurations as a
graph. Each node is a partial solution (some rules bound, some not). Best-first search
explores solutions in order of increasing cost.

**Three key optimizations**:
1. Overflow dominates cost (early termination)
2. Only bind rules for overflowing lines (focused expansion)
3. Prune dominated partial solutions (branch elimination)

**Escape hatch**: After 5,000 explored solutions, accept the best found so far.

**Advantages**: More principled than penalty-based search. Can handle complex Dart
formatting idioms (named parameters, cascades, collection literals).

**Disadvantages**: Complex implementation. The escape hatch means non-optimal results
are possible for pathological inputs.

**Used by**: dart format.

### Strategy 4: Dynamic Programming (Yelland/rfmt)

**How it works**: Associate each layout expression with a minimum cost function mapping
column -> minimum cost. Compose these functions algebraically through the layout tree.

**Advantages**: Guaranteed optimal. Principled mathematical foundation.

**Disadvantages**: O(n * m) time and space. More complex implementation than greedy.

**Used by**: Google's rfmt formatter.

### Strategy 5: Line-Preserving (gofmt)

**How it works**: Don't make line-breaking decisions at all. Preserve the user's line
breaks, only adjust whitespace within lines.

**Advantages**: Simple, fast, deterministic, respects user intent.

**Disadvantages**: Output quality depends on input quality. Two equivalent programs
can produce different formatted output.

**Used by**: gofmt, Ormolu (partially), elm-format.

### Strategy 6: Machine Learning (CodeBuff)

**Repository**: https://github.com/antlr/codebuff

**How it works**: Language-agnostic pretty-printing through machine learning. Learns
formatting patterns from a corpus of existing code. Uses k-NN to decide formatting
at each token position.

**Significance**: Demonstrates that formatting can be learned rather than hand-coded,
but hasn't been adopted in production formatters due to unpredictability.

---

## Comment Handling

Comments are the hardest part of formatting. They exist outside the formal grammar,
can appear between any two tokens, and their association with code is ambiguous.

### Approach 1: Store in Whitespace (Black/lib2to3)

Comments are shoved into the whitespace prefix of the next token during parsing
(`pgen2/driver.py:Driver.parse_tokens()`). This avoids modifying the grammar to include
all possible comment positions but makes manipulation harder.

### Approach 2: Comment Groups (gofmt)

Comments are grouped into `CommentGroup` structures (consecutive comments with no
intervening tokens or empty lines). A sequential list is attached to `ast.File`. Doc
comments are also attached to declaration nodes as metadata. The printer merges the
"token stream" with the "comment stream" based on position information.

**Acknowledged weakness**: Not attaching comments to AST nodes initially was described
by the Go team as the "biggest mistake" in gofmt's design. The `ast.CommentMap`
workaround is "cludgy."

### Approach 3: Comment Attachment (Ormolu)

Comments are attached to specific syntactic entities in the AST after parsing. Moving
an entity moves its comment. The `CommentStream` maintains ordering and metadata about
surrounding atoms. This is the cleanest approach but requires careful post-parse
processing.

### Approach 4: LineSuffix (Prettier)

Trailing comments use the `lineSuffix` command to buffer content until the next line
break. The `lineSuffixBoundary` command provides explicit flush points. This prevents
comments from "escaping" embedded code regions (e.g., a comment in a template literal
expression shouldn't appear outside it).

### Key Lessons for Comment Handling

1. **Attach comments to AST nodes during parsing** -- retrofitting is painful
2. **Distinguish leading, trailing, and dangling comments** -- they have different rules
3. **Trailing comments need buffering** -- they must appear before the next line break
4. **Comment relocation can break tools** -- tools assigning meaning to specific
   positions (e.g., type annotations, pragma comments) can be affected by reformatting
5. **Preserve comment content exactly** -- never modify comment text, only whitespace
   around it

---

## Macros, DSLs, and String Interpolation

### The Macro Problem

Formatters that work on ASTs before macro expansion face a fundamental challenge:
the macro's input may not be valid syntax in the host language.

**rustfmt's experience**: rustfmt sees macro calls in raw form with tokens as written
in source. Since it only formats valid Rust syntax, it cannot process arbitrary macro
content. Worse:
- Trailing commas can be semantically important in macros
- Procedural macros using `Span` information can change behavior when formatting
  changes token locations
- Non-idempotent formatting in some macro contexts
- Workaround: `#[rustfmt::skip::macros(name)]`

**General approaches**:
1. **Skip macro content entirely** -- safe but leaves code unformatted
2. **Treat macro content as host-language syntax** -- works sometimes, breaks when
   macro syntax diverges
3. **Format after expansion** -- can't round-trip back to source
4. **Language-specific macro handling** -- most complex but most correct

### String Interpolation

**Prettier's approach**: Uses a heuristic where interpolation expressions only split
across multiple lines if there was already a linebreak within the interpolation in the
original source. A literal will not be broken onto multiple lines even if it exceeds
print width.

### Embedded DSLs

Most formatters have limited DSL support. The UW PLSE analysis notes that "most
line-breaking formatters struggle with extensibility, especially for macro languages."
Approaches:
- **Injected language formatters**: Apply a different formatter to embedded content
  (e.g., format SQL inside a string literal)
- **Opaque treatment**: Leave embedded content untouched
- **Tagged regions**: Use comments or attributes to switch formatting modes

---

## Idempotency and Stability

### Idempotency Guarantees

**Strong idempotency** (Black, gofmt, Ormolu): Running the formatter twice produces
bit-identical output. `format(format(x)) == format(x)` for all valid inputs.

**Weak idempotency** (Prettier): Mostly idempotent but with known edge cases. Object
literals that become multiline won't collapse back. Adding then removing a property
can leave formatting different from the initial state.

**Verification approaches**:
- **AST comparison** (Black, Ormolu): Parse both input and output, verify AST
  equivalence
- **Fixed-point iteration** (rustfmt): Repeat formatting until output stabilizes
- **Test suites**: Extensive test cases covering known edge cases

### Stability Guarantees

**rustfmt** (RFC 2437): Strongest stability guarantee among major formatters. A newer
version cannot modify formatted output from a previous version, under specific
conditions (default config, stable Rust, error-free output). Style editions (RFC 3338)
allow evolution while maintaining backward compatibility.

**Prettier**: Uses semantic versioning but formatting changes can occur in minor
versions. No formal stability RFC.

**Black**: Uses a "year.month.patch" versioning scheme. Formatting changes are
documented in changelogs but can occur between versions.

---

## Performance Engineering

### Parsing Dominance

In most formatters, parsing is the primary bottleneck:
- **Black**: blib2to3 parsing = 30-50% of runtime
- **rustfmt**: Parsing = up to 60% of single-file runtime
- **Lesson**: Faster parsing directly translates to faster formatting

### Parallelization

Formatters that process multiple files benefit enormously from parallelism:
- **rustfmt**: Rayon-based parallelism, 8 threads for million-line codebases (~20s)
- **Biome**: Multi-threaded analysis and formatting
- **Ruff**: Rayon-based, no GIL limitation (30-100x faster than Python formatters)
- **Caveat**: Avoid over-parallelizing small files due to scheduler overhead

### Incremental Formatting

- **Biome**: Only re-parses changed portions of files
- **rustfmt**: 2-3x reduction for files with minor changes
- **Key challenge**: Ensuring incremental results equal full-reformat results

### Language Choice

Native implementations (Rust, Go) dramatically outperform interpreted ones:

| Formatter | Language | Time (250k lines) |
|-----------|----------|-------------------|
| Ruff | Rust | 0.10s |
| gofmt | Go | ~0.5s (est.) |
| Black | Python+mypyc | 3.20s |
| autopep8 | Python | 17.77s |
| YAPF | Python | 19.56s |

### Algorithmic Complexity Matters

- Greedy (Wadler): O(n) -- scales linearly
- Penalty search (clang-format): Can explode on deeply nested code (safeguards needed)
- Graph search (dartfmt): 5,000-solution escape hatch for pathological cases

---

## Architectural Patterns

### Pattern 1: Parse-Transform-Print

The dominant architecture. Three phases with clear interfaces:

```
Source -> [Parser] -> Tree -> [Transformer] -> IR -> [Printer] -> Output
```

**Variants**:
- **AST-based** (gofmt): Parse to AST, print directly from AST
- **CST-based** (Black, Biome): Parse to CST (preserves comments/whitespace), print
  from CST or convert to IR
- **Two-phase IR** (Prettier, Biome): Parse to tree, convert to Doc IR, render IR

### Pattern 2: Language-Agnostic Core + Language-Specific Plugins

**Prettier**: Core printer algorithm is language-agnostic. Language support via
parser + printer plugins. Each plugin converts AST nodes to Doc commands.

**Biome**: `biome_formatter` crate provides language-agnostic IR infrastructure.
Language-specific crates (`biome_js_formatter`, etc.) provide AST-to-IR conversion.

**Key benefit**: New language support only requires a parser and AST-to-IR converter.
The line-breaking algorithm, indentation logic, and rendering are shared.

### Pattern 3: Safety Verification

**Black**: Parses both input and output to AST, verifies equivalence. Slows down
processing but guarantees semantic preservation.

**Ormolu**: Tests if produced AST equals originally parsed AST.

**rustfmt**: Buffer management system for atomic writes. File modifications either
complete fully or abort on error.

### Pattern 4: Configuration Layering

**clang-format**: YAML configuration, multi-section (per-language), hierarchical
inheritance (`InheritParentConfig`), predefined style presets.

**rustfmt**: TOML configuration, stable vs unstable options, style editions tied to
language editions.

**Pattern**: Configuration should be simple to start (opinionated defaults) with
progressive disclosure of options.

---

## Design Tradeoffs

### Tradeoff 1: Opinionated vs Configurable

**Opinionated** (Black, gofmt, Ormolu, elm-format):
- Eliminates style debates
- Simpler implementation
- Universal consistency
- Risk: Some users reject the chosen style

**Configurable** (clang-format, YAPF, scalafmt):
- Accommodates existing codebases
- More complex implementation and testing
- Configuration becomes a source of debates itself
- Risk: Inconsistency across projects

**Middle ground** (rustfmt, Prettier): Few options, strong defaults, progressive
disclosure.

### Tradeoff 2: Line-Preserving vs Line-Breaking

**Line-preserving** (gofmt):
- Simpler implementation
- Respects author intent
- Cannot enforce line width
- Two equivalent programs may format differently

**Line-breaking** (Prettier, Black):
- Enforces consistent line width
- More complex (exponential decision space)
- Can produce surprising output
- Handles author intent through heuristics (magic trailing comma, etc.)

### Tradeoff 3: AST vs CST

**AST** (gofmt, Ormolu):
- Cleaner representation
- Comments are a problem (not in the AST)
- Simpler tree traversal

**CST** (Black, Biome):
- Preserves all syntactic information including whitespace and comments
- More complex representation
- Natural home for comments and trivia

### Tradeoff 4: Streaming vs Document-Based

**Streaming** (Oppen):
- O(m) space, begins output immediately
- Cannot make decisions based on future content
- Ideal for memory-constrained or real-time scenarios

**Document-based** (Wadler, Hughes):
- Requires full document before rendering
- Can make globally-informed decisions
- More expressive combinators
- Algebraic reasoning about layouts

### Tradeoff 5: Greedy vs Optimal

**Greedy** (Prettier/Wadler):
- Fast, simple, bounded lookahead
- Occasionally suboptimal (early decisions constrain later ones)
- Good enough for virtually all real code

**Optimal** (dartfmt, clang-format, Yelland):
- Better output for complex cases
- More expensive (search, DP)
- Need escape hatches for pathological inputs
- Complexity may not be worth it for most use cases

---

## Lessons for New Formatter Design

### Start with Wadler

The Wadler/Prettier algorithm is the proven default. It's well-understood, has
extensive prior art, and handles 99% of real code well. Start here and only add
complexity if specific formatting requirements demand it.

### The IR is the API

The intermediate representation is the most important design decision. It determines
what formatting layouts are expressible and how language plugins interact with the
core. Prettier's Doc commands are the reference design. Key commands to support:
- `group` (flat/broken duality)
- `indent` / `dedent`
- `line` / `softline` / `hardline`
- `fill` (text-layout mode)
- `ifBreak` (conditional content)

### Handle comments from the start

Every formatter team says the same thing: comments are the hardest part. Design
comment attachment into the parser from day one. Don't retrofit it later (see gofmt's
"biggest mistake").

### Idempotency is non-negotiable

Users expect `format(format(x)) == format(x)`. Test this aggressively. Consider AST
comparison as a safety check (Black's approach).

### Performance comes from the parser

If formatting is slow, optimize the parser first. Consider:
- Native language implementation (Rust, Go)
- Incremental parsing for editor integration
- Parallelism for multi-file formatting
- Sharing parser infrastructure with linters

### Skip macros gracefully

Don't try to format arbitrary macro content. Provide a skip mechanism
(`#[rustfmt::skip]`, `// prettier-ignore`). Format macro content only when it
follows known syntax patterns.

### Stability matters for adoption

Once people depend on your formatter in CI, changing output is a breaking change.
Consider stability guarantees, versioned formatting styles, and opt-in upgrades
(rustfmt's style editions model is the gold standard).

### Configuration is a spectrum

The evidence suggests starting opinionated (fewer debates, simpler implementation)
and adding options only when user demand is overwhelming and the options don't
interact in complex ways. The most successful formatters (gofmt, Black, Prettier)
have few options.

---

## References

### Foundational Papers

- Oppen, D.C. "Prettyprinting." ACM TOPLAS 2(4), 1980. [ACM](https://dl.acm.org/doi/pdf/10.1145/357114.357115)
- Hughes, J. "The Design of a Pretty-printing Library." AFP, 1995. [Chalmers](https://www.cse.chalmers.se/~rjmh/Papers/pretty.html)
- Wadler, P. "A prettier printer." 1998/2003. [PDF](https://homepages.inf.ed.ac.uk/wadler/papers/prettier/prettier.pdf)
- Yelland, P. "A New Approach to Optimal Code Formatting." Google, 2016. [Google Research](https://research.google/pubs/pub44667/)
- Chitil, O. "Pretty Printing with Delimited Continuations." Kent, 2005. [PDF](https://kar.kent.ac.uk/14464/1/Pretty_Printing.pdf)
- Bernardy, J.P. "A Pretty But Not Greedy Printer." ICFP, 2017. [PDF](https://jyp.github.io/pdf/Prettiest.pdf)
- Bonichon, R. & Weis, P. "Format Unraveled." [PDF](https://rbonichon.github.io/papers/format-unraveled.pdf)
- Pombrio, J. "A Twist on Wadler's Printer." 2024. [Blog](https://justinpombrio.net/2024/02/23/a-twist-on-Wadlers-printer.html)

### Implementation References

- [Prettier technical details](https://prettier.io/docs/technical-details)
- [Prettier commands.md](https://github.com/prettier/prettier/blob/main/commands.md)
- [Biome formatter implementation](https://deepwiki.com/biomejs/biome/6.2-formatter-implementation)
- [Ruff formatter announcement](https://astral.sh/blog/the-ruff-formatter)
- [gofmt cultural evolution](https://go.dev/talks/2015/gofmt-en.slide)
- [Ormolu DESIGN.md](https://github.com/tweag/ormolu/blob/master/DESIGN.md)
- [dartfmt: The Hardest Program I've Ever Written](https://journal.stuffwithstuff.com/2015/09/08/the-hardest-program-ive-ever-written/)
- [scalafmt thesis](https://geirsson.com/assets/olafur.geirsson-scalafmt-thesis.pdf)
- [RFC 2437: rustfmt stability](https://rust-lang.github.io/rfcs/2437-rustfmt-stability.html)
- [RFC 3338: Rust style evolution](https://rust-lang.github.io/rfcs/3338-style-evolution.html)
- [UW PLSE: The Science of Code Formatting](https://uwplse.org/2024/03/04/code-formatting.html)
- [On pretty printers (William Durand)](https://williamdurand.fr/2021/07/23/on-pretty-printers/)
