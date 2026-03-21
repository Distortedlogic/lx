# REPL Design Landscape

Research on read-eval-print loop design across programming languages, line editing libraries, rich output, remote REPLs, and design patterns.

---

## 1. Language-Specific REPL Implementations

### 1.1 Python

**The standard REPL** uses `code.InteractiveConsole` (which wraps `codeop.CommandCompiler`) to detect incomplete expressions. The compiler attempts to compile input; if parsing fails with an "unexpected EOF" error, the REPL requests another line. Complete expressions get `exec`'d in a persistent namespace dict.

**Display pipeline:**
- Expressions (not statements) trigger `sys.displayhook(result)`. The default displayhook calls `repr()` on the result, prints it to `sys.stdout`, and stores it in `builtins._`.
- `__repr__` is used by the REPL for display; `__str__` is used by `print()`. Convention: `__repr__` should return a valid Python expression when possible; `__str__` returns a human-readable form.
- PEP 217 introduced `sys.displayhook` to allow customization of how interactive results are printed. Reassigning `sys.displayhook` to a callable lets you intercept all REPL output.

**IPython** extends the standard REPL with:
- **Magic commands** prefixed with `%` (line magics) and `%%` (cell magics). `%` is not a valid unary operator in Python, which prevents collision with user code. Examples: `%timeit`, `%run`, `%load`, `%who`, `%debug`, `%%html`, `%%bash`.
- **Tab completion** via Jedi (static analysis) and runtime introspection of live objects. Completes attributes, file paths, dict keys, and function signatures.
- **Rich output** using `_repr_html_()`, `_repr_json_()`, `_repr_latex_()`, `_repr_png_()` methods on objects, enabling HTML tables, LaTeX math, and inline images.
- **Input transformation** pipeline: raw input -> magic expansion -> alias expansion -> AST transformation -> execution.
- **History** stored in SQLite database (`~/.ipython/profile_default/history.sqlite`), searchable with `%history` and Ctrl-R.

**Jupyter kernel architecture:**
- Decoupled frontend/backend via ZeroMQ. The kernel is a separate process; any number of frontends can connect.
- Five ZeroMQ sockets:
  - **Shell** (ROUTER/DEALER): handles `execute_request`, `complete_request`, `inspect_request`, `is_complete_request`
  - **IOPub** (XPUB/SUB): broadcasts `stream` (stdout/stderr), `display_data`, `execute_result`, `error`, `status`
  - **Stdin** (ROUTER/DEALER): kernel asks frontend for user input (`input_request`)
  - **Control** (ROUTER/DEALER): out-of-band messages like `shutdown_request`, `interrupt_request`, `debug_request`
  - **Heartbeat** (REQ/REP): simple echo for liveness detection
- **Wire protocol** (v5.5): messages are multipart ZeroMQ frames: `[identities, delimiter('<IDS|MSG>'), HMAC-SHA256 signature, header JSON, parent_header JSON, metadata JSON, content JSON, optional binary buffers]`
- **MIME-typed output**: `display_data` messages carry a `data` dict mapping MIME types to content: `text/plain`, `text/html`, `text/markdown`, `image/png` (base64), `image/svg+xml`, `application/json`, `application/latex`.
- **Execution model**: monotonically increasing `execution_count`, `status: busy` before processing, `status: idle` after. `is_complete_request` lets frontends ask whether input is a complete expression, incomplete, or invalid.

