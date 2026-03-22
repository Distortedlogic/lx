# Rust Codebase Quality Audit

Every item below is a binary check — a violation either exists or it does not. There is no "partially violates" or "could be improved." AGENTS.md R18–R30 and R31 govern all findings and suggested fixes. The audit checks each item across all `.rs` files in all crates.

Run the **High Frequency** list first — these violations are commonly introduced by both humans and AI agents. Run the **Low Frequency** list second — these are rarer structural issues.

---

## High Frequency Checks

- **Inline import paths** - a call site uses `module::path::Type` instead of a short name. Fix: add a `use` statement at the top of the file, use the short name at the call site.
  `grit apply '`$a::$b::$c`where { $a <: not within`use $path;` }' crates/ --language rust --dry-run`

- **Missing or underused preludes** - a library crate that exports 3+ types/functions commonly imported together by downstream crates does not have a `prelude` module, or a downstream crate imports individual items from a dependency that already provides a prelude. Both sides of the violation: (1) **missing prelude** — a library crate has 3+ public items that appear in `use` statements across 2+ downstream crates, but no `pub mod prelude` exists. Fix: create `src/prelude.rs` with `pub use` for the commonly imported items, declare `pub mod prelude;` in `lib.rs`. (2) **prelude exists but not used** — a downstream crate imports individual items from a crate that has a prelude covering those items (e.g., `use trading_types::tree::Node;` instead of `use trading_types::prelude::*;`). Fix: replace individual imports with `use <crate>::prelude::*;` and remove the now-redundant individual `use` lines. (3) **stale prelude** — a prelude exists but does not re-export items that are commonly imported by downstream crates, or re-exports items that no downstream crate uses. Fix: add missing items, remove unused ones.
  `rg 'pub mod prelude' --type rust crates/`
  `find crates/ -name 'prelude.rs' -print`
  For each crate without a prelude: `rg 'use <crate_name>::' --type rust crates/ -o --no-filename | sort | uniq -c | sort -rn | head -20` — if 3+ items appear across 2+ files, the crate needs a prelude.
  For each crate with a prelude: `rg 'use <crate_name>::(?!prelude)' --type rust crates/` — flag individual imports that the prelude already covers.

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

- **Cargo dependency version not hoisted to workspace** - a crate-level `Cargo.toml` declares a dependency version directly instead of using `workspace = true`. Fix: declare the dependency with its version/features/source in the workspace root `Cargo.toml` under `[workspace.dependencies]`, and reference it with `dep.workspace = true` in the crate-level `Cargo.toml`.
  `rg 'version\s*=' crates/*/Cargo.toml`
  For each hit: check whether the dependency uses `workspace = true`. Flag if a version is specified directly in a crate-level file.

- **Cargo dependency using string shorthand** - a dependency uses the shorthand string form (e.g., `thiserror = "2.0.18"`) instead of the object/table form (e.g., `thiserror = { version = "2.0.18" }`). This applies to both `[workspace.dependencies]` and crate-level `[dependencies]`. Fix: convert to object notation.
  `rg '^\w+ = "[^"]*"$' Cargo.toml crates/*/Cargo.toml`

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

- **Extracted function with single call site** - a non-pub function is called from exactly one place. Fix: inline the body at the call site and delete the function. Exception: (1) inlining would push the file over 300 lines, (2) the function is recursive, (3) the function represents a genuinely self-contained algorithm whose name communicates a distinct contract.
  For each non-pub function: `rg '\bfn_name\b' --type rust crates/` — if count is 2 (definition + call), inline.

- **Extraneous .context()** - a `.context()` call where the context string adds no information beyond the error itself. Fix: remove the `.context()` call.
  `rg '\.context\(' --type rust crates/`

- **Field spreading across structs** - a struct duplicates 2+ fields from another struct instead of holding it as a single field. Fix: hold the source struct as a single field.
  `grit apply '`struct $name { $field: $inner }`' crates/ --language rust --dry-run`
  Review: do any pair of structs share 2+ fields?

- **Duplicate types** - two types share 3+ identical fields. Fix: merge into one type. Do NOT create `From`/`Into` conversions between types that should be merged.
  `grit apply '`struct $name { $field: $inner }`' crates/ --language rust --dry-run`
  Extract field names, compute pairwise intersection, flag pairs sharing 3+ fields.

- **Duplicate methods** - two or more methods across different impl blocks or modules have identical or near-identical bodies. Fix: extract to a shared method, a trait default method, or a single function called by both. Do NOT leave both copies in place.
  Scriptable: extract `fn` bodies via tree-sitter, normalize whitespace, hash, group by hash, flag groups with 2+ methods.

- **Re-exports from non-defining crates** - a type/function is re-exported from a crate other than the one that defines it. Fix: import directly from the defining crate at usage sites.
  `rg '^pub use ' --type rust crates/`

