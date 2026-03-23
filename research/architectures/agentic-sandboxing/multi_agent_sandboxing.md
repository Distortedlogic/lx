# Multi-Agent Sandboxing Architectures

## Inter-Agent Isolation in Frameworks

### Current State

| Framework | Code Exec Isolation | Inter-Agent Isolation | Memory Isolation | Network Isolation |
|-----------|-------------------|----------------------|-----------------|-------------------|
| AutoGen | Docker per code exec task | None (shared Python process) | Shared | None |
| CrewAI | None built-in | Role-based context scoping | Shared crew store (SQLite) | None |
| LangGraph | None built-in | Graph-structure separation | Subgraph state | None |

None enforce OS-level isolation between agents by default. All rely on logical separation (context windows, role scoping, task boundaries).

### The Fundamental Problem

Current frameworks treat isolation as an application concern, not a security boundary. A prompt-injected agent can:
- Access other agents' tools
- Read/modify shared state
- Influence other agents through message manipulation
- Access host environment through shared runtime

---

## Inter-Agent Communication and Message Sanitization

### Attack Surface

Inter-LLM communication channels exploitable through prompt injection. Compromised outputs from one LLM cascade as malicious inputs to others. Memory poisoning and cross-agent inference are new attack surfaces.

### Sanitization Approaches

1. **NLP-aware sanitization**: Detect hidden instructions in natural-language payloads
2. **Structured messages**: Only pass typed JSON data, not free text (limits injection surface but reduces expressiveness)
3. **Message signing**: Ed25519 signatures prevent tampering by intermediaries
4. **Taint tracking**: Mark messages containing untrusted-source data
5. **Separate context windows**: Untrusted content in Agent A's context doesn't appear in Agent B's

### The Tension

Strict sanitization drops task success to 35-65% (Information-Theoretic Privacy Control, arxiv 2603.05520). Permissive communication enables cross-agent prompt injection cascades.

---

## Trust Boundaries Between Privilege Levels

### Hierarchical Trust Zones

```
Level 3: Orchestrator (highest)
  - Spawn/kill agents, access all resources
  - NOT exposed to untrusted input
Level 2: Privileged Agents
  - Specific APIs, databases, production writes
  - Receives sanitized input from Level 1
Level 1: Standard Agents
  - Workspace access, run tests, browse docs
  - Sandboxed code execution
Level 0: Untrusted Agents
  - Process untrusted content
  - Minimal capabilities (read-only)
  - Air-gapped from other agents
```

### IsolateGPT (NDSS 2025, Washington University)

Most rigorous academic treatment. Proposes a "hub" acting like an OS kernel:
- Intercepts user requests
- Routes with appropriate context to isolated apps
- Mediates collaboration between mutually distrusting apps
- Maintains system-wide context
- Performance overhead under 30% for 75% of queries

### SEAgent (arxiv 2601.11893)

Mandatory Access Control framework built on ABAC:
- Monitors agent-tool interactions via information flow graph
- Enforces customizable security policies based on entity attributes
- Blocks privilege escalation with low false positive rates

### BSI/ANSSI Zero-Trust LLM Principles

- LLM memory strictly isolated between users and sessions
- Every request needs fresh authentication
- Memory sanitization with controlled access to persistent content

---

## The Confused Deputy Problem

The confused deputy has made a significant comeback in AI agent systems.

### Examples

**Classic**: Malicious agent broadcasts crafted message → trusted smart lock agent invokes privileged tools on attacker's behalf.

**Agent-mediated lateral movement**: Attacker subverts decision layer that already wields legitimate identities. Control flow injected through untrusted content (webpages, emails become agent instructions).

**Semantic privilege escalation**: Agent tricked into reinterpreting role boundaries through crafted natural language. No technical exploit needed.

**Visual confused deputy** (arxiv 2603.14707): Computer-using agents tricked through visual manipulation (overlaid UI elements, manipulated screenshots).

### Defenses

Cannot be fixed with single guardrail. Requires:
- Least privilege (capabilities carried with request, not ambient)
- Policy checks on sensitive actions
- Step-up approvals
- Capability-based security with attenuation

---

## Sub-Agent Sandboxing Patterns

### Capability Monotonicity

Sub-agents must have ≤ capabilities of parent agent.

```
Parent (FS: /workspace, Network: github.com, Shell: yes)
├── Sub-Agent 1 (FS: /workspace/frontend, Network: none)
├── Sub-Agent 2 (FS: /workspace/backend, Network: github.com)
└── Sub-Agent 3 (FS: /workspace, Shell: yes)
✗ Sub-Agent 4 (Network: evil.com)  -- DENIED
```

