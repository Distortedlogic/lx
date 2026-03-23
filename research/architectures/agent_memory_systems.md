# Agent Memory and Context Management Systems (Early 2026)

This document surveys the state of the art in memory systems, context management, and persistent state for AI agents as of Q1 2026. The field has matured rapidly, with over $55M in venture funding flowing into dedicated memory-layer products and an ICLR 2026 workshop (MemAgents) devoted entirely to the topic.

## 1. Memory Taxonomy for Agents

Modern agent memory systems draw on a cognitive-science-inspired taxonomy with four primary memory types:

### Short-Term / Working Memory

The LLM's context window itself functions as working memory. It holds the current conversation, system instructions, and any injected context. It is bounded by the model's token limit and resets between sessions unless explicitly persisted. The fundamental challenge of agent memory is that this working memory is finite and ephemeral.

### Episodic Memory

Captures specific past experiences with temporal details -- what happened, when, and in what sequence. Stored using vector databases for semantic search and event logs for ground truth. Episodic memory enables agents to recall and reason about prior interactions, refer to past decisions, and learn from outcomes. The Zep/Graphiti system structures episodic memory as a subgraph of timestamped episode nodes linked to semantic entities.

### Semantic Memory

Stores factual knowledge independent of specific experiences: user profiles, product specs, domain expertise, extracted preferences. Typically backed by structured databases for discrete facts and vector databases for concept embeddings. Semantic memory is what allows an agent to "know" things about users and domains without re-deriving them each session.

### Procedural Memory

Encodes learned behaviors and strategies -- how the agent should act in specific situations. This is the least mature memory type in current systems and is mostly represented through system prompts, fine-tuned weights, or learned tool-use patterns rather than explicit external storage.

## 2. Long-Term Memory Architectures

### Vector Store Approaches

Vector databases (Qdrant, Pinecone, ChromaDB, Weaviate, PGVector) remain the backbone of most long-term memory systems. Memories are encoded as dense embeddings via transformer models, enabling semantic similarity search at retrieval time. This approach excels at finding contextually relevant information but struggles with relational reasoning -- knowing that entity A relates to entity B requires traversing connections, not just computing cosine similarity.

### Knowledge Graph Approaches

Knowledge graphs address the relational gap by storing entities as nodes and relationships as labeled edges. Two systems lead this space:

**Zep / Graphiti** -- A temporal knowledge graph architecture comprising three hierarchical subgraph tiers:

1. **Episode subgraph**: Raw timestamped interaction records
2. **Semantic entity subgraph**: Extracted entities and their typed relationships as triplets
3. **Community subgraph**: Hierarchical clusters detected via community algorithms, enabling multi-resolution summarization

Graphiti's distinguishing feature is temporal fact management: facts carry validity windows. When information changes, old facts are invalidated rather than deleted, enabling queries for what was true at any arbitrary point in time. The system is built on Neo4j.

**Mem0 Graph Memory (Mem0g)** -- Represents memories as a directed labeled graph where nodes are entities (with types, embeddings, and metadata) and edges are relationship triplets (source, relation, destination). Entity extraction runs on every memory write, storing embeddings in a vector database and mirroring relationships in a graph backend. Retrieval uses an entity-centric method: identify key entities in the query, locate corresponding nodes via semantic similarity, then traverse incoming and outgoing relationships to construct a contextual subgraph. This enables multi-hop reasoning across connected memories.

### Hybrid / Triple-Store Approaches

Mem0 pioneered the hybrid datastore pattern, storing each memory across three complementary backends simultaneously:

- **Vector store**: Captures semantic meaning for similarity search (22+ supported backends)
- **Key-value store**: Provides fast structured lookups for facts and preferences
- **Graph database**: Stores entity relationships for relational reasoning (Neo4j, Memgraph, Kuzu, Neptune)

On retrieval, vector search narrows candidates while the graph returns related context. This architecture has become the reference pattern for production memory systems.

## 3. Memory-Augmented Agent Systems

### Letta (formerly MemGPT)

Letta implements an LLM-as-operating-system paradigm where the model manages its own memory hierarchy, analogous to how an OS manages RAM and disk.

**Memory tiers:**