- **Custom code vs established crate** - custom utility code exists where an established crate provides the same functionality. Fix: use the crate. Check `reference/` submodules first.
  Manual review only.

- **Over-engineering** - unnecessary generics (generic over a single concrete type that is never substituted), unnecessary trait bounds never used polymorphically, unnecessary type parameters, or >2 levels of indirection for a simple operation. Fix: remove the unnecessary abstraction, use concrete types directly.
  `rg 'fn \w+<\w+>' --type rust crates/`
  `rg 'impl<\w+>' --type rust crates/`
  For each generic param, check if it's only ever instantiated with one concrete type.

- **Inappropriate defaulting** - a value is defaulted (via `Default::default()`, `unwrap_or_default()`, `unwrap_or(0)`, `unwrap_or(false)`, literal zero/empty initialization, `Option::None` as a stand-in, or `#[serde(default)]`) where the default silences a bug, masks a missing value, or is otherwise inappropriate. Reasons a default can be inappropriate: (1) **bug silencing** — an error or `None` signals a real problem but the default makes the code continue as if nothing happened, (2) **semantic incorrectness** — the default value (e.g. `0`, `""`, `false`, `Vec::new()`) is not a valid/meaningful value in context (a zero price is not "no price," an empty string is not "no name"), (3) **silent data loss** — a failed parse, missing field, or dropped result is replaced with a default instead of surfacing the absence, (4) **incorrect aggregation** — a default zero or empty value participates in sums, averages, min/max, or counts and skews the result, (5) **deferred panic** — the default creates an invalid state that causes a harder-to-debug failure later (e.g. defaulting an ID to 0 then using it as a map key), (6) **control flow masking** — a boolean defaulted to `false`/`true` hides the fact that the condition was never actually evaluated, (7) **option-as-default** — `Option<T>` with `None` used where the value is always expected to be present, making every access site pay for an `unwrap`/`if let` that can never legitimately be `None`. Fix: propagate the error, make the field non-optional, require explicit construction, or use a builder/constructor that validates.
  `rg '\.unwrap_or_default\(\)|\.unwrap_or\(0|\.unwrap_or\(false|\.unwrap_or\(""|Default::default\(\)' --type rust crates/`
  `rg '#\[serde\(default\)' --type rust crates/`
  Manual review: for each hit, determine whether the default is semantically valid in context or whether it masks an error/absence.

- **Unwrap defaults that should propagate** - `.unwrap_or(...)` or `.unwrap_or_default()` on a `Result` where the caller should decide how to handle the error. Fix: use the `?` operator.
  `rg '\.unwrap_or\(' --type rust crates/`

- **Mergeable code** - two or more functions, methods, match arms, modules, or impl blocks that share the majority of their logic. Fix: merge into a single unit with a parameter for the difference.
  Manual review. For match arms returning the same value:
  `grit apply '`match $e { $a => $b, $c => $b, $d => $f }`' crates/ --language rust --dry-run`

- **String literals instead of enums** - a string literal (e.g. `"buy"`, `"sell"`, `"pending"`, `"error"`) is used to represent a value from a fixed, known set of variants — in struct fields, function parameters, return values, match arms, HashMap keys, or comparisons — where an enum would provide exhaustiveness checking, typo prevention, and refactorability. Reasons this is inappropriate: (1) **no exhaustiveness** — `match` on a string requires a wildcard arm, so adding a new variant compiles silently instead of producing errors at every unhandled site, (2) **typo fragility** — `"recieve"` vs `"receive"` compiles and runs but silently does the wrong thing, (3) **no tooling support** — rename/find-all-references/go-to-definition do not work on string values, (4) **unnecessary allocation** — `String` fields and parameters heap-allocate where a `Copy` enum would be zero-cost, (5) **unclear domain** — the set of valid values is implicit (scattered across match arms and `if` chains) rather than explicit in a type definition. Fix: define an enum with the known variants, replace all string occurrences with enum variants. If the strings cross a serialization boundary, derive `serde::Serialize`/`Deserialize` on the enum.
  `rg '"[a-z_]{2,}"' --type rust crates/`
  Manual review: for each string literal that appears in comparisons (`==`, `!=`, `match`), check whether the set of possible values is fixed and known. Flag if so.

