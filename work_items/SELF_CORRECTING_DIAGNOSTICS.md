# Goal

Make the type checker's error messages self-correcting: every error tells the agent what it probably meant, not just what went wrong. When an identifier is unknown, suggest the closest name in scope. When an import fails, list available exports. When a function call has wrong types, show the full signature.

# Why

LLM agents writing lx programs don't know the language — it's not in their training data. When the agent writes `std.stream.collect` and the real name is `to_list`, it gets a bare `"unknown import"` error and has to guess again. Self-correcting diagnostics turn dead-end errors into hints — the agent reads the suggestion and fixes it in one shot.

The `TypeError`/`TypeContext` infrastructure and basic `help()` method already exist. The `similar` crate (v2.7.0) is already in Cargo.toml. The `SemanticModel` tracks all definitions in scope. The `stdlib_sigs` map has all std module exports. All the data is there — it just needs to be wired into the diagnostic emission paths.

# Verified facts

- **Unknown ident handling** is at `type_ops.rs:23-28`. When `resolve_in_scope` returns `None`, the code falls through to `self.sem.lookup_type(*name).unwrap_or(self.type_arena.unknown())`. No diagnostic is emitted — it silently returns `Unknown`.
- **Unknown import handling** is at `visit_stmt.rs:141-150`. Two paths: std imports check `std_translated.contains_key(name)`, non-std imports check `sig.bindings.contains_key(name) && sig.types.contains_key(name) && sig.traits.contains_key(name)`. Both emit `DiagnosticKind::UnknownImport { name, module }` with no suggestions.
- **Field access** at `type_ops.rs:44-51` returns `self.type_arena.todo()` — there is NO field existence checking at all. Field suggestion task is deferred (requires implementing field type checking first, which is out of scope).
- **SemanticModelBuilder** has `def_lookup: HashMap<(ScopeId, Sym), DefinitionId>` and `scope_stack: Vec<ScopeId>`. To collect all names in scope, iterate `def_lookup` keys where the ScopeId is in the current scope_stack.
- **`similar` crate API**: `similar::TextDiff::from_chars(a, b).ratio()` returns a f64 similarity ratio (0.0 to 1.0). This works for fuzzy name matching without adding new dependencies.
- **DiagnosticKind** variants are defined at `diagnostics.rs:37-49`. Current variants: `NegationRequiresNumeric`, `PropagateRequiresResultOrMaybe`, `TernaryCondNotBool`, `TimeoutMsNotNumeric`, `LogicalOpRequiresBool`, `MutableCaptureInConcurrent`, `NonExhaustiveMatch`, `DuplicateImport`, `UnknownImport`, `TypeMismatch`, `LintWarning`.
- **Diagnostic struct**: `{ level: DiagLevel, kind: DiagnosticKind, span: SourceSpan, secondary: Vec<(SourceSpan, String)>, fix: Option<Fix> }`.
- **Fix struct**: `{ description: String, edits: Vec<TextEdit>, applicability: Applicability }`.
- **TextEdit struct**: `{ range: SourceSpan, replacement: String }`.
- **Checker.emit()** at `mod.rs:102`: pushes Diagnostic with auto-generated fix via `kind.suggest_fix()`.
- **Function application** at `synth_compound.rs:95-123` uses hardcoded context `TypeContext::FuncArg { func_name: "apply".into(), param_name: "arg".into(), param_idx: 0 }` — the actual function name is never included. This needs fixing.
- **stdlib_sigs** is `HashMap<String, ModuleSignature>` built at `stdlib_sigs.rs`. Each `ModuleSignature` has `bindings: HashMap<Sym, TypeId>`.
- **TypeError::help()** at `unification.rs:269-277` has only 3 cases: Int↔Str and non-Func.

# What changes

**New file `crates/lx/src/checker/suggest.rs`:** Fuzzy name matching utility using `similar::TextDiff::from_chars().ratio()`.

