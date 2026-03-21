# Architectural Patterns for Extensibility

Design patterns for plugin systems, hook mechanisms, and extension architectures. Synthesized from the landscape survey of real-world systems.

---

## 1. Plugin Discovery

How the host finds plugins. The choice here determines the developer experience, security posture, and deployment complexity.

### 1.1 Filesystem scanning

The host scans directories for files matching a naming convention or extension.

**Examples:**
- **Git hooks:** Executable files in `.git/hooks/` named `pre-commit`, `post-commit`, etc.
- **Cargo plugins:** Binaries named `cargo-*` anywhere on `$PATH`.
- **Vim plugins:** Files under `plugin/`, `ftplugin/`, `autoload/` directories in `runtimepath`.
- **Neovim Lua:** Lua modules under `lua/` in runtimepath, discovered via `require()`.
- **Go plugin package:** Application scans a directory for `.so` files.
- **WASM plugins (wasmtime):** Application scans a directory for `.wasm` files.

**Trade-offs:**
- Simple to implement and understand.
- No central registry or manifest required.
- Hard to control load ordering and dependency resolution.
- Plugin must conform to naming/location conventions.
- Security: anything in the scan path gets loaded.

### 1.2 Entry points / registries

Plugins declare themselves in metadata that a discovery mechanism reads.

**Examples:**
- **Python setuptools entry_points:** Packages declare entry points in `pyproject.toml`; consumers discover via `importlib.metadata.entry_points(group='mygroup')`.
- **ESLint plugins:** npm packages named `eslint-plugin-*` registered in config.
- **VS Code extensions:** Declared in `package.json` with `contributes` and `activationEvents`.
- **WordPress plugins:** PHP files with a standard header comment block in `wp-content/plugins/`.
- **Minecraft/Paper plugins:** `plugin.yml` manifest declaring name, version, main class, dependencies.

**Trade-offs:**
- Structured metadata enables dependency resolution, versioning, conditional loading.
- Requires a packaging/distribution system.
- More ceremony for plugin authors.
- Enables tooling (listing, searching, validating plugins).

### 1.3 Explicit registration

The application code explicitly registers plugins by name, path, or object reference.

**Examples:**
- **pluggy:** `pm.register(MyPlugin())` -- explicit object registration.
- **Go database/sql:** `import _ "github.com/lib/pq"` triggers `init()` which calls `sql.Register()`.
- **Express.js:** `app.use(myMiddleware)` -- explicit middleware registration.
- **Rack:** `use MyMiddleware` in `config.ru`.
- **Elixir:** Module name passed as config or function argument; `@behaviour` adopted explicitly.
- **Tower:** `ServiceBuilder::new().layer(TimeoutLayer::new(...)).service(inner)`.

**Trade-offs:**
- Maximum control over what gets loaded and in what order.
- No magic -- all plugin wiring is visible in code.
- Every plugin must be manually added.
- Cannot discover plugins installed after the application was written without additional mechanism.

### 1.4 Convention-based

Plugins are discovered by following naming conventions, module structure, or interface conformance.

**Examples:**
- **Python namespace packages:** Any package contributing modules under `myapp.plugins.*` is automatically discovered.
- **Django apps:** Modules providing `models.py`, `views.py`, etc. in a standard layout.
- **Rails engines:** Gems with an `Engine` subclass are auto-detected via `Rails::Engine.subclasses`.
- **Cargo plugins:** Convention is `cargo-<name>` binary name.
- **Lua:** Module name maps to filesystem path via pattern substitution in `package.path`.

**Trade-offs:**
- Low friction for plugin authors who follow conventions.
- Can be fragile (typos in names, wrong directory = silent failure).
- Combines well with other patterns (filesystem scan + naming convention).

