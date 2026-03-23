# Augment Code Context Engine

## Overview

Augment Code's Context Engine is a semantic code understanding system that maintains a live index of entire codebases (up to 500,000 files). It goes beyond text search -- it understands meaning, relationships, and architectural patterns.

- **Product page**: [augmentcode.com/context-engine](https://www.augmentcode.com/context-engine)
- **MCP docs**: [docs.augmentcode.com/context-services/mcp/overview](https://docs.augmentcode.com/context-services/mcp/overview)
- **Blog**: [Context Engine MCP now live](https://www.augmentcode.com/blog/context-engine-mcp-now-live)

## Architecture

### What It Indexes

- Source code across all files
- Commit history
- Codebase patterns and conventions
- External sources (docs, tickets)
- "Tribal knowledge" (team patterns)

### Key Capabilities

1. **Relationship awareness**: understands how files connect across repos, services, architectures
2. **Smart context curation**: retrieves only what matters, compresses context without losing information
3. **200K-token context window** with AST-controlled context assembly
4. **Real-time indexing**: updates as you edit, no manual sync

### MCP Server

Available as an MCP server that exposes semantic search tools:
- Runs locally via Auggie CLI (stdio transport)
- Or connects to Augment-hosted service (HTTP transport)
- Compatible with Claude Code, Cursor, Zed, and any MCP client

### Performance Claims

- Adding Context Engine improved agent performance by **70%+** across Claude Code, Cursor, and Codex
- MCP-enabled runs required **fewer tool calls and conversation turns**

## Layered Architecture (from community implementations)

| Layer | Responsibility |
|---|---|
| Layer 3: MCP Interface | Exposes tools, validates I/O |
| Layer 4: Agents | Consume context, generate responses |
| Layer 5: Storage | Persists embeddings and metadata |

## What We Can Learn

1. **Semantic search > text search** for code understanding
2. **Real-time indexing** (incremental updates as files change) is essential for dev tool UX
3. **Relationship-aware retrieval** (not just individual chunks but how code connects) is a differentiator
4. **Context compression** -- retrieving relevant code and compressing it to fit context windows
5. **AST-controlled context assembly** -- using AST structure to select and compose context for the LLM
6. **Multi-repo support** -- enterprise codebases span many repos

## Comparison to Our Approach

Augment is a hosted service with significant infrastructure. Our MCP server targets:
- Local-first (no cloud dependency)
- Open-source
- Focused on workspace-level code (not enterprise-scale multi-repo)
- AST-based chunking as the primary splitting mechanism
- MCP protocol for tool exposure

## References

- [Context Engine product page](https://www.augmentcode.com/context-engine)
- [Context Engine MCP docs](https://docs.augmentcode.com/context-services/mcp/overview)
- [Context Engine MCP product page](https://www.augmentcode.com/product/context-engine-mcp)
- [Context Engine MCP blog](https://www.augmentcode.com/blog/context-engine-mcp-now-live)
- [Augment Code: Context Is the New Compiler (WorkOS)](https://workos.com/blog/augment-code-context-is-the-new-compiler)
- [Augment Code overview (Medium)](https://medium.com/@pritisolanki/augment-code-a-real-time-index-for-your-codebase-833c7591c808)
- [7 AI Agent Tactics for RAG-Driven Codebases](https://www.augmentcode.com/guides/7-ai-agent-tactics-for-multimodal-rag-driven-codebases)
- [Why Context Beats Prompting webinar](https://watch.getcontrast.io/register/why-context-beats-prompting-a-deep-dive-into-augment-code-s-context-engine)