- **Core memory (RAM)**: Always-visible structured blocks pinned in the context window. Each block has a label, description, token value, and character limit. Agents can read and write their own core memory via tool calls.
- **Recall memory (conversation history)**: Complete interaction history persisted to disk, searchable and retrievable even when outside the active message buffer.
- **Archival memory (disk)**: External knowledge in vector/graph databases, queried on demand and pulled into context when needed.

**Letta V1 architecture (2025-2026)**: The system has transitioned from the original MemGPT design (which routed everything through tool calls, including reasoning via a `thinking` parameter and loop control via `request_heartbeat`) to a new architecture that leverages native model capabilities. V1 eliminates heartbeats, the `send_message` tool, and forced tool-calling for reasoning. Instead, it relies on native reasoning and direct assistant message generation, optimized for GPT-5 and Claude 4.5 Sonnet.

**Context repositories** (Feb 2026): A rebuild of memory management based on programmatic context management and git-based versioning, enabling version-controlled agent memory.

**Sleep-time compute pattern**: Memory management runs asynchronously through specialized agents during idle periods, decoupling interaction latency from memory quality. Non-blocking operations improve response times while proactive refinement reorganizes memory between conversations.

**Benchmark results**: Letta agents using GPT-4o mini with simple filesystem operations (grep, search_files, open/close) achieved 74.0% accuracy on the LoCoMo benchmark, outperforming Mem0's reported top score of 68.5% with specialized graph-based memory tools. The key insight: agents excel at using tools present in their training data (like filesystem operations), and simple familiar tools often outperform complex specialized ones.

### Mem0

Mem0 is the most widely adopted dedicated memory layer for AI agents, backed by Y Combinator. Its core pipeline:

1. **Extraction**: When a message is added via `add()`, an LLM extracts relevant facts and preferences
2. **Deduplication**: New memories are compared against existing ones to avoid redundancy
3. **Storage**: Facts are written to vector store, key-value store, and graph database simultaneously
4. **Retrieval**: Dual strategy combining vector similarity search with graph traversal for multi-hop reasoning

Graph memory (introduced January 2026) extends the system by persisting nodes and edges alongside embeddings, so recall can stitch together people, places, and events rather than just returning keyword matches.

### Agentic Memory (AgeMem)

A research framework (January 2026) that proposes unifying long-term and short-term memory management directly within the agent's policy. Rather than treating memory as a separate system managed by heuristics or auxiliary controllers, AgeMem exposes memory operations (store, retrieve, update, summarize, discard) as tool-based actions. The agent autonomously decides what and when to perform each operation.

Training uses a three-stage progressive reinforcement learning approach with a step-wise GRPO algorithm, addressing challenges from sparse and discontinuous rewards that memory operations generate. Across five long-horizon benchmarks, AgeMem showed consistent improvements over memory-augmented baselines across multiple LLM backbones.

### Cognee

Focuses on structured knowledge extraction, building knowledge graphs from unstructured data with emphasis on enterprise use cases and data governance.

## 4. Retrieval-Augmented Generation (RAG) Advances

### From Static RAG to Agentic RAG

Traditional RAG follows a fixed pipeline: query -> retrieve -> generate. Agentic RAG transcends this by embedding autonomous agents into the pipeline, using agentic design patterns (reflection, planning, tool use, multi-agent collaboration) to dynamically manage retrieval strategies and iteratively refine contextual understanding. The agent decides when to retrieve, what queries to issue, whether results are sufficient, and whether to re-retrieve with refined queries.

### A-RAG: Hierarchical Retrieval Interfaces

A-RAG (February 2026) represents a significant advance in agentic retrieval. It organizes information at three granularity levels with corresponding retrieval interfaces:

1. **Keyword search**: Lexical matching with relevance scoring weighted toward longer, more specific keywords
2. **Semantic search**: Dense vector embeddings with cosine similarity, aggregating sentence-level results by parent chunks
3. **Chunk read**: Full content access for chunks identified through other searches, enabling progressive information disclosure

The agent autonomously selects retrieval strategies without predefined workflows, supports multi-round adaptation based on intermediate results, and follows observation-reasoning-action loops. A context tracker prevents redundant retrievals by marking previously accessed chunks.

**Results**: A-RAG with GPT-5-mini achieved superior results across HotpotQA, 2WikiMultiHopQA, MuSiQue, and GraphRAG-Bench while retrieving comparable or fewer tokens than traditional RAG methods. Increasing reasoning steps from 5 to 20 improved performance by ~8%, and scaling reasoning effort from minimal to high yielded ~25% improvements.

