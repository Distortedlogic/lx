# Topology Simplification Audit

Every item below is a binary check — a violation either exists or it does not. The audit identifies **knots**: points where control flow or data flow passes through an unnecessary intermediate layer — a function, type, module, or abstraction boundary that redirects without adding substantive logic. A knot is justified if it serves at least one of: (1) the target is called from 3+ distinct sites, (2) inlining would exceed the 300-line file limit, (3) the indirection represents a genuine abstraction boundary (e.g., trait with 2+ impls, type with distinct invariants, module with distinct responsibilities). A knot with none of these justifications is extraneous and must be unwound.

Every check requires reading actual code, tracing call chains, and understanding data lifecycles — these cannot be found by regex alone. Surface-level syntactic issues (identity transforms, one-liner forwarding, single-use extraction, etc.) belong in `rules/rust-audit.md` or `rules/dioxus-audit.md`, not here.

- **Proxy/handle types** — A type whose methods all follow the same indirection pattern: acquire a lock/reference/wrapper, then forward to the inner type's method. The entire type is a proxy that adds no behavior beyond access mediation. How to find: identify types that wrap another type behind `RwLock`, `Mutex`, `Arc`, `RefCell`, `Option`, or similar. Read every method on the wrapper type. If every method (or all but 1-2) follows the pattern `self.inner.lock().method()` / `self.inner.read().method()` / `self.inner.as_ref().method()`, the type is a proxy. Fix: expose the inner type directly (e.g., expose the `RwLock<T>` and let callers lock it), or provide a single `with_inner(|inner| ...)` method instead of N forwarding methods. Exception: the wrapper enforces an invariant that callers must not bypass (e.g., ensures writes go through a validation step).
  `rg 'struct \w+' --type rust crates/ -A 5`
  For each struct wrapping a locked/ref-counted inner type: read every `impl` method. Count how many are lock-and-forward vs. substantive logic.

- **Shuttle types** — A struct that is constructed at one site, passed to exactly one consumer, and immediately destructured or consumed. It exists only to group parameters for transfer between two functions. How to find: for each struct definition, search for construction sites (`TypeName {` or `TypeName::new(`) and consumption sites. If there is exactly 1 construction site and 1 consumption site (the consumer destructures it or calls a single method), the type is a shuttle. Fix: pass the fields directly as parameters, or construct the destination type directly at the source. Exception: the struct has 6+ fields (parameter list would be unwieldy) or appears in a public API.
  `rg 'struct \w+' --type rust crates/`
  For each non-pub struct: count construction sites and consumption sites. Flag if both are 1.

- **Intermediary representations** — A type that exists as a middle step in a conversion chain A→B→C, where B holds a subset of A's data and its only purpose is to be converted to C. How to find: for each `impl From<X> for Y`, check if Y is then converted to Z somewhere. If A→C is possible (no information is added in the A→B or B→C step that isn't available at the A→C boundary), B is intermediary. Also flag types that mirror another type's fields (3+ overlap) without adding semantics. Fix: convert directly from A to C and delete B. Exception: B serves an independent purpose (is stored, queried, or used in 2+ distinct contexts beyond the conversion chain).
  `rg 'impl From<' --type rust crates/`
  For each From impl: trace what happens to the target type. Is it converted again? Is it stored independently?

- **Result/wrapper nesting chains** — A result or output type that wraps another result type, adding only 1-2 fields per layer. Creates LayerC { results: Vec<LayerB { results: Vec<LayerA> }> } hierarchies where the intermediate layers exist only for grouping. How to find: look for struct fields of type `Vec<OtherResultType>` where the outer struct adds fewer than 3 fields beyond the inner collection. Trace how the outer type is consumed — if consumers always immediately unwrap to get the inner results, the outer layer is unnecessary. Fix: flatten into a single result type or return a tuple of (results, metadata).
  `rg 'struct.*Result' --type rust crates/`
  `rg 'Vec<.*Result' --type rust crates/`
  For each result type: check if it's wrapped by another result type that adds minimal fields.

