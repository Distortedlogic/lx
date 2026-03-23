# Agentic Sandboxing Research

Research compiled March 2026 covering OS-level sandboxing, permission models, network isolation,
filesystem isolation, WASM sandboxing, multi-agent architectures, runtime monitoring, commercial
platforms, MCP server sandboxing, and academic papers.

## Files

| File | Topic |
|------|-------|
| `os_level_sandboxing.md` | gVisor, Firecracker, seccomp, Landlock, AppArmor, SELinux, namespaces, cgroups |
| `permission_models.md` | Capability-based security, least privilege, tool approval flows, agent permission schemas |
| `network_isolation.md` | Egress control, DNS filtering, proxy-based controls, exfiltration prevention, air-gapped vs connected |
| `filesystem_sandboxing.md` | OverlayFS, Landlock, FUSE, git worktrees, AgentFS, secret protection |
| `wasm_sandboxing.md` | Wasmtime, WASI, Component Model, Extism, Spin, Fastly Compute, Wassette |
| `multi_agent_sandboxing.md` | Inter-agent isolation, confused deputy, trust boundaries, nested sandboxing |
| `runtime_monitoring.md` | eBPF, Tetragon, Falco, AgentSight, kill switches, CRIU, audit logging |
| `commercial_platforms.md` | E2B, Modal, Daytona, Runloop, Dagger, Nix, Devcontainers, Docker Sandboxes |
| `mcp_server_sandboxing.md` | MCP spec security model, transport security, container/WASM/OS sandboxing proposals |
| `academic_papers.md` | Key papers, frameworks (AEGIS, Progent, AgentSpec, LlamaFirewall, IsolateGPT) |
| `synthesis.md` | Consolidated architecture, comparison tables, recommendations |
