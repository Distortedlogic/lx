# WebAssembly (WASM) Based Sandboxing for AI Agents

## Runtimes

### Wasmtime (Bytecode Alliance)

- Reference implementation, de facto standard
- Cranelift JIT compiler with security focus (fuzzing, formal verification, Spectre mitigations)
- Most complete WASI and Component Model support
- ~3ms startup, ~15MB memory footprint
- Powers Fastly Compute, Fermyon Spin, Microsoft Wassette
- https://wasmtime.dev

Key sandboxing features:
- Fuel metering (computation budget, traps when exhausted)
- Epoch-based interruption (cooperative preemption)
- Memory limits (hard ceiling per module)

```rust
let engine = Engine::new(Config::new()
    .consume_fuel(true)
    .epoch_interruption(true))?;
let mut store = Store::new(&engine, ());
store.set_fuel(1_000_000)?;
store.limiter(|_| StoreLimits::new()
    .memory_size(64 * 1024 * 1024));  // 64MB
```

### Wasmer

- Multiple backends: Singlepass, Cranelift, LLVM
- AOT compilation, own package registry (WAPM)
- ~2ms cold start, ~12MB memory
- Had sandbox escape: CVE-2023-51661 (WASI path translation flaw allowing /etc/passwd reads)

### WasmEdge (CNCF Sandbox)

- Smallest footprint (2-8MB), fastest cold start (1.5ms), highest throughput (15K RPS)
- WASI-NN for TensorFlow/PyTorch inference inside WASM
- Lags on Component Model support

### Comparison

| Feature | Wasmtime | Wasmer | WasmEdge |
|---------|----------|--------|----------|
| Maturity | High (reference) | High | Medium |
| Security focus | Primary | Secondary | Secondary |
| Fuel metering | Yes | Via middleware | Limited |
| Component Model | Best | Partial | Partial |
| AI/ML support | Via WASI-NN | Via plugins | Best (native) |
| Cold start | ~3ms | ~2ms | ~1.5ms |
| Memory | ~15MB | ~12MB | ~8MB |

---

## WASI: Capability-Based System Access

WASI implements capability-based security: modules receive zero system access by default.

### Core Design Principles

- **Unforgeable handles**: Resources as Component Model handles, cannot be fabricated
- **Pre-opened directories**: FS access only within explicitly granted directories
- **No ambient authority**: No env vars, network, random, clock unless provided
- **Worlds**: Named collections of imports/exports defining what a module can do

```rust
let wasi_ctx = WasiCtxBuilder::new()
    .preopened_dir("/workspace", "workspace", DirPerms::READ, FilePerms::READ)?
    .preopened_dir("/output", "output", DirPerms::all(), FilePerms::all())?
    .stdout(stdout)
    .stderr(stderr)
    .env("LANG", "en_US.UTF-8")?
    // NO network, NO other filesystem
    .build();
```

### WASI Versions

- **WASI 0.2** (Jan 2024): Stable, built on Component Model
- **WASI 0.3** (expected 2025): Native async I/O support (critical for agent tools needing network/IO)

### Relevant WASI Interfaces

| Interface | Capability | Sandboxing Use |
|-----------|-----------|----------------|
| wasi:filesystem | File access | Per-directory, per-operation grants |
| wasi:sockets | Network | Per-address, per-port |
| wasi:http | HTTP requests | Per-URL/method control |
| wasi:clocks | Time | Can deny to prevent timing attacks |
| wasi:random | Randomness | Can provide deterministic source |
| wasi:nn | ML inference | Control model access |

---

## Extism Plugin Framework

Cross-language framework for WASM plugin systems. Built on Wasmtime, SDKs in 13+ languages.

### Architecture for Agent Tools

1. Each tool compiled to .wasm binary (zero host access by default)
2. Host selectively links "host functions" as capability grants
3. Configurable memory ceilings per plugin
4. K/V state persistence through host-controlled store

```rust
let manifest = Manifest::new([Wasm::file("tools/search_tool.wasm")])
    .with_memory_options(MemoryOptions { max_pages: Some(1000) })  // ~64MB
    .with_timeout(Duration::from_secs(30))
    .with_allowed_hosts(&["api.github.com"]);
let mut plugin = Plugin::new(&manifest, [], true)?;
let result = plugin.call::<&str, &str>("search", "query")?;
```

Published guide on sandboxing LLM-generated code with Extism + LangChain.

Limitation: doesn't fully support Component Model yet (uses own host function mechanism).

- https://extism.org/blog/sandboxing-llm-generated-code/
- https://github.com/extism/extism

---

## Spin (Fermyon) and Fastly Compute

### Spin

- CNCF Sandbox project for serverless WASM apps
- Deny-by-default everything; capabilities declared in component manifest
- Per-component isolation with individual sandbox and capability grants
- Sub-millisecond cold starts ("fresh sandbox per request")
- Acquired by Akamai (Dec 2025)

```toml
[[trigger.http]]
component = "agent-tool"
route = "/tool/search"

[component.agent-tool]
source = "tools/search.wasm"
allowed_outbound_hosts = ["https://api.github.com"]
files = [{ source = "data/", destination = "/" }]
```

### Fastly Compute

- Per-request sandbox in ~35 microseconds
- Cranelift inserts bounds checks inline (no context-switch overhead)
- No shared memory between tenants
- Production-proven at billions of instances daily

---

## The Component Model

