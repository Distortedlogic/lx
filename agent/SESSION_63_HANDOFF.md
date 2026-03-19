# Session 63 Handoff — Dict Type + Collection Trait + Class Refactor

## User's Design Philosophy

"Context with agents is a scarce resource so part of the goal of the language is to choose the less verbose way by default without reason to do otherwise."

Core principles that emerged from this session's discussion:
- **Rust-like, not Java-like** — no magic constructors, no init methods. Traits for shared behavior. The caller creates their constructor.
- **No indirection** — don't wrap a store inside a class. Don't wrap a dict inside a class method. If a method just forwards, it shouldn't exist.
- **Each case is different** — don't apply one-size-fits-all patterns. Think about what each specific thing IS, not what pattern to apply.
- **Batteries included** — generic operations (set/get/keys/filter) belong on the primitive type, not on wrappers. If you're writing a Class method that just calls `dict.set`, that method is waste.
- **Agents read markdown** — serialize to markdown by default, not JSON. Markdown is the agent-native format.
- **Errors should propagate by default** — user asked "wouldnt it be less verbose if errors propagated by default?" (open question for language design)

## What Was Built This Session

`Class` keyword and Trait default methods shipped. 80/80 tests pass. `just diagnose` clean.

### Class Implementation (Rust)

- **Token**: `ClassKw` in `token.rs:90`
- **Lexer**: `"Class" => TokenKind::ClassKw` in `lexer/keywords.rs:28`
- **AST**: `Stmt::ClassDecl { name, traits, fields, methods, exported }` in `ast/mod.rs:67-73`. `ClassField { name, default }` in `ast/types.rs:87-90`
- **Parser**: `parser/stmt_class.rs` — fields use `:` (name: default), methods use `=` (name = closure). Reuses `parse_agent_trait_list` from `stmt_agent.rs`
- **Value**: `Value::Class { name, traits, defaults, methods }` (constructor template) + `Value::Object { class_name, id, traits, methods }` (instance handle) in `value.rs:79-92`
- **Object store**: global `OBJECTS: LazyLock<DashMap<u64, IndexMap<String, Value>>>` in `value.rs:95`. Helper functions: `object_store_insert`, `object_store_get_field`, `object_store_set_field`, `object_store_update_nested`
- **Construction** (`apply.rs:104-134`): `ClassName {field: val}` merges field overrides with defaults, inserts into OBJECTS DashMap, returns Object handle
- **Field access** (`apply_helpers.rs:100-107`): dot access on Object checks methods first (with `inject_self`), falls back to DashMap field read
- **Field update** (`exec_stmt.rs:288-292`): `self.field <- val` on Object bypasses `env.reassign`, mutates DashMap directly via `object_store_update_nested`
- **Self injection** (`apply_helpers.rs:169-179`): `inject_self` clones the method closure and binds `self` to the Object value in its env. Called at dot-access time, not at definition time. This means Trait default methods that reference `self.entries` will correctly resolve to the Object's `entries` field.
- **Reference semantics**: `a = b` copies the Object handle (same `id`), both see mutations through the shared DashMap entry
- **Trait conformance**: same as Agent — at ClassDecl eval time, checks required methods exist, injects trait defaults for missing methods
- **Module exports** (`modules.rs`): `ClassDecl { exported: true }` exports the Class value
- **Checker** (`checker/mod.rs`): synths field defaults and method handlers
- **Walker** (`visitor/walk/mod.rs`): `walk_class_decl` visits field defaults and method handlers
- **agent.implements** (`stdlib/agent_ipc.rs`): works for Object values — checks traits list and method map

### Trait Default Methods

- **AST**: `defaults: Vec<AgentMethod>` added to `TraitDecl` in `ast/mod.rs:52`
- **Value**: `defaults: Arc<IndexMap<String, Value>>` added to `Value::Trait` in `value.rs:68`
- **Parser** (`stmt_trait.rs:28-34`): `name = expr` in Trait body → default method. `name: type -> type` → required signature
- **Eval** (`exec_stmt.rs:149-153`): default methods evaluated and stored on Trait value
- **Injection** (`exec_stmt.rs:197-201, 247-251`): at AgentDecl/ClassDecl eval, trait defaults injected into method map if not already present. Defaults injected BEFORE required method check

### How Self + Trait Defaults + Dict Will Interact

