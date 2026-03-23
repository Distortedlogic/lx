# OpenViking: Filesystem-Paradigm Context Database for AI Agents

OpenViking replaces flat vector stores with a **hierarchical virtual filesystem** for managing agent context (memories, resources, skills), delivering 3-tier on-demand loading that cuts token consumption while making retrieval paths fully observable. Built by ByteDance/Volcengine, it represents a significant architectural departure from traditional RAG by treating context management as a filesystem problem rather than a search problem.

## Repository Metrics

| Metric | Value |
|--------|-------|
| Stars | ~8,200 |
| Forks | ~570 |
| License | Apache 2.0 |
| Primary Language | Python (with C++/Rust/Go components) |
| Created | January 2026 |
| Last Updated | March 2026 |
| Open Issues | 57 |
| Repo Size | ~53 MB |
| Topics | agent, context-database, context-engineering, filesystem, memory, rag, skill, openclaw |

## Core Problem

The project identifies five pain points in agent context management:

1. **Fragmented Context** -- memories, resources, and skills scattered across different storage backends with no unified addressing
2. **Surging Context Demand** -- long-running agent tasks generate continuous context that simple truncation destroys
3. **Poor Retrieval Effectiveness** -- flat vector RAG lacks hierarchical structure, producing noisy results
4. **Unobservable Retrieval** -- traditional RAG operates as a black box with no way to debug what was retrieved or why
5. **Limited Memory Iteration** -- existing memory systems only record user interactions, ignoring agent-specific task memory and learned patterns

## Architecture

### The Filesystem Paradigm

OpenViking's central design decision is modeling all context as a **virtual filesystem** accessed via `viking://` URIs. Every piece of context -- whether a user preference, a conversation summary, or a tool usage pattern -- is a "file" in a directory tree.

The URI scheme maps to four root scopes:

| Scope | Path Pattern | Purpose |
|-------|-------------|---------|
| Session | `viking://session/{user_space}/{session_id}` | Conversation context, compressed summaries |
| User | `viking://user/{user_space}/memories/...` | Preferences, entities, events |
| Agent | `viking://agent/{agent_space}/memories/...` | Cases, patterns, tool/skill experience |
| Resources | `viking://resources/...` | Shared knowledge, not bound to user or agent |

User space is derived as `md5(user_id)[:8]` and agent space as `md5(user_id+agent_id)[:12]`, ensuring deterministic path mapping.

### Three-Tier Context Loading (L0/L1/L2)

Every directory node carries three levels of detail:

| Level | File | Content | Token Cost |
|-------|------|---------|------------|
| L0 | `.abstract.md` | One-line summary | Minimal |
| L1 | `.overview.md` | Structured overview | Medium |
| L2 | Full content | Complete detail | Full |

The system loads L0 summaries first, drilling into L1/L2 only when retrieval scores justify it. This tiered approach is the primary mechanism for reducing token consumption -- agents see directory structures at L0 cost and only pay full token cost for genuinely relevant content.

### Storage Layer

**VikingFS** wraps an underlying **AGFS** (Abstract Filesystem) server with a Viking URI abstraction layer on top. The storage stack:

- **AGFS Server** (Go 1.22+) -- the actual filesystem backend serving file operations over HTTP
- **VikingFS** (Python singleton) -- URI routing, access control, vector sync, relation management
- **VikingVectorIndexBackend** -- vector database adapter for semantic search across the filesystem
- **C++ extensions** (pybind11) -- performance-critical indexing operations

VikingFS implements all standard filesystem operations (`read`, `write`, `mkdir`, `rm`, `mv`, `ls`, `tree`, `glob`, `grep`, `stat`) plus semantic operations (`find`, `search`, `abstract`, `overview`, `relations`).

### Retrieval Engine

The **HierarchicalRetriever** implements directory-recursive retrieval:

1. **Intent Analysis** -- `IntentAnalyzer` uses an LLM to generate a `QueryPlan` with typed queries (memory/resource/skill) and priorities, incorporating session compression summaries and recent messages
2. **Global Vector Search** -- locates starting directories via embedding similarity
3. **Recursive BFS Traversal** -- breadth-first search through directory tree with score propagation (alpha=0.5) and convergence detection (max 3 rounds)
4. **Hotness Scoring** -- blends semantic similarity with access frequency via `sigmoid(log1p(active_count)) * time_decay(updated_at)` using a 7-day half-life
5. **Result Conversion** -- matched contexts returned with full URI paths and retrieval trajectories

The retriever uses `MAX_CONVERGENCE_ROUNDS = 3`, `MAX_RELATIONS = 5`, and `HOTNESS_ALPHA = 0.2` as default tuning parameters.

### Session Management

The **Session** class manages conversation lifecycles with:

- **Message tracking** with role-based token counting
- **Archive operations** creating historical snapshots with L0/L1 summaries
- **Copy-on-Write (COW) commits** -- changes stage to temporary URIs, then atomically switch via directory rename
- **Asynchronous memory extraction** -- enqueues semantic processing tasks for background L0/L1 generation