- **String-based enum matching** - an enum value is converted to a string (via `.to_string()`, `format!`, `Display` impl, `.as_str()`, `Into<String>`, or serde serialization) and then matched/compared as a string instead of matching on the enum variant directly. This defeats the purpose of having an enum — the compiler cannot check exhaustiveness on string comparisons, and renaming a variant's display string silently breaks the match. Patterns to flag: (1) `enum_val.to_string() == "variant"` or `match enum_val.to_string().as_str() { "variant" => ... }`, (2) serializing an enum to a string then comparing/switching on the string, (3) passing an enum through a string-typed channel/field and parsing it back, (4) `format!("{}", enum_val)` or `format!("{:?}", enum_val)` used for dispatch rather than display, (5) `.as_str()` on an enum followed by string comparison when a direct `match` on the enum would work. Fix: match on the enum variant directly. If the enum is in a different crate, import it. If the string comes from an external source, parse it into the enum first (via `FromStr` or serde) then match on the enum.
  `rg '\.to_string\(\)\s*==\s*"' --type rust crates/`
  `rg 'match.*\.to_string\(\)' --type rust crates/`
  `rg '\.as_str\(\)\s*==\s*"' --type rust crates/`
  `rg 'format!\("(\{\}|\{:\?\})",.*\)\s*==' --type rust crates/`
  Manual review: for each hit, check whether the value being stringified is an enum. Flag if a direct `match` on the enum would work.

- **Backwards compatibility code** - any code (shims, feature flags, migration logic, version checks, deprecated re-exports, `_old` / `_v2` type variants, conditional deserialization, fallback parsing, `From`/`Into` conversions between old and new types, or `#[deprecated]` items) that exists solely to handle old data formats, old API shapes, or old serialized state. This codebase is not in production and everything is in development — backwards compatibility is never a concern. Fix: remove the compatibility code entirely.
  `rg '#\[deprecated' --type rust crates/`
  `rg '_old|_v[0-9]|_legacy|_compat|backwards|backward|migrate|migration' --type rust -i crates/`
  `rg 'serde\(alias|serde\(rename' --type rust crates/`
  Manual review: look for `From`/`Into` impls between types that differ only in field names or added fields, conditional logic that handles "old" vs "new" formats, and default values that exist only because old data lacks the field.

---

## Low Frequency Checks

- **Newtype with no added behavior** - a struct wraps a single field but adds no methods, no invariant enforcement, and no trait impls beyond derives. Fix: use the inner type directly everywhere and delete the wrapper. Exception: (1) the newtype enforces a semantic distinction the compiler should track (e.g., `Meters(f64)` vs `Feet(f64)`), (2) it implements an external trait on the inner type that cannot be impl'd directly due to the orphan rule.
  `grit apply '`struct $name($inner);`' crates/ --language rust --dry-run`
  For each single-field tuple struct: check if it has any impl block with methods beyond construction/access. Flag if not.

- **Trait with single implementation** - a trait exists with exactly one concrete implementation and is not used polymorphically anywhere (no `dyn Trait`, no `impl Trait` in function signatures, no generic bounds). Fix: remove the trait and use the concrete type directly. Exception: (1) the trait is required by an external framework, (2) it enables mocking in tests that actually exist, (3) it is `pub` in a library crate whose downstream consumers depend on it.
  `rg 'trait \w+' --type rust crates/`
  For each trait: `rg 'impl .* for' --type rust crates/` — flag if only one concrete impl exists and no `dyn`/`impl Trait` usage appears.

- **Re-export-only module** - a `mod.rs` or module file contains only `pub use` re-exports and `mod` declarations with no logic, constants, or type definitions. Fix: import directly from the defining modules at usage sites and delete the re-export module.
  `rg '^pub use ' --type rust crates/`
  For each module file: check if it contains anything beyond `pub use` and `mod` declarations. Flag if not.

- **Unnecessary mod.rs intermediary** - a directory's `mod.rs` contains only a single `mod child;` plus `pub use child::*` (or selective re-exports of that child) and no other logic. The directory exists solely to namespace a single child file. Fix: rename the child file to the directory name (e.g., `run_thread/context.rs` → `run_thread.rs`), delete `mod.rs`, and remove the now-empty directory. No import path changes are needed — the module path is unchanged.
  `find crates/ -name 'mod.rs' -exec wc -l {} +`
  Flag short `mod.rs` files (≤5 lines). For each: check if it only declares one child module and re-exports from it.

- **Unnecessary Boxing** - a `Box<T>` wrapping a type inside an enum variant or struct field where `T` is small (≤ 16 bytes), `Copy`, or already behind an indirection (e.g. `Vec`, `Arc`, `String`). Also flag any proposal to add `Box<>` to an enum variant that has a protective comment forbidding it. Fix: remove the `Box`, store `T` inline. NEVER box variants of `Node<I>` — see the protective comment in `crates/trading-types/src/tree.rs`.
  `rg 'Box<' --type rust crates/`
  For each hit: check if the boxed type is small/Copy/already-indirect. Flag if so.

- **Root-cause patterns** - existing code patterns that are themselves the root cause of the problem being audited. Fix: flag the pattern as the root cause, do not propose fixes that preserve it.
  Manual review only.
