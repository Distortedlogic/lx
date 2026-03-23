# MCP Server Sandboxing

## Current State: No Major Client Sandboxes MCP Servers

### Claude Code

Most mature story. Uses `sandbox-runtime` (bwrap/Seatbelt) for OS-level filesystem and network isolation. MCP servers can be sandboxed with same runtime. stdio transport means no network exposure.

### Cursor

Weakest isolation. Servers run as fully trusted integrations. Pins trust to MCP server key name in config, not actual command. Led to CVE-2025-54136 ("MCPoison") where repo .cursor/mcp.json could be swapped to substitute malicious command.

### Docker MCP Toolkit

Container-per-server isolation. Each MCP server in own Docker container (1 CPU, 2GB memory per tool container). MCP Gateway adds cryptographic image signing. Most production-ready container-based approach.

### VS Code / Others

MCP servers run as child processes with user's full permissions. Trust based on configuration only.

---

## Security Risks

### Tool Poisoning / Prompt Injection

Malicious instructions in tool descriptions are invisible to users but visible to the LLM. WhatsApp MCP attack (April 2025) used this to steal chat histories without code exploit.

### Rug Pull Attacks

Tool definitions can mutate after installation. Approved tool on Day 1 silently changes behavior by Day 7. Definitions dynamically loaded, not pinned to verified hash.

### Cross-Server Exfiltration

Malicious server uses prompt injection to instruct agent to query legitimate server for sensitive data, then passes to attacker via tool call to malicious server.

### Shadow Tool Attacks

Malicious server registers tools with names identical to legitimate tools on other servers.

### DNS Rebinding

CVE-2025-66414 (TypeScript SDK < 1.24.0) and CVE-2025-66416 (Python SDK < 1.23.0): localhost-bound SSE/StreamableHTTP servers lacked DNS rebinding protection. Malicious websites could pivot through browser to invoke tools on local MCP servers.

### Supply Chain

Between Jan-Feb 2026: 30+ CVEs filed targeting MCP servers/clients/infrastructure. Of 2,614 implementations surveyed, 82% use file operations vulnerable to path traversal.

---

## Sandboxing Proposals

### Container-Per-Server (Docker MCP Toolkit)

```
Claude Code
├── MCP Server A → Docker Container A (network=none, /data:ro)
├── MCP Server B → Docker Container B (network=api.github.com, /workspace:rw)
└── MCP Server C → Docker Container C (network=none, /workspace:ro)
```

Most production-ready. ~50-200ms startup overhead per container.

### WASM-Based (Wassette, Cosmonic)

**Wassette** (Microsoft): Wasmtime-based runtime running WASM Components as MCP tools. Deny-by-default. Agents fetch components from OCI registries. Integrated with Claude Code, Copilot, Cursor, Gemini.

**Cosmonic Sandbox MCP**: Built on CNCF wasmCloud. Shared-nothing sandbox enforcing least privilege.

Strongest capability model but requires tools compiled to WASM.

### OS-Level (Anthropic sandbox-runtime)

Bubblewrap/Seatbelt without containers. Lightest weight but Linux/macOS only.

### Landlock-Based

```
Claude Code
├── fork() → Landlock restrict_self() → MCP Server A (FS: /data:ro)
├── fork() → Landlock restrict_self() → MCP Server B (FS: /workspace:rw, Net: TCP)
```

Nearly zero overhead, works with existing servers, unprivileged. Requires Linux 5.13+ (6.7+ for network).

### Bubblewrap-Based

```bash
bwrap --ro-bind / / --dev /dev --bind /workspace /workspace --unshare-net -- mcp-server
```

Lightweight namespaces, no container daemon, no root. Linux only.

### MCP Gateway/Proxy

Acuvity MiniBridge, Invariant Gateway: intermediary proxies enforcing auth, authz, input validation, logging between client and servers.

---

## MCP Specification Security Model

### What the Spec Defines

- Client-host-server architecture with "clear security boundaries"
- Capability declarations are self-asserted without verification
- Tool annotations MUST be considered untrusted unless from trusted server
- Hosts MUST obtain explicit user consent before invoking any tool
- Token audience validation: servers MUST NOT accept tokens not issued for them
- OAuth 2.1 for authorization (servers as resource servers, hosts as OAuth clients)
- Nov 2025: SEP-1046 (OAuth client credentials for M2M), SEP-990 (Enterprise IdP)

### What's Missing

| Feature | Status |
|---------|--------|
| Server permission declaration | Missing |
| Capability-based access control | Missing |
| Server code signing | Missing |
| Inter-server isolation | Missing |
| Resource limits specification | Missing |
| Audit logging standard | Missing |
| Sandbox configuration format | Missing |

The spec explicitly states it cannot enforce security at the protocol level. Sandboxing is entirely client implementor's responsibility.