**Modified `crates/lx/src/checker/diagnostics.rs`:** New `DiagnosticKind` variants: `UnknownIdent { name: Sym, suggestions: Vec<String> }`, `UnknownModule { name: String, suggestions: Vec<String> }`. Extended `UnknownImport` with `suggestions: Vec<String>`. Extended `help()` and `suggest_fix()` to handle new variants.

**Modified `crates/lx/src/checker/type_ops.rs`:** At lines 23-28, emit `UnknownIdent` diagnostic with scope-aware suggestions when identifier isn't found.

**Modified `crates/lx/src/checker/visit_stmt.rs`:** At lines 141-150, enrich `UnknownImport` with fuzzy-matched suggestions. Add `UnknownModule` emission when module name doesn't exist in `stdlib_sigs`.

**Modified `crates/lx/src/checker/synth_compound.rs`:** At lines 95-123, resolve the actual function name from the func expression and pass it to `TypeContext::FuncArg` instead of hardcoded `"apply"`. Add signature display to secondary span.

**Modified `crates/lx/src/checker/unification.rs`:** Expand `TypeError::help()` with more type conversion hints.

**Modified `crates/lx/src/checker/semantic.rs`:** Add `names_in_scope()` method to `SemanticModelBuilder`.

**Modified `crates/lx/src/checker/mod.rs`:** Add `pub(crate) mod suggest;`.

# Files affected

- NEW: `crates/lx/src/checker/suggest.rs`
- EDIT: `crates/lx/src/checker/mod.rs` — add module declaration
- EDIT: `crates/lx/src/checker/semantic.rs` — add `names_in_scope()` method
- EDIT: `crates/lx/src/checker/diagnostics.rs` — add variants, extend help/fix
- EDIT: `crates/lx/src/checker/type_ops.rs` — emit UnknownIdent at lines 23-28
- EDIT: `crates/lx/src/checker/visit_stmt.rs` — enrich import errors at lines 141-150
- EDIT: `crates/lx/src/checker/synth_compound.rs` — fix func_name in FuncArg context, add signature to secondary
- EDIT: `crates/lx/src/checker/unification.rs` — expand TypeError::help()

# Task List

### Task 1: Create fuzzy name matcher utility

**Subject:** Add suggest.rs with similarity-ratio-based name matching

**Description:** Create `crates/lx/src/checker/suggest.rs` with two functions:

```rust
use similar::TextDiff;

pub fn closest_matches(target: &str, candidates: &[&str], max: usize) -> Vec<String> {
    // 1. For each candidate, compute: TextDiff::from_chars(target, candidate).ratio()
    // 2. Filter: only keep candidates with ratio > 0.6
    // 3. Sort by ratio descending
    // 4. Take up to `max` results
    // 5. Return as Vec<String>
    // Edge cases: if target is empty or candidates is empty, return empty vec
}

pub fn format_suggestions(suggestions: &[String]) -> Option<String> {
    // If empty: return None
    // If 1 item: Some(format!("did you mean '{}'?", suggestions[0]))
    // If 2+ items: Some(format!("did you mean one of: {}?", suggestions.join(", ")))
}
```

In `crates/lx/src/checker/mod.rs`, add `pub(crate) mod suggest;` to the module declarations (alongside the existing `mod capture;`, `mod check_expr;`, etc. at lines 2-20).

**ActiveForm:** Creating fuzzy name matcher utility

### Task 2: Add names_in_scope method to SemanticModelBuilder

**Subject:** Collect all visible names from the current scope chain

**Description:** In `crates/lx/src/checker/semantic.rs`, add a method to `SemanticModelBuilder` (in the `impl SemanticModelBuilder` block starting at line 98):

```rust
pub fn names_in_scope(&self) -> Vec<Sym> {
    let mut names = Vec::new();
    for &scope_id in &self.scope_stack {
        for (&(sid, name), _) in &self.def_lookup {
            if sid == scope_id {
                names.push(name);
            }
        }
    }
    names.sort();
    names.dedup();
    names
}
```

