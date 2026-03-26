# Goal

Make lx type names (`Str`, `Int`, `Float`, `Bool`, `List`, `Record`, `Map`, `Tuple`) available as runtime values. `Str` evaluates to a type value that supports comparison (`type_of x == Str`), construction/coercion (`Str 42` returns `"42"`), and use as metadata (`params: {command: Str}`).

# Why

Type names are currently only parsed as type annotations or TypeConstructor AST nodes. At runtime, `Str` in expression context tries to look up an identifier and fails with `undefined constructor 'Str'`. This blocks tool definitions (`params: {command: Str}`), runtime type comparisons, and dynamic construction — all things agents need.

Every language with type introspection makes types first-class: Python (`str`, `int` are callable type objects), Ruby (`String`, `Integer` are class objects), JavaScript (`String`, `Number` are constructor functions). lx already has `type_of` returning strings — making type names runtime values completes the picture.

# Design

New `LxVal::Type` variant:

```rust
Type(Sym),  // interned type name: "Str", "Int", "Float", "Bool", "List", "Record", "Map", "Tuple"
```

## Behavior

### As a value
`Str` in expression context evaluates to `LxVal::Type(intern("Str"))`. Can be bound, passed, stored, returned.

### Comparison
`type_of x == Str` works because `type_of` returns `LxVal::Type` (not a string anymore) and `==` compares Type variants by their Sym.

**Breaking change:** `type_of` currently returns `LxVal::Str("Int")` etc. Changing it to return `LxVal::Type` means existing code like `type_of x == "Int"` breaks. Two options:
- **Option A:** Make `==` between `Type` and `Str` compare the name: `LxVal::Type(s) == LxVal::Str(t)` is true when `s == t`. Backwards compatible.
- **Option B:** Change `type_of` to return `Type`, break existing string comparisons. Cleaner but breaking.

**Choose Option A** — `Type == Str` coerces for comparison. No breaking changes. Existing `type_of x == "Int"` still works. New `type_of x == Int` also works.

### Construction/coercion
`Str 42` → `"42"`. `Int "42"` → `42`. `Float "3.14"` → `3.14`. `Bool 1` → `true`.

These are implemented as the application behavior of `LxVal::Type` in `apply_func`:

```rust
LxVal::Type(name) => {
    match name.as_str() {
        "Str" => Ok(LxVal::str(arg.to_string())),
        "Int" => { /* parse from Str, truncate from Float, identity from Int */ },
        "Float" => { /* parse from Str, promote from Int, identity from Float */ },
        "Bool" => { /* truthy conversion */ },
        _ => Err(LxError::runtime(format!("cannot construct {name} from value"), span)),
    }
}
```

### Display
`LxVal::Type(s)` displays as the type name: `Str`, `Int`, etc.

### In records
`{command: Str}` evaluates to `{command: Type("Str")}` — a record with a Type value. This is what Tool.params contains. `params | keys` still works. Schema generation can inspect the Type values for type information.

# Exact Files to Change

### `crates/lx/src/value/mod.rs`
Add `Type(Sym)` variant to `LxVal` enum. Add `#[strum(serialize = "Type")]` annotation. Add `LxVal::typ(name: &str) -> LxVal` constructor.

### `crates/lx/src/value/display.rs`
Add `LxVal::Type(s) => write!(f, "{s}")` to the Display impl.

### `crates/lx/src/value/impls.rs`
Add `LxVal::Type` arms to `PartialEq` — `Type(a) == Type(b)` when `a == b`. Also `Type(a) == Str(b)` when `a.as_str() == b.as_ref()` (Option A coercion).

### `crates/lx/src/value/serde_impl.rs`
Add `LxVal::Type(s) => serializer.serialize_str(s.as_str())` to Serialize. Add deserialization handling if needed.

### `crates/lx/src/interpreter/apply.rs`
Add `LxVal::Type(name)` arm to `apply_func` for construction/coercion behavior.

### `crates/lx/src/interpreter/mod.rs`
In the `Expr::TypeConstructor(name)` handler (line 135), before failing with "undefined constructor", check if `name` is a built-in type name and return `LxVal::Type(name)` instead of looking it up in the environment.

