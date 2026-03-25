# Anthropic/Claude Rust SDK Landscape

Survey of all available Rust crates for interacting with the Anthropic Claude API and the Claude Code CLI. Research conducted March 2026. Focus: tokio async runtime support and streaming responses.

**Official SDK Status:** Anthropic does not publish an official Rust SDK. Official SDKs exist for Python, TypeScript, Java, and Go. All Rust options below are community-maintained. The `anthropic_rust` crate on crates.io falsely lists `github.com/anthropics/anthropic-sdk-rust` as its homepage (returns 404).

## Category 1: Direct Anthropic API Clients

### 1. `anthropic-sdk` (Mixpeal)

**Repository:** https://github.com/Mixpeal/anthropic-sdk
**crates.io:** https://crates.io/crates/anthropic-sdk
**Version:** 0.1.5 | **Downloads:** 62,343 | **Stars:** 35 | **License:** MIT
**Last Updated:** 2024-07-23

Highest total download count of any Anthropic Rust crate. Builder pattern API with SSE-based streaming via async callbacks. Verbose mode for raw API response inspection.

**API Coverage:** Messages (streaming + non-streaming), tool use, system prompts, temperature/top_k/top_p/stop_sequences, beta header support.

**Tokio:** Yes (full features). **Streaming:** Yes (SSE with async callbacks).

**Concern:** Last updated July 2024 — increasingly stale. Missing newer API features (batches, files, extended thinking).

---

### 2. `anthropic-ai-sdk`

**Repository:** https://github.com/katsuhirohonda/anthropic-sdk-rs
**crates.io:** https://crates.io/crates/anthropic-ai-sdk
**Version:** 0.2.27 | **Downloads:** 37,089 | **Stars:** 14 | **License:** MIT
**Last Updated:** 2026-01-11

Broadest API surface of any Rust crate. 320 commits indicates active development.

**API Coverage:** Messages, streaming via `create_message_streaming`, models list/retrieve, message batches (full CRUD), files API (beta), admin API (users, invites, workspaces, API keys), token counting, pagination.

**Tokio:** Yes (full features). **Streaming:** Yes.

---

### 3. `async-anthropic` (bosun-ai)

**Repository:** https://github.com/bosun-ai/async-anthropic
**crates.io:** https://crates.io/crates/async-anthropic
**Version:** 0.6.0 | **Downloads:** 32,301 (9,071 recent) | **Stars:** 10 | **License:** MIT
**Last Updated:** 2025-05-03

Highest recent download velocity. Originally forked from `anthropic-sdk` but rewritten. From bosun-ai, an AI tooling company.

**API Coverage:** Messages API, Models API, tool use, all standard parameters. Non-text messages incomplete.

**Tokio:** Yes (via reqwest/async). **Streaming:** Yes.

**Notable Features:** Automatic backoff retry logic, tracing integration, clean builder API.

---

### 4. `anthropic` (AbdelStark / anthropic-rs)

**Repository:** https://github.com/AbdelStark/anthropic-rs
**crates.io:** https://crates.io/crates/anthropic
**Version:** 0.0.8 | **Downloads:** 20,408 | **Stars:** 69 | **License:** MIT
**Last Updated:** 2024-09-03

Most starred Rust Claude crate by a wide margin. Holds the prime `anthropic` namespace on crates.io. Clean typed builder API with futures-compatible streaming via `StreamExt`.

```rust
let stream = client.messages_stream(request).await?;
pin_mut!(stream);
while let Some(event) = stream.next().await {
    // handle MessageStreamEvent
}
```

**API Coverage:** Messages API only. Tool use supported. Configurable base URL, API version, beta headers, timeouts.

**Tokio:** Yes (required). **Streaming:** Yes, via `messages_stream()` returning `impl Stream<Item = Result<MessageStreamEvent>>`.

**Concern:** Messages-only, last updated September 2024.

---

### 5. `misanthropic`

**Repository:** https://github.com/mdegans/misanthropic
**crates.io:** https://crates.io/crates/misanthropic
**Version:** 0.5.1 | **Downloads:** 15,088 | **Stars:** 6 | **License:** MIT
**Last Updated:** 2024-11-30

Security-focused design. Encrypts API keys in memory (`memsecurity`), sanitizes input/output against injection attacks (`langsan`). Zero-copy optimizations. Bedrock/Vertex support planned.