This iterates `def_lookup` (which is `HashMap<(ScopeId, Sym), DefinitionId>`) and collects all `Sym` names where the `ScopeId` is in the current `scope_stack`. Deduplicates because the same name can be shadowed in different scopes.

**ActiveForm:** Adding names_in_scope to SemanticModelBuilder

### Task 3: Add UnknownIdent diagnostic and emit it for unresolved identifiers

**Subject:** Emit suggestions when an identifier can't be resolved

**Description:** Two files to edit:

**File 1: `crates/lx/src/checker/diagnostics.rs`**

Add two new variants to `DiagnosticKind` (after the existing `LintWarning` variant at line 49):
```rust
UnknownIdent { name: Sym, suggestions: Vec<String> },
UnknownModule { name: String, suggestions: Vec<String> },
```

In the `impl DiagnosticKind` block, add `help()` match arms:
```rust
DiagnosticKind::UnknownIdent { name, suggestions } => {
    super::suggest::format_suggestions(suggestions)
        .or_else(|| Some(format!("'{}' is not defined in this scope", name)))
},
DiagnosticKind::UnknownModule { name, suggestions } => {
    super::suggest::format_suggestions(suggestions)
        .or_else(|| Some(format!("module '{}' not found", name)))
},
```

**File 2: `crates/lx/src/checker/type_ops.rs`**

At lines 23-28, the current code for `Expr::Ident(name)` is:
```rust
Expr::Ident(name) => {
  if let Some(def_id) = self.sem.resolve_in_scope(*name) {
    self.sem.add_reference(eid, def_id);
  }
  if let Some(narrowed) = self.narrowing.lookup(*name) { narrowed } else { self.sem.lookup_type(*name).unwrap_or(self.type_arena.unknown()) }
},
```

Change to:
```rust
Expr::Ident(name) => {
  if let Some(def_id) = self.sem.resolve_in_scope(*name) {
    self.sem.add_reference(eid, def_id);
  } else {
    let scope_names = self.sem.names_in_scope();
    let candidates: Vec<&str> = scope_names.iter().map(|s| s.as_str()).collect();
    let suggestions = super::suggest::closest_matches(name.as_str(), &candidates, 3);
    self.emit(DiagLevel::Error, DiagnosticKind::UnknownIdent { name: *name, suggestions }, span);
  }
  if let Some(narrowed) = self.narrowing.lookup(*name) { narrowed } else { self.sem.lookup_type(*name).unwrap_or(self.type_arena.unknown()) }
},
```

The diagnostic is emitted when the name isn't in scope. The type still falls through to `unknown()` so checking continues.

**ActiveForm:** Adding UnknownIdent diagnostic with scope-aware suggestions

### Task 4: Enrich import errors with suggestions

**Subject:** Suggest available exports when an import name or module is unknown

**Description:** Two changes in `crates/lx/src/checker/visit_stmt.rs` and one in `diagnostics.rs`:

**File 1: `crates/lx/src/checker/diagnostics.rs`**

Change the `UnknownImport` variant from:
```rust
UnknownImport { name: Sym, module: Sym },
```
to:
```rust
UnknownImport { name: Sym, module: Sym, suggestions: Vec<String> },
```

Update the `help()` arm for `UnknownImport`:
```rust
DiagnosticKind::UnknownImport { name, module, suggestions } => {
    super::suggest::format_suggestions(suggestions)
        .or_else(|| Some(format!("'{}' is not exported by module '{}'", name, module)))
},
```

Update `suggest_fix()` if it references `UnknownImport` — add the `suggestions` field to the pattern.

**File 2: `crates/lx/src/checker/visit_stmt.rs`**

At line 141-142 (std import path), currently:
```rust
if std_translated.as_ref().is_some_and(|t| !t.contains_key(name)) {
    self.emit(DiagLevel::Error, DiagnosticKind::UnknownImport { name: *name, module: module_name.unwrap_or(*name) }, span);
}
```

