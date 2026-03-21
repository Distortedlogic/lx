# Module Systems Across Programming Languages

Research landscape for lx module system design.

## 1. Python

### Import Machinery

Python's import system uses a finder/loader architecture defined by PEP 302 (2002) and refined by PEP 451 (ModuleSpec, Python 3.4+).

**Five-stage import process:**

1. **Cache lookup** -- `sys.modules` checked first. If the module exists there (even partially initialized), it is returned immediately. This is writable: deleting an entry forces reimport, setting it to `None` forces `ModuleNotFoundError`.
2. **Meta path finder search** -- Each finder in `sys.meta_path` is called via `finder.find_spec(fullname, path, target)`. Python ships three default meta path finders: built-in module finder, frozen module finder, and `PathFinder`. For `import foo.bar.baz`, three separate `find_spec` calls happen (one per dotted component), each receiving the parent's `__path__`.
3. **Path-based finder** -- The `PathFinder` meta path finder searches `sys.path` entries. `sys.path_hooks` contains callables that create path entry finders for each entry. Results are cached in `sys.path_importer_cache`.
4. **ModuleSpec creation** -- Finders return `ModuleSpec(name, loader, origin, submodule_search_locations)` objects that encapsulate all import metadata.
5. **Loading and execution** -- The loader's `create_module(spec)` is called (returns `None` for default `ModuleType`), then the module is added to `sys.modules` *before* `exec_module(module)` runs the code. On failure, the module is removed from `sys.modules`.

**Key interfaces:**

```python
class MetaPathFinder:
    def find_spec(self, fullname, path, target=None) -> ModuleSpec | None

class Loader:
    def create_module(self, spec) -> module | None
    def exec_module(self, module) -> None
```

