# OS-Level and Container-Based Sandboxing for AI Agents

## How Major Companies Sandbox Agent Code Execution

### Anthropic (Claude Code)

- Uses Linux bubblewrap (bwrap) for containerization with pre-generated seccomp BPF filters for x86-64 and ARM
- macOS: sandbox-exec with dynamically generated Seatbelt profiles supporting glob patterns and real-time violation monitoring
- Network namespace removal forces all traffic through a proxy running outside the sandbox
- Open-sourced as `sandbox-runtime` (srt CLI + TypeScript library): https://github.com/anthropic-experimental/sandbox-runtime
- Reduced permission prompts by 84% internally

### OpenAI (Codex CLI)

- Code Interpreter: Kubernetes (Azure) with gVisor, tini as container init
- Codex CLI uses Landlock (kernel 5.13+) for filesystem access control -- read system-wide, restrict writes to allowlisted dirs
- seccomp filters block outbound network syscalls (socket, connect, bind) while permitting AF_UNIX for local IPC
- Sandbox runs as standalone helper process (`codex-linux-sandbox`)
- Two-phase model: setup phase (with network for deps) → agent phase (offline by default)
- Implementation: `codex-rs/linux-sandbox/src/landlock.rs` in https://github.com/openai/codex

### Google Cloud (Agent Sandbox)

- Kubernetes-native primitive under CNCF (SIG Apps), announced KubeCon NA 2025
- Built on gVisor (default) with Kata Containers support
- Kernel-level isolation per agent task with sub-second latency (90% improvement over cold starts)
- Pre-warmed pools of sandboxes on GKE
- Open source: https://github.com/kubernetes-sigs/agent-sandbox

### Cursor

- macOS Seatbelt, Linux Landlock + seccomp, WSL2 on Windows
- Sandboxed agents stop 40% less often than unsandboxed ones
- v2.5 added granular network domain allowlists
- Ignored files mounted via overlay filesystems and Landlocked to be inaccessible

### Docker (Docker Sandboxes, v4.60+)

- Each agent gets a dedicated microVM with a private Docker daemon
- Workspace bind-mounted at same absolute path
- HTTP/HTTPS filtering proxy controls network egress
- Supports Claude Code, Gemini, Codex, Copilot, Agent, Kiro natively
- Currently macOS/Windows only (Docker Desktop required), Linux on roadmap

### Manus AI

- Uses E2B (Firecracker microVMs) for per-task isolated Linux VMs
- Zero Trust architecture -- root inside each VM, no cross-contamination

---

## Lightweight VM/Sandbox Technologies

### gVisor (Google)

- User-space kernel ("Sentry") in Go intercepting ~70-80% of Linux syscalls
- Syscalls intercepted via ptrace or KVM and redirected to Sentry
- Used by: OpenAI Code Interpreter, Modal, Google Agent Sandbox
- No hypervisor overhead but incomplete syscall coverage
- https://gvisor.dev

### Firecracker (AWS)

- Minimalist VMM in Rust using Linux KVM to create microVMs
- Boots in ~125ms with ~5MB memory overhead
- Each VM runs its own guest kernel -- kernel exploit in one VM cannot reach host
- Companion "jailer" applies cgroups, seccomp, chroot as defense-in-depth
- Only 5 emulated devices exposed (minimal attack surface)
- Used by: AWS Lambda, E2B, Fly.io Sprites
- Gold standard for untrusted AI code execution
- https://github.com/firecracker-microvm/firecracker

### Kata Containers

- OCI containers with KVM hardware virtualization
- Lightweight VM with minimal Linux guest kernel and kata-agent inside
- Boot ~150-200ms, more memory than Firecracker but OCI/CRI compatible
- Used by: Google Agent Sandbox (optional), enterprise K8s deployments
- https://github.com/kata-containers/kata-containers

### Comparative Table

| Technology | Isolation Level | Boot Time | Memory Overhead | Syscall Coverage |
|---|---|---|---|---|
| gVisor | User-space kernel | Fast (ms) | Low (~15MB) | ~70-80% Linux |
| Firecracker | Hardware VM (KVM) | ~125ms | ~5MB | Full Linux kernel |
| Kata Containers | Hardware VM (KVM) | ~150-200ms | ~30MB | Full Linux kernel |
| Docker (vanilla) | Namespace/cgroup | Fast (~50ms) | Low (~1MB) | Full (shared kernel) |

---

## Linux Security Modules

### seccomp-bpf

- Filters syscalls at kernel level using BPF programs
- Docker default profile blocks ~44 of 300+ syscalls
- Anthropic sandbox-runtime ships pre-compiled BPF filters for x86-64 and ARM
- OpenAI Codex CLI uses seccompiler Rust crate for network syscall blocking
- Nearly zero performance overhead
- For agents: strict allowlist approach recommended

### AppArmor

- Path-based mandatory access control
- Profiles define file, capability, and network access per process
- Used by default in Ubuntu-based container runtimes
- Easier to write profiles than SELinux

### SELinux

- Label-based mandatory access control
- Fine-grained but complex
- Security Profiles Operator (SPO) for K8s provides CLI for recording/replaying profiles
- Default on Fedora/RHEL systems

