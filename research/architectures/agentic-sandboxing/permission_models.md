# Permission Models, Capability-Based Security, and Access Control for AI Agents

## How Coding Agents Handle Permissions

| Agent | Sandbox Tech | Permission Model | Network Controls | HITL |
|---|---|---|---|---|
| Claude Code | bwrap (Linux), Seatbelt (macOS) | deny/ask/allow rules with tool-qualified globs; 4 modes | Network isolation with approved host allowlist | Configurable per-tool |
| Cursor | Landlock+seccomp (Linux), Seatbelt (macOS), WSL2 (Win) | Sandboxed agents run freely inside boundaries | Domain-level allowlists (v2.5+) | Only when exiting sandbox |
| OpenAI Codex | OS-enforced workspace sandbox; cloud uses isolated containers | Permission profiles (workspace-write, danger-full-access) | Offline by default; per-project allowlists | Configurable approval policy |
| Devin | Isolated cloud VM (terminal + editor + browser) | Scoped GitHub permissions (r/w code, r/o deployments) | Cloud sandbox with port forwarding | PR-based review |
| SWE-agent | Docker containers | No granular permission model beyond container | Container-level network isolation | Manual monitoring |
| OpenHands | Docker sandbox per session | Configurable runtime class | Container networking | Event stream audit |
| Aider | None (local execution) | Relies on user trust, git for undo | None | None |

### Claude Code Permission System

Three-tier rule system: `deny` > `ask` > `allow`

Tool-qualified glob patterns:
- `Bash(npm run lint)` -- allow specific bash command
- `Read(./.env)` -- control file read access
- `Write(src/**)` -- scope file writes

Rules cascade: managed settings → user settings (~/.claude/settings.json) → project settings (.claude/settings.json)
Deny at any level is unoverridable.

Four modes: plan, normal, auto-accept, bypass

Known issue: permission deny rules in settings files are non-functional for Read and Write tools (GitHub #6631).

### OpenAI Codex Permission System

Permission profiles:
- `workspace-write`: Constrained to workspace directory
- `danger-full-access`: No restrictions

`require_approval` configurable per-tool or globally.

Two-phase runtime: setup phase (online, for deps) → agent phase (offline by default).

---

## Capability-Based Security (Object-Capability Model)

The ocap model: access to a resource requires holding an unforgeable capability reference. Capabilities can be attenuated (restricted) before delegation. Zero ambient authority -- you only have what you were explicitly given.

### Direct Applications to AI Agents

**WASI (WebAssembly System Interface)**:
- WASM module has zero capabilities by default
- Host explicitly passes file descriptors, network handles, etc.
- Module cannot access anything not given to it
- Capabilities are attenuated: read-only access to a directory instead of full access
- The clearest implementation of ocap for agent sandboxing

**Tenuo**:
- Capability-based authorization for AI agents
- Cryptographic warrants with offline attenuation
- Enforces least-privilege boundaries on LLM tool calls

**MCP OAuth-based authorization**:
- Functions as a de facto capability token system
- Access tokens scoped via RFC 8707 Resource Indicators bound to specific MCP servers

### Proposed Agent Capability Schema

```yaml
AgentCapabilities:
  filesystem:
    read: ["/workspace/**", "/data/readonly/**"]
    write: ["/workspace/**"]
    execute: ["/usr/bin/python3", "/usr/bin/git"]
  network:
    outbound: ["api.github.com:443", "pypi.org:443"]
    inbound: none
  tools:
    allowed: ["code_interpreter", "file_search"]
    blocked: ["shell_exec", "email_send"]
  resources:
    cpu: "2 cores"
    memory: "4GB"
    disk: "10GB"
    timeout: "30 minutes"
```

---

## Permission Escalation Risks

### Attack Vectors

**Prompt injection → tool abuse**: Primary vector. "Your AI, My Shell" (arXiv:2509.22040) demonstrated up to 84% attack success rates on Copilot and Cursor using 314 payloads covering 70 MITRE ATT&CK techniques.

**Tool chaining escalation**: Read config → extract creds → use creds with API tool.

**Indirect permission escalation**: Modify cron jobs, CI/CD pipelines, git hooks.

**Sub-agent escalation**: Parent spawns sub-agent with elevated privileges.

**Env var exfiltration**: Agent reads API keys from environment variables.

**Semantic privilege escalation**: Agent reinterprets role boundaries through carefully crafted natural language, no technical exploit needed.

### Mitigations

1. Separation of reasoning from execution (injected instructions remain inert)
2. OS-level sandboxing (prevents prompt-injected agents from modifying system)
3. Dynamic privilege de-escalation (auto-reduce to minimum when idle)
4. Time-bounded privileges (expire after task/timeout)
5. Multi-step approval for high-risk operations
6. Treat the LLM as a hostile user (API gateways, rate limiters, IAM boundaries)
7. Monotonic capability attenuation (sub-agents get ≤ parent capabilities)

---

## Standards and Proposals

### MCP Authorization (Nov 2025 spec)

- Servers classified as OAuth 2.1 Resource Servers
- Clients must implement RFC 8707 Resource Indicators to bind tokens to specific servers
- Tools support `require_approval`: globally, per-tool map, or grouped
- Tool annotations are UNTRUSTED by default unless from a trusted server
- OAuth scopes follow least-privilege with step-up authorization

### OWASP Top 10 for Agentic Applications (2026)

- First globally peer-reviewed framework for agentic AI security risks
- Key principle: **Least Agency** -- only grant minimum autonomy for safe, bounded tasks
- Mandates trust boundaries between agents with validation of inter-agent comms

### FINOS AI Governance Framework v2.0

- Agent identity in all API invocations
- Contextual privilege adjustment based on risk/value/sensitivity
- Time-bounded privileges auto-expiring after task completion
- Separation of duties preventing single agents from completing high-risk workflows

### Academic Frameworks

**MiniScope** (UC Berkeley, Dec 2025): Permission solver analyzing tasks to determine minimum privilege scope. 1-6% latency overhead.

**Progent** (Apr 2025): First privilege control framework for LLM agents with a DSL. Reduces attack success to 0% on AgentDojo/ASB/AgentPoison while preserving utility.

---

## Key Gap

No universally adopted, cross-vendor standard for agent permission schemas exists. MCP's authorization focuses on server-to-server auth, not fine-grained tool-level permissions. Academic work (MiniScope, Progent) proposes formal frameworks but none are adopted as standards.

---

## Sources

- https://arxiv.org/abs/2509.22040
- https://arxiv.org/abs/2601.17548
- https://arxiv.org/abs/2512.11147
- https://arxiv.org/abs/2504.11703
- https://www.franziroesner.com/pdf/wu-agentperms-sp26.pdf
- https://modelcontextprotocol.io/specification/2025-11-25
- https://auth0.com/blog/mcp-specs-update-all-about-auth/
- https://genai.owasp.org/resource/owasp-top-10-for-agentic-applications-for-2026/
- https://air-governance-framework.finos.org/mitigations/mi-18_agent-authority-least-privilege-framework.html
- https://cheatsheetseries.owasp.org/cheatsheets/AI_Agent_Security_Cheat_Sheet.html
- https://www.pillar.security/blog/the-hidden-security-risks-of-swe-agents-like-openai-codex-and-devin-ai
- https://vercel.com/blog/security-boundaries-in-agentic-architectures
- https://docs.aws.amazon.com/wellarchitected/latest/generative-ai-lens/gensec05-bp01.html
- https://github.com/anthropics/claude-code/issues/6631