**Failure mode shift**: Traditional RAG fails ~50% of the time from retrieval limitations (cannot find documents). A-RAG shifts the bottleneck: 82% of failures come from reasoning chain errors (found documents but reasoned incorrectly), with entity confusion as the primary challenge.

### GraphRAG

Microsoft's GraphRAG constructs entity-relation graphs from corpora to develop holistic understanding of large-scale knowledge bases. It has evolved into a mainstream RAG paradigm with innovations in knowledge graph structure design and retrieval strategies. GraphRAG is particularly effective for queries requiring synthesis across multiple documents rather than point lookups.

### MultiRAG

Emerging multi-modal and multi-source RAG frameworks retrieve not just text but images, videos, structured data, and live sensor inputs, extending agentic retrieval to richer information environments.

## 5. Context Window Management Strategies

### Context Engineering

Context engineering -- the strategic management of what information enters an LLM's context window at each step -- has emerged as a discipline distinct from prompt engineering. Andrej Karpathy describes it as "the delicate art and science of filling the context window with just the right information for the next step." Four foundational techniques define the practice:

1. **Write context**: Persist information outside the context window via scratchpads and cross-session memories
2. **Select context**: Retrieve relevant information on demand via RAG, memory search, or tool description selection
3. **Compress context**: Reduce token usage through summarization of trajectories and trimming of older messages
4. **Isolate context**: Separate context across multiple agents or sandboxed environments for focused windows

### Observation Masking

Research from JetBrains (December 2025) tested context management strategies for software engineering agents on 500 SWE-bench Verified instances. Observation masking preserves the agent's reasoning and action history while replacing older environmental observations (tool outputs, file contents) with placeholders. A rolling window keeps approximately 10 recent turns visible.

**Key finding**: Observation masking achieved over 50% cost reduction compared to unmanaged contexts and often matched or exceeded LLM summarization performance while being cheaper. With Qwen3-Coder 480B, masking improved solve rates by 2.6% and cost 52% less on average.

### LLM Summarization

A separate model compresses past interactions into abbreviated form. Theoretically enables infinite scaling through repeated summarization. However, summarization caused agents to run 13-15% longer (trajectory elongation) because generated summaries obscure failure signals that would normally trigger agent termination. Summary API calls consumed 7%+ of total costs with minimal cache reuse benefits.

### Hierarchical Memory Architectures

Production systems implement multi-tier memory with different retention policies:

- **Immediate / short-term**: Recent conversation turns verbatim at full fidelity
- **Medium-term**: Compressed summaries of recent sessions
- **Long-term**: Extracted facts, entities, and relationships in external stores

The system allocates more context budget to short-term memory while including relevant summaries from longer-term stores. Composite scoring operationalizes recency, semantic relevance, and explicit utility scores: low-scoring items are deleted, medium items consolidated, and high-value items retained in fast-access storage.

### Conversation Summary Buffer

A practical pattern (implemented in LangChain as `ConversationSummaryBufferMemory`) keeps a buffer of the most recent interactions verbatim while maintaining a running summary of older exchanges. When the token limit is exceeded, the oldest messages are summarized and merged into the summary section. This balances recent detail with condensed history.

### Token Budgeting

Allocate explicit token limits to different context categories (e.g., 1k for instructions, 5k for retrieved knowledge, 1k buffer) and proactively drop lower-priority information when approaching limits. Claude Code implements "auto-compact" after reaching 95% context capacity.

### Tool Output Management

Three strategies prevent verbose tool outputs from consuming context:

1. **Quiet mode**: Tools return only summaries or error messages rather than complete output
2. **Post-processing**: Filter for specific patterns, truncate to last N lines, or apply LLM-based summarization
3. **Pagination/scoping**: Treat large outputs as separate searchable resources rather than dumping full content into context

### Plan-and-Execute Pattern

Separate planning from execution: a first LLM call generates a high-level multi-step plan, then each step executes in isolation with minimal context (just relevant inputs). Code manages the plan and intermediate results rather than relying on LLM memory, preventing context bloat. This also enables using smaller models for execution steps and larger models for planning.

## 6. Persistent State Management Across Sessions

### Architectural Principles

Persistent agent architecture requires explicit separation and orchestration of three layers:

1. **Memory management**: Multi-store memory across working, episodic, and semantic levels
2. **Process control**: Procedural scaffolds maintaining workflows and task state
3. **Long-term adaptation**: Mechanisms for agents to evolve behavior based on accumulated experience

### Checkpointing and State Serialization

LangGraph manages persistent data across execution cycles by updating state objects as data flows through graph nodes. State is checkpointed at each node transition, enabling replay, branching, and recovery. This approach treats agent state as a first-class versioned artifact.

### Session Management

Several production approaches have emerged:

- **AWS AgentCore Runtime**: Provisions dedicated microVMs that persist for up to 8 hours, maintaining accumulated context and state across multiple invocations within a session
- **Google Vertex AI Memory Bank**: Structured, topic-aware memory organization with retrieval strategies supporting both short-term and long-term memory at general availability
- **Letta Conversations API** (January 2026): Agents maintain shared memory across parallel experiences with users, enabling a single agent to manage multiple concurrent relationships

### Memory Extraction Strategies

AWS AgentCore Memory defines three extraction strategies for converting interactions into persistent memories:

1. **Summary strategy**: Generates conversation summaries
2. **Semantic facts strategy**: Extracts key assertions and factual claims
3. **User preferences strategy**: Identifies and stores user preferences and patterns

These are stored in indexed namespaces for efficient retrieval across sessions.

### Mitigating Memory Inflation

Long-lived agents face "memory inflation" where accumulated state degrades performance. Successful systems apply composite scoring combining recency, semantic relevance, and explicit utility. Low-scoring memories are pruned, medium-scoring ones consolidated, and high-value items retained in fast-access storage. This prevents "context rot" where stale details confuse the model.

## 7. Design Pattern Comparison

Four dominant design philosophies have emerged for agent memory:

| Aspect | MemGPT/Letta | OpenAI Memory | Claude Memory | Toolkit-Based |
|---|---|---|---|---|
| Paradigm | OS-style virtualization | Product-first global | User-controlled scoped | Developer primitives |
| User base | Technical | Consumer | Professional | Developers |
| Control | Autonomous (agent manages) | Automatic (classifiers) | Explicit (user curates) | Full custom |
| Compartmentalization | Partial | Global (leakage risk) | Strict (project-scoped) | Configurable |
| Memory formation | Self-managed write-back at ~70% capacity | Explicit commands + automatic extraction | User-curated + version control | Developer-defined pipelines |
| Scalability | Single-agent limits | Global scale | Manual scaling | Custom scaling |

The field is progressing from raw text + vector search (phase 1) through entity/relationship integration via knowledge graphs (phase 2, current) toward autonomous memory orchestration with relational reasoning enabling collaborative multi-agent systems (phase 3, emerging).

## 8. Key Trends and Open Problems

**Trends:**

- Convergence on hybrid storage (vector + graph + key-value) as the standard architecture
- Shift from retrieval-time intelligence to write-time intelligence (extracting structure at ingestion rather than query time)
- Sleep-time compute and asynchronous memory management decoupling latency from memory quality
- Temporal awareness becoming a first-class concern (validity windows, fact versioning)
- Simple tools outperforming complex specialized ones when agents are familiar with them from training
- Context engineering emerging as a distinct discipline from prompt engineering

**Open problems:**

- Unified memory management integrated into agent policy (AgeMem direction) vs. external orchestration
- Cross-agent memory synchronization in multi-agent systems
- Memory governance, privacy, and selective forgetting
- Scaling memory systems beyond single-agent workloads
- Balancing compression aggressiveness against information loss
- Failure mode shift from retrieval failures to reasoning failures as retrieval improves

## Sources

