# Academic Literature Review: Scripting Language Design

A comprehensive survey of research relevant to designing a new scripting/workflow language, organized by topic area with cross-references between related work.

---

## 1. Foundational Papers on Scripting Languages

### 1.1 Ousterhout's Dichotomy

**John K. Ousterhout. "Scripting: Higher-Level Programming for the 21st Century." IEEE Computer, 31(3):23--30, March 1998.**
- [PDF (Stanford)](https://web.stanford.edu/~ouster/cgi-bin/papers/scripting.pdf)
- [IEEE Xplore](https://ieeexplore.ieee.org/document/660187/)

Key contributions:
- Articulates the fundamental distinction between *system programming languages* (C, Java) designed for building data structures and algorithms from scratch, and *scripting languages* (Tcl, Perl, Python) designed for *gluing* existing components together.
- Argues that scripting languages trade type safety and execution speed for development speed, expressiveness, and a higher level of abstraction.
- Predicts that as component frameworks mature, scripting languages will handle an increasing share of programming work.

Relevance to lx: The Ousterhout thesis directly frames the design space. lx sits at the scripting end but with a twist -- it scripts *agents* rather than system components. The "glue" metaphor applies: lx glues together LLM calls, tool invocations, and subprocess coordination rather than C libraries. However, lx should be cautious about the false dichotomy Ousterhout draws; modern languages like Julia (Section 6.2) show that scripting-level productivity and systems-level performance can coexist.

### 1.2 In Praise of Scripting

**Ronald P. Loui. "In Praise of Scripting: Real Programming Pragmatism." IEEE Computer, 41(7):22--26, July 2008.**
- [IEEE Xplore](https://ieeexplore.ieee.org/document/4563874/)
- [PDF](https://www.cs.mun.ca/~harold/Courses/Old/CS2500.F08/Diary/04563874.pdf)

Key contributions:
- Argues that scripting languages (Perl, Python, JavaScript) have fulfilled their promise and are displacing Java in practice.
- Introduces the concept of *programming language pragmatics* -- evaluating languages not just on syntax and semantics but on what they enable developers to accomplish quickly and practically.
- Claims academics are too focused on theoretical issues and have failed to recognize the ascendance of scripting.

Relevance to lx: Loui's pragmatism lens is essential. For an agent-oriented language, the measure of success is not theoretical elegance but how quickly and reliably an agent can express a workflow. Token efficiency, generation friendliness, and error-recovery-in-context matter more than Turing completeness proofs.

### 1.3 Programming Language Design Principles

**Niklaus Wirth. "On the Design of Programming Languages." In IFIP Congress 1974, pages 386--393. North Holland, 1974.**
- [PDF (MIT)](https://people.csail.mit.edu/feser/pld-s23/Wirth_Design.pdf)

Key contributions:
- Lists competing design goals: ease of learning, self-sufficiency (no need for feature additions), compiler efficiency, compilation speed, and library/system compatibility.
- Argues that language design should enhance reliability by constraining the programmer, applying this principle to specific language constructs.

Relevance to lx: Wirth's principle that a language should be "usable without new features being added" maps to lx's goal of being complete enough for agent workflows without requiring escape hatches to a host language. The reliability-through-constraint principle aligns with lx's approach of providing a small, well-defined set of primitives.

---

## 2. Type Systems for Scripting Languages

### 2.1 Gradual Typing: Foundations

**Jeremy G. Siek and Walid Taha. "Gradual Typing for Functional Languages." In Scheme and Functional Programming Workshop, pages 81--92, 2006.**
- [PDF](http://scheme2006.cs.uchicago.edu/13-siek.pdf)
- [ResearchGate](https://www.researchgate.net/publication/213883236_Gradual_typing_for_functional_languages)

Key contributions:
- Introduces the simply-typed lambda calculus extension lambda-?-arrow with a dynamic type `?` representing unknown types.
- Formalizes how a single program can mix statically-typed and dynamically-typed code, with the programmer controlling which parts are typed by adding or omitting annotations.
- Defines the *consistency* relation on types (replacing naive subtyping) to govern when implicit casts are inserted.

**Jeremy G. Siek and Walid Taha. "Gradual Typing for Objects." In ECOOP 2007, LNCS 4609, pages 2--27. Springer, 2007.**
- [SpringerLink](https://link.springer.com/chapter/10.1007/978-3-540-73589-2_2)

Key contributions:
- Extends gradual typing to object-oriented languages with structural subtyping.

Relevance to lx: Gradual typing is the most natural fit for a language that must support both quick-and-dirty agent scripts (fully dynamic) and hardened production workflows (fully typed). The `?` type concept could map directly to lx's approach for message types and tool signatures.

### 2.2 The Gradual Guarantee

**Jeremy G. Siek, Michael M. Vitousek, Matteo Cimini, Sam Tobin-Hochstadt, and Ronald Garcia. "Refined Criteria for Gradual Typing." In SNAPL 2015, LIPIcs 32, pages 274--293. Dagstuhl, 2015.**
- [PDF (Dagstuhl)](https://drops.dagstuhl.de/storage/00lipics/lipics-vol032-snapl2015/LIPIcs.SNAPL.2015.274/LIPIcs.SNAPL.2015.274.pdf)

Key contributions:
- Defines the *Gradual Guarantee*: if a gradually typed program is well-typed, removing type annotations always produces a program that is still well-typed; if it evaluates to a value, removing annotations produces an equivalent value.
- Surveys existing gradual type systems and critiques those that fail the guarantee.
- Establishes the *Blame-Subtyping Theorem*: casts from a subtype to a supertype are guaranteed not to fail at runtime.

Relevance to lx: The gradual guarantee should be a design requirement for lx's type system. It ensures that adding types to a working lx program never breaks it -- critical for iterative development where agents write initial code untyped and humans harden it later.

### 2.3 What Is Gradual Typing

**Jeremy G. Siek. "What is Gradual Typing." Blog post / position paper, 2014--present.**
- [Web page](https://jsiek.github.io/home/WhatIsGradualTyping.html)

Key contributions:
- Clarifies that gradual typing is *not* just optional type annotations; it requires specific formal properties including the gradual guarantee.
- Distinguishes gradual typing from related approaches like soft typing, optional typing (TypeScript), and pluggable type systems.

### 2.4 Typed Racket and Occurrence Typing

**Sam Tobin-Hochstadt and Matthias Felleisen. "The Design and Implementation of Typed Scheme." In POPL 2008, pages 395--406. ACM, 2008.**
- [PDF](https://www2.ccs.neu.edu/racket/pubs/popl08-thf.pdf)
- [arXiv extended version](https://arxiv.org/abs/1106.2575)

Key contributions:
- Introduces *occurrence typing*: the type system refines variable types based on control flow predicates (e.g., after `(number? x)` succeeds, `x` is known to be a number).
- Presents the first practical gradual type system for an existing dynamically-typed language (Racket/Scheme).
- Combines recursive types, true unions, subtyping, and polymorphism with local inference.

**Sam Tobin-Hochstadt and Matthias Felleisen. "Interlanguage Migration: From Scripts to Programs." In DLS 2006. ACM, 2006.**

Key contributions:
- Frames the practical problem: how to incrementally migrate an untyped codebase to a typed one, module by module.
- Proposes *migratory typing* as the methodology where typed and untyped modules interoperate.

**Sam Tobin-Hochstadt and Matthias Felleisen. "Logical Types for Untyped Languages." In ICFP 2010. ACM, 2010.**

Key contributions:
- Extends occurrence typing with logical reasoning about type predicates.

**Matthias Felleisen, Robert Bruce Findler, Matthew Flatt, Sam Tobin-Hochstadt, et al. "Migratory Typing: Ten Years Later." In SNAPL 2017, LIPIcs. Dagstuhl, 2017.**
- [PDF](https://www2.ccs.neu.edu/racket/pubs/typed-racket.pdf)

Key contributions:
- Reflects on a decade of experience building and maintaining Typed Racket, the first practical migratory typing system.
- Reports on successes (module-level migration works), challenges (performance overhead at typed/untyped boundaries), and open problems.

Relevance to lx: Occurrence typing is highly relevant. lx programs frequently test message types at runtime (`match` on variants, type guards). A type system that refines types through control flow would catch errors without requiring verbose annotations. Migratory typing maps to lx's expected usage pattern: agents write quick untyped scripts, then humans add types for production.

### 2.5 The Performance Debate

**Asumu Takikawa, Daniel Feltey, Ben Greenman, Max S. New, Jan Vitek, and Matthias Felleisen. "Is Sound Gradual Typing Dead?" In POPL 2016, pages 456--468. ACM, 2016.**
- [ACM DL](https://dl.acm.org/doi/10.1145/2837614.2837630)
- [ResearchGate](https://www.researchgate.net/publication/301274144_Is_sound_gradual_typing_dead)

Key contributions:
- Proposes a rigorous methodology for evaluating gradually typed language performance: explore the full space of partial type migrations (typed vs. untyped modules).
- Applies the methodology to Typed Racket on real-world benchmarks.
- Finds devastating overhead: some configurations suffer >100x slowdown due to boundary checks between typed and untyped modules.
- Concludes that sound gradual typing faces serious performance challenges with then-current implementation technology.

**Spenser Bauman, Carl Friedrich Bolz-Tereick, Jeremy Siek, and Sam Tobin-Hochstadt. "Sound Gradual Typing: Only Mostly Dead." In OOPSLA 2017, PACMPL 1(OOPSLA). ACM, 2017.**
- [ACM DL](https://dl.acm.org/doi/10.1145/3133878)
- [ResearchGate](https://www.researchgate.net/publication/320391380_Sound_gradual_typing_only_mostly_dead)

Key contributions:
- Demonstrates that the overhead identified by Takikawa et al. is *not* fundamental -- it can be addressed by better implementation technology.
- Uses Pycket, an experimental tracing JIT compiler for Racket built on RPython/PyPy's meta-tracing framework (see Section 4.2).
- Pycket eliminates >90% of gradual typing overhead on the same benchmarks, using hidden classes and JIT optimization of chaperones/impersonators.

**Fabian Muehlboeck and Ross Tate. "Sound Gradual Typing is Nominally Alive and Well." In OOPSLA 2017, PACMPL 1(OOPSLA). ACM, 2017. (Distinguished Paper Award)**
- [PDF (Cornell)](https://www.cs.cornell.edu/~ross/publications/nomalive/nomalive-oopsla17.pdf)
- [ACM DL](https://dl.acm.org/doi/10.1145/3133880)

Key contributions:
- Proposes a *nominal* approach to gradual typing where runtime type information is attached to objects, enabling O(1) type checks instead of deep structural checks.
- Shows minimal overhead even on adversarial benchmarks from Takikawa et al.
- Argues that designing the type system and implementation together from scratch avoids the performance pitfalls of retrofitting types onto an existing language.

Relevance to lx: This debate is critical for lx's design. Three options emerge:
1. *Unsound* optional types (TypeScript approach) -- easy but gives up safety guarantees.
2. *Sound structural* gradual types with JIT optimization -- powerful but complex to implement.
3. *Sound nominal* gradual types -- simpler runtime checks but requires designing types into the language from the start.
Option 3 is most practical for lx: design nominal message types and agent interfaces from day one.

### 2.6 Abstracting Gradual Typing (AGT)

**Ronald Garcia, Alison M. Clark, and Eric Tanter. "Abstracting Gradual Typing." In POPL 2016, pages 429--442. ACM, 2016.**
- [PDF](https://pleiad.cl/papers/2016/garciaAl-popl2016.pdf)
- [ACM DL](https://dl.acm.org/doi/10.1145/2837614.2837670)

Key contributions:
- Presents a systematic methodology for deriving gradually typed languages from statically typed ones using abstract interpretation (see Section 7.1).
- Languages designed with AGT automatically satisfy formal criteria for gradual typing identified by Siek et al.
- Provides two abstractions: one for static semantics (type checking) and one for dynamic semantics (runtime checks).

**Felipe Banados Schwerter, Alison M. Clark, Khurram A. Jafery, and Ronald Garcia. "Abstracting Gradual Typing Moving Forward: Precise and Space-Efficient." In POPL 2021. ACM, 2021.**
- [ACM DL](https://dl.acm.org/doi/10.1145/3434342)
- [arXiv](https://arxiv.org/abs/2010.14094)

Key contributions:
- Addresses space efficiency problems in AGT-derived languages (coercion accumulation).

Relevance to lx: AGT provides a principled way to derive lx's gradual type system. Rather than ad-hoc design, one could start with a fully static type system for lx's core constructs and systematically "gradualize" it using AGT.

### 2.7 Deep vs. Shallow Gradual Types

**Ben Greenman. "Deep and Shallow Types for Gradual Languages." In PLDI 2022, pages 580--593. ACM, 2022.**
- [PDF](https://users.cs.utah.edu/~blg/publications/apples-to-apples/g-pldi-2022.pdf)
- [ACM DL](https://dl.acm.org/doi/10.1145/3519939.3523430)

Key contributions:
- Identifies two extremes: *deep types* (compositional guarantees via higher-order contracts, expensive) and *shallow types* (first-order checks only, cheap).
- Proposes a language design supporting *both* deep and shallow types simultaneously, letting programmers choose the tradeoff per module.
- Deep types use the "Natural" semantics (wrapper contracts at boundaries); shallow types use the "Transient" semantics (inline checks).

**Ben Greenman, Christos Dimoulas, and Matthias Felleisen. "Complete Monitors for Gradual Types." In OOPSLA 2019, PACMPL 3(OOPSLA). ACM, 2019.**
- [ACM DL](https://dl.acm.org/doi/10.1145/3360548)

Key contributions:
- Defines *complete monitoring*: a property ensuring that the runtime system fully enforces all type annotations.
- Provides a framework for assessing blame quality in systems that lack complete monitoring.

**Bader, Aldrich, Dimoulas, and Greenman. "Gradually Typed Languages Should Be Vigilant!" In OOPSLA 2024. ACM, 2024.**
- [PDF](https://users.cs.northwestern.edu/~chrdimo/pubs/oopsla24-gmda.pdf)

Key contributions:
- Introduces *vigilance* as a property subsuming type soundness and complete monitoring.
- Shows that Transient semantics (shallow types) can be vigilant for a tag type system, while Natural semantics (deep types) is not always vigilant.

Relevance to lx: The deep/shallow distinction maps to lx's needs. For development/debugging, shallow (transient) checks catch obvious type errors with minimal overhead. For production workflows handling sensitive data, deep (natural) enforcement provides full guarantees. Offering both modes is feasible.

---

## 3. Language Design for Toolability

### 3.1 Concrete Syntax Trees and Red-Green Trees

**Eric Lippert. "Persistence, Facades, and Roslyn's Red-Green Trees." Microsoft DevBlogs, June 2012.**
- [Microsoft Learn](https://learn.microsoft.com/en-us/archive/blogs/ericlippert/persistence-facades-and-roslyns-red-green-trees)

Key contributions:
- Describes the *red-green tree* design used in Microsoft's Roslyn C# compiler.
- The "green tree" is immutable, persistent, built bottom-up, with no parent references; nodes track width but not absolute position.
- The "red tree" is an immutable facade built top-down on demand, computing parent references as you descend.
- On edit, only O(log n) of green tree nodes need rebuilding; the rest are reused.
- This design enables incremental parsing, IDE features (completion, refactoring), and efficient memory usage.

Relevance to lx: A red-green tree or similar CST-preserving design is essential if lx wants first-class formatter and refactoring support. Since lx programs will be generated and modified by agents, round-trip fidelity (parse -> transform -> emit identical formatting) is critical.

### 3.2 Tree-sitter and Incremental Parsing

**Max Brunsfeld. "Tree-sitter -- A New Parsing System for Programming Tools." Strange Loop 2018.**
- [GitHub](https://github.com/tree-sitter/tree-sitter)

**Tim A. Wagner. "Practical Algorithms for Incremental Software Development Environments." PhD thesis, University of California, Berkeley, EECS Department, March 1998.**
- [PDF](https://www2.eecs.berkeley.edu/Pubs/TechRpts/1997/CSD-97-946.pdf)

Key contributions (Wagner):
- Describes algorithms for incremental parsing, incremental semantic analysis, and version management of structured documents.
- Proposes a self-versioning representation supporting unrestricted user editing with persistent, structured documents.
- This thesis is cited as a major influence on tree-sitter's design.

Key contributions (tree-sitter):
- Implements incremental parsing: on edit, marks affected nodes in the old tree, then reparses reusing unaffected nodes.
- Produces a *concrete syntax tree* (preserving all tokens including whitespace, comments, delimiters).
- Supports error recovery: produces a valid tree even for syntactically invalid input.
- Designed for real-time use in text editors; sub-millisecond reparse times.

Relevance to lx: lx should target tree-sitter grammar compatibility to get instant editor support. The CST approach means lx's parser preserves all syntactic information, enabling agents to perform precise code transformations without losing formatting.

### 3.3 Language Server Protocol

**Language Server Protocol Specification. Microsoft, 2016--present.**
- [Official site](https://microsoft.github.io/language-server-protocol/)

**Stefan Marr, Humphrey Burchell, and Fabio Niephaus. "Execution vs. Parse-Based Language Servers: Tradeoffs and Opportunities for Language-Agnostic Tooling for Dynamic Languages." In DLS 2022, pages 1--14. ACM, 2022.**
- [Author's page](https://stefan-marr.de/papers/dls-marr-et-al-execution-vs-parse-based-language-servers/)
- [PDF](https://stefan-marr.de/downloads/dls22-marr-et-al-execution-vs-parse-based-language-servers.pdf)

Key contributions:
- Compares two approaches to language servers for dynamic languages:
  - *Execution-based* (Truffle/GraalVM): runs the code to capture type information and call targets. Precise but requires running the program.
  - *Parse-based*: extracts structural information from the syntax tree without execution. Less precise for dynamic features but works on incomplete/broken code.
- The parse-based approach requires <1,000 lines of code per language for typical IDE functionality.
- Argues that for dynamic languages, the parse-based approach is more practical for IDE tooling because it works on incomplete programs.

**Barros et al. "The Specification Language Server Protocol: A Proposal for Standardised LSP Extensions." arXiv:2108.02961, 2021.**
- [arXiv](https://arxiv.org/abs/2108.02961)

Key contributions:
- Identifies limitations of LSP for non-programming languages (specification languages, DSLs).
- Proposes standardized extensions rather than ad-hoc per-language extensions.

Relevance to lx: LSP support is non-negotiable for a modern language. The Marr et al. paper suggests that a parse-based language server (built on lx's CST) is the right first step, with execution-based features added later. For a workflow DSL like lx, the parse-based approach may actually be sufficient since workflows have more static structure than general-purpose dynamic code.

### 3.4 Error Recovery in Parsers

**Lukas Diekmann and Laurence Tratt. "Don't Panic! Better, Fewer, Syntax Errors for LR Parsers." In ECOOP 2020, LIPIcs 166, pages 6:1--6:32. Dagstuhl, 2020.**
- [PDF (Dagstuhl)](https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.ECOOP.2020.6)
- [arXiv](https://arxiv.org/abs/1804.07133)

Key contributions:
- Introduces the CPCT+ algorithm for LR parser error recovery.
- Reports the *complete set* of minimum-cost repair sequences for a given error location, letting the user choose the intended fix.
- On 200,000 real-world syntactically invalid Java programs: repairs 98.37% of files within 0.5s timeout.
- Reduces cascading errors from 981,628 (panic mode) to 435,812 error locations.
- Establishes that advanced error recovery *can* be practical, contradicting the common belief that it is too slow.

**Sergio Medeiros and Fabio Mascarenhas. "Syntax Error Recovery in Parsing Expression Grammars." In SAC 2018. ACM, 2018.**
- [arXiv](https://arxiv.org/abs/1806.11150)

Key contributions:
- Extends PEG parsers with *labeled failures* (inspired by exception handling) for error recovery.
- Associates recovery expressions with labels to reach synchronization points in the input.
- Addresses the limitation that standard PEG parsers fail completely on the first syntax error, making them unsuitable for IDE use.

**Sergio Medeiros, Fabio Mascarenhas, and Roberto Ierusalimschy. "Error Recovery in Parsing Expression Grammars through Labeled Failures." In Journal of Computer Languages (COLA), 2018.**
- [ScienceDirect](https://www.sciencedirect.com/science/article/abs/pii/S1045926X18301897)

Key contributions:
- Formalizes labeled failures as a PEG extension.
- Demonstrates practical error recovery comparable to hand-written recovery in parser generators like ANTLR.

**Zachary Yedidia. "Incremental PEG Parsing." Harvard University Senior Thesis, 2017.**
- [PDF](https://zyedidia.github.io/notes/yedidia_thesis.pdf)

Key contributions:
- Combines PEG parsing with incremental reparsing techniques.

Relevance to lx: Error recovery is critical for lx because agents will frequently generate syntactically incomplete programs. The parser must produce a meaningful partial parse tree, not just fail. The CPCT+ approach (for LR grammars) or labeled failures (for PEG grammars) both offer practical solutions. The choice depends on lx's grammar class.

---

## 4. Performance of Scripting Languages

### 4.1 JIT Compilation Survey

**John Aycock. "A Brief History of Just-in-Time." ACM Computing Surveys, 35(2):97--113, June 2003.**
- [ACM DL](https://dl.acm.org/doi/10.1145/857076.857077)
- [PDF](https://www.cs.tufts.edu/comp/150IPL/papers/aycock03jit.pdf)

Key contributions:
- Surveys 40 years of JIT compilation (1960--2000).
- Defines JIT as "any translation performed dynamically after a program has started execution."
- Classifies JIT systems along multiple axes: granularity (method vs. trace vs. region), trigger (counter-based vs. hot-path detection), and optimization level.
- Traces the evolution from McCarthy's LISP compile function (1960) through Self (1980s) to Java HotSpot (1990s).

### 4.2 Tracing JIT: TraceMonkey

**Andreas Gal, Brendan Eich, Mike Shaver, David Anderson, David Mandelin, Mohammad R. Haghighat, Blake Kaplan, Graydon Hoare, Boris Zbarsky, Jason Orendorff, Jesse Ruderman, Edwin W. Smith, Rick Reitmaier, Michael Bebenita, Mason Chang, and Michael Franz. "Trace-based Just-in-Time Type Specialization for Dynamic Languages." In PLDI 2009, pages 465--478. ACM, 2009.**
- [ACM DL](https://dl.acm.org/doi/10.1145/1542476.1542528)
- [PDF](https://www.cs.williams.edu/~freund/cs434/gal-trace.pdf)

Key contributions:
- Presents TraceMonkey, Firefox's tracing JIT for JavaScript.
- Identifies frequently executed loop traces at runtime and generates machine code *specialized for the actual dynamic types* occurring on each path.
- Traces cross function boundaries, enabling interprocedural optimization without whole-program analysis.
- Demonstrates 2--20x speedups on the SunSpider benchmark suite compared to interpretation.

Key insight: Tracing JITs excel at loops with predictable type patterns -- exactly the execution pattern of workflow loops in lx (iterate over messages, dispatch by type, invoke tools).

### 4.3 Meta-Tracing: PyPy/RPython

**Carl Friedrich Bolz, Antonio Cuni, Maciej Fijalkowski, and Armin Rigo. "Tracing the Meta-Level: PyPy's Tracing JIT Compiler." In ICOOOLPS 2009. ACM, 2009.**
- [ACM DL](https://dl.acm.org/doi/10.1145/1565824.1565827)
- [PDF](https://dom.parnaiba.pi.gov.br/assets/diarios-anteriores/bolz-tracing-jit-final.pdf)

Key contributions:
- Introduces *meta-tracing*: instead of tracing the user's program directly, trace the *interpreter* executing the program.
- The interpreter author provides two kinds of hints to guide the tracer: (1) identify the bytecode dispatch loop, and (2) mark operations that should be unrolled/inlined.
- Evaluated on both a small toy language and the full Python interpreter.
- Key result: unmodified tracing of a bytecode interpreter yields limited speedup; meta-tracing with hints yields dramatic improvement.

**Carl Friedrich Bolz and Sam Tobin-Hochstadt. "Meta-tracing Makes a Fast Racket." In DLS workshop, 2014.**
- [PDF](https://homes.luddy.indiana.edu/samth/pycket-dyla.pdf)

Key contributions:
- Applies meta-tracing (RPython framework) to build Pycket, a fast Racket implementation.
- Achieves performance competitive with Racket's existing JIT while supporting gradual typing checks.

Relevance to lx: Meta-tracing is the most practical path to high performance for a new language. Rather than building a JIT from scratch, implement lx's interpreter in RPython (or a similar framework) and get JIT compilation automatically. This is exactly how Pycket achieved the gradual typing performance improvements described in Section 2.5.

### 4.4 Graal/Truffle: Self-Optimizing AST Interpreters

**Thomas Wurthinger, Andreas Woss, Lukas Stadler, Gilles Duboscq, Doug Simon, and Christian Wimmer. "Self-Optimizing AST Interpreters." In DLS 2012. ACM, 2012.**
- [ACM DL](https://dl.acm.org/doi/10.1145/2384716.2384723)

Key contributions:
- Presents Truffle, a framework where language implementers write an AST interpreter in Java.
- The AST *rewrites itself* during interpretation: generic nodes specialize to type-specific nodes (e.g., a generic `Add` node rewrites to `IntAdd` when both operands are integers).
- Combined with the Graal JIT compiler, specialized AST nodes compile to efficient machine code.

**Christian Humer, Christian Wimmer, Christian Wirth, Andreas Woss, and Thomas Wurthinger. "A Domain-Specific Language for Building Self-Optimizing AST Interpreters." In GPCE 2014. ACM, 2014.**

Key contributions:
- Introduces the Truffle DSL: annotations that generate specialization boilerplate automatically.
- Reduces the effort of writing a high-performance language implementation from months to days.

Relevance to lx: Truffle/Graal is the most realistic performance path for lx. Write lx's interpreter as a Truffle language, and it gets JIT compilation, debugging, profiling, and polyglot interoperability for free. The self-optimizing AST approach is particularly well-suited to lx's pattern matching and message dispatch patterns.

### 4.5 Inline Caching and Dynamic Dispatch

**L. Peter Deutsch and Allan M. Schiffman. "Efficient Implementation of the Smalltalk-80 System." In POPL 1984, pages 297--302. ACM, 1984.**
- [ACM DL](https://dl.acm.org/doi/10.1145/800017.800542)

Key contributions:
- Introduces *inline caching*: at each call site, cache the previously looked-up method. On subsequent calls, check if the receiver's class matches the cache; if so, call directly without lookup.
- First practical implementation of dynamic compilation for a production language.

**Urs Holzle, Craig Chambers, and David Ungar. "Optimizing Dynamically-Typed Object-Oriented Languages with Polymorphic Inline Caches." In ECOOP 1991, LNCS 512, pages 21--38. Springer, 1991.**
- [SpringerLink](https://link.springer.com/chapter/10.1007/BFb0057013)
- [PDF](https://bibliography.selflanguage.org/_static/pics.pdf)

Key contributions:
- Introduces *polymorphic inline caches* (PICs): extend inline caches to store multiple (class, method) pairs per call site.
- Achieves median 11% speedup for SELF programs.
- As a side effect, PICs *collect type information* by recording all receiver types at each call site. The compiler exploits this for type-specialized recompilation, achieving 27% median speedup.

**Jiho Choi, Thomas Shull, and Josep Torrellas. "Reusable Inline Caching for JavaScript Performance." In PLDI 2019, pages 889--901. ACM, 2019.**
- [PDF](https://iacoma.cs.uiuc.edu/iacoma-papers/pldi19_2.pdf)
- [ACM DL](https://dl.acm.org/doi/10.1145/3314221.3314587)

Key contributions:
- Addresses *startup time*: inline caches are cold on each program start, requiring re-learning.
- Proposes persisting and reusing IC information across executions.

**Stefan Brunthaler. "Inline Caching Meets Quickening." In ECOOP 2010, LNCS 6183, pages 429--451. Springer, 2010.**
- [SpringerLink](https://link.springer.com/chapter/10.1007/978-3-642-14107-2_21)
- [PDF](https://publications.sba-research.org/publications/ecoop10.pdf)

Key contributions:
- Demonstrates inline caching *without* JIT compilation, purely in an interpreter via bytecode quickening (rewriting bytecodes to specialized versions).
- Achieves up to 1.71x speedup over standard interpretation.

Relevance to lx: Even without a JIT, lx's interpreter can benefit from inline caching for message dispatch and tool invocation. Brunthaler's interpreter-only approach is especially relevant for an early lx implementation. PICs are relevant because lx's `match` expressions are essentially polymorphic dispatch sites.

---

## 5. DSL and Workflow Language Design

### 5.1 Domain-Specific Languages: Theory

**Paul Hudak. "Building Domain-Specific Embedded Languages." ACM Computing Surveys, 28(4es):196, December 1996.**
- [ACM DL](https://dl.acm.org/doi/10.1145/242224.242477)

Key contributions:
- Coins the term *domain-specific embedded language* (DSEL).
- Argues for embedding DSLs in a host language (specifically Haskell) rather than building standalone parsers/interpreters.
- Advantages of embedding: reuse the host compiler, leverage the host type system, seamless interop with host libraries.
- Limitations: constrained to the host's syntax and error messages.

**Marjan Mernik, Jan Heering, and Anthony M. Sloane. "When and How to Develop Domain-Specific Languages." ACM Computing Surveys, 37(4):316--344, December 2005.**
- [PDF](https://inkytonik.github.io/assets/papers/compsurv05.pdf)

Key contributions:
- Provides a systematic decision framework: *when* to create a DSL (vs. using a GPL or library).
- Catalogs DSL implementation approaches: interpreter, compiler, preprocessor, embedding, compiler generator.
- Identifies *domain analysis* as the critical first step: understanding the domain's concepts, operations, and constraints before designing syntax.

**Martin Fowler. *Domain-Specific Languages*. Addison-Wesley, 2010. ISBN 978-0-321-71294-3.**
- [Publisher page](https://martinfowler.com/books/dsl.html)
- [Pattern catalog](https://martinfowler.com/dslCatalog/)

Key contributions:
- Distinguishes *internal DSLs* (fluent APIs in a host language) from *external DSLs* (standalone languages with their own parser).
- Catalogs dozens of patterns for DSL design: Expression Builder, Method Chaining, Nested Function, Semantic Model, etc.
- Pragmatic treatment accessible to practitioners.

Relevance to lx: lx is an *external* DSL (standalone language with its own parser and interpreter), but it fills the role of an *embedded* DSL -- it is embedded in the agent's workflow, not in a host programming language. Mernik et al.'s domain analysis framework is directly applicable: the "domain" is agent orchestration, and the "operations" are message passing, tool invocation, and subprocess coordination.

### 5.2 Workflow Patterns

**Wil M.P. van der Aalst, Arthur H.M. ter Hofstede, Bartek Kiepuszewski, and Alistair P. Barros. "Workflow Patterns." Distributed and Parallel Databases, 14(1):5--51, July 2003.**
- [SpringerLink](https://link.springer.com/article/10.1023/A:1022883727209)
- [Workflow Patterns site](http://www.workflowpatterns.com/)

Key contributions:
- Defines 20 fundamental *control-flow patterns* for workflow systems: Sequence, Parallel Split, Synchronization, Exclusive Choice, Simple Merge, Multi-Choice, Structured Synchronizing Merge, Multi-Merge, Structured Discriminator, Arbitrary Cycles, Implicit Termination, etc.
- Uses these patterns to systematically evaluate commercial workflow systems and proposed standards (BPEL, BPMN, UML Activity Diagrams, XPDL).
- Extended with additional pattern sets: *Data Patterns*, *Resource Patterns*, *Exception Handling Patterns*.

Relevance to lx: The workflow patterns provide a checklist for lx's control-flow primitives. lx should be able to express all 20 basic patterns. Currently, lx supports Sequence, Parallel Split (spawn), Exclusive Choice (match), and Arbitrary Cycles (loops). Missing or implicit: Synchronization (join), Multi-Choice, Discriminator, Cancellation patterns. The Exception Handling Patterns are especially relevant for agent error recovery.

### 5.3 Dataflow Programming

**Jack B. Dennis. "First Version of a Data Flow Procedure Language." In Symposium on Programming, LNCS. Springer, 1974.**
- [SpringerLink](https://link.springer.com/chapter/10.1007/3-540-06859-7_145)

Key contributions:
- Introduces the first formal dataflow programming language.
- Programs are directed graphs where nodes are operations and edges carry data tokens.
- An operation fires as soon as all its input tokens are available -- inherently parallel.
- No concept of sequential execution or shared mutable state.

Relevance to lx: lx's pipe operator (`|>`) and message-passing model are essentially dataflow. Agent workflows are naturally expressed as dataflow graphs: data flows from one agent/tool to the next, with operations firing when inputs are ready. Understanding this heritage helps lx's design stay true to the dataflow model rather than accidentally introducing sequential bottlenecks.

---

## 6. Modern Scripting Language Research

### 6.1 Raku (Perl 6)

While Raku/Perl 6 lacks a single defining academic paper (it was designed through community RFCs and Larry Wall's "Apocalypse" documents), its design innovations are documented in:

**Larry Wall. "Apocalypse" series (2001--2009) and "Synopsis" series.**

Key design innovations relevant to lx:
- *Gradual typing* with a rich type system supporting both static and dynamic typing, including subset types, role composition, and type constraints.
- *Grammars as first-class language feature*: Raku's `grammar` construct provides PEG-like parsing with backtracking, named captures, and inheritance -- making the language self-parsing.
- *Junctions*: superpositions of values enabling concurrent evaluation and speculative execution.
- *Multiple dispatch* (multimethods) as the default dispatch mechanism.

Relevance to lx: Raku's grammar system is a template for how lx could allow users to define custom syntax extensions for domain-specific workflow patterns. Raku's gradual typing demonstrates that a practical, maximalist approach to gradual typing is possible.

### 6.2 Julia

**Jeff Bezanson, Alan Edelman, Stefan Karpinski, and Viral B. Shah. "Julia: A Fresh Approach to Numerical Computing." SIAM Review, 59(1):65--98, 2017.**
- [SIAM Review](https://epubs.siam.org/doi/10.1137/141000671)
- [arXiv](https://arxiv.org/abs/1411.1607)

Key contributions:
- Demonstrates that a dynamic language can achieve C-like performance through *type specialization* and *multiple dispatch*.
- Challenges three "laws" of numerical computing: (1) high-level must be slow, (2) you must prototype in one language and rewrite in another, (3) experts must build the fast parts.
- Design centered on multiple dispatch as the organizing principle for code selection.

**Jeff Bezanson, Jiahao Chen, Benjamin Chung, Stefan Karpinski, Viral B. Shah, Jan Vitek, and Lionel Zoubritzky. "Julia: Dynamism and Performance Reconciled by Design." In OOPSLA 2018, PACMPL 2(OOPSLA). ACM, 2018.**
- [ACM DL](https://dl.acm.org/doi/10.1145/3276490)
- [PDF](https://janvitek.org/pubs/oopsla18b.pdf)

Key contributions:
- Formalizes Julia's design: dynamic typing, automatic memory management, rich type annotations, and multiple dispatch.
- Shows how Julia's *specializing JIT compiler* eliminates the overhead of dynamic features when types are predictable.
- Introduces the concept of "world age" for managing method redefinition in a JIT-compiled language.

**Francesco Zappa Nardelli, Julia Belyakova, Artem Pelenitsyn, Benjamin Chung, Jeff Bezanson, and Jan Vitek. "Julia Subtyping: A Rational Reconstruction." In OOPSLA 2018, PACMPL 2(OOPSLA). ACM, 2018.**
- [ACM DL](https://dl.acm.org/doi/10.1145/3276490)
- [INRIA HAL](https://inria.hal.science/hal-01882137v1)

Key contributions:
- First formal definition of Julia's subtype relation, which is undecidable in the general case.
- Validated empirically: their implementation matched Julia's on 6,014,354 of 6,014,476 tests (122 differences were Julia bugs).

**Jeff Bezanson. "Abstraction in Technical Computing." PhD thesis, MIT, 2015.**
- [MIT DSpace](https://dspace.mit.edu/handle/1721.1/99811)

Key contributions:
- Argues that multiple dispatch + data-flow type inference provides the right abstraction mechanism for technical computing.
- Introduces Julia's approach of integrating code selection with code specialization.

Relevance to lx: Julia demonstrates that a dynamic language with the *right* design can achieve performance without sacrificing expressiveness. Key takeaways for lx:
- Multiple dispatch is more powerful than single dispatch for selecting behavior based on message types.
- Type specialization by the compiler can eliminate dynamic overhead when types are locally predictable (which they usually are in workflow steps).
- The "world age" concept may be relevant for lx's hot-reloading of workflow definitions.

### 6.3 Elixir and the BEAM VM

**Jose Valim. "Elixir Design Goals." Blog post, August 2013.**
- [Elixir blog](http://elixir-lang.org/blog/2013/08/08/elixir-design-goals/)

Key design goals documented:
- *Compatibility*: run on the BEAM VM, interoperate with Erlang/OTP.
- *Productivity*: Ruby-inspired syntax, Mix build tool, doctests, ExUnit.
- *Extensibility*: macros for metaprogramming, protocols for polymorphism.

**Joe Armstrong. "Making Reliable Distributed Systems in the Presence of Software Errors." PhD thesis, Royal Institute of Technology, Stockholm, 2003.**
- [PDF (Erlang.org)](https://erlang.org/download/armstrong_thesis_2003.pdf)

Key contributions:
- Describes the design philosophy behind Erlang and OTP: lightweight processes, message passing, "let it crash" supervision, and hot code swapping.
- Documents the AXD301, a major Ericsson telecom product with >1M lines of Erlang code, described as "one of the most reliable products ever made by Ericsson."
- Formalizes Erlang's six rules for fault-tolerant systems: isolation, concurrency, failure detection, fault identification, live code upgrade, and stable storage.

**NVLang: Unified Static Typing for Actor-Based Concurrency on the BEAM. arXiv:2512.05224, December 2025.**
- [arXiv](https://arxiv.org/abs/2512.05224)

Key contributions:
- Presents a statically typed functional language for the BEAM VM.
- Uses algebraic data types (ADTs) to naturally encode actor message protocols: each actor declares the sum type representing its message vocabulary.
- The type system enforces protocol conformance at compile time.

Relevance to lx: Erlang/Elixir's actor model is the closest established paradigm to lx's agent model. Key parallels:
- lx agents ~ Erlang processes (lightweight, isolated, message-passing).
- lx `spawn` ~ Erlang `spawn/3`.
- lx message matching ~ Erlang `receive` patterns.
Key differences: lx agents are LLM-backed (nondeterministic, expensive), so lx needs stronger guarantees about message protocol conformance. NVLang's approach of typing message protocols with ADTs is directly applicable.

---

## 7. Static Analysis for Dynamic Languages

### 7.1 Abstract Interpretation: Foundations

**Patrick Cousot and Radhia Cousot. "Abstract Interpretation: A Unified Lattice Model for Static Analysis of Programs by Construction or Approximation of Fixpoints." In POPL 1977, pages 238--252. ACM, 1977.**
- [ACM DL](https://dl.acm.org/doi/10.1145/512950.512973)
- [Author's page](https://www.di.ens.fr/~cousot/COUSOTpapers/POPL77.shtml)

Key contributions:
- Introduces abstract interpretation: computing in an "abstract" domain that approximates the concrete semantics.
- Shows that program properties obtained by abstract interpretation are *sound* (consistent with the concrete semantics).
- Formalizes the framework using lattice theory and fixpoint computation.
- Foundational work for all subsequent static analysis research.

Relevance to lx: Abstract interpretation provides the theoretical foundation for any static analysis of lx programs. Even simple analyses (unreachable code, unused variables, dead message patterns) can be formulated as abstract interpretations. The AGT framework (Section 2.6) explicitly uses abstract interpretation to derive gradual type systems.

### 7.2 Type Analysis for JavaScript (TAJS)

**Simon Holm Jensen, Anders Moller, and Peter Thiemann. "Type Analysis for JavaScript." In SAS 2009, LNCS 5673, pages 238--255. Springer, 2009.**
- [Author's page](https://cs.au.dk/~amoeller/papers/tajs/)
- [PDF](https://cs.au.dk/~amoeller/papers/tajs/paper.pdf)

Key contributions:
- Presents TAJS, a static analysis infrastructure for JavaScript using abstract interpretation.
- Infers *sound* type information for the full ECMAScript 3 language, including the peculiar object model and all built-in functions.
- Uses dataflow analysis with monotone frameworks: constructs control flow graphs, defines abstract domains (lattices over types), and computes fixpoints.
- Results can prove absence of common errors (type errors, null dereferences, unreachable code).

**Anders Moller. "Static Analysis for JavaScript -- Challenges and Techniques." SAS 2015, invited talk.**
- [PDF](http://sas2015.inria.fr/Moller.pdf)

Key contributions:
- Surveys the challenges of analyzing JavaScript: eval, dynamic property access, prototype chains, implicit coercions.
- Describes how TAJS evolved to handle increasingly complex JavaScript patterns.

Relevance to lx: TAJS demonstrates that sound static analysis of a dynamic language is *possible* but requires modeling the language's semantics in detail. For lx, a TAJS-style analysis could infer message types flowing through pipelines, detect unreachable match arms, and verify that agents send well-formed messages -- all without running the program.

### 7.3 Flow: Type Checking at Scale

**Avik Chaudhuri, Panagiotis Vekris, Sam Goldman, Marshall Roch, and Gabriel Levi. "Fast and Precise Type Checking for JavaScript." In OOPSLA 2017, PACMPL 1(OOPSLA). ACM, 2017.**
- [ACM DL](https://dl.acm.org/doi/abs/10.1145/3133872)

Key contributions:
- Describes Flow, Facebook's type checker for JavaScript used by thousands of developers.
- Uses sophisticated type inference to understand common JavaScript idioms without requiring annotations.
- Finds non-trivial bugs without significant code rewriting.
- Designed for *scale*: incremental checking, modular analysis, fast response times.

Relevance to lx: Flow demonstrates the pragmatic approach to typing dynamic languages: don't aim for full soundness, aim for catching real bugs with minimal annotation burden. This pragmatic stance may be appropriate for lx's early type system, before investing in full gradual typing.

---

## 8. Actor Model and Concurrency

### 8.1 Original Actor Model

**Carl Hewitt, Peter Bishop, and Richard Steiger. "A Universal Modular ACTOR Formalism for Artificial Intelligence." In IJCAI 1973, pages 235--245. Morgan Kaufmann, 1973.**
- [PDF (IJCAI)](https://www.ijcai.org/Proceedings/73/Papers/027B.pdf)
- [ACM DL](https://dl.acm.org/doi/10.5555/1624775.1624804)

Key contributions:
- Proposes that all computation can be modeled as *actors* that communicate by sending messages.
- Each actor can: (1) send messages to other actors, (2) create new actors, (3) designate the behavior to be used for the next message it receives.
- No shared state, no locks -- concurrency is fundamental, not bolted on.

**Gul Agha. *Actors: A Model of Concurrent Computation in Distributed Systems*. MIT Press, 1986. ISBN 0-262-01092-5.**
- [MIT Press](https://direct.mit.edu/books/monograph/4794/ActorsA-Model-of-Concurrent-Computation-in)

Key contributions:
- Produces both a syntactic definition and a denotational model of the actor paradigm.
- Addresses fairness, nondeterminism, and compositionality in actor systems.
- Formalizes the notion that concurrency in actors is constrained only by hardware resources and logical dependencies.

Relevance to lx: lx's agent model is essentially the actor model with LLM-backed "behavior functions." The three actor primitives (send, create, become) map directly to lx's message passing, agent spawning, and state transitions. Understanding the actor model's formal properties (fairness guarantees, compositionality) helps ensure lx's semantics are well-defined.

---

## 9. Languages for AI Agent Systems

### 9.1 Pel: A Language for Agent Orchestration

**Behnam Mohammadi. "Pel, A Programming Language for Orchestrating AI Agents." arXiv:2505.13453, June 2025.**
- [arXiv](https://arxiv.org/abs/2505.13453)
- [PDF](https://arxiv.org/pdf/2505.13453)

Key contributions:
- Presents Pel, a novel programming language specifically for LLM orchestration, inspired by Lisp, Elixir, Gleam, and Haskell.
- Homoiconic design: programs are data, enabling LLMs to generate and manipulate code as structured data.
- Key features: piping mechanism for linear composition, first-class closures, built-in natural language conditions evaluated by LLMs, Common Lisp-style restarts with LLM-powered error correction.
- Automatic parallelization via static dependency analysis.
- Minimal grammar designed for constrained LLM generation -- capability control at the syntax level eliminates the need for sandboxing.

### 9.2 Declarative Agent Workflow Languages

**Anonymous (PayPal). "A Declarative Language for Building and Orchestrating LLM-Powered Agent Workflows." arXiv:2512.19769, December 2025.**
- [arXiv](https://arxiv.org/abs/2512.19769)

Key contributions:
- Separates workflow *specification* from *implementation*, enabling the same pipeline to execute across Java, Python, and Go backends.
- Key insight: most agent workflows consist of common patterns (data serialization, filtering, RAG retrieval, API orchestration) expressible through a unified DSL.
- Supports A/B testing of agent strategies with automatic metric collection.
- Non-engineers can modify agent behaviors safely through the declarative interface.
- Evaluated at PayPal: 60% reduction in development time, 3x improvement in deployment velocity; complex workflows expressed in <50 lines of DSL vs. 500+ lines of imperative code.

Relevance to lx: These are lx's closest competitors/peers. Key observations:
- Pel validates lx's core design choice of a language designed for agents, but takes a Lisp-inspired approach (homoiconic) vs. lx's custom syntax.
- The PayPal DSL validates the declarative approach but is enterprise-focused (multi-backend deployment). lx targets a different niche: agent-to-agent workflows where the "programmer" is also an agent.
- Both papers confirm that existing frameworks (LangChain, AutoGen) are insufficient for complex agent orchestration, validating the need for a purpose-built language.

---

## Cross-Reference Map

The following connections between research areas are especially important for lx's design:

| Connection | Papers | Implication for lx |
|---|---|---|
| Gradual typing + JIT optimization | Takikawa 2016, Bauman 2017, Muehlboeck 2017 | Sound gradual typing is viable if the implementation is designed for it from the start |
| Meta-tracing + gradual typing | Bolz 2009, Bolz/Tobin-Hochstadt 2014 | PyPy/RPython framework can eliminate gradual typing overhead automatically |
| Abstract interpretation + gradual typing | Cousot 1977, Garcia 2016 (AGT) | AGT systematically derives gradual types using abstract interpretation theory |
| Actor model + typed protocols | Hewitt 1973, Agha 1986, Armstrong 2003, NVLang 2025 | Type message protocols with ADTs; enforce conformance at compile time |
| Workflow patterns + DSL design | van der Aalst 2003, Mernik 2005, Hudak 1996 | Use workflow patterns as requirements checklist for lx's control-flow primitives |
| Error recovery + IDE tooling | Diekmann 2020, Medeiros 2018, Marr 2022 | Parser error recovery is prerequisite for language server; CPCT+ or labeled failures are practical |
| CST + incremental parsing | Lippert 2012, Wagner 1998, tree-sitter | Red-green trees enable both IDE features and round-trip code generation by agents |
| Inline caching + interpreter optimization | Deutsch 1984, Holzle 1991, Brunthaler 2010 | Even without JIT, interpreter-level inline caching speeds up message dispatch |
| Dynamic language static analysis + type inference | Jensen 2009 (TAJS), Chaudhuri 2017 (Flow) | Sound analysis of lx programs is feasible; pragmatic approach (Flow) may be appropriate initially |
| Agent orchestration languages | Mohammadi 2025 (Pel), PayPal 2025 | Validates lx's niche; homoiconicity and declarative specification are alternative design points |

---

## Recommended Reading Order for lx Designers

1. **Start with the problem space**: Ousterhout 1998, Loui 2008, van der Aalst 2003
2. **Understand the actor/agent model**: Hewitt 1973, Armstrong 2003, Mohammadi 2025
3. **Design the type system**: Siek & Taha 2006, Siek et al. 2015, Tobin-Hochstadt & Felleisen 2008, Muehlboeck & Tate 2017
4. **Plan for toolability**: Lippert 2012 (red-green trees), Diekmann & Tratt 2020 (error recovery), Marr et al. 2022 (language servers)
5. **Plan for performance**: Holzle et al. 1991 (PICs), Brunthaler 2010 (quickening), Bolz et al. 2009 (meta-tracing), Wurthinger et al. 2012 (Truffle)
6. **Study peer languages**: Julia (Bezanson et al. 2017, 2018), Elixir/Erlang (Armstrong 2003), Raku (Wall Apocalypses)

---

## Bibliography (Alphabetical)

- Agha, G. *Actors: A Model of Concurrent Computation in Distributed Systems*. MIT Press, 1986.
- Armstrong, J. "Making reliable distributed systems in the presence of software errors." PhD thesis, KTH Stockholm, 2003.
- Aycock, J. "A brief history of just-in-time." ACM Computing Surveys, 35(2):97--113, 2003.
- Banados Schwerter, F., Clark, A.M., Jafery, K.A., Garcia, R. "Abstracting Gradual Typing Moving Forward: Precise and Space-Efficient." POPL 2021.
- Bauman, S., Bolz-Tereick, C.F., Siek, J., Tobin-Hochstadt, S. "Sound Gradual Typing: Only Mostly Dead." OOPSLA 2017.
- Bezanson, J. "Abstraction in Technical Computing." PhD thesis, MIT, 2015.
- Bezanson, J., Chen, J., Chung, B., Karpinski, S., Shah, V.B., Vitek, J., Zoubritzky, L. "Julia: Dynamism and Performance Reconciled by Design." OOPSLA 2018.
- Bezanson, J., Edelman, A., Karpinski, S., Shah, V.B. "Julia: A Fresh Approach to Numerical Computing." SIAM Review, 59(1):65--98, 2017.
- Bolz, C.F., Cuni, A., Fijalkowski, M., Rigo, A. "Tracing the Meta-Level: PyPy's Tracing JIT Compiler." ICOOOLPS 2009.
- Bolz, C.F., Tobin-Hochstadt, S. "Meta-tracing Makes a Fast Racket." DLS workshop, 2014.
- Brunthaler, S. "Inline Caching Meets Quickening." ECOOP 2010.
- Chaudhuri, A., Vekris, P., Goldman, S., Roch, M., Levi, G. "Fast and Precise Type Checking for JavaScript." OOPSLA 2017.
- Choi, J., Shull, T., Torrellas, J. "Reusable Inline Caching for JavaScript Performance." PLDI 2019.
- Cousot, P., Cousot, R. "Abstract Interpretation: A Unified Lattice Model for Static Analysis of Programs." POPL 1977.
- Dennis, J.B. "First Version of a Data Flow Procedure Language." Symposium on Programming, 1974.
- Deutsch, L.P., Schiffman, A.M. "Efficient Implementation of the Smalltalk-80 System." POPL 1984.
- Diekmann, L., Tratt, L. "Don't Panic! Better, Fewer, Syntax Errors for LR Parsers." ECOOP 2020.
- Fowler, M. *Domain-Specific Languages*. Addison-Wesley, 2010.
- Gal, A., Eich, B., et al. "Trace-based Just-in-Time Type Specialization for Dynamic Languages." PLDI 2009.
- Garcia, R., Clark, A.M., Tanter, E. "Abstracting Gradual Typing." POPL 2016.
- Greenman, B. "Deep and Shallow Types for Gradual Languages." PLDI 2022.
- Greenman, B., Dimoulas, C., Felleisen, M. "Complete Monitors for Gradual Types." OOPSLA 2019.
- Greenman, B., Dimoulas, C., Felleisen, M. "Gradually Typed Languages Should Be Vigilant!" OOPSLA 2024.
- Hewitt, C., Bishop, P., Steiger, R. "A Universal Modular ACTOR Formalism for Artificial Intelligence." IJCAI 1973.
- Holzle, U., Chambers, C., Ungar, D. "Optimizing Dynamically-Typed Object-Oriented Languages with Polymorphic Inline Caches." ECOOP 1991.
- Hudak, P. "Building Domain-Specific Embedded Languages." ACM Computing Surveys, 28(4es), 1996.
- Jensen, S.H., Moller, A., Thiemann, P. "Type Analysis for JavaScript." SAS 2009.
- Loui, R.P. "In Praise of Scripting: Real Programming Pragmatism." IEEE Computer, 41(7), 2008.
- Marr, S., Burchell, H., Niephaus, F. "Execution vs. Parse-Based Language Servers." DLS 2022.
- Medeiros, S., Mascarenhas, F. "Syntax Error Recovery in Parsing Expression Grammars." SAC 2018.
- Mernik, M., Heering, J., Sloane, A.M. "When and How to Develop Domain-Specific Languages." ACM Computing Surveys, 37(4), 2005.
- Mohammadi, B. "Pel, A Programming Language for Orchestrating AI Agents." arXiv:2505.13453, 2025.
- Muehlboeck, F., Tate, R. "Sound Gradual Typing is Nominally Alive and Well." OOPSLA 2017.
- Ousterhout, J.K. "Scripting: Higher-Level Programming for the 21st Century." IEEE Computer, 31(3), 1998.
- PayPal (anonymous). "A Declarative Language for Building and Orchestrating LLM-Powered Agent Workflows." arXiv:2512.19769, 2025.
- Siek, J.G., Taha, W. "Gradual Typing for Functional Languages." Scheme Workshop, 2006.
- Siek, J.G., Taha, W. "Gradual Typing for Objects." ECOOP 2007.
- Siek, J.G., Vitousek, M.M., Cimini, M., Tobin-Hochstadt, S., Garcia, R. "Refined Criteria for Gradual Typing." SNAPL 2015.
- Takikawa, A., Feltey, D., Greenman, B., New, M.S., Vitek, J., Felleisen, M. "Is Sound Gradual Typing Dead?" POPL 2016.
- Tobin-Hochstadt, S., Felleisen, M. "The Design and Implementation of Typed Scheme." POPL 2008.
- Tobin-Hochstadt, S., Felleisen, M. "Interlanguage Migration: From Scripts to Programs." DLS 2006.
- Tobin-Hochstadt, S., Felleisen, M. "Logical Types for Untyped Languages." ICFP 2010.
- van der Aalst, W.M.P., ter Hofstede, A.H.M., Kiepuszewski, B., Barros, A.P. "Workflow Patterns." Distributed and Parallel Databases, 14(1):5--51, 2003.
- Wagner, T.A. "Practical Algorithms for Incremental Software Development Environments." PhD thesis, UC Berkeley, 1998.
- Wirth, N. "On the Design of Programming Languages." IFIP Congress 1974.
- Wurthinger, T., Woss, A., Stadler, L., Duboscq, G., Simon, D., Wimmer, C. "Self-Optimizing AST Interpreters." DLS 2012.
- Zappa Nardelli, F., Belyakova, J., Pelenitsyn, A., Chung, B., Bezanson, J., Vitek, J. "Julia Subtyping: A Rational Reconstruction." OOPSLA 2018.
