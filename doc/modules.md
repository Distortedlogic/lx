# Modules — Reference

## Importing

```
use std/fs                 -- import entire module
use std/fs {read write}    -- selective import
use ./util                 -- relative path (same directory)
use ../shared/types        -- relative path (parent directory)
```

Aliasing: `use std/http : h` -- access as `h.get`, `h.post`, etc.

## Exporting

`+` at column 0 marks a binding as exported:

```
+add = (x y) x + y            -- exported
helper = (x) x * x            -- private
+Point = {x: Float  y: Float} -- exported type
```

Everything without `+` is module-private. All exports are named.

## Module Access

```
use std/fs
content = fs.read "file.txt" ^

use std/fs {read write}
content = read "file.txt" ^   -- selective imports are direct
```

## Variant Constructor Scoping

Importing a module that exports a tagged union brings variant constructors into scope as bare names. `use ./types` where `types` exports `Color = | Red | Green | Blue` makes `Red`, `Green`, `Blue` available. On name conflicts, use `types.Red`.

## Import Conflicts

Selective imports with the same name are a compile error. Fix with aliasing (`use ./a : a` / `use ./b : b`).

## Re-exports

```
use ./internal {helper}
+helper = helper               -- re-export
+public_name = helper          -- re-export with different name
```

Circular imports are a compile error (error shows full cycle chain).

## Entry Point

1. All `use` imports are resolved
2. Top-level statements execute top to bottom
3. If `+main` exists and is a function, it is called after all top-level bindings

```
+main = () {
  env.args ? {
    [path] -> fs.read path ^ | lines | len | (n) $echo "{n}"
    _      -> $echo "usage: count <file>"
  }
}
```

`+main` must be a function. When imported with `use`, `main` is NOT called.

Simple scripts without `+main` just run top-level statements.

## Import Shadowing

Selective imports that shadow built-in names produce a warning. Whole-module imports never shadow (`lib.map` vs `map`).