**API Coverage:** Messages API, streaming with stream filtering (`filter_rate_limit()`, `.text()`), tool use, image support, prompt caching.

**Tokio:** Indirect (via reqwest, no direct tokio dep). **Streaming:** Yes.

---

### 6. `clust`

**Repository:** https://github.com/mochi-neko/clust
**crates.io:** https://crates.io/crates/clust
**Version:** 0.9.0 | **Downloads:** 14,298 | **Stars:** 40 | **License:** Apache 2.0 / MIT
**Last Updated:** 2024-06-30

Companion `clust_macros` crate provides proc macros for tool definitions via `#[clust_tool]` attribute. Fork `langdb_clust` (6,490 downloads) maintained by LangDB.

```rust
#[clust_tool]
fn get_weather(location: String) -> String {
    // tool implementation
}
```

**API Coverage:** Messages API (sync + streaming via `create_a_message_stream()` with `StreamOption::ReturnStream`), tool use.

**Tokio:** Yes. **Streaming:** Yes.

---

### 7. `misanthropy` (cortesi)

**Repository:** https://github.com/cortesi/misanthropy
**crates.io:** https://crates.io/crates/misanthropy
**Version:** 0.0.8 | **Downloads:** 12,129 | **Stars:** 34 | **License:** MIT
**Last Updated:** 2025-06-08

Includes CLI tool (`misan`). Configurable defaults for model and token limits. 89 commits.

**API Coverage:** Messages, streaming via `messages_stream()`, tool use, extended thinking, text + image content.

**Tokio:** Yes. **Streaming:** Yes.

---

### 8. `claudius`

**Repository:** https://github.com/rescrv/claudius
**crates.io:** https://crates.io/crates/claudius
**Version:** 0.19.0 | **Downloads:** 9,076 (2,372 recent) | **Stars:** 13 | **License:** Apache 2.0
**Last Updated:** 2026-03-04

Goes beyond raw API client into full agent framework. Includes built-in tools (filesystem, shell, text editing, web search), state management, budget/token tracking, prompt testing framework, CLI tools. 19 versions released.

**API Coverage:** Messages, streaming via `MessageStreamEvent`, tool use with built-in tool implementations.

**Tokio:** Yes. **Streaming:** Yes.

---

### 9. `anthropic-sdk-rust` (dimichgh)

**Repository:** https://github.com/dimichgh/anthropic-sdk-rust
**crates.io:** https://crates.io/crates/anthropic-sdk-rust
**Version:** 0.1.1 | **Downloads:** 6,033 (3,191 recent) | **Stars:** 8 | **License:** MIT
**Last Updated:** 2025-06-11

Claims full TypeScript SDK parity. Only 3 commits — likely bulk-generated.

**API Coverage:** Messages, streaming with backpressure, tools, vision (base64 + URL), files, batches, models.

**Tokio:** Yes (rt-multi-thread + macros). **Streaming:** Yes.

---

### 10. `anthropic-rs` (roushou/mesh)

**Repository:** https://github.com/roushou/mesh
**crates.io:** https://crates.io/crates/anthropic-rs
**Version:** 0.1.7 | **Downloads:** 10,821 | **Stars:** 2 | **License:** MIT / Apache 2.0
**Last Updated:** 2024-09-07

Part of the "Mesh" multi-provider LLM SDK (also supports OpenAI, Groq, Perplexity, Replicate). Minimal Anthropic-specific features. Useful if you want a single crate covering multiple LLM providers.

**Tokio:** Yes. **Streaming:** Unknown.

---

### 11. `claude-sdk`

**Repository:** https://github.com/mcfearsome/claude-agent-sdk-rust
**crates.io:** https://crates.io/crates/claude-sdk
**Version:** 1.0.0 | **Downloads:** 1,155 | **Stars:** 2 | **License:** MIT
**Last Updated:** 2025-12-11

90 tests. Exponential backoff retry. Interactive REPL. Built for Colony Shell.

**API Coverage:** Full Claude API + AWS Bedrock. Prompt caching, batch processing, extended thinking, vision, documents, token counting, structured outputs.

**Tokio:** Yes. **Streaming:** Yes (SSE).

---

### Minor/Emerging Crates