### Two Patterns

**Isolate the tool**: Agent on trusted infrastructure, only generated code in sandbox. Simpler but agent process is attack surface.

**Isolate the agent**: Entire agent + tools in sandbox, communicate via control plane. More secure, more complex.

### Progent DSL

Fine-grained control: which tools permissible, conditions on arguments, fallback actions when blocked, dynamic policy updates.

---

## Claude Code Sub-Agent Isolation

### OS-Level Sandboxing

Bubblewrap (Linux) / Seatbelt (macOS) for filesystem and network isolation. 84% reduction in permission prompts.

### Git Worktree Isolation

`--worktree` flag starts Claude in isolated git worktree. Each agent gets own checkout, branch, working directory while sharing Git object database.

### Sub-Agent Architecture

Built-in sub-agents (Explore, Plan, Task):
- Own context window with custom system prompts
- Specific tool access and independent permissions
- Cannot spawn other sub-agents (prevents infinite nesting)

### Swarm Mode (2026)

Transforms Claude Code into multi-agent team orchestrator. Work distributed across agents with independent context windows for parallel reasoning.

---

## Docker-in-Docker and Nested Sandboxing

### The Problem

Agents need Docker to build/test code, but Docker socket access = root-equivalent on host.

### Docker Sandboxes (Jan 2026)

Each sandbox is a microVM with own kernel and private Docker daemon. Agent runs inside VM, cannot access host Docker. Currently the only solution allowing agents to build/run containers while remaining isolated.

### Sysbox (Nestybox/Docker)

"System containers" enabling Docker-in-Docker securely without privileged mode or host socket bind-mounting. Inner Docker totally isolated from host Docker.

### Kata + Firecracker

Firecracker: ~125ms boot, <5MiB per VM, 150 VMs/second/host.
Kata orchestrates multiple VMMs through standard container APIs.

### Rule of Thumb

Max 2 levels of nesting practical:
Host → Agent Sandbox (VM/container) → Agent's containers (rootless Podman)

---

## Real-World Multi-Agent Architectures

### Kubernetes Agent Sandbox (CNCF/SIG Apps)

- CRD: Sandbox, SandboxTemplate, SandboxClaim
- gVisor default, Kata Containers support
- Warm Pool Orchestrator for <1s cold starts
- https://github.com/kubernetes-sigs/agent-sandbox

### AgenticCyOps (arxiv 2603.09134)

Enterprise SOC multi-agent framework:
- MCP as structural basis
- Phase-scoped agents (Monitor, Analyze, Admin, Report)
- Consensus validation loops
- Reduces exploitable trust boundaries by 72% vs flat multi-agent

### Memory Poisoning in Production

- MINJA: Regular users can poison agent long-term memory through query-only interactions
- Contagious jailbreak: Malicious instructions spread through shared memory structures
- Agent Security Bench (ICLR 2025): Mixed attack success rates of 84.3%

---

## Key Takeaways

1. No framework provides strong inter-agent isolation by default
2. Confused deputy is the central security challenge
3. IsolateGPT's hub architecture is the most rigorous proposal
4. Docker Sandboxes (microVM-based) is the only production solution for agents building containers
5. K8s Agent Sandbox is the emerging standard for production
6. Message sanitization remains an unsolved tension (security vs coordination)
7. Sub-agents must always get ≤ parent capabilities
8. Defense-in-depth reduces exploitable trust boundaries by 72%

---

## Sources

- https://arxiv.org/abs/2403.04960
- https://arxiv.org/html/2601.11893v1
- https://arxiv.org/abs/2504.11703
- https://arxiv.org/abs/2603.14707
- https://arxiv.org/html/2603.09134
- https://arxiv.org/html/2508.19870v1
- https://arxiv.org/html/2603.05520
- https://arxiv.org/html/2602.11510v1
- https://arxiv.org/html/2601.05504v2
- https://www.bsi.bund.de/SharedDocs/Downloads/EN/BSI/Publications/ANSSI-BSI-joint-releases/LLM-based_Systems_Zero_Trust.pdf
- https://docs.docker.com/ai/sandboxes/architecture/
- https://blog.nestybox.com/2019/11/11/docker-sandbox.html
- https://github.com/kubernetes-sigs/agent-sandbox
- https://www.anthropic.com/engineering/claude-code-sandboxing
- https://docs.anthropic.com/en/docs/claude-code/sub-agents
- https://browser-use.com/posts/two-ways-to-sandbox-agents
- https://christian-schneider.net/blog/ai-agent-lateral-movement-attack-pivots/
