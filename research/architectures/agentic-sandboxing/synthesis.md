# Agentic Sandboxing — Consolidated Synthesis

## What the Major Players Actually Use (March 2026)

| Player | Technology | Key Detail |
|--------|-----------|------------|
| Anthropic (Claude Code) | Bubblewrap (Linux) + Seatbelt (macOS) | Open-sourced as sandbox-runtime. 84% fewer permission prompts |
| OpenAI (Codex CLI) | Landlock + seccomp | Standalone codex-linux-sandbox. Two-phase: setup (online) → agent (offline) |
| Cursor | Landlock + seccomp (Linux), Seatbelt (macOS), WSL2 (Win) | Domain-level network allowlists since v2.5 |
| Google | gVisor + Kata Containers | K8s-native CRD (agent-sandbox), CNCF SIG Apps |
| Docker | MicroVMs with private Docker daemon | Docker Sandboxes (v4.60+), supports 6 agent types natively |
| E2B | Firecracker microVMs | ~150ms boot, <5MiB/VM, ~50% Fortune 500 |
| Microsoft | Wassette (WASM) + Hyperlight (WASM+VM) | Wasmtime-based, deny-by-default, MCP-native |
| Meta | LlamaFirewall | PromptGuard 2 + Agent Alignment Checks + CodeShield |

---

## Performance Comparison

| Technology | Cold Start | Memory/Instance | CPU Overhead | Multi-tenant Safe |
|-----------|-----------|----------------|-------------|-------------------|
| WASM (Wasmtime) | 35μs - 3ms | 2-15 MB | 5-30% | Yes |
| Landlock+seccomp | Instant | 0 | ~0% | Partial |
| Bubblewrap | Instant | ~1 MB | ~0% | Partial |
| Docker (runc) | 50-200ms | 20-50 MB | ~0% | No (shared kernel) |
| gVisor | ~150ms | ~15 MB | 5-15% | Yes |
| Firecracker | ~125ms | ~5 MB | 1-3% | Yes |
| Kata Containers | ~500ms | ~30 MB | 3-5% | Yes |

---

## The 2026 Architecture Consensus

```
┌─ Model Layer (probabilistic, ~99% catch rate) ──────────────┐
│  Constitutional AI, RLHF, LlamaFirewall, NeMo Guardrails    │
├─ Pre-Execution Layer (deterministic, per-tool-call) ─────────┤
│  AEGIS, Progent, AgentSpec, Invariant Gateway                │
├─ OS Sandbox Layer (kernel-enforced, per-process) ────────────┤
│  Landlock + seccomp (lightweight)                             │
│  Bubblewrap/Seatbelt (medium)                                │
│  gVisor (heavy, K8s-native)                                  │
│  Firecracker microVM (strongest)                             │
├─ Network Layer (proxy-enforced) ─────────────────────────────┤
│  Default-deny egress, domain allowlists, GET-only mode       │
│  DNS filtering, Cilium L7 policies                           │
├─ Filesystem Layer (CoW + audit) ─────────────────────────────┤
│  OverlayFS/AgentFS for disposable workspaces                 │
│  Git worktrees for code isolation                            │
│  Landlock/bwrap for path restrictions                        │
│  tmpfs over ~/.ssh, ~/.aws, .env                             │
├─ Runtime Monitoring (eBPF, out-of-band) ─────────────────────┤
│  Tetragon (enforce + detect), Falco (detect + alert)         │
│  AgentSight (intent→action correlation, <3% overhead)        │
│  Append-only audit logs with hash chaining, WORM storage     │
├─ Human Oversight (risk-tiered) ──────────────────────────────┤
│  Auto-approve low-risk, sample medium, always high-risk      │
│  Kill switches in external control plane                     │
│  CRIU checkpoints before destructive actions                 │
└──────────────────────────────────────────────────────────────┘
```

---

## Critical Gaps

### MCP Server Sandboxing

The biggest unaddressed risk. No sandboxing in the MCP spec. Claude Code is the only major client with OS-level sandboxing. 30+ CVEs in 60 days (Jan-Feb 2026). Three viable approaches:

1. **Container-per-server** (Docker MCP Toolkit) — most production-ready today
2. **WASM-per-tool** (Wassette/Cosmonic) — strongest capability model, emerging
3. **OS primitives** (sandbox-runtime, Landlock) — lightest weight

### Multi-Agent Isolation

No framework (CrewAI, AutoGen, LangGraph) provides OS-level inter-agent isolation. Confused deputy is the central risk. IsolateGPT's hub architecture is the most rigorous proposal but not yet productionized.

### Message Sanitization

Unsolved tension: strict sanitization drops task success to 35-65%. Permissive communication enables cross-agent injection cascades.

---

## Real-World Breaches That Shaped the Landscape

