# Sandboxing & Capability Systems Landscape

Research on sandboxing approaches, capability-based security, resource limits, and isolation strategies across programming languages and runtimes.

---

## 1. Language/Runtime-Specific Sandboxing

### 1.1 Deno

Deno is the most thoroughly designed permission system in a mainstream runtime. Deny-by-default: programs have zero access to I/O, network, environment, or subprocesses unless explicitly granted.

**Permission flags:**
| Flag | Short | Scope |
|------|-------|-------|
| `--allow-read[=paths]` | `-R` | File reading. Paths can be files or directories. |
| `--allow-write[=paths]` | `-W` | File writing. Same path scoping. |
| `--allow-net[=hosts]` | `-N` | Network. Scoped to hosts, optional ports, IPv6. |
| `--allow-env[=vars]` | `-E` | Environment variables. Wildcard suffix matching (`"AWS_*"`) since v2.1. |
| `--allow-run[=programs]` | | Subprocess spawning. Scoped to specific executables. |
| `--allow-ffi[=libs]` | | FFI/native library loading. Scoped to specific `.so`/`.dylib` files. |
| `--allow-sys[=apis]` | `-S` | System info APIs (`hostname`, `osRelease`, `systemMemoryInfo`, etc). |
| `--allow-import[=hosts]` | | Dynamic code imports. Default trusted hosts: `deno.land`, `jsr.io`, `esm.sh`. |
| `--allow-all` | `-A` | Grant everything (disables sandbox). |

**Deny flags** take precedence over allow flags:
```
deno run --allow-read=/etc --deny-read=/etc/hosts script.ts
```
This grants read access to `/etc` except `/etc/hosts`. Available: `--deny-read`, `--deny-write`, `--deny-net`, `--deny-env`, `--deny-run`, `--deny-ffi`, `--deny-sys`.

**Runtime permission prompts**: when stdout/stderr are connected to a TTY, Deno interactively prompts the user to grant permissions. Suppressed with `--no-prompt` or when output is redirected.

**Permission broker** (advanced): set `DENO_PERMISSION_BROKER_PATH` to a Unix domain socket or Windows named pipe. All permission checks are forwarded to an external broker process via JSON. CLI flags are ignored; no interactive prompts. This enables centralized policy management.

**Auditing**: `DENO_TRACE_PERMISSIONS=1` generates stack traces for every permission check. `DENO_AUDIT_PERMISSIONS=<path>` logs all permission accesses to a JSONL file with timestamps.

**Symlink handling**: permissions are checked at the symlink location, not the target. Reading symlinks to `/proc`, `/dev`, `/sys` requires `--allow-all` to prevent escalation.

**Key limitation**: code at the same privilege level can `eval`, `new Function`, dynamic import, or spawn Workers freely. The sandbox controls I/O, not computation.

