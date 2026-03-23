# MCP Server Design Patterns

Architecture, primitives, transport, security, and composition patterns for production MCP servers.

## Core Architecture: Host / Client / Server

```
Host (Claude Code, Cursor, etc.)
  +-- MCP Client 1 ---- MCP Server A (filesystem)
  +-- MCP Client 2 ---- MCP Server B (workflow)
  +-- MCP Client 3 ---- MCP Server C (code search)
```

- **Host**: User-facing application. Creates and manages MCP clients.
- **Client**: Protocol entity maintaining 1:1 connection with a server. Handles capability negotiation and message routing.
- **Server**: Lightweight process exposing capabilities (tools, resources, prompts) via MCP protocol.

## Protocol Lifecycle

1. **Initialize**: Client sends `initialize` with capabilities and protocol version
2. **Capability Negotiation**: Both sides declare supported features
3. **Steady State**: JSON-RPC 2.0 messages flow bidirectionally
4. **Shutdown**: Either side terminates cleanly

## When to Use Each Primitive

### Tools (Model-Controlled)

Functions the LLM decides to call. Can have side effects.

| Use For | Examples |
|---|---|
| Computations | `calculate_hash`, `parse_ast` |
| API calls | `create_issue`, `send_message` |
| Side effects | `write_file`, `run_test`, `git_commit` |
| Queries requiring logic | `search_code`, `find_references` |

### Resources (Application-Controlled)

Data exposed for reading, identified by URIs. Read-only.

| Use For | Examples |
|---|---|
| File contents | `file:///path/to/file.rs` |
| Database records | `db://tasks/42` |
| Configuration | `config://workspace/settings` |
| Parameterized access | `symbol://{name}` (URI templates) |

### Prompts (User-Controlled)

Pre-built prompt templates expandable with arguments.

### Decision Matrix

| Question | Tool | Resource | Prompt |
|---|---|---|---|
| Who controls it? | LLM | App/User | User |
| Has side effects? | Can | No | No |
| Addressed by? | Name + params | URI | Name + args |
| Returns? | Action results | Data content | Messages |

## Transport Selection

### stdio (Local)

Server runs as child process, communicates via stdin/stdout with newline-delimited JSON-RPC. Best for: local dev tools, single-user, CLI integrations, security-sensitive operations. Zero config, natural process isolation, no network attack surface.

### Streamable HTTP (Current Standard, replaces SSE)

All communication through a single HTTP endpoint. Client sends JSON-RPC via POST. Server responds with direct JSON or SSE stream. Best for: remote/shared servers, multi-client, cloud deployments, when auth is required.

| Scenario | Transport |
|---|---|
| Local dev tool / IDE plugin / CLI | stdio |
| Team-shared / Cloud / SaaS | Streamable HTTP |

## Tool Annotations

```json
{ "readOnlyHint": true, "destructiveHint": false, "idempotentHint": true, "openWorldHint": false }
```

- Mark read-only tools explicitly so UIs can auto-approve them
- Mark destructive tools so UIs prompt for confirmation
- Mark idempotent tools so clients know retries are safe
- Distinguish open-world (web, email) from closed-world (local DB) tools

## Security

**OAuth 2.1** (HTTP): Servers expose `/.well-known/oauth-protected-resource` metadata. Clients auto-register via DCR. Bearer tokens with scope-based authorization.

**Trust Boundaries**: Host trusts clients. Clients do NOT fully trust servers. Servers do NOT trust clients. Users must approve destructive tool calls.

**DNS Rebinding**: For HTTP transports, validate Host and Origin headers against allowlists.

**stdio**: Natural process isolation, no network surface, inherits user permissions.

## Server Structure Pattern

```
src/
  main.rs         -- transport + server creation
  handler.rs      -- ServerHandler impl, request routing
  tools/          -- tool structs + call_tool impls
  resources/      -- Resource and ResourceTemplate defs
  state.rs        -- shared server state (Arc<RwLock<T>>)
```