When a Class conforms to a Trait with defaults:
1. At ClassDecl eval, default method closures are cloned into the Class's method map
2. These closures capture the environment at Trait definition time
3. At construction (`ClassName {}`), methods are copied to the Object
4. At dot access (`obj.method`), `inject_self` binds `self = obj` in the method's closure env
5. Inside the default method, `self.entries` does Object field access → reads `entries` field from OBJECTS DashMap → returns the `Value::Dict`
6. Then `self.entries.get key` does Dict dot access → dispatches to Dict's built-in `get`

This chain already works for Object fields. The only new piece is Dict dot access dispatch.

### Existing std/store (Reference for Dict Design)

`std/store` in `stdlib/store.rs` is the existing key-value primitive. Dict is essentially store-as-a-value:

- Backed by `STORES: LazyLock<DashMap<u64, StoreState>>` where `StoreState { data: IndexMap<String, Value>, path: Option<PathBuf> }`
- Handle is a Record `{ __store_id: Int }` — passed as first arg to every function
- 12 functions: `create`, `set`, `get`, `update`, `remove`, `keys`, `entries`, `query`, `count`, `clear`, `persist`, `load`
- `query` takes a predicate function, calls it via `builtins::call_value`
- `update` takes a transform function, applies it to current value
- Persistence: optional `path` on create, auto-persists on every write via JSON

Dict should provide all of this as dot-access methods on a first-class value type. The DashMap pattern is identical — just the access pattern changes from `store.set handle key val` to `dict.set key val`.

### Existing Map Type (`%{}`)

lx has `Value::Map(Arc<IndexMap<ValueKey, Value>>)` — immutable maps with `%{key: val}` syntax. These are value types (no mutation, no methods, no reference semantics). Dict is DIFFERENT:

| | Map (`%{}`) | Dict |
|---|---|---|
| Mutability | Immutable | Mutable (DashMap-backed) |
| Methods | None (use builtins) | Built-in dot access |
| Semantics | Value (copy) | Reference (shared) |
| Keys | Any hashable Value | String |
| Analogy | Record but with computed keys | Python dict |

### Relationship Between All Storage Types

```
Record  → immutable named fields, value semantics, dot access
Map     → immutable computed keys, value semantics, no dot access
Object  → mutable named fields, reference semantics, dot access + methods
Dict    → mutable string keys, reference semantics, dot access methods
Store   → mutable string keys, reference semantics, handle-passing functions (DEPRECATED by Dict)
```

Object is to Record as Dict is to Map. Dict replaces Store for Class field usage.

## The Design Problem

After building Class, a verbosity review found the converted Classes are bloated.

### Verbosity Audit Findings (All 11 Patterns)

Ranked by frequency × token cost:

1. **`self.ensure()` boilerplate** — 111 calls across 8 Classes. Every method starts with lazy init. Root cause: wrapping std/store, which needs create at construction time.
2. **Nested ternary chains** — 30+ instances. `a ? b : (c ? d : (e ? f : g))` because lx has no if/elif/else. Language-level issue.
3. **`self.method ()` explicit unit** — 111 calls. Zero-arg methods need `()` because `self.method` returns the closure. 1 wasted token per call.
4. **`<-` is statement not expression** — forces extraction of tiny helper methods for conditional mutation. Can't write `cond ? (self.field <- val) : ()`.
5. **Variable rebinding for intermediates** — 20+ unnecessary `let` bindings that could be inlined.
6. **`{..item, field: val}` spreads** — 20 instances for single-field updates.
7. **`== None` checks** — 12+ instances where pattern matching would be cleaner.
8. **`kv.entries | map (.value)` chains** — 11 instances. Every store access needs this unwrap.
9. **String interpolation boilerplate** — 15-20 instances of unnecessary wrapping.
10. **Repetitive save/load/query** — same code in every collection Class. Should be Traits.
11. **Error propagation ceremony** — `result ^` on every line. User asked: "wouldnt it be less verbose if errors propagated by default?"

Items 1, 8, 10 are solved by Dict + Collection Trait. Items 2, 3, 4, 11 are language-level issues for future ticks.

### Rejected Solutions (DO NOT RE-PROPOSE THESE)