Sources: [Python import system docs](https://docs.python.org/3/reference/import.html), [importlib docs](https://docs.python.org/3/library/importlib.html), [PEP 451](https://peps.python.org/pep-0451/)

### Packages and `__init__.py`

- **Regular packages**: Directory with `__init__.py`. Importing the package executes `__init__.py`. Any module with a `__path__` attribute is a package.
- **Namespace packages** (PEP 420): No `__init__.py` required. Multiple directory portions across different filesystem locations contribute to a single namespace. `__path__` uses a custom auto-refreshing iterable.

### `__all__`

Module-level list defining the public API for `from module import *`. Without it, all names not starting with `_` are exported by star-import.

### Relative Imports

```python
from .sibling import func        # current package
from ..parent_sibling import cls # parent package
```

Relative imports require `from ... import` syntax; `import .foo` is illegal.

### Import Hooks (PEP 302)

Two hook points:
- **Meta path hooks**: Installed in `sys.meta_path`, called before any other import processing.
- **Path hooks**: Installed in `sys.path_hooks`, called to create finders for specific path entries (e.g., zip files).

### Lazy Imports (PEP 690)

PEP 690 proposed interpreter-level lazy imports (rejected, but instructive design):
- On `import foo`, a lazy import placeholder is stored in the module namespace dict.
- A flag (`dk_lazy_imports`) on the dict signals lazy objects may be present.
- First attribute access transparently triggers the real import and replaces the placeholder.
- Exceptions: `try/except` blocks, star imports, `__import__()`, and function/class body imports remain eager.
- Reference implementation showed 40-70% startup time improvement and up to 40% memory reduction.
- Python stdlib already has `importlib.util.LazyLoader` for opt-in lazy module objects.

PEP 810 proposes explicit lazy imports as a successor approach.

Sources: [PEP 690](https://peps.python.org/pep-0690/), [PEP 810](https://peps.python.org/pep-0810/)

### Circular Import Handling

Python permits circular imports via partial module objects. The module is added to `sys.modules` before its code executes:

1. `import A` -- creates `sys.modules['A']` (empty), begins executing A.
2. A's code hits `import B` -- creates `sys.modules['B']` (empty), begins executing B.
3. B's code hits `import A` -- finds partial A in `sys.modules`, returns it.
4. B completes. A continues and completes.

Consequence: If B tries `from A import some_function` and that function hasn't been defined yet during A's partial execution, it raises `ImportError`. The standard workaround is to move the import inside the function body where it's used.

---

## 2. Rust

### Module Tree

Rust's module system is a compile-time tree rooted at the crate root (`lib.rs` or `main.rs`). Modules are declared with `mod`, not discovered:

```rust
mod foo;        // looks for foo.rs or foo/mod.rs
mod bar {       // inline module
    mod baz;    // looks for bar/baz.rs
}
```

### Edition Changes (2015 vs 2018+)

| Aspect | 2015 | 2018+ |
|--------|------|-------|
| External crates | `extern crate foo;` required, only in scope at crate root | Automatic from `Cargo.toml`, in scope everywhere |
| `use` paths | Always from crate root | Same as expression paths (relative to current scope) |
| `crate::` prefix | Not available | Unambiguous reference to crate root |
| `::foo` | Ambiguous (crate root or external) | External crate only |
| Module files | `foo/mod.rs` required for submodules | `foo.rs` + `foo/` directory can coexist |
| Macro imports | `#[macro_use] extern crate` | `use crate_name::macro_name;` |

Source: [Rust Edition Guide: Path Changes](https://doc.rust-lang.org/edition-guide/rust-2018/path-changes.html), [RFC 2126](https://rust-lang.github.io/rfcs/2126-path-clarity.html)

### `use` and Path Resolution

```rust
use std::collections::HashMap;     // external crate
use crate::models::User;           // crate root
use super::sibling_mod::Thing;     // parent module
use self::child_mod::Widget;       // current module
```

Glob imports: `use std::io::prelude::*;`

Re-exports: `pub use self::internal::PublicApi;` -- allows external access through a shorter path while keeping the internal module private.

### Visibility System

Five levels, all enforced at compile time:

| Modifier | Scope |
|----------|-------|
| (none) | Private to current module + descendants |
| `pub` | Unrestricted (accessible if all ancestors are accessible) |
| `pub(crate)` | Current crate only |
| `pub(super)` | Parent module |
| `pub(in crate::path)` | Specified ancestor module (2018+: must start with `crate`, `self`, or `super`) |

**Privacy algorithm** -- Two rules:
1. Public items accessible from module M if all ancestor modules are accessible from M.
2. Private items accessible by the current module and its descendants.

Exceptions: `pub` trait associated items are always public. `pub` enum variants are always public.

Source: [Rust Reference: Visibility](https://doc.rust-lang.org/reference/visibility-and-privacy.html)

### Orphan Rules

A trait impl is only allowed if either the trait or the type is defined in the current crate. This prevents conflicting impls across crates. The "orphan rule" ensures coherence -- no two crates can provide the same impl.

### Circular Dependencies

Rust modules within a crate can freely reference each other. The compiler resolves all types and names across the module tree before codegen. Circular *crate* dependencies are prohibited by Cargo (the dependency graph must be a DAG), though dev-dependency cycles are permitted for non-workspace members.

---

## 3. JavaScript / Node.js

### CommonJS (CJS)

Synchronous, runtime module system. Used by Node.js historically.

**`require()` algorithm** (pseudocode from Node.js docs):

```
require(X) from module at path Y:
1. If X is a core module, return it.
2. If X begins with '/', set Y to filesystem root.
3. If X begins with './', '/', or '../':
   a. LOAD_AS_FILE(Y + X)
   b. LOAD_AS_DIRECTORY(Y + X)
4. If X begins with '#': LOAD_PACKAGE_IMPORTS
5. LOAD_PACKAGE_SELF
6. LOAD_NODE_MODULES(X, dirname(Y))
7. THROW "not found"
```

**node_modules climbing** -- `LOAD_NODE_MODULES` walks up the directory tree:
```
/home/user/projects/app/src/node_modules/X
/home/user/projects/app/node_modules/X
/home/user/projects/node_modules/X
/home/user/node_modules/X
/home/node_modules/X
/node_modules/X
```

**Caching** -- Modules cached in `require.cache` keyed by resolved filename. Code executes once. `delete require.cache[require.resolve('./mod')]` forces reload.

**Module wrapper** -- Every module is wrapped in:
```javascript
(function(exports, require, module, __filename, __dirname) {
  // module code
});
```

This provides scoping and the `exports`/`module`/`require` bindings.

**Circular dependencies** -- Returns the *partially evaluated* `exports` object. If A requires B which requires A, B gets A's `exports` as they exist at that point (incomplete). Properties added to `exports` after the circular require become visible because it's the same object reference.

Source: [Node.js CommonJS docs](https://nodejs.org/api/modules.html)

### ECMAScript Modules (ESM)

Static, asynchronous module system. The web standard.

**Three phases** (per the spec):
1. **Parsing** -- Static analysis of `import`/`export` declarations. Dependency graph built before any code runs.
2. **Instantiation** -- Memory allocated for exports. Import bindings linked to export bindings (live bindings).
3. **Evaluation** -- Module code executes top-to-bottom.

**Live bindings** -- Imports are read-only references to the exporter's binding, not copies. If the exporting module changes the value, the importing module sees the new value. This is fundamental to how ESM handles circular dependencies.

**Circular dependencies in ESM** -- Because parsing and instantiation happen before evaluation, the full module graph is known. During evaluation, a circular import may find a variable uninitialized (TDZ). But by the time the variable is actually *used* (e.g., a function is called), evaluation has typically completed and the binding is populated. This makes many practical circular patterns work that would fail in CJS.

**Top-level await** -- ES modules can use `await` at the top level. A module that uses top-level await blocks evaluation of any module that imports it until the promise resolves.

**Dynamic `import()`** -- Returns a promise. Works in both CJS and ESM contexts. Enables code splitting and conditional loading.

Source: [Node.js ESM docs](https://nodejs.org/api/esm.html)

### Dual CJS/ESM Packages

Node.js supports dual packages via `package.json`:

```json
{
  "exports": {
    ".": {
      "import": "./index.mjs",
      "require": "./index.cjs"
    }
  }
}
```

**Conditional exports** resolve based on the import context. Conditions include `"import"`, `"require"`, `"node"`, `"default"`, and custom conditions.

The `"type"` field in `package.json` sets the default: `"module"` means `.js` files are ESM, `"commonjs"` means CJS. File extensions `.mjs` and `.cjs` override regardless of `"type"`.

### Import Maps

Browser-side mechanism (and increasingly in runtimes) that maps bare specifiers to URLs:

```json
{
  "imports": {
    "lodash": "https://cdn.example.com/lodash@4.17.21/lodash.mjs"
  }
}
```

---

## 4. Go

### Package-per-Directory

Every directory is exactly one package. The package name is declared at the top of each `.go` file. All files in a directory must declare the same package name. Convention: package name matches directory name.

```go
package http  // in net/http/*.go
```

Import paths are directory paths relative to the module root:

```go
import "net/http"
import "example.com/mylib/internal/helpers"
```

### GOPATH Era vs Go Modules

**GOPATH** (pre-2018): All Go code lived under `$GOPATH/src`. Dependencies resolved by directory structure. No versioning -- `go get` always fetched HEAD. No reproducible builds.

**Go modules** (Go 1.11+, default since 1.16): `go.mod` declares module path and dependencies with versions. Code can live anywhere on the filesystem. Minimum version selection for resolution. `go.sum` for integrity verification.

### Internal Packages

A package with `internal` in its import path can only be imported by packages rooted at the parent of `internal/`:

```
example.com/mylib/internal/helpers    -- can be imported by example.com/mylib/**
                                      -- cannot be imported by example.com/other/**
```

Enforced at compile time. No runtime overhead. This is Go's primary encapsulation mechanism beyond exported/unexported identifiers (capitalization).

Source: [Go Modules Reference](https://go.dev/ref/mod)

### Circular Dependencies

Go prohibits circular package imports. The compiler rejects any import cycle. This is a deliberate design choice that forces clear dependency hierarchies. The only escape hatch is interfaces -- package A can define an interface that package B implements, without A importing B.

### Visibility

Go uses capitalization as the sole visibility mechanism:
- `Exported` (uppercase first letter) -- accessible from other packages
- `unexported` (lowercase) -- package-private

No module-private or crate-private equivalent. `internal/` packages provide the coarser-grained restriction.

---

## 5. Elixir

### Four Module Directives

Elixir provides three directives and one macro for module interaction, all lexically scoped:

**`alias`** -- Creates a short name for a module:
```elixir
alias Math.List, as: List   # refer to Math.List as List
alias Math.List              # same, inferred alias
```

**`require`** -- Makes macros from a module available. Must be called before using the module's macros. Ensures the module is compiled first.

**`import`** -- Brings functions/macros into the current scope without qualification:
```elixir
import List, only: [flatten: 1]    # selective import
import List, except: [flatten: 1]  # exclusion import
```

**`use`** -- Calls `__using__/1` callback on the target module, which can inject code (functions, macros, aliases, imports) into the caller:
```elixir
use GenServer  # injects GenServer callbacks and default implementations
```

Source: [Elixir docs: alias, require, import, use](https://hexdocs.pm/elixir/alias-require-and-import.html)

### Protocol Dispatch

Protocols provide ad-hoc polymorphism (similar to Rust traits or Haskell type classes):

```elixir
defprotocol Size do
  def size(data)
end

defimpl Size, for: BitString do
  def size(string), do: byte_size(string)
end

defimpl Size, for: Map do
  def size(map), do: map_size(map)
end
```

Dispatch is based on the runtime type of the first argument. `@fallback_to_any true` enables a default `Any` implementation. Protocol consolidation at compile time optimizes dispatch to direct function calls.

Source: [Elixir Protocols](https://hexdocs.pm/elixir/protocols.html)

### Behaviour Callbacks

Behaviours define a set of function signatures a module must implement (like interfaces/traits):

```elixir
defmodule MyBehaviour do
  @callback my_function(arg :: term()) :: {:ok, term()} | {:error, term()}
end

defmodule MyImpl do
  @behaviour MyBehaviour
  def my_function(arg), do: {:ok, arg}
end
```

---

## 6. Lua

### `require` and Searcher Chain

Lua's module system is minimal and extensible. `require(modname)` follows this process:

1. Check `package.loaded[modname]` -- if non-false value, return it (cache).
2. Iterate through `package.searchers` table, calling each searcher function with `modname`.
3. First searcher to return a loader function wins. The loader is called, its return value stored in `package.loaded[modname]`.

**Four default searchers** (Lua 5.3/5.4):

| # | Searcher | Description |
|---|----------|-------------|
| 1 | Preload | Looks up `package.preload[modname]` for a preregistered loader function. Returns `":preload:"` as extra value. |
| 2 | Lua loader | Searches `package.path` for a `.lua` file. Template pattern uses `?` as placeholder: `"./?.lua;/usr/share/lua/5.4/?.lua"` |
| 3 | C loader | Searches `package.cpath` for a shared library (`.so`/`.dll`). Looks for `luaopen_modname` function. |
| 4 | All-in-one | For dotted names like `a.b.c`, searches C path for root module `a` and looks for `luaopen_a_b_c` within it. |

**Path templates** -- `package.path` contains semicolon-separated patterns. `?` is replaced with the module name (dots become directory separators). `package.searchpath(name, path)` implements the actual file search.

Source: [Lua 5.4 Reference Manual](https://www.lua.org/manual/5.4/), [DeepWiki: Lua Module Loading](https://deepwiki.com/lua/lua/10.2-module-loading-system)

### Extensibility

The searcher chain is fully programmable. Custom searchers can be added to `package.searchers` to load modules from databases, network, or generate them dynamically. This is Lua's equivalent of Python's import hooks.

### Module Caching

`package.loaded` is a simple table. Setting `package.loaded[modname] = nil` forces reimport. Modules return a value from their loader function (typically a table), which becomes the cached value.

---

## 7. Haskell

### Module Exports

```haskell
module Data.Map
  ( Map        -- type only, constructors hidden (abstract type)
  , empty      -- specific function
  , insert
  , lookup
  , module Data.Map.Internal  -- re-export everything from another module
  ) where
```

If the export list is omitted, all top-level bindings are exported.

### Import Variants

```haskell
import Data.Map                          -- all exports, unqualified
import qualified Data.Map                -- all exports, qualified only (Data.Map.lookup)
import qualified Data.Map as Map         -- qualified with alias (Map.lookup)
import Data.Map (empty, insert)          -- selective import
import Data.Map hiding (lookup, insert)  -- import everything except
import qualified Data.Map as Map (empty) -- qualified + selective
```

Source: [HaskellWiki: Import](https://wiki.haskell.org/Import), [Haskell 2010 Report: Modules](https://www.haskell.org/onlinereport/haskell2010/haskellch5.html)

### Re-exports

The `module M` form in an export list re-exports everything imported from M:

```haskell
module MyLib (module Data.Map, myFunction) where
import Data.Map
myFunction = ...
```

### Package Dependencies (Cabal / Stack)

Dependencies declared in `.cabal` files under `build-depends`:

```cabal
library
  exposed-modules: MyLib
  other-modules:   MyLib.Internal
  build-depends:   base >=4.7 && <5, containers, text
```

`exposed-modules` -- visible to downstream packages. `other-modules` -- private implementation modules.

**Cabal** is the build system and package format. **Stack** is an alternative build tool that uses Cabal packages but adds curated package sets (Stackage snapshots) for reproducible builds.

### Circular Module Dependencies

Haskell allows circular module imports via `.hs-boot` files that provide forward declarations. The compiler uses these to break the cycle. In practice, circular imports are rare and discouraged.

---

## Cross-Cutting Analysis

### Circular Dependency Handling

| Language | Approach | Mechanism |
|----------|----------|-----------|
| Python | Allowed (partial objects) | Module added to `sys.modules` before execution. Circular import gets the partially-initialized module object. |
| Rust | Allowed within crate, prohibited across crates | Compiler resolves all names across module tree before codegen. Cargo enforces DAG for crate dependencies. |
| JavaScript (CJS) | Allowed (partial exports) | `require()` returns the partially-evaluated `exports` object reference. |
| JavaScript (ESM) | Allowed (live bindings) | Three-phase loading separates graph construction from evaluation. Imports are live references that populate after evaluation completes. |
| Go | Prohibited | Compiler rejects import cycles. Forces clear dependency hierarchies. |
| Elixir | Prohibited at compile time | Compilation order determined by dependency graph. Cycles cause compiler error. |
| Lua | Allowed (via `package.loaded`) | `package.loaded[modname]` set during loading. Circular require gets the in-progress return value (usually `true` if not yet returned). |
| Haskell | Allowed via `.hs-boot` | Forward declaration files break the cycle for the compiler. |

### Visibility / Encapsulation Spectrum

| Level | Languages |
|-------|-----------|
| No restrictions | Lua (everything returned from module is public) |
| Binary (exported/unexported) | Go (capitalization), Python (`_` prefix convention + `__all__`) |
| Module-level granular | Rust (`pub`, `pub(crate)`, `pub(super)`, `pub(in path)`), Haskell (export lists with constructor control) |
| Package-level | Go (`internal/` packages), Elixir (no direct equivalent, uses `@moduledoc false`) |

### Module Resolution Algorithms

| Language | Strategy |
|----------|----------|
| Python | `sys.meta_path` finder chain, then `sys.path` search with path hooks |
| Rust | Compile-time mod tree, filesystem convention (`foo.rs` or `foo/mod.rs`) |
| Node.js CJS | File/directory loading, then node_modules directory climbing from current to root |
| Node.js ESM | Package.json `exports` field, conditional exports, file extensions required |
| Go | Module path = import path, resolved via module proxy or VCS |
| Elixir | Module name is an atom, resolved at compile time. File path is conventional but not enforced. |
| Lua | `package.searchers` chain: preload table, path template search, C library search, all-in-one |
| Haskell | Module name maps to file path. Cabal specifies which modules are exposed. |

### Lazy vs Eager Loading

| Language | Default | Lazy Options |
|----------|---------|-------------|
| Python | Eager (import executes module code) | `importlib.util.LazyLoader`, PEP 690 (rejected), PEP 810 (proposed) |
| Rust | N/A (compile-time, no runtime loading) | N/A |
| JavaScript CJS | Eager (synchronous execution on `require()`) | `require()` inside function bodies |
| JavaScript ESM | Eager (evaluation after full graph construction) | `import()` dynamic import returns Promise |
| Go | N/A (compile-time) | Plugins (`plugin.Open`) for dynamic loading |
| Elixir | Eager (compiled) | Code loading via `:code.load_file` at runtime |
| Lua | Lazy by default (`require` on first call, cached thereafter) | Inherent -- modules load only when `require`'d |
| Haskell | Lazy language, but module loading is eager at program start | Template Haskell for compile-time code generation |

### Module as Value / First-Class Modules

**OCaml** is the canonical example. First-class modules allow packing a module into a value:
```ocaml
let m = (module MyModule : S)   (* pack module as value *)
let module M = (val m : S)      (* unpack value as module *)
```
This bridges the language's stratification between the core language (values, expressions) and the module language (types, signatures). Modules can be passed to functions, stored in data structures, and returned from functions. Functors (module-level functions) take modules as parameters and return modules.

Source: [OCaml Manual: First-Class Modules](https://ocaml.org/manual/5.4/firstclassmodules.html), [Real World OCaml: First-Class Modules](https://dev.realworldocaml.org/first-class-modules.html)

**Elixir** modules are atoms at runtime and can be stored in variables and passed around:
```elixir
mod = String
mod.upcase("hello")  # "HELLO"
```

**Lua** modules are just tables -- inherently first-class values.

**Python** module objects are first-class values assignable to variables, passable to functions, and storable in data structures.

### Namespacing: Flat vs Hierarchical

| Approach | Languages | Details |
|----------|-----------|---------|
| Flat | Go, Lua | Go: package name is the short identifier. Lua: module name is a string key in `package.loaded`. |
| Hierarchical | Python, Rust, Haskell, Elixir | Dotted/path-based names forming a tree. |
| Hybrid | JavaScript | npm scopes (`@org/pkg`) + file paths. ESM uses URL-like resolution. |

### Aliasing and Re-exports

| Language | Aliasing | Re-exports |
|----------|----------|-----------|
| Python | `import numpy as np` | `from .internal import public_func` in `__init__.py` |
| Rust | `use std::io::Result as IoResult` | `pub use internal::Thing;` |
| JavaScript | `import { foo as bar }` | `export { thing } from './other'` |
| Go | `import alias "full/path"` | Not supported (packages must be imported directly) |
| Elixir | `alias Long.Module.Name, as: Short` | Not built-in; `use` macro can inject aliases |
| Haskell | `import qualified Data.Map as Map` | `module Lib (module Data.Map) where` |
| Lua | `local M = require("module")` | Return a table that includes other modules' functions |
