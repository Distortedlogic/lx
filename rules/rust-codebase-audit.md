---
type: "agent_requested"
description: "Deep audit of a Rust codebase for duplication, over-engineering, verbosity, and idiomatic issues."
---

# Rust Codebase Quality Audit

Every item below is a binary check — a violation either exists or it does not. There is no "partially violates" or "could be improved." AGENTS.md R18–R30 and R31 govern all findings and suggested fixes. The audit checks each item across all `.rs` files in all crates.

Run the **High Frequency** list first — these violations are commonly introduced by both humans and AI agents. Run the **Low Frequency** list second — these are rarer structural issues.

---

## High Frequency Checks

- **Inline import paths** - a call site uses `module::path::Type` instead of a short name. Fix: add a `use` statement at the top of the file, use the short name at the call site.
  `grit apply '`$a::$b::$c` where { $a <: not within `use $path;` }' crates/ --language rust --dry-run`

- **Verbose patterns with idiomatic alternatives** - unnecessary `.clone()` where a reference suffices, `match` with exactly two arms (one wildcard) where `if let` suffices, manual loop where an iterator combinator chain does the same work, explicit type annotations where inference works. Fix: use the idiomatic alternative.
  `rg '\.clone\(\)' --type rust crates/`
  `grit apply '`match $expr { $pattern => $body, _ => $default }`' crates/ --language rust --dry-run`
  `grit apply '`for $item in $iter { $vec.push($expr); }`' crates/ --language rust --dry-run`
  `grit apply '`for $item in $iter { if $cond { $body } }`' crates/ --language rust --dry-run`

- **Self-assignments** - `let x = x;` or `let mut x = x;` or `let x = (x);` exists. Fix: make the original binding `mut` or restructure the closure/capture.
  `grit apply '`let $x = $x;`' crates/ --language rust --dry-run`
  Fallback: `rg 'let (mut )?\w+ = \w+;' --type rust crates/`

- **Repeated literals** - a literal value (string, number, array) appears at 2+ call sites without being extracted. Fix: extract to a `const` or `static`.
  `rg -o '"[^"]{4,}"' --type rust --no-filename crates/ | sort | uniq -c | sort -rn | head -30`
  `rg -o '\b[0-9][0-9][0-9][0-9]*\b' --type rust --no-filename crates/ | sort | uniq -c | sort -rn | head -30`

- **#[allow(...)] macros** - any `#[allow(...)]` attribute exists. Fix: remove the allow, fix the underlying warning or remove the unused code.
  `rg '#\[allow\(' --type rust crates/`

- **`&String` / `&Vec<T>` parameters** - a function takes `&String` instead of `&str`, or `&Vec<T>` instead of `&[T]`. Fix: change `&String` to `&str`, `&Vec<T>` to `&[T]`.
  `rg '&String' --type rust crates/`
  `rg '&Vec<' --type rust crates/`
  Exclude struct fields and return types — only flag function/method parameters.

- **`&Arc<T>` / `&Rc<T>` parameters** - a function takes `&Arc<T>` or `&Rc<T>` instead of `&T`. Fix: change the parameter to `&T`. Only keep `&Arc<T>` if the function needs to `Arc::clone` to take shared ownership.
  `rg '&Arc<' --type rust crates/`
  `rg '&Rc<' --type rust crates/`
  For each hit: check if the function calls `Arc::clone`/`Rc::clone` on the param. If not, replace with `&T`.

- **Vec where SmallVec or slice suffices** - a `Vec<T>` used where a `SmallVec<[T; N]>` or `&[T]` slice would avoid heap allocation. Flag: (1) `Vec<T>` function parameters that are only read — should be `&[T]`, (2) `Vec<T>` locals/fields with a known small upper bound (≤ 8 elements typical) in hot paths — should be `SmallVec`, (3) `Vec<T>` return values immediately collected into another container — should return an iterator or accept `&mut Vec` to reuse. Do NOT flag `Vec` used for genuinely unbounded/large collections or where `SmallVec` would add complexity with no measurable benefit.
  `rg 'Vec<' --type rust crates/`
  For each hit in function signatures: check if the param is only read (replace with `&[T]`). For locals/fields: check if size is bounded and small.

