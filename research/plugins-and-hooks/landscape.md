# Plugin Systems and Hook Mechanisms Across Languages and Ecosystems

Research document covering how major languages and ecosystems implement plugins, hooks, and extension points.

---

## 1. Python

### 1.1 setuptools entry_points

Entry points are the standard Python mechanism for installed packages to advertise components for discovery by other code. Defined in `setup.py` or `pyproject.toml`, they map a group name to a dotted module path and optional object reference.

**How it works:**

- Packages declare entry points in their distribution metadata (stored in `entry_points.txt` inside `*.dist-info`).
- Consumers discover plugins via `importlib.metadata.entry_points(group='mygroup')`.
- Each `EntryPoint` object has `.name`, `.group`, and `.value`, plus a `.load()` method that imports the referenced object.
- The `pytest11` entry point group is the canonical example: any package registering under `pytest11` is automatically loaded as a pytest plugin.

**Example declaration (pyproject.toml):**
```toml
[project.entry-points."timmins.display"]
plain = "timmins_plugin_plain:PlainDisplay"
```

**Discovery code:**
```python
from importlib.metadata import entry_points
plugins = entry_points(group='timmins.display')
for ep in plugins:
    impl = ep.load()
```

Sources: [Python Packaging Guide: Creating and discovering plugins](https://packaging.python.org/en/latest/guides/creating-and-discovering-plugins/), [setuptools: Entry Points](https://setuptools.pypa.io/en/latest/userguide/entry_point.html), [Entry points specification](https://packaging.python.org/en/latest/specifications/entry-points/)

### 1.2 pluggy (pytest's plugin framework)

pluggy is the most sophisticated hook-based plugin system in the Python ecosystem. It powers pytest, tox, and devpi.

**Core components:**

| Component | Role |
|-----------|------|
| `PluginManager` | Central registry; manages plugin registration and hook dispatch |
| `HookspecMarker` | Decorator marking functions as hook specifications (contracts) |
| `HookimplMarker` | Decorator marking functions as hook implementations |
| `HookCaller` | Performs 1:N invocation of all registered implementations |
| `HookRelay` | Attribute of PluginManager; contains HookCaller objects by name |

**Hook specifications** define the contract -- only the function signature (name + argument names) matters; the body is typically just a docstring:

```python
hookspec = pluggy.HookspecMarker("myproject")

@hookspec
def myhook(config, args):
    """Called when X happens."""
```

**Hook implementations** match specs by function name:

```python
hookimpl = pluggy.HookimplMarker("myproject")

@hookimpl
def myhook(config, args):
    return some_result
```

**Key options on hookspec:**

- `firstresult=True` -- Stop calling after the first non-None return; return a single value instead of a list.
- `historic=True` -- Allow calling the hook before all plugins are registered; late-registering plugins receive the historic call automatically.
- `warn_on_impl=DeprecationWarning(...)` -- Emit warnings when plugins implement a deprecated hook.

**Key options on hookimpl:**

- `tryfirst=True` / `trylast=True` -- Influence ordering within the LIFO call chain.
- `wrapper=True` (new-style) -- Wraps all other hookimpls; uses `yield` to receive aggregated results:
  ```python
  @hookimpl(wrapper=True)
  def myhook(config, args):
      result = yield  # other hookimpls execute here
      return modified_result
  ```
- `hookwrapper=True` (legacy) -- Similar but uses an `outcome` object.
- `optionalhook=True` -- No warning if no matching spec exists.
- `specname="other_name"` -- Override name matching, allowing multiple implementations of the same spec in one module.

**Call ordering:** Default is LIFO (last registered, first called). `tryfirst`/`trylast` create priority categories within that ordering. Wrappers always surround non-wrappers.

**The 1:N call loop:** `pm.hook.myhook(arg1=val)` iterates all registered implementations, collects non-None returns into a list (or stops at first non-None if `firstresult=True`). Exceptions halt execution and propagate through wrappers.

**Argument flexibility:** Implementations may accept fewer arguments than specs -- enables forward-compatible hook signatures.

Sources: [pluggy documentation](https://pluggy.readthedocs.io/en/stable/), [pluggy API reference](https://pluggy.readthedocs.io/en/stable/api_reference.html), [pluggy on GitHub](https://github.com/pytest-dev/pluggy), [pluggy talk](https://devork.be/talks/pluggy/pluggy.html)

### 1.3 importlib plugin loading and namespace packages

**Direct importlib loading:**
```python
import importlib
mod = importlib.import_module("myplugin.module")
```

**Namespace packages as plugin mechanism:** By making `myapp.plugins` a namespace package (no `__init__.py`), any separately-installed distribution can contribute modules under that path. Discovery uses `pkgutil.iter_modules()`:

```python
import pkgutil, importlib
import myapp.plugins

def iter_namespace(ns_pkg):
    return pkgutil.iter_modules(ns_pkg.__path__, ns_pkg.__name__ + ".")

discovered = {
    name: importlib.import_module(name)
    for finder, name, ispkg in iter_namespace(myapp.plugins)
}
```

PEP 420 eliminated the need for `__init__.py` files with path manipulation -- the import machinery now handles namespace packages automatically.

Sources: [Python Packaging Guide: Creating and discovering plugins](https://packaging.python.org/en/latest/guides/creating-and-discovering-plugins/), [importlib documentation](https://docs.python.org/3/library/importlib.html)

### 1.4 AST transformers as plugins (pytest assertion rewriting)

pytest rewrites `assert` statements at import time to provide rich failure messages. The mechanism:

1. A PEP 302 import hook (`AssertionRewritingHook`) is installed early during pytest startup.
2. When test modules are imported, the hook intercepts the import, parses the source to AST, walks the tree to find `assert` nodes, and rewrites them to capture intermediate values.
3. The modified AST is compiled to bytecode and loaded as the module.
4. Only test modules and registered plugins are rewritten (controlled by `python_files` config and `pytest11` entry point).
5. Plugins can opt in via `pytest.register_assert_rewrite("myplugin")`.

This is a general pattern: import hooks + AST transformation = compile-time plugin mechanism within Python's dynamic import system.

Sources: [pytest: Writing plugins](https://docs.pytest.org/en/stable/how-to/writing_plugins.html), [pytest assertion rewrite source](https://github.com/pytest-dev/pytest/blob/main/src/_pytest/assertion/rewrite.py), [Assertion rewriting in Pytest (analysis)](https://www.pythoninsight.com/2018/02/assertion-rewriting-in-pytest-part-4-the-implementation/)

---

## 2. Rust

### 2.1 Procedural macros

Proc macros are Rust's primary compile-time extension mechanism. They operate on `TokenStream` values -- the raw token representation of Rust source code.

**Three kinds:**

| Kind | Attribute | Input | Output |
|------|-----------|-------|--------|
| Derive | `#[derive(MyMacro)]` | The struct/enum `TokenStream` | Additional impl items |
| Attribute | `#[my_attr]` | The annotated item | Replacement item |
| Function-like | `my_macro!(...)` | The macro invocation body | Arbitrary tokens |

**How they work:** Each proc macro is a function `fn(TokenStream) -> TokenStream` (derive macros also receive the attribute arguments). The compiler loads them from a separate crate (`proc-macro = true` in Cargo.toml), invokes them during compilation, and splices the output tokens into the token stream.

**The ecosystem triad:**

- **`proc-macro2`** -- A wrapper around `proc_macro` types that works outside of proc-macro context. Enables unit testing of macro logic, since `proc_macro` types are only valid within the compiler.
- **`syn`** -- Parses `TokenStream` into a full Rust AST (structs like `DeriveInput`, `ItemFn`, etc.). Provides `parse_macro_input!` for convenient parsing.
- **`quote`** -- The `quote!` macro converts Rust syntax back into `TokenStream`, with `#variable` interpolation for splicing values.

**Typical workflow:**
```
Input TokenStream
  -> proc_macro2::TokenStream (via .into())
  -> syn::parse2() -> structured AST
  -> analyze/transform
  -> quote!{...} -> proc_macro2::TokenStream
  -> proc_macro::TokenStream (via .into())
```

Sources: [Rust Reference: Procedural Macros](https://doc.rust-lang.org/reference/procedural-macros.html), [proc_macro2 docs](https://docs.rs/proc-macro2), [syn docs](https://docs.rs/syn), [quote docs](https://docs.rs/quote), [Procedural macros in Rust (LogRocket)](https://blog.logrocket.com/procedural-macros-in-rust/)

### 2.2 Cargo plugins

Cargo's plugin system is filesystem/PATH-based: any binary named `cargo-xxx` on `$PATH` (typically `~/.cargo/bin/`) becomes invokable as `cargo xxx`. The invocation `cargo something` translates to executing `cargo-something`.

- `cargo --list` discovers and lists all available subcommands.
- `cargo help something` invokes `cargo-something something --help`.
- Plugins receive the `CARGO` environment variable to call back into cargo.
- Examples: `cargo-expand`, `cargo-edit`, `cargo-release`, `cargo-semver-checks`.

Sources: [The Rust Programming Language: Extending Cargo](https://doc.rust-lang.org/book/ch14-05-extending-cargo.html), [Third party cargo subcommands](https://github.com/rust-lang/cargo/wiki/Third-party-cargo-subcommands)

### 2.3 Compiler plugins (unstable)

Rust's unstable compiler plugin API (`#![feature(plugin)]`) allows loading dylibs into the compiler to add custom lint passes, syntax extensions, and derive-like macros. Lints run late in compilation with full type information. This API has been largely superseded by stable proc macros and is subject to change. Custom lint tools now typically use `clippy`-style lint passes or the `rustc_driver` API.

Sources: [Rust Compiler Plugins (historical docs)](https://doc.rust-lang.org/1.5.0/book/compiler-plugins.html), [rustc_lint source](https://github.com/rust-lang/rust/blob/master/compiler/rustc_lint/src/lib.rs)

### 2.4 Trait-based plugin patterns

The idiomatic Rust plugin pattern:

1. Define a trait as the plugin contract in a shared "core" crate.
2. Plugin authors implement the trait in their own crate.
3. Registration happens via a function (often `extern "C"`) that constructs and returns a `Box<dyn Trait>`.

**Dynamic loading with `libloading`:** Plugins compile as `cdylib` crates. The host uses `libloading::Library::new()` to open the `.so`/`.dll`/`.dylib`, then `lib.get::<fn(&mut dyn Registrar)>(b"register")` to find the registration symbol.

**Critical safety issues:**

- **ABI instability:** Rust has no stable ABI. The plugin and host must be compiled with the same rustc version. A `PluginDeclaration` struct exports `rustc_version` and `core_version` for verification.
- **Vtable invalidation:** Trait object vtables live in the library's code segment. If the library is unloaded while trait objects survive, calling any method crashes. Solution: the **Proxy Pattern** -- wrap each `Box<dyn Trait>` alongside an `Rc<Library>` to prevent the library from being dropped while objects exist:
  ```rust
  pub struct FunctionProxy {
      function: Box<dyn Function>,
      _lib: Rc<Library>,
  }
  ```
- **Memory layout divergence:** If struct layouts differ between host and plugin (different compiler versions), reading fields produces garbage. Panics across FFI boundaries abort the process.

Sources: [Plugins in Rust (Michael F Bryan)](https://adventures.michaelfbryan.com/posts/plugins-in-rust/), [Plugins in Rust: Diving into Dynamic Loading (NullDeref)](https://nullderef.com/blog/plugin-dynload/), [Dynamic Loading and Plugins (unofficial FFI guide)](https://s3.amazonaws.com/temp.michaelfbryan.com/dynamic-loading/index.html)

---

## 3. JavaScript / Node.js

### 3.1 npm packages as plugins

The npm ecosystem's plugin convention: packages named `<host>-plugin-<name>` or `@scope/<host>-plugin-<name>` are discovered by name convention or explicit configuration. The host application `require()`s or `import()`s them, then calls an expected export (often a function or class with a standard interface). No formal plugin registry exists beyond npm itself -- discovery is either explicit (config file lists plugin names) or convention-based (scan `node_modules` for matching prefixes).

### 3.2 webpack / tapable plugin system

tapable is webpack's hook library, providing 9 hook classes along two dimensions:

**Control flow patterns:**

| Pattern | Behavior |
|---------|----------|
| Basic | Calls all tapped functions sequentially, ignores return values |
| Waterfall | Passes each function's return value as input to the next |
| Bail | Stops when any function returns a non-undefined value |
| Loop | Restarts from the first function if any returns non-undefined |

**Execution models:** Sync, AsyncSeries (sequential), AsyncParallel (concurrent).

**Hook classes:** `SyncHook`, `SyncBailHook`, `SyncWaterfallHook`, `SyncLoopHook`, `AsyncParallelHook`, `AsyncParallelBailHook`, `AsyncSeriesHook`, `AsyncSeriesBailHook`, `AsyncSeriesWaterfallHook`.

**Tapping methods:**
- `.tap(name, fn)` -- synchronous
- `.tapAsync(name, fn)` -- callback-based async (function receives a `callback` parameter)
- `.tapPromise(name, fn)` -- Promise-returning async

**Invocation methods (for hook owners):**
- `.call(...args)` -- synchronous
- `.callAsync(...args, callback)` -- async with callback
- `.promise(...args)` -- returns a Promise

**Interception:** All hooks support interceptor objects:
```javascript
hook.intercept({
  call: (...args) => {},       // when hook fires
  tap: (tap) => {},            // when a plugin registers
  loop: (...args) => {},       // each loop iteration
  register: (tap) => tap       // modify tap configuration at registration time
});
```

**HookMap:** Manages keyed collections of hooks (`new HookMap(key => new SyncHook([...]))`). `.for(key)` creates or retrieves; `.get(key)` retrieves without creation.

**Context:** Plugins can share data through a context object passed as the first parameter when `context: true` is set in the tap options.

**Performance:** tapable compiles optimized execution functions based on the number and type of registered plugins, avoiding unnecessary overhead.

Sources: [tapable on GitHub](https://github.com/webpack/tapable), [webpack Plugin API](https://webpack.js.org/api/plugins/), [The new plugin system (Tobias Koppers)](https://medium.com/webpack/the-new-plugin-system-week-22-23-c24e3b22e95)

### 3.3 Babel plugins (visitor-based AST transforms)

Babel's compilation pipeline: **Parse** (source -> AST) -> **Transform** (traverse/modify AST via plugins) -> **Generate** (AST -> source + sourcemaps).

Plugins export a function receiving Babel's API and options, returning an object with a `visitor`:

```javascript
module.exports = function(api, options) {
  return {
    visitor: {
      Identifier: {
        enter(path, state) { /* ... */ },
        exit(path, state) { /* ... */ }
      }
    }
  };
};
```

**Key concepts:**

- **Visitor pattern:** Methods keyed by AST node type. Babel does depth-first traversal, calling `enter` on descent and `exit` on ascent.
- **Path objects:** Represent the node's position in the tree (not the node itself). Provide methods for parent/sibling access, type checking, replacement, removal, and scope information.
- **Scope handling:** Tracks variable bindings and references. Methods for checking bindings, generating unique identifiers, and safely renaming.
- **State:** Persists across visitor calls within a single plugin run.
- **Pre/post hooks:** Plugins can define `pre()` and `post()` methods running before/after the main traversal.
- **Composition:** Multiple plugins' visitors are merged; Babel optimizes to minimize traversal passes.

Sources: [Babel Plugin Handbook](https://github.com/jamiebuilds/babel-handbook/blob/master/translations/en/plugin-handbook.md), [Step-by-step guide for writing a custom babel transformation](https://lihautan.com/step-by-step-guide-for-writing-a-babel-transformation), [Understanding ASTs by Building Your Own Babel Plugin (SitePoint)](https://www.sitepoint.com/understanding-asts-building-babel-plugin/)

### 3.4 ESLint plugins

An ESLint plugin is an npm module exporting an object with:

- **`meta`** -- `{ name, version, namespace }`. The namespace (typically the suffix after `eslint-plugin-`) is used to qualify rules.
- **`rules`** -- Object mapping rule IDs to rule objects. Each rule has a `create(context)` function returning an AST visitor (same pattern as Babel). The `context` object provides reporting, settings, scope analysis, and source code access.
- **`processors`** -- Transform non-JS files into lintable code blocks. Have `preprocess(text, filename)` and `postprocess(messages, filename)` methods.
- **`configs`** -- Named configuration presets bundling rules, parser options, and settings.

Naming convention: `eslint-plugin-*` (unscoped) or `@scope/eslint-plugin-*` (scoped).

Sources: [ESLint: Create Plugins](https://eslint.org/docs/latest/extend/plugins), [ESLint: Configure Plugins](https://eslint.org/docs/latest/use/configure/plugins), [ESLint: Core Concepts](https://eslint.org/docs/latest/use/core-concepts/)

### 3.5 VS Code extension API

VS Code extensions live in their own **Extension Host** process (isolated from the UI process).

**Manifest (`package.json`):**
- `engines.vscode` -- minimum API version
- `main` -- entry point JS file
- `activationEvents` -- conditions triggering extension activation (e.g., `onLanguage:python`, `onCommand:myext.doThing`, `*` for startup)
- `contributes` -- static declarations of UI elements (commands, menus, keybindings, themes, languages, snippets, views, etc.)

**Lifecycle:**
- `activate(context: vscode.ExtensionContext)` -- called when activation event fires. Register commands, providers, etc. Use `context.subscriptions` for cleanup.
- `deactivate()` -- called on shutdown/disable.

**Lazy activation:** Extensions are loaded only when their activation events fire. As of VS Code 1.74, commands declared in `contributes.commands` automatically serve as activation events.

**Extension isolation:** Each extension runs in the Extension Host process, sharing it with other extensions but isolated from the main UI thread. Extensions cannot directly access the DOM.

Sources: [VS Code Extension Anatomy](https://code.visualstudio.com/api/get-started/extension-anatomy), [Activation Events](https://code.visualstudio.com/api/references/activation-events), [Contribution Points](https://code.visualstudio.com/api/references/contribution-points)

---

## 4. Lua

### 4.1 require / package system

Lua's `require(modname)` function loads modules through a four-stage searcher chain:

1. **package.loaded** -- Cache check. If the module is already loaded, return the cached value.
2. **Lua searcher** -- Searches `package.path` (a semicolon-separated list of file patterns like `./?.lua;/usr/local/share/lua/5.4/?.lua`) for a `.lua` file matching the module name.
3. **C searcher** -- Searches `package.cpath` for a shared library (`.so` on Linux, `.dll` on Windows, `.dylib` on macOS). Looks for a `luaopen_modname` function.
4. **All-in-one loader** -- For submodules like `a.b`, tries to load the root library `a` and find `luaopen_a_b` within it.

**C API for native extensions:** C modules must export a `luaopen_<modname>` function that receives a `lua_State*` pointer. The function uses the Lua C API (`lua_pushcfunction`, `lua_setfield`, etc.) to register functions and create module tables. `luaL_requiref` allows embedding modules directly into the executable without filesystem lookup.

### 4.2 LuaRocks

LuaRocks is Lua's package manager. Self-contained packages called "rocks" include the module source/binary plus version and dependency metadata. The `rockspec` file (a Lua table) declares dependencies, build instructions, and module paths.

- `luarocks install <rockname>` downloads, builds, and installs into a "tree" (structured directory of modules + metadata).
- Multiple trees are supported (system-wide, per-user, per-project).
- After installation, modules are available via standard `require()` -- LuaRocks configures `package.path` and `package.cpath`.

Sources: [Programming in Lua: require](https://www.lua.org/pil/8.2.html), [Lua Module Loading System (DeepWiki)](https://deepwiki.com/lua/lua/10.2-module-loading-system), [LuaRocks](https://luarocks.org/), [LuaRocks on GitHub](https://github.com/luarocks/luarocks), [Building Modules (lua-users wiki)](http://lua-users.org/wiki/BuildingModules)

---

## 5. Ruby

### 5.1 Gems

RubyGems is the standard package manager. A `.gemspec` file declares the gem's name, version, dependencies, and file listing. Gems are installed to a load path; `require 'gemname'` loads them. Bundler manages per-project gem sets via `Gemfile` / `Gemfile.lock`.

### 5.2 Rack middleware pattern

Rack defines a minimal interface: a Ruby object responding to `call(env)` that returns `[status, headers, body]`. Middleware wraps this: each middleware is itself a Rack app that holds a reference to the "next" app in the chain.

```ruby
class MyMiddleware
  def initialize(app)
    @app = app
  end

  def call(env)
    # before
    status, headers, body = @app.call(env)
    # after
    [status, headers, body]
  end
end
```

The middleware stack is ordered; requests flow down through `call`, responses flow back up. `Rack::Builder` and Rails' `ActionDispatch::MiddlewareStack` manage composition. This is the canonical chain-of-responsibility pattern for web request processing.

### 5.3 Rails engines

A Rails engine is a miniature Rails application (controllers, models, views, routes, migrations) mountable inside a host application. Engines inherit from `Rails::Engine`, which itself inherits from `Rails::Railtie`. They can define initializers, middleware, generators, and rake tasks. Engines share the Rack middleware stack and can add their own layers.

### 5.4 Monkey patching and refinements

Ruby's **open classes** allow any class to be reopened and modified at any time -- adding methods, overriding existing ones, or changing behavior globally. This is "monkey patching."

**Problems:** Changes are global; all code sees the modification, creating invisible coupling and potential breakage.

**Refinements (Ruby 2.0+):** Scoped monkey patching:
```ruby
module StringExtensions
  refine String do
    def shout
      upcase + "!!!"
    end
  end
end

using StringExtensions  # activates refinements in this lexical scope
"hello".shout  # => "HELLO!!!"
```

Refinements are lexically scoped -- active only from the `using` call to the end of the file or module definition. They don't leak into other code.

Sources: [Rack on GitHub](https://github.com/rack/rack), [Rails on Rack Guide](https://guides.rubyonrails.org/rails_on_rack.html), [Rails::Engine API](https://edgeapi.rubyonrails.org/classes/Rails/Engine.html), [Responsible Monkeypatching in Ruby (AppSignal)](https://blog.appsignal.com/2021/08/24/responsible-monkeypatching-in-ruby.html), [Scope the Monkey: Refinements in Ruby](https://blog.alex-miller.co/ruby/2017/07/22/scope-the-monkey.html)

---

## 6. Go

### 6.1 go plugin package (shared libraries)

The standard library `plugin` package loads `.so` files at runtime:

```go
p, _ := plugin.Open("myplugin.so")
sym, _ := p.Lookup("MyFunction")
fn := sym.(func(int) int)
result := fn(42)
```

**Severe limitations:**
- **Platform:** Linux, FreeBSD, macOS only.
- **Version coupling:** Plugin and host must be built with the exact same Go compiler version and the exact same versions of all shared packages.
- **No unloading:** Once loaded via `plugin.Open`, a plugin cannot be unloaded or freed.
- **Race detection:** Poorly supported by the Go race detector.
- **Build coordination:** In practice, application and plugins must be built together.

### 6.2 Interface-based plugin patterns

The more common Go pattern: define an interface in a shared package, compile implementations into the host binary (via blank imports and `init()` registration), or load them as shared objects and type-assert symbols to the interface.

```go
type Greeter interface {
    Greet(name string) string
}

// plugin registers via init()
func init() {
    registry.Register("english", &EnglishGreeter{})
}
```

Database drivers (`database/sql`) use this pattern: `import _ "github.com/lib/pq"` triggers `init()` which calls `sql.Register()`.

### 6.3 hashicorp/go-plugin (gRPC-based)

The production-grade Go plugin system, used in Terraform, Vault, Nomad, and Packer.

**Architecture:**
1. Host launches plugin as a subprocess.
2. Plugin and host perform a handshake (magic cookie, protocol version).
3. A single connection is established using net/rpc or gRPC.
4. The plugin implements Go interfaces; go-plugin handles serialization/deserialization transparently.
5. For net/rpc, yamux multiplexes additional channels. For gRPC, HTTP/2 provides native multiplexing.

**Key features:**
- **Process isolation:** Plugin panics cannot crash the host.
- **Language agnostic (gRPC):** Plugins can be written in any language with gRPC support.
- **Bidirectional communication:** Host can pass interface implementations back to plugins (MuxBroker enables creating new RPC connections for complex arguments like `io.Reader`).
- **Protocol versioning:** Basic version negotiation invalidates incompatible plugins.
- **Security:** TLS encryption, checksum verification of plugin binaries.
- **Logging integration:** Plugin `log.*` output streams to host with source prefix.
- **TTY attachment:** Plugins share the host's stdin for interactive programs (e.g., SSH).
- **Reattachment:** `ReattachConfig` allows host restarts while plugins stay running.

Sources: [Go plugin package](https://pkg.go.dev/plugin), [Plugins in Go (Eli Bendersky)](https://eli.thegreenplace.net/2021/plugins-in-go/), [go-plugin on GitHub](https://github.com/hashicorp/go-plugin), [RPC-based plugins in Go (Eli Bendersky)](https://eli.thegreenplace.net/2023/rpc-based-plugins-in-go/)

---

## 7. Elixir

### 7.1 Behaviours as plugin contracts

Behaviours are Elixir's primary abstraction for pluggable module implementations:

```elixir
defmodule Printer do
  @callback print(text :: String.t()) :: :ok | {:error, String.t()}
end

defmodule ConsolePrinter do
  @behaviour Printer

  @impl Printer
  def print(text), do: IO.puts(text)
end
```

**Key attributes:**
- `@callback` -- Declares required functions with type specs. Same syntax as `@spec`.
- `@macrocallback` -- For required macro implementations.
- `@optional_callbacks` -- Callbacks that implementations may omit.
- `@behaviour` -- Declares that a module adopts the contract.
- `@impl` (Elixir 1.5+) -- Documents which behaviour a function satisfies.

**Compile-time checking:** The compiler warns if required callbacks aren't implemented (checks names and arities, not types). Dialyxir can catch type mismatches.

**The `__using__` macro pattern:** Behaviours can inject default implementations:
```elixir
defmacro __using__(_) do
  quote do
    @behaviour Printer
    def print_up_to_max(text) do
      Printer.print_up_to_max(__MODULE__, text)
    end
    defoverridable print_up_to_max: 1
  end
end
```

### 7.2 Protocols

Protocols provide type-based dispatch (similar to Haskell typeclasses or Clojure protocols):
```elixir
defprotocol Size do
  def size(data)
end

defimpl Size, for: BitString do
  def size(string), do: byte_size(string)
end
```

Behaviours plug in modules; protocols plug in data types. Both provide extension points but along different axes.

### 7.3 Macros for DSLs

Elixir macros operate on AST at compile time, enabling DSLs like Phoenix router, Ecto schema, and ExUnit test definitions. Combined with behaviours, they provide both syntactic extension and interface contracts.

**Runtime module selection** (no auto-discovery): via direct invocation, passing modules as arguments, application config, or `function_exported?/3` predicates.

Sources: [Extensibility in Elixir Using Behaviours (Stacktrace)](https://stacktracehq.com/blog/extensibility-in-elixir-using-behaviours/), [Behaviours (Elixir School)](https://elixirschool.com/en/lessons/advanced/behaviours), [Differences Between Elixir's Protocols and Behaviours](https://samuelmullen.com/articles/differences-between-elixirs-protocols-and-behaviours), [Writing extensible Elixir with Behaviours](https://www.djm.org.uk/posts/writing-extensible-elixir-with-behaviours-adapters-pluggable-backends/)

---

## 8. Vim / Neovim

### 8.1 VimL/Lua plugin system

**Vim's directory-based plugin loading:**

| Directory | When loaded | Scope |
|-----------|------------|-------|
| `plugin/` | Every Vim startup | Global |
| `ftplugin/` | When filetype is set | Filetype-specific |
| `autoload/` | On first call to `autoload#func()` | Lazy/on-demand |
| `syntax/` | When filetype is set | Syntax highlighting |
| `indent/` | When filetype is set | Indentation rules |

The `runtimepath` option lists directories to search. Plugin managers (vim-plug, lazy.nvim, packer) manipulate `runtimepath` to add plugin directories and control load ordering.

**Autoload:** Vim's lazy loading mechanism. When calling `somefile#SomeFunc()`, Vim looks for `autoload/somefile.vim` and sources it on first use. This avoids loading all plugin code at startup.

### 8.2 Neovim-specific extensions

Neovim adds first-class Lua support:

- **Lua modules:** Files under `lua/` in runtimepath are loadable via `require()`.
- **`vim.*` API:** Lua code accesses Neovim's full API through `vim.api.*`, `vim.fn.*`, `vim.lsp.*`, `vim.diagnostic.*`, etc.
- **Remote plugins:** External processes communicate via msgpack-RPC over stdin/stdout. Can be written in any language with a msgpack-RPC client.
- **`vim.filetype.add()`:** Programmatic filetype rule registration.
- **`setup(opts)` pattern:** The community convention for plugin configuration -- plugins export a `setup` function that accepts a config table:
  ```lua
  require('telescope').setup({ defaults = { ... } })
  ```

### 8.3 Autocommands as hooks

Autocommands are Neovim's event hook mechanism. They execute callbacks in response to editor events:

```lua
vim.api.nvim_create_autocmd("BufWritePre", {
  pattern = "*.lua",
  callback = function() vim.lsp.buf.format() end,
})
```

**Key event categories:** file operations (`BufReadPost`, `BufWritePre`), buffer lifecycle (`BufEnter`, `BufLeave`, `BufDelete`), language features (`FileType`, `LspAttach`), text changes (`TextChanged`, `InsertEnter`), window/tab events.

**Autocommand groups** provide namespace isolation and batch management -- `autocmd!` within a group clears all autocommands in that group, enabling safe re-sourcing.

Sources: [Plugin Layout in the Dark Ages (Learn Vimscript the Hard Way)](https://learnvimscriptthehardway.stevelosh.com/chapters/42.html), [Autoloading (Learn Vimscript the Hard Way)](https://learnvimscriptthehardway.stevelosh.com/chapters/53.html), [Neovim Lua-plugin docs](https://neovim.io/doc/user/lua-plugin.html), [Neovim Extension Mechanisms (DeepWiki)](https://deepwiki.com/neovim/neovim/5-extension-mechanisms), [Autocommands and Events (DeepWiki)](https://deepwiki.com/neovim/neovim/5.4-autocommands-and-events)

---

## 9. Hook Systems (Cross-Cutting)

### 9.1 Git hooks

Filesystem-based: executable scripts in `.git/hooks/` named after the hook event. Any language works. Non-zero exit codes from pre-* hooks abort the operation.

**Client-side hooks:**

| Hook | Trigger | Parameters | Can abort? |
|------|---------|-----------|------------|
| `pre-commit` | Before commit message editor | None | Yes |
| `prepare-commit-msg` | After default message created, before editor | File path, commit type, SHA | Yes |
| `commit-msg` | After message entered | Temp file path | Yes |
| `post-commit` | After commit completes | None | No |
| `pre-rebase` | Before rebase | None | Yes |
| `post-rewrite` | After commit-rewriting commands | Command name; rewrites on stdin | No |
| `post-checkout` | After checkout | None | No |
| `post-merge` | After merge | None | No |
| `pre-push` | During push, before transfer | Remote name/URL; refs on stdin | Yes |
| `pre-auto-gc` | Before garbage collection | None | Yes |

**Server-side hooks:**

| Hook | Trigger | Granularity |
|------|---------|-------------|
| `pre-receive` | First hook on push | All refs (reject all or none) |
| `update` | Per-branch during push | Per-ref (can reject individually) |
| `post-receive` | After push completes | All refs (notification only) |

**Stdin format for receive hooks:** `<old-sha> <new-sha> <ref-name>` per line.

**Limitation:** Client-side hooks are not copied on clone. For enforcement, use server-side hooks or tools like [pre-commit](https://pre-commit.com/) which manage hooks via config files.

Sources: [Git Book: Git Hooks](https://git-scm.com/book/en/v2/Customizing-Git-Git-Hooks), [githooks documentation](https://git-scm.com/docs/githooks), [Atlassian Git Hooks Tutorial](https://www.atlassian.com/git/tutorials/git-hooks)

### 9.2 WordPress hooks (actions and filters)

The canonical web plugin hook system, with two types:

**Actions:** Side-effect hooks. Callbacks do something (output, database write) and return nothing.
```php
add_action('save_post', 'my_callback', 10, 2);
function my_callback($post_id, $post) { /* ... */ }
do_action('save_post', $post_id, $post);
```

**Filters:** Data transformation hooks. Callbacks receive a value, modify it, and must return it.
```php
add_filter('the_title', 'my_filter', 10, 1);
function my_filter($title) { return 'The ' . $title; }
$title = apply_filters('the_title', $raw_title);
```

**Priority system:** Integer priority (default 10); lower numbers execute first. Within the same priority, callbacks execute in registration order.

**Additional functions:** `remove_action()`, `remove_filter()`, `has_action()`, `has_filter()`, `did_action()`, `current_filter()`.

**Internal architecture:** Both actions and filters are stored in the global `$wp_filter` array. Actions are internally implemented as filters that ignore return values. WordPress Core defines thousands of hooks across 700+ locations.

Sources: [WordPress Plugin Handbook: Hooks](https://developer.wordpress.org/plugins/hooks/), [WordPress Plugin Handbook: Actions](https://developer.wordpress.org/plugins/hooks/actions/), [WordPress Plugin Handbook: Filters](https://developer.wordpress.org/plugins/hooks/filters/), [WordPress Hooks Bootcamp (Kinsta)](https://kinsta.com/blog/wordpress-hooks/)

### 9.3 Emacs hooks and advice system

**Hook variables:** Lists of functions called at specific events. Normal hooks (suffix `-hook`) take no arguments; abnormal hooks may take arguments and use return values.

```elisp
(add-hook 'python-mode-hook #'my-python-setup)
(remove-hook 'python-mode-hook #'my-python-setup)
```

**Ordering:** The `depth` argument to `add-hook` (range -100 to 100) controls position; higher values run later. Buffer-local hooks (via the `local` argument) allow per-buffer customization.

**Advice system (nadvice.el):** Modifies existing functions without redefining them:

```elisp
(advice-add 'find-file :before #'my-before-advice)
(advice-add 'find-file :around #'my-around-advice)
(define-advice find-file (:after (&rest args) my-after-advice)
  (message "File opened"))
```

**Advice combinators:**

| Combinator | Behavior |
|------------|----------|
| `:before` | Run advice before original; same args; advice return value ignored |
| `:after` | Run advice after original; same args; advice return value ignored |
| `:around` | Advice receives original function + args; controls whether/when to call original |
| `:override` | Completely replace original; advice receives same args |
| `:before-while` | Run advice before; if it returns nil, skip original |
| `:before-until` | Run advice before; if it returns non-nil, skip original and use advice's return |
| `:after-while` | Run original; if it returns non-nil, also run advice |
| `:after-until` | Run original; if it returns nil, also run advice |
| `:filter-args` | Advice receives args, returns modified args passed to original |
| `:filter-return` | Original runs first; advice receives its return value and returns modified result |

**Historical note:** The older `defadvice` system (1993) is more complex; `advice-add`/nadvice.el (2012) is simpler and recommended for new code.

Sources: [GNU Emacs Manual: Hooks](https://www.gnu.org/software/emacs/manual/html_node/elisp/Hooks.html), [Setting Hooks](https://www.gnu.org/software/emacs/manual/html_node/elisp/Setting-Hooks.html), [Advising Functions](https://www.gnu.org/software/emacs/manual/html_node/elisp/Advising-Functions.html), [Advising Named Functions](https://www.gnu.org/software/emacs/manual/html_node/elisp/Advising-Named-Functions.html), [Emacs Lisp: Advice Combinators](https://scripter.co/emacs-lisp-advice-combinators/)

### 9.4 Event systems vs. hook systems

**Observer pattern:** Observers register with a subject; when the subject changes state, it notifies all observers. The classic GoF pattern.

**Pub/sub:** Decouples publishers from subscribers via a message broker or event bus. Publishers emit named events; subscribers bind to event names. Events are typically string-keyed and dynamic.

**Signal/slot (Qt):** Each event type is a specific `signal` declared on a class. Slots are member functions connected to signals. The Meta Object Compiler (MOC) generates the infrastructure. Advantages over raw observer: type-safe connections, automatic cleanup on object destruction, signals can connect to signals for chaining.

**How hooks differ from events:**
- Hooks are typically **synchronous** and **ordered** -- the host defines exactly when each hook fires in its execution flow, and hook results can influence the host's behavior (e.g., bail hooks, filter hooks).
- Events are typically **asynchronous** and **unordered** -- the emitter fires and forgets; listeners run independently.
- Hooks often have **return value semantics** (waterfall, firstresult, filter) while events typically don't.

Sources: [Signal-slot vs observer (Qt Forum)](https://forum.qt.io/topic/74425/signal-slot-mechanism-vs-observer-pattern-good-practice), [Signals and slots (Wikipedia)](https://en.wikipedia.org/wiki/Signals_and_slots), [Signal/Slot design pattern](https://signalslot.readthedocs.io/en/latest/pattern.html)

### 9.5 Middleware patterns

**Express.js:** Middleware functions `(req, res, next)` form a linear pipeline. Each can modify `req`/`res`, call `next()` to continue, or end the cycle. Error-handling middleware takes four arguments `(err, req, res, next)`.

**Rack (Ruby):** Each middleware wraps an inner app, calls `@app.call(env)`, and can modify the request (env hash) before and the response `[status, headers, body]` after. Stack is ordered; composition is explicit via `Rack::Builder` or `use` in Rails.

**Tower (Rust):** The `Service` trait (`poll_ready` + `call`) represents an async request->response function. The `Layer` trait wraps one Service into another, adding behavior before/after the inner call. `ServiceBuilder` chains layers. `poll_ready` enables backpressure -- services signal readiness before accepting requests. Common middleware: timeout, rate limit, retry, load balancing, buffering.

**Common thread:** All three implement the chain-of-responsibility pattern with request/response pipelines. The key architectural choice is whether middleware is a function taking `(req, next)` (Express, Rack) or a service wrapper (Tower, Finagle).

Sources: [Understanding the Middleware Pattern in Express.js](https://dzone.com/articles/understanding-middleware-pattern-in-expressjs), [tower docs](https://docs.rs/tower), [tower middleware guide](https://github.com/tower-rs/tower/blob/master/guides/building-a-middleware-from-scratch.md), [Adding middleware support to Rust reqwest (TrueLayer)](https://truelayer.com/blog/engineering/adding-middleware-support-to-rust-reqwest/)

### 9.6 Lifecycle hooks

**React:** `useEffect` (replaces `componentDidMount`/`componentDidUpdate`/`componentWillUnmount`), `useState`, `useCallback`, `useRef`, `useMemo`. Hooks run after render, in declaration order. Cleanup functions run before next effect or on unmount.

**Vue:** `onMounted`, `onUpdated`, `onUnmounted`, `onBeforeMount`, `onBeforeUpdate`, `onBeforeUnmount`. The Composition API groups related lifecycle logic together rather than splitting across lifecycle methods.

**How framework hooks differ from plugin hooks:** Framework lifecycle hooks are fixed extension points in a component's lifecycle -- they are called by the framework at specific moments. Plugin hooks are extension points in a host program's execution flow -- they are called by the host at specific program positions. Both use the same underlying mechanism (registered callbacks at named points) but differ in scope: lifecycle hooks are per-component-instance, plugin hooks are per-application.

Sources: [Vue.js Lifecycle Hooks](https://vuejs.org/guide/essentials/lifecycle.html), [React Hooks documentation](https://react.dev/reference/react)