### Landlock LSM

- Unprivileged sandboxing since Linux 5.13
- Process voluntarily restricts its own filesystem and network access
- No root, no daemon, no security module setup required
- ABI 4 (Linux 6.7) adds TCP bind/connect restrictions
- Used by OpenAI Codex CLI and Cursor IDE

Key Landlock tooling:
- `landrun`: Go CLI wrapping Landlock for arbitrary commands
- `ai-sandbox-landlock`: Rust binary with YAML profiles for AI agents
- `sandboxec`: Minimal single-binary Landlock wrapper

Rust crate: `landlock` provides safe bindings

---

## Linux Namespaces and Cgroups

### Namespaces

| Namespace | Isolates | Agent Use |
|---|---|---|
| PID | Process IDs | Agent can't see/signal host processes |
| NET | Network stack | Agent gets own network (or none) |
| MNT | Filesystem mounts | Agent sees only its workspace |
| USER | UID/GID mapping | Root inside, unprivileged outside |
| UTS | Hostname | Isolated hostname |
| IPC | Shared memory | No host IPC access |
| CGROUP | Cgroup hierarchy | Can't see host resource usage |

Bubblewrap (used by Anthropic) leverages unprivileged user namespaces -- no root, no daemon, no setuid.

### cgroups v2

- Unified hierarchy, cleaner API
- CPU: `cpu.max` for hard ceiling
- Memory: `memory.max` for hard limit, `memory.high` for throttling
- PIDs: `pids.max` to prevent fork bombs
- I/O: `io.max` for bandwidth/IOPS limits

Recommended config for agent sandbox:
```
cpu.max: 200000 100000  (2 cores)
memory.max: 4G
memory.swap.max: 0
pids.max: 256
io.max: 8:0 rbps=50000000 wbps=50000000
```

### Critical Note

Three runc CVEs in 2025 (CVE-2025-31133, CVE-2025-52565, CVE-2025-52881) demonstrated mount race conditions allowing host path writes from inside containers. Namespaces/cgroups alone are NOT a complete security boundary -- additional layers (gVisor, Firecracker, Landlock + seccomp) are required.

---

## Open-Source Sandboxing Projects

| Project | Tech | Stars | Description |
|---|---|---|---|
| [E2B](https://github.com/e2b-dev/E2B) | Firecracker | ~8.9K | Purpose-built agent sandbox platform |
| [Daytona](https://github.com/daytonaio/daytona) | Docker/Kata | ~21K | AI code execution infrastructure |
| [microsandbox](https://github.com/zerocore-ai/microsandbox) | libkrun microVMs | ~3.3K | Self-hosted, MCP integration |
| [OpenSandbox](https://github.com/alibaba/OpenSandbox) | Docker+K8s | ~3.8K | Unified API, 4 sandbox types |
| [K8s Agent Sandbox](https://github.com/kubernetes-sigs/agent-sandbox) | gVisor/Kata | — | K8s-native CRD |
| [sandbox-runtime](https://github.com/anthropic-experimental/sandbox-runtime) | bwrap/Seatbelt | — | Anthropic's lightweight sandbox |
| [Codex CLI](https://github.com/openai/codex) | Landlock+seccomp | — | OpenAI's local sandbox |
| [Wassette](https://github.com/microsoft/wassette) | Wasmtime | — | WASM-based MCP tool sandbox |
| [nsjail](https://github.com/google/nsjail) | Namespaces+seccomp | — | Google's process isolation tool |
| [ai-jail](https://github.com/akitaonrails/ai-jail) | bwrap+Landlock | — | Multi-OS agent sandbox |

Curated list: https://github.com/restyler/awesome-sandbox

---

## Sources

- https://www.anthropic.com/engineering/claude-code-sandboxing
- https://github.com/anthropic-experimental/sandbox-runtime
- https://code.claude.com/docs/en/sandboxing
- https://developers.openai.com/api/docs/guides/tools-code-interpreter/
- https://itnext.io/openais-code-execution-runtime-replicating-sandboxing-infrastructure-a2574e22dc3c
- https://zread.ai/openai/codex/14-linux-landlock-and-seccomp
- https://cloud.google.com/blog/products/containers-kubernetes/agentic-ai-on-kubernetes-and-gke
- https://docs.cloud.google.com/kubernetes-engine/docs/how-to/agent-sandbox
- https://github.com/e2b-dev/E2B
- https://e2b.dev/blog/how-manus-uses-e2b-to-provide-agents-with-virtual-computers
- https://github.com/alibaba/OpenSandbox
- https://github.com/zerocore-ai/microsandbox
- https://northflank.com/blog/how-to-sandbox-ai-agents
- https://edera.dev/stories/kata-vs-firecracker-vs-gvisor-isolation-compared
- https://blog.senko.net/sandboxing-ai-agents-in-linux
- https://pierce.dev/notes/a-deep-dive-on-agent-sandboxes
- https://opensource.microsoft.com/blog/2025/08/06/introducing-wassette-webassembly-based-tools-for-ai-agents/