### 2026 Roadmap

"Deeper security and authorization work" listed as "on the horizon" but NOT in top four priorities. Next spec release tentatively June 2026.

---

## Transport Security

| Aspect | stdio | SSE (deprecated) | Streamable HTTP |
|--------|-------|-------------------|-----------------|
| Encryption | N/A (local) | Needs TLS | Needs TLS |
| Authentication | Implicit (local) | None built-in | OAuth 2.1 |
| Attack surface | None (local pipes) | HTTP | HTTP |
| Best for | Local/CLI | Deprecated | Remote/production |

SSE deprecated as of protocol version 2024-11-05 (tokens in URL query strings, single identity check at establishment). Streamable HTTP is recommended for all remote deployments.

---

## Permission Scoping (Proposed)

```json
{
  "mcpServers": {
    "filesystem-server": {
      "command": "npx",
      "args": ["-y", "@mcp/server-filesystem", "/workspace"],
      "permissions": {
        "filesystem": {
          "read": ["/workspace/**", "/tmp/mcp-*"],
          "write": ["/workspace/**"],
          "deny": ["**/.env", "**/*.pem"]
        },
        "network": { "outbound": "none" },
        "process": { "spawn": false },
        "resources": { "memory": "512MB", "cpu": "1 core", "timeout": "30s" }
      }
    }
  }
}
```

### Scope-Based Restrictions

- Servers emit precise scope challenges (mcp:tools:read, mcp:tools:write)
- Default deny: new tools not accessible until policies permit
- JIT access: time-limited grants rather than permanent permissions
- Third-party enforcement: Cerbos, Acuvity policy engines between client and server

---

## Defense-in-Depth Architecture

```
Layer 1: Installation Security
  Code signing, dependency scanning, source audit
Layer 2: Process Isolation
  Container / Landlock / bwrap sandbox, separate user, cgroups
Layer 3: Capability Restriction
  FS: only declared paths. Network: only declared endpoints. Process: limited spawn.
Layer 4: Communication Security
  Tool result sanitization, output scanning for credentials, rate limiting
Layer 5: Runtime Monitoring
  eBPF syscall auditing, anomaly detection, kill switch
Layer 6: Human Oversight
  Tool call approval for sensitive actions, audit log review
```

---

## Community Discussions

### GitHub Issues

- **#544**: Insufficient security design increasing phishing risk via proxy servers
- **#630**: "Server" terminology creates dangerous user misconceptions
- **#1452**: Security Working Group meeting notes (Sept 2025), reviewed HTTP Message Signing

### Analysis

- **Doyensec (Mar 2026)**: "The MCP AuthN/Z Nightmare" documenting fundamental architectural problems
- **Stacklok**: Solving MCP server trust problem via namespace authentication + Sigstore verification
- **67,057 MCP servers across 6 registries**: substantial number hijackable due to lack of vetted submission

### Community Consensus

1. MCP servers are a significant attack surface
2. Sandboxing should be a priority
3. Spec should define permission format
4. Clients should enforce isolation
5. Backward compatibility matters
6. UX is critical (complex permission management won't be adopted)

---

## Sources

- https://modelcontextprotocol.io/specification/draft/basic/security_best_practices
- https://modelcontextprotocol.io/specification/draft/basic/authorization
- https://blog.modelcontextprotocol.io/posts/2026-mcp-roadmap/
- https://simonwillison.net/2025/Apr/9/mcp-prompt-injection/
- https://unit42.paloaltonetworks.com/model-context-protocol-attack-vectors/
- https://authzed.com/blog/timeline-mcp-breaches
- https://www.heyuan110.com/posts/ai/2026-03-10-mcp-security-2026/
- https://www.docker.com/blog/mcp-horror-stories-github-prompt-injection/
- https://opensource.microsoft.com/blog/2025/08/06/introducing-wassette-webassembly-based-tools-for-ai-agents/
- https://christian-schneider.net/blog/securing-mcp-defense-first-architecture/
- https://www.straiker.ai/blog/agentic-danger-dns-rebinding-exposing-your-internal-mcp-servers
- https://www.varonis.com/blog/model-context-protocol-dns-rebind-attack
- https://www.cerbos.dev/blog/mcp-authorization
- https://auth0.com/blog/mcp-streamable-http/
- https://blog.doyensec.com/2026/03/05/mcp-nightmare.html
- https://stacklok.com/blog/from-unknown-to-verified-solving-the-mcp-server-trust-problem/
- https://docs.docker.com/ai/mcp-catalog-and-toolkit/toolkit/
- https://github.com/anthropic-experimental/sandbox-runtime
- https://cheatsheetseries.owasp.org/cheatsheets/MCP_Security_Cheat_Sheet.html