1. **`init` method on Class** — "rust doesnt do this retarded shit and we have traits .. the caller should create their constructor"
2. **Factory functions** (`+kb = (path) { KnowledgeBase {handle: ...} }`) — "still unnecessarily verbose"
3. **std/store handle as a field on Class** — "WHY ARE U DEFINING A STORE INSIDE A CLASS!?!?!?" — the Class IS backed by a DashMap, wrapping another DashMap inside is double indirection
4. **List fields** (`tasks: []`, `entries: []`) with anonymous records — "this using list to represent what is natively objects doesnt seem correct"
5. **Dynamic field access on Objects** (`self.(key)` read/write) — "I dont like this dynamic keys solution either"
6. **Dict as a field with verbose forwarding methods** — user saw `store = (key val meta) { self.entries.set key {...} }` and said "why is this needed? what about batteries included confuses u?" — if Dict is batteries-included, wrapper methods that just forward shouldn't exist
7. **Killing the Class entirely, just using Dict** — user's instruction was specifically "a field on the class can be this batteries included store"

### The User's Design (What to Build)

**1. `Dict` — batteries-included mutable key-value type**

A first-class `Value::Dict` backed by DashMap, with ALL common operations built in via dot access. "Batteries included" means: the Dict handles EVERYTHING generic. This includes not just set/get/keys but also timestamping, metadata attachment, filtering, serialization. If you find yourself writing a Class method that wraps a Dict call, that wrapper is waste — the operation should be on Dict itself.

Built-in methods: `set`, `get`, `keys`, `values`, `entries`, `remove`, `len`, `has`, `clear`, `filter`, `map`, `update`. Plus serialization to markdown. Plus auto-timestamping on set (optional). Plus query/filter with predicates.

Implementation: `Value::Dict { id: u64 }` handle into `DICTS: LazyLock<DashMap<u64, IndexMap<String, Value>>>`. Dot access in `apply_helpers.rs` dispatches to built-in operations. Constructor: `Dict ()`.

**2. `Collection` Trait — shared behavior for Classes holding a Dict**

```lx
Trait Collection = {
  get = (key) { self.entries.get key }
  keys = () { self.entries.keys () }
  values = () { self.entries.values () }
  remove = (key) { self.entries.remove key }
  query = (pred) { self.entries.values () | filter pred }
}
```

Any Class with `entries: Dict ()` conforms to Collection and gets all these for free via Trait defaults. The Trait defaults reference `self.entries` — when injected into a Class, `self` is the Object, `self.entries` reads the Dict field, and `self.entries.get key` calls Dict's built-in get.

**3. Classes define ONLY domain-specific behavior**

The goal state for a collection Class:

```lx
Class KnowledgeBase : [Collection] = {
  entries: Dict ()
}
```

That's it. No methods. Dict provides set/get/keys/remove/filter/query. Collection Trait provides the delegation. The only reason to add a method is if there's genuinely domain-specific logic that can't be a generic Dict operation.

For TaskStore, the state machine (start/submit/audit/pass/fail/revise/complete) IS domain-specific — that stays. But get/list/save/load/create are generic and should come from Dict + Traits.

**4. Markdown as native serialization**

Dict and Class instances serialize to markdown by default. `dict.save path` writes markdown. `dict.load path` reads markdown. Agents read markdown. JSON is for machines.

**5. Open question: default error propagation**

User asked "wouldnt it be less verbose if errors propagated by default?" Currently every fallible call needs `^` to propagate. If errors propagated by default, you'd need explicit handling only when you want to catch. This is a language-level change — significant but would eliminate hundreds of `^` tokens across the codebase. TBD for a future tick.

## Current State of Files

### Broken Package Files (Current Content)

The 5 collection packages were partially refactored from std/store to list fields. They are currently broken — the list-based approach was rejected by the user during the design discussion. They need to be rewritten to use Dict fields once Dict is built.

Current state of each:
- `pkg/knowledge.lx` — uses list field `items: []`, no std/store import. Tests fail (first/Some unwrap issues)
- `pkg/tasks.lx` — uses list field `tasks: []`, free functions `find_task`/`replace_task`. Tests fail
- `pkg/trace.lx` — uses list field `spans: []`. Tests fail except 45_trace_progress
- `pkg/memory.lx` — uses list field `memories: []`. Tests fail
- `pkg/context.lx` — uses list field `items: []` + fixed fields `capacity`/`seq`. Tests fail

### Working Package Files

