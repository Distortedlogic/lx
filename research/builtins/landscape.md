# Built-in Functions and Runtime Primitives Across Languages

Research survey covering built-in functions, runtime primitives, and their implementation across seven languages: Python, Rust, JavaScript, Lua, Ruby, Go, and Elixir.

---

## 1. Python

### 1.1 The `builtins` Module

CPython ships ~70 built-in functions, always available without import. The canonical list (Python 3.14):

**Type constructors:** `str`, `bytes`, `int`, `bool`, `float`, `complex`, `list`, `tuple`, `dict`, `set`, `frozenset`, `bytearray`, `memoryview`, `object`, `type`

**Functional:** `abs`, `all`, `any`, `ascii`, `bin`, `callable`, `chr`, `compile`, `delattr`, `dir`, `divmod`, `enumerate`, `eval`, `exec`, `filter`, `format`, `getattr`, `globals`, `hasattr`, `hash`, `hex`, `id`, `input`, `isinstance`, `issubclass`, `iter`, `len`, `map`, `max`, `min`, `next`, `oct`, `open`, `ord`, `pow`, `print`, `range`, `repr`, `reversed`, `round`, `setattr`, `slice`, `sorted`, `sum`, `super`, `vars`, `zip`

**Decorators:** `classmethod`, `staticmethod`, `property`

**Exception base classes:** `BaseException` and ~66 subclasses (`TypeError`, `ValueError`, `KeyError`, etc.)

### 1.2 LEGB Scope Resolution

Python resolves names through four scopes in strict order:

1. **Local** -- current function body
2. **Enclosing** -- outer function (for closures/nested defs)
3. **Global** -- module-level `__dict__`
4. **Built-in** -- the `builtins` module (final fallback)

Every frame holds a reference to its `f_globals` dict, which in turn contains `__builtins__`. When a name lookup exhausts L, E, and G, the interpreter falls back to `__builtins__`. This is a CPython implementation detail -- other implementations (PyPy, GraalPython) may handle the lookup differently.

The recommended way to access builtins programmatically is `import builtins`, not the `__builtins__` global attribute.