Change to:
```rust
if std_translated.as_ref().is_some_and(|t| !t.contains_key(name)) {
    let export_names: Vec<&str> = std_translated.as_ref()
        .map(|t| t.keys().map(|k| k.as_str()).collect())
        .unwrap_or_default();
    let suggestions = super::suggest::closest_matches(name.as_str(), &export_names, 5);
    self.emit(DiagLevel::Error, DiagnosticKind::UnknownImport { name: *name, module: module_name.unwrap_or(*name), suggestions }, span);
}
```

At lines 143-151 (non-std import path), currently:
```rust
} else if std_translated.is_none()
    && let Some(mod_sym) = module_name
    && let Some(sig) = self.import_signatures.get(&mod_sym)
    && !sig.bindings.contains_key(name)
    && !sig.types.contains_key(name)
    && !sig.traits.contains_key(name)
{
    self.emit(DiagLevel::Error, DiagnosticKind::UnknownImport { name: *name, module: mod_sym }, span);
}
```

Change the emit to include suggestions:
```rust
{
    let mut export_names: Vec<&str> = sig.bindings.keys().map(|k| k.as_str()).collect();
    export_names.extend(sig.types.keys().map(|k| k.as_str()));
    export_names.extend(sig.traits.keys().map(|k| k.as_str()));
    let suggestions = super::suggest::closest_matches(name.as_str(), &export_names, 5);
    self.emit(DiagLevel::Error, DiagnosticKind::UnknownImport { name: *name, module: mod_sym, suggestions }, span);
}
```