Sources: [Python Packaging Guide](https://packaging.python.org/en/latest/guides/creating-and-discovering-plugins/), [The Rust Programming Language](https://doc.rust-lang.org/book/ch14-05-extending-cargo.html), [VS Code Extension API](https://code.visualstudio.com/api/get-started/extension-anatomy)

---

## 2. Plugin Isolation

How the host protects itself from plugin misbehavior: crashes, memory corruption, resource exhaustion, malicious code.

### 2.1 No isolation (in-process, same address space)

The plugin runs in the host's process with full access to memory.

**Examples:** Python plugins (pluggy, importlib), Ruby gems, Lua `require()`, Vim/Neovim plugins, Express.js middleware, Elixir behaviours.

**Risks:** Plugin crashes take down the host. Plugins can corrupt host memory, access secrets, monkey-patch shared state.

**Mitigations:** Code review, trust boundaries (only load vetted plugins), language-level safety (Rust's type system prevents memory corruption even in-process).

### 2.2 Process isolation

The plugin runs in a separate OS process. Communication via IPC (pipes, sockets, RPC).

**Examples:**
- **hashicorp/go-plugin:** Plugin is a subprocess; communication via net/rpc or gRPC.
- **VS Code Extension Host:** Extensions run in a separate Node.js process from the UI.
- **Neovim remote plugins:** External processes communicate via msgpack-RPC.
- **Language servers (LSP):** Separate process per language server, communicating via JSON-RPC.

**Trade-offs:**
- Plugin crash does not crash host.
- Plugins have their own memory space -- no corruption risk.
- Higher latency (serialization, IPC overhead).
- More complex setup (process lifecycle management).
- Plugin cannot directly share data structures with host.

### 2.3 WASM sandboxing

WebAssembly provides a memory-safe virtual machine with no default system access.

**Examples:**
- **Extism:** Cross-language framework wrapping wasmtime/wazero/V8. Plugins compiled to WASM from any language.
- **wasmtime Component Model:** WIT interfaces define typed contracts. Hosts load `.wasm` components, instantiate with explicit capabilities.
- **Figma plugins:** Run user code in WASM sandbox with a rich design manipulation API.
- **Envoy/Istio WASM filters:** Network proxy plugins run in WASM sandbox.

**Key properties:**
- **Linear memory:** Each WASM module gets its own memory space. Cannot access host memory.
- **Capability-based security:** The host explicitly provides capabilities (filesystem, network, etc.) to the plugin. Default is nothing.
- **No syscall interface:** WASM has no syscalls. All I/O goes through host-provided imports.
- **Language agnostic:** Plugins can be written in Rust, C/C++, Go, AssemblyScript, Python, etc.
- **Deterministic:** Execution is sandboxed and reproducible.

**Architecture (Extism):**
- **Host SDK:** Language-specific library for loading and running WASM plugins.
- **PDK (Plugin Development Kit):** Language-specific library compiled into the plugin, providing utilities for reading input, returning output, accessing config, making HTTP calls (if permitted), setting/getting variables.
- **Bytes-in/bytes-out interface:** Host calls plugin functions by name, passing and receiving byte arrays. Serialization format is plugin-defined (JSON, protobuf, Cap'n Proto, etc.).
- **Host functions:** The host can expose functions callable by plugins, enabling controlled access to host capabilities.

**Architecture (wasmtime Component Model):**
- **WIT files:** Define the interface contract (imports and exports) in a language-agnostic IDL.
- **`bindgen!` macro (Rust):** Generates type-safe bindings from WIT.
- **Engine/Store/Linker/Component model:** Engine is shared, Store is per-instance (state), Linker connects imports, Component is compiled WASM.

**Trade-offs:**
- Near-process-isolation security without the overhead of separate processes.
- Language agnostic -- the best path to polyglot plugin ecosystems.
- WASM compilation/instantiation has startup cost.
- Complex types require explicit serialization across the boundary.
- WASI (WebAssembly System Interface) is still evolving.

Sources: [Extism documentation](https://extism.org/docs/concepts/plug-in-system/), [Extism on GitHub](https://github.com/extism/extism), [wasmtime plugins guide](https://docs.wasmtime.dev/wasip2-plugins.html), [Wasmtime Security](https://docs.wasmtime.dev/security.html), [Building Native Plugin Systems with WASM Components](https://tartanllama.xyz/posts/wasm-plugins/)

### 2.4 Language-level isolation

Some languages provide isolation mechanisms short of process boundaries.

**Examples:**
- **Lua sandboxing:** Create a new `lua_State` with restricted standard library. The C API allows fine-grained control over what functions are available.
- **Ruby refinements:** Lexically-scoped modifications prevent global monkey-patching.
- **Java SecurityManager (deprecated):** Class-level permission checks for untrusted code.
- **Deno permissions:** Runtime capabilities (filesystem, network, env) granted via CLI flags.

---

## 3. Plugin APIs / Contracts

How the host defines what plugins can do.

### 3.1 Trait/interface-based contracts

The host defines a trait (Rust), interface (Go/Java), protocol (Elixir), or abstract class (Python) that plugins implement.

**Examples:**
- **Rust:** `trait Plugin { fn name(&self) -> &str; fn execute(&self, ctx: &Context) -> Result<()>; }`
- **Go:** `type Greeter interface { Greet(name string) string }`
- **Elixir:** `@callback process(data :: map()) :: {:ok, map()} | {:error, String.t()}`
- **Java:** `interface Plugin { void onEnable(); void onDisable(); }`

**Properties:**
- Compiler-enforced (Rust, Go, Elixir compile-time checks).
- Clear documentation of the contract.
- Versioning through interface evolution (add optional methods with defaults).
- Enables static dispatch (Rust generics) or dynamic dispatch (trait objects, interfaces).

### 3.2 Hook specifications

The host declares hook points as function signatures. Plugins implement matching functions.

**Examples:**
- **pluggy:** `@hookspec def pytest_runtest_protocol(item, nextitem): ...`
- **WordPress:** `do_action('save_post', $post_id, $post)` / `apply_filters('the_title', $title)`
- **tapable:** `this.hooks.compile = new SyncHook(["params"])`

**Properties:**
- Decoupled: plugins don't need to know about each other.
- Flexible: hook specs can be added without changing the plugin API.
- Less type safety than trait/interface contracts (especially in dynamic languages).
- Supports multiple calling conventions (collect all results, first result, waterfall, bail).

### 3.3 Event buses / message passing

Plugins communicate through events (named messages with payloads) rather than direct function calls.

**Examples:**
- **Node.js EventEmitter:** `emitter.on('data', handler)` / `emitter.emit('data', payload)`.
- **Neovim autocommands:** `vim.api.nvim_create_autocmd("BufWritePre", { callback = fn })`.
- **Qt signals/slots:** Typed signals connected to slot methods.
- **Emacs hooks:** `(add-hook 'after-save-hook #'my-function)`.

**Properties:**
- Loose coupling -- emitter doesn't know who's listening.
- Hard to return values to the emitter (unlike filter hooks).
- Natural fit for async/concurrent systems.
- Can be ordered (priority) or unordered.

### 3.4 AST visitor contracts

Plugins define visitors over a syntax tree. The host traverses and calls visitor methods for matching node types.

**Examples:**
- **Babel:** `visitor: { Identifier(path, state) { ... } }`
- **ESLint rules:** `create(context) { return { CallExpression(node) { ... } }; }`
- **Rust compiler lints:** Lint passes implement `LateLintPass` with methods for each HIR node type.
- **pytest assertion rewriting:** AST walker that transforms `assert` nodes.

**Properties:**
- Extremely powerful for code analysis and transformation tools.
- Contract is defined by the AST schema (node types).
- Composition: multiple visitors can be merged for single-pass traversal.
- Requires the host to own parsing/traversal.

Sources: [Babel Plugin Handbook](https://github.com/jamiebuilds/babel-handbook/blob/master/translations/en/plugin-handbook.md), [ESLint: Create Plugins](https://eslint.org/docs/latest/extend/plugins), [pluggy docs](https://pluggy.readthedocs.io/en/stable/)

---

## 4. Plugin Configuration

How plugins declare, receive, and validate their configuration.

### 4.1 Options passed at registration

Configuration is passed directly when the plugin is registered or called.

**Examples:**
- **Babel plugins:** `module.exports = function(api, options) { ... }`
- **Express middleware:** `app.use(cors({ origin: 'http://example.com' }))`
- **Neovim `setup()`:** `require('plugin').setup({ key = value })`
- **Tower layers:** `TimeoutLayer::new(Duration::from_secs(30))`

### 4.2 Declarative manifest

Configuration declared in a structured file (TOML, YAML, JSON).

**Examples:**
- **VS Code extensions:** `package.json` with `contributes`, `activationEvents`, configuration schemas.
- **ESLint:** `eslint.config.js` with rule severity and options.
- **Minecraft plugins:** `plugin.yml` with `name`, `version`, `main`, `depend`, `commands`.
- **Cargo.toml:** Package metadata, features, dependencies.
- **Drupal:** YAML schema files using Kwalify-inspired schema language.

### 4.3 Configuration schemas

Plugins declare a schema for their configuration, enabling validation and editor autocompletion.

**Examples:**
- **VS Code:** Extensions declare `contributes.configuration` with JSON Schema for settings.
- **Backstage:** Separate `.json` schema files with `"$schema"` declaration.
- **ESLint:** Rule schemas (JSON Schema) validate rule options.

### 4.4 Environment-based

Configuration via environment variables, often for sensitive values.

**Examples:**
- **hashicorp/go-plugin:** `PLUGIN_MIN_PORT`, `PLUGIN_MAX_PORT`, magic cookie env vars.
- **Cargo plugins:** `CARGO` env var for calling back to cargo.
- **Docker plugins:** `--env` flags passed to plugin containers.

---

## 5. Plugin Dependencies

How plugins declare and resolve dependencies on other plugins.

### 5.1 Topological sort

Dependencies between plugins form a DAG (Directed Acyclic Graph). A topological sort determines load order.

**Algorithm (Kahn's):**
1. Compute in-degree (number of dependencies) for each plugin.
2. Add all plugins with in-degree 0 to a queue.
3. Dequeue a plugin, add it to the sorted order, decrement in-degrees of dependents.
4. Repeat until queue is empty.
5. If sorted order has fewer plugins than total, a cycle exists -- error.

**Examples:**
- **Django migrations:** Topological sort resolves migration dependency ordering.
- **Webpack plugin dependencies:** Load order determined by dependency graph.
- **Minecraft plugins:** `depend` and `softdepend` in `plugin.yml` determine load order.

### 5.2 Explicit ordering

No automatic resolution; load order is specified manually.

**Examples:**
- **Express.js middleware:** Order is the order of `app.use()` calls.
- **Rack middleware:** Order specified in `config.ru` or `application.rb`.
- **pluggy:** LIFO registration order, overridable with `tryfirst`/`trylast`.
- **Emacs hooks:** `depth` parameter controls position in hook list.
- **WordPress hooks:** Integer priority (default 10) controls call order.
- **tapable:** Plugins tap in order; the hook type (waterfall, bail, loop) determines how results flow.

### 5.3 Soft dependencies

Optional dependencies that enhance functionality but aren't required.

**Examples:**
- **pluggy `optionalhook`:** Implementation for a hook that may or may not exist.
- **Elixir `@optional_callbacks`:** Callbacks that modules may omit.
- **Minecraft `softdepend`:** Loaded after the dependency if present, but loads without it.
- **npm `peerDependencies`/`optionalDependencies`:** Declared but not required.

Sources: [Topological Sorting (Wikipedia)](https://en.wikipedia.org/wiki/Topological_sorting), [Django migration dependencies analysis](https://medium.com/swlh/how-django-uses-topological-sorting-for-resolving-migration-dependencies-1a93b544037f)

---

## 6. Extension Points

How to design a system with clear, well-defined extension points.

### 6.1 The open/closed principle applied

A system is "open for extension, closed for modification" when new behavior can be added without changing existing code. Extension points are the mechanism.

**Patterns for creating extension points:**

1. **Named hooks at strategic locations:** Insert hook calls at every point where behavior might vary. WordPress does this pervasively (thousands of `do_action` and `apply_filters` calls throughout core).

2. **Pipeline/middleware architecture:** The entire request-response flow is an extension point. Each middleware can intercept, modify, or short-circuit. Express, Rack, Tower, Plug.

3. **Trait/interface abstraction:** Define abstractions at boundaries where implementations may vary. Elixir behaviours, Rust traits, Go interfaces.

4. **Event emission:** Emit events at significant moments. Listeners extend behavior without the emitter knowing. Neovim autocommands, Node.js EventEmitter.

5. **Visitor pattern over data structures:** For tree/graph processing, let plugins define visitors. Babel, ESLint, compiler lint passes.

### 6.2 Designing for extensibility

**Principles from surveyed systems:**

- **Prefer many small hooks over few large ones.** WordPress's thousands of hooks provide fine-grained control. tapable's multiple hook instances per compilation phase let plugins target exactly what they need.

- **Separate specification from implementation.** pluggy's clean split between `@hookspec` and `@hookimpl` lets specs evolve independently of implementations. Elixir's `@callback` attributes define contracts without implementation.

- **Allow plugins to see each other's results.** Waterfall hooks (tapable), filter hooks (WordPress), and wrapper hooks (pluggy) let plugins compose their effects.

- **Support both "before" and "after" extension.** Emacs advice (`:before`, `:after`, `:around`), middleware (wrap inner call), pluggy wrappers (yield before/after).

- **Make extension points discoverable.** VS Code's contribution points are documented JSON schemas. WordPress hooks are searchable. ESLint AST node types are defined by the parser.

---

## 7. Hot Reloading

Dynamic loading and unloading of plugins at runtime without restarting the host.

### 7.1 Challenges

**Dangling references (Rust/C/C++):** Trait object vtables and function pointers live in the loaded library's code segment. If the library is unloaded while references exist, any call through them segfaults. There is no safe way to detect all references at runtime in these languages.

**Thread-local storage (Linux):** On Linux, a dynamic library cannot be unloaded until all threads that have accessed TLS from that library have exited. If the main thread ever calls into the library, it can never be unloaded within the process's lifetime.

**Memory layout changes:** If a struct's layout changes between library versions, existing instances in memory become corrupt. The type system cannot help here because layouts are fixed at compile time.

**Global state:** Frameworks that initialize singletons (OpenGL contexts, database connections, loggers) will fail or panic when reloaded code tries to re-initialize.

**Panics across boundaries:** In Rust, panicking across an FFI boundary aborts the process. In Go, plugin panics crash the host (unless using process isolation).

### 7.2 Solutions

**Library copying:** Copy the `.so`/`.dll` to a temp location before loading. This allows the original file to be replaced on disk. Load the copy, keep it loaded until all references are gone, then unload.

**Reference counting:** Track all objects created by a plugin. Only unload the library when the reference count drops to zero.

**Indirection layers:** Instead of calling plugin functions directly, call through an indirection table that can be swapped. Fyrox game engine uses this: plugin code constructs declarative scene descriptions that the host renders, rather than rendering directly.

**Message-based architecture:** If all plugin communication is message-based (bytes in, bytes out), hot reload is straightforward -- stop sending messages to the old plugin, start the new one, resume. This is how Erlang/OTP hot code loading works.

**WASM reload:** WASM modules have no shared memory with the host. Instantiate the new module, migrate state (if needed), drop the old instance. No vtable or TLS issues.

Sources: [Hot Reloading in Fyrox](https://fyrox-book.github.io/beginning/hot_reloading.html), [Hot reloading Rust (John Austin)](https://johnaustin.io/articles/2022/hot-reloading-rust), [Hot Reloading Rust (Robert Krahn)](https://robert.kra.hn/posts/hot-reloading-rust/), [Bevy dynamic plugin discussion](https://github.com/bevyengine/bevy/issues/4843)

---

## 8. WASM as Plugin System

WebAssembly is emerging as the primary mechanism for language-agnostic, sandboxed plugin systems.

### 8.1 Architecture overview

```
+------------------+          +------------------+
|   Host App       |          |   Plugin (WASM)  |
|                  |          |                  |
|  Host SDK        |  bytes   |  PDK             |
|  (Rust/Go/JS/    | <------> |  (Rust/Go/C/     |
|   Python/etc.)   |  in/out  |   JS/etc.)       |
|                  |          |                  |
|  WASM Runtime    |          |  Compiled to     |
|  (wasmtime/      |          |  .wasm           |
|   wazero/V8)     |          |                  |
+------------------+          +------------------+
```

### 8.2 Interface definition

**WIT (WebAssembly Interface Types):**
```wit
package myapp:plugins;

interface host {
    log: func(msg: string);
    get-config: func(key: string) -> option<string>;
}

world plugin {
    import host;
    export init: func();
    export process: func(input: list<u8>) -> list<u8>;
}
```

WIT defines typed imports (host-provided) and exports (plugin-provided). The component model provides rich types: records, variants, enums, lists, options, results, resources.

**Extism's approach:** Simpler -- plugins export named functions with bytes-in/bytes-out semantics. Host functions are registered programmatically. No IDL file required, but less type safety.

### 8.3 Key systems

| System | Runtime | Approach | Notable users |
|--------|---------|----------|---------------|
| Extism | wasmtime, wazero, V8, SpiderMonkey | High-level SDK + PDK | Various |
| wasmtime (direct) | wasmtime | Component model + WIT | Spin (Fermyon) |
| Envoy WASM | V8/wamr | Proxy-Wasm ABI | Istio, Envoy |
| Figma | Custom runtime | API-based sandbox | Figma |
| Zellij | wasmer | Plugin trait | Zellij terminal |
| moonrepo | wasmtime | WASM plugins for build tools | moon |

### 8.4 Trade-offs vs. other isolation mechanisms

| Property | In-process | Process isolation | WASM |
|----------|-----------|------------------|------|
| Startup overhead | None | High (fork/exec) | Medium (compile + instantiate) |
| Call overhead | None (function call) | High (IPC serialization) | Low-medium (host function calls) |
| Memory isolation | None | Full (OS-level) | Full (linear memory) |
| Crash isolation | None | Full | Full |
| Language support | Same language | Any (with RPC) | Any (with WASM target) |
| Hot reload | Hard (vtable issues) | Easy (restart process) | Easy (new instance) |
| Capability control | None | OS-level | Fine-grained (WASI) |

Sources: [Extism plug-in system](https://extism.org/docs/concepts/plug-in-system/), [wasmtime plugins](https://docs.wasmtime.dev/wasip2-plugins.html), [Wasmtime security model](https://docs.wasmtime.dev/security.html), [WASM plugins (moonrepo)](https://moonrepo.dev/docs/guides/wasm-plugins), [Building Native Plugin Systems with WASM Components](https://tartanllama.xyz/posts/wasm-plugins/)

---

## 9. Language-Level Extensibility

How programming languages themselves provide extensibility mechanisms that blur the line between "language feature" and "plugin."

### 9.1 Operator overloading

Redefine the behavior of operators (`+`, `-`, `*`, `==`, `<`, `[]`, etc.) for user-defined types.

**Examples:**
- **Rust:** `impl Add for MyType { fn add(self, rhs: Self) -> Self { ... } }` via `std::ops` traits.
- **Python:** `__add__`, `__eq__`, `__getitem__`, etc. (dunder methods).
- **C++:** `Type operator+(const Type& a, const Type& b)`.
- **Kotlin:** `operator fun plus(other: Money): Money`.
- **Haskell:** Typeclass instances for `Num`, `Eq`, `Ord`, etc.

**Design tension:** Enables expressive DSLs (matrix math, monetary calculations, parser combinators) but can obscure behavior when overused.

### 9.2 Custom literals

User-defined literal syntax for constructing values from literal tokens.

**Examples:**
- **C++:** `100_km` via `operator"" _km(unsigned long long)`.
- **Swift:** `ExpressibleByStringLiteral`, `ExpressibleByIntegerLiteral` protocols.
- **Rust:** Proc macros as function-like macros provide similar capabilities: `sql!("SELECT * FROM users")`.

### 9.3 Macros as extension mechanism

Macros let users define new syntax, control flow, and code generation.

**Compile-time (AST-level):**
- **Rust proc macros:** Arbitrary token transformations at compile time. Derive, attribute, and function-like macros.
- **Elixir macros:** AST manipulation enabling DSLs (Phoenix routes, Ecto schemas, ExUnit tests).
- **Scheme/Racket hygienic macros:** `syntax-rules` and `syntax-case` for safe syntactic abstractions.
- **Julia macros:** AST transformation via `@macro` with hygiene.

**Text-level:**
- **C preprocessor:** `#define` for token substitution. No AST awareness, leading to well-known pitfalls.
- **Lisp/Clojure macros:** Code-as-data (homoiconicity) makes macros natural. `defmacro` receives and returns S-expressions.

**Runtime (reflection-based):**
- **Ruby:** `method_missing`, `define_method`, `class_eval` for dynamic method definition.
- **Python metaclasses:** `__init_subclass__`, `__class_getitem__` for class creation customization.

### 9.4 Protocols and interfaces

Type-based dispatch mechanisms that allow extending behavior for existing types.

**Examples:**
- **Elixir protocols:** `defprotocol` + `defimpl` -- dispatch on data type. Any module can implement a protocol for any type.
- **Rust traits:** Can be implemented for foreign types (with orphan rule restrictions). Enables extending behavior without modifying the original type.
- **Go interfaces:** Structural typing -- any type satisfying the interface is accepted, no explicit declaration needed.
- **Swift protocol extensions:** Default implementations for protocol methods.
- **Clojure protocols:** `defprotocol` + `extend-protocol` for type-based dispatch with runtime extension.

### 9.5 Extension methods / categories

Adding methods to existing types without subclassing or modifying the original.

**Examples:**
- **Kotlin extension functions:** `fun String.isPalindrome(): Boolean`.
- **C# extension methods:** Static methods in static classes, invoked as instance methods.
- **Swift extensions:** `extension Array where Element: Comparable { ... }`.
- **Ruby open classes/refinements:** Reopen any class to add methods (globally or scoped).
- **Rust traits on foreign types:** Implement a local trait for a foreign type.

Sources: [Extensibility in Programming Languages (ACM)](https://dl.acm.org/doi/pdf/10.1145/1499949.1500003), [Macros for Domain-Specific Languages](https://www2.ccs.neu.edu/racket/pubs/oopsla20-bkf.pdf), [Operator overloading (Wikipedia)](https://en.wikipedia.org/wiki/Operator_overloading)

---

## 10. Comparative Analysis: Choosing a Pattern

### 10.1 Decision matrix

| Requirement | Best pattern |
|-------------|-------------|
| Untrusted plugins | WASM sandbox or process isolation |
| Polyglot plugins | WASM (any lang -> .wasm) or gRPC (any lang with gRPC) |
| Lowest latency | In-process, trait/interface-based |
| Maximum flexibility | Hook system (pluggy-style) |
| Code transformation | AST visitor pattern (Babel/ESLint model) |
| Request/response pipeline | Middleware pattern (Tower/Rack/Express) |
| Simple CLI extensibility | Filesystem/PATH convention (Cargo/Git) |
| Rich editor/IDE extension | Manifest + activation events (VS Code model) |
| Data pipeline | Waterfall hooks (tapable) or filter hooks (WordPress) |
| Hot reloadable | WASM or process-isolated |

### 10.2 Composition patterns

Many real systems combine multiple patterns:

- **pytest:** Entry point discovery + pluggy hook system + AST transformation (assertion rewriting).
- **webpack:** npm package discovery + tapable hooks + Babel-style AST visitors (loaders).
- **VS Code:** Manifest-based discovery + activation events + contribution points + extension host (process isolation).
- **Neovim:** Filesystem scanning + autocommand events + RPC for remote plugins + Lua `require()`.
- **Rails:** Gem discovery + Rack middleware + engine mounting + monkey patching (refinements).
- **Terraform:** hashicorp/go-plugin (process isolation + gRPC) + plugin discovery via registry.

### 10.3 Evolution patterns

Plugin systems tend to evolve along predictable paths:

1. **Start simple:** Filesystem scanning or explicit registration.
2. **Add contracts:** Define traits/interfaces/hooks for type safety.
3. **Add isolation:** Move from in-process to process-isolated or WASM.
4. **Add discovery:** Build a registry or convention system.
5. **Add configuration schemas:** Validate plugin config, enable tooling.
6. **Add dependency resolution:** Topological sort for load ordering.

The critical early decision is the **communication boundary**: in-process function calls (fast but coupled), message passing (flexible but slower), or bytes-in/bytes-out (language-agnostic but requires serialization). Changing this boundary later is architecturally expensive.
