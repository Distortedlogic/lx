# Verbose Patterns and Repeated Literals

Convert verbose patterns to idiomatic Rust alternatives and extract repeated literals.

---

## Task 1: Two-arm matches → if let (stdlib/test_mod)

These files have repetitive two-arm matches that should be `if let` or `.map_or()`:

**File:** `stdlib/test_mod/test_run.rs`

Five instances:

1. Lines 82-87: `match &scores_val { LxVal::Record(r) => r.as_ref().clone(), _ => return Err(...) }` → `let LxVal::Record(r) = &scores_val else { return Err(...); };`

2. Lines 122-125: `match spec_fields.get(&intern("weights")) { Some(LxVal::Record(r)) => r.as_ref().clone(), _ => IndexMap::new() }` → `if let Some(LxVal::Record(r)) = spec_fields.get(&intern("weights")) { r.as_ref().clone() } else { IndexMap::new() }`

3. Lines 150-153: inside `.filter_map`: `match s { LxVal::Record(r) => ..., _ => None }` → `if let LxVal::Record(r) = s { ... } else { None }`

4. Lines 169-172: `match spec_fields.get(&intern("scenarios")) { Some(LxVal::List(list)) => list.as_ref().clone(), _ => Vec::new() }` → `if let Some(LxVal::List(list)) = spec_fields.get(&intern("scenarios")) { list.as_ref().clone() } else { Vec::new() }`

5. Lines 179-182: same pattern as #4 above.

**File:** `stdlib/test_mod/test_report.rs`
Lines 16-18, 48-50 — two instances. Same `Some(LxVal::List/Record) => ..., _ => default` pattern → convert to `if let`.

**File:** `stdlib/test_mod/mod.rs`
Lines 30-32, 100-106 — two instances. Same pattern.

---

## Task 2: Two-arm matches → if let (other modules)

**File:** `interpreter/ambient.rs:17`
```rust
// Before
match interp.env.get(...) {
    Some(LxVal::Record(r)) => ...,
    _ => IndexMap::new(),
}
// After
if let Some(LxVal::Record(r)) = interp.env.get(...) { ... } else { IndexMap::new() }
```

**File:** `stdlib/schema.rs:183`
```rust
// Before
match do_validate(...) {
    LxVal::Ok(_) => Ok(LxVal::Bool(true)),
    _ => Ok(LxVal::Bool(false)),
}
// After
Ok(LxVal::Bool(matches!(do_validate(...), LxVal::Ok(_))))
```

**File:** `builtins/register.rs:233-236`
Current: `let name = match &args[1] { LxVal::Str(s) => s.as_ref(), _ => return Ok(LxVal::None) };`
Convert to: `let LxVal::Str(s) = &args[1] else { return Ok(LxVal::None); }; let name = s.as_ref();`

**File:** `builtins/agent.rs:25-27`
Convert to `if let ... else { return Err(...) }`.

---

## Task 3: for..push → iterator chains

**File:** `interpreter/modules.rs`
Lines 101-102: `for segment in &path[1..] { result.push(segment); }` → `result.extend(&path[1..]);`
Lines 125-126: same pattern with different slice index → use `result.extend(&path[...]);`
Lines 209-210: same pattern → use `result.extend(&path[...]);`

**File:** `checker/semantic.rs:189-190`
```rust
// Before
for &def_id in defs {
    names.push(self.definitions[def_id.index()].name);
}
// After
let names: Vec<_> = defs.iter().map(|&d| self.definitions[d.index()].name).collect();
```

**File:** `builtins/hof_parallel.rs:21-22`
```rust
// Before
for r in results {
    out.push(r?);
}
// After
let out: Vec<_> = results.into_iter().collect::<Result<_, _>>()?;
```

---

## Task 4: Extract repeated CSS class literals

Create a new file `crates/lx-desktop/src/styles.rs` with:
```rust
pub const PAGE_HEADING: &str = "text-2xl font-bold uppercase tracking-wider text-[var(--on-surface)] font-[var(--font-display)]";
pub const FLEX_BETWEEN: &str = "flex items-center justify-between";
```

Add `pub mod styles;` to `crates/lx-desktop/src/lib.rs` (after the existing `pub mod routes;` line).

Replace occurrences of `PAGE_HEADING`:
- `crates/lx-desktop/src/pages/activity.rs:12`
- `crates/lx-desktop/src/pages/tools/mod.rs:12`
- `crates/lx-desktop/src/pages/accounts.rs:23`

In each file: add `use crate::styles::PAGE_HEADING;` and replace the string literal with `PAGE_HEADING`.

Replace occurrences of `FLEX_BETWEEN`:
- `crates/lx-desktop/src/pages/settings/mod.rs:18`
- `crates/lx-desktop/src/pages/activity.rs:11`
- `crates/lx-desktop/src/pages/tools/mod.rs:11`
- `crates/lx-desktop/src/pages/accounts.rs:22`

In each file: add `use crate::styles::FLEX_BETWEEN;` and replace the string literal with `FLEX_BETWEEN`.
