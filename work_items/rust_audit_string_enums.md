# String-to-Enum — Additional Conversions

Additional string-vs-enum violations beyond existing string_to_enum_types.md.

---

## Task 1: "Any" sentinel type name → constant

**File:** `crates/lx/src/interpreter/trait_apply.rs`

Lines 19 and 86: `field.type_name != "Any"` — the string `"Any"` is a magic sentinel for a type-system concept. `field.type_name` is of type `Sym` (interned string).

Fix: Define a constant in a shared location (e.g., `crates/lx/src/ast/types.rs` near the `Field` struct definition):
```rust
pub const ANY_TYPE_NAME: &str = "Any";
```

Replace both occurrences:
- Line 19: `field.type_name != "Any"` → `field.type_name != ANY_TYPE_NAME`
- Line 86: `field.type_name != "Any"` → `field.type_name != ANY_TYPE_NAME`

Import `ANY_TYPE_NAME` in `trait_apply.rs`.

---

## Task 2: Constructor names Some/Ok/Err → constants

**File:** `crates/lx/src/checker/infer_pattern.rs`

Lines 138, 145, 147: `ctor_name.as_str() == "Some"`, `== "Ok"`, `== "Err"`. These are well-known constructor names matched as strings.

Fix: Define constants (simpler than an enum since these are only used in 3 comparisons):
```rust
const CTOR_SOME: &str = "Some";
const CTOR_OK: &str = "Ok";
const CTOR_ERR: &str = "Err";
```

Place at the top of `infer_pattern.rs` (module-level). Replace the string comparisons with constant references.

---

## Task 3: Module names in classify_call → enum

**File:** `crates/lx/src/stdlib/diag/diag_walk_expr.rs:66-88`

`classify_call` matches module names as strings against a fixed set. The complete set of matched strings from the actual code:

Category `NodeKind::Fork`: `"pool"`, `"saga"`, `"plan"`
Category `NodeKind::Loop`: `"retry"`, `"cron"`
Category `NodeKind::Resource`: `"circuit"` (except `"check"` method → `Decision`), `"trace"`, `"knowledge"`, `"memory"`, `"budget"`, `"context"`, `"tasks"`, `"profile"`
Category `NodeKind::User`: `"user"`
Category `NodeKind::Io`: `"http"`, `"fs"`, `"git"`

Fix: Define an enum in `diag_walk_expr.rs`:
```rust
enum StdlibModule {
    Pool, Saga, Plan,
    Retry, Cron,
    Circuit, Trace, Knowledge, Memory, Budget, Context, Tasks, Profile,
    User,
    Http, Fs, Git,
}

impl StdlibModule {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "pool" => Some(Self::Pool),
            "saga" => Some(Self::Saga),
            "plan" => Some(Self::Plan),
            "retry" => Some(Self::Retry),
            "cron" => Some(Self::Cron),
            "circuit" => Some(Self::Circuit),
            "trace" => Some(Self::Trace),
            "knowledge" => Some(Self::Knowledge),
            "memory" => Some(Self::Memory),
            "budget" => Some(Self::Budget),
            "context" => Some(Self::Context),
            "tasks" => Some(Self::Tasks),
            "profile" => Some(Self::Profile),
            "user" => Some(Self::User),
            "http" => Some(Self::Http),
            "fs" => Some(Self::Fs),
            "git" => Some(Self::Git),
            _ => None,
        }
    }
}
```

Then `classify_call` matches on `StdlibModule` variants instead of string literals.

---

## Task 4: Entry point names "run"/"main" → constants

**File:** `crates/lx/src/stdlib/test_mod/test_invoke.rs:62-64`

Function entry names matched as `"run"` and `"main"`.

Fix: Define constants at module level:
```rust
const ENTRY_RUN: &str = "run";
const ENTRY_MAIN: &str = "main";
```

Replace the string comparisons at lines 62-64.

---

## Task 5: "std" stdlib root → constant

**Files:**
- `crates/lx/src/checker/visit_stmt.rs:93` — `s.as_str() == "std"`
- `crates/lx/src/stdlib/mod.rs:36` — `path[0] != "std"`

Fix: Define a single constant in `crates/lx/src/stdlib/mod.rs`:
```rust
pub const STDLIB_ROOT: &str = "std";
```

Replace line 36: `path[0] != "std"` → `path[0] != STDLIB_ROOT`.
Import in `visit_stmt.rs`: `use crate::stdlib::STDLIB_ROOT;` and replace line 93: `s.as_str() == "std"` → `s.as_str() == STDLIB_ROOT`.

