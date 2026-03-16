# Goal

Fix all code style violations: wildcard imports, TODO/FIXME comments, inline import statements, redundant bindings, duplicate struct fields, free functions that should be methods, and vague function names like do_the_thing.

# Why

The codebase has systematic style violations that harm readability and maintainability. Wildcard imports obscure dependencies. TODO and FIXME comments indicate unfinished work committed to the codebase. Inline imports inside function bodies violate the project convention of top-level imports. Duplicate fields across Config and DbConfig should be consolidated by embedding. Free functions operating on a struct should be methods. Vague names like do_the_thing give no indication of purpose.

# What changes

- Replace all wildcard imports (use foo::*) with specific named imports
- Remove all TODO and FIXME comments — convert to work items or fix inline
- Move all inline import statements from function bodies to file top
- Remove redundant variable bindings that exist only to be returned on the next line
- Consolidate duplicate struct fields between Config and DbConfig by embedding DbConfig in Config
- Convert free functions that take a struct as first parameter into method implementations
- Rename do_the_thing and other vague function names to describe their actual purpose

# Files affected

- src/handler.rs — wildcard imports, TODO/FIXME comments, inline import in function body, vague function name do_the_thing
- src/utils.rs — free functions that should be methods, redundant variable bindings
- src/types.rs — duplicate struct fields across Config and DbConfig

# Task List

## Task 1: Fix src/handler.rs

Replace wildcard imports with specific imports. Remove TODO and FIXME comments. Move inline import to file top. Rename do_the_thing to a descriptive name.

```
just fmt
git add src/handler.rs
git commit -m "fix: clean up handler.rs style violations"
```

## Task 2: Fix src/utils.rs

Convert free functions that take a struct as first parameter into methods on that struct. Remove redundant variable bindings.

```
just fmt
git add src/utils.rs
git commit -m "fix: convert free functions to methods, remove redundant bindings"
```

## Task 3: Fix src/types.rs

Consolidate duplicate fields between Config and DbConfig — embed DbConfig as a field in Config instead of duplicating host, port, name fields.

```
just fmt
git add src/types.rs
git commit -m "fix: consolidate duplicate struct fields via embedding"
```

## Task 4: Verification

```
just test
just diagnose
just fmt
git add -A
git commit -m "chore: final verification after style audit"
```

# CRITICAL REMINDERS

- Run `just fmt` after every file change
- Run `just test` and `just diagnose` before final commit
- No wildcard imports — always import specific items
- No TODO/FIXME in committed code
- All imports at file top, never inline

# Task Loading Instructions

Load these instructions by reading this file, then execute each task in order. After each task, run `just fmt` and commit. After all tasks, run `just test` and `just diagnose` to verify.