- `pkg/circuit.lx` — CircuitBreaker class, fixed fields, no collection semantics. Working.
- `pkg/pool.lx` — Pool class, fixed fields (workers, counts). Working.
- `pkg/introspect.lx` — Inspector class, fixed fields. Working.
- `pkg/prompt.lx` — pure functional record builder, not a Class. Working.

### Test Files Needing Updates

- `tests/27_tasks.lx` — uses `TaskStore ()`, `store.create`, `store.get`, `store.start` etc.
- `tests/30_knowledge.lx` — uses `KnowledgeBase ()`, `kb.store`, `kb.get`, `kb.keys`, `kb.query`
- `tests/37_memory.lx` — uses `MemoryStore ()`, `mem.store`, `mem.recall`, `mem.promote`, `mem.tier`
- `tests/38_trace.lx` — uses `TraceStore ()`, `t.record`, `t.score`, `t.all`, `t.summary`, `t.query`
- `tests/45_trace_progress.lx` — uses `TraceStore ()`, `t.record`, `t.improvement_rate`, `t.should_stop`. Currently PASSES.
- `tests/61_context.lx` — uses `ContextWindow {capacity: N}`, `win.add`, `win.usage`, `win.evict`, etc.

### Callers in flows/brain/workgen

All callers were already updated once (from handle-passing to Class methods). They'll need updating again after the Dict refactor, but the changes should be smaller — mostly constructor syntax and potentially fewer method calls if Dict handles operations directly.

Key caller files: `flows/lib/react.lx`, `flows/lib/memory.lx`, `flows/lib/workflow.lx`, `flows/examples/*.lx` (6 files), `brain/lib/*.lx` (10 files), `brain/agents/*.lx`, `brain/main.lx`, `brain/orchestrator.lx`, `workgen/main.lx`, `workgen/tests/run.lx`.

## lx Language Gotchas (MUST READ Before Writing lx Code)

- **`? { }` always parsed as match block** — NEVER write `cond ? { ... }`. Use `cond ? (expr) : other` or extract a method
- **`self.field <- val` is a STATEMENT, not an expression** — cannot appear inside ternary. Must be at statement level. For conditional mutation, extract a helper method
- **Function body extent** — `filter (e) e.key == key | first` makes `| first` part of the filter body. Fix: `filter (e) { e.key == key } | first`
- **`{}` is Unit, not empty Record** — `f x {}` passes Unit
- **Export names shadow builtins** — `+filter` breaks internal `filter` calls. Capture builtin first: `keep = filter`
- **Adjacent string interpolation fails** — `"{a}{b}"` breaks. Use `a ++ b`
- **`first` returns `Some(val)` or `None`** — not the raw value. Unwrap with `^` or `?? default`
- **Multi-arg calls before `?`** — `f a b ? { ... }` binds `?` to `b`. Fix: `(f a b) ? { ... }`

## lx Syntax Reference (For Writing lx Code)

Read these context files for full syntax — don't guess:
- `agent/LANGUAGE.md` — core lx syntax: bindings (`name = val`, `name := val` mutable, `name <- val` reassign), functions (`(params) { body }`), pipes (`x | f`), pattern matching, collections, error handling (`^` propagate, `??` coalesce)
- `agent/AGENTS.md` — Class/Agent/Trait/Protocol syntax, `self` usage, messaging
- `agent/REFERENCE.md` — how to add Value variants, stdlib modules, language features (step-by-step checklists)
- `agent/STDLIB.md` — all stdlib modules and builtins

Key syntax patterns for this task:
```lx
-- bindings
x = 5                           -- immutable
x := 5                          -- mutable
x <- x + 1                      -- reassign mutable

-- functions and pipes
double = (x) { x * 2 }
[1 2 3] | map double            -- pipe
[1 2 3] | filter (x) { x > 1 } -- lambda with block (MUST use block if piping after)

-- classes
Class Foo : [SomeTrait] = {
  field: default_value           -- field (colon)
  method = (params) { body }     -- method (equals)
}
f = Foo {field: override}        -- construction with overrides
f = Foo ()                       -- construction with defaults
f.method args                    -- method call (self auto-injected)
f.field                          -- field read
f.field <- new_value             -- field write (in-place via DashMap)

-- traits with defaults
Trait Describable = {
  describe: {} -> Str            -- required signature (colon)
  summary = () { self.describe () }  -- default method (equals)
}

-- exports and imports
+name = val                      -- export binding
+Class Foo = { ... }             -- export class
use pkg/circuit {CircuitBreaker} -- selective import
use std/time                     -- whole module import

-- error handling
result = fallible_call () ^      -- propagate error
val = x ?? default               -- coalesce None/Err
```