### `crates/lx/src/builtins/register.rs`
Register built-in type names in the environment:
```rust
env.bind_str("Str", LxVal::typ("Str"));
env.bind_str("Int", LxVal::typ("Int"));
env.bind_str("Float", LxVal::typ("Float"));
env.bind_str("Bool", LxVal::typ("Bool"));
env.bind_str("List", LxVal::typ("List"));
env.bind_str("Record", LxVal::typ("Record"));
env.bind_str("Map", LxVal::typ("Map"));
env.bind_str("Tuple", LxVal::typ("Tuple"));
```

### `crates/lx/src/builtins/register.rs` — `bi_type_of`
Change return value from `LxVal::str(name)` to `LxVal::typ(name)`. Since Option A coercion makes `Type == Str` work, this is backwards compatible.

### All exhaustive matches on LxVal
Grep for `match.*self` and `match.*val` patterns in the value/, interpreter/, builtins/, formatter/ directories. Add `LxVal::Type(_)` arms where needed. Key locations:
- `value/impls.rs` — PartialEq, PartialOrd, Hash
- `interpreter/apply.rs` — function application
- `builtins/collections.rs` — collection operations
- `builtins/register.rs` — type_of
- `formatter/` — if it formats LxVal
- `stdlib/wasm_marshal.rs` — JSON marshaling (Type → JSON string)
- `stdlib/store/store_dispatch.rs` — store field access

# Gotchas

- `IntoStaticStr` derive on LxVal will generate `"Type"` for the variant name. That's correct for `type_of (type_of x)` which should return `Type`.
- `Sym` is an interned string. `LxVal::Type(intern("Str"))` is cheap — no allocation after first use.
- The `TypeConstructor` AST node is used for both user-defined tagged constructors (`Ok`, `Circle`) and built-in type names (`Str`, `Int`). The interpreter must try environment lookup first (for user constructors), then fall back to built-in types. Current code: `self.env.get(name).ok_or_else(|| ...)` — change to: try get, if None and name is a builtin type, return `LxVal::Type(name)`.
- Adding a variant to `LxVal` is a ripple — every exhaustive match needs updating. Use `just rust-diagnose` to find all locations.

# Task List

### Task 1: Add LxVal::Type variant
Add `Type(Sym)` to `LxVal` enum in `value/mod.rs`. Add `LxVal::typ` constructor. Add Display arm in `display.rs`. Run `just rust-diagnose` — this will produce errors at every exhaustive match. Fix each: add `LxVal::Type(_)` arm that does the reasonable thing (usually error or passthrough).

### Task 2: Register type names as values
In `builtins/register.rs`, bind `Str`, `Int`, `Float`, `Bool`, `List`, `Record`, `Map`, `Tuple` as `LxVal::typ(...)` values. Update `bi_type_of` to return `LxVal::typ(name)` instead of `LxVal::str(name)`.

### Task 3: Add Type == Str coercion
In `value/impls.rs`, add PartialEq arm: `(Type(a), Str(b)) => a.as_str() == b.as_ref()` and the symmetric case. This makes `type_of x == "Int"` still work.

### Task 4: Add Type application (construction/coercion)
In `interpreter/apply.rs`, add `LxVal::Type(name)` arm to `apply_func`. `Str x` → `to_string`, `Int x` → parse/truncate, `Float x` → parse/promote, `Bool x` → truthy.

### Task 5: Handle TypeConstructor fallback
In `interpreter/mod.rs` line 135, change `Expr::TypeConstructor` handler: try env lookup first, if not found and name matches a builtin type, return `LxVal::Type(name)`.

### Task 6: Update JSON marshaling
In `stdlib/wasm_marshal.rs`, add `LxVal::Type(s)` to `lxval_to_json_value` — serialize as the string. Add reverse mapping in `json_to_lxval` if a string matches a type name and context expects a Type.

### Task 7: Write tests
Create `tests/type_values.lx`:
- `assert (type_of 42 == Int)` — Type comparison
- `assert (type_of "hello" == Str)` — Type comparison
- `assert (type_of 42 == "Int")` — Type == Str coercion
- `assert (Str 42 == "42")` — construction
- `assert (Int "42" == 42)` — construction
- `assert (Float 42 == 42.0)` — construction
- `r = {name: Str  age: Int}` and `assert (r.name == Str)` — Type in records

---

## Task Loading Instructions

```
mcp__workflow__load_work_item({ path: "work_items/TYPE_VALUES.md" })
```
