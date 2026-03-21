# Debugger & Profiler Design Landscape

A survey of Debug Adapter Protocol implementations, debugger architectures for interpreted languages, trace hooks, profiling techniques, and flamegraph generation -- informing lx's debugger and profiler design.

## Table of Contents

1. [Debug Adapter Protocol](#debug-adapter-protocol)
2. [Debugger Implementations](#debugger-implementations)
3. [Debugger Design for Interpreters](#debugger-design-for-interpreters)
4. [Profiling](#profiling)
5. [DAP Rust Crates](#dap-rust-crates)
6. [Design Recommendations for lx](#design-recommendations-for-lx)

---

## Debug Adapter Protocol

Source: https://microsoft.github.io/debug-adapter-protocol/specification, https://microsoft.github.io/debug-adapter-protocol/overview

### Architecture

DAP standardizes communication between development tools (editors/IDEs) and debuggers, analogous to how LSP standardizes language intelligence. A **debug adapter** sits between the IDE's generic debug UI and a specific debugger/runtime, translating between them.

Communication uses JSON messages over stdin/stdout (subprocess mode) or TCP sockets. The adapter can be implemented in any language.

### Message Types

Three message types, similar to LSP:

- **Request**: client-to-adapter command with `command` field, `arguments`, and `seq` (sequence number)
- **Response**: adapter-to-client reply with `request_seq` linking to originating request, `success` boolean, optional `body`
- **Event**: unsolicited adapter-to-client notification (e.g., `stopped`, `output`, `terminated`)

### Session Lifecycle

**Initialization**:
1. IDE sends `initialize` request with client capabilities
2. Adapter responds with `Capabilities` object (what features it supports)
3. Adapter sends `initialized` event when ready for configuration
4. IDE sends configuration: `setBreakpoints`, `setFunctionBreakpoints`, `setExceptionBreakpoints`
5. IDE sends `configurationDone` to signal configuration complete

**Launch vs Attach**:
- `launch`: adapter starts the debuggee process, manages its lifecycle
- `attach`: adapter connects to an already-running process; user manages its lifecycle

**Execution**:
- Program runs until a stopping condition (breakpoint, exception, pause, step completion)
- Adapter sends `stopped` event with reason
- IDE queries state: threads -> stack frames -> scopes -> variables (hierarchical waterfall)

**Termination**:
- Launched: IDE sends `terminate` (graceful) or `disconnect` (forced)
- Attached: IDE sends `disconnect` (detach without killing)
- Adapter sends `terminated` event to formally end session

### Capability Negotiation

The `initialize` response advertises supported features:
- `supportsConfigurationDoneRequest`
- `supportsFunctionBreakpoints`
- `supportsConditionalBreakpoints`
- `supportsHitConditionalBreakpoints`
- `supportsEvaluateForHovers`
- `supportsStepBack` / `supportsRestartFrame`
- `supportsCompletionsRequest`
- `supportsModulesRequest`
- `supportsSetVariable`
- `supportsTerminateRequest`
- `supportsDataBreakpoints`
- `supportsReadMemoryRequest`
- `supportsSingleThreadExecutionRequests`

### Breakpoint Types

| Type | Request | Description |
|------|---------|-------------|
| Source breakpoints | `setBreakpoints` | Line-based breaks in source files |
| Function breakpoints | `setFunctionBreakpoints` | Break on function entry by name |
| Data breakpoints | `setDataBreakpoints` | Break on variable/memory value change |
| Instruction breakpoints | `setInstructionBreakpoints` | Break at specific instruction addresses |
| Exception breakpoints | `setExceptionBreakpoints` | Break on thrown exceptions (configurable filters) |

Each breakpoint can have:
- **condition**: expression that must evaluate to true for the break to trigger
- **hitCondition**: break only after N hits (e.g., `>= 5`)
- **logMessage**: log a message instead of stopping (logpoints)

### Stepping Commands

| Command | Behavior |
|---------|----------|
| `continue` | Resume execution until next breakpoint or program end |
| `next` | Step over -- execute one statement, skip function calls |
| `stepIn` | Step into -- descend into function calls |
| `stepOut` | Step out -- run until current function returns |
| `pause` | Suspend execution |
| `stepBack` | Reverse step (if supported) |
| `reverseContinue` | Run backward to previous breakpoint (if supported) |

All stepping commands are tied to a specific `threadId`.

### Variable Inspection

**Hierarchical model**: stackTrace -> scopes -> variables

- `stackTrace` request: returns `StackFrame[]` for a thread, with pagination (`startFrame`, `levels`)
- `scopes` request: returns `Scope[]` for a frame (local, global, closure, etc.)
- `variables` request: returns `Variable[]` for a scope/variable reference, with filtering (indexed/named) and pagination
- Each scope/variable gets an integer `variablesReference` -- clients use this to expand nested structures

**Lifetime**: variable references are valid only during the current suspended state. Once execution resumes, all references become invalid. Adapters can assign sequential integers.

### Expression Evaluation

The `evaluate` request executes an expression in a stack frame's context:
- **context**: `"watch"`, `"repl"`, `"hover"`, `"clipboard"`
- Returns result string, type, and optional `variablesReference` for structured results
- Can be used for REPL-style interaction during debugging

### Key Events

| Event | When | Data |
|-------|------|------|
| `stopped` | Breakpoint hit, step complete, exception, pause | reason, threadId, description |
| `continued` | Execution resumed | threadId |
| `exited` | Process terminated | exitCode |
| `terminated` | Debug session ended | optional restart flag |
| `output` | stdout/stderr/debugger output | category, output text, optional source location |
| `thread` | Thread created/exited | threadId, reason |
| `breakpoint` | Breakpoint state changed | breakpoint object |
| `module` | Module loaded/changed/removed | module info |
| `loadedSource` | Source loaded/changed/removed | source info |
| `memory` | Memory content changed | memoryReference, offset, count |

---

## Debugger Implementations

### CPython Debugger (pdb/bdb)

Source: https://docs.python.org/3/library/sys.html, https://docs.python.org/3/library/bdb.html

#### sys.settrace -- The Foundation

Python's debugging infrastructure rests on `sys.settrace(tracefunc)`, which installs a global trace function called by the interpreter at every execution event.

**Trace function signature**:
```python
def tracefunc(frame, event, arg):
    return tracefunc  # or None to stop tracing this scope
```

**Events**:

| Event | When | arg |
|-------|------|-----|
| `'call'` | Function/code block entered | `None` |
| `'line'` | About to execute new source line | `None` |
| `'return'` | Function about to return | return value |
| `'exception'` | Exception occurred | `(type, value, traceback)` tuple |
| `'opcode'` | About to execute opcode (opt-in) | `None` |

**Frame object fields**:
- `f_code`: code object (function name, filename, constants)
- `f_locals`: local variable dict
- `f_globals`: global variable dict
- `f_lineno`: current line number
- `f_back`: previous stack frame (caller)
- `f_trace`: per-frame trace function (can override global)
- `f_trace_lines`: enable/disable per-line events (bool)
- `f_trace_opcodes`: enable/disable per-opcode events (bool)

**Key behaviors**:
- Tracing is disabled during trace function execution (prevents infinite recursion)
- Each thread needs its own `sys.settrace()` call
- If the trace function raises an error, tracing is disabled (`settrace(None)`)
- `sys.call_tracing(func, args)` enables recursive tracing from within a trace function
- Return value semantics: return self to keep tracing, return `None` to stop tracing that scope

#### bdb -- The Debugger Framework

`bdb.Bdb` provides the standard debugger base class. Two backends (Python 3.12+):
- `'settrace'` (default): uses `sys.settrace()`, best backward compatibility
- `'monitoring'`: uses `sys.monitoring`, better performance by disabling unused events

**Dispatch architecture**:

The `trace_dispatch()` method routes events to specialized handlers:

| Event | Handler | Action |
|-------|---------|--------|
| `"line"` | `dispatch_line()` | Check `stop_here()` or `break_here()`, call `user_line()` |
| `"call"` | `dispatch_call()` | Check `stop_here()`, call `user_call()` |
| `"return"` | `dispatch_return()` | Check `stop_here()`, call `user_return()` |
| `"exception"` | `dispatch_exception()` | Check `stop_here()`, call `user_exception()` |

**Stepping implementation**:

| Method | Behavior | Mechanism |
|--------|----------|-----------|
| `set_step()` | Stop after exactly one line | Trace every line in current and nested calls |
| `set_next(frame)` | Step over -- stop on next line in same frame | Record frame, skip called functions |
| `set_return(frame)` | Step out -- stop when frame returns | Record frame, wait for return event |
| `set_until(frame, lineno)` | Run to line or return | Useful for skipping to end of loop |
| `set_continue()` | Run to next breakpoint | Clear stepping flags; if no breakpoints, remove trace function entirely |

**Breakpoint management**:

`bdb.Breakpoint` supports:
- Temporary breakpoints (auto-delete after triggering)
- Conditional breakpoints (expression must evaluate true)
- Ignore counts (skip N hits before stopping)
- Enable/disable without deletion
- Indexed by `(file, line)` tuples and by breakpoint number

**Breakpoint evaluation** (`effective(file, line, frame)`):
1. Find breakpoints at (file, line)
2. Check enabled
3. Check function name match (via `checkfuncname`)
4. Evaluate condition (if any)
5. Check ignore count
6. Return `(breakpoint, should_delete_temporary)` or `(None, None)`

**Frame stopping logic**:
- `stop_here(frame)`: returns True if frame is at or below the stepping frame (prevents stepping into debugger internals)
- `break_here(frame)`: checks for effective breakpoints at current file/line
- `break_anywhere(frame)`: quick check -- any breakpoints in this file at all?

### Lua Debug Interface

Source: https://www.lua.org/pil/23.html, https://www.lua.org/manual/5.4/manual.html

Lua provides a debug library with two categories of functions:

**1. Introspective functions** (inspect running program):

| Function | Purpose |
|----------|---------|
| `debug.getinfo(f, what)` | Stack frame information. `what` selects fields: `n` (name), `S` (source), `l` (line), `t` (tail call), `u` (upvalues), `f` (function), `L` (valid lines), `r` (transfer info) |
| `debug.getlocal(level, n)` | Get name and value of local variable n at stack level |
| `debug.setlocal(level, n, v)` | Set value of local variable n at stack level |
| `debug.getupvalue(f, n)` | Get name and value of upvalue n of function f |
| `debug.setupvalue(f, n, v)` | Set upvalue n of function f |
| `debug.traceback([thread,] [message [, level]])` | Stack traceback as string |

**2. Hook functions** (trace execution):

`debug.sethook(hook, mask [, count])`:

| Mask | Event | When |
|------|-------|------|
| `"c"` | call | Function called |
| `"r"` | return | Function returning (includes tail returns) |
| `"l"` | line | About to execute new source line |
| count | count | After every `count` instructions |

The hook function receives `(event, line)` where event is the event name string and line is the current line number (for line events).

**C API equivalents**: `lua_sethook`, `lua_getinfo`, `lua_getlocal`, `lua_setlocal` -- same semantics but accessible from C extensions.

**Performance caveat**: debug hooks have significant overhead. The documentation warns that "some of [the debug library's] functionality is not exactly famous for performance" and that it "breaks some sacred truths of the language."

### Erlang/BEAM Debugger

Source: https://www.erlang.org/doc/apps/debugger/debugger_chapter

Erlang's debugger works through a fundamentally different model due to BEAM's process architecture:

**Meta-process architecture**: when you attach to a debugged process, a separate meta process is created to manage the debugging session. This enables non-intrusive monitoring.

**Interpreted execution**: debugging requires modules to be compiled with `debug_info` flag. The debugger interprets modules rather than executing compiled BEAM code. Interpreted code is stored in a database, and debugged processes use only this stored code.

**Breakpoint types**:
- Line breakpoints at executable lines
- Conditional breakpoints: stop when `CModule:CFunction(Bindings)` returns true
- Function breakpoints at first line of each clause

**Key constraint**: "When a process reaches a breakpoint, only that process is stopped. Other processes are not affected." This is natural for BEAM's isolated-process model.

**Stack trace emulation**: the `:int` module tracks recently called interpreted functions. Three options: track all calls, track non-tail calls only, or disabled.

**Distributed debugging**: two modes:
- Local: interpret code only on current node
- Global: interpret across all known nodes, display remote debugged processes

**Performance impact**: interpreted code runs significantly slower than compiled code. Programs with timers may behave unexpectedly during debugging.

### GDB/LLDB (Compiled Languages)

For reference -- how compiled-language debuggers work:

**DWARF debug info**: compilers emit debug information in DWARF format mapping machine code addresses to source locations, variable names, types, and scopes. This is embedded in the binary or separate debug files.

**Hardware breakpoints**: x86 processors have debug registers (DR0-DR3) that trigger on memory access. Limited to 4 simultaneous hardware breakpoints.

**Software breakpoints**: replace the instruction at the breakpoint address with a trap instruction (INT 3 / 0xCC on x86). When hit, the debugger restores the original instruction, single-steps, re-inserts the breakpoint.

**Watchpoints**: monitor memory addresses for reads/writes. Hardware watchpoints use debug registers; software watchpoints single-step and check memory after each instruction (extremely slow).

**ptrace**: on Linux, debuggers use the `ptrace` system call to control child processes -- attach, detach, read/write memory, read/write registers, single-step, continue.

### Chrome DevTools Protocol (V8)

V8's debugging protocol (used by Chrome DevTools and Node.js inspector):

**Breakpoint mechanism**: V8 patches bytecode at breakpoint locations with `DebugBreak` bytecodes. When hit, V8 enters the debugger, which can inspect the isolate's state.

**Stepping**: V8 supports step-over, step-into, step-out by setting temporary breakpoints at appropriate locations (next statement, function entry, return address).

**Evaluation**: the `Runtime.evaluate` and `Debugger.evaluateOnCallFrame` commands execute JavaScript in a specific scope. V8 compiles the expression in the target frame's scope chain.

**Profiling**: V8 exposes CPU profiling (sampling-based), heap snapshots, and allocation tracking through the protocol.

---

## Debugger Design for Interpreters

### Trace-Based Debugging

The most common approach for interpreted languages. The interpreter calls a hook function at execution events.

**Python model** (`sys.settrace`):
- Hook on: call, return, line, exception, opcode
- Per-frame trace function (return from hook to set next frame's trace)
- Global + per-frame hooks enable selective tracing

**Lua model** (`debug.sethook`):
- Hook on: call, return, line, count (every N instructions)
- Single hook function receives event type and line number
- Access locals/upvalues via `debug.getlocal`/`debug.getupvalue`

**Key pattern**: the interpreter checks a hook at every line/call/return. The hook function decides whether to stop execution (breakpoint check, step check) or continue.

### Implementing Trace Hooks for lx

For a tree-walking interpreter, the natural hook points are:

1. **Before evaluating each expression/statement** (line event): check source position, compare against breakpoints and stepping state
2. **On function call** (call event): push stack frame, check function breakpoints
3. **On function return** (return event): pop stack frame, check step-out condition
4. **On exception/error** (exception event): check exception breakpoints

**Minimal implementation**:
```
// Conceptual -- in the interpreter's eval loop:
fn eval(&mut self, expr: &Expr) -> Result<Value> {
    if let Some(hook) = &self.debug_hook {
        hook.on_line(expr.span, &self.call_stack, &self.env)?;
        // hook returns: Continue, Break, StepInto, StepOver, StepOut
    }
    match expr {
        Expr::Call { .. } => {
            hook.on_call(...);
            let result = self.eval_call(...);
            hook.on_return(...);
            result
        }
        ...
    }
}
```

### Breakpoint Implementation for Interpreters

Unlike compiled debuggers (which patch instructions), interpreted language debuggers check breakpoints at each execution step:

**Naive approach**: at every line event, iterate through all breakpoints checking `(file, line)`. O(B) per line where B = breakpoint count.

**Optimized approach** (Python/bdb pattern):
1. `break_anywhere(frame)`: quick hash lookup -- any breakpoints in this file? If no, skip all breakpoint checks for this file.
2. `break_here(frame)`: lookup `(file, line)` in breakpoint hash map. O(1).
3. `effective()`: evaluate conditions, check ignore counts, handle temporary breakpoints.

**Conditional breakpoints**: evaluate a user-provided expression in the current scope. If it returns truthy, break. This requires the debugger to have access to the interpreter's evaluation capability.

### Stepping Implementation

**Step-into** (`set_step`): set a flag that causes the next line event to stop. Simplest stepping mode.

**Step-over** (`set_next(frame)`): record the current stack frame. Stop at the next line event where the frame is the same as or a parent of the recorded frame. This skips into function calls.

**Step-out** (`set_return(frame)`): record the current stack frame. Stop when a return event fires for that frame.

**Implementation in a tree-walker**:

```
enum StepMode {
    Continue,       // run until breakpoint
    StepInto,       // stop at next line
    StepOver(FrameId),  // stop at next line in same or parent frame
    StepOut(FrameId),   // stop on return from this frame
}
```

At each line event:
- `StepInto`: always stop
- `StepOver(f)`: stop if current frame is f or a caller of f (stack depth <= f's depth)
- `StepOut(f)`: stop only on return event from frame f
- `Continue`: stop only at breakpoints

### Expression Evaluation in Debug Context

When stopped at a breakpoint, users expect to evaluate expressions in the current scope:

1. The debugger captures the current environment/scope chain at the breakpoint
2. User expression is parsed by the same parser as the language
3. Expression is evaluated using the interpreter with the captured scope
4. Result is formatted and returned

**Challenges**:
- Side effects: evaluating expressions may modify state. Some debuggers offer "pure evaluation" that rolls back side effects.
- Scope access: the debugger must provide access to local variables, closure variables, and globals.
- Error handling: expression evaluation errors should not crash the debug session.

### Stack Frame Representation

DAP requires exposing stack frames with specific fields:

```
StackFrame {
    id: integer,           // unique frame ID
    name: string,          // function name or description
    source: Source,        // file path + reference
    line: integer,         // current line in source
    column: integer,       // current column
    endLine: integer,      // optional end line
    endColumn: integer,    // optional end column
}
```

For a tree-walking interpreter, maintain a call stack:
- Push on function call (record function name, source location, local environment)
- Pop on function return
- Each frame gets a sequential ID (valid only during current stop)

Scopes within each frame:
- **Local**: current function's local variables
- **Closure**: captured variables from enclosing scopes
- **Global**: module-level or global variables

### Hot Reload During Debug

Editing code while stopped at a breakpoint:

**Simple approach**: re-read and re-parse the file, replace the function definitions in the environment. The current call stack continues with old code, but future calls use new definitions.

**Challenges**:
- Active stack frames reference old code -- can't safely replace them mid-execution
- Variable bindings may have changed shape
- Breakpoint line numbers may shift

**Practical strategy for lx**: support "edit and continue" by replacing function definitions in the environment when a file is saved. Active frames keep their current code. This matches Erlang's hot code loading model.

---

## Profiling

### Sampling Profilers

**Mechanism**: periodically (e.g., every 1ms or 10ms) interrupt the program and record the current call stack. After many samples, frequently-appearing functions indicate hot spots.

**Advantages**:
- Low overhead (typically 1-5% CPU)
- No code modification needed
- Statistical accuracy improves with more samples
- Can profile production systems

**Disadvantages**:
- Cannot capture short-lived function calls (below sampling interval)
- Statistical noise in low-sample counts
- May miss infrequent but important code paths

**Implementation for lx**: use a timer thread that periodically reads the interpreter's call stack. Store stacks as vectors of (function_name, source_location). Aggregate into frequency tables.

**Tools**: Linux `perf`, macOS Instruments, py-spy (Python), rbspy (Ruby).

### Tracing Profilers

**Mechanism**: instrument every function call and return, recording exact timestamps. Produces complete call trees with precise timing.

**Advantages**:
- Exact timing for every function call
- Complete call tree reconstruction
- No statistical noise

**Disadvantages**:
- High overhead (10-100x slowdown typical)
- Generates massive data volumes
- Perturbs timing-sensitive code

**Implementation for lx**: use the same trace hook mechanism as the debugger. On call event, record timestamp and function name. On return event, compute elapsed time. Build a call tree.

```
struct TraceEvent {
    kind: Call | Return,
    function: String,
    timestamp: Instant,
    source_location: Span,
}
```

**Tools**: Python cProfile, Lua's `debug.sethook` with count, Chrome DevTools Timeline.

### Memory Profiling

**Heap snapshots**: capture all live objects at a point in time. Shows what's consuming memory and who's holding references.

**Allocation tracking**: record every allocation with its call stack. Shows where memory is being allocated (not just what's live).

**Leak detection**: compare heap snapshots over time. Growing object counts suggest leaks.

**Implementation for lx**:
- Track allocations in the interpreter's value/object system
- Each allocation records its source location and allocating function
- Periodic snapshots count objects by type and source
- Diff snapshots to find growth patterns

### Flamegraphs

Source: https://www.brendangregg.com/flamegraphs.html

**Visualization**: each rectangle represents a function in a stack. Width = frequency (how often it appeared in samples). Y-axis = stack depth. X-axis is alphabetically sorted (not temporal).

**Key insight**: wider frames = more time spent. The widest frames at the top of the graph are the direct CPU consumers; wide frames lower in the graph are callers of hot code.

**Types**:

| Type | Shows | Color |
|------|-------|-------|
| CPU flamegraph | CPU-consuming code paths | Warm colors (red/orange/yellow) |
| Memory flamegraph | Allocation sites with byte counts | Green |
| Off-CPU flamegraph | Blocking time and context switches | Blue |
| Differential flamegraph | Comparison between two profiles | Red (regression) / Blue (improvement) |

**Generation process**:
1. Collect stack traces (via sampling profiler or tracing)
2. Convert to folded stack format: one line per sample, semicolon-separated stack
3. Render with flamegraph tools into interactive SVG

**Folded stack format**:
```
main;parse_file;tokenize 42
main;parse_file;parse_expr 108
main;eval;eval_call;eval_body 256
```

Each line: stack frames separated by `;`, space, then sample count.

**Tools**: Brendan Gregg's FlameGraph scripts (https://github.com/brendangregg/FlameGraph), inferno (Rust port: https://github.com/jonhoo/inferno), speedscope (web-based viewer).

**Implementation for lx**: collect sampling profiler data in folded stack format, then use inferno (Rust crate) to generate SVG flamegraphs directly from the lx runtime.

---

## DAP Rust Crates

### `dap` crate

Source: https://docs.rs/dap/latest/dap/

Version 0.4.1-alpha1. Provides Rust types for DAP messages.

**Modules**:
- `server`: server instantiation and request polling
- `requests`: DAP request definitions
- `responses`: DAP response structures
- `events`: DAP event types
- `types`: protocol data types
- `base_message`: message framing with sequence numbers

**Usage pattern**:
```rust
let server = Server::new(stdin, stdout);
loop {
    let request = server.poll_request()?;
    match request.command {
        Command::Initialize { .. } => { server.respond(response)?; }
        Command::SetBreakpoints { .. } => { ... }
        Command::Continue { .. } => { ... }
        ...
    }
}
```

**Maturity**: early stage (0.4.1-alpha1), ~80% documented. Suitable for building custom DAP adapters.

### `debug-adapter-protocol` crate

Source: https://docs.rs/debug-adapter-protocol/latest/debug_adapter_protocol/

Version 0.1.0. Provides typed message definitions.

**Key types**:
- `ProtocolMessage`: base class for all messages
- `ProtocolMessageContent`: enum of request/response/event content
- `SequenceNumber`: type alias for message sequencing
- Modules: `events`, `requests`, `responses`, `types`

Uses `typed-builder` for constructing complex protocol messages.

### Comparison

| Aspect | `dap` | `debug-adapter-protocol` |
|--------|-------|--------------------------|
| Server abstraction | Yes (`Server` type) | No (types only) |
| Request polling | Built-in | Manual |
| Maturity | 0.4.1-alpha1 | 0.1.0 |
| API style | Server-oriented | Type definitions |

**Recommendation for lx**: use the `dap` crate for its server abstraction, or implement the protocol manually with `serde_json` (the DAP wire format is simple enough). The protocol is simpler than LSP -- a manual implementation may be preferable for full control over the debug session lifecycle.

---

## Design Recommendations for lx

### Debugger Architecture

**Phase 1: Trace hooks in the interpreter**

Add hook points to the interpreter's evaluation loop:
- `on_line(span, env)` -- before evaluating each expression with a new source line
- `on_call(function_name, args, span)` -- on function entry
- `on_return(function_name, return_value, span)` -- on function exit
- `on_error(error, span)` -- on error/exception

The hook interface should be a trait that can be implemented by:
- A no-op (production mode, zero overhead if compiled out)
- A debugger (breakpoints, stepping, variable inspection)
- A profiler (timing, call counting)
- A tracer (execution logging)

**Phase 2: DAP adapter**

Build a DAP adapter that:
1. Communicates with the IDE via stdin/stdout JSON messages
2. Controls the interpreter via the trace hook interface
3. Manages breakpoints, stepping state, and variable inspection
4. Exposes the interpreter's call stack as DAP StackFrames
5. Exposes environments as DAP Scopes/Variables

**Phase 3: Profiler**

Implement both sampling and tracing profilers using the same hook interface:
- Sampling: timer thread reads call stack periodically
- Tracing: record every call/return with timestamps
- Output: folded stack format for flamegraph generation via inferno

### Interpreter Integration Points

The interpreter needs these capabilities for debugging:

1. **Source position tracking**: every AST node must carry its source span. The interpreter must know which source line it's currently executing.

2. **Call stack exposure**: maintain an explicit call stack with function name, source location, and local environment for each frame.

3. **Environment inspection**: the debugger must be able to enumerate local variables, closure captures, and globals for any stack frame.

4. **Evaluation in context**: the debugger must be able to parse and evaluate arbitrary expressions in a stopped frame's scope.

5. **Controllable execution**: the interpreter must be able to pause, resume, step, and abort execution under debugger control.

### Stepping State Machine

```
                  +-----------+
    breakpoint -> |  STOPPED  | <- step complete
                  +-----------+
                   |  |  |  |
          continue |  |  |  | stepOut
                   v  |  |  v
              +-------+  +-------+
              |RUNNING|  |STEP_OUT|
              +-------+  +-------+
                  |stepIn    |stepOver
                  v          v
             +---------+ +----------+
             |STEP_INTO| |STEP_OVER |
             +---------+ +----------+
```

At each line event, check the stepping state:
- `RUNNING`: check breakpoints only
- `STEP_INTO`: always stop
- `STEP_OVER`: stop if stack depth <= recorded depth
- `STEP_OUT`: stop if stack depth < recorded depth (frame returned)

### Variable Representation for DAP

Map lx values to DAP variables:

| lx Type | DAP Representation |
|---------|-------------------|
| String | value = string content, type = "string" |
| Number | value = formatted number, type = "number" |
| Bool | value = "true"/"false", type = "bool" |
| Null | value = "null", type = "null" |
| List | value = "[3 items]", variablesReference = ref (expand to show elements) |
| Record/Map | value = "{3 fields}", variablesReference = ref (expand to show fields) |
| Function | value = "fn(a, b)", type = "function" |
| Error | value = error message, type = "error" |

Structured values (lists, records) get a `variablesReference` so the IDE can lazily expand them. References are integers assigned sequentially, valid only during the current stopped state.

### Key Dependencies (Rust)

| Crate | Purpose |
|-------|---------|
| `dap` or manual impl | DAP protocol messages and server |
| `serde` / `serde_json` | JSON serialization for DAP messages |
| `inferno` | Flamegraph SVG generation from folded stacks |
| `crossbeam` | Channels for debugger <-> interpreter communication |

---

## Sources

- DAP Specification: https://microsoft.github.io/debug-adapter-protocol/specification
- DAP Overview: https://microsoft.github.io/debug-adapter-protocol/overview
- DAP Implementations: https://microsoft.github.io/debug-adapter-protocol/implementors/adapters/
- Python sys.settrace: https://docs.python.org/3/library/sys.html
- Python bdb module: https://docs.python.org/3/library/bdb.html
- Lua debug library: https://www.lua.org/pil/23.html, https://www.lua.org/manual/5.4/manual.html
- Erlang debugger: https://www.erlang.org/doc/apps/debugger/debugger_chapter
- ElixirLS: https://github.com/elixir-lsp/elixir-ls
- `dap` crate: https://docs.rs/dap/latest/dap/
- `debug-adapter-protocol` crate: https://docs.rs/debug-adapter-protocol/latest/debug_adapter_protocol/
- Flamegraphs: https://www.brendangregg.com/flamegraphs.html
- inferno (Rust flamegraph): https://github.com/jonhoo/inferno