### Handler Pattern (rust-mcp-sdk)

Trait-based: `handle_list_tools_request`, `handle_call_tool_request`, `handle_list_resources_request`, `handle_read_resource_request`.

### Tool Registration

```rust
#[mcp_tool(name = "search_code", description = "...")]
struct SearchCode { query: String, limit: Option<i32> }

tool_box!(CodeTools, [SearchCode, GetDefinition, FindReferences]);
```

## Server Instructions & Discovery

Since the client (Claude Code) already implements dynamic tool discovery via ToolSearch, MCP servers should NOT implement their own tool_search meta-tool. Instead, focus on:

1. **Rich `instructions` in InitializeResult**: Structured text with categorized tool catalog
2. **Meaningful tool descriptions**: The `description` field becomes the catalog one-liner
3. **Tool categorization**: Group tools as core vs. specialized in instructions

### Instructions Template

```
{Server Name} -- {one-line purpose}

TOOLS:
  Core:
    {tool_name} -- {one-liner}
  {Category}:
    {tool_name} -- {one-liner}

WORKFLOW:
  {Typical tool call sequence}

TIPS:
  {Usage guidance}
```

### Persistent Tool Memory

For MCP servers with expensive queries:
- **Search result caching**: LRU cache keyed by query hash, configurable TTL
- **Result IDs in responses**: Return a stable ID so the LLM can request cached results
- **Recall mechanism**: A dedicated tool or parameter to retrieve cached results by ID

### Key Design Decisions

1. **Registry at client level, not server level** -- the client already implements discovery
2. **Instructions over meta-tools** -- structured InitializeResult instructions provide discovery without added complexity
3. **Categorize, don't hide** -- categorize tools as "core" vs "advanced" in instructions
4. **Budget results, don't count tokens** -- character budgets on tool results are simpler than token counting

## Composition Patterns

**Multi-Server**: One client per server, host connects to many. Host aggregates all tools into unified list for the LLM.

**Gateway / Proxy**: MCP server proxying downstream servers. Centralizes auth, enables per-user tool filtering, transforms/augments tool calls.

**Shared State**: Tools share state via `Arc<RwLock<T>>`. Common for servers with interconnected tools.

**Tool Chaining**: The LLM naturally chains tools across servers. No explicit composition needed.

## Spec Evolution

| Version | Key Changes |
|---|---|
| 2024-11-05 | Initial: tools, resources, prompts, stdio, SSE |
| 2025-03-26 | Streamable HTTP, deprecate SSE-only |
| 2025-06-18 | Task support, execution hints, batch messages |
| 2025-11-25 | Elicitation, OAuth 2.1, icons, DNS rebinding, Tasks refinement |

## Anti-Patterns

1. **Overly broad tools** -- one tool doing everything
2. **Swallowing errors** -- silent failures instead of structured error responses
3. **Missing annotations** -- not declaring readOnly/destructive hints
4. **Blocking async runtime** -- CPU-heavy work on tokio threads
5. **Unbounded channels** -- memory growth during bulk operations
6. **Ignoring capability negotiation** -- sending features before confirming support
7. **Mapping every API endpoint to a tool** -- group related operations

## References

- [MCP Specification (2025-11-25)](https://modelcontextprotocol.io/specification/2025-11-25)
- [MCP Best Practices](https://modelcontextprotocol.info/docs/best-practices/)
- [MCP Architecture & Design](https://modelcontextprotocol.info/docs/concepts/architecture/)
- [MCP Best Practice Guide](https://mcp-best-practice.github.io/mcp-best-practice/)
- [MCP Spec Updates June 2025 (Auth0)](https://auth0.com/blog/mcp-specs-update-all-about-auth/)
- [Agent Workflows and Tool Design (Glama)](https://glama.ai/blog/2025-08-22-agent-workflows-and-tool-design-for-edge-mcp-servers)
