# A2A and MCP Protocol Analysis

## Overview

Two emerging interop standards for AI agents: **MCP** (Model Context Protocol, Anthropic) for agent-to-tool connectivity, and **A2A** (Agent-to-Agent Protocol, Google) for agent-to-agent coordination. Both use HTTP + JSON-RPC 2.0. Both donated to the Linux Foundation. Together they define the protocol landscape lx must navigate.

---

## MCP (Model Context Protocol)

### Identity

Created by Anthropic, announced November 2024. Inspired by the Language Server Protocol (LSP). Donated to the Agentic AI Foundation (AAIF) under Linux Foundation, December 2025. Co-founded by Anthropic, Block, and OpenAI. 97M+ monthly SDK downloads, 5,800+ MCP servers, 300+ MCP clients. Adopted by OpenAI (March 2025), Microsoft/GitHub (May 2025).

### Architecture

Client-server model over JSON-RPC 2.0. Three roles: **Host** (LLM application), **Client** (connector within host, one per server), **Server** (provides capabilities).

**Transports:**
- **STDIO:** Client launches server as subprocess. JSON-RPC on stdin/stdout. Simplest, most performant for local single-client.
- **Streamable HTTP (recommended for remote):** Single endpoint supporting POST and GET. Optional SSE for streaming. Session ID via `Mcp-Session-Id` header. Stateless architecture for horizontal scaling.
- **SSE (legacy, deprecated):** Two separate endpoints. Superseded by Streamable HTTP.

### Three Server Primitives

| Primitive | Control | Purpose |
|-----------|---------|---------|
| **Tools** | Model-controlled | Executable functions the LLM discovers and invokes |
| **Resources** | Application-controlled | Data/context exposed to the model |
| **Prompts** | User-controlled | Reusable structured message templates |

### Two Client Features

| Feature | Purpose |
|---------|---------|
| **Sampling** | Servers request LLM completions FROM the client (reverse direction) |
| **Elicitation** | Servers request information FROM users |

### Tool Definition

Discovery: `tools/list` with cursor pagination. Tool schema uses JSON Schema for `inputSchema` and `outputSchema`. Behavioral hints via `annotations`: `readOnlyHint`, `destructiveHint`, `idempotentHint`, `openWorldHint`.

Invocation: `tools/call` with `name` and `arguments`. Results: `text`, `image`, `audio`, `resource_link`, `resource`, `structuredContent`. Two error types: protocol errors (JSON-RPC) and tool execution errors (`isError: true`, actionable feedback for LLM self-correction).

### Resources

Application-controlled context. Discovery via `resources/list` and `resources/templates/list` (RFC 6570 URI templates). Reading via `resources/read`. Subscriptions: `resources/subscribe` for change notifications.

### Sampling (Reverse LLM Calls)

Servers can request LLM completions from the client. Enables agentic behaviors nested inside MCP features. Includes `tools` array and `toolChoice` for multi-turn tool loops within sampling. Model preferences are advisory (hints, cost/speed/intelligence priorities).

### Security

OAuth 2.1 with PKCE mandatory. Client ID Metadata Documents (CIMD) replace Dynamic Client Registration. Client-Credentials for M2M. Cross App Access (XAA) for enterprise SSO. Incremental scope negotiation (zero trust). Human-in-the-loop SHOULD always be present for sampling.

### The Token Cost Problem

**Specific measurements:**
- GitHub MCP: 55,000 tokens (93 tools)
- Playwright MCP: 13,700 tokens (21 tools)
- Chrome DevTools MCP: 18,000 tokens (26 tools)
- Average tool definition: 300-600 tokens each
- Power user (10 servers × 15 tools × 500 tokens): **75,000 tokens** before any user input

**Impact:** Output degradation beyond ~50 tools. Cursor enforces hard limit of 40 tools. ~$375/month overhead for a 5-person team at $5/MTok.

**Mitigation strategies:**
1. **Tool deferral** (Claude Code): Load schemas on demand. One session saved 13.2k tokens.
2. **Hierarchical routing:** Replace 100 tools with 2 meta-tools (`discover` + `execute`). 75k → ~1.4k tokens.
3. **Pi's alternative:** CLI tools with README files. Agent reads docs only when needed. Token cost pay-as-you-go. No server infrastructure.
4. **Dynamic toolsets:** Speakeasy reported 100x reduction loading tools contextually.

### Criticisms