Sources:
- [Deno Security and Permissions](https://docs.deno.com/runtime/fundamentals/security/)
- [Deno 2.5 Permissions in Config](https://deno.com/blog/v2.5)
- [Deno Permissions API](https://docs.deno.com/api/deno/permissions)

---

### 1.2 WASM/WASI

WebAssembly provides the strongest isolation model among widely-deployed runtimes, because the sandbox is enforced by the instruction set itself rather than runtime checks.

**Linear memory isolation:**
- Each Wasm module gets a private linear memory: a contiguous byte array addressed by 32-bit (or 64-bit) offsets.
- Every load/store is bounds-checked. Out-of-bounds access traps immediately.
- Pointers compile to offsets into linear memory, hiding host addresses from guest code.
- 2GB guard regions precede addressable memory (in Wasmtime) to catch sign-extension bugs in the compiler.

**No ambient authority:**
- Wasm modules have no implicit access to anything. All interaction with the outside world happens through explicitly imported functions.
- No raw system calls, no filesystem access, no network access unless the host provides import functions for them.
- The callstack is inaccessible: return addresses and spilled registers are stored in memory only the implementation can access. Stack-smashing attacks targeting return addresses are impossible.

**WASI capability model:**
- WASI (WebAssembly System Interface) provides filesystem, network, and clock APIs following capability-based security.
- Access is granted via **preopened directories**: the host opens specific directories and passes handles to the guest. The guest can only access files within those directories.
- No path traversal: the WASI implementation validates that resolved paths stay within the preopened directory tree.
- Resource access is declarative: modules list their imports, making it clear what capabilities they require.

**Execution limits:**
- **Fuel**: Wasmtime's mechanism for limiting computation. Each Wasm instruction consumes a configurable amount of fuel. When fuel runs out, execution traps. Enables CPU time budgeting.
- **Epoch-based interruption**: alternative to fuel with lower overhead. A global epoch counter is periodically incremented; modules check the counter at function entries and loop backedges. Better for coarse-grained time limits.
- **Memory limits**: linear memory has a declared maximum size. The host can refuse `memory.grow` requests.

**Spectre mitigations**: Wasmtime applies mitigations for bounds-check speculation, indirect call speculation, and branch table speculation.

**Memory zeroing**: Wasmtime zeros instance memory after completion to prevent information leakage between instances.

**Runtimes:**
- **Wasmtime**: Bytecode Alliance reference runtime. Cranelift JIT compiler. Rust API. Focus on correctness and security.
- **Wasmer**: alternative runtime. Supports multiple compiler backends (Cranelift, LLVM, Singlepass). More focus on startup speed and language bindings.
- **wasm-micro-runtime (WAMR)**: interpreter-based, designed for embedded/IoT. Smallest footprint.

**Primary escape vector**: JIT compiler bugs. If the compiler generates incorrect native code, the sandbox guarantees are void. Defense: use interpreter mode for untrusted code (slower but compiler bugs irrelevant), formal verification of compiler (ongoing research at CMU).

Sources:
- [Wasmtime Security](https://docs.wasmtime.dev/security.html)
- [WASI Capability-Based Security](http://www.chikuwa.it/blog/2023/capability/)
- [Provably-Safe Sandboxing with WebAssembly (CMU)](https://www.cs.cmu.edu/~csd-phd-blog/2023/provably-safe-sandboxing-wasm/)
- [WASI and WebAssembly Component Model](https://eunomia.dev/blog/2025/02/16/wasi-and-the-webassembly-component-model-current-status/)

---

### 1.3 Java SecurityManager (deprecated)

The SecurityManager was Java's original sandboxing mechanism, introduced for applets. Deprecated in JDK 17 (JEP 411), permanently disabled in JDK 24 (JEP 486). Its failure is deeply instructive.

**How it worked:**
- `System.setSecurityManager(sm)` installs a SecurityManager instance.
- Sensitive operations call `sm.checkPermission(Permission)`. If denied, a `SecurityException` is thrown.
- `AccessController.doPrivileged(action)` allows trusted library code to temporarily elevate privileges, telling the SecurityManager to consider only the library's permissions.
- **Policy files** define which codebases get which permissions. Example:
  ```
  grant codeBase "file:/path/to/app.jar" {
    permission java.io.FilePermission "/tmp/*", "read,write";
    permission java.net.SocketPermission "*.example.com:443", "connect";
  };
  ```

**Why it failed:**

1. **Maintenance burden**: dozens of permission types, hundreds of `checkPermission` calls throughout the JDK, 1,000+ methods requiring permission checks, 1,200+ methods needing `doPrivileged` wrappers. This was unsustainable for JDK maintainers.

2. **Brittle permission model**: no partial security. Denying one operation could inadvertently block unrelated functionality because permissions interact in surprising ways. No support for negative permissions (deny rules).

3. **Ambient authority**: libraries couldn't encapsulate their security requirements. Application code had to transitively grant permissions for all dependencies, violating separation of concerns.

4. **Performance penalty**: always disabled by default on the command line due to unacceptable overhead from the access-control algorithm.

5. **Ineffective against modern threats**: could not address 19 of the 25 most dangerous software weaknesses (XXE injection, input validation, deserialization attacks, etc.). These required direct countermeasures in class libraries, not permission checks.

6. **Near-zero adoption**: "hardly any discussion in the Java ecosystem" about deprecation warnings, confirming almost complete irrelevance to developers.

**Lessons for language designers:**
- Lower-level integrity mechanisms (module boundaries, hardened implementations) are more effective than runtime permission checks.
- A permission system that most users disable or grant `AllPermission` provides no security.
- The maintenance cost of pervasive permission checks must be weighed against the security benefit.
- Capability-based approaches (pass explicit handles, not ambient strings) avoid the confused deputy problem that plagued SecurityManager.

Sources:
- [JEP 411: Deprecate the Security Manager](https://openjdk.org/jeps/411)
- [JEP 486: Permanently Disable the Security Manager](https://openjdk.org/jeps/486)
- [SecurityManager is getting removed (Snyk)](https://snyk.io/blog/securitymanager-removed-java/)

---

### 1.4 Lua Sandboxing

Lua's design makes it one of the easiest languages to sandbox, because the environment is a first-class value.

**Lua 5.1 approach (setfenv):**
```lua
local sandbox_env = {
  print = print,
  type = type,
  tonumber = tonumber,
  tostring = tostring,
  pairs = pairs,
  ipairs = ipairs,
  next = next,
  pcall = pcall,
  xpcall = xpcall,
  select = select,
  unpack = unpack,
  table = { insert = table.insert, remove = table.remove, concat = table.concat, sort = table.sort },
  string = { sub = string.sub, find = string.find, format = string.format, rep = string.rep, gsub = string.gsub },
  math = { sin = math.sin, cos = math.cos, random = math.random, floor = math.floor, ceil = math.ceil },
}
local chunk = loadstring(untrusted_code)
setfenv(chunk, sandbox_env)
chunk()
```
`setfenv` replaces the function's environment table, restricting what globals it can see.

**Lua 5.2+ approach (load with env parameter):**
```lua
local chunk = load(untrusted_code, "sandbox", "t", sandbox_env)
chunk()
```
`setfenv` was removed in 5.2. Instead, `load()` accepts an environment parameter directly. The `"t"` flag restricts to text chunks only (no bytecode, which could bypass the sandbox).

**Unsafe functions to remove** (whitelist, not blacklist):
- `os.execute`, `os.exit`, `os.remove`, `os.rename`, `os.tmpname`, `os.getenv`
- `io.*` (all file I/O)
- `dofile`, `loadfile` (load code from filesystem)
- `require` (loads and executes modules)
- `debug.*` (entire debug library: can modify metatables, function environments, upvalues)
- `rawget`, `rawset` (bypass metatables, could circumvent protections)
- `setfenv` (5.1: can escape the sandbox by modifying environments of functions up the call chain)
- `collectgarbage` (denial of service)
- `newproxy` (5.1: creates userdata that can have arbitrary metatables)

**Resource limiting:**
```lua
local instruction_count = 0
local max_instructions = 1000000
debug.sethook(function()
  instruction_count = instruction_count + 1
  if instruction_count > max_instructions then
    error("CPU limit exceeded")
  end
end, "", 1) -- hook every instruction
```
`debug.sethook` with a count hook is the standard approach for CPU limiting. Note: the debug library must be available to the host but NOT to sandboxed code.

**Memory limiting**: Lua does not provide a built-in memory limit. Must be implemented at the C API level by providing a custom allocator that tracks and caps allocations.

**Metatables for protection**: use `__index`, `__newindex`, and `__metatable` metamethods to create read-only tables or tables with access control. Setting `__metatable` to a false value prevents `getmetatable` from revealing the real metatable.

**Luau (Roblox)**: Luau is a Lua derivative designed specifically for sandboxed execution. It removes `debug.*`, disables bytecode loading, adds type checking, and includes a memory-safe VM. This is the most production-proven Lua sandboxing approach.

Sources:
- [Lua-Users Wiki: Sandboxes](http://lua-users.org/wiki/SandBoxes)
- [Luau Sandbox](https://luau.org/sandbox)
- [How to Create a Secure Lua Sandbox](https://www.codegenes.net/blog/how-can-i-create-a-secure-lua-sandbox/)

---

### 1.5 Python Sandboxing

Python sandboxing is widely considered impossible at the language level. The language's introspection capabilities provide too many escape paths.

**Why language-level sandboxing fails:**
- Given any object, `type(obj)` returns its type, which is itself an object with `__subclasses__()`, enabling navigation of the entire type hierarchy.
- `obj.__class__.__mro__` exposes the method resolution order, including `object`.
- `object.__subclasses__()` lists every class in the process, including `os._wrap_close`, `subprocess.Popen`, etc.
- `__builtins__` can be accessed through frame introspection even if removed from the exec globals.
- C extension modules can bypass any Python-level restrictions.

**RestrictedPython** (Zope Foundation):
- Not a sandbox: a code transformer that restricts what AST nodes are allowed.
- Rewrites attribute access, item access, and function calls to go through guard functions.
- The caller defines what the guard functions allow.
- Supports CPython 3.9-3.13. Does NOT support PyPy.
- Used by Plone for TTW (through-the-web) Python scripts.
- Limitations: only works on source code (not bytecode), cannot restrict all escape paths.

**PyPy sandbox:**
- "Full virtualization" approach: `pypy-c-sandbox` executable makes zero system calls.
- Two-process model: untrusted code runs in a sandboxed PyPy process, which communicates with a trusted controller process via a pipe.
- The controller intercepts all I/O and decides what to allow.
- Status: experimental, development stalled since ~2020. The library to use it is "unpolished."

**Practical alternatives (what people actually use):**
- **seccomp**: Linux kernel feature restricting system calls. Used by Figma and others. The process can only call `exit`, `sigreturn`, `read`, `write` (in strict mode).
- **Docker/containers**: namespace isolation. Most common approach for running untrusted Python.
- **nsjail**: Google's lightweight process isolation using Linux namespaces + seccomp-bpf + cgroups.
- **gVisor**: reimplements Linux syscalls in a user-space kernel (Go). Used by Google Cloud Run.
- **Firecracker microVMs**: hardware-enforced isolation via KVM. Each workload gets its own kernel. Boots in ~125ms.
- **WASM**: compile Python to Wasm (Pyodide, RustPython) and run in a Wasm sandbox.

Sources:
- [Python Wiki: Sandboxed Python](https://wiki.python.org/moin/SandboxedPython)
- [Running Untrusted Python Code](https://healeycodes.com/running-untrusted-python-code)
- [PyPy Sandboxing](https://doc.pypy.org/en/stable/sandbox.html)
- [RestrictedPython](https://github.com/zopefoundation/RestrictedPython)

---

### 1.6 Capability-Based Security

**Core concepts:**

A **capability** is a communicable, unforgeable token of authority. It is a reference that both designates an object and authorizes access to it. You can only access a resource if you hold a capability for it.

**Object-capability model** (OCM): treats object references themselves as capabilities. In a memory-safe language, a reference to an object is unforgeable (you cannot manufacture a pointer). If you have a reference, you can call methods on it. If you don't have a reference, you cannot. This is the simplest form of capability security.

**Principle of Least Authority (POLA)**: every component should receive only the capabilities it needs to do its job, nothing more. Capabilities make POLA natural because you must explicitly pass each resource reference.

**Ambient authority**: the opposite of capabilities. In most systems, any code can call `open("/etc/passwd")` because the filesystem is ambient: always available, controlled only by ACLs checked against the caller's identity. Capabilities eliminate ambient authority by requiring all resources to be passed explicitly.

**The confused deputy problem**: a privileged program (the "deputy") is tricked by a less-privileged caller into misusing its authority. Example: a compiler with write access to a billing log is tricked into overwriting a system file. ACLs cannot prevent this because the deputy's identity authorizes the operation. Capabilities prevent this because the deputy only uses the file handle it was given, not an ambient filesystem.

**The E language** (Mark Miller):
- Designed as an object-capability language from the ground up.
- No global mutable state. No ambient authority.
- Objects communicate via message passing. References are capabilities.
- Introduced **eventual sends** (async message passing) and **promises** as core language features.
- Mark Miller's PhD thesis ("Robust Composition: Towards a Unified Approach to Access Control and Concurrency Control") is the foundational work on object-capability security.

**Practical implementations:**
- **Capsicum** (FreeBSD): capability mode for the OS. Once a process enters capability mode, it cannot open new file descriptors except by deriving them from existing ones.
- **seL4**: formally verified microkernel using capabilities for all resource access.
- **KeyKOS/EROS**: capability-based operating systems (research).
- **Agoric** (blockchain): Mark Miller's company applying object-capability patterns to smart contracts using Hardened JavaScript.
- **Hardened JavaScript** (SES): `lockdown()` freezes all primordials, creating a capability-safe JavaScript environment. Used by Agoric and MetaMask Snaps.

Sources:
- [Capability-based security (Wikipedia)](https://en.wikipedia.org/wiki/Capability-based_security)
- [Awesome Object Capabilities](https://github.com/dckc/awesome-ocap)
- [F# Capability-Based Security](https://fsharpforfunandprofit.com/posts/capability-based-security/)
- [Capability-Based Security and Macaroons](https://medium.com/swlh/capability-based-security-and-macaroons-aaa64fb9fc01)

---

### 1.7 Docker/Container-Based Sandboxing

Containers use multiple Linux kernel features layered for defense-in-depth.

**Namespaces** (isolation of what a container can see):
- **PID namespace**: container sees only its own processes. PID 1 inside maps to a different PID on the host.
- **Mount namespace**: isolated filesystem view. Container has its own root filesystem.
- **Network namespace**: own network interfaces, routing tables, iptables rules.
- **User namespace**: map container root (UID 0) to an unprivileged host UID. Rootless containers.
- **UTS namespace**: separate hostname and domain name.
- **IPC namespace**: isolated shared memory, semaphores, message queues.
- **Cgroup namespace**: isolated view of cgroup hierarchy.

**Cgroups** (limits on what a container can consume):
- CPU: shares, quotas, periods (e.g., 50% of one CPU core).
- Memory: hard limits, soft limits, swap limits. OOM killer targets the container.
- I/O: block device bandwidth limits, IOPS limits.
- PIDs: maximum number of processes.
- Important: cgroups prevent denial-of-service but are NOT a security boundary. They prevent resource exhaustion, not sandbox escape.

**Seccomp** (filtering how a container can behave):
- **Seccomp-BPF**: Berkeley Packet Filter programs that inspect system call numbers and arguments.
- Docker's default profile disables ~44 of 300+ syscalls. Blocks: `reboot`, `mount`, `kexec_load`, `init_module`, `ptrace`, `keyctl`, `add_key`, `bpf`, etc.
- Custom profiles can further restrict to a minimal set.
- Used by nsjail with the Kafel BPF language for readable filter definitions.

**AppArmor/SELinux** (mandatory access control):
- **AppArmor**: path-based. Profiles restrict file access, network access, capability usage per-program. Docker applies a default profile to all containers.
- **SELinux**: label-based. More granular but more complex. Used by RHEL/Fedora container runtimes.

**Linux capabilities** (fine-grained root powers):
- Root's powers are split into ~40 capabilities: `CAP_NET_BIND_SERVICE`, `CAP_SYS_ADMIN`, `CAP_DAC_OVERRIDE`, etc.
- Docker drops most capabilities by default. Containers get a minimal set: `CAP_CHOWN`, `CAP_NET_BIND_SERVICE`, `CAP_SETUID`, etc.
- `--cap-drop ALL --cap-add <specific>` for strict control.

**Key limitation**: all containers share the host kernel. A kernel vulnerability can allow escape. This is fundamentally different from VMs, which have separate kernels.

Sources:
- [Figma: Server-side sandboxing with containers and seccomp](https://www.figma.com/blog/server-side-sandboxing-containers-and-seccomp/)
- [nsjail](https://github.com/google/nsjail)
- [Docker Seccomp Profiles](https://docs.docker.com/engine/security/seccomp/)
- [Container Isolation: Namespaces and Control Groups](https://dev.to/hexshift/container-isolation-understanding-namespaces-and-control-groups-in-docker-318b)

---

### 1.8 Browser Sandboxing

Browsers are the most battle-tested sandboxing environments, executing untrusted code from millions of origins.

**V8 Isolates:**
- An Isolate is a completely independent V8 engine instance with its own heap, garbage collector, and execution state.
- One isolate cannot access another's memory, even within the same process.
- Start in ~5ms with an order of magnitude less memory than a container.
- Cloudflare Workers runs thousands of isolates per process, millions per server. Isolates are grouped into **cordons** by trust level.
- Each isolate gets its own execution context with separate global scope.

**Process isolation** (Chrome):
- Each renderer process handles one site (site isolation). A compromised renderer cannot access another site's data.
- GPU process, network process, browser process are all separate, communicating via IPC.
- Renderer processes are sandboxed via seccomp-BPF (Linux), seatbelt (macOS), restricted tokens (Windows).

**iframe sandboxing:**
- `sandbox` attribute restricts iframe capabilities: `allow-scripts`, `allow-forms`, `allow-same-origin`, `allow-popups`, `allow-top-navigation`.
- `allow-scripts` + `allow-same-origin` together is dangerous: the iframe can remove its own sandbox attribute.
- Cross-origin iframes are fully isolated: cannot access parent's DOM.
- Communication between iframe and parent: `postMessage` API only. Messages are structured-cloneable values, not live references.

**Content Security Policy (CSP):**
- HTTP header controlling which resources a page can load: `script-src`, `style-src`, `img-src`, `connect-src`, `font-src`, `frame-src`, etc.
- `script-src 'none'`: no scripts at all.
- `script-src 'self'`: only same-origin scripts.
- `script-src 'nonce-abc123'`: only scripts with matching nonce attribute.
- `script-src 'strict-dynamic'`: trust scripts loaded by already-trusted scripts.
- Blocks inline scripts and `eval()` by default (unless `'unsafe-inline'` or `'unsafe-eval'` is specified).

**Web Workers:**
- Separate thread with no DOM access. Communicates via `postMessage`.
- `SharedArrayBuffer` enables shared memory (disabled post-Spectre unless cross-origin isolation is active).

**WebAssembly in browsers:**
- Runs in the same V8 isolate but with linear memory isolation.
- Cannot access JavaScript objects or DOM directly; must go through imported JavaScript functions.
- Memory is a separate `ArrayBuffer`, bounds-checked on every access.

Sources:
- [Cloudflare Workers Security Model](https://developers.cloudflare.com/workers/reference/security-model/)
- [How Workers Works](https://developers.cloudflare.com/workers/reference/how-workers-works/)
- [V8 Isolates (InfoQ)](https://www.infoq.com/presentations/cloudflare-v8/)
- [Cloudflare Workers Security Hardening](https://blog.cloudflare.com/safe-in-the-sandbox-security-hardening-for-cloudflare-workers/)
- [Deno Isolate Cloud Anatomy](https://deno.com/blog/anatomy-isolate-cloud)

---

## 2. Sandboxing Design Patterns

### 2.1 Capability vs ACL

| Aspect | ACL (Access Control List) | Capabilities |
|--------|---------------------------|-------------|
| Authority location | On the resource (who can access this?) | With the subject (what can I access?) |
| Ambient authority | Yes (check identity + permission table) | No (must hold explicit reference) |
| Confused deputy | Vulnerable (deputy uses its own identity) | Immune (deputy uses passed capability) |
| Revocation | Easy (edit the ACL) | Hard (must revoke the capability token) |
| Delegation | Hard (requires ACL modification by admin) | Easy (pass the capability reference) |
| Audit | Easy (check the ACL on each resource) | Hard (capabilities are distributed) |
| Performance | O(n) search through ACL on each access | O(1) capability check (it's a reference) |
| Default behavior | Deny unless listed (if well-designed) | Deny unless capability held |
| Best for | Centralized systems, few resources | Distributed systems, fine-grained control |

**Key insight**: ACLs answer "who can access this resource?" Capabilities answer "what authority does this code hold?" The capability question is the right one for sandboxing, because sandboxed code should not have an identity that grants ambient powers.

**Hybrid approaches**: Macaroons (Google) are bearer tokens (capabilities) with embedded caveats (restrictions). They can be attenuated: you can create a more restricted macaroon from an existing one without contacting the server. This combines capability delegation with ACL-like restriction.

Sources:
- [ACL vs Capabilities (Iowa)](https://homepage.divms.uiowa.edu/~jones/security/notes/18.shtml)
- [Capability vs ACL (Storj)](https://storj.dev/learn/concepts/access/capability-based-access-control)
- [Cornell CS 513: Capability-based Access Control](https://www.cs.cornell.edu/courses/cs513/2005fa/L08.html)

### 2.2 Deny-by-Default

The principle: start with zero permissions, explicitly grant only what is needed.

**Implementations:**
- **Deno**: no I/O without flags. The most explicit implementation.
- **WASM/WASI**: no imports means no capabilities. The module must declare what it needs.
- **Seccomp strict mode**: only `exit`, `sigreturn`, `read`, `write`. Everything else kills the process.
- **Docker default seccomp profile**: allowlist of ~256 of 300+ syscalls.
- **Lua whitelist sandbox**: construct an environment table containing only safe functions. Everything else is `nil`.

**Contrast with blacklisting** (deny-specific):
- Java SecurityManager: started with everything allowed, then tried to deny specific operations. Failed because the deny list was never complete.
- Python `exec` with `restricted globals`: tried to remove `__builtins__`, but introspection provided escape paths.
- Blacklisting is fundamentally fragile: you must enumerate every dangerous operation, and new ones can be added by language evolution.

**Pattern**: deny-by-default is strictly preferable. The security perimeter is the whitelist, which is auditable. With blacklisting, the perimeter is "everything not on the list," which is unbounded.

### 2.3 Resource Limits

**CPU time:**
- **WASM fuel**: per-instruction accounting. Configurable cost per operation. Trap when exhausted. Fine-grained but has overhead.
- **WASM epochs**: coarse-grained. Global counter incremented by host (e.g., every 10ms). Modules check counter at function entries and loop backedges. Lower overhead than fuel.
- **Lua `debug.sethook`**: callback every N instructions. Can count and abort. The standard approach for Lua.
- **Linux `setrlimit(RLIMIT_CPU)`**: OS-level CPU time limit per process. Sends `SIGXCPU` then `SIGKILL`.
- **cgroups cpu.cfs_quota_us/cpu.cfs_period_us**: CPU bandwidth limiting for containers.

**Memory:**
- **WASM linear memory max**: declared in module, enforced by runtime. `memory.grow` can be rejected.
- **cgroups memory.max**: hard limit for containers. OOM killer fires.
- **Lua custom allocator**: replace `lua_Alloc` with a tracking allocator that fails when a cap is reached.
- **V8 isolate heap limits**: `--max-old-space-size`, `--max-heap-size`. Isolate is terminated on OOM.

**File handles / network connections:**
- **`setrlimit(RLIMIT_NOFILE)`**: maximum open file descriptors per process.
- **Deno `--allow-net=host:port`**: restrict which hosts/ports can be connected to.
- **Seccomp**: block `socket()`, `connect()`, `bind()` syscalls entirely.

**Process spawning:**
- **Deno `--allow-run=specific_binary`**: whitelist specific executables.
- **cgroups pids.max**: limit number of processes/threads in a cgroup.
- **Seccomp**: block `fork()`, `clone()`, `execve()`.

### 2.4 Backend Trait Pattern (lx's approach)

lx uses pluggable backends with Deny variants. Each I/O operation goes through a backend trait. The default implementation denies everything. Specific backends grant specific capabilities.

**How this compares:**

This is essentially the **object-capability model applied to a trait system**:
- The backend trait is the capability interface.
- A Deny backend is the null capability (no authority).
- A real backend (e.g., `FsBackend`) is a live capability.
- The runtime passes backend references to the interpreter, which passes them to user code.
- User code cannot manufacture a backend; it can only use what it's given.

**Advantages over other approaches:**
- Unlike Java SecurityManager: no ambient authority (backends are passed, not checked against identity).
- Unlike Deno: permissions are structural (trait implementations), not flag-based. This enables static analysis of capability requirements.
- Like WASI: explicit imports, deny-by-default. But with richer typing (traits vs flat function imports).
- Like Lua environment tables: the execution environment is explicitly constructed. But with type safety.

**Design recommendation**: the backend trait should be granular enough that a caller can provide only filesystem-read-from-specific-directory, not just "filesystem." WASI's preopened-directory pattern is a good model: the capability is not "can read files" but "can read files within this specific directory handle."

### 2.5 Taint Tracking

**Perl's taint mode** (`-T` flag):
- All data from external sources (user input, environment variables, file reads, command-line arguments) is automatically marked as **tainted**.
- Tainted data cannot be used in security-sensitive operations: `open()`, `system()`, `exec()`, `unlink()`, backticks, etc.
- Taint propagates: if a tainted variable is used in an expression, the result is tainted.
- **Untainting**: the ONLY way to untaint data is regex matching. Extract matched subpatterns (`$1`, `$2`), which are untainted. This forces developers to validate data shape before using it.
- If taint mode detects tainted data in a dangerous context, it aborts the program immediately. Crash > vulnerability.

**Limitations:**
- Cannot detect all dataflow vulnerabilities (implicit flows, timing channels).
- Regular expression validation may be insufficient (a regex that matches "anything" provides no real validation).
- Only tracks data flow, not control flow.

**Other implementations:**
- **Ruby** has a similar `$SAFE` level system (deprecated in Ruby 3.0) and taint flag on objects (removed in Ruby 2.7).
- **Ballerina** has taint checking as a compile-time feature.
- **Research**: taint tracking in JavaScript (TaintFox, DTA++) for detecting XSS and injection attacks.

**Relevance to lx**: taint tracking is valuable for agentic workflows where data flows through multiple agents with different trust levels. An agent receiving data from an untrusted source should not be able to pass that data to a filesystem write without explicit validation/untainting.

Sources:
- [perlsec - Perl security](https://perldoc.perl.org/perlsec)
- [CERT Perl: Taint Mode Limitations](https://wiki.sei.cmu.edu/confluence/display/perl/IDS01-PL.+Use+taint+mode+while+being+aware+of+its+limitations)
- [Taint Checking (Wikipedia)](https://en.wikipedia.org/wiki/Taint_checking)

### 2.6 Privilege Escalation

How sandboxed code can request elevated permissions:

**Interactive prompts** (Deno): when code attempts a restricted operation, the runtime pauses and asks the user. The user can grant once or permanently. This works for developer tools but not for automated/production systems.

**Capability delegation**: a trusted parent holds capabilities and selectively passes them to children. The child cannot request capabilities it hasn't been given. This is the pure capability model.

**Permission broker** (Deno advanced): an external process makes all permission decisions via a socket protocol. The broker can apply complex policies (time-of-day restrictions, rate limiting, audit logging) without modifying the runtime.

**Escalation via subprocess**: if sandboxed code can spawn a subprocess, and the subprocess inherits the parent's permissions, the sandbox is bypassed. Deno addresses this with `--allow-run=specific_binary`. Docker addresses it by running the container runtime itself with restricted capabilities.

**Key anti-pattern**: escalation via introspection. Python's `type(obj).__subclasses__()`, Java's reflection, JavaScript's prototype chain traversal. Any language feature that lets you discover capabilities you weren't given is a potential escalation vector.

### 2.7 Sandbox Escape Risks

**Common vulnerability classes:**

1. **JIT compiler bugs**: the compiler generates native code that violates the sandbox's memory isolation. Primary escape vector for V8, Wasmtime, and all JIT-based sandboxes. V8's sandbox reserves a large virtual address region to confine all heap pointers, limiting the impact of compiler bugs.

2. **Shared kernel**: all containers share the host kernel. Kernel vulnerabilities (e.g., dirty pipe, dirty cow) allow escape. Mitigated by gVisor (user-space kernel) or microVMs (separate kernel per workload).

3. **Language introspection**: Python's `__subclasses__()`, Java's reflection, JavaScript's `constructor.constructor("return this")()`. Any path from user code to the global scope or to sensitive built-ins.

4. **Async/Promise leakage**: vm2 (Node.js) was broken because async functions returned global Promises instead of sandboxed ones, allowing attackers to intercept function calls in the host context. CVE-2026-22709 demonstrates this class.

5. **Symlink/path traversal**: WASI implementations must validate that resolved paths stay within preopened directories. Symlinks can point outside the sandbox.

6. **Environment variable leakage**: forgetting to restrict `--allow-env` can leak API keys, database credentials, etc.

7. **Side channels**: Spectre-class attacks can read memory across isolation boundaries by measuring timing. V8 reduced `SharedArrayBuffer` precision and requires cross-origin isolation headers.

**Defense-in-depth layers:**
1. Language-level (capability model, no ambient authority)
2. Runtime-level (V8 isolates, WASM linear memory)
3. OS-level (seccomp, namespaces, cgroups)
4. Hardware-level (KVM/VT-x for microVMs)

No single layer is sufficient. Production sandboxing should use at least two independent layers.

Sources:
- [V8 Sandbox Escape (Theori)](https://theori.io/blog/a-deep-dive-into-v8-sandbox-escape-technique-used-in-in-the-wild-exploit)
- [vm2 Sandbox Escape CVE-2026-22709](https://socradar.io/blog/cve-2026-22709-vm2-sandbox-escape-vulnerability/)
- [Awesome Sandbox](https://github.com/restyler/awesome-sandbox)
- [Firecracker vs gVisor](https://northflank.com/blog/firecracker-vs-gvisor)
- [Sandbox Isolation Discussion](https://www.shayon.dev/post/2026/52/lets-discuss-sandbox-isolation/)

---

## 3. Key Takeaways for lx

1. **Backend trait pattern is sound**. It is essentially the object-capability model applied to Rust traits. The key improvement is making capabilities granular (not just "filesystem" but "read from this directory") and composable (combine multiple backends into a capability set).

2. **Deny-by-default is non-negotiable**. Every system that tried blacklisting (Java SecurityManager, Python restricted exec, early container runtimes) failed. Start with zero capabilities.

3. **WASM's fuel/epoch model** is the right approach for CPU limiting in an interpreter. Fuel gives fine-grained control; epochs give low-overhead coarse control. lx should support both.

4. **Taint tracking** is uniquely relevant for agentic workflows. Data flowing between agents with different trust levels should carry provenance. An untrusted agent's output should not be usable in privileged operations without explicit sanitization.

5. **Defense-in-depth**: lx's backend traits provide language-level sandboxing. For production, combine with OS-level isolation (seccomp/namespaces for self-hosted, WASM for embedded). Don't rely on any single layer.

6. **Deno's permission broker pattern** is worth studying for agentic workflows where a central orchestrator should control what capabilities each spawned agent receives.

7. **Lua and WASM prove** that the simplest sandboxing models are the most effective: construct an environment with only what's needed, hand it to untrusted code, and prevent the code from discovering anything not in that environment.