Sources:
- [PEP 217 - Display Hook](https://peps.python.org/pep-0217/)
- [IPython Overview](https://ipython.readthedocs.io/en/stable/overview.html)
- [IPython Magic Commands](https://ipython.readthedocs.io/en/stable/interactive/magics.html)
- [Jupyter Architecture](https://docs.jupyter.org/en/latest/projects/architecture/content-architecture.html)
- [Jupyter Messaging Protocol](https://jupyter-client.readthedocs.io/en/stable/messaging.html)

---

### 1.2 Node.js

The `repl` module provides a programmatic REPL with:

**Incomplete expression detection** via `repl.Recoverable`. The eval function catches `SyntaxError` and checks for messages like "Unexpected end of input" or "Unexpected token". When detected, it throws `new repl.Recoverable(err)` to signal the REPL should prompt for more input rather than reporting an error.

**Built-in commands:**
- `.break` / `.clear`: abandon current multi-line expression
- `.editor`: enter multi-line editor mode (Ctrl-D to execute, Ctrl-C to cancel)
- `.exit`: exit the REPL
- `.help`: list commands
- `.save` / `.load`: save/load REPL history to/from file

**Tab completion** via a configurable `completer` function. Default completion covers global scope, Node.js modules, filenames, and object properties on the current context.

**Persistent history** via the `historyPath` option or `NODE_REPL_HISTORY` env variable. Defaults to `~/.node_repl_history`. History size controlled by `NODE_REPL_HISTORY_SIZE` (default 1000).

**Two modes:**
- `repl.REPL_MODE_SLOPPY`: default, uses sloppy/non-strict JavaScript
- `repl.REPL_MODE_STRICT`: wraps input in `'use strict';`

**Custom eval**: you can replace the eval function entirely, enabling REPLs for non-JavaScript languages or custom execution environments. The eval function signature is `(cmd, context, filename, callback)`.

Sources:
- [Node.js REPL Documentation](https://nodejs.org/api/repl.html)
- [Better handling of recoverable errors PR](https://github.com/nodejs/node/pull/18915)

---

### 1.3 Clojure

Clojure's REPL ecosystem is uniquely central to its development culture. "REPL-driven development" means the editor has a live connection to the running application, and developers evaluate forms in the context of the running process.

**nREPL (Network REPL):**
- Asynchronous message-based protocol over TCP sockets.
- **Bencode** encoding by default: two scalar types (byte strings, integers), two collection types (dicts, lists). Zero runtime dependencies.
- **EDN transport** available since nREPL 0.7 for richer data types.
- **TTY transport** for basic telnet-style connections.
- **Message format**: maps with string-valued `:op` key. Example: `{:op "eval" :code "(+ 2 2)" :session "abc123"}`.
- **Core operations**: `eval` (evaluate code), `clone` (create new session), `describe` (server capabilities), `interrupt` (cancel evaluation), `close` (end session), `stdin` (provide input), `completions`, `lookup`.
- **Session management**: `clone` creates a new session with isolated bindings. Each session maintains its own `*ns*`, `*e`, `*1`/`*2`/`*3` vars. Sessions are identified by UUIDs.
- **Middleware architecture**: nREPL handlers are composed as middleware stacks, similar to Ring. Middleware can intercept, modify, or handle messages. `cider-nrepl` provides middleware for completion, definition lookup, debugging, test running, etc.

**Socket REPL** (Clojure 1.8+): simpler alternative built into Clojure itself. No protocol overhead, just a socket that accepts s-expressions and returns results. Started via JVM system properties.

**CIDER**: Emacs package that connects to nREPL. Inline evaluation, stack traces, test integration, debugging, code navigation. Despite its name, `cider-nrepl` middleware is editor-agnostic and used by Calva (VS Code), vim-fireplace, and others.

Sources:
- [nREPL GitHub](https://github.com/nrepl/nrepl)
- [nREPL Transports](https://nrepl.org/nrepl/design/transports.html)
- [CIDER Docs](https://docs.cider.mx/cider-nrepl/index.html)
- [JUXT nREPL Overview](https://www.juxt.pro/blog/nrepl/)
- [Lambda Island Clojure REPLs Guide](https://lambdaisland.com/guides/clojure-repls/clojure-repls)

---

### 1.4 Elixir IEx

IEx is Elixir's interactive shell, built on top of the Erlang shell infrastructure.

**Helpers** (accessed via `h()`):
- `h/1`: documentation for modules/functions
- `i/1`: inspect data type, showing type, description, reference modules, implemented protocols
- `v/0`, `v/1`: return value of previous/nth expression
- `r/1`: recompile and reload a module
- `c/1`: compile a file
- `l/1`: load a module's object code
- `t/1`: show type specs for a module
- `b/1`: show callbacks for a behaviour
- `exports/1`: list all exported functions

**Pipe operator shorthand**: lines starting with `|>` automatically prepend `v()`, enabling chained exploration without variables.

**Debugging:**
- `IEx.pry()`: pauses execution and opens an IEx session with access to local variables and lexical scope. Invoked with `iex --dbg pry -S mix`.
- `break!/2,4`: sets instrumented breakpoints on functions with configurable stop counts. Supports pattern matching: `break! URI.parse("https" <> _, _)`.
- Navigation: `n()` / `next()` for stepping into pipe stages, `continue()` to resume, `respawn()` to leave the pried process.
- Related helpers: `breaks()`, `whereami()`, `remove_breaks/1`, `reset_break/1`.

**Remote shells**: connect to running nodes via `iex --sname bar --remsh foo@hostname`. `Ctrl+G` opens the user switch command menu for managing multiple shell sessions and connecting to remote nodes.

**Configuration:**
- `.iex.exs`: loaded from current directory or `~/.iex.exs` on startup. Code evaluated in shell context, so bindings and imports are available immediately.
- `IEx.configure/1`: runtime configuration for `width`, `colors` (ANSI), `inspect` options (limit, pretty), `history_size`, prompt templates with `%counter` and `%node` substitution.

**Multi-line**: evaluates input line-by-line. Incomplete expressions show `...(n)>` continuation prompts. `#iex:break` forces abandonment of incomplete expressions.

**Autocomplete**: type module name + dot + Tab. Functions marked `@doc false` or `@impl true` are excluded.

Sources:
- [IEx Documentation](https://hexdocs.pm/iex/IEx.html)
- [Elixir IEx source](https://github.com/elixir-lang/elixir/blob/main/lib/iex/lib/iex.ex)

---

### 1.5 Rust (evcxr)

Evcxr provides a REPL and Jupyter kernel for Rust, a compiled language where interactive evaluation is fundamentally harder.

**How it works:**
- Each expression/statement is wrapped in a function, compiled with `rustc` into a shared library, and dynamically linked into the running process.
- State persists between evaluations by moving variables into and out of compiled functions via serialization or raw pointer passing.
- Incremental compilation is used where possible to reduce latency.
- The `:dep` command adds crate dependencies (fetched from crates.io) for subsequent expressions.

**Limitations of compiled-language REPLs:**
- **Latency**: every expression triggers a full compilation cycle. Even with incremental compilation, this is noticeably slower than interpreted REPLs.
- **References across evaluations**: storing references into variables that persist between compilations is not permitted, because lifetimes cannot span compilation units. This is a fundamental tension with Rust's ownership model.
- **Thread interruption**: Jupyter's "interrupt kernel" feature does not work because Rust threads cannot be interrupted from outside.
- **Long programs**: the tool is designed for brief, interactive exploration. Writing substantial programs in the REPL is impractical.
- **Error recovery**: compilation errors are reported inline, but the feedback loop is slower than in interpreted languages.

**Jupyter kernel features:**
- Custom HTML display by implementing the `Debug` trait with HTML output.
- Standard Jupyter rich output (plots, tables) via MIME-type responses.
- Variable persistence and mutation across cells.

Sources:
- [evcxr GitHub](https://github.com/evcxr/evcxr)
- [Interactive Rust with EVCXR](https://depth-first.com/articles/2020/09/21/interactive-rust-in-a-repl-and-jupyter-notebook-with-evcxr/)
- [Rust Notebooks with Evcxr](https://blog.abor.dev/p/evcxr)

---

### 1.6 Lua

Lua's REPL reflects its design philosophy of simplicity and minimal footprint (entire implementation: ~25,000 lines of C, ~200KB binary).

**Interactive mode design:**
- Each input line is treated as a separate **chunk** (a sequence of statements). If a line cannot form a complete chunk, the interpreter waits for more input.
- The `=` prefix is a convenience: `= expr` is equivalent to `return expr`, causing the result to be printed.
- Chunks are compiled with `load()` (or `loadstring()` in 5.1), which returns a function. The function is then called. This two-step process (compile then execute) is central to Lua's design.
- `load()` is pure and total: no side effects, always returns either a function or an error message. This differs from `eval` in other languages.
- `dofile()` combines loading and execution; `loadfile()` separates them.

**Why Lua REPLs are easy to build:**
- The entire language is embeddable by design. All interaction with the host happens through a clean C API.
- Lua favors mechanisms representable through the C API over special syntax, so everything the REPL does is accessible programmatically.
- First-class functions and `load()` make eval trivial.
- The `lua-repl` library (pure Lua) provides REPL logic as a library, extensible through polymorphism and plugins, abstracting away I/O.

**Limitations:**
- No built-in tab completion, syntax highlighting, or history (the standalone interpreter is deliberately minimal).
- Enhanced REPLs like `luaish` add tab completion and shell sub-modes on top.

Sources:
- [A Look at the Design of Lua (CACM)](https://cacm.acm.org/research/a-look-at-the-design-of-lua/)
- [Programming in Lua - Chunks](https://www.lua.org/pil/1.1.html)
- [lua-repl library](https://github.com/hoelzro/lua-repl)
- [luaish](https://github.com/stevedonovan/luaish)

---

### 1.7 Haskell GHCi

GHCi is GHC's interactive environment. Notably, it supports debugging in a lazy evaluation context, which presents unique challenges.

**Core commands:**
- `:type expr` / `:t expr`: display type of an expression without evaluating it
- `:info name` / `:i name`: show everything known about a name (class instances, type definition, source location)
- `:browse Module` / `:browse *Module`: list exported identifiers (with `*`, show all in-scope)
- `:kind Type` / `:k Type`: infer and print the kind of a type expression
- `:load file` / `:l file`: recursively load modules
- `:reload` / `:r`: recompile changed modules
- `:set +t`: show types of bound variables after evaluation
- `:set +s`: show timing and memory statistics
- `:set +m`: enable automatic multi-line mode (detect incomplete expressions)
- `:def name expr`: define custom GHCi commands as Haskell expressions (macro system)

**The `it` variable**: automatically binds the result of the previous expression, similar to `_` in Python.

**Multi-line input**: `:{ ... :}` blocks, or `:set +m` for automatic detection.

**Debugging** (enabled by default for interpreted code):
- `:break Module line [col]`: set breakpoint at a source location
- `:break function`: set breakpoint on a function definition
- `:step`: single-step through reductions
- `:steplocal`: step only within the current top-level function
- `:stepmodule`: step only within the current module
- `:continue`: resume from breakpoint
- `:trace`: evaluate with history tracking (enables backward navigation)
- `:hist`: show evaluation history
- `:back` / `:forward`: navigate through history steps
- `:print var`: display value without forcing evaluation (respects laziness)
- `:force var`: fully evaluate thunks and display
- `:list`: show source code around current breakpoint or identifier
- `-fbreak-on-exception`: break on all exceptions
- `-fbreak-on-error`: break only on uncaught exceptions

**Unique aspects for a compiled-language REPL:**
- Can load pre-compiled modules alongside interpreted ones for performance.
- Custom print functions via `-interactive-print` flag.
- Extended type defaulting rules at the interactive prompt (relaxes type ambiguity compared to compiled code).
- Debugging in a lazy language: the "stack" bears little resemblance to a lexical call stack; execution is demand-driven. GHCi provides backward stepping through evaluation history as a workaround.

**Configuration**: `.ghci` file in home directory, loaded on startup. Supports multi-line commands via `:{` and `:}`.

Sources:
- [GHCi User's Guide](https://downloads.haskell.org/ghc/latest/docs/users_guide/ghci.html)

---

### 1.8 Ruby IRB/Pry

**IRB** is Ruby's built-in REPL. Minimal but functional: evaluates expressions, prints results, maintains history.

**Pry** is a runtime developer console that replaces IRB with powerful introspection:

**Object navigation:**
- `cd object`: change context into any Ruby object. Navigate classes, instances, modules as if they were directories.
  - `cd Person.first` (into an instance), `cd Person` (into the class itself)
  - `cd ..` (up one level), `cd /` (back to root/start), `cd -` (toggle between two scopes)
- `ls`: unified wrapper around Ruby's introspection methods (`methods`, `instance_variables`, `constants`, `local_variables`, `instance_methods`, `class_variables`). Shows what's available in the current scope, colored by category.

**Source code viewing:**
- `show-source method_name`: display method source code with syntax highlighting. Works for Ruby methods and (with `pry-doc` gem) C methods.
- `show-doc method_name`: display documentation.
- Code longer than a page is sent through a pager (`less`).

**Debugging integration:**
- `binding.pry`: insert a breakpoint anywhere in code. When hit, drops into a Pry session with access to local scope.
- `whereami`: show source code around the current execution point.
- Integration with `pry-byebug` for step/next/continue/finish commands.

**Plugin system:**
- Pry has a robust plugin architecture. Plugins can add commands, modify the REPL behavior, or integrate with frameworks.
- `pry-rails`: switches Rails console to Pry, adds `show-models`, `show-routes`, `show-middleware` commands.
- `pry-doc`: enables source viewing for C-level Ruby methods.
- `pry-rescue`: automatically opens Pry on unhandled exceptions.
- `pry-stack_explorer`: navigate the call stack (up/down/frame commands).

**Command system**: Pry commands are Ruby classes, making them composable and extensible. Custom commands can be defined in `.pryrc`.

Sources:
- [Pry GitHub](https://github.com/pry/pry)
- [Pry homepage](https://pry.github.io/)
- [Pry State Navigation Wiki](https://github.com/pry/pry/wiki/State-navigation)

---

## 2. REPL Design Patterns

### 2.1 Incomplete Expression Detection

Three main approaches:

**Bracket/brace counting**: track open brackets, braces, and parentheses. If input ends with unbalanced delimiters, request more input. Simple, language-agnostic, but misses string literals and comments containing brackets.

**Parser-based detection**: attempt to parse input. If the parser fails with an "unexpected end of input" or "unexpected EOF" error, the expression is incomplete. More accurate but requires a parser that distinguishes "incomplete" from "invalid". Node.js uses `repl.Recoverable`, Python uses `codeop.compile_command()` (returns `None` for incomplete code, a code object for complete code, raises `SyntaxError` for invalid code). Jupyter's `is_complete_request` message type formalizes this as a protocol operation returning `"complete"`, `"incomplete"`, `"invalid"`, or `"unknown"`.

**Heuristic detection**: check for specific error message strings ("Unexpected end of input", "missing ) after argument list"). Used by Node.js as a fallback. Brittle across language versions.

**Multi-line prompts**: most REPLs show a distinct continuation prompt (Python: `...`, Elixir: `...(n)>`, Node.js: `...`) to signal that more input is expected.

### 2.2 Tab Completion

**Static analysis**: parse the source to determine available names. Works without running code. Used by Jedi (Python), Haskell Language Server.

**Runtime introspection**: query live objects for their attributes/methods. More accurate for dynamic languages. IPython uses this extensively. Pry's `ls` is essentially a completion over runtime introspection results.

**File path completion**: detect when the cursor is in a string context and complete filesystem paths. Most REPLs support this.

**Contextual completion**: complete differently based on context (e.g., after `import` complete module names, after `.` complete attributes, after `--` complete CLI flags). IPython and Elixir IEx do this.

**Implementation**: typically a `completer(text, state)` callback. Readline calls it repeatedly with increasing `state` until it returns `None`. Modern libraries like reedline provide menu-based completion UIs.

### 2.3 History

**Persistent storage**: readline uses flat files (`~/.python_history`, `~/.node_repl_history`). IPython uses SQLite. Reedline supports both file and SQLite backends.

**Search**: Ctrl-R reverse incremental search (readline standard). Fish-style autosuggestions show the most recent matching history entry as grayed-out text. Reedline supports both.

**History expansion**: bash-style `!!` (last command), `!$` (last argument), `!n` (nth command). Reedline supports this via the `bashisms` feature.

**Session vs global**: some REPLs maintain per-session history (Clojure nREPL sessions) while others share history across sessions (most terminal REPLs).

**Context metadata**: reedline's SQLite backend can store arbitrary `HistoryEntryContext` (timestamp, working directory, exit status) alongside each entry.

### 2.4 Rich Output

**Syntax highlighting**: reedline and IPython highlight input as you type. Requires a tokenizer for the target language. Reedline accepts a pluggable `Highlighter` trait implementation.

**Terminal graphics protocols**:
- **Sixel**: oldest protocol (DEC VT340, 1987). Palette-based, widely supported. Encodes images as escape sequences.
- **iTerm2 inline images**: base64-encoded image data in escape sequences. Supported by iTerm2, WezTerm, VSCode terminal, and many others. Fewer bytes and full color compared to Sixel.
- **Kitty graphics protocol**: Kitty-specific, supports animation, placement, and reference by ID. Supported by Kitty and (partially) WezTerm.
- Choice between protocols depends on target terminal. Libraries like `rasterm` (Go) and `term-image` (Python) abstract over multiple protocols.

**Tables**: Rich (Python) renders formatted tables with borders, alignment, and color in the terminal. IPython displays DataFrames as HTML tables in notebooks.

**MIME bundles**: Jupyter's approach. A single output can carry multiple representations (`text/plain` + `text/html` + `image/png`), and the frontend chooses the best one. This is the most flexible system for rich output.

### 2.5 Error Presentation

**Key principle**: errors must not kill the REPL session. Catch all exceptions from user code, display them, and return to the prompt.

**Stack traces**: Python shows full tracebacks. IPython enhances them with syntax highlighting, context lines, and `%debug` magic to enter post-mortem debugging. Pry can be configured to automatically open on unhandled exceptions (`pry-rescue`).

**Partial evaluation**: GHCi's `:print` shows values without forcing evaluation, preventing exceptions from lazy thunks. This is unique to lazy languages.

**Error context**: Elixir IEx shows the error type, message, and (via `Exception.blame/2`) annotates function arguments that didn't match. Clojure nREPL returns structured error data (exception class, message, stacktrace) that editors can render.

### 2.6 REPL vs Notebook

**REPL advantages**: fast startup, low overhead, works over SSH, integrates with shell workflows, sequential command history. Best for: quick experiments, debugging, system administration, script development.

**Notebook advantages**: persistent visual record of computation, rich media output, mix code/prose/output, shareable. Best for: data analysis, visualization, documentation, education, reproducible research.

**Jupyter architecture insight**: the kernel protocol can serve both. A terminal frontend (jupyter-console) connects to the same kernel as a web notebook. The `is_complete_request` message was added specifically to support terminal frontends that need incomplete expression detection.

**Hybrid approaches**: VS Code's interactive window, IPython's `%notebook` magic, Elixir Livebook. These blur the line by embedding notebook-like cells in editors or adding code execution to documents.

### 2.7 Line Editing Libraries

**GNU Readline** (C): the original. Emacs and vi keybindings, history, completion, undo, kill ring. License: GPL (which forces linking programs to be GPL). Used by Python, Ruby, Lua, GHCi (via Haskeline wrapper), and many others.

**Rustyline** (Rust): readline implementation based on Antirez's linenoise. Features: file-backed history, customizable completers, hints, Emacs and vi modes, UTF-8 support, Windows support. Simpler API than reedline. MIT licensed.

**Reedline** (Rust): modern line editor powering Nushell. Features beyond rustyline:
- SQLite history backend with metadata
- Fish-style autosuggestion hints
- Content-aware syntax highlighting (pluggable `Highlighter`)
- Menu-based completion system
- Configurable keybinding engine (Emacs and vi modes)
- Multi-line editing with validation
- Clipboard integration (system clipboard via feature flag)
- Undo support
- Bash-style history expansion (via `bashisms` feature)
- Async external printer for concurrent output
- MIT licensed.

**Crossterm** (Rust): not a line editor but a cross-platform terminal manipulation library. Provides raw mode, event reading, cursor control, styling. Both rustyline and reedline use crossterm internally for terminal I/O.

**Haskeline** (Haskell): Haskell binding providing readline-like functionality for GHCi. Cross-platform, supports completion, history, vi/emacs modes.

**Comparison summary:**
| Feature | Readline | Rustyline | Reedline |
|---------|----------|-----------|----------|
| Language | C | Rust | Rust |
| History backends | File | File | File + SQLite |
| Syntax highlighting | No | Hints only | Full |
| Completion UI | Inline | Inline | Menu + inline |
| Vi/Emacs modes | Both | Both | Both |
| Multi-line | Limited | Basic | Full with validation |
| License | GPL | MIT | MIT |

Sources:
- [Reedline GitHub](https://github.com/nushell/reedline)
- [Rustyline GitHub](https://github.com/kkawakam/rustyline)
- [Kitty Graphics Protocol](https://sw.kovidgoyal.net/kitty/graphics-protocol/)
- [Crafting Interpreters REPL Discussion](https://github.com/munificent/craftinginterpreters/issues/799)
- [Create Your Own Programming Language with Rust - REPL](https://createlang.rs/01_calculator/repl.html)

### 2.8 Remote/Network REPL

**nREPL** (Clojure): the most mature network REPL protocol. Bencode over TCP, session management, middleware extensibility. Designed for tool integration (editors, debuggers, profilers). Has been adopted beyond Clojure (e.g., Alda music language).

**Elixir remote shell**: leverages Erlang's distribution protocol. `iex --remsh node@host` connects to a running Erlang VM. The BEAM VM was designed for this: hot code loading, process isolation, and distribution are built in. `Ctrl+G` menu provides session management.

**Jupyter kernel protocol**: effectively a network REPL. ZeroMQ transport, JSON messages, language-agnostic. Over 100 language kernels exist. The architecture separates "what to run" (kernel) from "how to display" (frontend), enabling web, terminal, and IDE frontends.

**Debug Adapter Protocol (DAP)**: Microsoft's protocol for debugger communication. Jupyter integrates DAP for debugging in notebooks. Relevant because debugging via a REPL often happens over a network protocol.

**Production REPL access**: Elixir is the strongest example. Connecting to a production Erlang node with IEx is common practice for inspection and hot-fixes. Clojure's nREPL is also used this way. Python's `code.interact()` can be embedded for emergency access, though this is rare in practice.

**Security considerations**: remote REPLs are powerful attack surfaces. nREPL has no built-in authentication (relies on network-level controls). Elixir remote shells require the Erlang cookie (a shared secret). Jupyter uses HMAC-SHA256 message signing with a shared key.

Sources:
- [nREPL Building Clients](https://nrepl.org/nrepl/building_clients.html)
- [nREPL Beyond Clojure](https://metaredux.com/posts/2019/01/12/nrepl-beyond-clojure.html)
- [Alda and nREPL](https://blog.djy.io/alda-and-the-nrepl-protocol/)

---

## 3. Key Takeaways for lx

1. **Incomplete expression detection** should be parser-based, returning `Complete`, `Incomplete`, or `Invalid`. This is the approach used by Jupyter's `is_complete_request` and Python's `codeop`.

2. **Network REPL protocol** (like nREPL) is worth considering early. The protocol should be transport-agnostic with pluggable encodings. nREPL's bencode is simple but limiting; Jupyter's JSON is verbose but universal.

3. **Rich output** via MIME types is the most flexible approach. Even a terminal REPL can benefit from structured output types (table, error, tree) that degrade gracefully to plain text.

4. **Line editing**: reedline is the most capable Rust option. Its pluggable highlighter, completer, and validator traits map well to a custom language REPL.

5. **The Elixir model** of remote REPL access to running processes is highly relevant for an agentic workflow language where agents are long-running processes.

6. **Lua's simplicity** is instructive: `load()` + `call()` is all you need for a basic REPL. The two-step compile-then-execute model keeps the implementation clean and the semantics clear.