Additionally, add `UnknownModule` detection: In the `resolve_use` method, after `let std_data` is computed (line 94-102), if `is_std && u.path.len() >= 2` and `std_data` is `None` (meaning the module doesn't exist in stdlib_sigs), emit `UnknownModule`:
```rust
if is_std && u.path.len() >= 2 && std_data.is_none() {
    let module_key = u.path[1].as_str();
    let available: Vec<&str> = self.stdlib_sigs.keys().map(|k| k.as_str()).collect();
    let suggestions = super::suggest::closest_matches(module_key, &available, 3);
    self.emit(DiagLevel::Error, DiagnosticKind::UnknownModule { name: format!("std.{}", module_key), suggestions }, span);
}
```

Insert this check after line 102, before the `match &u.kind` at line 107.

**ActiveForm:** Enriching import errors with fuzzy-matched suggestions

### Task 5: Show actual function name and signature on arg mismatch

**Subject:** Include real function name and full signature in application type errors

**Description:** In `crates/lx/src/checker/synth_compound.rs`, in the `synth_apply_type` method (around lines 95-123):

The current code creates context with hardcoded name:
```rust
let ctx = TypeContext::FuncArg { func_name: "apply".into(), param_name: "arg".into(), param_idx: 0 };
```

Change to resolve the actual function name from the func expression:
```rust
let func_name = match self.arena.expr(func) {
    Expr::Ident(name) => name.to_string(),
    Expr::FieldAccess(fa) => match &fa.field {
        FieldKind::Named(name) => name.to_string(),
        _ => "<fn>".into(),
    },
    _ => "<fn>".into(),
};
let sig_display = self.type_arena.display(resolved);
let ctx = TypeContext::FuncArg { func_name, param_name: "arg".into(), param_idx: 0 };
```

Then, when the type error is emitted:
```rust
if let Err(te) = self.table.unify_with_context(inst_param, arg_t, ctx, &mut self.type_arena) {
    self.emit_type_error(&te, arg_span);
```

Add a secondary span annotation showing the function signature. Access the function span:
```rust
if let Err(te) = self.table.unify_with_context(inst_param, arg_t, ctx, &mut self.type_arena) {
    let func_span = self.arena.expr_span(func);
    let mut diag = self.make_type_error_diagnostic(&te, arg_span);
    diag.secondary.push((func_span, format!("signature: {}", sig_display)));
    self.diagnostics.push(diag);
```

Check if `make_type_error_diagnostic` exists as a method. If not (if `emit_type_error` pushes directly), refactor: extract the Diagnostic construction from `emit_type_error` into a helper that returns the Diagnostic without pushing, so the secondary can be added before pushing.

Also handle the `_ =>` branch (non-function applied), which currently silently returns `unknown()`:
```rust
_ => {
    self.synth_expr(arg);
    let type_display = self.type_arena.display(resolved);
    self.emit(DiagLevel::Error, DiagnosticKind::TypeMismatch {
        error: TypeError {
            expected: self.type_arena.alloc(Type::Func { param: self.type_arena.unknown(), ret: self.type_arena.unknown() }),
            found: resolved,
            context: TypeContext::General,
            expected_origin: None,
        }
    }, self.arena.expr_span(func));
    self.type_arena.unknown()
},
```

This needs to be adjusted based on how `emit_type_error` and `TypeError` constructors actually work. The key point: emit a diagnostic for "this value is not callable" instead of silently returning unknown.

**ActiveForm:** Including real function name and signature in application errors

### Task 6: Expand TypeError::help() with more type conversion hints

**Subject:** Add type conversion hints for common type mismatch patterns

**Description:** In `crates/lx/src/checker/unification.rs`, expand the `TypeError::help()` method (around lines 269-277). The current code:

```rust
pub fn help(&self, ta: &TypeArena) -> Option<String> {
    match (ta.get(self.expected), ta.get(self.found)) {
        (Type::Int, Type::Str) => Some("did you mean to pass a number?".into()),
        (Type::Str, Type::Int) => Some("did you mean to convert this to a string?".into()),
        (Type::Func { .. }, _) => Some("this value is not callable".into()),
        _ => None,
    }
}
```

Add these additional arms before the `_ => None`:

```rust
(Type::Maybe(_), t) | (t, Type::Maybe(_)) if !matches!(t, Type::Maybe(_)) => {
    Some("wrap with Some(...) to create a Maybe, or use ?? to unwrap with a default".into())
},
(Type::Result { .. }, t) | (t, Type::Result { .. }) if !matches!(t, Type::Result { .. }) => {
    Some("wrap with Ok(...) to create a Result, or use ^ to propagate errors".into())
},
(Type::List(_), t) | (t, Type::List(_)) if !matches!(t, Type::List(_)) => {
    Some("to create a single-element list, use [value]".into())
},
(Type::Bool, Type::Int) | (Type::Int, Type::Bool) => {
    Some("booleans and integers are not interchangeable in lx".into())
},
(Type::Record(expected_fields), Type::Record(found_fields)) => {
    let missing: Vec<String> = expected_fields.iter()
        .filter(|(name, _)| !found_fields.iter().any(|(n, _)| n == name))
        .map(|(name, _)| name.to_string())
        .collect();
    if !missing.is_empty() {
        Some(format!("record is missing fields: {}", missing.join(", ")))
    } else {
        let extra: Vec<String> = found_fields.iter()
            .filter(|(name, _)| !expected_fields.iter().any(|(n, _)| n == name))
            .map(|(name, _)| name.to_string())
            .collect();
        if !extra.is_empty() {
            Some(format!("record has unexpected fields: {}", extra.join(", ")))
        } else {
            None
        }
    }
},
```

Note: The `Type` enum variants must be matched by reference since `ta.get()` returns `&Type`. Adjust the match patterns to use references: `(Type::Maybe(_), t)` may need to be `(&Type::Maybe(_), t)` depending on what `ta.get()` returns. Check the return type of `TypeArena::get()` and adjust accordingly.

**ActiveForm:** Expanding TypeError help with type conversion hints

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.

---

## Task Loading Instructions

To execute this work item, load it with the workflow MCP:

```
mcp__workflow__load_work_item({ path: "work_items/SELF_CORRECTING_DIAGNOSTICS.md" })
```

Then call `next_task` to begin.
