# Interpreter Architectures & VM Design Across Languages

A comprehensive survey of interpreter and virtual machine design, covering architectural
patterns, dispatch techniques, and language-specific implementations.

---

## 1. Interpreter Architecture Families

### 1.1 Tree-Walking Interpreters

A tree-walking interpreter traverses an AST directly, evaluating each node during the walk.
There is no compilation step; the source is parsed into a tree and executed immediately.

**How they work:**

1. Parse source code into an AST.
2. Recursively traverse the tree (typically via the Visitor pattern or pattern matching).
3. At each node, evaluate the operation: look up variables in an environment, apply
   operators, call functions by evaluating argument subtrees then the body.
4. Return values propagate up through the recursion.

**When they are appropriate:**

- Prototyping and educational implementations
- Languages where startup latency matters more than throughput
- REPLs and configuration DSLs where programs are short-lived
- Situations where simplicity and correctness are prioritized over speed

**Performance characteristics:**

- Pointer chasing through heap-allocated tree nodes causes poor cache locality
- Each AST node visit requires a virtual dispatch or pattern match
- Typically 10-100x slower than bytecode interpreters on tight loops
- However, the gap narrows for short-lived or I/O-bound programs

An important counterpoint: Perl's AST interpreter is faster than CPython's bytecode
interpreter. Perl augments its AST with a "next node" pointer that linearizes execution
order, largely eliminating the tree-walking overhead. This demonstrates that the AST vs
bytecode distinction is not a clean performance boundary.

**Notable examples:**

- **Crafting Interpreters Part II (jlox)**: Bob Nystrom's tree-walking Lox interpreter in
  Java, implemented in under 2,000 lines
- **Early Ruby (MRI before YARV)**: Ruby 1.8 used an AST interpreter; YARV (Ruby 1.9+)
  replaced it with a bytecode VM
- **Simple Lisp/Scheme interpreters**: The classic metacircular evaluator is a tree-walker
- **GNU Octave**: Used AST interpretation before switching to bytecode

