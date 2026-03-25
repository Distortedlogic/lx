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

These crates wrap the Claude Code binary rather than calling the Anthropic API directly. They give programmatic control over a Claude Code session.

### 12. `cc-sdk`

**Repository:** https://github.com/ZhangHanDong/claude-code-api-rs
**crates.io:** https://crates.io/crates/cc-sdk
**Version:** 0.7.0 | **Downloads:** 4,272 (1,447 recent)
**Last Updated:** 2026-03-18

Claims 100% parity with the Python Claude Agent SDK v0.1.14. Wraps the `claude` CLI binary. Most recently updated of the CLI wrappers.

---

### 13. `anthropic-agent-sdk`

**Repository:** https://github.com/bartolli/anthropic-agent-sdk
**crates.io:** https://crates.io/crates/anthropic-agent-sdk
**Version:** 0.2.75 | **Downloads:** 2,393 (2,389 recent)
**Last Updated:** 2025-12-23

Claude Code CLI wrapper with hooks system (PreToolUse, PostToolUse, etc.), permissions model, MCP integration, bidirectional streaming.

**Tokio:** Yes. **Streaming:** Yes (bidirectional).

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

### CLI Wrappers

| Crate | Approach | Parity Target | Last Updated | Downloads |
|-------|----------|---------------|--------------|-----------|
| `cc-sdk` | CLI binary wrapper | Python SDK v0.1.14 | 2026-03 | 4,272 |
| `anthropic-agent-sdk` | CLI binary wrapper | N/A | 2025-12 | 2,393 |

## Relevance to lx

lx needs programmatic Claude API access for its AI backend. The requirements:

1. **Tokio-native async** — lx's runtime is tokio-based; the SDK must compose with `tokio::spawn`, `select!`, and the broader async ecosystem without blocking.
2. **Streaming** — lx's `ai.prompt` and `ai.prompt_structured` need token-by-token streaming for real-time output and early termination in refine loops.
3. **Tool use** — lx agents invoke tools; the SDK must support Claude's tool use protocol for the `AiBackend` trait.
4. **Maintenance trajectory** — lx cannot depend on abandoned crates.

**Top candidates for lx:**

- **`anthropic-ai-sdk`** — Broadest API surface (messages, batches, files, admin), actively maintained (Jan 2026), tokio-native. Best fit if lx needs the full API.
- **`async-anthropic`** — Highest recent velocity, retry logic, tracing. Clean builder API from an AI tooling company (bosun-ai). Best fit for a focused, reliable Messages API client.
- **`claudius`** — Most recently updated (March 2026), includes agent framework abstractions. If lx wants to study or borrow agent patterns, this is the reference implementation.
- **`misanthropy`** — Extended thinking support, actively maintained. Good if lx needs thinking token access.

**CLI wrappers (`cc-sdk`, `anthropic-agent-sdk`)** are relevant if lx wants to orchestrate Claude Code sessions as subprocesses rather than calling the API directly. This is the pattern used by the official Python/TypeScript Claude Agent SDKs.

## Sources

- [anthropic-sdk on crates.io](https://crates.io/crates/anthropic-sdk)
- [anthropic-ai-sdk on crates.io](https://crates.io/crates/anthropic-ai-sdk)
- [async-anthropic on crates.io](https://crates.io/crates/async-anthropic)
- [anthropic on crates.io](https://crates.io/crates/anthropic)
- [misanthropic on crates.io](https://crates.io/crates/misanthropic)
- [clust on crates.io](https://crates.io/crates/clust)
- [misanthropy on crates.io](https://crates.io/crates/misanthropy)
- [claudius on crates.io](https://crates.io/crates/claudius)
- [anthropic-sdk-rust on crates.io](https://crates.io/crates/anthropic-sdk-rust)
- [anthropic-agent-sdk on crates.io](https://crates.io/crates/anthropic-agent-sdk)
- [cc-sdk on crates.io](https://crates.io/crates/cc-sdk)
- [claude-sdk on crates.io](https://crates.io/crates/claude-sdk)
- [anthropic-rs on crates.io](https://crates.io/crates/anthropic-rs)
- [AbdelStark/anthropic-rs on GitHub](https://github.com/AbdelStark/anthropic-rs)
- [bosun-ai/async-anthropic on GitHub](https://github.com/bosun-ai/async-anthropic)
- [cortesi/misanthropy on GitHub](https://github.com/cortesi/misanthropy)
- [mdegans/misanthropic on GitHub](https://github.com/mdegans/misanthropic)
- [mochi-neko/clust on GitHub](https://github.com/mochi-neko/clust)
- [rescrv/claudius on GitHub](https://github.com/rescrv/claudius)
- [Mixpeal/anthropic-sdk on GitHub](https://github.com/Mixpeal/anthropic-sdk)
- [ZhangHanDong/claude-code-api-rs on GitHub](https://github.com/ZhangHanDong/claude-code-api-rs)
- [bartolli/anthropic-agent-sdk on GitHub](https://github.com/bartolli/anthropic-agent-sdk)
