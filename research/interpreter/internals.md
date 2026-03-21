# Interpreter Internals: Value Representation, GC, Closures, Hash Tables, Error Handling

Technical deep dive into the cross-cutting concerns that every interpreter must address,
with specific implementation details from production language runtimes.

---

## 1. Value Representation

Every dynamically-typed language must answer: how do you represent a value that could be an
integer, a float, a boolean, nil, a string, or an arbitrary object — all in the same
variable? The choice profoundly affects memory usage, cache performance, and the cost of
type checks.

### 1.1 Tagged Unions

The simplest approach: a struct with a type tag and a union of possible payloads.

```c
typedef enum { VAL_BOOL, VAL_NIL, VAL_NUMBER, VAL_OBJ } ValueType;

typedef struct {
    ValueType type;
    union {
        bool boolean;
        double number;
        Obj* obj;
    } as;
} Value;
```

**Size**: 16 bytes on 64-bit platforms (8 bytes for the tag + padding, 8 bytes for the
union payload). Arrays of values consume 16 bytes per element.

**Pros:**
- Simple, portable, easy to debug
- Type checks are a simple integer comparison
- Works on any architecture, any compiler

**Cons:**
- 50% space overhead: the tag is 8 bytes (after alignment) but carries only 2 bits of
  information
- Cache unfriendly: arrays of values have half their cache lines occupied by tags
- Every operation must branch on the type tag

**Used by**: clox (default mode), many educational interpreters

### 1.2 NaN Boxing

Exploits the IEEE 754 double-precision format to encode non-double values inside the NaN
bit space.

**IEEE 754 double layout** (64 bits):
```
[S:1][Exponent:11][Mantissa:52]
 63   62      52   51        0
```

A value is NaN when all 11 exponent bits are 1 and the mantissa is non-zero. This means
there are ~2^53 bit patterns that represent NaN, but hardware only ever produces one of
them (the "quiet NaN" with the highest mantissa bit set). The remaining ~2^52 patterns are
free for encoding other types.

**Encoding scheme (one common approach):**

```
Quiet NaN signature: 0_11111111111_1xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
                     ^             ^
                     sign bit      quiet NaN bit (bit 51)
```

Since modern x86-64 CPUs use only 48 bits for virtual addresses, a pointer fits in the
lower 48 bits of a NaN:

| Value type | Bit pattern |
|------------|-------------|
| Double | Any valid IEEE 754 double (not NaN-boxed) |
| Pointer | Sign bit = 1, exponent = all 1s, quiet NaN bit = 1, lower 48 bits = address |
| Nil | `0x7FFC000000000000` (specific NaN pattern) |
| True | `0x7FFC000000000001` |
| False | `0x7FFC000000000002` |
| Integer | Type tag in bits 48-50, 32-bit integer in lower 32 bits |

**Type checking**: To check if a value is a double, verify it is NOT a NaN (exponent bits
are not all 1, or mantissa is zero). To extract a pointer, mask off the upper 16 bits. To
extract an integer, mask the lower 32 bits.

**Performance vs tagged unions:**
- 8 bytes per value instead of 16 — doubles the cache density
- Floats are "free" — no encoding/decoding needed for the most common numeric type
- Type checks use bitwise operations instead of branches
- Trade-off: more complex encoding/decoding logic, harder to debug, not portable to
  architectures with >48-bit address spaces

**Encoding/decoding in C:**

```c
// Double to Value: reinterpret bits, no conversion
Value from_double(double d) {
    return *(Value*)(&d);
}

// Pointer to Value: set sign + NaN signature + address
Value from_obj(Obj* obj) {
    return SIGN_BIT | QNAN | (uint64_t)(uintptr_t)obj;
}

// Value to Pointer: mask off tag bits
Obj* to_obj(Value v) {
    return (Obj*)(uintptr_t)(v & ~(SIGN_BIT | QNAN));
}
```

**Used by**: clox (optional), SpiderMonkey (Firefox), JavaScriptCore (Safari/WebKit),
LuaJIT