Source: [Crafting Interpreters — A Tree-Walk Interpreter](https://craftinginterpreters.com/a-tree-walk-interpreter.html)

---

### 1.2 Bytecode Interpreters

A bytecode interpreter compiles source code to a compact intermediate representation
(bytecode), then executes that bytecode on a virtual machine. This two-phase approach
separates concerns: the compiler handles parsing/analysis once, while the VM focuses on
efficient execution.

**The compilation pipeline:**

```
Source code → Lexer → Parser → AST → Compiler → Bytecode
                                                    ↓
                                              Virtual Machine
```

**Why bytecode is faster than tree-walking:**

1. **Compact representation**: Bytecode instructions are 1-4 bytes each, fitting more
   instructions in a cache line than pointer-heavy AST nodes
2. **Sequential memory access**: Instructions are laid out linearly in memory, enabling
   hardware prefetching
3. **Reduced indirection**: A switch/jump table dispatch replaces recursive tree traversal
4. **Optimization opportunities**: The compiler can perform constant folding, dead code
   elimination, and peephole optimization before the VM ever runs

**Typical bytecode format:**

- Each instruction has a 1-byte opcode followed by 0-N bytes of operands
- Constants are stored in a separate pool, referenced by index
- Instructions implicitly communicate through a stack or register file

**Performance**: Bytecode interpreters are typically 5-20x faster than tree-walkers.
Adding computed gotos for dispatch can yield another 15-25% improvement. JIT compilation
on top of bytecode interpretation can approach native code speed.

Source: [Crafting Interpreters — A Virtual Machine](https://craftinginterpreters.com/a-virtual-machine.html)

---

### 1.3 Register-Based vs Stack-Based VMs

The two dominant bytecode VM architectures differ in how operands are communicated
between instructions.

#### Stack-Based VMs

Operands are pushed onto and popped from an implicit operand stack. Instructions consume
their inputs from the top of the stack and push results back.

```
; compute a + b
LOAD a      ; push a
LOAD b      ; push b
ADD         ; pop two, push sum
STORE c     ; pop into c
```

**Properties:**

- **Compact bytecode**: No operand fields needed for most instructions; stack position is
  implicit
- **Simple code generation**: A recursive AST walk naturally maps to stack operations
- **More instructions per operation**: Each value movement requires explicit push/pop
- **Examples**: JVM, CPython, CLR, Ruby YARV, Crafting Interpreters' clox

#### Register-Based VMs

Instructions specify source and destination registers (virtual, not hardware) explicitly.
Registers map to slots in the function's stack frame.

```
; compute a + b
ADD R2, R0, R1    ; R2 = R0 + R1
```

**Properties:**

- **Fewer instructions**: Binary operations are single instructions, not push-push-op
  sequences
- **Larger instructions**: Each instruction must encode register operands (8-18 bits each)
- **Better optimization surface**: Register allocation can minimize data movement
- **Examples**: Lua 5.x, LuaJIT (interpreter), Dalvik, V8 Ignition

#### Head-to-Head Comparison

| Dimension | Stack-Based | Register-Based |
|-----------|-------------|----------------|
| Bytecode size | Smaller (1-2 bytes/instr) | Larger (4 bytes/instr) |
| Instruction count | More instructions | Fewer instructions |
| Code generation | Simpler | Requires register allocation |
| Dispatch overhead | Higher (more dispatches) | Lower (fewer dispatches) |
| Optimization potential | Limited | Better (register reuse) |

The landmark paper "Virtual Machine Showdown: Stack Versus Registers" (Yunhe Shi et al.,
ACM TACO 2008) translated JVM bytecode to a register format and found a 26.5% reduction
in executed instructions with register-based encoding, despite 25% larger bytecode.

#### Real-World Design Decisions

**JVM (stack-based)**: Chose stack architecture for platform independence. The operand
stack is an abstract concept not tied to hardware register counts, making the same bytecode
valid on any CPU architecture.

**Dalvik (register-based)**: Google chose register-based for Android because mobile CPUs
are register-rich and register-based bytecode maps more naturally to physical registers
during JIT compilation, reducing the JIT compiler's work.

**Lua 5.x (register-based)**: The Lua team switched from stack-based to register-based in
Lua 5.0. Registers map directly to slots in the C stack, and the compiler performs register
allocation. This reduced the number of executed instructions significantly.

Sources:
- [Stack Based vs Register Based VM Architecture](https://markfaction.wordpress.com/2012/07/15/stack-based-vs-register-based-virtual-machine-architecture-and-the-dalvik-vm/)
- [Virtual Machine Showdown: Stack vs Registers](https://dl.acm.org/doi/10.1145/1328195.1328197)
- [Register-Based and Stack-Based VMs: JIT Comparison (2025)](https://onlinelibrary.wiley.com/doi/10.1002/spe.70014?af=R)

---

## 2. Dispatch Techniques

The dispatch loop is where a bytecode interpreter spends most of its time. The choice of
dispatch mechanism can yield 15-100% performance differences.

### 2.1 Switch Dispatch

The simplest approach: a `while(true)` loop containing a `switch` on the opcode byte.

```c
for (;;) {
    uint8_t op = *ip++;
    switch (op) {
        case OP_ADD: /* ... */ break;
        case OP_LOAD: /* ... */ break;
        // ...
    }
}
```

**Performance**: The branch predictor sees a single indirect branch at the switch, which
must predict across all possible opcodes. On modern CPUs, this single prediction site
becomes a bottleneck — the branch target buffer (BTB) entry is shared across all opcodes.

Benchmark: ~856M instructions/sec on Apple M1, ~1,095M IPS on AMD R9 5950x.

### 2.2 Computed Gotos (Direct Threading)

GCC/Clang extension using labels-as-values (`&&label`) to build a jump table. Each
instruction handler ends by jumping directly to the next handler, eliminating the loop
back-edge.

```c
static void *dispatch_table[] = { &&op_add, &&op_load, ... };
#define DISPATCH() goto *dispatch_table[*ip++]

op_add:
    // ... execute ADD ...
    DISPATCH();
op_load:
    // ... execute LOAD ...
    DISPATCH();
```

**Why it is faster**: Each opcode has its own indirect branch instruction, so the CPU's
branch predictor can track per-opcode branch targets independently. If `OP_LOAD` is
typically followed by `OP_ADD`, the BTB entry for the branch at the end of `OP_LOAD`'s
handler learns this pattern.

Benchmark: ~1,336M IPS on M1, ~1,187M IPS on AMD — roughly 15-55% faster than switch.

CPython uses this technique when compiled with GCC/Clang (`USE_COMPUTED_GOTOS`), and it is
enabled by default on supported platforms.

### 2.3 Direct Threading

In direct threading, the bytecode stream itself contains function pointers (or code
addresses) rather than opcode numbers. Each handler reads the next address from the
instruction stream and jumps to it.

```c
// Bytecode is an array of code addresses
void **code = { &&op_add, &&op_load, ... };
goto *(*vpc++);
```

This eliminates the table lookup entirely — the address is right there in the instruction
stream. However, this makes bytecode larger (pointer-sized entries instead of byte-sized
opcodes) and complicates serialization.

### 2.4 Indirect Threading

Adds one level of indirection to direct threading: the instruction stream contains indices
into a table of handler addresses. This allows runtime redirection (e.g., switching between
normal execution, profiling, and debugging modes) without modifying the bytecode.

### 2.5 Subroutine Threading

Each bytecode instruction compiles to a native `CALL` to its handler. The instruction
stream is a sequence of call instructions. This leverages the CPU's call/return predictor
stack but adds call/return overhead per instruction.

### 2.6 Tail-Call Threading

Each instruction handler is a C function that tail-calls the next handler via a function
pointer embedded in the instruction stream:

```c
void op_add(union instr *instrs, int32_t *sp, int32_t *input) {
    int32_t b = POP();
    int32_t a = POP();
    PUSH(a + b);
    instrs[1].fn(&instrs[1], sp, input);  // tail call
}
```

This requires the compiler to implement tail-call optimization. When it works, there is no
jump table lookup and no loop — each handler directly invokes the next.

Benchmark: ~1,427M IPS on M1, ~2,171M IPS on AMD — the fastest portable interpreter
technique before JIT compilation.

### Dispatch Comparison Summary

| Technique | M1 (Clang) | AMD R9 (GCC) | Portability |
|-----------|------------|--------------|-------------|
| Switch | 856M IPS | 1,095M IPS | Universal |
| Computed goto | 1,336M IPS | 1,187M IPS | GCC/Clang only |
| Tail calls | 1,427M IPS | 2,171M IPS | Requires TCO |
| AOT to C | 5,882M IPS | 25,700M IPS | Not an interpreter |

Note: These numbers are from a simple integer-arithmetic benchmark. In dynamically-typed
languages, dispatch overhead is dwarfed by type checking and method resolution costs.

Sources:
- [Dispatch Techniques for Interpreters](https://gist.github.com/pqnelson/01d60d8312c6194ce5effd77228a5557)
- [Faster Virtual Machines: Speeding Up Language Execution](https://mort.coffee/home/fast-interpreters/)
- [Dispatch Techniques — Matz Dissertation](https://www.cs.toronto.edu/~matz/dissertation/matzDissertation-latex2html/node6.html)

---

## 3. Crafting Interpreters Patterns (clox)

Bob Nystrom's *Crafting Interpreters* (2021) presents a complete bytecode VM in C (clox)
that demonstrates core patterns used by production interpreters.

### 3.1 Bytecode Design

**Chunk format**: A dynamic byte array (`uint8_t*`) stores instructions sequentially. A
parallel `ValueArray` holds constants referenced by index.

**Instruction encoding**: Simple and byte-oriented:
- Opcode: 1 byte
- Operands: 0-2 bytes following the opcode
- `OP_CONSTANT` takes a 1-byte index into the constant pool
- `OP_ADD`, `OP_NEGATE`, etc. have no operands — they operate on the stack

**The dispatch loop**: A `for(;;)` with a switch statement. The instruction pointer (`ip`)
is stored as a `uint8_t*` rather than an integer index because pointer dereference is
faster than array indexing. The `READ_BYTE()` macro is `(*vm.ip++)`.

### 3.2 Value Representation

clox supports two value representations, selectable at compile time:

**Tagged union** (default):
```c
typedef struct {
    ValueType type;    // VAL_BOOL, VAL_NIL, VAL_NUMBER, VAL_OBJ
    union {
        bool boolean;
        double number;
        Obj* obj;
    } as;
} Value;
```
Size: 16 bytes on 64-bit (8 bytes tag + padding, 8 bytes payload).

**NaN boxing** (optimization flag): Packs all values into a single `uint64_t` by exploiting
IEEE 754's NaN representation. See the internals document for full details.

### 3.3 Object System

Heap-allocated objects share a common header:
```c
struct Obj {
    ObjType type;
    bool isMarked;
    struct Obj* next;    // intrusive linked list for GC
};
```

Concrete types (ObjString, ObjFunction, ObjClosure) embed this header as their first field,
enabling C-style polymorphism via pointer casting.

### 3.4 Closures and Upvalues

See the internals document for the full upvalue mechanism. Key structs:

```c
typedef struct ObjUpvalue {
    Obj obj;
    Value* location;         // points to stack slot or closed field
    Value closed;            // heap storage after hoisting
    struct ObjUpvalue* next; // linked list for open upvalue tracking
} ObjUpvalue;
```

The compiler resolves upvalues at compile time by walking the enclosing compiler chain.
At runtime, `OP_CLOSURE` creates closures with operand pairs (`isLocal`, `index`) for each
captured variable.

### 3.5 Garbage Collection

Mark-sweep with a tricolor abstraction (white/gray/black). Roots include the VM stack,
global variable table, call frame closures, open upvalues, and compiler state. The heap
threshold self-adjusts: `nextGC = bytesAllocated * GC_HEAP_GROW_FACTOR` (factor = 2).

### 3.6 Hash Tables

clox uses open addressing with linear probing, a load factor of 75%, and tombstone entries
for deletion. String keys are interned (deduplicated) so equality checks reduce to pointer
comparison.

Source: [Crafting Interpreters](https://craftinginterpreters.com/)

---

## 4. Language-Specific Deep Dives

### 4.1 CPython

#### Compilation Pipeline

```
Source → Tokenizer → Parser → AST → Symbol Table → CFG → Bytecode
```

1. **Tokenizer**: Produces token stream from source text
2. **Parser**: PEG parser (since Python 3.9) produces a concrete syntax tree, converted to
   AST
3. **AST → Symbol Table**: Resolves scoping (local, global, free, cell variables)
4. **AST → CFG**: The compiler produces a control flow graph of basic blocks
5. **CFG → Bytecode**: Linearized to a sequence of 2-byte instructions (opcode + arg)

#### Bytecode Format

As of CPython 3.14, there are ~223 distinct opcodes. Instructions are 2 bytes wide: 1 byte
opcode + 1 byte argument (0-255). For larger arguments, `EXTENDED_ARG` prefix instructions
extend the argument to up to 4 bytes.

#### The Evaluation Loop (`_PyEval_EvalFrameDefault`)

The function in `Python/ceval.c` is where CPython spends most of its time. Structure:

1. **Dispatch mechanism**: Two modes:
   - **Computed goto** (preferred, with GCC/Clang): A static jump table
     `opcode_targets[]` maps opcodes to label addresses. Each handler ends with
     `DISPATCH()` which fetches the next opcode and jumps via the table.
   - **Switch fallback**: Standard switch/case for compilers without labels-as-values.

2. **Frame objects** (`_PyInterpreterFrame`): Contain `f_executable` (bytecode),
   `previous` (caller frame pointer), `instr_ptr` (instruction pointer), and
   `localsplus[]` — a single array holding both local variables and the evaluation stack.
   `LOAD_FAST` uses integer indexing into this array rather than dictionary lookup.

3. **Auto-generated handlers**: Opcode implementations are defined in
   `Python/bytecodes.c` and expanded into `Python/generated_cases.c.h`.

#### The GIL

The Global Interpreter Lock is a mutex that prevents multiple native threads from executing
Python bytecodes simultaneously. It simplifies memory management (reference counting is not
thread-safe without it) but limits parallelism. Note: CPython 3.13+ includes an
experimental free-threaded build removing the GIL, using biased reference counting and
stop-the-world pauses for GC.

#### Specializing Adaptive Interpreter (PEP 659, Python 3.11+)

A runtime bytecode specialization system that replaces generic instructions with
type-specialized variants:

1. **Quickening**: When a function is called, generic instructions are replaced with
   "adaptive" versions (e.g., `LOAD_ATTR_ADAPTIVE`).
2. **Warm-up counters**: Adaptive instructions track execution frequency. When a counter
   reaches zero, the runtime attempts specialization.
3. **Specialization**: Based on observed types, the instruction is replaced with a fast
   variant:
   - `LOAD_ATTR_INSTANCE_VALUE`: Direct slot access for instance attributes
   - `LOAD_ATTR_MODULE`: Module attribute with version check
   - `LOAD_GLOBAL_MODULE`: Global lookup with namespace key validation
   - `LOAD_GLOBAL_BUILTIN`: Builtin access with change detection
4. **De-specialization**: If the specialized instruction encounters unexpected types, a
   saturating counter decrements. When it hits minimum, the instruction reverts to its
   adaptive form. This is trivially cheap — just replace the opcode byte.
5. **Inline caches**: Specialized instructions store metadata (type versions, offsets) in
   16-bit entries immediately following the instruction in the bytecode stream.

Performance impact: 10-60% speedups, with the largest gains from attribute lookup, global
variable access, and function calls.

Memory overhead: ~6 bytes/instruction (vs 2 bytes in 3.10), approximately 25% increase in
total code object size.

Sources:
- [CPython VM Internals](https://blog.codingconfessions.com/p/cpython-vm-internals)
- [PEP 659 — Specializing Adaptive Interpreter](https://peps.python.org/pep-0659/)
- [Python Behind the Scenes #4](https://tenthousandmeters.com/blog/python-behind-the-scenes-4-how-python-bytecode-is-executed/)
- [CPython Interpreter Documentation](https://github.com/python/cpython/blob/main/InternalDocs/interpreter.md)

---

### 4.2 Lua 5.x and LuaJIT

#### Lua 5.x Register-Based VM

Lua switched from a stack-based to a register-based VM in version 5.0. Virtual registers
map directly to slots in the C call stack, with the compiler performing register allocation.

**Instruction encoding** (Lua 5.3):

All instructions are 32 bits wide with a 6-bit opcode:

| Format | Layout | Field sizes |
|--------|--------|-------------|
| iABC | `[B:9][C:9][A:8][Op:6]` | B,C=9 bits, A=8 bits |
| iABx | `[Bx:18][A:8][Op:6]` | Bx=18 bits |
| iAsBx | `[sBx:18][A:8][Op:6]` | sBx=18 bits (signed, excess-K) |

Lua 5.4 expanded to a 7-bit opcode and added two formats:

| Format | Layout | Field sizes |
|--------|--------|-------------|
| iABC | `[C:8][B:8][k:1][A:8][Op:7]` | 7-bit opcode, k flag |
| iABx | `[Bx:17][A:8][Op:7]` | |
| iAsBx | `[sBx:17][A:8][Op:7]` | |
| iAx | `[Ax:25][Op:7]` | |
| isJ | `[sJ:25][Op:7]` | |

**RK encoding**: In Lua 5.3, the MSB of a B or C operand distinguishes register references
(MSB=0) from constant pool references (MSB=1). In 5.4, a separate `k` flag bit serves
this purpose.

**Key design features:**
- The A operand always specifies the destination register
- Relational tests (EQ, LT, LE) always pair with a following JMP instruction
- LOADBOOL includes a skip field to implement conditionals without extra JMPs
- The entire instruction fits in one machine word — a single memory read decodes it

**Table implementation**: Lua tables are the sole data structure, using a hybrid design
with an array part (integer keys 1..n, O(1) access) and a hash part (all other keys,
open addressing). The array part auto-sizes so that at least half of slots between 1 and n
are occupied.

**String interning**: All strings are interned in a global hash table, making string
equality a pointer comparison.

#### LuaJIT

LuaJIT is a trace-compiling JIT for Lua, created by Mike Pall, achieving performance close
to C for numeric code.

**Architecture**: Unlike Lua 5.x, LuaJIT uses a stack-based bytecode for its interpreter
and converts to an SSA-based IR for JIT compilation.

**Trace recording:**
1. The interpreter identifies hot loops (executed N times) or hot function calls
2. Recording begins: each executed bytecode is converted to an IR instruction
3. Type information observed at recording time is embedded as guards
4. When the loop closes or a return is reached, the trace is complete
5. Side exits handle cases where guards fail — execution returns to the interpreter

**SSA IR:**
- Instructions are stored in a linear array, numbered sequentially
- Each instruction references operands by their instruction number (SSA)
- Key optimization passes:
  - **Constant folding**: Evaluates constant expressions at compile time
  - **Common subexpression elimination (CSE)**: Reuses redundant computations
  - **Dead code elimination (DCE)**: Removes unreachable instructions via skip-lists
  - **Narrowing**: Converts floating-point operations to integer when safe
  - **Type specialization**: Inlines type checks based on trace observations

**Snapshots**: The trace compiler records snapshots at guard points. If a guard fails,
the snapshot provides enough information to reconstruct interpreter state and resume
execution in the interpreter.

**Code generation backends**: LuaJIT includes hand-tuned assembly backends for x86,
x86-64, ARM, ARM64, PPC, and MIPS.

Sources:
- [The Implementation of Lua 5.0 (Ierusalimschy et al.)](https://www.lua.org/doc/jucs05.pdf)
- [Lua 5.3 Bytecode Reference](https://the-ravi-programming-language.readthedocs.io/en/latest/lua_bytecode_reference.html)
- [Lua 5.4 source: lopcodes.h](https://www.lua.org/source/5.4/lopcodes.h.html)
- [LuaJIT SSA IR 2.0](http://wiki.luajit.org/SSA-IR-2.0)
- [Interesting things about the Lua interpreter](https://thesephist.com/posts/lua/)

---

### 4.3 Ruby (YARV / CRuby)

#### YARV Bytecode VM

YARV (Yet Another Ruby VM) replaced the MRI tree-walking interpreter in Ruby 1.9. It is a
stack-based bytecode VM.

**Instruction types:**
- Stack manipulation: `dup`, `pop`, `swap`, `topn`
- Variable access: `getlocal`, `setlocal`, `getinstancevariable`, `setinstancevariable`
- Method dispatch: `opt_send_without_block`, `send`, `invokesuper`
- Optimized operations: `opt_plus`, `opt_minus`, `opt_lt`, etc. — specialized instructions
  for common operations on known types (fixnum, float, string)
- Control flow: `jump`, `branchif`, `branchunless`, `leave`

**Inline caches**: Method dispatch instructions include inline cache slots that store the
last-seen class and the resolved method, avoiding repeated method lookup when the receiver
type is stable.

#### JIT Evolution: MJIT → YJIT → ZJIT

**MJIT (Ruby 2.6-3.2)**: A method-based JIT that generated C code for hot methods, compiled
them with an external C compiler (GCC/Clang), and loaded the resulting shared library.
Slow compilation made it impractical for short-lived programs.

**YJIT (Ruby 3.1+)**: A Lazy Basic Block Versioning (LBBV) JIT written in Rust, built
inside CRuby.

How YJIT works:

1. **Lazy compilation**: Methods are compiled only after reaching a call threshold
   (default: 30 invocations). Compilation happens one basic block at a time, as execution
   reaches each block.

2. **Basic block versioning**: YJIT maintains a type context (knowledge about operand
   types) for each compiled block. If the same block is reached with different type
   contexts, a new version of the block is compiled with different type assumptions.

3. **Side exits**: When compiled code encounters an unexpected type, it takes a "side exit"
   — a jump to a stub that returns control to the YARV interpreter. Side exit counters
   track which exits are hot.

4. **Code layout**: Two code regions:
   - `cb` (code block): Inlined normal code for Ruby operations
   - `ocb` (out-of-line code block): Side exits, error paths, uncommon cases

5. **No IR**: YJIT translates directly from YARV bytecode to machine code (x86-64 and
   ARM64) without an intermediate representation. This keeps the JIT lightweight but limits
   optimization possibilities.

6. **Code invalidation**: When method definitions change (e.g., monkey-patching), inline
   caches invalidate and compiled code is patched or discarded.

**ZJIT (Ruby 4.0+)**: A next-generation JIT that adds an SSA-based IR (High-level IR →
Low-level IR → machine code) to enable more advanced optimizations while retaining YJIT's
lazy compilation approach.

Sources:
- [YJIT: A Basic Block Versioning JIT for CRuby (VMIL 2021)](https://dl.acm.org/doi/10.1145/3486606.3486781)
- [YJIT Documentation](https://docs.ruby-lang.org/en/master/jit/yjit_md.html)
- [Ruby's JIT Journey: From MJIT to YJIT to ZJIT](https://codemancers.com/blog/rubys-jit-journey)

---

### 4.4 V8 (JavaScript)

V8 is a multi-tier JIT-compiling JavaScript engine. Its architecture demonstrates the
state-of-the-art in dynamic language optimization.

#### The Compilation Pipeline

```
Source → Parser → AST → Ignition (bytecode) → Sparkplug → Maglev → TurboFan
                           ↑                                          ↓
                           └──────── deoptimization ──────────────────┘
```

**Tier 0 — Ignition (bytecode interpreter)**:
- Register-based with an accumulator: Most operations read from and write to a special
  accumulator register, minimizing explicit register operands
- Registers are "virtual" — they map to slots in the stack frame, not hardware registers
- Bytecode handlers are generated by TurboFan's macro-assembler, compiled to native code
- Dispatch: Indirect threading — each handler tail-calls the next handler
- Bytecode is 25-50% the size of equivalent baseline machine code

**Tier 1 — Sparkplug (baseline compiler)**:
- Very fast compilation, no optimization
- Compiles bytecode directly to machine code with minimal analysis
- Exists to reduce time spent in the interpreter for warm-but-not-hot code

**Tier 2 — Maglev (mid-tier optimizing compiler)**:
- SSA-based IR with lightweight optimizations
- Faster compilation than TurboFan with moderate optimization quality
- Added in 2023 to bridge the gap between Sparkplug and TurboFan

**Tier 3 — TurboFan (optimizing compiler)**:
- "Sea of Nodes" IR: Combines data flow and control flow in a single graph where nodes
  represent operations and edges represent dependencies. This representation allows
  aggressive reordering and optimization.
- Key optimization passes: type specialization, inlining, escape analysis, loop peeling,
  load elimination, dead code elimination, register allocation
- Produces highly optimized machine code based on speculative type assumptions

#### Hidden Classes (Maps) and Inline Caching

**Hidden classes**: V8 assigns every object a "Map" (hidden class) describing its shape —
what properties it has and at what offsets. When properties are added in the same order,
objects share Maps via a transition tree.

Key data structures:
- **Map**: First pointer in every object. Contains pointer to DescriptorArray and
  TransitionArray.
- **DescriptorArray**: Lists properties with their offsets and attributes. Shared across
  maps in the same transition chain — each map knows how many entries it may read.
- **TransitionArray**: Edges from one Map to child Maps, keyed by property name. If only
  one transition exists, the map stores it directly (no array needed).

Property storage:
- **In-object properties**: Stored directly in the object, fast access
- **Backing store**: An external array used when in-object slots are exhausted

**Inline caching (IC)**:
- Every property access site has an associated IC slot in the FeedbackVector
- On first execution, the IC is "uninitialized"
- On first hit, the IC becomes "monomorphic": it records the Map and property offset
- Subsequent accesses with the same Map skip the full lookup — just check Map, then load at
  known offset
- If different Maps appear, the IC becomes "polymorphic" (2-4 Maps tracked) or
  "megamorphic" (too many Maps, falls back to generic lookup)

#### Deoptimization

When TurboFan's speculative assumptions are violated at runtime:

1. The optimized code detects the violation (Map mismatch, type guard failure, overflow)
2. Execution transfers to a deoptimization trampoline
3. The trampoline uses recorded deoptimization data to reconstruct the Ignition frame
   (bytecode offset, register values)
4. Execution resumes in the interpreter at the correct bytecode position
5. The function may be re-optimized later with updated type information

If a function is deoptimized too many times, it is marked as "non-optimizable" to prevent
optimization thrashing.

Sources:
- [V8 Ignition and TurboFan Pipeline](https://github.com/thlorenz/v8-perf/blob/master/compiler.md)
- [Maps (Hidden Classes) in V8](https://v8.dev/docs/hidden-classes)
- [Launching Ignition and TurboFan](https://v8.dev/blog/launching-ignition-and-turbofan)
- [Value Representation in JavaScript Implementations](https://wingolog.org/archives/2011/05/18/value-representation-in-javascript-implementations)
- [Introduction to TurboFan](https://doar-e.github.io/blog/2019/01/28/introduction-to-turbofan/)

---

### 4.5 Erlang (BEAM)

The BEAM (Bogdan/Björn's Erlang Abstract Machine) is a process-oriented VM where
concurrency is the primary design goal, not single-threaded performance.

#### Architecture

BEAM is a register-based VM with two register types:
- **X registers**: Temporary/argument registers. Function arguments are passed in
  `{x,0}`, `{x,1}`, ..., and results return in `{x,0}`.
- **Y registers**: Local to each stack frame. Allocated with `allocate` and freed with
  `deallocate`.

On x86-64, key VM state is pinned to hardware registers for fast access.

#### Compilation Pipeline

```
Erlang source → Tokenizer → Parser → Abstract Forms
    → Core Erlang → Kernel Erlang (pattern match compilation)
    → BEAM assembly → Generic BEAM bytecode → Specific BEAM bytecode
```

The loader transforms generic bytecodes into specialized forms optimized for specific
operand types. BeamAsm (OTP 24+) JIT-compiles BEAM instructions to native code.

#### Reductions-Based Scheduling

Instead of time-slicing, BEAM uses reduction counting for preemptive scheduling:

1. Each process has a reduction counter initialized to ~4000 reductions per time slice
2. Function calls, BIF calls, GC, message send/receive, and other operations decrement
   the counter
3. When reductions hit zero, the process is preempted and placed at the end of the run
   queue
4. The scheduler picks the next process from the queue

This guarantees fair scheduling without wall-clock interrupts. GC itself counts as
reductions, so a process performing a large collection is naturally preempted.

**Priority levels**: `max` (internal only) > `high` > `normal` > `low`. High-priority
processes run before any normal/low processes; normal and low processes are interleaved.

**Multi-core scheduling**: One scheduler thread per CPU core, each with its own run queue.
Work-stealing balances load: idle schedulers steal processes from busy schedulers' queues.

#### Per-Process Garbage Collection

Each process has its own heap and stack, allocated in the same memory block and growing
toward each other. GC is per-process: a generational semi-space copying collector using
Cheney's algorithm.

Key consequences:
- **No global GC pauses**: Only the process being collected is paused
- **Millions of processes**: Each process starts with a tiny heap (a few hundred words)
- **Immutable data**: Since Erlang values are immutable, cross-process references require
  copying — there are no shared mutable objects to track

#### Pattern Matching Compilation

The Erlang compiler transforms pattern matches into decision trees during the Core Erlang →
Kernel Erlang phase. Compilation uses a variant of the classic pattern match compiler
algorithm:

1. Clauses are grouped by constructor type (atom, tuple, list, binary, integer, etc.)
2. A `select_val` instruction tests a value against multiple options (like a jump table)
3. Tuple matching uses `test_arity` to check size, then `get_element` to extract fields
4. Binary matching uses a "match context" — a cursor that advances through binary data
   without creating intermediate sub-binaries

The `receive` construct compiles to `loop_rec` (iterate message queue) + pattern match +
`remove_message` (on match) or `wait` (block until new message arrives).

Sources:
- [A Brief BEAM Primer](https://www.erlang.org/blog/a-brief-beam-primer/)
- [The BEAM Book](https://blog.stenmans.org/theBeamBook/)
- [Deep Diving Into the Erlang Scheduler](https://blog.appsignal.com/2024/04/23/deep-diving-into-the-erlang-scheduler.html)
- [BEAM Instruction Set](https://www.cs-lab.org/historical_beam_instruction_set.html)
- [Erlang Garbage Collection](https://www.erlang.org/doc/apps/erts/garbagecollection)

---

## 5. Emerging Patterns and Meta-Compilation

### RPython and Truffle/Graal

Meta-compiler frameworks allow writing an interpreter and automatically deriving a JIT:

**RPython** (PyPy): Write your interpreter in a restricted subset of Python. The RPython
toolchain analyzes it and generates a tracing JIT compiler. This is how PyPy achieves its
performance — the JIT is not hand-written but meta-generated.

**Truffle/Graal** (Oracle): Write an AST interpreter in Java using the Truffle framework.
Graal's partial evaluation automatically specializes and compiles hot paths. This powers
GraalPython, TruffleRuby, GraalJS, and others.

Both systems demonstrate that the interpreter architecture (tree-walking vs bytecode) may
matter less than the meta-compilation framework around it — TruffleRuby uses an AST
interpreter yet achieves near-native performance through Graal's specialization.

### Multi-Tier JIT

Modern VMs increasingly use multiple compilation tiers (V8 has four, as described above).
The pattern is:

1. Interpreter for cold code (fast startup, low memory)
2. Baseline compiler for warm code (fast compilation, moderate speed)
3. Optimizing compiler for hot code (slow compilation, peak speed)
4. Deoptimization to fall back when assumptions break

This avoids the "cliff" between interpreted and JIT-compiled code, providing a smooth
performance gradient.

Source: [A Lightweight Method for Generating Multi-Tier JIT Compilation](https://arxiv.org/html/2504.17460v3)
