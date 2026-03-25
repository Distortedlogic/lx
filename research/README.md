# lx Language Research

Comprehensive survey of how programming languages are designed and implemented — informing design decisions for lx's tooling, runtime, and language features.

## Table of Contents

### Core Language Implementation

#### Lexing
- [lexing/landscape.md](lexing/landscape.md) — Lexer designs across Python, Rust, V8, Lua, Ruby, Go; handwritten vs generated, keyword recognition, string interning

#### Parsing
- [parsing/landscape.md](parsing/landscape.md) — Parser designs, PEG, recursive descent, Pratt parsing, error recovery, incremental parsing (tree-sitter)

#### AST Design
- [ast/landscape.md](ast/landscape.md) — AST design across languages, ASDL, Rust's 5-level IR pipeline, CST vs AST, memory layouts
- [ast/visitor-patterns.md](ast/visitor-patterns.md) — Visitor pattern variants, tree traversal, tree rewriting, cache-friendly layouts

#### Pattern Matching
- [pattern-matching/landscape.md](pattern-matching/landscape.md) — ML, Rust, Python 3.10+, Elixir, Scala, Swift, C#/Java pattern matching
- [pattern-matching/design-patterns.md](pattern-matching/design-patterns.md) — Maranget's usefulness algorithm, exhaustiveness checking, pattern compilation, design trade-offs

#### Interpreter & VM
- [interpreter/landscape.md](interpreter/landscape.md) — Tree-walking, bytecode, register vs stack VMs, dispatch techniques, CPython/Lua/V8/BEAM deep dives
- [interpreter/internals.md](interpreter/internals.md) — Value representation, NaN boxing, GC, closures, string interning, hash tables, error handling

#### Error Handling
- [error-handling/landscape.md](error-handling/landscape.md) — Exceptions, Result/Option, Go tuples, Common Lisp conditions, algebraic effects, Zig error sets
- [error-handling/design-patterns.md](error-handling/design-patterns.md) — Propagation patterns, error context, recovery strategies, async error handling

#### Builtins & Standard Library
- [builtins/landscape.md](builtins/landscape.md) — Built-in functions across 7 languages, module systems, reflection, error handling primitives
- [builtins/design-patterns.md](builtins/design-patterns.md) — Batteries-included vs minimal, preludes, protocols, iterators, string APIs, concurrency primitives

#### Modules & Packages
- [modules/landscape.md](modules/landscape.md) — Import machinery across Python, Rust, JS/Node, Go, Elixir, Lua, Haskell; circular deps, visibility, resolution
- [package-management/landscape.md](package-management/landscape.md) — Cargo, npm, pip/uv, Go modules, Mix; PubGrub resolver, lockfiles, registries, supply chain security