## Concrete End-State Examples

### What "batteries included" looks like at the call site

Dict handles ALL generic operations. Domain data is just fields on the record you pass to `dict.set`. No wrapper methods needed:

```lx
-- KnowledgeBase is just a Class with a Dict field + Collection Trait
kb = KnowledgeBase ()

-- Caller adds domain data (timestamps, metadata) inline at the call site
-- No wrapper method needed — dict.set stores whatever you give it
now = time.now ()
kb.entries.set "auth_module" {
  val: {entry: "src/auth/mod.rs"}
  meta: {source: "reviewer"  confidence: 0.9}
  stored_at: now.iso
}

-- Generic operations come from Dict (via Collection Trait delegation)
entry = kb.get "auth_module"     -- Collection Trait default → self.entries.get
all_keys = kb.keys ()            -- Collection Trait default → self.entries.keys
results = kb.query (e) e.meta.confidence > 0.8  -- Collection Trait default
kb.remove "auth_module"          -- Collection Trait default → self.entries.remove
```

### What KnowledgeBase looks like after refactor

```lx
+Class KnowledgeBase : [Collection] = {
  entries: Dict ()
}
```

That's the entire file (plus the `--` header and `use` for Collection). All operations come from Dict + Collection Trait.

### What TaskStore looks like (has domain logic)

```lx
+Class TaskStore : [Collection] = {
  entries: Dict ()

  -- State machine IS domain-specific — only this stays on the Class
  transition = (id allowed_from new_status extra) {
    task = self.get id
    task == None ? (Err "not found") : {
      (contains? task.status allowed_from) ? {
        base = {..task, status: new_status, updated_at: (time.now ()).iso}
        merged = extra == () ? base : ({..base ..extra})
        self.entries.set id merged
        Ok ()
      } : (Err "invalid transition from {task.status}")
    }
  }

  start = (id) { self.transition id ["todo" "revision"] "in_progress" () }
  submit = (id extra) { self.transition id ["in_progress" "revision"] "submitted" extra }
  audit = (id) { self.transition id ["submitted"] "pending_audit" () }
  pass = (id) { self.transition id ["pending_audit"] "passed" () }
  fail = (id extra) { self.transition id ["pending_audit"] "failed" extra }
  revise = (id) { self.transition id ["failed"] "revision" () }
  complete = (id extra) { self.transition id ["passed"] "complete" extra }
}
```

create/get/list/save/load all come from Dict + Collection. Only the state machine stays.

### What the Collection Trait provides

```lx
Trait Collection = {
  get = (key) { self.entries.get key }
  keys = () { self.entries.keys () }
  values = () { self.entries.values () }
  remove = (key) { self.entries.remove key }
  query = (pred) { self.entries.values () | filter pred }
  len = () { self.entries.len () }
  has = (key) { self.entries.has key }
  save = (path) { self.entries.save path }
  load = (path) { self.entries.load path }
}
```

## Dict Implementation Sketch (Rust)

### Value variant
```rust
// In value.rs
Value::Dict { id: u64 },

// Global store (same pattern as OBJECTS)
static DICTS: LazyLock<DashMap<u64, IndexMap<String, Value>>> = LazyLock::new(DashMap::new);
static NEXT_DICT_ID: AtomicU64 = AtomicU64::new(1);
```

### Constructor
`Dict` should be a TypeName keyword (like `Agent`, `Class`, `Protocol`). When called as `Dict ()`, it creates a new empty dict. Implementation in `apply.rs` — add a match arm for `Value::Dict` construction, or register `Dict` as a builtin function.

Simpler approach: register `Dict` as a builtin function in a stdlib module or in the builtins. `Dict ()` returns `Value::Dict { id }` with a fresh empty IndexMap in DICTS.