Sources:
- [Dynamic Typing and NaN Boxing](https://leonard.swiss/blog/nan-boxing/)
- [Value Representation in JavaScript Implementations](https://wingolog.org/archives/2011/05/18/value-representation-in-javascript-implementations)
- [How Dynamic Languages Handle Data Types](https://deepsource.com/blog/dynamic-values)

### 1.3 Pointer Tagging

Exploits the fact that heap-allocated objects are aligned to 8-byte (or 4-byte) boundaries.
The lowest 2-3 bits of every valid pointer are always zero, so they can store type
information.

**V8's tagged pointer scheme:**

On 64-bit systems, values are 64 bits wide:
- **Smi (Small Integer)**: Lowest bit = 0. The integer value is stored in the upper 31 (or
  63) bits, left-shifted by 1. To extract: arithmetic right shift by 1.
- **HeapObject pointer**: Lowest bit = 1. To dereference: subtract 1 (or mask the bit).

```
Smi:        [integer value << 1][0]
HeapObject: [aligned pointer   ][1]
```

Type check: test the lowest bit. If 0, it is a Smi — no heap allocation, no pointer
chase. This makes integer arithmetic very fast: add two Smis, check for overflow, done.

**Ruby's tagging (CRuby):**

```
Fixnum:   [integer << 1][1]       (lowest bit = 1)
Object:   [pointer     ][00]      (lowest 2 bits = 00)
Symbol:   [id << 8     ][0x0e]    (specific tag pattern)
True:     0x02
False:    0x00
Nil:      0x04
```

Ruby uses the lowest bits for tagging and reserves specific small values for singletons
(true, false, nil). Fixnum integers are stored directly in the tagged value with no heap
allocation.

**Trade-offs vs NaN boxing:**
- Pointer tagging and NaN boxing have similar memory footprints (8 bytes per value)
- Pointer tagging is better for integer-heavy code (Smis are just a shift away)
- NaN boxing is better for float-heavy code (doubles require no encoding at all)
- Both avoid heap allocation for common primitives

Sources:
- [Pointer Compression in V8](https://v8.dev/blog/pointer-compression)
- [Theory and Implementation of Tagged Pointers](https://fedang.net/posts/pointer-tagging/)

### 1.4 Object Headers

Heap-allocated objects need metadata for the runtime. Common header designs:

**Minimal header (clox):**
```c
struct Obj {
    ObjType type;        // enum: string, function, closure, class, ...
    bool isMarked;       // GC mark bit
    struct Obj* next;    // intrusive linked list of all objects (for GC sweep)
};
```

**CPython object header:**
```c
typedef struct {
    Py_ssize_t ob_refcnt;    // reference count
    PyTypeObject* ob_type;   // pointer to type object
} PyObject;
```

For GC-tracked objects, CPython prepends an additional GC header with doubly-linked list
pointers.

**V8 object header:**
- First word: Map pointer (hidden class)
- Subsequent words: in-object properties or other type-specific data

The header design determines per-object overhead. CPython's 16-byte header means even a
small integer object uses 28+ bytes. V8 and Ruby avoid this for small integers via tagging.

---

## 2. Garbage Collection

### 2.1 Reference Counting (CPython)

Every CPython object contains an `ob_refcnt` field. The runtime automatically
increments/decrements this count as references are created/destroyed. When the count reaches
zero, the object is immediately deallocated.

**Implementation:**
```c
#define Py_INCREF(op)  (++(op)->ob_refcnt)
#define Py_DECREF(op)  if (--(op)->ob_refcnt == 0) _Py_Dealloc(op)
```

During deallocation, `_Py_Dealloc` calls the object's type-specific destructor, which
decrements refcounts of contained objects — potentially triggering a cascade of
deallocations.

**Advantages:**
- Deterministic: objects are freed the instant they become unreachable
- Incremental: GC cost is spread across normal execution
- Simple mental model: Python developers rely on deterministic cleanup (file closing, etc.)

**Disadvantage: cycles.**
If object A references B and B references A, both have refcount >= 1 forever, even if
nothing else references them. CPython supplements reference counting with a cycle detector.

**CPython's cycle detector:**

A generational, tracing collector that runs periodically to find and collect reference
cycles:

1. **Generations**: Four generations — young (gen 0), two old generations (gen 1, gen 2),
   and a permanent generation for immortal objects. New objects start in gen 0.

2. **Trigger**: Collection runs when `allocations - deallocations > threshold0` (default:
   700). Gen 1 collects after every 10 gen 0 collections; gen 2 after every 10 gen 1
   collections.

3. **Trial deletion algorithm** (for detecting cycles):
   - **Phase 1**: Copy each object's `ob_refcnt` into a temporary `gc_refs` field
     (stored in spare bits of the GC header's `_gc_prev` pointer).
   - **Phase 2**: For each tracked object, visit its referents via `tp_traverse()`.
     For each internal reference to another tracked object, decrement that object's
     `gc_refs`. After this phase, objects with `gc_refs == 0` have only internal
     references — they are *tentatively unreachable*.
   - **Phase 3**: Objects with `gc_refs > 0` are definitely reachable (they have external
     references). Walk their references, rescuing any tentatively-unreachable objects they
     point to. Objects still unreachable after this phase are garbage.

4. **Breaking cycles**: Each container type implements `tp_clear()` which sets contained
   references to NULL, decrementing refcounts and allowing normal reference counting to
   clean up.

5. **Promotion**: Surviving objects are promoted to the next generation, reducing future
   GC work.

**Free-threaded CPython (3.13+)**: Uses biased reference counting — each object has
separate local and shared refcounts. The owning thread modifies the local count without
synchronization; other threads use atomics on the shared count. GC uses stop-the-world
pauses.

Sources:
- [CPython GC Internals](https://blog.codingconfessions.com/p/cpython-garbage-collection-internals)
- [Design of CPython's Garbage Collector](https://devguide.python.org/garbage_collector/)
- [CPython InternalDocs/garbage_collector.md](https://github.com/python/cpython/blob/main/InternalDocs/garbage_collector.md)

### 2.2 Tracing Garbage Collection

Tracing GC identifies live objects by following references from "roots" (stack, globals,
registers) through the object graph. Anything not reached is garbage.

#### Mark-Sweep

The simplest tracing algorithm. Two phases:

1. **Mark**: Start from roots. For each reachable object, set a mark bit. Recursively
   follow all references, marking transitively reachable objects. Uses a worklist (explicit
   stack or queue) to avoid deep recursion.

2. **Sweep**: Walk the entire heap linearly. Free any object without a mark bit. Clear mark
   bits on surviving objects.

**Tricolor abstraction** (used by clox and many others):
- **White**: Not yet visited (potentially garbage)
- **Gray**: Reachable but references not yet traced (in the worklist)
- **Black**: Reachable and all references traced

The invariant: no black object points to a white object. The mark phase moves objects from
white → gray → black. When the gray set is empty, all whites are garbage.

**clox implementation:**
- Roots: VM stack, global variable table, call frame closures, open upvalues, compiler
  state
- Gray stack: A dynamic array of `Obj*` pointers
- Each object type's `blackenObject()` function knows what references to trace
- Weak references: Before sweeping, `tableRemoveWhite()` cleans interned strings that were
  not marked, preventing dangling pointers
- Trigger: When `bytesAllocated > nextGC`. After collection:
  `nextGC = bytesAllocated * 2`

**Advantages**: Simple, handles cycles naturally.
**Disadvantages**: Stop-the-world pauses, heap fragmentation (freed objects leave holes).

Source: [Crafting Interpreters — Garbage Collection](https://craftinginterpreters.com/garbage-collection.html)

#### Mark-Compact

Like mark-sweep, but after marking, surviving objects are compacted (slid together) to
eliminate fragmentation. Requires updating all pointers to reflect new object locations.
More expensive per collection but eliminates fragmentation and improves cache locality.

Used by: V8's old-generation collector, Hotspot JVM's Serial and Parallel collectors.

#### Semi-Space Copying (Cheney's Algorithm)

Divides memory into two equal halves (from-space and to-space). Allocation uses simple
pointer bumping in from-space. When full:

1. Copy all live objects from from-space to to-space
2. Update all references to point to new locations
3. Swap the roles of the two spaces

**Advantages**: No fragmentation, allocation is O(1) pointer bump, dead objects have zero
cost (they are simply not copied).
**Disadvantages**: Halves the available memory. Copying cost is proportional to live data.

Used by: Erlang BEAM (per-process), V8's young-generation collector (Scavenger).

#### Generational GC

Based on the "generational hypothesis" — most objects die young. Divides the heap into
generations:

- **Nursery (young generation)**: Small, collected frequently. Uses copying collection
  (semi-space or similar). Fast allocation via pointer bumping.
- **Old generation(s)**: Larger, collected rarely. Uses mark-sweep or mark-compact.

Objects surviving a nursery collection are "promoted" (tenured) to the old generation.

**Write barriers**: Since old-generation objects might reference nursery objects, the
runtime must track cross-generational pointers. A write barrier intercepts every pointer
store and records old→young references in a "remembered set." During nursery collection,
the remembered set provides additional roots.

Used by: JVM (multiple collectors), V8, .NET CLR, Lua (incremental generational since 5.4).

#### Incremental and Concurrent GC

**Incremental**: The collector does a small amount of work, then yields to the application
("mutator"). Reduces pause times but requires write barriers to maintain the tricolor
invariant — if the mutator creates a black→white reference while the collector is running,
the white object might be incorrectly collected.

**Concurrent**: The collector runs on a separate thread simultaneously with the mutator.
Even more complex barrier requirements. Used by Go's GC, Java's ZGC and Shenandoah.

### 2.3 Erlang's Per-Process GC

Erlang's approach is unique: each lightweight process has its own small heap (starting at a
few hundred words). GC is per-process using generational semi-space copying (Cheney's
algorithm).

Key properties:
- **No global pauses**: Only the process being collected stops; all other processes
  continue running
- **GC as reductions**: Collection work counts against the process's reduction budget, so
  a large collection naturally preempts the process
- **No cross-process references** (for most data): Since Erlang values are immutable,
  message passing copies data into the receiving process's heap. This eliminates the need
  for cross-process write barriers.
- **Scalability**: With millions of processes each having tiny heaps, GC pauses are
  microseconds, not milliseconds

Source: [Erlang Garbage Collection Documentation](https://www.erlang.org/doc/apps/erts/garbagecollection)

### 2.4 Rust's Ownership as Alternative

Rust eliminates GC entirely through its ownership system:
- Each value has exactly one owner
- When the owner goes out of scope, the value is dropped (deterministic destruction)
- Borrowing rules enforce aliasing XOR mutability at compile time

For an interpreter written in Rust, the host language manages its own memory without GC.
But values in the interpreted language still need lifecycle management — options include:
- Arena allocation (all values in a function/scope share a single allocation)
- Reference counting (`Rc<T>` / `Arc<T>`) with cycle detection
- A tracing GC implemented in Rust (crates like `gc`, `heph-rt`)
- Region-based memory management

---

## 3. Closures and Scope

### 3.1 The Problem

A closure is a function that captures variables from its enclosing scope. The challenge:
those variables might outlive the scope that created them.

```python
def make_counter():
    count = 0
    def increment():
        nonlocal count
        count += 1
        return count
    return increment

c = make_counter()  # make_counter's stack frame is gone
c()                 # but count must still exist → returns 1
c()                 # and be mutable → returns 2
```

The variable `count` lives on `make_counter`'s stack. When `make_counter` returns, that
stack frame is destroyed. But `increment` still needs access to `count`. The runtime must
ensure `count` outlives its original scope.

### 3.2 Lua's Upvalue Mechanism

Lua's solution (also used by clox) is the most elegant and well-documented approach.

**Key data structures:**

```c
typedef struct ObjUpvalue {
    Obj obj;
    Value* location;         // pointer to the variable
    Value closed;            // heap storage for when variable leaves stack
    struct ObjUpvalue* next; // linked list of open upvalues
} ObjUpvalue;
```

**Open vs closed upvalues:**

- **Open**: The upvalue's `location` pointer points to a slot on the stack. The variable
  is still alive in its original scope.
- **Closed**: When the enclosing scope exits, the variable's value is copied into the
  upvalue's `closed` field, and `location` is redirected to point to `&self->closed`.

This is the key insight: code that reads/writes through the upvalue does not need to know
whether the variable is on the stack or the heap. It always dereferences `location`.

**The closing operation:**

```c
static void closeUpvalues(Value* last) {
    while (vm.openUpvalues != NULL &&
           vm.openUpvalues->location >= last) {
        ObjUpvalue* upvalue = vm.openUpvalues;
        upvalue->closed = *upvalue->location;   // copy value from stack
        upvalue->location = &upvalue->closed;    // redirect pointer
        vm.openUpvalues = upvalue->next;         // unlink from open list
    }
}
```

**Sharing**: Multiple closures capturing the same variable share a single upvalue object.
The VM maintains a sorted linked list of open upvalues, ordered by stack address.
`captureUpvalue()` searches this list before creating a new upvalue:

```c
static ObjUpvalue* captureUpvalue(Value* local) {
    ObjUpvalue* prevUpvalue = NULL;
    ObjUpvalue* upvalue = vm.openUpvalues;
    while (upvalue != NULL && upvalue->location > local) {
        prevUpvalue = upvalue;
        upvalue = upvalue->next;
    }
    if (upvalue != NULL && upvalue->location == local) {
        return upvalue;  // already captured — reuse
    }
    // create new, insert in sorted position
    ObjUpvalue* created = newUpvalue(local);
    created->next = upvalue;
    if (prevUpvalue == NULL) vm.openUpvalues = created;
    else prevUpvalue->next = created;
    return created;
}
```

**Compile-time resolution**: The compiler resolves upvalues by walking the chain of
enclosing `Compiler` structs. Each upvalue is tracked as `(index, isLocal)`:
- `isLocal = true`: The variable is a local in the immediately enclosing function.
  Capture it directly from the enclosing function's stack.
- `isLocal = false`: The variable is an upvalue in the enclosing function. Thread it
  through — the closure captures the enclosing function's upvalue.

The `OP_CLOSURE` instruction emits pairs of bytes `(isLocal, index)` for each upvalue.

Sources:
- [Crafting Interpreters — Closures](https://craftinginterpreters.com/closures.html)
- [Closures in Lua (Ierusalimschy & de Figueiredo)](https://www.cs.tufts.edu/~nr/cs257/archive/roberto-ierusalimschy/closures-draft.pdf)
- [Functions & Closures in Lua 5.3](https://poga.github.io/lua53-notes/function_closure.html)

### 3.3 Python's Cell Objects

Python uses a similar but less unified mechanism:

- **Free variables**: Variables used in a function but defined in an enclosing scope.
- **Cell variables**: Variables in an enclosing scope that are captured by an inner function.

When the compiler detects that a variable will be captured, it marks it as a "cell"
variable. Instead of storing the value directly on the stack, the runtime wraps it in a
`cell` object (a simple heap-allocated container). Both the defining scope and the capturing
closures reference the same cell object.

```python
# Bytecode for the enclosing function:
LOAD_CLOSURE    0    # load cell object for 'count'
BUILD_TUPLE     1
LOAD_CONST      1    # the function code object
MAKE_FUNCTION   8    # 8 = has free variables

# Bytecode for the inner function:
LOAD_DEREF      0    # load value from cell object
STORE_DEREF     0    # store value into cell object
```

**Difference from Lua**: Python's cell objects are always heap-allocated — there is no
"open" phase where they live on the stack. This is simpler but means every captured
variable incurs a heap allocation even if the closure never outlives the defining scope.

### 3.4 Closure Conversion and Lambda Lifting

Two compiler transforms that eliminate closures from the language:

**Closure conversion**: Convert every function that captures variables into one that takes
an explicit "environment" parameter. The environment is a struct/record containing the
captured variables.

```
// Before:
fn make_adder(x) { fn(y) { x + y } }

// After closure conversion:
fn make_adder(x) {
    env = { x: x }
    fn(env, y) { env.x + y }  // explicit environment parameter
}
```

**Lambda lifting**: Move all functions to the top level by adding captured variables as
explicit parameters. This eliminates the need for environments entirely but requires
threading values through all call sites.

```
// After lambda lifting:
fn adder(x, y) { x + y }
fn make_adder(x) { partial(adder, x) }
```

Lambda lifting is simpler (no heap allocation for environments) but requires partial
application support and can increase parameter counts significantly in deeply nested code.

---

## 4. String Handling

### 4.1 String Interning

Store only one copy of each unique string in a global table. All references to the same
string content point to the same object.

**How it works:**

1. When a new string is created, hash its contents
2. Look up the hash in the intern table
3. If a matching string exists, return a pointer to it (discard the new string)
4. If not, insert the new string into the table and return it

**Consequences:**
- **O(1) equality check**: Two interned strings are equal if and only if their pointers are
  equal. No character-by-character comparison needed.
- **Reduced memory**: Repeated strings (e.g., common identifiers like `self`, `return`,
  `None`) share a single allocation.
- **Creation cost**: Every string creation requires a hash + table lookup. This is
  amortized by the savings in equality checks.

**Used by:**
- **Lua**: All strings are interned in a global hash table. This makes table key lookup
  very fast — equality is pointer comparison.
- **CPython**: Automatically interns strings matching `[a-zA-Z0-9_]*` and all
  variable/attribute names. Other strings may be interned explicitly with `sys.intern()`.
- **Java**: String literals are interned. `String.intern()` adds strings to the pool.
- **clox**: All `ObjString` values are interned at creation time.
- **Ruby**: Symbols are interned strings. Regular strings are not interned.

Source: [String Interning — Wikipedia](https://en.wikipedia.org/wiki/String_interning)

### 4.2 Rope Data Structures

A rope is a binary tree where leaf nodes hold short strings and internal nodes hold the
concatenation of their children. This makes concatenation O(1) — just create a new internal
node — at the cost of O(log n) random access.

**Structure:**
```
         [weight=6]
        /          \
   "Hello "       [weight=5]
                 /          \
            "world"         "!"
```

Each internal node stores the total length of its left subtree (the "weight"), enabling
efficient indexing: to access character at position i, if i < weight go left, else go
right with i -= weight.

**Operations:**
- **Concatenation**: O(1) — create a new node with the two ropes as children
- **Index**: O(log n) — walk the tree using weights
- **Split**: O(log n) — split at a position by navigating the tree
- **Insert/Delete**: O(log n) — split + concatenate

**Used by**: TruffleRuby (specializing ropes for Ruby's mutable string semantics), some
text editors (e.g., Xi editor), Crop (Neovim), and libraries for heavy text manipulation.

**When to use**: When the language performs many string concatenations (e.g., building up a
large string in a loop). For languages with immutable strings where concatenation returns a
new string, ropes prevent the O(n) copy on each concatenation.

Source: [Rope Data Structure — Wikipedia](https://en.wikipedia.org/wiki/Rope_(data_structure))

### 4.3 Small String Optimization (SSO)

Store short strings inline in the string object rather than in a separate heap allocation.

**How it works in C++:**

A `std::string` object on a 64-bit platform typically contains:
```c
struct string {
    char* data;        // 8 bytes
    size_t size;        // 8 bytes
    size_t capacity;    // 8 bytes
};  // total: 24 bytes
```

With SSO, the 24-byte struct is reused as a character buffer for short strings:
```c
union {
    struct { char* data; size_t size; size_t capacity; } heap;
    struct { char buf[23]; uint8_t remaining; } sso;
    // remaining == 0 means the buffer is full (23 chars)
    // The top bit of remaining distinguishes SSO from heap mode
};
```

This allows strings of up to 22-23 characters to avoid heap allocation entirely.

**Performance impact:**
- Eliminates malloc/free overhead for short strings
- Improves cache locality (the string data is in the object itself, not a separate
  allocation)
- Most strings in typical programs are short: identifiers, keywords, small messages

**In Rust**: The standard `String` does not use SSO, but crates like `compact_str`,
`smartstring`, and `smol_str` provide SSO-enabled string types.

**In interpreters**: An interpreter can use SSO for string values to avoid heap allocation
for identifiers and short literals. Combined with interning, this means most string
operations touch only cache-resident data.

Sources:
- [Small String Optimization](https://pvs-studio.com/en/blog/terms/6658/)
- [compact_str crate](https://docs.rs/compact_str)
- [C++ Small String Optimization](https://giodicanio.com/2023/04/26/cpp-small-string-optimization/)

### 4.4 Immutable vs Mutable Strings

Languages differ on whether strings are mutable:

**Immutable** (Python, Java, Lua, JavaScript, Erlang):
- Enables safe sharing: interning, shared references, concurrent access without locks
- String operations always return new strings
- Hash can be cached in the string object (computed once)
- Concatenation loops are O(n^2) without optimization (ropes or builder patterns)

**Mutable** (Ruby, C, C++):
- In-place modification avoids allocation for append, replace, etc.
- Cannot safely intern mutable strings
- Must be careful with aliasing (modifying one reference affects all views)
- Ruby uses copy-on-write to mitigate: strings share backing storage until one is modified

---

## 5. Hash Table Design

Hash tables are the backbone of dynamic languages — used for dictionaries, object property
storage, module namespaces, string interning, and more.

### 5.1 Collision Resolution: Chaining vs Open Addressing

**Separate chaining**: Each bucket contains a linked list of entries. Collisions add to the
list. Simple but cache-unfriendly due to pointer chasing.

**Open addressing**: All entries live in the array itself. On collision, probe for the next
available slot according to a probing sequence (linear, quadratic, double hashing).

Most modern interpreter hash tables use open addressing for cache locality.

### 5.2 Probing Strategies

**Linear probing**: On collision, try the next slot, then the next, etc. Excellent cache
locality (sequential memory access) but suffers from "clustering" — groups of occupied
slots merge, causing long probe chains.

**Quadratic probing**: Probe at offsets 1, 4, 9, 16, ... (squares). Reduces clustering
but can miss some slots. CPython uses a modified quadratic probing.

**Double hashing**: Use a second hash function to determine the probe step. Eliminates
clustering but loses cache locality (non-sequential probing).

**CPython's perturbed probing**:
```python
probe[i] = (probe[i-1] * 5 + perturb + 1) & mask
perturb >>= 5  # each iteration
```
This combines a linear congruential generator (guaranteeing all slots are visited) with
perturbation from higher hash bits. The `perturb >>= 5` gradually mixes in upper bits,
distributing entries that collide in lower bits. This is effectively a hybrid of linear and
pseudo-random probing.

### 5.3 Robin Hood Hashing

An open-addressing strategy that equalizes probe sequence lengths:

**Core idea**: When inserting, if the current slot is occupied by an entry whose probe
sequence length (PSL) is shorter than the inserting entry's PSL, swap them. The displaced
entry continues probing. This "takes from the rich (short PSL) and gives to the poor
(long PSL)."

**Key property**: Dramatically reduces variance in probe lengths. At load factor 0.9:
- Standard open addressing: PSL variance = 16.2
- Robin Hood: PSL variance = 0.98

At load factor 0.99: standard = 194, Robin Hood = 1.87.

**Consequence**: Lookups can terminate early — if the current slot's PSL is less than what
the searched key would have, the key is not in the table. This bounds the worst-case lookup.

**History in Rust**: Rust's standard `HashMap` originally used Robin Hood hashing with
linear probing. It was replaced by `hashbrown` (a Swiss Table implementation) for even
better performance.

Sources:
- [Robin Hood Hashing](https://programming.guide/robin-hood-hashing.html)
- [Robin Hood Hashing Should Be Your Default](https://www.sebastiansylvan.com/post/robin-hood-hashing-should-be-your-default-hash-table-implementation/)

### 5.4 Swiss Tables

Google's hash table design (2017), now used in Rust's `hashbrown`/`HashMap`, Go 1.24's
built-in map, and Abseil C++.

**Architecture:**

The table consists of two parallel arrays:
1. **Control bytes**: 1 byte per slot, encoding slot state
2. **Slots**: The actual key-value data

**Control byte encoding:**

| Value | Meaning |
|-------|---------|
| `0x80` (`10000000`) | Empty |
| `0xFE` (`11111110`) | Deleted (tombstone) |
| `0x00`-`0x7F` | Full — stores H2 (7-bit fingerprint from hash) |

**Hash splitting:**
- **H1** (upper 57 bits): Determines the starting group index
- **H2** (lower 7 bits): Stored in the control byte as a fingerprint

**SIMD-based group probing:**

Control bytes are processed in groups of 16 (128-bit SIMD register width). To look up a
key:

1. Compute hash, split into H1 and H2
2. H1 selects the starting group (group = `H1 % num_groups`)
3. Load 16 control bytes into a SIMD register
4. Compare all 16 bytes against H2 simultaneously (`_mm_cmpeq_epi8` on x86)
5. The comparison produces a 16-bit bitmask where set bits indicate potential matches
6. For each potential match, verify the actual key
7. Also check for empty slots (compare against `0x80`) — if found, the key is absent
8. If no match and no empty slot, advance to the next group (quadratic probing)

**Performance:**
- Probing 16 slots simultaneously dramatically reduces the number of memory accesses
- On average, a successful lookup touches 1-2 groups
- Go 1.24 reported up to 60% faster map operations after switching to Swiss Tables
- Memory savings up to 70% reported in some workloads

**Fallback**: On platforms without SIMD, Swiss Tables can emulate group operations using
scalar code with bit manipulation, retaining most of the algorithmic benefits.

Sources:
- [Inside Google's Swiss Table](https://bluuewhale.github.io/posts/swiss-table/)
- [hashbrown crate (Rust)](https://docs.rs/hashbrown/)

### 5.5 CPython's Compact Dictionary

Since Python 3.6, dictionaries use a split structure for memory efficiency and insertion-
order preservation:

**Structure:**
```
Hash table (indices):  [  2 | -1 |  0 | -1 |  1 | -1 | -1 | -1 ]
                        ↓          ↓          ↓
Entries (dense):       [ {hash_a, key_a, val_a},    ← index 0
                         {hash_b, key_b, val_b},    ← index 1
                         {hash_c, key_c, val_c} ]   ← index 2
```

- **Indices array** (sparse): Contains indices into the entries array. Each slot is 1, 2,
  4, or 8 bytes depending on the table size. Empty slots contain -1. This array is the
  actual hash table.
- **Entries array** (dense): Stores `(hash, key, value)` tuples in insertion order.

**Benefits:**
1. **Memory savings**: Empty slots in the indices array cost only 1-8 bytes each, not the
   full 24 bytes of an entry
2. **Insertion order**: Entries are stored in insertion order in the dense array
3. **Cache-friendly iteration**: Iterating a dict walks the dense entries array
   sequentially
4. **Reduced probing cost**: Fewer cache misses because the indices array is compact

**Key sharing (Python 3.3+)**: Instance dictionaries of the same class share a single
keys/hashes structure stored on the class. Each instance stores only its values array. This
dramatically reduces memory when many instances of the same class exist.

**Resizing**: When load factor exceeds 2/3, the table doubles. When deleted entries
dominate, it may shrink. Size is always a power of 2 for fast modulo via bitwise AND.

Sources:
- [Python Behind the Scenes #10 — Dictionaries](https://tenthousandmeters.com/blog/python-behind-the-scenes-10-how-python-dictionaries-work/)
- [Python Hash Tables Under the Hood](https://adamgold.github.io/posts/python-hash-tables-under-the-hood/)

---

## 6. Error Handling in Interpreters

### 6.1 Implementation Techniques

An interpreter must handle errors at two levels:
1. **Host-level**: Errors in the interpreter implementation itself (out of memory, stack
   overflow)
2. **Guest-level**: Errors in the interpreted program (exceptions, panics, division by
   zero)

Three primary implementation strategies:

#### setjmp/longjmp

Used by: Lua, CPython (partially), many C-based interpreters.

```c
jmp_buf error_handler;

void execute() {
    if (setjmp(error_handler) != 0) {
        // error was thrown — handle it
        return;
    }
    // normal execution
    run_bytecode();
}

void throw_error(const char* msg) {
    // jump back to the setjmp point, unwinding all C frames
    longjmp(error_handler, 1);
}
```

**How it works**: `setjmp` saves the CPU register state (program counter, stack pointer,
etc.) into a `jmp_buf`. `longjmp` restores that state, effectively rewinding the call
stack to the point where `setjmp` was called.

**Advantages:**
- Very fast on the happy path — no per-function overhead
- Simple to implement for "abort-style" error handling
- Works well for non-recoverable errors (stack overflow, out of memory)

**Disadvantages:**
- Does not call destructors or cleanup code for intermediate frames
- Resources (malloc'd memory, open files) allocated between setjmp and longjmp are leaked
  unless manually tracked
- Interacts poorly with C++ (no stack unwinding)
- Non-local control flow makes reasoning about program state difficult

**Lua's approach**: Lua uses `setjmp`/`longjmp` for its error handling (`lua_pcall` and
`lua_error`). Protected calls set up a `jmp_buf`; `lua_error` triggers `longjmp` back to
the most recent protected call. Since Lua manages its own stack and heap, the leak problem
is mitigated — Lua's GC will eventually collect orphaned objects.

#### Exception Tables (Stack Unwinding)

Used by: C++ exceptions (Itanium ABI), Java, LLVM.

The compiler generates static exception tables stored alongside the code. On the happy path,
there is zero runtime overhead — no `setjmp` calls, no handler registration. When an
exception is thrown:

1. The runtime walks the call stack using frame metadata
2. For each frame, it consults the exception table to find handlers
3. If a handler matches, control transfers to it
4. Cleanup code (destructors, finally blocks) is called for each unwound frame

**Advantages:**
- Zero-cost on the happy path (no `setjmp` overhead)
- Proper cleanup of resources via destructors/finally blocks
- Well-defined semantics for nested exception handlers

**Disadvantages:**
- Throwing is expensive (stack walk + table lookup)
- Exception tables consume static memory (code size increase)
- Complex implementation

#### Result Propagation

Used by: Rust (`Result<T, E>`), Go (error return values), many modern interpreters.

Every function returns a result type that explicitly indicates success or failure. Errors
propagate through normal return values, not through control flow manipulation.

```rust
fn execute_instruction(&mut self) -> Result<(), RuntimeError> {
    match self.read_opcode() {
        OP_ADD => {
            let b = self.pop()?;  // ? propagates errors
            let a = self.pop()?;
            self.push(a + b)?;
        }
        // ...
    }
    Ok(())
}
```

**Advantages:**
- Explicit error handling — no hidden control flow
- Composable with normal language features (pattern matching, combinators)
- No runtime overhead beyond the check-and-branch
- Works naturally with Rust's ownership (no leaked resources)

**Disadvantages:**
- Every instruction dispatch must check the result — this adds a branch per instruction in
  the hot loop
- Can be verbose without syntactic sugar (`?` operator in Rust, `try!` macro)
- Deep call chains accumulate result-checking overhead

**Performance consideration for interpreters**: In a tight dispatch loop, an extra branch
per instruction to check for errors can measurably slow execution. Some interpreters
optimize this by using setjmp/longjmp for exceptional conditions while using result types
for expected errors, combining the benefits of both approaches.

### 6.2 Implementing User-Visible Exceptions

For languages that expose exceptions to users (try/catch, raise/rescue), the interpreter
needs:

**Exception handler stack**: A stack of handler frames, each containing:
- The type(s) of exceptions to catch
- The bytecode offset of the handler code
- The stack depth at the try-block entry (for unwinding the operand stack)

**Throwing an exception**:
1. Search the handler stack for a matching handler
2. If found: unwind the operand stack to the saved depth, jump to the handler's bytecode
   offset
3. If not found: pop the current call frame, repeat the search in the caller

**Stack traces**: Each call frame records source location information (file, line, column).
When an exception is thrown, the interpreter walks the call frames to construct a stack
trace. Source mapping data (a parallel array mapping bytecode offsets to source positions)
enables accurate line numbers.

**CPython's approach**: Each `try` block pushes a handler entry onto a block stack. The
`SETUP_EXCEPT`/`SETUP_FINALLY` instructions (or their modern equivalents, `SETUP_CLEANUP`)
record the handler's target offset and stack depth. `RAISE_VARARGS` triggers the search.

**clox's approach**: clox does not implement exceptions — errors abort execution. This
simplifies the VM considerably. A real implementation would add an exception handler stack
and unwinding logic similar to CPython's.

### 6.3 Source Mapping

Bytecode compilers must maintain a mapping from bytecode positions back to source locations
for error messages, debuggers, and stack traces.

Common approaches:

**Line number table** (CPython, clox): A parallel array where entry `i` gives the source
line for bytecode offset `i`. clox stores this as a simple `int*` array alongside the
bytecode. CPython uses a compressed encoding (delta-encoded line/column numbers).

**Source maps** (JavaScript): A separate mapping file (or inline data) that maps generated
code positions to original source positions. The VLQ-encoded format supports column-level
precision and multiple source files.

**Span annotations**: Attach a source span (start offset, end offset) to each AST node or
IR instruction. More precise than line numbers but consumes more memory.

---

## 7. Design Decision Matrix

A summary of how major languages make these cross-cutting choices:

| Language | Value repr. | GC | Closures | Strings | Hash table | Errors |
|----------|-------------|-----|----------|---------|------------|--------|
| CPython | Tagged ptr (refcount in header) | Refcount + generational cycle detector | Cell objects | Interned (selective), immutable | Open addressing, perturbed probing, compact dict | setjmp + exception tables + result |
| Lua 5.x | Tagged union (TValue) | Incremental mark-sweep (generational in 5.4) | Upvalues (open/closed) | Interned, immutable | Hybrid array+hash table | setjmp/longjmp |
| LuaJIT | NaN boxing | Same as Lua | Same as Lua | Interned, immutable | Same as Lua | setjmp/longjmp |
| Ruby (CRuby) | Tagged pointer (fixnum, symbol) | Mark-sweep + generational | Similar to Python cells | Mutable, CoW | Open addressing | setjmp/longjmp + exception objects |
| V8 | Tagged pointer (Smi) / NaN boxing (older) | Generational: Scavenger (young) + mark-compact (old) | Context objects | Immutable, interned (for keys) | Hidden classes + IC | Exception objects + deoptimization |
| Erlang (BEAM) | Tagged words (tag bits in pointer) | Per-process generational semi-space copying | N/A (no mutable closures) | Immutable binaries | N/A (pattern matching, not hash tables for dispatch) | Exception triples {Class, Reason, Stacktrace} |
| clox | Tagged union or NaN boxing | Mark-sweep | Upvalues (open/closed) | Interned, immutable | Open addressing, linear probing | Abort (no user exceptions) |
| Rust (as host) | Enums (tagged unions, zero-cost) | Ownership (no GC) | Compiler-generated anonymous structs | Immutable `&str` / owned `String` | Swiss tables (hashbrown) | `Result<T, E>` |