**Prompt injection (#1 risk).** Simon Willison: "The great challenge of prompt injection is that LLMs will trust anything that can send them convincing sounding tokens." MCPTox: o1-mini had 72.8% attack success rate.

**Tool poisoning.** Malicious instructions in tool descriptions. Rug pull attacks: tools mutate definitions post-installation.

**Command injection.** 43% of analyzed servers had flaws.

**No enforcement at protocol level.** Security principles are SHOULD-level recommendations.

---

## A2A (Agent-to-Agent Protocol)

### Identity

Designed by Google, announced April 2025. V1.0 current, with gRPC support, signed agent cards. Linux Foundation governance (June 2025). 150+ supporting organizations including Atlassian, Box, Cohere, Intuit, LangChain, MongoDB, PayPal, Salesforce, SAP.

### Architecture

HTTP + JSON-RPC 2.0 + SSE. Optional gRPC binding. Three protocol bindings: JSON-RPC over HTTP/WebSocket, gRPC (native streaming), HTTP+JSON/REST.

### Agent Cards (Discovery)

JSON metadata document at `https://{domain}/.well-known/agent-card.json` (RFC 8615). Contains: name, description, provider, capabilities (streaming, pushNotifications), interfaces (URL, protocol version, preferred transport), skills (id, name, description, inputModes, outputModes, examples), security schemes (OpenAPI-aligned), signature (JWS-based, RFC 7515 + RFC 8785).

Three discovery mechanisms: well-known URI, registry-based, direct configuration.

### Task Model

Task is the core unit of action. States (SCREAMING_SNAKE_CASE in v1.0): `ACCEPTED`, `WORKING`, `INPUT_REQUIRED`, `AUTH_REQUIRED`, `COMPLETED`, `FAILED`, `CANCELED`, `REJECTED`. Terminal states cannot restart; subsequent work requires new tasks within the same `contextId`.

`contextId` groups related tasks. `referenceTaskIds` enable cross-references. `returnImmediately` parameter for async handling.

### Communication Patterns

**Request/Response:** `SendMessage`, blocks until terminal (unless `returnImmediately: true`).
**Streaming:** `SendStreamingMessage` with SSE. Events: `TaskStatusUpdateEvent`, `TaskArtifactUpdateEvent`.
**Push notifications:** Webhooks via `PushNotificationConfig`.
**Polling:** `GetTask` periodically.

### Parts and Artifacts

`Part` (unified in v1.0): `text`, `url`, `raw` (binary), `data` (structured JSON). Each with `mime_type` and `metadata`.

`Artifact`: Immutable output with id, name, description, parts, mimeType.

### Security

HTTPS + TLS 1.2+ mandatory. OpenAPI-aligned auth (API Keys, OAuth 2.0/OIDC, mTLS). Device Code flow for CLI/IoT. PKCE mandatory for auth code flow. Agent Card signing via JWS.

### Framework Support

Google ADK (native), LangGraph, CrewAI, Semantic Kernel, LlamaIndex, AutoGen, BeeAI. `hybroai/a2a-adapter` provides adapter SDKs.

### Criticisms

**Point-to-point scaling.** N-squared connections as agents grow. Needs complementary event mesh for large deployments.

**Weak skill schemas.** Agent Cards list skills with descriptions but no machine-readable input/output definitions (no JSON Schema equivalent). Automated orchestration is harder without typed contracts.

**Security gaps.** Tool squatting (registering fake agents). Malicious instruction propagation between agents. No SCA requirements. Insufficient token scoping.

**No monitoring tools.** No integrated observability covering both A2A and MCP stacks.

---

## Cross-Protocol Analysis

### Relationship

The canonical framing is accurate: **MCP = vertical (agent-to-tool), A2A = horizontal (agent-to-agent).** One agent uses MCP internally for tools, then hands off to another agent via A2A. Both use HTTP + JSON-RPC 2.0 underneath.

### What lx Needs to Support

**Essential:**
- Tool invocation with JSON Schema parameters (MCP's model)
- Task lifecycle (working/completed/failed/input-required/canceled)
- Streaming (SSE/Streamable HTTP)
- Agent discovery/addressing (Agent Cards or equivalent)
- Typed message passing (text, data, file parts)
- Capability negotiation
- Context continuity (`contextId`)

**Important:**
- **Progressive tool disclosure** -- do NOT statically inject all tool definitions (the token cost problem is real)
- Structured output schemas
- Push notifications/webhooks for async
- Multi-turn interactions with input-required state
- OAuth 2.1/PKCE for auth

**Nice-to-have:**
- Sampling (reverse LLM calls)
- Resource subscriptions
- Prompt templates
- gRPC binding
- Agent Card signing

### Design Insight for lx

The protocol layer is a transport concern. lx should express WHAT agents do and HOW they coordinate, not WHICH wire protocol they use. Support MCP and A2A as pluggable connectors. lx's native constructs (tasks, messages, tools, agents) should map cleanly onto both protocols but not be constrained by either.

Pi's CLI-tools-with-README alternative validates that MCP is not the only viable tool integration model. lx should support both: declared tool bindings that compile to MCP tool definitions AND simpler bash-invocable tools with lazy documentation loading. Let the user choose the token cost tradeoff.