Source: [Understanding all of Python, through its builtins](https://tush.ar/post/builtins/)

### 1.3 Why Some Things Are Builtins vs Stdlib

The criteria for Python builtins:

- **Universality**: needed in virtually every program (`print`, `len`, `range`)
- **Performance**: C-level implementation critical (`len` calls `PyObject_Size` in C, avoiding Python dispatch overhead)
- **Language integration**: tied to syntax or core semantics (`type` for the class system, `super` for MRO, `iter`/`next` for `for` loops, `open` for `with` statements)
- **Bootstrap**: needed before the import system is available (`__import__`, `__build_class__`)

Things that are stdlib-but-not-builtin (like `json`, `os`, `sys`) are domain-specific or add significant code weight.

### 1.4 The `__builtins__` Mechanism

When CPython creates a new module, it injects `__builtins__` into the module's `__dict__`. In the `__main__` module, `__builtins__` is the `builtins` module object itself. In other modules, `__builtins__` is the module's `__dict__`. This asymmetry is a CPython implementation detail.

The interpreter bootstrap works as follows:
1. `Python/pylifecycle.c` initializes the interpreter state
2. `Python/bltinmodule.c` defines all builtin functions as C functions (e.g., `builtin_len`, `builtin_print`)
3. These are registered in a `PyMethodDef` array and installed as the `builtins` module
4. Every new frame gets access via the global scope chain

Source: [CPython bltinmodule.c](https://github.com/python/cpython/blob/main/Python/bltinmodule.c)

### 1.5 Special Methods (Dunder Protocol)

Python's builtin functions dispatch to special methods on objects:

| Builtin     | Dispatches to       | Protocol        |
|-------------|---------------------|-----------------|
| `len(x)`    | `x.__len__()`       | Sized           |
| `iter(x)`   | `x.__iter__()`      | Iterable        |
| `next(x)`   | `x.__next__()`      | Iterator        |
| `repr(x)`   | `x.__repr__()`      | Representable   |
| `str(x)`    | `x.__str__()`       | Stringable      |
| `hash(x)`   | `x.__hash__()`      | Hashable        |
| `bool(x)`   | `x.__bool__()`      | Truthy          |
| `x[k]`      | `x.__getitem__(k)`  | Subscriptable   |
| `x + y`     | `x.__add__(y)`      | Addable         |
| `x == y`    | `x.__eq__(y)`       | Equatable       |
| `x in c`    | `c.__contains__(x)` | Container       |

Fallback chains exist: if `__contains__` is missing, Python falls back to `__iter__`; if `__bool__` is missing, it tries `__len__`.

Source: [Python Data Model](https://docs.python.org/3/reference/datamodel.html)

### 1.6 CPython C Implementation

Each builtin is a C function in `Python/bltinmodule.c`. The pattern:

```c
// builtin_len: takes one object, returns its size
static PyObject *
builtin_len(PyObject *module, PyObject *obj)
{
    Py_ssize_t res;
    res = PyObject_Size(obj);  // dispatches to tp_as_sequence->sq_length or tp_as_mapping->mp_length
    if (res < 0) {
        assert(PyErr_Occurred());
        return NULL;
    }
    return PyLong_FromSsize_t(res);
}
```

`PyObject_Size` is the C-level dispatcher. It checks the type's `tp_as_sequence` and `tp_as_mapping` slots for a `sq_length` / `mp_length` function pointer. This is how `len()` works on any type -- the type object's C struct has function pointer slots that correspond to Python's special methods.

The `PyMethodDef` array at the bottom of `bltinmodule.c` maps Python names to C functions with flags indicating calling convention (`METH_O` for single arg, `METH_VARARGS` for variable args, `METH_FASTCALL|METH_KEYWORDS` for print's complex signature).

Source: [CPython source](https://github.com/python/cpython/blob/main/Python/bltinmodule.c)

---

## 2. Rust

### 2.1 Std Primitives

Rust's primitive types are built into the compiler, not the standard library:

**Scalar:** `bool`, `char`, `i8`/`i16`/`i32`/`i64`/`i128`/`isize`, `u8`/`u16`/`u32`/`u64`/`u128`/`usize`, `f32`/`f64`

**Compound:** tuples `(T, U, ...)`, arrays `[T; N]`, slices `[T]`, `str` (string slice), references `&T`/`&mut T`, raw pointers `*const T`/`*mut T`, function pointers `fn(T) -> U`, the never type `!`

These types have inherent methods defined in `std` but are recognized by the compiler itself. For example, `str` cannot have user-defined inherent methods in another crate -- the compiler special-cases primitive types.

### 2.2 The Prelude

The prelude is automatically imported into every Rust module via `use std::prelude::v1::*`. Contents by category:

**Marker traits** (`std::marker`): `Copy`, `Send`, `Sized`, `Sync`, `Unpin`

**Closure/function traits** (`std::ops`): `Fn`, `FnMut`, `FnOnce`, `AsyncFn`, `AsyncFnMut`, `AsyncFnOnce`, `Drop`

**Memory** (`std::mem`): `drop()`, `size_of()`, `size_of_val()`, `align_of()`, `align_of_val()`

**Heap** (`std::boxed`): `Box<T>`

**Conversion** (`std::convert`, `std::borrow`): `AsRef`, `AsMut`, `Into`, `From`, `ToOwned`, `Clone`

**Comparison** (`std::cmp`): `PartialEq`, `Eq`, `PartialOrd`, `Ord`

**Default** (`std::default`): `Default`

**Iterator** (`std::iter`): `Iterator`, `IntoIterator`, `DoubleEndedIterator`, `ExactSizeIterator`, `Extend`

**Core types**: `Option` (with `Some`/`None`), `Result` (with `Ok`/`Err`), `String`, `ToString`, `Vec`

**Edition 2021 additions**: `TryFrom`, `TryInto`, `FromIterator`

**Edition 2024 additions**: `Future`, `IntoFuture`

The prelude is intentionally minimal. The threshold for inclusion is high because additions are effectively permanent -- removing something would break all Rust code. Items earn prelude status by being used in nearly every Rust program and by being traits that enable methods on primitive types (like `Iterator` enabling `.map()` on iterators).

Source: [std::prelude](https://doc.rust-lang.org/std/prelude/index.html), [RFC 0503](https://rust-lang.github.io/rfcs/0503-prelude-stabilization.html)

### 2.3 Built-in Traits

Rust's traits serve the same role as Python's special methods -- they define protocols that integrate with language syntax and built-in operations.

**Formatting:**
- `Display` -- user-facing string representation (`{}` in `format!`), also auto-implements `ToString`
- `Debug` -- programmer-facing representation (`{:?}` in `format!`), derivable via `#[derive(Debug)]`

**Cloning/Copying:**
- `Clone` -- explicit deep duplication via `.clone()`
- `Copy` -- implicit bitwise copy (marker trait, requires `Clone`), enables pass-by-value semantics

**Comparison/Hashing:**
- `PartialEq`/`Eq` -- equality (`==`/`!=`), `Eq` adds reflexivity (NaN breaks `Eq` for floats)
- `PartialOrd`/`Ord` -- ordering (`<`/`>`/`<=`/`>=`), `Ord` requires total ordering
- `Hash` -- hashing for `HashMap`/`HashSet`

**Operator overloading** (`std::ops`):
- `Add`/`Sub`/`Mul`/`Div`/`Rem` -- arithmetic operators
- `Neg`/`Not` -- unary operators
- `BitAnd`/`BitOr`/`BitXor`/`Shl`/`Shr` -- bitwise operators
- `Index`/`IndexMut` -- `[]` indexing
- `Deref`/`DerefMut` -- `*` dereference and auto-deref coercion
- `AddAssign`/`SubAssign`/etc. -- compound assignment

**Conversion:**
- `From`/`Into` -- infallible conversion; implementing `From<T> for U` auto-implements `Into<U> for T`
- `TryFrom`/`TryInto` -- fallible conversion returning `Result`
- `AsRef`/`AsMut` -- cheap reference conversion
- `Borrow`/`BorrowMut` -- borrowed reference with hash/eq guarantees
- `ToOwned` -- generalized `Clone` for borrowed types

**Iterator:**
- `Iterator` -- core trait with `next() -> Option<Item>`, provides 70+ combinator methods (`.map()`, `.filter()`, `.fold()`, `.collect()`, `.zip()`, `.chain()`, `.enumerate()`, etc.)
- `IntoIterator` -- enables `for x in collection` syntax
- `FromIterator` -- enables `.collect()` to produce any collection type
- `ExactSizeIterator` -- accurate `len()` on iterators
- `DoubleEndedIterator` -- iteration from both ends (`.rev()`)
- `Extend` -- append iterator items to existing collections

**I/O:**
- `Read` -- byte-oriented input (`.read()`, `.read_to_string()`)
- `Write` -- byte-oriented output (`.write()`, `.flush()`)
- `BufRead` -- buffered reading (`.lines()`, `.read_line()`)
- `Seek` -- random access

**Error handling:**
- `Error` -- base trait for error types, provides `.source()` chain
- `Display` -- required by `Error` for human-readable messages

Source: [Tour of Rust's Standard Library Traits](https://github.com/pretzelhammer/rust-blog/blob/master/posts/tour-of-rusts-standard-library-traits.md)

### 2.4 Built-in Macros

Macros serve as Rust's "built-in functions" for things that can't be expressed as regular functions:

**Why macros, not functions:**
- Variable argument counts (`println!("x={}, y={}", x, y)` -- can't do this with Rust functions without variadic support)
- Compile-time format string validation (`format!` checks format strings at compile time)
- Code generation (`vec![1, 2, 3]` generates `Vec::new()` + `.push()` calls)

**Core macros:**
- `println!`/`print!`/`eprintln!`/`eprint!` -- stdout/stderr output, expand to `io::_print(format_args_nl!(...))`. `format_args!` creates a stack-allocated `fmt::Arguments` value (no heap allocation)
- `format!` -- returns `String`, wraps `format_args!`
- `vec![...]` -- creates `Vec<T>` from literal values
- `panic!` -- unrecoverable error, unwinds (or aborts)
- `assert!`/`assert_eq!`/`assert_ne!` -- test assertions
- `todo!`/`unimplemented!`/`unreachable!` -- placeholder/unreachable markers
- `cfg!` -- compile-time configuration checking
- `include!`/`include_str!`/`include_bytes!` -- compile-time file inclusion
- `env!`/`option_env!` -- compile-time environment variable access
- `stringify!` -- convert tokens to string literal
- `concat!` -- concatenate literals at compile time
- `dbg!` -- debug print with file/line info, returns value

**Derive macros** generate trait implementations from struct/enum definitions. When you write `#[derive(Debug, Clone, PartialEq)]`, the compiler invokes procedural macros that inspect the type's structure and generate impl blocks. Custom derive macros operate on `TokenStream` input/output via the `proc_macro` crate.

Source: [Rust Book ch20.5 Macros](https://doc.rust-lang.org/book/ch20-05-macros.html), [GitHub issue #17190](https://github.com/rust-lang/rust/issues/17190)

---

## 3. JavaScript

### 3.1 Global Objects

JavaScript's global scope contains built-in objects that serve as namespaces and constructors:

**Value properties:** `Infinity`, `NaN`, `undefined`, `null`, `globalThis`

**Function properties:** `eval()`, `isFinite()`, `isNaN()`, `parseFloat()`, `parseInt()`, `decodeURI()`, `encodeURI()`, `decodeURIComponent()`, `encodeURIComponent()`

**Namespace objects (not constructors):**
- `Math` -- math functions (`Math.floor`, `Math.random`, `Math.PI`, etc.)
- `JSON` -- `JSON.parse()`, `JSON.stringify()`
- `Reflect` -- mirrors Proxy trap operations
- `Atomics` -- atomic operations for SharedArrayBuffer
- `console` -- logging (technically host-defined, not in the spec, but universally available)

**Constructor/type objects:** `Object`, `Function`, `Array`, `String`, `Number`, `Boolean`, `Symbol`, `BigInt`, `Date`, `RegExp`, `Error` (and subclasses), `Map`, `Set`, `WeakMap`, `WeakSet`, `Promise`, `Proxy`, `ArrayBuffer`, `DataView`, `TypedArray` family

Source: [MDN Standard built-in objects](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects)

### 3.2 Prototype Methods

Array, String, and Object have rich method sets on their prototypes. Key Array methods:

**Iteration:** `forEach`, `map`, `filter`, `reduce`, `reduceRight`, `find`, `findIndex`, `findLast`, `findLastIndex`, `some`, `every`, `flatMap`

**Mutation:** `push`, `pop`, `shift`, `unshift`, `splice`, `sort`, `reverse`, `fill`, `copyWithin`

**Non-mutating:** `slice`, `concat`, `flat`, `join`, `includes`, `indexOf`, `lastIndexOf`, `at`, `with`, `toReversed`, `toSorted`, `toSpliced`

**Conversion:** `keys`, `values`, `entries`, `toString`

**Static:** `Array.from`, `Array.of`, `Array.isArray`

The prototype chain means these methods are shared across all instances. When you call `[1,2,3].map(f)`, the engine looks up `map` on `[1,2,3].__proto__` which is `Array.prototype`. Custom objects can shadow or extend prototype methods.

### 3.3 Symbol and Well-Known Symbols

ES6 introduced Symbols as a primitive type for metaprogramming. Well-known symbols customize how the engine treats objects:

| Symbol                     | Customizes                                    |
|----------------------------|-----------------------------------------------|
| `Symbol.iterator`          | `for...of` loops, spread `...`, destructuring |
| `Symbol.asyncIterator`     | `for await...of` loops                        |
| `Symbol.hasInstance`       | `instanceof` operator                         |
| `Symbol.toPrimitive`      | Type coercion (accepts `'number'`/`'string'`/`'default'` hint) |
| `Symbol.toStringTag`      | `Object.prototype.toString()` output          |
| `Symbol.species`          | Constructor used by derived objects in built-in methods |
| `Symbol.isConcatSpreadable`| `Array.prototype.concat()` behavior           |
| `Symbol.match`/`replace`/`search`/`split` | String method delegation to regex-like objects |
| `Symbol.unscopables`      | Properties hidden from `with` statement       |

The design rationale for using symbols instead of string-named magic methods: symbols are guaranteed unique, so they can't collide with user-defined property names. Python uses `__dunder__` names which are just strings and could theoretically collide (though convention prevents this). JavaScript chose guaranteed uniqueness via the type system.

Source: [Customizing ES6 via well-known symbols](https://2ality.com/2015/09/well-known-symbols-es6.html), [MDN Symbol](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Symbol)

### 3.4 Reflect API and Proxy

The Reflect API provides functional equivalents of all fundamental object operations. Every Proxy trap has a corresponding Reflect method:

| Trap / Reflect method   | Intercepts               |
|-------------------------|--------------------------|
| `get`                   | Property read            |
| `set`                   | Property write           |
| `has`                   | `in` operator            |
| `deleteProperty`        | `delete` operator        |
| `apply`                 | Function call            |
| `construct`             | `new` operator           |
| `getPrototypeOf`        | `Object.getPrototypeOf`  |
| `setPrototypeOf`        | `Object.setPrototypeOf`  |
| `isExtensible`          | `Object.isExtensible`    |
| `preventExtensions`     | `Object.preventExtensions` |
| `getOwnPropertyDescriptor` | `Object.getOwnPropertyDescriptor` |
| `defineProperty`        | `Object.defineProperty`  |
| `ownKeys`               | `Object.keys` / `Object.getOwnPropertyNames` |

This represents three levels of metaprogramming: introspection (read-only access to structure), self-modification (changing structure), and intercession (redefining semantics of operations).

Source: [MDN Meta programming](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Meta_programming), [Exploring JS: Proxies](https://exploringjs.com/es6/ch_proxies.html)

---

## 4. Lua

### 4.1 Minimal Builtins Philosophy

Lua's design principle: **mechanisms, not policies**. Rather than providing comprehensive built-in functionality, Lua provides a small set of powerful meta-mechanisms (tables, functions, coroutines) that programmers combine to build what they need.

Lua offers exactly one general mechanism for each major aspect:
- **Tables** for data (arrays, records, objects, modules, namespaces -- all the same thing)
- **Functions** for abstraction
- **Coroutines** for control flow
- **Metatables** for extensibility

This results in an extremely small language: 8 data types (nil, boolean, number, string, userdata, table, function, thread) and ~25 basic library functions.

Source: [Roberto Ierusalimschy, "Small is Beautiful: the design of Lua"](https://web.stanford.edu/class/ee380/Abstracts/100310-slides.pdf)

### 4.2 Basic Library Functions (Global)

These are available without any `require`:

| Function          | Purpose                                              |
|-------------------|------------------------------------------------------|
| `assert(v, msg)`  | Error if v is falsy                                  |
| `collectgarbage`  | GC control interface                                 |
| `dofile(f)`       | Load and execute a Lua file                          |
| `error(msg, lvl)` | Raise an error (never returns)                       |
| `getmetatable(t)` | Get a table's metatable                              |
| `ipairs(t)`       | Iterator for integer-keyed sequences                 |
| `load(chunk)`     | Load a chunk (string or function) as a function      |
| `loadfile(f)`     | Load a file as a function without executing          |
| `next(t, k)`      | Raw table traversal (next key after k)               |
| `pairs(t)`        | Iterator for all table key-value pairs               |
| `pcall(f, ...)`   | Protected call (catch errors, returns ok, result)    |
| `xpcall(f, h)`    | Protected call with error handler                    |
| `print(...)`      | Print values to stdout (using tostring)              |
| `rawequal(a, b)`  | Equality without metamethods                         |
| `rawget(t, k)`    | Table access without metamethods                     |
| `rawset(t, k, v)` | Table assignment without metamethods                 |
| `rawlen(t)`       | Length without metamethods                           |
| `require(mod)`    | Load a module (with caching)                         |
| `select(i, ...)`  | Return args from position i, or count with `"#"`     |
| `setmetatable(t, m)` | Set a table's metatable                           |
| `tonumber(v, b)`  | Convert to number (with optional base)               |
| `tostring(v)`     | Convert to string (calls __tostring metamethod)      |
| `type(v)`         | Return type name as string                           |
| `warn(...)`       | Issue a warning                                      |

The `raw*` functions are notable: they bypass metatables entirely, providing access to the underlying table operations. This is essential for implementing OOP systems and proxies.

### 4.3 Standard Libraries

Lua's standard libraries are loaded as tables, not merged into the global scope:

| Library       | Table     | Scope                                          |
|---------------|-----------|------------------------------------------------|
| String        | `string`  | `byte`, `char`, `find`, `format`, `gmatch`, `gsub`, `len`, `lower`, `match`, `rep`, `reverse`, `sub`, `upper` |
| Table         | `table`   | `concat`, `insert`, `move`, `pack`, `remove`, `sort`, `unpack` |
| Math          | `math`    | `abs`, `acos`, `asin`, `atan`, `ceil`, `cos`, `deg`, `exp`, `floor`, `fmod`, `huge`, `log`, `max`, `maxinteger`, `min`, `mininteger`, `modf`, `pi`, `rad`, `random`, `randomseed`, `sin`, `sqrt`, `tan`, `tointeger`, `type`, `ult` |
| I/O           | `io`      | `close`, `flush`, `input`, `lines`, `open`, `output`, `popen`, `read`, `tmpfile`, `type`, `write` + `stdin`, `stdout`, `stderr` |
| OS            | `os`      | `clock`, `date`, `difftime`, `execute`, `exit`, `getenv`, `remove`, `rename`, `setlocale`, `time`, `tmpname` |
| Coroutine     | `coroutine`| `close`, `create`, `isyieldable`, `resume`, `running`, `status`, `wrap`, `yield` |
| Debug         | `debug`   | `debug`, `gethook`, `getinfo`, `getlocal`, `getmetatable`, `getregistry`, `getupvalue`, `getuservalue`, `sethook`, `setlocal`, `setmetatable`, `setupvalue`, `setuservalue`, `traceback`, `upvalueid`, `upvaluejoin` |
| UTF-8         | `utf8`    | `char`, `charpattern`, `codepoint`, `codes`, `len`, `offset` |

Portability constrains the stdlib: it only offers what ISO C provides. No networking, no threads, no GUI -- these come from external libraries (LuaSocket, lfs, etc.).

Source: [Lua 5.4 Reference Manual](https://www.lua.org/manual/5.4/manual.html)

### 4.4 The C API

Lua communicates with C through a virtual stack. Almost all API calls operate on values on this stack:

```c
// Push values onto stack
lua_pushstring(L, "hello");
lua_pushnumber(L, 42);

// Call a function: push function, push args, call
lua_getglobal(L, "print");  // push the function
lua_pushstring(L, "hi");    // push arg 1
lua_call(L, 1, 0);          // 1 arg, 0 results

// Register a C function as a Lua global
int my_add(lua_State *L) {
    double a = luaL_checknumber(L, 1);
    double b = luaL_checknumber(L, 2);
    lua_pushnumber(L, a + b);
    return 1;  // one return value
}
lua_register(L, "my_add", my_add);
```

All builtins are implemented as C functions registered through this API. The entire Lua standard library is ~6000 lines of C. The stack-based design means no C struct alignment issues, no ABI concerns beyond the Lua C API itself, and C and Lua can exchange any Lua value type naturally.

Source: [Programming in Lua: C API overview](https://www.lua.org/pil/24.html)

### 4.5 Metatables and Metamethods

Metatables are Lua's protocol system (analogous to Python's `__dunder__` methods and Rust's trait implementations):

| Metamethod        | Triggered by          | Purpose                            |
|-------------------|-----------------------|------------------------------------|
| `__index`         | `t[k]` (missing key)  | Delegation/inheritance             |
| `__newindex`      | `t[k] = v` (new key)  | Assignment interception            |
| `__call`          | `t(...)`              | Make tables callable               |
| `__tostring`      | `tostring(t)`         | String representation              |
| `__len`           | `#t`                  | Length operator                    |
| `__eq`            | `==`                  | Equality                          |
| `__lt`/`__le`     | `<`/`<=`              | Ordering                          |
| `__add`/`__sub`/`__mul`/`__div` | arithmetic | Operator overloading      |
| `__concat`        | `..`                  | Concatenation operator             |
| `__gc`            | GC collection         | Destructor/finalizer               |
| `__pairs`         | `pairs(t)`            | Custom iteration                   |
| `__metatable`     | `getmetatable(t)`     | Protect/hide metatable             |

The `__index` metamethod is particularly powerful: it enables prototype-based OOP by chaining tables. When a key is missing, Lua checks `__index` -- if it's a table, lookup continues there (forming a prototype chain like JavaScript). If it's a function, it's called with `(table, key)`.

---

## 5. Ruby

### 5.1 Object Hierarchy

Ruby's class hierarchy has four critical layers:

```
BasicObject          -- absolute root, ~8 methods (new, ==, !, etc.)
  └── Object         -- default root for user classes
        includes Kernel  -- provides "global" functions
        └── Module
              └── Class
```

**BasicObject** is the bare minimum: `new`, `initialize`, `==`, `!=`, `!`, `__send__`, `instance_eval`, `instance_exec`, `__id__`, `equal?`. Exists for creating "blank slate" objects (proxy patterns, DSLs).

**Object** is what all normal classes inherit from. It includes the `Kernel` module, which is where "global functions" actually live.

Source: [Ruby Object hierarchy](https://www.leighhalliday.com/object-hierarchy-in-ruby), [BasicObject docs](https://docs.ruby-lang.org/en/2.1.0/BasicObject.html)

### 5.2 The Kernel Module

`Kernel` is mixed into `Object`, making its methods available everywhere. They appear to be "global functions" but are actually private instance methods on every object:

**I/O:** `puts`, `print`, `printf`, `putc`, `p`, `gets`, `readline`, `readlines`

**Loading:** `require`, `require_relative`, `load`, `autoload`

**Process:** `exec`, `system`, `fork`, `exit`, `abort`, `at_exit`, `trap`, `sleep`

**Conversion:** `Array()`, `Float()`, `Integer()`, `String()`, `Complex()`, `Rational()`, `Hash()`

**Control flow:** `raise`/`fail`, `catch`/`throw`, `loop`, `block_given?`, `lambda`, `proc`

**Reflection:** `caller`, `binding`, `eval`, `global_variables`, `local_variables`, `respond_to_missing?`

**Object creation:** `rand`, `srand`, `open`, `format`/`sprintf`, `select`

**Misc:** `freeze`, `frozen?`, `taint`, `untaint`, `pp`

Ruby's `Kernel#puts` being a private method on every object is what makes `puts "hello"` work everywhere -- you're calling `self.puts("hello")` where `self` is whatever the current object context is.

Source: [Ruby Kernel module](https://docs.ruby-lang.org/en/master/Kernel.html), [Appendix A: Kernel methods](https://rubyreferences.github.io/rubyref/appendix-a.html)

### 5.3 Enumerable Mixin

`Enumerable` is Ruby's answer to iterator protocols. Any class that defines `each` and includes `Enumerable` gets ~50+ methods for free:

`map`, `select`/`filter`, `reject`, `reduce`/`inject`, `flat_map`, `each_with_object`, `group_by`, `sort_by`, `min_by`, `max_by`, `count`, `find`/`detect`, `find_all`, `include?`, `any?`, `all?`, `none?`, `zip`, `take`, `drop`, `first`, `last`, `chunk`, `each_cons`, `each_slice`, `tally`, `sum`, `uniq`, etc.

This is the mixin pattern: define one method (`each`), get dozens of derived methods. `Comparable` works similarly -- define `<=>` (spaceship operator), get `<`, `>`, `<=`, `>=`, `between?`, `clamp`.

---

## 6. Go

### 6.1 The `builtin` Package

Go's `builtin` package is a pseudo-package -- it exists for documentation purposes but its identifiers are predeclared, not imported. The complete list:

**Container operations:**
- `make(T, args)` -- allocate and initialize slices, maps, channels
- `new(T)` -- allocate zero-value `T`, return `*T`
- `append(slice, elems...)` -- append to slice, may reallocate
- `copy(dst, src)` -- copy elements between slices
- `delete(m, key)` -- remove map entry
- `clear(t)` -- clear map or slice (Go 1.21+)

**Inspection:**
- `len(v)` -- length of string, array, slice, map, channel
- `cap(v)` -- capacity of array, slice, channel

**Comparison/selection:**
- `min(x, y...)` -- minimum value (Go 1.21+)
- `max(x, y...)` -- maximum value (Go 1.21+)

**Complex numbers:**
- `complex(r, i)` -- construct complex number
- `real(c)` -- extract real part
- `imag(c)` -- extract imaginary part

**Error handling:**
- `panic(v)` -- stop normal execution, begin panicking
- `recover()` -- regain control of a panicking goroutine (only in `defer`)

**Output:**
- `print(args...)` -- low-level print (for bootstrapping, not for normal use)
- `println(args...)` -- low-level println

**Types:**
- `error` -- the interface type `interface { Error() string }`
- `any` -- alias for `interface{}`
- `comparable` -- constraint for types that support `==` and `!=`

Source: [Go builtin package](https://pkg.go.dev/builtin)

### 6.2 Why Go Keeps Builtins Minimal

Go's design rationale (from Rob Pike and the Go team):

1. **Builtins are special syntax, not normal functions.** `len`, `cap`, `make`, `append` can't be expressed as Go functions because they're generic over types (before Go had generics) and some operate on language-level constructs (channels, maps).

2. **Built-in types substitute for generics.** Slices, maps, and channels are built-in generic types. `make` creates them. `append` and `delete` operate on them. Before Go 1.18 (which added type parameters), these builtins were the only way to have generic container operations.

3. **Simplicity over completeness.** Go's standard library is moderately rich (HTTP server, crypto, compression, encoding), but the language itself deliberately provides minimal built-in operations. Rob Pike: "comprehensibility" is the goal -- features should be orthogonal so they interact in clear, predictable ways.

4. **`panic`/`recover` is not try/catch.** Go deliberately avoids exception-based control flow. `panic` is for truly exceptional situations (programmer errors, impossible states). Normal errors use multiple return values `(result, error)`. `recover()` can only be called inside a `defer` block, limiting where error handling logic can live.

Source: [Go at Google: Language Design in the Service of Software Engineering](https://go.dev/talks/2012/splash.article)

---

## 7. Elixir

### 7.1 The Kernel Module

All functions and macros in `Kernel` are auto-imported into every Elixir module. This module is Elixir's equivalent of Python's builtins.

**Arithmetic:** `+`, `-`, `*`, `/`, `div/2`, `rem/2`, `abs/1`, `ceil/1`, `floor/1`, `round/1`, `trunc/1`, `min/2`, `max/2`, `**/2`

**Comparison:** `==`, `===`, `!=`, `!==`, `<`, `>`, `<=`, `>=`

**Boolean:** `and`/`or` (strict boolean), `&&`/`||` (truthy/falsy), `not`/`!`

**Type guards (usable in pattern match guards):** `is_atom/1`, `is_binary/1`, `is_bitstring/1`, `is_boolean/1`, `is_float/1`, `is_integer/1`, `is_number/1`, `is_function/1`, `is_function/2`, `is_list/1`, `is_map/1`, `is_tuple/1`, `is_pid/1`, `is_port/1`, `is_reference/1`, `is_struct/1`, `is_struct/2`, `is_nil/1`, `is_exception/1`, `is_map_key/2`

**Collection:** `hd/1`, `tl/1`, `length/1`, `elem/2`, `map_size/1`, `tuple_size/1`, `++`, `--`, `<>`

**Pattern matching:** `match?/2`, `in/2`, `../2`, `..///3`

**Definition macros:** `def`, `defp`, `defmacro`, `defmacrop`, `defmodule`, `defstruct`, `defexception`, `defprotocol`, `defimpl`, `defguard`, `defdelegate`, `defoverridable`

**Control flow:** `if/2`, `unless/2`, `raise/1`, `raise/2`, `exit/1`, `throw/1`

**Process:** `spawn/1`, `spawn/3`, `spawn_link/1`, `spawn_monitor/1`, `self/0`, `node/0`, `send/2`

**Pipe and function application:** `|>/2` (pipe), `tap/2` (side-effect in pipe, returns original), `then/2` (transform in pipe, returns result)

**Nested access:** `get_in/2`, `put_in/3`, `get_and_update_in/3`, `pop_in/2`, `update_in/3`

**Sigils:** `~s`, `~r`, `~w`, `~D`, `~T`, `~N`, `~U`

Source: [Kernel module docs](https://hexdocs.pm/elixir/Kernel.html)

### 7.2 The Pipe Operator and Its Effect on Stdlib Design

The pipe operator `|>` takes the result of the left expression and passes it as the **first argument** to the function on the right:

```elixir
"  hello  "
|> String.trim()
|> String.upcase()
|> String.split("")
```

This has profoundly shaped Elixir's standard library design:

1. **Data-first argument convention.** Every Enum, String, Map, List function takes the data structure as its first parameter. This isn't arbitrary -- it's required for pipe compatibility.

2. **Small, composable functions over monolithic ones.** Instead of a single function with many options, Elixir favors chains of small transformations.

3. **Enum module as the universal collection interface.** The `Enum` module works with any type that implements the `Enumerable` protocol: lists, maps, ranges, streams, etc. Functions like `Enum.map/2`, `Enum.filter/2`, `Enum.reduce/3` all take the enumerable as the first argument.

4. **`tap/2` and `then/2` for pipeline ergonomics.** `tap` lets you insert side effects (logging, debugging) without breaking the pipe chain. `then` lets you use an arbitrary function in a pipe.

Source: [Elixir getting started: Enumerables and Streams](https://elixir-lang.org/getting-started/enumerables-and-streams.html)

### 7.3 Kernel.SpecialForms

Some constructs look like functions but are special forms -- they're part of the compiler, not regular functions:

`{}` (tuple literal), `<<>>` (binary/bitstring literal), `%{}` (map literal), `%Struct{}`, `fn -> end` (anonymous function), `case`, `cond`, `receive`, `try`, `for` (comprehension), `with`, `quote`/`unquote` (metaprogramming), `import`, `alias`, `require`, `use`, `__ENV__`, `__MODULE__`, `__DIR__`, `__CALLER__`, `__STACKTRACE__`, `super`, `&` (capture operator)

Source: [Kernel.SpecialForms docs](https://hexdocs.pm/elixir/Kernel.SpecialForms.html)

---

## 8. Module and Import Systems

### 8.1 Python Import Machinery

Python's import system is a multi-stage pipeline:

1. **`sys.modules` cache check** -- if the module is already imported, return the cached version
2. **Finder search** -- iterate `sys.meta_path` finders (`BuiltinImporter`, `FrozenImporter`, `PathFinder`)
3. **Loader execution** -- the finder returns a loader, which creates a module object and executes the module's code into its namespace
4. **Binding** -- the name is bound in the importing module's namespace

**Circular import handling:** When module A imports module B, and module B imports module A, Python returns a partially-initialized module A to B. Module A exists in `sys.modules` (so the cache check succeeds) but its code hasn't finished executing, so some names may be missing. This leads to `ImportError` or `AttributeError` at runtime.

Mitigations: local imports (import inside a function), restructuring to a third module, or using `importlib.import_module()` for deferred imports.

The import system itself is written in Python (`importlib`), but bootstrap versions are "frozen" -- compiled to bytecode and embedded in the interpreter binary, because the import system can't use itself to load itself.

Source: [Python docs: import system](https://docs.python.org/3/reference/import.html)

### 8.2 Rust Module System

Rust separates modules from compilation units:

- **Crate** = compilation unit (one binary or one library), analogous to a "package"
- **Module** = namespace within a crate, declared with `mod` keyword

Key properties:
- Modules within a crate **can** have circular dependencies (module `a` uses items from module `b` and vice versa)
- Crates **cannot** have circular dependencies (the dependency graph is a DAG enforced by Cargo)
- Visibility is explicit: `pub`, `pub(crate)`, `pub(super)`, or private (default)
- `use` statements create name bindings -- they don't "execute" anything

The module tree is resolved entirely at compile time. There is no runtime import machinery, no module cache, no loader chain.

Source: [Rust at scale: packages, crates, and modules](https://mmapped.blog/posts/03-rust-packages-crates-modules)

### 8.3 Node.js Module System

Node.js supports two module systems:

**CommonJS (`require`):**
- Synchronous, blocking load
- Module cache in `require.cache` (keyed by resolved file path)
- Circular dependencies: returns the partially-executed `module.exports` object
- `module.exports` is a plain object; assignments to it are the module's public API

**ES Modules (`import`/`export`):**
- Asynchronous evaluation
- Static analysis of imports/exports at parse time (before execution)
- Circular dependencies: bindings are "live" -- they reference the exporting module's variable, so they see updates even if the exporting module hasn't finished executing
- No manipulable module cache

Source: [Node.js CommonJS modules](https://nodejs.org/api/modules.html)

---

## 9. Reflection and Introspection

### 9.1 Python: `inspect` Module

Python has extensive runtime introspection via the `inspect` module:

- `inspect.getmembers(obj)` -- list all attributes with values
- `inspect.getsource(obj)` -- retrieve source code as string
- `inspect.signature(func)` -- get function signature with parameter info
- `inspect.getfile(obj)` -- get the file where object was defined
- `inspect.getmro(cls)` -- method resolution order
- `inspect.isclass()`, `inspect.isfunction()`, `inspect.ismethod()`, etc. -- type predicates
- `inspect.stack()` / `inspect.currentframe()` -- call stack introspection

Additionally, every object has: `type(obj)`, `dir(obj)`, `vars(obj)`, `hasattr(obj, name)`, `getattr(obj, name)`, `isinstance(obj, cls)`, `issubclass(cls, parent)`. Classes have `__mro__`, `__bases__`, `__subclasses__()`.

Source: [Python inspect module](https://docs.python.org/3/library/inspect.html)

### 9.2 Rust: `Any` Trait

Rust deliberately avoids runtime reflection. The `Any` trait is the minimal concession:

- `TypeId::of::<T>()` -- compile-time computed unique identifier for each type
- `dyn Any` -- trait object that enables runtime type checking
- `.is::<T>()` -- check if the value is type T
- `.downcast_ref::<T>()` -- get `Option<&T>` if the type matches
- `.downcast_mut::<T>()` -- get `Option<&mut T>`
- `Box<dyn Any>` has `.downcast::<T>()` -- returns `Result<Box<T>, Box<dyn Any>>`

Limitations: `Any` can only check concrete types, not trait implementations. You can't ask "does this value implement `Display`?" at runtime. Rust's position: type information should be resolved at compile time; runtime type checks indicate a design that should use enums or trait objects instead.

Source: [std::any](https://doc.rust-lang.org/std/any/index.html), [Understanding Rust's Any Trait](https://leapcell.io/blog/understanding-rust-any-trait)

### 9.3 JavaScript: Reflect API

JavaScript has several layers of introspection:

**Basic:** `typeof`, `instanceof`, `Object.keys()`, `Object.getOwnPropertyNames()`, `Object.getOwnPropertyDescriptor()`, `Object.getPrototypeOf()`

**Reflect API (ES6):** mirrors every Proxy trap as a function -- `Reflect.get()`, `Reflect.set()`, `Reflect.has()`, `Reflect.deleteProperty()`, `Reflect.ownKeys()`, `Reflect.apply()`, `Reflect.construct()`, etc.

**Proxy:** intercepts fundamental operations on objects, enabling transparent wrappers for validation, logging, reactivity (Vue.js uses this), lazy loading, access control, etc.

The combination of Proxy + Reflect enables full metaobject protocol -- you can redefine what property access, assignment, deletion, enumeration, and function invocation mean for any object.

Source: [MDN Reflect](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Reflect)

---

## 10. Error Handling Primitives

### 10.1 Exception-Based (Python, Ruby, JavaScript)

```python
# Python: try/except/else/finally
try:
    result = operation()
except ValueError as e:
    handle_error(e)
else:
    use_result(result)
finally:
    cleanup()
```

Exceptions are objects with inheritance hierarchies. The runtime maintains a call stack and unwinds it when an exception is raised. The error is a first-class value but the control flow mechanism (stack unwinding) is built into the language.

### 10.2 Value-Based (Rust, Go, Elixir)

**Rust:** `Result<T, E>` and `Option<T>` are enums, not special runtime constructs. The `?` operator is syntax sugar for early return on `Err`/`None`. `panic!` exists for unrecoverable errors (like array bounds violation) and unwinds the stack or aborts.

**Go:** Multiple return values `(result, error)`. `error` is an interface, not a special type. `panic`/`recover` exists but is discouraged for normal error handling. `recover()` only works inside `defer` blocks, deliberately limiting its use.

**Elixir:** Pattern matching on tagged tuples `{:ok, result}` / `{:error, reason}`. Also has `try`/`rescue` for OTP compatibility, but idiomatic Elixir uses pattern matching. Process-level error handling via supervisors and `link`/`monitor`.

### 10.3 Protected Calls (Lua)

Lua uses `pcall(f, ...)` and `xpcall(f, handler, ...)` for error handling:
```lua
local ok, result = pcall(function()
    return risky_operation()
end)
if not ok then
    print("Error: " .. result)
end
```

`error()` raises an error (any Lua value, not just strings). `pcall` returns a boolean status and either the result or the error value. This is essentially `Result` semantics implemented as a function rather than a type.

---

## 11. Concurrency Primitives

### 11.1 Built Into the Language

**Go:** Goroutines (`go f()`) and channels (`make(chan T)`, `ch <- v`, `v := <-ch`, `select`) are language primitives. The runtime includes a scheduler that multiplexes goroutines onto OS threads (M:N scheduling). This is arguably Go's most distinctive feature.

**Erlang/Elixir:** Lightweight processes are the fundamental unit. `spawn`, `send`, `receive` are language primitives. The BEAM VM provides preemptive scheduling, per-process garbage collection, and transparent distribution across nodes. Processes share no memory -- all communication is message passing.

**Lua:** Coroutines (`coroutine.create`, `coroutine.resume`, `coroutine.yield`) are cooperative (no preemption). They're not concurrency in the parallel sense but provide the mechanism for implementing concurrent patterns.

### 11.2 Library-Provided

**Python:** `async`/`await` syntax is in the language, but the event loop (`asyncio`) is a stdlib module. Threading (`threading`) and multiprocessing (`multiprocessing`) are also stdlib. The GIL prevents true parallelism in threads for CPU-bound work.

**Rust:** `async`/`await` syntax is in the language, `Future` trait is in `core`, but the runtime/executor is a third-party library (`tokio`, `async-std`). `std::thread` provides OS threads. `std::sync` provides `Mutex`, `RwLock`, `Arc`, `mpsc` channels.

**JavaScript:** `Promise` and `async`/`await` are in the language. The event loop is provided by the runtime (browser or Node.js), not the language spec.

---

## Sources

- [Understanding all of Python, through its builtins](https://tush.ar/post/builtins/)
- [Python Built-in Functions docs](https://docs.python.org/3/library/functions.html)
- [Python Data Model](https://docs.python.org/3/reference/datamodel.html)
- [CPython bltinmodule.c](https://github.com/python/cpython/blob/main/Python/bltinmodule.c)
- [LEGB Rule - Real Python](https://realpython.com/python-scope-legb-rule/)
- [std::prelude - Rust](https://doc.rust-lang.org/std/prelude/index.html)
- [RFC 0503: Prelude Stabilization](https://rust-lang.github.io/rfcs/0503-prelude-stabilization.html)
- [Rust 2021 Prelude additions](https://doc.rust-lang.org/stable/edition-guide/rust-2021/prelude.html)
- [Rust 2024 Prelude additions](https://doc.rust-lang.org/edition-guide/rust-2024/prelude.html)
- [Tour of Rust's Standard Library Traits](https://github.com/pretzelhammer/rust-blog/blob/master/posts/tour-of-rusts-standard-library-traits.md)
- [Why println! and vec! are macros - GitHub issue #17190](https://github.com/rust-lang/rust/issues/17190)
- [Rust Book: Macros](https://doc.rust-lang.org/book/ch20-05-macros.html)
- [MDN: Standard built-in objects](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects)
- [MDN: Symbol](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Symbol)
- [Customizing ES6 via well-known symbols](https://2ality.com/2015/09/well-known-symbols-es6.html)
- [MDN: Meta programming (Proxy/Reflect)](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Meta_programming)
- [Exploring JS: Proxies](https://exploringjs.com/es6/ch_proxies.html)
- [Lua 5.4 Reference Manual](https://www.lua.org/manual/5.4/manual.html)
- [A Look at the Design of Lua](https://cacm.acm.org/research/a-look-at-the-design-of-lua/)
- [Small is Beautiful: the design of Lua (slides)](https://web.stanford.edu/class/ee380/Abstracts/100310-slides.pdf)
- [Programming in Lua: C API](https://www.lua.org/pil/24.html)
- [Ruby Kernel module](https://docs.ruby-lang.org/en/master/Kernel.html)
- [Ruby BasicObject docs](https://docs.ruby-lang.org/en/2.1.0/BasicObject.html)
- [Ruby Object hierarchy](https://www.leighhalliday.com/object-hierarchy-in-ruby)
- [Go builtin package](https://pkg.go.dev/builtin)
- [Go at Google: Language Design](https://go.dev/talks/2012/splash.article)
- [Effective Go](https://go.dev/doc/effective_go)
- [Elixir Kernel module](https://hexdocs.pm/elixir/Kernel.html)
- [Elixir Kernel.SpecialForms](https://hexdocs.pm/elixir/Kernel.SpecialForms.html)
- [Elixir: Enumerables and Streams](https://elixir-lang.org/getting-started/enumerables-and-streams.html)
- [Node.js CommonJS modules](https://nodejs.org/api/modules.html)
- [Rust at scale: packages, crates, modules](https://mmapped.blog/posts/03-rust-packages-crates-modules)
- [Python inspect module](https://docs.python.org/3/library/inspect.html)
- [Rust std::any](https://doc.rust-lang.org/std/any/index.html)
- [Understanding Rust's Any Trait](https://leapcell.io/blog/understanding-rust-any-trait)
- [std::io - Rust](https://doc.rust-lang.org/std/io/index.html)
- [Rust Book: Error Handling](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
- [PEP 492: Coroutines with async and await syntax](https://peps.python.org/pep-0492/)
- [Concurrency in Go vs Erlang](https://dev.to/pancy/concurrency-in-go-vs-erlang-595a)