| Crate | Version | Downloads | Last Updated | Notes |
|-------|---------|-----------|--------------|-------|
| `anthropic-api` | 0.0.5 | 4,719 | 2025-03 | ~85% coverage, admin API, no batches |
| `anthropic-sdk-rs` | 0.1.2 | 1,597 | 2026-02 | Very new, all downloads recent |
| `anthropic-async` | 0.5.1 | 767 | 2026-03 | Prompt caching, minimal docs |
| `turboclaude` | 0.3.0 | 129 | 2026-01 | Claims Python SDK parity, minimal adoption |
| `anthropic_rust` | 0.1.3 | 1,415 | 2025-09 | Falsely claims official repo, wiremock tests |

---

## Category 2: Claude Code CLI Wrappers

These crates wrap the Claude Code binary (`claude`) rather than calling the Anthropic API directly. They give programmatic control over Claude Code sessions via subprocess stdin/stdout JSON/JSONL protocol. This is the same approach the official Python/TypeScript Claude Agent SDKs use.

The ecosystem is extremely fragmented — 20+ published crates doing roughly the same thing, none with more than ~10K downloads.

### Tier 1: Serious Contenders

#### 12. `claude-agent-sdk-rs` (tyrchen)

**Repository:** https://github.com/tyrchen/claude-agent-sdk-rs
**crates.io:** https://crates.io/crates/claude-agent-sdk-rs
**Version:** 0.6.4 | **Downloads:** 10,064 | **Stars:** 61 | **License:** MIT
**Last Updated:** 2026-02-09 (crate), 2026-03-17 (repo)

Most downloaded and most starred dedicated CLI wrapper crate. Claims 100% feature parity with the official Python SDK. 24 examples.

**Features:** Bidirectional streaming, 6 hook types (PreToolUse, PostToolUse, UserPromptSubmit, Stop, SubagentStop, PreCompact), custom tools via `tool!` macro, in-process MCP servers, plugin system, session management (fork_session), permission callbacks, cost/budget control, extended thinking, multimodal input, fallback models.

**Tokio:** Yes. **Streaming:** Yes (bidirectional).

---

#### 13. `cc-sdk` (ZhangHanDong)

**Repository:** https://github.com/ZhangHanDong/claude-code-api-rs
**crates.io:** https://crates.io/crates/cc-sdk
**Version:** 0.7.0 | **Downloads:** 4,278 | **Stars:** 137 | **License:** NOASSERTION
**Last Updated:** 2026-03-18 (crate), 2026-03-24 (repo)

Highest star count of any Rust CLI wrapper repo. Claims 100% parity with Python SDK v0.1.14. Same repo also publishes `claude-code-api` (v0.1.3, 1,578 downloads) — an OpenAI-compatible API gateway for Claude Code CLI — and `agent-teams` (v0.1.0, 26 downloads) — multi-agent orchestration framework.

**Tokio:** Yes. **Streaming:** Yes.

---

#### 14. `claude-codes` (meawoppl)

**Repository:** https://github.com/meawoppl/rust-code-agent-sdks
**crates.io:** https://crates.io/crates/claude-codes
**Version:** 2.1.53 | **Downloads:** 3,586 | **Stars:** 7 | **License:** Apache 2.0
**Last Updated:** 2026-03-20

Low-level typed protocol bindings for Claude Code CLI JSON/JSONL protocol. Also wraps Codex CLI in the same workspace. Three feature flags: `types`, `sync-client`, `async-client`. WASM-compatible types-only mode. Version tracking mirrors CLI versions.

Does NOT provide hooks, MCP, tools, or session management — intentionally minimal, focused on protocol correctness.

**Tokio:** Yes (async-client feature). **Streaming:** Yes (JSONL).

---

#### 15. `anthropic-agent-sdk` (bartolli)

**Repository:** https://github.com/bartolli/anthropic-agent-sdk
**crates.io:** https://crates.io/crates/anthropic-agent-sdk
**Version:** 0.2.75 | **Downloads:** 2,404 | **Stars:** 6 | **License:** MIT
**Last Updated:** 2025-12-23

Hooks system (PreToolUse, PostToolUse, etc.), permissions model, MCP integration, bidirectional streaming.

**Tokio:** Yes. **Streaming:** Yes (bidirectional).

---

#### 16. `claude-code-sdk` (epsilla-cloud)