Transforms WASM from flat bytecode into composable, type-safe module system. Foundation of WASI 0.2+.

### Key Concepts for Agent Sandboxing

**WIT (WebAssembly Interface Types)**: Language-neutral IDL for typed component interfaces.

```wit
world agent-tool {
    import wasi:filesystem/types@0.2.0;
    import wasi:random/random@0.2.0;
    export run: func(input: string) -> result<string, string>;
}
```

**Resource types**: Unforgeable, revocable handles. Host passes file-handle granting specific access; component cannot fabricate other handles.

**Composition without shared memory**: Data crosses boundaries through typed, copied values. One component cannot corrupt another's memory.

**Virtualization**: Component imports satisfied by another component (not just host), enabling capability attenuation.

Status (early 2026): Wasmtime has most complete implementation. Wasm 3.0 became W3C standard Sept 2025.

---

## Projects Using WASM for Agent Sandboxing

### Microsoft Wassette (Aug 2025)

- Rust-based runtime bridging WASM Components and MCP
- Agents fetch Wasm Components from OCI registries, execute as MCP tools
- Deny-by-default sandbox (no FS, no network, no env unless allowed)
- Built on Wasmtime, zero runtime dependencies
- Integrated with Claude Code, GitHub Copilot, Cursor, Gemini CLI
- https://github.com/microsoft/wassette

### NVIDIA Agentic AI Sandboxing

- Uses Pyodide (CPython compiled to WASM) for LLM-generated Python
- Block network egress, block writes outside workspace
- Inject secrets through host, enforce lifecycle limits
- https://developer.nvidia.com/blog/sandboxing-agentic-ai-workflows-with-webassembly/

### amla-sandbox

- WASM-based bash shell sandbox for AI agents
- WASI for minimal syscall interface, Wasmtime runtime
- Authorization, deterministic replay, context budget control
- No containers, no VMs, no cloud -- runs in-process
- https://github.com/amlalabs/amla-sandbox

### Microsoft Hyperlight Wasm (Mar 2025)

- Combines WASM sandbox with hypervisor-backed micro-VM
- 1-2ms micro-VM with no OS, for defense-in-depth
- Donated to CNCF
- https://opensource.microsoft.com/blog/2025/03/26/hyperlight-wasm-fast-secure-and-os-free/

### Cosmonic Sandbox MCP

- Built on CNCF wasmCloud
- Generates MCP servers as secure WASM Components
- Shared-nothing sandbox enforcing least privilege

---

## Performance: WASM vs Containers vs MicroVMs

| Metric | WASM (Wasmtime) | Containers (Docker) | MicroVMs (Firecracker) |
|---|---|---|---|
| Cold start | 35μs - 3ms | 50-200ms | ~125ms |
| Memory/instance | 2-15 MB | 20-50 MB | ~5 MB |
| CPU overhead | 5-30% (bounds checking) | ~0% | 1-3% |
| Isolation granularity | Per-function/request | Per-container | Per-VM |
| Density | 50x+ more than containers | Baseline | Similar to containers |

WASM wins for high-frequency, short-lived, stateless tool invocations. Containers/VMs win for sustained CPU computation or heavy I/O.

---

## Limitations

1. **Side-channel attacks**: Spectre-class timing attacks not fully mitigatable
2. **JIT compiler bugs**: Primary real-world escape vector (CVE-2024-30266, CVE-2023-51661)
3. **Denial of service**: Must enforce timeouts/memory limits externally
4. **Covert channels**: Co-located modules can communicate via timing
5. **Host function escape**: Sandbox only as secure as exposed host functions
6. **Limited language support**: Rust/C/C++/Go compile well; Python/Ruby via interpreter (slow)
7. **No GPU access**: WASI-NN limited; agents needing GPU need containers/VMs
8. **Memory limits**: WASM32 caps at 4GB (WASM64 not widely supported)
9. **No threading**: Component Model threads proposal pending
10. **Obfuscation**: Binary format hard to audit

### Defense-in-Depth

For high-security: combine WASM sandbox with hypervisor layer (Hyperlight) + network egress controls. No single isolation layer sufficient against motivated attacker controlling the module.

---

## Sources

- https://reintech.io/blog/wasmtime-vs-wasmer-vs-wasmedge-wasm-runtime-comparison-2026
- https://arxiv.org/html/2404.12621v1
- https://marcokuoni.ch/blog/15_capabilities_based_security/
- https://github.com/WebAssembly/WASI/blob/main/docs/DesignPrinciples.md
- https://eunomia.dev/blog/2025/02/16/wasi-and-the-webassembly-component-model-current-status/
- https://docs.wasmtime.dev/security.html
- https://extism.org/blog/sandboxing-llm-generated-code/
- https://www.fermyon.com/spin
- https://www-astral.fastly.com/blog/how-we-vetted-cranelift-for-secure-sandboxing-in-compute-edge
- https://component-model.bytecodealliance.org/
- https://opensource.microsoft.com/blog/2025/08/06/introducing-wassette-webassembly-based-tools-for-ai-agents/
- https://developer.nvidia.com/blog/sandboxing-agentic-ai-workflows-with-webassembly/
- https://www.softwareseni.com/firecracker-gvisor-containers-and-webassembly-comparing-isolation-technologies-for-ai-agents/
- https://webassembly.org/docs/security/
- https://bytecodealliance.org/articles/security-and-correctness-in-wasmtime
