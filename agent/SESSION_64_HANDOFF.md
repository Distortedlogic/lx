# Session 64 Handoff — Store Promotion + Type Hierarchy Refactor

## What Was Built

### Store as First-Class Value Type
- `Value::Store { id: u64 }` replaces old Record handle `{__store_id: Int}`
- One DashMap (STORES in `stdlib/store.rs`) backs both Store values and Object fields
- Dot-access methods: set, get, keys, values, entries, remove, len, has, clear, filter, query, map, update, save, load, persist, reload
- `Store ()` builtin constructor registered in `builtins/register.rs`
- Reference semantics: `a = b` shares the same backing data
- Store cloning in Class constructor (each instance gets its own Store)
- `len`/`empty?` builtins handle Store

### Collection Trait (`pkg/collection.lx`)
- Trait defaults: get, keys, values, remove, query, len, has, save, load
- All delegate to `self.entries` (a Store field)
- Any Class with `entries: Store ()` conforming to Collection gets these for free

### 5 Collection Packages Rewritten
- `pkg/knowledge.lx` — domain: store, merge, expire
- `pkg/tasks.lx` — domain: create, get, children, list, update, state machine
- `pkg/memory.lx` — domain: store, recall, promote, demote, consolidate, tier
- `pkg/trace.lx` — domain: record, score, summary, query, progress analysis
- `pkg/context.lx` — domain: add, items, capacity, eviction, pinning, pressure

### Type Hierarchy Refactor (Store → Class → Agent)

**OBJECTS DashMap eliminated:**
- Object fields now live in STORES via helpers in `stdlib/store_dispatch.rs`
- `object_insert`, `object_get_field`, `object_update_nested` wrap STORES access

**Value::Agent removed — Agent is a Trait in `pkg/agent.lx`:**
- `Value::Class` has exactly 4 fields: `name`, `traits`, `defaults`, `methods`. No `ClassKind` enum.
- `Agent` keyword auto-imports `pkg/agent {Agent}` and auto-adds "Agent" to traits list
- `Class Worker : [Agent] = { ... }` also works (explicitly adding Agent to traits)
- Agent Trait provides real defaults: init, perceive, reason, act, reflect, handle (auto-dispatch by msg.action via `method_of`), run (yield/loop message loop), think/think_with/think_structured (AI), use_tool/tools (tool hooks), describe (self-description via `methods_of`), ask/tell (inter-agent communication)
- `init`/`on` go into the methods map; `uses` dropped
- Two new builtins: `method_of(obj, name)` — returns a method by name or None; `methods_of(obj)` — returns list of method names
- Shared `inject_traits` helper in `interpreter/traits.rs`
- Display: checks traits list for "Agent" → `<Agent X>` if present, `<Class X>` otherwise

**Value::Protocol removed:**
- Protocol declarations produce `Value::Trait` with non-empty `fields: Arc<Vec<ProtoFieldDef>>`
- Behavioral Traits have empty `fields` vec
- Trait-with-fields is callable as constructor (guard: `!fields.is_empty()`)
- Display: `<Protocol X>` for Traits with fields, `<Trait X>` for behavioral

### Parser Bug Fix
- `is_func_def()` in `parser/func.rs` had ambiguity: `(to_str counter) {name: name}` parsed as a function literal instead of expression + record
- Root cause: no context awareness — parser didn't know it was inside an application chain
- Fix: `application_depth` counter on Parser. Incremented when parsing application arguments. In application context with 2+ bare-ident params and no strong signals, rejects func-def when:
  - Body starts with `{` and looks like a record
  - Body starts with an identifier not matching any param name
- `application_depth` reset when entering record field value parsing (so `f {grader: (a b) {record}}` still works)

### All Clippy Warnings Fixed
- `Value::Func(Box<LxFunc>)` — boxed large variant
- `ProtocolEntry::Field(Box<ProtocolField>)` — boxed large variant
- Visitor methods refactored to use context structs: `TraitDeclCtx`, `AgentDeclCtx`, `RefineCtx`
- `type DeliveryResults` alias in agent_pubsub.rs

## Key Design Decisions

1. **Store IS the primitive** — no separate Dict type. Store already had the DashMap. Just gave it dot-access and a first-class Value type.
2. **Agent IS a Trait** — defined in `pkg/agent.lx`, not a ClassKind enum. `Agent` keyword auto-imports the Trait. `Value::Class` has 4 fields (name, traits, defaults, methods), no init/on/uses fields.
3. **Protocol IS a Trait** — with field requirements instead of method requirements. Same conformance concept applied to data vs behavior.
4. **One DashMap** — STORES backs everything. Objects and Stores share one global store.

## API Changes That Affect Callers
- `save path` / `load path` return Unit (not `Ok ()`). Remove `^` after these calls.
- `remove key` returns the removed value directly (not `Ok ()`). Remove `^` after remove calls.
- Store constructor: `Store ()` (builtin) or `store.create ()` (module function)
- Store dot-access: `s.set key val`, `s.get key`, `s.keys ()`, etc.