- **Intermediate `collect()` into Vec** - an iterator is `.collect::<Vec<_>>()` only to be immediately iterated again. Fix: chain the iterator directly, or collect once at the final consumer. Do NOT flag collects needed for borrowck reasons, parallel iteration (`par_iter`), or indexing.
  `rg '\.collect::<Vec' --type rust crates/`
  `rg '\.collect()' --type rust crates/`
  For each hit: check if the resulting Vec is immediately iterated or only used once. Flag if so.

- **Excessive serde attribute macros** - `#[serde(default)]`, `#[serde(rename = "...")]`, `#[serde(alias = "...")]`, `#[serde(skip_serializing_if = "...")]`, `#[serde(with = "...")]`, `#[serde(deserialize_with = "...")]`, or other serde field/container attributes that exist solely for backwards compatibility with old serialized data, migration from a previous schema, or defensive deserialization of formats we control. Fix: remove the attribute. We control all serialized formats and do not need backwards compatibility. Keep attributes that serve a genuine structural purpose (e.g. `#[serde(tag = "type")]` for internally tagged enums, `#[serde(transparent)]` for newtype wrappers).
  `rg '#\[serde\(' --type rust crates/`

- **Files exceeding 300 lines** - any `.rs` file exceeds 300 lines. Fix: split into multiple files. This is an explicit codebase rule (see CLAUDE.md).
  `find crates/ -name '*.rs' -exec awk 'END { if (NR > 300) print FILENAME, NR }' {} \;`

- **Dead imports / unused dependencies** - a `use` statement imports a name never referenced in the file, or a Cargo.toml dependency is unused. Fix: remove the unused import or dependency.
  `cargo shear` (AST-based, no nightly required, auto-fix with `cargo shear --fix`).

- **Swallowed errors** - `let _ = ...`, `.ok()`, or silent `.unwrap_or_default()` on a `Result` where the error is discarded. Fix: propagate the error, log it, or surface it to the user/UX.
  `rg 'let _ =' --type rust crates/`
  `rg '\.ok\(\)' --type rust crates/`
  `rg '\.unwrap_or_default\(\)' --type rust crates/`

- **Free functions on structs** - a free function takes a struct/enum as its first parameter or accesses that type's fields. Fix: move it to an `impl` block on that type.
  `rg '^(pub )?fn ' --type rust crates/`

- **Extraneous .context()** - a `.context()` call where the context string adds no information beyond the error itself. Fix: remove the `.context()` call.
  `rg '\.context\(' --type rust crates/`

- **Field spreading across structs** - a struct duplicates 2+ fields from another struct instead of holding it as a single field. Fix: hold the source struct as a single field.
  `grit apply '`struct $name { $field: $inner }`' crates/ --language rust --dry-run`
  Review: do any pair of structs share 2+ fields?

- **Extraneous wrappers** - a wrapper type, wrapper function, or intermediate abstraction only forwards to an inner type/function with no added behavior. Fix: inline the inner type/function, remove the wrapper.
  `grit apply '`fn $name(&self) -> $ret { self.$field }`' crates/ --language rust --dry-run`
  `grit apply '`fn $name(&self) -> $ret { self.$inner.$method() }`' crates/ --language rust --dry-run`
  `grit apply '`fn $name(&self, $a: $at) -> $ret { self.$inner.$method($a) }`' crates/ --language rust --dry-run`

- **Duplicate types** - two types share 3+ identical fields. Fix: merge into one type. Do NOT create `From`/`Into` conversions between types that should be merged.
  `grit apply '`struct $name { $field: $inner }`' crates/ --language rust --dry-run`
  Extract field names, compute pairwise intersection, flag pairs sharing 3+ fields.

- **Duplicate methods** - two or more methods across different impl blocks or modules have identical or near-identical bodies. Fix: extract to a shared method, a trait default method, or a single function called by both. Do NOT leave both copies in place.
  Scriptable: extract `fn` bodies via tree-sitter, normalize whitespace, hash, group by hash, flag groups with 2+ methods.

- **Single-implementation traits** - a trait has exactly one implementation and no second is planned/requested. Fix: remove the trait, inline the implementation.
  `rg 'trait (\w+)' --type rust -or '$1' crates/ | sort -u` then for each trait: `rg "impl $t for" --type rust -c crates/`. Flag count = 1.

- **Re-exports from non-defining crates** - a type/function is re-exported from a crate other than the one that defines it. Fix: import directly from the defining crate at usage sites.
  `rg '^pub use ' --type rust crates/`