The **SessionCompressor** orchestrates long-term memory extraction using a 6-category taxonomy:

| Category | Scope | Examples |
|----------|-------|---------|
| Profile | User | Demographics, communication style |
| Preferences | User | Topic-specific likes/dislikes |
| Entities | User | Projects, people, concepts |
| Events | User | Historical records |
| Cases | Agent | Task execution patterns |
| Patterns | Agent | Recurring problem-solution pairs |

Two additional categories exist for tool and skill memories, tracking execution statistics (duration, tokens, call counts, success rates, common failures, optimal parameters).

### Memory Deduplication

**MemoryDeduplicator** implements a two-stage process:

1. **Vector pre-filtering** -- find similar existing memories by embedding similarity
2. **LLM decision-making** -- for candidates with matches, an LLM decides: SKIP (duplicate), CREATE (new), or NONE (merge/delete existing)

Decision normalization rules enforce logical consistency -- skip decisions never carry per-memory actions, create decisions with merge actions normalize to NONE.

### Multi-Tenant Design

A three-layer identity model:

| Role | Capabilities |
|------|-------------|
| ROOT | Create/delete workspaces, assign admins, cross-tenant access |
| ADMIN | Manage workspace users, issue API keys, access all workspace data |
| USER | Access own isolated space, share account-level resources |

API keys are pure random tokens (`secrets.token_hex(32)`) with a two-tier lookup: root key first, then user key index. Storage isolation operates on three dimensions: account (workspace), user (private space), and agent (user x agent combination).

## Technology Stack

| Component | Technology | Purpose |
|-----------|-----------|---------|
| Core Server | Python 3.10+ / FastAPI / Uvicorn | HTTP API, business logic |
| Filesystem Backend | Go 1.22+ (AGFS) | File storage and operations |
| Performance Extensions | C++ (pybind11) | Indexing, vector operations |
| CLI | Rust (ov_cli) | Command-line interface |
| Vector Search | VikingDB (adapter-based) | Semantic similarity |
| Document Parsing | Python (tree-sitter for 7 languages) | Code and document ingestion |
| LLM Integration | OpenAI, Volcengine (Doubao), LiteLLM | Intent analysis, memory extraction |
| Bot Framework | Vikingbot (OpenClaw-compatible) | Chat platform integrations |

### Dependencies (40+)