**Repository:** https://github.com/epsilla-cloud/claude-code-sdk-rust
**crates.io:** https://crates.io/crates/claude-code-sdk
**Version:** 0.0.3 | **Downloads:** 1,903 | **Stars:** 15 | **License:** MIT
**Last Updated:** 2025-06-22

One of the first community Rust CLI wrappers. Async streaming via tokio-stream, tool integration (allowed_tools, permission_mode), safety limits (memory, timeout, buffer), tracing/logging ecosystem. Not updated since June 2025 — likely stale.

**Tokio:** Yes. **Streaming:** Yes.

---

#### 17. `claude-sdk-rs` (bredmond1019)

**Repository:** https://github.com/bredmond1019/claude-sdk-rs
**crates.io:** https://crates.io/crates/claude-sdk-rs
**Version:** 1.0.2 | **Downloads:** 1,817 | **Stars:** 20 | **License:** MIT
**Last Updated:** 2026-03-01

Streaming with backpressure, session management with SQLite persistence, MCP integration (feature flag), tool integration, security validation levels, analytics tracking, token usage/cost metadata.

**Tokio:** Yes (1.40+). **Streaming:** Yes.

---

#### 18. `claude-code-agent-sdk` (soddygo)

**Repository:** https://github.com/soddygo/claude-code-agent-sdk
**crates.io:** https://crates.io/crates/claude-code-agent-sdk
**Version:** 0.1.39 | **Downloads:** 1,607 | **Stars:** 0 | **License:** MIT
**Last Updated:** 2026-02-13

Bidirectional streaming, hooks, custom tools, plugin support. Feature set matches tyrchen's — likely a fork or derivative.

**Tokio:** Yes. **Streaming:** Yes.

---

### Tier 2: Smaller/Specialized

#### 19. `claude-wrapper` (joshrotenberg)

**Repository:** https://github.com/joshrotenberg/claude-wrapper
**crates.io:** https://crates.io/crates/claude-wrapper
**Version:** 0.4.1 | **Downloads:** 713 | **Stars:** 0 | **License:** Apache 2.0
**Last Updated:** 2026-03-21

Not a raw SDK — an orchestration layer. Parallel task execution in git worktrees, JSON/TOML manifest-driven workflows, pre/post/finally hooks, retry with error context, streaming output with per-task coloring, cost tracking, AI-assisted manifest generation.

**Tokio:** Yes. **Streaming:** Yes.

---

#### 20. `clau` (frgmt0)

**Repository:** https://github.com/frgmt0/clau.rs
**crates.io:** https://crates.io/crates/clau (also `clau-core`, `clau-macros`, `clau-mcp`, `clau-runtime`)
**Version:** 0.1.1 | **Downloads:** ~1,086 | **Stars:** 5 | **License:** MIT
**Last Updated:** 2025-05-23

One of the earliest (May 2025). Modular multi-crate workspace design (core, macros, MCP, runtime). Streaming, session management, type-safe MCP config, raw JSON access. Not updated since May 2025.

**Tokio:** Yes. **Streaming:** Yes.

---

#### 21. `cc-agent-sdk` (louloulin)

**Repository:** https://github.com/louloulin/claude-agent-sdk
**crates.io:** https://crates.io/crates/cc-agent-sdk (also `claude-code-sdk-rs` v0.1.0)
**Version:** 0.1.7 | **Downloads:** 146 | **Stars:** 12 | **License:** MIT
**Last Updated:** 2026-03-15

V2 session-based API, 8 hook types, skills validation, security auditor, MCP integration, subagents, orchestrators, streaming, multimodal input, hot reload.

**Tokio:** Yes. **Streaming:** Yes.

---

#### 22. `claude-cli-sdk` (pomdotdev)

**Repository:** https://github.com/pomdotdev/claude-cli-sdk
**crates.io:** https://crates.io/crates/claude-cli-sdk
**Version:** 0.5.1 | **Downloads:** 79 | **Stars:** 0 | **License:** MIT / Apache 2.0
**Last Updated:** 2026-03-04

Wraps CLI via stdin/stdout NDJSON protocol. One-shot and streaming queries, persistent multi-turn Client, per-tool approval callbacks, 8 lifecycle hooks, extended thinking, fallback model, multimodal input, MCP server attachment, cooperative cancellation via CancellationToken, MockTransport for testing. Low downloads but thoughtful design.