#### Pipes & Functional Composition
- [pipes-composition/landscape.md](pipes-composition/landscape.md) — Pipe operators across 15 languages (Elixir, F#, Haskell, R, Clojure, Nim, etc.), UFCS, method chaining
- [pipes-composition/design-patterns.md](pipes-composition/design-patterns.md) — First-arg vs placeholder, Railway-Oriented Programming, pipe+async, API design for pipeability

#### Shell Integration
- [shell-integration/landscape.md](shell-integration/landscape.md) — Shell execution across 15 languages (Perl, Ruby, Python, zx, Nushell, YSH, Amber)
- [shell-integration/design-patterns.md](shell-integration/design-patterns.md) — Injection security, error handling, interpolation/escaping, streaming I/O, cross-platform

#### Traits, Protocols & Interfaces
- [traits-protocols/landscape.md](traits-protocols/landscape.md) — Trait systems across 10 languages (Rust, Haskell, Go, Elixir, Swift, Scala, Clojure, etc.)
- [traits-protocols/design-patterns.md](traits-protocols/design-patterns.md) — Nominal vs structural, composition, linearization, dynamic dispatch, the expression problem

#### Coroutines, Yield & Continuations
- [coroutines-yield/landscape.md](coroutines-yield/landscape.md) — Coroutines across 10 languages, stackful vs stackless, symmetric vs asymmetric, continuations
- [coroutines-yield/design-patterns.md](coroutines-yield/design-patterns.md) — State machine transformation, CPS, algebraic effects, orchestrator yield patterns

### Concurrency & Agents

#### Concurrency & Actor Model
- [concurrency/landscape.md](concurrency/landscape.md) — Erlang/OTP, Akka, Orleans, Pony, CSP/Go, structured concurrency, async/await, dataflow
- [concurrency/design-patterns.md](concurrency/design-patterns.md) — Message passing, supervision trees, scheduling, deadlock detection, backpressure

#### Workflow DSLs & Agent Frameworks
- [workflow-dsls/landscape.md](workflow-dsls/landscape.md) — Temporal, Restate, Prefect, Airflow, Step Functions; LangGraph, CrewAI, AutoGen, DSPy, Mastra
- [workflow-dsls/design-patterns.md](workflow-dsls/design-patterns.md) — Van der Aalst's 43 workflow patterns, saga, durable execution, human-in-the-loop

#### AI/LLM Integration
- [ai-llm-integration/landscape.md](ai-llm-integration/landscape.md) — DSPy, BAML, Guidance, LMQL, SGLang, Outlines, Instructor; structured output, constrained decoding
- [ai-llm-integration/design-patterns.md](ai-llm-integration/design-patterns.md) — Structured output patterns, prompt composition, refine loops, RAG, embeddings, token budgeting

#### Refine Loops (Iterative AI Improvement)
- [refine-loops/landscape.md](refine-loops/landscape.md) — Self-Refine, Reflexion, CRITIC, LLM-as-Judge, convergence theory, evaluation frameworks
- [refine-loops/design-patterns.md](refine-loops/design-patterns.md) — Generate-critique-revise, grading rubrics, termination strategies, lx refine construct analysis

### Tooling

#### Linting
- [linting/landscape.md](linting/landscape.md) — 12+ lint tools across Python, Rust, JS, Go, Ruby, Shell, Lua, Elixir
- [linting/design-patterns.md](linting/design-patterns.md) — Parsing strategies, rule systems, auto-fix, plugin architectures, performance patterns

#### Formatting
- [formatting/landscape.md](formatting/landscape.md) — 10 formatters across 10 languages (black, rustfmt, prettier, gofmt, etc.)
- [formatting/design-patterns.md](formatting/design-patterns.md) — Pretty-printing algorithms (Wadler/Oppen), IR design, line-breaking strategies

#### Type Checking
- [type-checking/landscape.md](type-checking/landscape.md) — mypy, pyright, Rust, TypeScript, Hack, Sorbet, Flow, academic type systems
- [type-checking/design-patterns.md](type-checking/design-patterns.md) — Soundness trade-offs, inference patterns, gradual typing, decision matrix

#### Testing
- [testing/landscape.md](testing/landscape.md) — Testing frameworks across 7 languages (pytest, cargo test, Jest, Go testing, etc.)
- [testing/design-patterns.md](testing/design-patterns.md) — Fixtures, assertion design, mocking, property-based testing, fuzzing, coverage

#### Diagnostics & Error Messages
- [diagnostics/landscape.md](diagnostics/landscape.md) — Rust/Elm/Python/TypeScript/Clang error messages, ariadne/miette/codespan-reporting libraries
- [diagnostics/design-patterns.md](diagnostics/design-patterns.md) — Snippet rendering, structured diagnostics, error recovery, writing guidelines, progressive disclosure

#### CLI Toolchain Design
- [cli-toolchain/landscape.md](cli-toolchain/landscape.md) — Cargo, Go, Mix, npm, Poetry/uv, Deno, Zig, Just CLI designs
- [cli-toolchain/design-patterns.md](cli-toolchain/design-patterns.md) — Subcommand organization, help, errors, output formats, plugins, watch mode, shell completions

#### Language Server Protocol
- [lsp/landscape.md](lsp/landscape.md) — LSP specification, rust-analyzer/Pyright/gopls/Nickel implementations, tower-lsp vs lsp-server, Salsa incremental computation

#### Debugger & Profiler
- [debugger/landscape.md](debugger/landscape.md) — DAP specification, Python/Lua/Erlang debugger internals, trace hooks, breakpoints, flamegraphs, profilers

#### Anthropic/Claude Rust SDKs
- [tooling/anthropic-rust-sdks.md](tooling/anthropic-rust-sdks.md) — 30+ Rust crates: 11 direct API clients + 20+ Claude Code CLI wrappers; tokio/streaming, hooks, MCP, adoption metrics, ecosystem fragmentation analysis

#### Agent Harness Design
- [harness/agent_harness_design.md](harness/agent_harness_design.md) — What harnesses are, context management, tool design, context rot, Anthropic's long-running agent pattern
- [harness/agent_tool_orchestration.md](harness/agent_tool_orchestration.md) — Tool consolidation, CodeAct, PTC, response optimization, naming, sandboxing, parallel execution
- [harness/agent_session_and_state_management.md](harness/agent_session_and_state_management.md) — Session persistence, state bridging, checkpoint/recovery
- [harness/agent_observability_and_feedback_loops.md](harness/agent_observability_and_feedback_loops.md) — Observability patterns, feedback loops, debugging agent behavior
- [harness/agent_configuration_as_code.md](harness/agent_configuration_as_code.md) — Configuration patterns, CLAUDE.md, project instructions, context files
- [harness/context_engineering_deep_dive.md](harness/context_engineering_deep_dive.md) — Context window engineering, compaction, summarization, sub-agent compression
- [harness/domain_specific_harness_patterns.md](harness/domain_specific_harness_patterns.md) — Domain-specific harness patterns for coding, research, data analysis
- [harness/multi_agent_coordination_harness.md](harness/multi_agent_coordination_harness.md) — Multi-agent coordination, delegation, fan-out/fan-in
- [harness/opencode_harness_analysis.md](harness/opencode_harness_analysis.md) — OpenCode (SST): client-server architecture, Plan/Build modes, Hashline editing, 75+ providers, LSP-as-context
- [harness/pi_harness_analysis.md](harness/pi_harness_analysis.md) — Pi (badlogic): minimal 4-tool harness, <1K token system prompt, anti-MCP, tree-structured sessions, extensibility
- [harness/REF_anthropic_effective_harnesses.md](harness/REF_anthropic_effective_harnesses.md) — Anthropic's two-agent long-running pattern, JSON feature tracking, session initialization protocol
- [harness/REF_openai_harness_engineering.md](harness/REF_openai_harness_engineering.md) — OpenAI's harness engineering guidance

### Runtime & Security

#### Backend Architecture
- [backend-architecture/landscape.md](backend-architecture/landscape.md) — Pluggable runtimes (tokio, Java SPI, SLF4J, Tower, wgpu, React renderers), DI frameworks
- [backend-architecture/design-patterns.md](backend-architecture/design-patterns.md) — Trait design, deny backends, composition, mock testing, lifecycle, async traits

#### Plugins & Hooks
- [plugins-and-hooks/landscape.md](plugins-and-hooks/landscape.md) — Plugin systems (pluggy, proc macros, tapable, WASM), hook mechanisms, middleware patterns
- [plugins-and-hooks/design-patterns.md](plugins-and-hooks/design-patterns.md) — Discovery, isolation, hot reloading, WASM sandboxing, extensibility patterns

#### REPL
- [repl/landscape.md](repl/landscape.md) — Python/IPython/Jupyter, Node, Clojure nREPL, Elixir IEx, GHCi, Pry; line editing, rich output, remote REPLs

#### Sandboxing & Capabilities
- [sandboxing/landscape.md](sandboxing/landscape.md) — Deno permissions, WASM/WASI, capability-based security, Lua sandboxing, container isolation

### Academic Research
- [scripting-languages-academic.md](scripting-languages-academic.md) — 50+ papers on scripting language design, gradual typing, JIT, workflow patterns, actor model, agent DSLs
