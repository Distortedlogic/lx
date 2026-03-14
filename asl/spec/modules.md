# Modules

File = module. Every `.lx` file is a module. No explicit module declarations.

## Importing

`use` imports a module. Path separators are `/`.

```
use std/fs                 -- import entire module
use std/net/http           -- nested module
use std/fs {read write}    -- selective import
use ./util                 -- relative path (same directory)
use ../shared/types        -- relative path (parent directory)
```

Aliasing with `:`:

```
use std/net/http : h       -- h.get, h.post, etc.
use std/fs : f             -- f.read, f.write, etc.
```

## Exporting

`+` at column 0 marks a binding as exported:

```
+add = (x y) x + y        -- exported
helper = (x) x * x        -- private
+Point = {x: Float  y: Float}  -- exported type
```

Everything without `+` is module-private. There is no default export — all exports are named.

## Module Access

Imported modules are accessed with `.`:

```
use std/fs
content = fs.read "file.txt" ^
```

Selective imports bring names into scope directly:

```
use std/fs {read write}
content = read "file.txt" ^
write "out.txt" content ^
```

## Package Management

Stdlib: `use std/...` — always available, no configuration needed. This includes core modules (fs, http, json) and agent modules (agent, mcp, ctx, md, cron).

Local: `use ./path` and `use ../path` — relative to the current file.

External deps: Declared in `pkg.lx` at the project root (like `Cargo.toml` but in lx syntax). Content-addressed caching for reproducibility. Lock file tracks exact versions.

```
-- pkg.lx
deps = {
  http_client: {url: "https://pkg.lx/http/v2"  hash: "sha256:abc123"}
  csv: {url: "https://pkg.lx/csv/v1"  hash: "sha256:def456"}
}
```

External deps are imported by their declared name:

```
use http_client
use csv
```

## Variant Constructor Scoping

When a module exports a tagged union type, importing that module (any form) makes the variant constructors available as bare names. `use ./types` where `types` exports `Color = | Red | Green | Blue` brings `Red`, `Green`, `Blue` into scope as constructors. This is required for pattern matching and construction to work without verbose qualified names. If two imports define the same variant name, use module-qualified access (`types.Red`).

## Import Conflicts

Selective imports that introduce the same name are a compile error:

```
use ./a {foo}
use ./b {foo}     -- ERROR: `foo` already imported from ./a
```

Fix with aliasing:

```
use ./a : a
use ./b : b
a.foo             -- unambiguous
b.foo             -- unambiguous
```

Whole-module imports never conflict because they're accessed via the module name (`a.foo` vs `b.foo`).

## Re-exports

A module re-exports a binding from another module by importing and marking it `+`:

```
use ./internal {helper}
+helper = helper
```

Or re-export with a different name:

```
use ./internal {helper}
+public_name = helper
```

## Circular Imports

Circular imports are a compile error. The error message shows the full cycle chain (`a -> b -> c -> a`) to help with restructuring.

## Entry Point

When `lx run file.lx` executes:

1. All `use` imports are resolved
2. Top-level statements execute in order (top to bottom)
3. If a `+main` binding exists and is a function, it is called after all top-level bindings are evaluated

Simple scripts need no `main`:

```
use std/env
name = env.get "USER" ?? "world"
$echo "hello {name}"
```

Structured programs use `main`:

```
use std/env
use std/fs

+main = () {
  env.args ? {
    [path] -> fs.read path ^ | lines | len | (n) $echo "{n}"
    _      -> $echo "usage: count <file>"
  }
}
```

After `use std/env`, `env.args` contains command-line arguments regardless of whether `main` exists. When `main` exists, top-level bindings are still evaluated first (type definitions, constants, helpers), then `main` is called.

## Script vs Library

A file without `+main` that has exports (`+` bindings) is a library — it's only useful when imported. A file with `+main` is an executable script. A file can be both: exports for library use, `main` for standalone execution.

When imported with `use`, `main` is NOT called — only exported bindings are visible.

## `+main` Requirements

`+main` must be a function value. If `+main` exists but is not callable (e.g., `+main = 42`), the interpreter reports `error[type]: +main must be a function, got Int`.

## Import Shadowing

Selective imports shadow built-in names. The compiler warns when this happens:

```
use ./lib {map}           -- warning: import 'map' shadows built-in
```

Whole-module imports never shadow — `lib.map` is distinct from `map`. If shadowing is intentional (custom `map` implementation), the warning is informational only and does not prevent compilation.