- **Dual-representation enums** — An enum where multiple variants hold different representations of the same concept, requiring pattern matching on every access to provide a uniform interface. Every accessor method must match all variants to return the same logical value. How to find: look for enums where 3+ methods each match on all variants and extract the same-named field or compute the same value from each variant. Fix: extract the common interface into a stored struct (computed once at construction), or collapse the variants if the representations can be unified. Exception: the variants genuinely represent different states with different capabilities (not just different storage strategies for the same data).
  `rg 'enum \w+' --type rust crates/`
  For each enum: read all methods in impl blocks. Count how many methods match on all variants to provide a uniform accessor.

- **Call chain depth (>4 hops)** — For the system's key operations (evolution loop, evaluation pipeline, data loading, websocket handling), trace the full call chain from entry point to the function that does actual work. Count how many function boundaries the data crosses. If intermediate functions add no logic beyond parameter forwarding, timing instrumentation, or orchestration of a single sub-call, they are knots. How to find: start from the top-level entry points. For each, follow the call chain, reading each function body. Record each hop. At each hop, classify: does this function (a) add substantive logic/branching, (b) only forward parameters to a single next function, or (c) only add timing/logging around a single call? Type (b) and (c) are knots. Fix: inline the knot functions into their callers. Exception: the function is called from 3+ sites, or inlining would exceed 300 lines.
  Entry points to trace:
  `rg 'pub fn run_evolution' --type rust crates/`
  `rg 'pub fn evaluate_population' --type rust crates/`
  `rg 'pub fn run_generation' --type rust crates/`
  `rg 'pub async fn' --type rust crates/` (for server/websocket entry points)
  For each: follow the call chain, count hops, classify each intermediate function.

- **Repeated lock/access acquisition** — Multiple method calls on a locked/wrapped resource in the same scope, each independently acquiring the lock or reference. The lock is acquired N times for N reads when a single acquisition could serve all of them. How to find: in any function body, look for 2+ calls to methods on the same handle/wrapper type within the same scope. If each call independently acquires a lock (visible in the wrapper's method bodies), the pattern is a repeated acquisition. Fix: acquire the lock once at the caller and call methods on the inner type directly. This often pairs with "proxy/handle types" — fixing the proxy also fixes the repeated acquisition.
  `rg '\.read\(\)' --type rust crates/`
  `rg '\.lock\(\)' --type rust crates/`
  For each locked type: find functions that call 2+ methods on it in sequence.

- **Multi-pass collection iteration** — The same collection is iterated multiple times in a single function to extract different information, when a single pass could compute all results simultaneously. How to find: in any function body, look for the same variable name appearing as the iterator source in 2+ separate `for` loops or `.iter()` chains. Fix: fold all computations into a single iteration. Exception: intermediate iterations produce results needed by subsequent iterations (data dependency), or the collection is consumed (moved) by the first iteration.
  Manual review of functions >50 lines that process collections.
  `rg 'for .* in &' --type rust crates/`
  For each function with multiple loops: check if they iterate the same collection.

- **Configuration parameter threading** — A config value is extracted from a struct/context, passed as a function parameter, received by the callee, and either (a) passed further to another function, or (b) used once trivially. The value could be accessed directly from the config at the point of use. How to find: look for functions that take 4+ parameters where 2+ are config-like values (probabilities, limits, flags, thresholds) that originated from a single config struct. Trace where those parameters come from at the call site — if they're all extracted from `self.config` or a similar source available to the callee, they're threaded unnecessarily. Fix: have the callee access the config directly (from `self`, a static, or a context). Exception: the function is a pure algorithm that should be decoupled from the config source for testability, AND tests actually exercise it with different configs.
  `rg 'fn \w+\(' --type rust crates/ -A 5`
  For each function with 4+ parameters: check if 2+ come from the same config struct at call sites.