- **Custom code vs established crate** - custom utility code exists where an established crate provides the same functionality. Fix: use the crate. Check `reference/` submodules first.
  Manual review only.

- **Over-engineering** - unnecessary generics (generic over a single concrete type that is never substituted), unnecessary trait bounds never used polymorphically, unnecessary type parameters, or >2 levels of indirection for a simple operation. Fix: remove the unnecessary abstraction, use concrete types directly.
  `rg 'fn \w+<\w+>' --type rust crates/`
  `rg 'impl<\w+>' --type rust crates/`
  For each generic param, check if it's only ever instantiated with one concrete type.

- **Unwrap defaults that should propagate** - `.unwrap_or(...)` or `.unwrap_or_default()` on a `Result` where the caller should decide how to handle the error. Fix: use the `?` operator.
  `rg '\.unwrap_or\(' --type rust crates/`

- **One-liner forwarding methods** - a struct method body is a single field access or single method call forwarding. Fix: inline the body at call sites, remove the method.
  `grit apply '`fn $name(&self) -> $ret { self.$field }`' crates/ --language rust --dry-run`
  `grit apply '`fn $name(&self) -> $ret { self.$inner.$method() }`' crates/ --language rust --dry-run`

- **Unnecessary parameter threading of statics** - a `LazyLock`, `OnceLock`, `static`, or `const` value is passed as a parameter through method chains instead of being accessed directly at the usage site. Also flag values that *should* be `LazyLock`/`static`/`const` but are instead constructed at a call site and threaded through as parameters. Fix: make the value a module-level `static LazyLock` (or `const` if possible) and access it directly where needed — remove the parameter from all intermediate signatures.
  `rg 'LazyLock|OnceLock' --type rust crates/`
  For each hit: trace whether the value is passed as a function/method parameter. If so, flag it.
  `rg 'fn \w+\(.*&.*LazyLock' --type rust crates/`
  Manual review: look for values constructed once and threaded through 2+ call layers that could be module-level statics.

- **Needless multi-hop redirection** - function A calls B which calls C, where A could call C directly. Fix: remove the intermediate function.
  `grit apply '`fn $name() -> $ret { $other() }`' crates/ --language rust --dry-run`
  Variadics (`$..params`) still unsupported — zero-param only. Review manually for false positives.

- **Mergeable code** - two or more functions, methods, match arms, modules, or impl blocks that share the majority of their logic. Fix: merge into a single unit with a parameter for the difference.
  Manual review. For match arms returning the same value:
  `grit apply '`match $e { $a => $b, $c => $b, $d => $f }`' crates/ --language rust --dry-run`

- **Backwards compatibility code** - any code (shims, feature flags, migration logic, version checks, deprecated re-exports, `_old` / `_v2` type variants, conditional deserialization, fallback parsing, `From`/`Into` conversions between old and new types, or `#[deprecated]` items) that exists solely to handle old data formats, old API shapes, or old serialized state. This codebase is not in production and everything is in development — backwards compatibility is never a concern. Fix: remove the compatibility code entirely.
  `rg '#\[deprecated' --type rust crates/`
  `rg '_old|_v[0-9]|_legacy|_compat|backwards|backward|migrate|migration' --type rust -i crates/`
  `rg 'serde\(alias|serde\(rename' --type rust crates/`
  Manual review: look for `From`/`Into` impls between types that differ only in field names or added fields, conditional logic that handles "old" vs "new" formats, and default values that exist only because old data lacks the field.

---

## Low Frequency Checks

- **Unnecessary Boxing** - a `Box<T>` wrapping a type inside an enum variant or struct field where `T` is small (≤ 16 bytes), `Copy`, or already behind an indirection (e.g. `Vec`, `Arc`, `String`). Also flag any proposal to add `Box<>` to an enum variant that has a protective comment forbidding it. Fix: remove the `Box`, store `T` inline. NEVER box variants of `Node<I>` — see the protective comment in `crates/trading-types/src/tree.rs`.
  `rg 'Box<' --type rust crates/`
  For each hit: check if the boxed type is small/Copy/already-indirect. Flag if so.

- **Root-cause patterns** - existing code patterns that are themselves the root cause of the problem being audited. Fix: flag the pattern as the root cause, do not propose fixes that preserve it.
  Manual review only.