- [The 6 Best AI Agent Memory Frameworks (2026) - Machine Learning Mastery](https://machinelearningmastery.com/the-6-best-ai-agent-memory-frameworks-you-should-try-in-2026/)
- [AI Agent Memory: Build Stateful AI Systems - Redis](https://redis.io/blog/ai-agent-memory-stateful-systems/)
- [Powering Long-Term Memory for Agents with LangGraph and MongoDB](https://www.mongodb.com/company/blog/product-release-announcements/powering-long-term-memory-for-agents-langgraph)
- [ICLR 2026 Workshop: MemAgents - Memory for LLM-Based Agentic Systems](https://openreview.net/pdf?id=U51WxL382H)
- [Mem0: Memory in Agents - What, Why and How](https://mem0.ai/blog/memory-in-agents-what-why-and-how/)
- [Mem0 Graph Memory Documentation](https://docs.mem0.ai/open-source/features/graph-memory)
- [Mem0: Building Production-Ready AI Agents with Scalable Long-Term Memory](https://arxiv.org/html/2504.19413v1)
- [Demystifying the Architecture of Mem0 - Medium](https://medium.com/@parthshr370/from-chat-history-to-ai-memory-a-better-way-to-build-intelligent-agents-f30116b0c124)
- [Agentic RAG Survey (arXiv 2501.09136)](https://arxiv.org/abs/2501.09136)
- [A-RAG: Scaling Agentic RAG via Hierarchical Retrieval Interfaces (arXiv 2602.03442)](https://arxiv.org/html/2602.03442v1)
- [RAG in 2026: Bridging Knowledge and Generative AI - Squirro](https://squirro.com/squirro-blog/state-of-rag-genai)
- [Efficient Context Management for LLM-Powered Agents - JetBrains Research](https://blog.jetbrains.com/research/2025/12/efficient-context-management/)
- [Context Engineering for Agents - LangChain](https://blog.langchain.com/context-engineering-for-agents/)
- [Context Window Management in Agentic Systems - jroddev](https://blog.jroddev.com/context-window-management-in-agentic-systems/)
- [Context Window Management Strategies - Maxim AI](https://www.getmaxim.ai/articles/context-window-management-strategies-for-long-context-ai-agents-and-chatbots/)
- [Top Techniques to Manage Context Length in LLMs - Agenta](https://agenta.ai/blog/top-6-techniques-to-manage-context-length-in-llms)
- [Letta V1 Agent Architecture](https://www.letta.com/blog/letta-v1-agent)
- [Benchmarking AI Agent Memory: Is a Filesystem All You Need? - Letta](https://www.letta.com/blog/benchmarking-ai-agent-memory)
- [Agent Memory: How to Build Agents that Learn and Remember - Letta](https://www.letta.com/blog/agent-memory)
- [Letta/MemGPT Documentation](https://docs.letta.com/concepts/memgpt/)
- [Stateful AI Agents: A Deep Dive into Letta (MemGPT) - Medium](https://medium.com/@piyush.jhamb4u/stateful-ai-agents-a-deep-dive-into-letta-memgpt-memory-models-a2ffc01a7ea1)
- [Graphiti: Knowledge Graph Memory for an Agentic World - Neo4j](https://neo4j.com/blog/developer/graphiti-knowledge-graph-memory/)
- [Zep: A Temporal Knowledge Graph Architecture for Agent Memory (arXiv 2501.13956)](https://arxiv.org/abs/2501.13956)
- [AriGraph: Learning Knowledge Graph World Models with Episodic Memory (IJCAI 2025)](https://arxiv.org/abs/2407.04363)
- [Design Patterns for Long-Term Memory in LLM-Powered Architectures - Serokell](https://serokell.io/blog/design-patterns-for-long-term-memory-in-llm-powered-architectures)
- [Agentic Memory: Unified LTM and STM Management (arXiv 2601.01885)](https://arxiv.org/abs/2601.01885)
- [Beyond Short-Term Memory: 3 Types of Long-Term Memory AI Agents Need - Machine Learning Mastery](https://machinelearningmastery.com/beyond-short-term-memory-the-3-types-of-long-term-memory-ai-agents-need/)
- [Memory Mechanisms in LLM Agents - EmergentMind](https://www.emergentmind.com/topics/memory-mechanisms-in-llm-based-agents)
- [Persistent Agent Architecture - EmergentMind](https://www.emergentmind.com/topics/persistent-agent-architecture)
- [Deep Dive into State Persistence Agents in AI - SparkCo](https://sparkco.ai/blog/deep-dive-into-state-persistence-agents-in-ai)
- [Build Persistent Memory with Mem0, ElastiCache, and Neptune - AWS](https://aws.amazon.com/blogs/database/build-persistent-memory-for-agentic-ai-applications-with-mem0-open-source-amazon-elasticache-for-valkey-and-amazon-neptune-analytics/)
- [Context Engineering: OpenAI Agents SDK - OpenAI Cookbook](https://developers.openai.com/cookbook/examples/agents_sdk/context_personalization/)
- [LLM Chat History Summarization Guide - Mem0](https://mem0.ai/blog/llm-chat-history-summarization-guide-2025)