### Dot access dispatch
```rust
// In apply_helpers.rs eval_field_access, add arm:
Value::Dict { id } => {
    // Dispatch to built-in dict methods
    match name {
        "set" => /* return a BuiltinFunc that takes (key, val) and inserts */,
        "get" => /* return a BuiltinFunc that takes (key) and looks up */,
        "keys" => /* return a BuiltinFunc that returns key list */,
        "values" => /* return a BuiltinFunc that returns value list */,
        "entries" => /* return a BuiltinFunc that returns {key, value} list */,
        "remove" => /* return a BuiltinFunc that removes by key */,
        "len" => /* return a BuiltinFunc that returns count */,
        "has" => /* return a BuiltinFunc that returns bool */,
        "clear" => /* return a BuiltinFunc that empties */,
        "filter" => /* return a BuiltinFunc(pred) using call_value */,
        "update" => /* return a BuiltinFunc(key, f) using call_value */,
        "save" => /* serialize to markdown, write to path */,
        "load" => /* read markdown from path, deserialize */,
        _ => Ok(Value::None),
    }
}
```

Each method returns a `Value::BuiltinFunc` with the dict `id` pre-applied. The BuiltinFunc captures the id and operates on `DICTS.get(&id)` / `DICTS.get_mut(&id)`.

Reference: `stdlib/store.rs` has the exact same pattern but as module functions instead of dot methods. The Rust code for each operation is nearly identical — just change the handle extraction from `store_id(&args[0])` to using the pre-captured id.

### call_value for predicates
Dict's `filter` and `update` methods need to call lx closures from Rust. Use `crate::builtins::call_value(f, arg, span, ctx)` — see `builtins/call.rs` for the implementation and `builtins/hof.rs` for examples of HOF builtins that call user functions.

## What Needs to Be Built (In Order)

1. **`Value::Dict` type** — new Value variant, DICTS DashMap global store, constructor `Dict ()`
2. **Dict dot-access dispatch** — in `apply_helpers.rs`, handle `Value::Dict` in `eval_field_access`. Built-in methods: set, get, keys, values, entries, remove, len, has, clear, filter, map, update
3. **Dict display/eq/hash** — `value_display.rs`, `value_impls.rs`
4. **Dict test** — `tests/80_dict.lx`
5. **`Collection` Trait** — lx file with default methods delegating to `self.entries`
6. **Refactor 5 collection Classes** — `entries: Dict ()`, conform to Collection, domain-only methods
7. **Update all tests** — 27, 30, 37, 38, 45, 61
8. **Markdown serialization** — Dict/Class serialize to markdown
9. **Update callers** — flows, brain, workgen
10. **Update context files** — TICK, INVENTORY, AGENTS, STDLIB, REFERENCE, DEVLOG

## Files That Matter

| File | What | Why |
|------|------|-----|
| `crates/lx/src/value.rs` | Value enum + stores | Add `Value::Dict`, DICTS global, dict helpers |
| `crates/lx/src/interpreter/apply_helpers.rs` | Dot access dispatch | Add Dict method dispatch |
| `crates/lx/src/interpreter/apply.rs` | Function application | Dict constructor |
| `crates/lx/src/interpreter/exec_stmt.rs` | Statement eval | Dict field in FieldUpdate on Objects |
| `crates/lx/src/value_display.rs` | Display impl | Dict display |
| `crates/lx/src/value_impls.rs` | Eq/Hash impls | Dict eq/hash |
| `crates/lx/src/stdlib/store.rs` | Existing store | Reference impl — same DashMap pattern, 243 lines |
| `crates/lx/src/builtins/call.rs` | call_value helper | Needed for Dict's query/filter/update (call lx predicates) |
| `pkg/*.lx` | 5 collection packages | Rewrite with Dict + Collection |
| `tests/*.lx` | 6 test files | Update for new APIs |
| `agent/GOTCHAS.md` | Parser traps | Read before writing ANY lx code |

## Key Constraints

- No code comments except `--` headers in lx files
- 300 line file limit
- Use justfile recipes: `just diagnose`, `just test`, `just fmt`, `just run`
- No `#[allow()]` macros. No doc strings. No re-exports.
- Verbosity is the enemy — if a pattern repeats, abstract it
- "Batteries included" — if a Class method just forwards to Dict, eliminate it
- Markdown over JSON for agent-facing serialization
- Rust-like design — no magic init, use Traits, caller creates constructor
- Don't wrap stores inside Classes, don't use lists for objects, don't add indirection
- Read `agent/GOTCHAS.md` before writing lx code
