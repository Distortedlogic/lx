# Commercial and Open-Source Platforms for Sandboxed AI Agent Execution

## Platform Comparison

| Platform | Isolation | Cold Start | GPU | Self-Host | Pricing (1 vCPU/hr) | Open Source |
|---|---|---|---|---|---|---|
| E2B | Firecracker microVM | ~150ms | No | Yes* | ~$0.05 | Apache-2.0 |
| Modal | gVisor | Sub-second | Yes (H100/A100) | No | ~$0.30 | No |
| Daytona | Docker/Kata | 27-90ms | Yes | Yes | ~$0.067 | AGPL-3.0 |
| Morph Cloud | Snapshot/instance VMs | Fast | Unknown | No | Unknown | No |
| Runloop | MicroVM | <2s (10GB images) | Unknown | BYOC | Usage-based | No |
| Dagger (Container Use) | Docker + worktrees | Seconds | No | Yes (BYO) | Free | Yes |
| Fly.io Sprites | Firecracker | 1-2s (300ms restore) | No | No | Usage-based | No |
| K8s Agent Sandbox | gVisor + Kata | <1s (warm pool) | Via K8s | Yes (BYO K8s) | Free | Yes |
| Docker Sandboxes | MicroVM + private daemon | Seconds | No | Yes (Desktop) | Free (BYO) | Partial |
| Nix/NixOS | Configurable | Varies | No | Yes | Free | Yes |
| Devcontainers | Docker + firewall | Seconds | No | Yes | Free | Yes (spec) |

---

## E2B (e2b.dev)

**Architecture**: Firecracker microVMs. Each sandbox gets own kernel (KVM). Only 5 emulated devices, jailer companion applies cgroups/seccomp/chroot.

**Sandboxes**: Defined via Dockerfile → snapshotted microVM image (not container). Restore from snapshot yields ~150ms cold start, <5MiB overhead. Up to 24hr sessions.

**SDK**: Python, TypeScript. Also provides MCP server for tool-use integration.

**Pricing**: Hobby free ($100 credit). Pro $150/mo. ~$0.05/hr per vCPU. Enterprise BYOC/on-prem.

**Open source**: https://github.com/e2b-dev/E2B (Apache-2.0), infra uses Terraform + Nomad.

**Adoption**: ~50% Fortune 500. Used by Hugging Face, Groq, Manus AI.

---

## Modal

**Architecture**: gVisor (user-space kernel). Container-level isolation, not VM-level. Lazy-loading FUSE filesystem for fast image pulls.

**Key differentiator**: First-class GPU support (H100, A100). Autoscales 0 to 50K+ concurrent. Code-first Python SDK (no YAML).

**Sandbox features**: Runtime-defined environments, timeout-bounded lifecycle, granular egress policies.

**Pricing**: ~$0.30/hr CPU, ~$1/hr GPU. Free tier $30/mo.

**Notable users**: Lovable, Quora (millions of untrusted executions/day).

---

## Daytona

**Architecture**: Docker containers (default), Kata Containers for hardware-level isolation, Sysbox for rootless. Long-lived workspaces (not ephemeral).

**Key differentiator**: Fastest cold start (27-90ms). Lifecycle automation: auto-stop, auto-archive, auto-delete.

**SDK**: Python, TypeScript. Direct access to process exec, file ops, Git, code analysis.

**Pricing**: $0.067/hr for 1 vCPU/1 GiB. $200 free credits. GPU available.

**Funding**: $24M Series A (Feb 2026).

**Open source**: https://github.com/daytonaio/daytona (AGPL-3.0).

---

## Morph Cloud (morph.so)

**Architecture**: Snapshot-and-fork model. Two primitives: snapshots (immutable) and instances (ephemeral).

**Key differentiator**: Infinibranch -- instant environment branching for parallel exploration with minimal overhead.

**Use cases**: RL environments, test-time scaling, CI/CD debugging, parallel agent exploration.

**SDK**: Python, TypeScript.

---

## Runloop

**Architecture**: MicroVM-based devboxes. SOC 2 compliant.

**Features**: Blueprints (pre-configured templates), snapshots, auto-scaling CPU/memory. Custom images up to 10GB boot in <2s. 10K+ parallel instances.

**Deployment**: Fully hosted or BYOC (your AWS/GCP).

**Funding**: $7M seed (July 2025).

---

## Dagger (Container Use)

**Architecture**: MCP server giving each agent Docker container + Git worktree. Content-addressable caching across parallel environments.

**Key differentiator**: Code isolation via worktrees + runtime isolation via containers. Branch per agent, no merge conflicts during execution.

**DX**: Real-time command history/logs, drop into any agent's terminal. Works as MCP server with Claude Code, Cursor.

**Cost**: Free and open source. BYO compute.

https://github.com/dagger/container-use

---

## Fly.io Sprites (Jan 2026)

**Architecture**: Persistent Firecracker microVMs for AI coding agents. 100GB NVMe persistent root filesystem.

