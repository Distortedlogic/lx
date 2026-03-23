# Rust-Specific AST Chunking Patterns

How to map tree-sitter-rust node types to chunking decisions for optimal code retrieval.

## tree-sitter-rust Grammar

**Repo**: [github.com/tree-sitter/tree-sitter-rust](https://github.com/tree-sitter/tree-sitter-rust)

### Top-Level Declaration Nodes

These are direct children of `source_file` and represent the primary chunk boundaries:

| Node Type | Rust Construct | Chunk Behavior |
|---|---|---|
| `function_item` | `fn foo() {}` | Standalone chunk |
| `struct_item` | `struct Foo {}` | Standalone chunk |
| `enum_item` | `enum Bar {}` | Standalone chunk (with variants) |
| `union_item` | `union Baz {}` | Standalone chunk |
| `trait_item` | `trait MyTrait {}` | Chunk with all method signatures |
| `impl_item` | `impl Foo {}` or `impl Trait for Foo {}` | See impl block strategy below |
| `mod_item` | `mod foo {}` or `mod foo;` | Container -- chunk children |
| `type_item` | `type Alias = ...` | Small, group with neighbors |
| `const_item` | `const X: T = ...` | Small, group with neighbors |
| `static_item` | `static X: T = ...` | Small, group with neighbors |
| `use_declaration` | `use crate::foo::Bar` | Never chunk alone -- attach to context |
| `macro_definition` | `macro_rules! foo {}` | Standalone chunk |
| `extern_crate_declaration` | `extern crate foo` | Group with imports |
| `attribute_item` | `#[...]` | Attach to the item it decorates |
| `foreign_mod_item` | `extern "C" {}` | Standalone chunk |

### tags.scm Capture Mappings

The official tree-sitter-rust tags.scm maps Rust constructs to these roles:

```
struct_item      -> @definition.class
enum_item        -> @definition.class
union_item       -> @definition.class
type_item        -> @definition.class
trait_item       -> @definition.interface
function_item    -> @definition.function
  (inside declaration_list) -> @definition.method
mod_item         -> @definition.module
macro_definition -> @definition.macro

call_expression  -> @reference.call
macro_invocation -> @reference.call
impl_item        -> @reference.implementation
```

## Chunking Strategies by Construct

### Functions (`function_item`)

Simple case -- each function is a natural chunk.

**Include with the chunk:**
- The function signature (name, params, return type)
- Preceding `#[...]` attributes (e.g., `#[test]`, `#[derive(...)]`)
- Doc comments (`///` or `//!`)
- The full body

**Metadata to extract:**
- Scope: which module/impl block contains this function
- Visibility: `pub`, `pub(crate)`, private
- Parameters and return type for the signature index

### Structs / Enums (`struct_item`, `enum_item`)

Chunk the full definition including all fields/variants.

**For large enums** (many variants): the entire enum is usually one chunk since variant definitions are compact. If it exceeds the chunk size limit, recurse into individual variants.

**Include:**
- Derive attributes (`#[derive(Debug, Clone, Serialize)]`)
- All fields with types
- Doc comments

### Impl Blocks (`impl_item`)

The most complex case in Rust. An impl block can contain many methods.

**Strategy:**
1. If the entire impl block fits in one chunk, keep it together
2. If too large, split into individual methods BUT:
   - Always include the impl header (`impl Foo` or `impl Trait for Foo`) with each method chunk
   - Include the struct/enum definition as metadata context (signature only, not full body)

**Example split:**
```
Chunk 1: impl header + method_a + method_b
Chunk 2: impl header + method_c + method_d
```

Never orphan a method from its impl header -- the LLM needs to know which type the method belongs to.

### Traits (`trait_item`)

Usually chunk as a single unit since traits define interfaces (method signatures, not full implementations).

**Include:**
- Trait bounds (`trait Foo: Bar + Baz`)
- Associated types
- Default method implementations
- All method signatures

### Modules (`mod_item`)

Two cases:
1. **Inline module** (`mod foo { ... }`): container, chunk its children individually. Include `mod foo` as scope context.
2. **File module** (`mod foo;`): just a declaration, group with other `mod` declarations.

### Use Declarations (`use_declaration`)

**Never chunk use statements alone.** They have no semantic meaning in isolation.

**Strategy:**
- Collect all `use` declarations at the top of a file
- Attach relevant imports to each chunk as metadata
- "Relevant" = imports that are actually referenced within the chunk

### Constants and Statics (`const_item`, `static_item`)

Usually small. Group consecutive const/static items together into one chunk. If a const is large (e.g., a large array literal), it gets its own chunk.

### Macro Definitions (`macro_definition`)

Chunk as a standalone unit. Macros are self-contained.

**Challenge:** `macro_rules!` bodies are opaque to tree-sitter's structural matching. The macro arms are token trees, not fully parsed AST nodes. Treat the entire macro body as a single text block.

### Macro Invocations (`macro_invocation`)

Common case: `derive`, `cfg`, `println!`, etc.
- For attribute macros (`#[derive(...)]`): attach to the item they decorate
- For expression macros (`println!(...)`, `vec![...]`): part of the containing function's chunk
- For item macros (custom `define_routes!()` that generate code): treat as a standalone chunk

## Attribute Handling

Attributes (`#[...]`) always attach to the following item. When chunking:
- Walk backwards from each item to collect its attributes
- `#[cfg(...)]` attributes affect whether code is included -- preserve them
- `#[derive(...)]` tells you what traits are auto-implemented -- useful metadata
- `#[test]` marks test functions -- useful for filtering

## Scope Chain Construction for Rust

For metadata enrichment, build the scope chain by walking up the AST:

```
source_file > mod_item("auth") > impl_item("AuthService") > function_item("login")
```

Produces scope: `auth::AuthService::login`

For `impl Trait for Type`:
```
source_file > mod_item("auth") > impl_item(trait="Display", type="AuthService") > function_item("fmt")
```

Produces scope: `auth::<Display for AuthService>::fmt`

## Rust-Specific Metadata

Beyond generic metadata (scope, imports, siblings), extract Rust-specific information:

| Metadata | Source | Value |
|---|---|---|
| Visibility | `visibility_modifier` node | pub, pub(crate), private |
| Generics | `type_parameters` node | `<T: Clone, E: Error>` |
| Where clause | `where_clause` node | `where T: Display` |
| Lifetime params | `lifetime` nodes | `'a, 'static` |
| Derive traits | `attribute_item` with `derive` | Debug, Clone, Serialize, etc. |
| Async | `function_modifiers` | async fn |
| Unsafe | `function_modifiers` | unsafe fn |
| Return type | child of `function_item` | `-> Result<T, E>` |

## References

- [tree-sitter-rust GitHub](https://github.com/tree-sitter/tree-sitter-rust)
- [tree-sitter-rust tags.scm](https://github.com/tree-sitter/tree-sitter-rust/blob/master/queries/tags.scm)
- [tree-sitter-rust highlights.scm](https://github.com/tree-sitter/tree-sitter-rust/blob/master/queries/highlights.scm)
- [type-sitter (typed wrappers)](https://github.com/Jakobeha/type-sitter)
- [tree-sitter-rust EBNF grammar](https://github.com/mingodad/plgh/blob/main/tree-sitter-rust.ebnf)
- [tree-sitter node types docs](https://docs.rs/tree-sitter/latest/tree_sitter/struct.Node.html)
- [tree-sitter code navigation](https://tree-sitter.github.io/tree-sitter/4-code-navigation.html)