Heavyweight dependency tree including FastAPI, pydantic, httpx, tree-sitter (Python/JS/TS/Java/C++/Rust/Go/C#), openai, litellm, pdfplumber, python-docx/pptx/openpyxl, ebooklib, markdownify, apscheduler, and Volcengine SDKs. Optional bot integrations for Telegram, Feishu, DingTalk, Slack, and QQ.

## Key Design Decisions

**Filesystem over flat vector store** -- by imposing hierarchical structure, retrieval becomes navigable and debuggable. Agents can `ls` a directory to understand what context exists before retrieving it, rather than hoping a vector search returns something useful.

**Three-tier loading** -- the L0/L1/L2 approach is the most impactful design choice. It means an agent can "browse" thousands of context items at L0 cost (one-line summaries) and only pay full token cost for the handful it actually needs.

**COW atomic commits** -- session state changes write to temporary URIs first, then atomically swap. This prevents partial updates from corrupting the context tree during concurrent access.

**LLM-in-the-loop for memory management** -- both intent analysis and memory deduplication rely on LLM calls rather than heuristics. This increases accuracy but adds latency and cost to every session commit.

**Hotness decay scoring** -- the `sigmoid * exponential_decay` formula naturally demotes stale memories while boosting frequently-accessed ones, without requiring manual curation.

**MCP integration** -- the `mcp_converter.py` module converts MCP tool definitions to OpenViking's SKILL.md format with YAML frontmatter, bridging between tool protocols and context management.

## Strengths

- **Observable retrieval** -- full directory trajectory preserved, making it possible to debug why specific context was or wasn't retrieved
- **Token efficiency** -- L0/L1/L2 tiering dramatically reduces context window consumption compared to flat retrieval
- **Self-evolving memory** -- agents accumulate tool usage patterns, success rates, and optimal parameters across sessions
- **Unified addressing** -- `viking://` URIs provide a single namespace for all context types, eliminating the "which store has this?" problem
- **Multi-tenant from the start** -- account/user/agent isolation built into the path scheme, not bolted on
- **Rich document ingestion** -- parsers for PDF, DOCX, PPTX, XLSX, EPUB, HTML, Markdown, code repos (7 languages via tree-sitter)

## Weaknesses

- **Heavy dependency footprint** -- 40+ Python packages, plus Go/C++/Rust components; deployment complexity is high
- **LLM dependency for core operations** -- intent analysis and memory deduplication require LLM calls, adding latency and cost to every retrieval and session commit
- **Volcengine ecosystem coupling** -- VikingDB as the vector backend, Doubao as a first-class LLM provider; the abstractions exist for portability but the defaults favor the Volcengine stack
- **Early maturity** -- created January 2026, still alpha status per pyproject.toml; 57 open issues
- **Complex operational model** -- requires AGFS server (Go), C++ extensions, Python server, and vector DB; significantly more operational overhead than a simple vector store
- **Single-server architecture** -- the VikingFS singleton pattern and per-account file storage suggest scaling limitations for high-throughput multi-tenant scenarios

## Relation to Agentic AI Patterns

### Context Engineering

OpenViking is a pure **context engineering** system. Rather than improving the model or prompt, it focuses entirely on what context reaches the model's context window and how efficiently. The L0/L1/L2 tiering is a direct answer to the "context window is expensive and finite" constraint.

### Agent Harnesses

The project integrates with agent harnesses through two patterns:
- **Vikingbot** (OpenClaw-compatible) serves as a reference harness with 7 dedicated tools for resource/memory management
- **MCP bridge** converts MCP tool definitions into OpenViking skills, allowing any MCP-compatible harness to leverage the context database

The OpenClaw integration design reveals the intended usage pattern: compact-triggered batch uploads (not per-message), per-turn retrieval using concatenated recent messages as queries, and retrieved memories presented as simulated function call results.

### Tool Orchestration

OpenViking tracks tool usage at a granular level -- execution duration, token consumption, call counts, success rates, common failures, optimal parameters, and recommended workflows. This data feeds back into skill memories that enrich future tool invocations, creating a closed loop between tool use and tool knowledge.

### Multi-Agent Coordination

The multi-tenant design with account/user/agent space isolation enables multiple agents to share a resource pool while maintaining private memory spaces. The `agent_space = md5(user_id+agent_id)[:12]` scheme means each user-agent pair gets isolated learned context, while `viking://resources/...` provides a shared knowledge layer.

### Memory Evolution

The six-category memory taxonomy (profile, preferences, entities, events, cases, patterns) plus tool/skill memories creates a structured knowledge base that grows with each session. The deduplication system prevents unbounded growth, and hotness scoring naturally surfaces relevant memories while decaying stale ones.

## Practical Takeaways

1. **Hierarchical context beats flat retrieval** -- imposing directory structure on agent context enables browsing, debugging, and tiered loading that flat vector stores cannot provide
2. **Three-tier loading is the highest-leverage optimization** -- L0 summaries let agents scan broad context cheaply before committing tokens to full retrieval
3. **Agent memory should be structured, not append-only** -- categorizing memories (preferences vs entities vs patterns) with deduplication and decay scoring produces a knowledge base that improves over time rather than accumulating noise
4. **COW commits prevent context corruption** -- atomic directory swaps via temporary URIs are essential when multiple processes (session management, memory extraction, background indexing) write to the same context tree
5. **Observable retrieval is a debugging necessity** -- when agents make wrong decisions, the first question is "what context did it see?" Full retrieval trajectories answer this directly
6. **Tool memory creates compounding returns** -- tracking tool success rates, optimal parameters, and failure patterns across sessions means agents get better at using tools without explicit training

## Project Structure

```
openviking/
  core/           context.py, directories.py, building_tree.py, mcp_converter.py, skill_loader.py
  retrieve/       hierarchical_retriever.py, intent_analyzer.py, memory_lifecycle.py
  session/        session.py, compressor.py, memory_extractor.py, memory_deduplicator.py, tool_skill_utils.py
  storage/        viking_fs.py, viking_vector_index_backend.py, vikingdb_manager.py, queuefs.py, observers.py
  server/         app.py, bootstrap.py (FastAPI with 12+ routers)
  service/        OpenVikingService, FSService, SearchService, SessionService, ResourceService, etc.
  parse/          PDF, Markdown, HTML, code repo, plain text parsers + VLM processor
  pyagfs/         AGFS HTTP client + optional native binding client
  models/         Data models
  prompts/        LLM prompt templates
  eval/           Evaluation framework
  client/         Python client libraries
  console/        Web console
  message/        Message handling
  utils/          Utilities
bot/              Vikingbot (OpenClaw-compatible agent framework)
crates/ov_cli/    Rust CLI
src/              C++ core (pybind11 bindings, indexing, store)
docs/design/      Multi-tenant design, OpenClaw integration specs
```

## Sources

- [GitHub Repository](https://github.com/volcengine/OpenViking)
- [README (English)](https://github.com/volcengine/OpenViking/blob/main/README.md)
- [Multi-Tenant Design Document](https://github.com/volcengine/OpenViking/blob/main/docs/design/multi-tenant-design.md)
- [OpenClaw Integration Design](https://github.com/volcengine/OpenViking/blob/main/docs/design/openclaw-integration.md)
- [Bot Documentation](https://github.com/volcengine/OpenViking/blob/main/bot/README.md)
- [PyPI Project Configuration](https://github.com/volcengine/OpenViking/blob/main/pyproject.toml)