**Tokio:** Yes. **Streaming:** Yes.

---

#### 23. `claude-agent-sdk-rust` (Wally869)

**Repository:** https://github.com/Wally869/claude_agent_sdk_rust
**crates.io:** https://crates.io/crates/claude-agent-sdk-rust
**Version:** 1.0.0 | **Downloads:** 78 | **Stars:** 24 | **License:** MIT
**Last Updated:** 2026-03-03 (crate), 2026-03-19 (repo)

Bidirectional streaming, tool/permission control, hook system, MCP tools, session management, extended thinking, budget limits.

**Tokio:** Yes. **Streaming:** Yes.

---

### Tier 3: Minimal/Micro

| Crate | Version | Downloads | Last Updated | Notes |
|-------|---------|-----------|--------------|-------|
| `claude-agent-sdk` | 0.1.1 | 2,439 | 2025-09 | **Name squat.** Lists fake repo `anthropics/claude-agent-sdk-rust` (404). Do not use. |
| `claude-agent-rs` (ExpertVagabond) | 1.0.0 | 142 | 2026-03 | 8 built-in tools, MCP, hooks, skills, sub-agents. Repo deleted/private. |
| `claude-agents-sdk` (jimmystridh) | 0.1.7 | 128 | 2026-02 | MIT. Minimal info. |
| `apiari-claude-sdk` | 0.1.0 | 79 | 2026-03 | No repo listed. "Spawn, stream, and manage Claude agent sessions." |
| `claude-code-client-sdk` (tosimpletech) | 0.1.46 | 46 | 2026-03 | Typed subprocess API. Apache 2.0. |
| `claude-code-rs` (decisiongraph) | 0.1.1 | 32 | 2026-02 | Bidirectional JSON streaming, optional WebSocket transport, hooks, in-process MCP. |
| `clauders` (xorpse) | 0.1.1 | 28 | 2026-03 | Minimal info. |
| `claude-code-proxy` (i-am-logger) | 0.4.0 | 25 | 2026-03 | OpenAI-compatible API proxy for Claude Code CLI. Not an SDK. |
| `chimera-claude` (ooojustin) | 0.1.0 | 11 | 2026-03 | Claude backend for chimera unified AI agent SDK (also Codex, OpenCode). |
| `agentisc-relay-adapter-claude` | 0.2.1 | 10 | 2026-03 | Claude Code adapter for agentisc-relay system. |

### GitHub-Only (No crates.io Publish)

| Repo | Stars | License | Last Updated | Notes |
|------|-------|---------|--------------|-------|
| `dhuseby/claude-agent-sdk-rust` | 8 | MIT | 2026-01 | "Claude-written re-implementation of claude-agent-sdk-python in Rust" |
| `kcodes0/clau.rs` | 5 | MIT | 2026-03 | Type-safe SDK, different from frgmt0/clau.rs despite similar name |

---

## Comparison Matrix

### API Clients: Tokio + Streaming Focus

| Crate | Tokio | Streaming | Tools | Extended Thinking | Batches | Files | Admin | Active (2025+) | Downloads |
|-------|-------|-----------|-------|-------------------|---------|-------|-------|----------------|-----------|
| `anthropic-sdk` | Yes | Yes (SSE) | Yes | No | No | No | No | No (2024-07) | 62,343 |
| `anthropic-ai-sdk` | Yes | Yes | Yes | No | Yes | Yes | Yes | Yes (2026-01) | 37,089 |
| `async-anthropic` | Yes | Yes | Yes | No | No | No | No | Yes (2025-05) | 32,301 |
| `anthropic` | Yes | Yes (Stream) | Yes | No | No | No | No | No (2024-09) | 20,408 |
| `misanthropic` | Indirect | Yes | Yes | No | No | No | No | No (2024-11) | 15,088 |
| `clust` | Yes | Yes | Yes (macros) | No | No | No | No | No (2024-06) | 14,298 |
| `misanthropy` | Yes | Yes | Yes | Yes | No | No | No | Yes (2025-06) | 12,129 |
| `claudius` | Yes | Yes | Yes (built-in) | No | No | No | No | Yes (2026-03) | 9,076 |
| `claude-sdk` | Yes | Yes (SSE) | Yes | Yes | Yes | No | No | Yes (2025-12) | 1,155 |

