# Inline Imports — lx builtins, interpreter, and other modules

Replace inline `std::` and `crate::` paths with `use` imports at the top of each file.

**Supersedes:** code_cleanup.md Task 2 (test_invoke.rs), Task 3 (source.rs), Task 4 (ast/mod.rs).

---

## builtins/convert.rs

Add:
```
use std::thread;
use std::time::Duration;
```
Replace `std::thread::sleep(std::time::Duration::from_secs_f64(secs))` → `thread::sleep(Duration::from_secs_f64(secs))` (line 102).

---

## builtins/coll.rs

Add:
```
use std::cmp::Ordering;
```
Replace all `std::cmp::Ordering` → `Ordering` (lines 12, 16, 17, 20).

---

## builtins/hof.rs

Add:
```
use std::ops::Deref;
```
Replace `impl<'a> std::ops::Deref for ListRef<'a>` → `impl<'a> Deref for ListRef<'a>` (line 58).

---

## builtins/mod.rs

Add:
```
use std::future::Future;
```
Replace `dyn std::future::Future` → `dyn Future` (line 23).

---

## builtins/shell.rs

Add:
```
use std::process::Command;
```
Replace `std::process::Command::new` → `Command::new` (line 12).

---

## builtins/register.rs

Add:
```
use serde_json;
```
Replace inline `serde_json::from_str`, `serde_json::Value`, `serde_json::to_string`, `serde_json::to_string_pretty` (lines 216, 223, 228).

Also add:
```
use crate::interpreter::ambient::{global_context_current, global_context_get};
```
Replace inline `crate::interpreter::ambient::global_context_current` and `global_context_get` (lines 263, 267).

---

## builtins/call.rs

Add:
```
use crate::interpreter::Interpreter;
use crate::ast::AstArena;
use crate::env::Env;
use crate::error::EvalSignal;
```
Replace inline paths (lines 24, 47-51).

---

## builtins/hof_extra.rs

Add:
```
use crate::value::ValueKey;
```
Replace `crate::value::ValueKey` → `ValueKey` (line 118).

---

## value/func.rs

Add:
```
use miette::SourceSpan;
use crate::runtime::RuntimeCtx;
use std::future::Future;
```
`miette::SourceSpan` is NOT currently imported in this file — add it.

Replace inline paths in type aliases:
- `miette::SourceSpan` → `SourceSpan` (lines 24, 27, 30)
- `crate::runtime::RuntimeCtx` → `RuntimeCtx` (lines 24, 27, 30)
- `dyn std::future::Future` → `dyn Future` (line 27)

---

## value/impls.rs

Add:
```
use std::mem;
```
Replace `std::mem::discriminant` → `mem::discriminant` (line 78).

**Do NOT touch** `indexmap::IndexMap` at line 8 — it is inside a `#[macro_export]` macro (`record!`) and must remain fully qualified because the macro expands at call sites where `IndexMap` may not be in scope.

---

## error.rs

`Arc` is NOT currently imported in this file. Add:
```
use std::sync::Arc;
```
Replace `std::sync::Arc<str>` → `Arc<str>` (lines 78, 115).

---

## source.rs

Add:
```
use std::collections::HashMap;
```
Replace `std::collections::HashMap` → `HashMap` (line 117).

---

## lexer/strings.rs

Add:
```
use std::mem;
```
Replace `std::mem::take` → `mem::take` (line 11).

---

## interpreter/apply.rs

Add:
```
use std::mem;
use indexmap::IndexMap;
```
Replace `std::mem::replace` → `mem::replace` (line 21), `indexmap::IndexMap` → `IndexMap` (line 88).

---

## interpreter/ambient.rs

Add:
```
use std::cell::RefCell;
```
Replace `std::cell::RefCell` → `RefCell` (lines 56-57).

---

## interpreter/patterns.rs

Add:
```
use std::collections::HashSet;
use indexmap::IndexMap;
```
Replace inline usages (lines 119, 120).

---

## interpreter/modules.rs

Add:
```
use std::env;
use std::fs;
use std::path::Path;
use crate::parser::parse;
use crate::source::FileId;
use crate::stdlib::wasm::load_plugin;
```
Replace inline paths (lines 74, 86, 141, 156, 169, 171, 195).

---

## interpreter/type_apply.rs

Add:
```
use crate::error::EvalSignal;
```
Replace `crate::error::EvalSignal::Error` and `EvalSignal::Break` → `EvalSignal::Error`, `EvalSignal::Break` (lines 14-15).

---

## interpreter/default_tools.rs

Add:
```
use crate::parser::parse;
use crate::source::FileId;
use crate::error::EvalSignal;
```
Replace inline paths (lines 18, 25-26).

---

## interpreter/trait_apply.rs

Add:
```
use indexmap::IndexMap;
```
Replace `indexmap::IndexMap` → `IndexMap` (line 82).

---

## ast/comment_attach.rs

Add:
```
use std::cmp::Reverse;
```
Replace `std::cmp::Reverse` → `Reverse` (line 34).

---

## checker/suggest.rs

Add:
```
use std::cmp::Ordering;
```
Replace `std::cmp::Ordering::Equal` → `Ordering::Equal` (line 8).

---

## linter/rules/ (7 files)

Each of these files uses `std::mem::take(&mut self.diagnostics)` inline:

- `break_outside_loop.rs:61`
- `empty_match.rs:53`
- `redundant_propagate.rs:63`
- `duplicate_record_field.rs:60`
- `single_branch_par.rs:56`
- `unreachable_code.rs:71`
- `unused_import.rs:71`

For each: add `use std::mem;` at the top, replace `std::mem::take` → `mem::take`.

---

## folder/desugar_http.rs

Add:
```
use crate::ast::BindTarget;
```
Replace `crate::ast::BindTarget::Name` → `BindTarget::Name` (line 64).

---

## folder/desugar.rs

Add:
```
use crate::visitor::walk_transform::walk_transform_stmt;
```
Replace inline paths (lines 41, 46).