| Incident | Date | Impact |
|----------|------|--------|
| GTG-1002 Claude Code espionage | Sept 2025 | AI performed 80-90% of cyber espionage across 30+ orgs |
| EchoLeak (CVE-2025-32711) | 2025 | Zero-click data exfiltration from M365 Copilot |
| ServiceNow (CVE-2025-12420) | 2025 | Second-order injection, privilege escalation via ticket fields |
| OpenClaw crisis | Jan-Mar 2026 | 900 malicious skills (20% of registry), macOS stealer distribution |
| MCP DNS rebinding | 2025 | Official SDKs vulnerable, local servers exposed to web |
| 30+ MCP CVEs | Jan-Feb 2026 | 82% of 2,614 implementations vulnerable to path traversal |

---

## Key Open-Source Projects

| Project | Technology | Purpose |
|---------|-----------|---------|
| [sandbox-runtime](https://github.com/anthropic-experimental/sandbox-runtime) | bwrap/Seatbelt | Anthropic's agent sandboxing |
| [Codex CLI](https://github.com/openai/codex) | Landlock+seccomp | OpenAI's local sandbox |
| [E2B](https://github.com/e2b-dev/E2B) | Firecracker | Agent sandbox platform |
| [OpenSandbox](https://github.com/alibaba/OpenSandbox) | Docker+K8s | Unified sandbox API |
| [microsandbox](https://github.com/zerocore-ai/microsandbox) | libkrun microVMs | Self-hosted, MCP integration |
| [Wassette](https://github.com/microsoft/wassette) | Wasmtime | WASM MCP tool sandbox |
| [agent-sandbox](https://github.com/kubernetes-sigs/agent-sandbox) | gVisor/Kata | K8s-native CRD |
| [Container Use](https://github.com/dagger/container-use) | Docker+worktrees | Per-agent container+branch |
| [AgentFS](https://github.com/tursodatabase/agentfs) | FUSE+SQLite | CoW overlay with audit trail |
| [LlamaFirewall](https://github.com/meta-llama/PurpleLlama/tree/main/LlamaFirewall) | ML classifiers | Meta's agent firewall |
| [AEGIS](https://arxiv.org/abs/2603.12621) | Framework-agnostic | Pre-execution firewall |
| [AgentSight](https://github.com/eunomia-bpf/agentsight) | eBPF | Agent observability |
| [landrun](https://github.com/Zouuup/landrun) | Landlock | Minimal sandbox CLI |
| [awesome-sandbox](https://github.com/restyler/awesome-sandbox) | — | Curated list of solutions |

---

## Key Academic Papers

| Paper | Year | Key Contribution |
|-------|------|-----------------|
| Systems Security Foundations (2512.01295) | 2025 | Model as untrusted component, probabilistic TCB |
| AgentSpec (2503.18666, ICSE 2026) | 2025 | DSL for runtime constraints, >90% prevention |
| Progent (2504.11703) | 2025 | Privilege control DSL, 0% attack success |
| AEGIS (2603.12621) | 2026 | Pre-execution firewall, 14 frameworks, Ed25519 audit |
| IsolateGPT (2403.04960, NDSS 2025) | 2025 | Hub architecture for multi-agent isolation |
| AgentSight (2508.02736) | 2025 | eBPF observability, <3% overhead |
| ceLLMate (2512.12594) | 2025 | Browser agent sandboxing at HTTP layer |
| MiniScope (2512.11147) | 2025 | Minimum privilege scope solver |
| Visual Confused Deputy (2603.14707) | 2026 | Screen-based attack on GUI agents |
| "Your AI, My Shell" (2509.22040) | 2025 | 84% attack success on coding editors |

---

## Practical Recommendations

### Immediate (No Infrastructure Changes)

- Use Landlock or Bubblewrap for MCP server processes
- Mount tmpfs over sensitive directories (~/.ssh, ~/.aws)
- Set `--network=none` on agent containers where possible
- Enable Claude Code sandboxing (already built-in)

### Short-Term (Container Infrastructure)

- Container-per-MCP-server with seccomp profiles and network allowlists
- OverlayFS workspaces with per-agent upper layers
- Tetragon for runtime enforcement + Falco for detection
- Structured audit logging to WORM storage

### Medium-Term (Advanced Isolation)

- WASM Component Model for individual tool sandboxing (Wassette)
- Firecracker microVMs for untrusted code execution
- AgentSight for eBPF-based intent→action correlation
- CRIU checkpoints before destructive operations

### Long-Term (Standards)

- Push for MCP spec permission declaration format
- Server code signing and verification standard
- Kubernetes Agent Sandbox CRD as deployment standard
- Formal verification of sandbox configurations

---

## The Bottom Line

Probabilistic guardrails are a UX feature (friendlier agent). Deterministic sandboxing is a security feature (safer agent). You need both. If you can only have one, choose sandboxing. The model is an untrusted component. Every real-world breach in this space has reinforced this lesson.