**Key differentiator**: Checkpoint/restore in ~300ms. Auto-idle (billing stops, state preserved). Comes with Claude pre-installed.

https://sprites.dev/

---

## Docker Sandboxes (v4.60+, Jan 2026)

**Architecture**: MicroVM with own kernel and private Docker daemon per agent.

**Key differentiator**: Only solution allowing agents to build/run Docker containers while remaining isolated. HTTP/HTTPS filtering proxy for egress.

**Supports**: Claude Code, Gemini, Codex, Copilot, Agent, Kiro natively.

**Platform**: macOS/Windows (Docker Desktop). Linux on roadmap.

---

## Kubernetes Agent Sandbox (CNCF SIG Apps)

**Architecture**: K8s CRD with gVisor (default) + Kata Containers. Warm Pool Orchestrator for <1s cold starts.

**CRDs**: Sandbox, SandboxTemplate, SandboxClaim.

**Open source**: https://github.com/kubernetes-sigs/agent-sandbox

Positioning as the open standard for K8s-native agent execution.

---

## Nix/NixOS

**Approaches**:

**microvm.nix** (Stapelberg, Feb 2026): Ephemeral microVMs on NixOS. Shared workspace, 8GB disk overlay, configurable hypervisor. Entire setup in flake.nix with cryptographic hash guarantees.

**nix-sandbox-mcp**: MCP server providing shell/Python/Node sandboxes with project mounted read-only. jail.nix (bwrap + namespaces), planned microvm.nix backend.

**agent-sandbox.nix**: Bubblewrap sandbox unsharing PID/user/IPC/UTS/cgroup namespaces. No network by default.

**Strengths**: Unmatched reproducibility. Declarative. Composable isolation layers. Free.

**Weaknesses**: Steep learning curve. No managed service.

---

## Devcontainers

Standardized .devcontainer format for containerized dev environments.

**For agents**: Anthropic published official reference devcontainer for Claude Code:
- Multi-layered firewall (iptables + ipset)
- Default-deny egress, allowlist: GitHub, Anthropic API, npm, Statsig, Sentry
- Enables `--dangerously-skip-permissions` for unattended operation
- Trail of Bits published security-hardened variant

**Weakness**: Shared kernel. Malicious project can still exfiltrate creds accessible within container.

---

## YC / Startup Accelerator Companies

**Castari (YC)**: "Vercel for AI Agents." Deploy agents in secure autoscaling sandboxes. Built on Claude Agent SDK. Private beta.

**Blaxel (YC S25)**: Perpetual sandbox platform. Environments resume in <25ms. Co-locates agent APIs alongside sandboxes.

**Cua (YC)**: "Give every agent a cloud desktop." Full desktop environments for agents.

**Lifo**: Browser-native OS for AI sandboxing. Runs entirely in-browser (IndexedDB, Fetch, Web Workers behind POSIX interfaces). Sub-millisecond latency. MIT license.

Nearly 50% of recent YC batches are AI agent companies. Agent infrastructure is an explicit "Requests for Startups" category.

---

## Key Tradeoffs

**Strongest isolation**: E2B, Fly.io Sprites, Runloop (hardware-level Firecracker/KVM)

**Fastest cold start**: Daytona (27-90ms) > Blaxel (25ms) > E2B (~150ms) > Modal (sub-second)

**Best for GPU**: Modal (purpose-built for ML)

**Best DX**: E2B (most polished SDKs, largest community)

**Best for persistent/stateful**: Fly.io Sprites (100GB persistent NVMe, auto-idle) and Daytona (long-lived workspace)

**Best for parallel multi-agent**: Dagger Container Use (worktree per agent, free) and Morph Cloud (snapshot-and-fork)

**Best for self-hosted/reproducible**: Nix/NixOS and Devcontainers

**Best for enterprise**: Runloop (SOC 2) and E2B Enterprise (BYOC)

**Cheapest CPU-only**: E2B at $0.05/hr. Dagger/Nix/Devcontainers free (BYO compute)

**Emerging standard**: K8s Agent Sandbox CRD for Kubernetes-native deployments

---

## Sources

- https://e2b.dev/
- https://github.com/e2b-dev/E2B
- https://modal.com/
- https://modal.com/docs/guide/sandboxes
- https://www.daytona.io/
- https://github.com/daytonaio/daytona
- https://cloud.morph.so/docs/developers
- https://www.runloop.ai/
- https://docs.runloop.ai/docs/devboxes/overview
- https://dagger.io/
- https://github.com/dagger/container-use
- https://sprites.dev/
- https://docs.docker.com/ai/sandboxes/architecture/
- https://github.com/kubernetes-sigs/agent-sandbox
- https://michael.stapelberg.ch/posts/2026-02-01-coding-agent-microvm-nix/
- https://code.claude.com/docs/en/devcontainer
- https://github.com/trailofbits/claude-code-devcontainer
- https://modal.com/blog/top-code-agent-sandbox-products
- https://northflank.com/blog/best-sandboxes-for-coding-agents
- https://e2b.dev/blog/yc-companies-ai-agents