### CLI Wrappers: Ranked by Adoption

| Crate | Downloads | Stars | Last Updated | Hooks | MCP | Tools | Sessions | Differentiator |
|-------|-----------|-------|--------------|-------|-----|-------|----------|----------------|
| `claude-agent-sdk-rs` (tyrchen) | 10,064 | 61 | 2026-03 | 6 types | Yes | `tool!` macro | Yes (fork) | Most complete, 24 examples |
| `cc-sdk` (ZhangHanDong) | 4,278 | 137 | 2026-03 | Yes | Yes | Yes | Yes | Also ships API gateway + agent-teams |
| `claude-codes` (meawoppl) | 3,586 | 7 | 2026-03 | No | No | No | No | Low-level typed protocol, WASM-compat |
| `anthropic-agent-sdk` (bartolli) | 2,404 | 6 | 2025-12 | Yes | Yes | Yes | Yes | — |
| `claude-code-sdk` (epsilla) | 1,903 | 15 | 2025-06 | No | No | Yes | No | Safety limits, tracing. Stale. |
| `claude-sdk-rs` (bredmond1019) | 1,817 | 20 | 2026-03 | No | Yes | Yes | SQLite | Session persistence, analytics |
| `claude-code-agent-sdk` (soddygo) | 1,607 | 0 | 2026-02 | Yes | No | Yes | Yes | Likely tyrchen fork |
| `clau` (frgmt0) | 1,086 | 5 | 2025-05 | No | Yes | No | Yes | Multi-crate workspace. Stale. |
| `claude-wrapper` (joshrotenberg) | 713 | 0 | 2026-03 | Yes | Deprecated | No | No | Manifest-driven worktree orchestration |
| `cc-agent-sdk` (louloulin) | 146 | 12 | 2026-03 | 8 types | Yes | Yes | V2 | Skills, security auditor, hot reload |
| `claude-cli-sdk` (pomdotdev) | 79 | 0 | 2026-03 | 8 types | Yes | No | Yes | MockTransport, CancellationToken |
| `claude-agent-sdk-rust` (Wally869) | 78 | 24 | 2026-03 | Yes | Yes | Yes | Yes | — |

## Relevance to lx

lx needs programmatic Claude access for its AI backend. Two distinct integration paths exist:

### Path A: Direct API Client

Call the Anthropic Messages API directly. Full control over request construction, token-level streaming, tool schemas.

**Requirements:** Tokio-native async, streaming, tool use protocol, maintenance trajectory.

**Top candidates:**

- **`anthropic-ai-sdk`** — Broadest API surface (messages, batches, files, admin), actively maintained (Jan 2026), tokio-native. Best fit if lx needs the full API.
- **`async-anthropic`** — Highest recent velocity, retry logic, tracing. Clean builder API from bosun-ai. Best fit for a focused, reliable Messages API client.
- **`claudius`** — Most recently updated (March 2026), includes agent framework abstractions worth studying.
- **`misanthropy`** — Extended thinking support, actively maintained. Good if lx needs thinking token access.

### Path B: Claude Code CLI Wrapper

Spawn `claude` as a subprocess and communicate via JSON/JSONL. Gets the full Claude Code agent loop (TAOR, tools, memory, sub-agents) for free. The official approach — Python/TypeScript SDKs work this way.

**Top candidates:**

- **`claude-agent-sdk-rs` (tyrchen)** — Clear leader. Most downloads (10K), most complete feature set, `tool!` macro, 24 examples, claims Python SDK parity.
- **`cc-sdk` (ZhangHanDong)** — Most starred repo (137), most recently updated, also provides an OpenAI-compatible API gateway and multi-agent orchestration.
- **`claude-codes` (meawoppl)** — If lx wants low-level typed protocol bindings without SDK opinions. WASM-compatible. Good for building a custom integration layer on top.
- **`claude-cli-sdk` (pomdotdev)** — Best testing story (MockTransport), cooperative cancellation. Low adoption but clean design.

### Ecosystem Assessment

The CLI wrapper space is extremely fragmented: 20+ crates, none exceeding 10K downloads, many clearly AI-generated bulk commits. The top 3 (`tyrchen`, `ZhangHanDong`, `meawoppl`) are meaningfully differentiated; the rest are largely redundant. Expect consolidation — most of these crates will be abandoned within 6 months.

## Sources

- [claude-agent-sdk-rs on crates.io](https://crates.io/crates/claude-agent-sdk-rs)
- [cc-sdk on crates.io](https://crates.io/crates/cc-sdk)
- [claude-codes on crates.io](https://crates.io/crates/claude-codes)
- [claude-code-sdk on crates.io](https://crates.io/crates/claude-code-sdk)
- [claude-sdk-rs on crates.io](https://crates.io/crates/claude-sdk-rs)
- [claude-cli-sdk on crates.io](https://crates.io/crates/claude-cli-sdk)
- [claude-wrapper on crates.io](https://crates.io/crates/claude-wrapper)
- [cc-agent-sdk on crates.io](https://crates.io/crates/cc-agent-sdk)
- [claude-code-agent-sdk on crates.io](https://crates.io/crates/claude-code-agent-sdk)
- [claude-agent-sdk-rust on crates.io](https://crates.io/crates/claude-agent-sdk-rust)
- [clau on crates.io](https://crates.io/crates/clau)
- [anthropic-agent-sdk on crates.io](https://crates.io/crates/anthropic-agent-sdk)
- [anthropic-sdk on crates.io](https://crates.io/crates/anthropic-sdk)
- [anthropic-ai-sdk on crates.io](https://crates.io/crates/anthropic-ai-sdk)
- [async-anthropic on crates.io](https://crates.io/crates/async-anthropic)
- [anthropic on crates.io](https://crates.io/crates/anthropic)
- [misanthropic on crates.io](https://crates.io/crates/misanthropic)
- [clust on crates.io](https://crates.io/crates/clust)
- [misanthropy on crates.io](https://crates.io/crates/misanthropy)
- [claudius on crates.io](https://crates.io/crates/claudius)
- [anthropic-sdk-rust on crates.io](https://crates.io/crates/anthropic-sdk-rust)
- [anthropic-rs on crates.io](https://crates.io/crates/anthropic-rs)
- [claude-sdk on crates.io](https://crates.io/crates/claude-sdk)
- [tyrchen/claude-agent-sdk-rs on GitHub](https://github.com/tyrchen/claude-agent-sdk-rs)
- [ZhangHanDong/claude-code-api-rs on GitHub](https://github.com/ZhangHanDong/claude-code-api-rs)
- [meawoppl/rust-code-agent-sdks on GitHub](https://github.com/meawoppl/rust-code-agent-sdks)
- [bartolli/anthropic-agent-sdk on GitHub](https://github.com/bartolli/anthropic-agent-sdk)
- [epsilla-cloud/claude-code-sdk-rust on GitHub](https://github.com/epsilla-cloud/claude-code-sdk-rust)
- [bredmond1019/claude-sdk-rs on GitHub](https://github.com/bredmond1019/claude-sdk-rs)
- [pomdotdev/claude-cli-sdk on GitHub](https://github.com/pomdotdev/claude-cli-sdk)
- [louloulin/claude-agent-sdk on GitHub](https://github.com/louloulin/claude-agent-sdk)
- [Wally869/claude_agent_sdk_rust on GitHub](https://github.com/Wally869/claude_agent_sdk_rust)
- [frgmt0/clau.rs on GitHub](https://github.com/frgmt0/clau.rs)
- [joshrotenberg/claude-wrapper on GitHub](https://github.com/joshrotenberg/claude-wrapper)
- [AbdelStark/anthropic-rs on GitHub](https://github.com/AbdelStark/anthropic-rs)
- [bosun-ai/async-anthropic on GitHub](https://github.com/bosun-ai/async-anthropic)
- [cortesi/misanthropy on GitHub](https://github.com/cortesi/misanthropy)
- [mdegans/misanthropic on GitHub](https://github.com/mdegans/misanthropic)
- [mochi-neko/clust on GitHub](https://github.com/mochi-neko/clust)
- [rescrv/claudius on GitHub](https://github.com/rescrv/claudius)
- [Mixpeal/anthropic-sdk on GitHub](https://github.com/Mixpeal/anthropic-sdk)
- [dhuseby/claude-agent-sdk-rust on GitHub](https://github.com/dhuseby/claude-agent-sdk-rust)
