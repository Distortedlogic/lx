# Agent Memory Systems Deep Dive: Memory-as-a-Service

## Overview

This document surveys dedicated memory-as-a-service tools for AI agents, analyzing Mem0, Zep/Graphiti, and Hindsight as alternatives to Letta's self-managed memory approach. The focus is on architecture, retrieval strategies, benchmark performance, and what primitives an agentic workflow DSL needs to support agent memory natively.

---

## Mem0

### Identity

Mem0 is a managed memory layer for AI agents and applications. Founded January 2024 by Taranjeet Singh (CEO) and Deshraj Yadav (CTO, previously led AI Platform at Tesla Autopilot). YC-backed (W24), $24M total raised ($3.9M seed + $20M Series A from Peak XV and Basis Set Ventures, announced October 2025).

- **GitHub**: [mem0ai/mem0](https://github.com/mem0ai/mem0) -- 51.1k stars, 5.7k forks
- **License**: Apache 2.0
- **Languages**: Python (64.8%), TypeScript (24.4%), MDX (5.6%)
- **Latest release**: v1.0.7 (March 2026)
- **Paper**: "Mem0: Building Production-Ready AI Agents with Scalable Long-Term Memory" (arXiv:2504.19413, April 2025)

By Q3 2025, Mem0 processed 186 million API calls monthly, growing ~30% month-over-month from 35 million in Q1 2025.

### Architecture: The Two-Phase Pipeline

Mem0 implements a two-phase memory pipeline: **Extraction** then **Update**.

**Extraction Phase**: The system ingests three context sources:
1. The latest exchange (message pair m_t-1, m_t)
2. A rolling summary S from the database
3. The m most recent messages (m=10 by default)

An LLM-based extractor function phi generates candidate memories: Omega = {omega_1, omega_2, ..., omega_n}. A background module refreshes the long-term summary asynchronously so inference never stalls.

**Update Phase**: Each candidate fact undergoes semantic evaluation. The system retrieves the top s=10 similar existing memories via vector embeddings, then presents these to an LLM via function-calling interface. The LLM selects one of four operations:

| Operation | When Used |
|-----------|-----------|
| **ADD** | Fact is new, no semantic equivalents exist |
| **UPDATE** | Augment existing memory with complementary information |
| **DELETE** | Contradicted or obsolete information |
| **NOOP** | No modification needed |

The pipeline flow: ingest conversation -> LLM extracts facts -> retrieve similar memories via embeddings -> LLM decides ADD/UPDATE/DELETE -> store results.

### Storage Architecture: Hybrid Datastore

Mem0 uses a triple-store approach:

1. **Vector Database** (primary): Dense embeddings for semantic similarity search. Supports Qdrant, Pinecone, Weaviate, Chroma, PostgreSQL+pgvector. This is the "primary brain" -- atomic fact strings indexed by embedding vectors.
2. **Key-Value Store**: Fast lookup for metadata, timestamps, user/agent scoping.
3. **Graph Database** (optional, Pro tier only): Knowledge graph for entity-relationship data. Supports Neo4j, Memgraph, Amazon Neptune, Kuzu, Apache AGE.

The graph runs as a completely independent parallel system. Graph edges do NOT reorder vector search results. Graph memory adds related context but maintains vector-based ranking.

### Scoring and Retrieval

The scoring layer evaluates memories on three dimensions:

- **Relevance**: Embedding distance between memory and current query
- **Importance**: Critical information (allergies) ranks above trivial information (music preferences)
- **Recency**: More recently mentioned preferences get priority

Weights are tunable per use case. A medical agent might set importance=0.5 while a casual chatbot weights recency higher.

### Graph Memory Feature (Mem0^g)

The graph variant enhances the base architecture with directed labeled graphs G=(V,E,L):
- Nodes represent entities with type classification, embedding vector, and timestamp
- Edges represent relationships as triplets (v_source, relation, v_dest)
- Two-stage LLM pipeline: first extracts entities, then derives relationship connections

Dual retrieval: entity-centric matching plus semantic triplet similarity against query embeddings. Graph and vector operations run in parallel via concurrent.futures.ThreadPoolExecutor.

Configuration for Neo4j:
```python
config = {
    "graph_store": {
        "provider": "neo4j",
        "config": {
            "url": "neo4j+s://<instance>.databases.neo4j.io",
            "username": "neo4j",
            "password": "<PASSWORD>",
        }
    }
}
memory = Memory.from_config(config)
```

### API Surface

```python
from mem0 import MemoryClient
client = MemoryClient(api_key="your-api-key")

# Store memories from conversation
client.add(messages, user_id="user123")

# Semantic search
results = client.search("dietary restrictions?", filters={"user_id": "user123"})

# Retrieve all memories
memories = client.get_all(user_id="user123")

# Modify existing memory
client.update(memory_id, data="Updated information")

# Remove memory
client.delete(memory_id)
```

Scoping via `user_id`, `agent_id`, and `run_id` parameters. Multi-agent workflows scope memory per agent or share across teams.

### OpenMemory MCP Server

Launched May 2025. Local-first memory layer using Model Context Protocol. APIs: `add_memories`, `search_memory`, `list_memories`, `delete_all_memories`. Vector-backed search using Qdrant. Cross-client memory: store in Cursor, retrieve in Claude Desktop. All data stays on-machine.

### Framework Integrations

- **LangChain**: Custom memory providers implementing BaseChatMemory interface
- **CrewAI**: Shared memory across multi-agent collaborations via agent_id parameter
- **AutoGen**: ConversableAgent class with Mem0 context retrieval
- **LangGraph, Flowise**: Memory layer works with any architecture through API calls

### Deployment and Pricing

| Tier | Price | Memories | Retrieval Calls | Key Features |
|------|-------|----------|-----------------|--------------|
| Hobby | Free | 10K | 1K/month | Community support |
| Starter | $19/mo | 50K | 5K/month | Vector search only |
| Pro | $249/mo | Unlimited | 50K/month | Graph Memory, analytics, multiple projects |
| Enterprise | Custom | Unlimited | Unlimited | On-prem, SSO, audit logs, SLA, SOC 2, HIPAA |

Startup program: 3 months free Pro for companies with under $5M funding.

Self-hosted: Apache 2.0, fully self-hostable with Docker. Mature documentation. The managed platform adds scaling, observability, and compliance (SOC 2, HIPAA, zero-trust).

### Benchmark Results

**LoCoMo Benchmark** (from Mem0's own paper, GPT-4o-mini):

| System | Overall J Score |
|--------|-----------------|
| Full-Context | 72.90% +/- 0.19 |
| Mem0^g (graph) | 68.44% +/- 0.17 |
| Mem0 (vector) | 66.88% +/- 0.15 |
| Zep | 65.99% +/- 0.16 |
| Best RAG (k=2, 256 tokens) | 60.97% +/- 0.20 |
| MemGPT | ~41% |

Note: Zep's team disputed this evaluation, claiming implementation errors inflated Zep's score downward. Corrected evaluation: Zep at 75.14% +/- 0.17, approximately 10% relative improvement over Mem0's best.

**LongMemEval Benchmark** (independent evaluation): Mem0 scored 49.0%. This is the harder, more realistic benchmark.

**Efficiency numbers** (p95 latency):
- Mem0 total: 1.440s (search: 0.200s)
- Mem0^g total: 2.590s (search: 0.657s)
- Zep total: 2.926s
- Full-context total: 17.117s

**Token efficiency**: Mem0 ~7K tokens per conversation vs Zep 600K+ tokens (Zep stores much more context in the graph).

### Criticisms and Limitations

1. **Graph-gating creates a pricing cliff**: The most architecturally interesting feature (graph memory) requires Pro at $249/month. Free/Starter tiers get basic vector search only.

2. **No native temporal model**: Memories carry creation timestamps but lack fact validity windows or temporal supersession. When a user changes jobs or moves cities, Mem0 either appends the new fact alongside the old one (contradictory memories) or silently overwrites with no audit trail.

3. **Context-free extraction**: If a user mentions "I hate mushrooms," the system extracts that -- but they might hate mushrooms on pizza but love mushroom soup. Automated extraction loses nuance.

4. **Benchmark methodology disputed**: Mem0's LoCoMo evaluation of Zep contained documented implementation errors: incorrect user model, non-standard timestamp handling, sequential rather than parallel searches inflating latency.

5. **LoCoMo itself is insufficient**: Conversations averaging 16K-26K tokens fall within modern context windows. A full-context baseline achieves J=73% vs Mem0's 68%. LongMemEval (where Mem0 scores 49%) is the harder test.

6. **Reliability at scale**: Reports of indexing reliability issues -- memories not being added consistently, context recall failures under load.

7. **Vector-only architecture limitation**: Vector memory retrieves similar past exchanges but treats each memory independently. It loses explicit relationships between facts.

---

## Zep / Graphiti

### Identity

Zep is a context engineering platform for building AI agents, powered by Graphiti -- a temporal knowledge graph engine. Founded 2023 by Daniel Chalef (CEO). YC-backed (W24), based in Oakland, CA.

- **GitHub (Graphiti)**: [getzep/graphiti](https://github.com/getzep/graphiti) -- 24.2k stars, 2.4k forks
- **License**: Apache 2.0 (Graphiti). Zep Community Edition deprecated; existing repo remains open under Apache 2.0 with no further updates.
- **Language**: Python
- **Paper**: "Zep: A Temporal Knowledge Graph Architecture for Agent Memory" (arXiv:2501.13956, January 2025)
- **Key personnel**: Daniel Chalef, Preston Rasmussen (co-authors on the paper)

### Core Architecture: The Temporal Knowledge Graph

Zep's fundamental bet: **graph-native with time as a first-class dimension**. Unlike Mem0's vector-first approach, everything is built around Graphiti's temporally-aware knowledge graph.

The graph comprises three hierarchical subgraphs:

**1. Episode Subgraph (G_e)**: Raw input data stored non-lossily. Episodes are messages, text blocks, or JSON objects. They serve as the foundation from which semantic information is extracted. Every episode has a reference_time.

**2. Semantic Entity Subgraph (G_s)**: Extracted entity nodes and semantic edges representing relationships derived from episodes. Entities are embedded into 1024-dimensional vectors for similarity matching. Entities receive type classification and summaries.

**3. Community Subgraph (G_c)**: Clusters of strongly connected entities with high-level summaries. The highest abstraction level. Communities receive map-reduce-style summaries with names containing key terms for embedding and similarity search.

### The Bi-Temporal Model

This is Zep's key differentiator. Every graph edge tracks two independent timelines:

- **Timeline T (Event time)**: When this fact was true in the real world. Fields: `t_valid` (when fact became true), `t_invalid` (when fact ceased being true)
- **Timeline T' (Ingestion time)**: When the system learned about it. Fields: `t'_created`, `t'_expired`

This enables queries like: "What did we know about this customer's preferences as of March 1st?" and "When did this information change?"

When new information contradicts existing facts with temporal overlap, the system invalidates old edges by setting their invalidation timestamp to match the new edge's validity start. Old data is preserved, not deleted.

### Entity and Relationship Processing

**Entity Extraction**: Processes current message plus four prior messages for context. Speakers are automatically extracted as entities. Uses a reflection technique inspired by Reflexion to minimize hallucinations. 4-6 LLM calls per episode for node operations.

**Entity Resolution**: Three-tier deduplication strategy:
1. Exact match
2. Fuzzy similarity (hybrid cosine + full-text search)
3. LLM reasoning for ambiguous cases

Critical at scale: the system must determine whether "Sarah Johnson," "S. Johnson," and "the new engineer" all refer to the same person.

**Relationship Extraction**: Facts extracted between identified entities. The same fact can be extracted multiple times between different entities, enabling hyperedge modeling. 2-4 LLM calls per episode for edge operations.

**Edge Invalidation**: When a new episode says information has changed, the system sets `invalid_at` on the old edge instead of deleting it, preserving history.

### Community Detection

Uses label propagation rather than the Leiden algorithm. Chosen because label propagation enables dynamic extension through single recursive steps. When a new node is added, it is assigned to whichever community contains most neighboring nodes. This allows incremental updates without full recomputation.

### Retrieval Mechanism

The retrieval function f(alpha) -> beta comprises three stages:

**Search (phi)**: Three hybrid search methods identify candidates:
- **Cosine similarity (phi_cos)**: Semantic vector matching against 1024-dim embeddings
- **BM25 full-text (phi_bm25)**: Keyword matching via Neo4j/Lucene. For edges searches the fact field, for nodes the entity name, for communities the community name
- **Breadth-first graph search (phi_bfs)**: Contextual n-hop traversal from identified entities

**Reranking (rho)**: Multiple strategies applied:
- Reciprocal Rank Fusion across the three search methods
- Maximal Marginal Relevance (balances relevance and diversity)
- Episode-mention frequency prioritization
- Node-distance ordering from centroid entity
- Cross-encoder LLM relevance scoring (highest cost, highest quality)

**Constructor (chi)**: Formats selected nodes and edges into text including facts with temporal ranges, entity names/summaries, and community summaries.

P95 retrieval latency: 300ms.

### The add_episode Method

```python
await graphiti.add_episode(
    name="conversation_turn_42",
    episode_body="User said they moved from NYC to SF last month",
    source="message",           # "text", "message", or "json"
    source_description="Support chat session",
    group_id="user_12345",      # namespace for isolation
    reference_time=datetime.now(),
)
```

Processing is incremental: no full graph recomputation needed. Each episode is analyzed by LLM to extract entities and relationships, which are deduplicated against existing graph elements.

### Custom Entity Types

Graphiti supports domain-specific entity definitions via Pydantic models, enabling precise context extraction for specialized domains (healthcare, legal, finance).

### Database Support

Neo4j 5.26, FalkorDB 1.1.2, Kuzu 0.11.2, Amazon Neptune (Database Cluster or Analytics Graph + OpenSearch Serverless). Neo4j is the primary/reference implementation.

### Implementation Details

- **Embeddings**: BGE-m3 (BAAI), 1024-dimensional
- **Graph Construction LLM**: GPT-4o-mini
- **Query LLM**: GPT-4o-mini or GPT-4o
- **Schema**: Predefined Cypher queries (avoids LLM-generated database commands)
- **MCP Server**: Available for integration with AI assistants

### Deployment and Pricing

| Tier | Price | Credits | Rate Limit | Features |
|------|-------|---------|------------|----------|
| Free | $0 | 1K/month | Low, variable | Lower priority processing |
| Flex | $25/mo | 20K/month | 600 req/min | 5 projects, 10 custom entity types |
| Flex Plus | $475/mo | 300K/month | 1K req/min | 20 custom entity types, webhooks, API logs |
| Enterprise | Custom | Custom | Guaranteed | SOC 2 Type II, HIPAA BAA, BYOK/BYOM/BYOC |

Credit system: each Episode costs 1 credit. Episodes over 350 bytes bill in multiples. Credits roll over up to 60 days.

Key pricing difference: Zep provides full feature access at every tier (temporal graph, entity resolution, full Graphiti engine) for $25/month. Mem0 gates graph features behind $249/month Pro.

Self-hosting: Use Graphiti directly (Apache 2.0), but requires managing Neo4j/FalkorDB yourself. Zep Community Edition is deprecated with no further updates.

### Benchmark Results

**Deep Memory Retrieval (DMR)**:
- Zep (gpt-4-turbo): 94.8% vs MemGPT 93.4%
- Zep (gpt-4o-mini): 98.2% vs full-context baseline 98.0%

**LongMemEval**:
- Zep (gpt-4o-mini): 63.8% vs baseline 55.4% (+15.2%)
- Zep (gpt-4o): 71.2% vs baseline 60.2% (+18.5%)
- Latency: 2.58s vs 28.9s with full-context gpt-4o (90% reduction)
- Context: ~1.6K tokens vs ~115K full-context

Strongest improvements in temporal-reasoning, multi-session, and preference questions.

### When Zep/Graphiti Is Better vs Worse

**Better than Mem0 when**:
- Temporal reasoning is central (compliance tracking, audit trails, evolving relationships)
- Queries involve "what changed?" or "what was true on date X?"
- Multi-hop entity traversal required
- Full features needed at lower price point ($25 vs $249)
- Structurally complex queries

**Worse than Mem0 when**:
- Simple personalization (preferences, history recall)
- Self-hosting is required (managing Neo4j is operationally expensive)
- Token efficiency matters (Zep stores 600K+ tokens vs Mem0's 7K per conversation)
- Ecosystem breadth matters (Mem0's integrations are more mature)
- Immediate post-ingestion retrieval needed (Zep's graph construction can delay results)

---

## Hindsight

### Identity

Hindsight is an open-source agent memory system built by Vectorize, co-founded by Chris Latimer (CEO). Released December 2025. Validated by collaborators from The Washington Post and Virginia Tech.

- **GitHub**: [vectorize-io/hindsight](https://github.com/vectorize-io/hindsight) -- 6.3k stars, 355 forks
- **License**: MIT
- **Languages**: Python, Node.js, JavaScript/TypeScript
- **Database**: PostgreSQL
- **Deployment**: Docker, Kubernetes (Helm)
- **Paper**: "Hindsight is 20/20: Building Agent Memory that Retains, Recalls, and Reflects" (arXiv:2512.12818, December 2025)
- **LLM Support**: OpenAI, Anthropic, Gemini, Groq, Ollama

### Architecture: TEMPR + CARA

Hindsight ties together two components:

**TEMPR** (Temporal Entity Memory Priming Retrieval): Implements the `retain` and `recall` operations over long-term memory.

**CARA** (Coherent Adaptive Reasoning Agents): Implements the `reflect` operation -- preference-conditioned reasoning over memory.

### The Four Knowledge Networks

Hindsight organizes memories into four distinct logical networks:

| Network | Symbol | Stores | Perspective |
|---------|--------|--------|-------------|
| **World** | W | Objective facts about the external environment | Third-person, factual |
| **Experience** (Bank) | B | Agent's own actions and interactions | First-person, biographical |
| **Opinion** | O | Subjective judgments with confidence scores (0-1) and timestamps | Agent's evolving beliefs |
| **Observation** | S | Preference-neutral entity summaries synthesized from underlying facts | Neutral, synthesized |

This fact/opinion separation is unique among memory systems. No other system (MemGPT, Zep, Mem0, A-Mem) provides it.

### Memory Unit Structure

Each memory unit contains:
- Unique identifier and bank identifier
- Narrative text
- Embedding vector (R^d)
- Occurrence interval (tau_s, tau_e) and mention timestamp (tau_m)
- Fact type (world/experience/opinion/observation)
- Optional confidence score (opinions only)
- Auxiliary metadata (context, access count, full-text search vectors)

### Graph Link Types

Four edge types connect memories:
- **Temporal**: Weight decays exponentially with time distance
- **Semantic**: Created when cosine similarity exceeds threshold theta_s
- **Entity**: Bidirectional links between memories mentioning same canonical entity
- **Causal**: Represent cause-effect (causes, caused_by, enables, prevents)

### Four-Way Parallel Retrieval (TEMPR Recall)

Every query triggers four simultaneous retrieval channels:

1. **Semantic Retrieval**: Vector similarity search using cosine similarity via HNSW indexing
2. **Keyword Retrieval (BM25)**: Full-text search for precise matching of proper nouns and technical terms
3. **Graph Retrieval**: Spreading activation across the memory graph, traversing entity, temporal, semantic, and causal links
4. **Temporal Graph Retrieval**: Rule-based and sequence-to-sequence date parsing, filtering memories by temporal intervals

The four ranked lists merge via Reciprocal Rank Fusion (RRF), followed by neural cross-encoder reranking and token budget filtering.

### Reflection Mechanism (CARA)

The `reflect` operation generates preference-conditioned responses:

**Behavioral Profile**: Three disposition parameters plus bias-strength:
- **Skepticism** (1-5): Higher = more cautious, evidence-demanding; Lower = trusting, exploratory
- **Literalism** (1-5): Higher = exact wording, explicit instructions; Lower = reading between lines, inferring
- **Empathy** (1-5): Influences interpretation and reasoning about information
- **Bias-strength** (beta, 0-1): Overall influence of dispositions on reasoning

**Opinion Formation**: Retrieved facts combined with verbalized behavioral profile generate LLM responses with confidence scores.

**Opinion Reinforcement**: New facts assessed as reinforcing, weakening, contradicting, or neutral:
- Reinforcement: c' = min(c + alpha, 1.0)
- Contradiction: c' = max(c - 2*alpha, 0.0)

**Background Merging**: LLM-powered function resolves conflicting biographical information while enriching descriptions.

Given the same facts, agents with different dispositions form different opinions -- just like humans.

### LongMemEval Benchmark Results

| Configuration | Overall | Single-Session | Multi-Session | Temporal | Knowledge Update | Preference |
|---------------|---------|----------------|---------------|----------|-----------------|------------|
| Hindsight (OSS-20B) | 83.6% | 95.7% | 79.7% | 79.7% | 84.6% | 66.7% |
| Hindsight (OSS-120B) | 89.0% | -- | -- | -- | -- | -- |
| Hindsight (Gemini-3 Pro) | 91.4% | -- | -- | -- | -- | -- |
| Full-context OSS-20B baseline | 39.0% | -- | -- | -- | -- | -- |
| Full-context GPT-4o | 60.2% | -- | -- | -- | -- | -- |

The structured memory provides dramatic gains:
- Multi-session: 21.1% -> 79.7% (+58.6 points)
- Temporal reasoning: 31.6% -> 79.7% (+48.1 points)
- Knowledge update: 60.3% -> 84.6% (+24.3 points)

### Comparison with Other Systems

Feature matrix from the Hindsight paper:

| Feature | Hindsight | MemGPT | Zep | A-Mem | Mem0 |
|---------|-----------|--------|-----|-------|------|
| Fact/opinion separation | Yes | No | No | No | No |
| Opinion evolution + confidence | Yes | No | No | No | No |
| Behavioral dispositions | Yes | No | No | No | No |
| Multi-strategy retrieval | Yes | Minimal | Partial | Minimal | Partial |
| External-only memory | Yes | Yes | Yes | Yes | Yes |

### When Hindsight Is Better vs Worse

**Better when**:
- Institutional knowledge (team decisions, process evolution)
- Complex queries spanning temporal ranges and multiple entities
- Full-featured memory needed without pricing tiers
- Preference-conditioned reasoning needed (different agents, different dispositions)
- LongMemEval-style workloads (long conversations, knowledge updates)

**Worse when**:
- Simple personalization (Mem0 is simpler to deploy)
- Established ecosystem integration needed (Mem0's integrations are more mature)
- Graph-native temporal queries with bi-temporal model (Zep's valid_from/valid_to is more explicit)
- Smaller community and ecosystem (6.3K stars vs Mem0's 51K)

---

## The LongMemEval Benchmark

### What It Tests

LongMemEval is the leading benchmark for evaluating long-term AI memory. Created by Di Wu, Hongwei Wang, Wenhao Yu, Yuwei Zhang, Kai-Wei Chang, and Dong Yu. Accepted at ICLR 2025. GitHub: [xiaowu0162/LongMemEval](https://github.com/xiaowu0162/LongMemEval).

**500 high-quality questions** across conversations up to 1.5 million tokens spanning multiple sessions.

### Five Core Competencies Tested

1. **Information Extraction**: Can the system find specific facts from history?
2. **Multi-Session Reasoning**: Can it synthesize across multiple conversations?
3. **Knowledge Updates**: When facts change, does the system track the latest version?
4. **Temporal Reasoning**: Can it answer "when" questions and reason about time?
5. **Abstention**: Does the system know when NOT to answer?

### Question Types

single-session-user, single-session-assistant, single-session-preference, temporal-reasoning, knowledge-update, multi-session, and abstention variants.

### Dataset Variants

- **LongMemEval_S**: ~40 history sessions (~115K tokens)
- **LongMemEval_M**: ~500 sessions per history
- **LongMemEval_Oracle**: Only evidence sessions included

### Why It Matters Over LoCoMo

LoCoMo (used by Mem0's paper) has 10 conversations averaging 16K-26K tokens. These fit within modern context windows. A full-context baseline achieves 73%. LongMemEval is broadly considered the harder, more realistic test because:
- Significantly longer conversations (avg. 115K tokens)
- Explicitly tests temporal understanding and information change
- Human-curated for quality
- Better represents enterprise workloads

### Comparative Scores on LongMemEval

| System | Score | Notes |
|--------|-------|-------|
| Hindsight (Gemini-3 Pro) | 91.4% | SOTA as of January 2026 |
| Hindsight (OSS-120B) | 89.0% | |
| Hindsight (OSS-20B) | 83.6% | |
| Zep (gpt-4o) | 71.2% | |
| Zep (gpt-4o-mini) | 63.8% | |
| Full-context GPT-4o | 60.2% | |
| Full-context baseline | 55.4% | |
| Mem0 | 49.0% | Independent evaluation |

---

## Cross-System Analysis

### Architectural Paradigms

| Paradigm | Representative | How It Works | Strength | Weakness |
|----------|---------------|--------------|----------|----------|
| **Vector-based** | Mem0 | Embed facts as vectors, cosine similarity retrieval | Simple, fast, low token overhead | Loses relationships, no temporal model |
| **Graph-based** | Zep/Graphiti | Temporal knowledge graph, entity resolution, edge validity | Rich temporal/relational queries | Expensive construction, high token footprint |
| **Tiered self-managed** | Letta/MemGPT | Agent manages own core/recall/archival memory via tool calls | Agent autonomy, self-improvement | Depends on agent reasoning quality |
| **Multi-strategy** | Hindsight | Four parallel retrieval, four knowledge networks, behavioral reflection | Highest benchmark scores, epistemic clarity | More complex, newer ecosystem |

### Architecture Decision Matrix

| Requirement | Best Choice | Why |
|-------------|-------------|-----|
| Simple personalization | Mem0 | Easiest integration, largest ecosystem |
| Temporal reasoning ("when did X change?") | Zep/Graphiti | Bi-temporal model is native |
| Knowledge update tracking | Hindsight or Zep | Both handle supersession; Mem0 struggles |
| Multi-hop entity queries | Zep/Graphiti | Graph traversal is native |
| Agent self-improvement | Letta | Agent decides what to remember/forget |
| Highest raw accuracy | Hindsight | 91.4% LongMemEval |
| Cheapest full-featured | Zep ($25/mo) | All features at every tier |
| Self-hosted simplicity | Mem0 | Apache 2.0, Docker, mature docs |
| Preference-conditioned reasoning | Hindsight | CARA with behavioral dispositions |

### Memory Consolidation, Forgetting, and Temporal Decay

**Mem0's approach**: LLM decides ADD/UPDATE/DELETE/NOOP for each new fact. Simple but lacks temporal awareness. Contradictory information either creates duplicates or silently overwrites.

**Zep's approach**: Edge invalidation with temporal markers. Old facts are never deleted, only marked with `invalid_at`. Full audit trail. But graph grows unboundedly.

**Hindsight's approach**: Opinion confidence scores update via reinforcement/contradiction. Temporal edges decay exponentially. Causal links capture cause-effect chains.

**Letta's approach**: Agent self-manages via tool calls. Memory pressure triggers at 70% context, force-eviction at 100% with recursive summarization. Progressive compression where older messages have diminishing influence.

**FadeMem** (academic, January 2026): Biologically-inspired forgetting using Ebbinghaus's forgetting curve. Differential decay rates across dual-layer hierarchy. Retention governed by adaptive exponential decay modulated by semantic relevance, access frequency, and temporal patterns. 45% storage reduction with superior multi-hop reasoning.

### What Primitives a DSL Needs for Agent Memory

Based on this survey, an agentic workflow DSL should consider these memory primitives:

**1. Storage Operations**
- `retain(fact, network, confidence?)` -- store with optional type classification and confidence
- `forget(fact_id)` -- explicit removal
- `update(fact_id, new_value)` -- supersession with audit trail
- `invalidate(fact_id, reason)` -- temporal invalidation, not deletion

**2. Retrieval Operations**
- `recall(query)` -- multi-strategy retrieval (semantic + keyword + graph + temporal)
- `recall_temporal(query, time_range)` -- time-bounded retrieval
- `recall_entity(entity_name)` -- entity-centric graph traversal
- `recall_causal(event)` -- cause-effect chain traversal

**3. Reflection Operations**
- `reflect(memories, disposition?)` -- synthesize opinions from facts with optional behavioral parameters
- `consolidate(memories)` -- merge related memories into higher-level abstractions
- `summarize(memory_set)` -- progressive compression

**4. Scoping and Isolation**
- `memory_scope(agent_id, user_id, session_id)` -- namespace memories
- `shared_memory(group_id)` -- cross-agent memory sharing
- `private_memory(agent_id)` -- agent-local memories

**5. Temporal Primitives**
- `valid_at(fact, timestamp)` -- check fact validity at a point in time
- `history_of(entity)` -- temporal evolution of an entity
- `changes_since(timestamp)` -- delta queries

**6. Decay and Lifecycle**
- `decay_policy(strategy, rate)` -- configure temporal decay (exponential, linear, step)
- `importance(fact, weight)` -- set importance for scoring
- `access_count(fact)` -- track retrieval frequency for relevance

**7. Knowledge Network Types**
- `world_memory` -- objective external facts
- `experience_memory` -- agent's own actions (first-person)
- `opinion_memory` -- subjective beliefs with confidence
- `observation_memory` -- synthesized entity summaries

### The Benchmark Wars

Different vendors benchmark on different datasets, making direct comparison difficult:
- **Mem0** prefers LoCoMo (shorter, where they score well)
- **Zep** prefers LongMemEval and DMR (where temporal reasoning matters)
- **Hindsight** prefers LongMemEval (where multi-strategy retrieval excels)

LongMemEval is the more demanding, realistic test. LoCoMo conversations fit within modern context windows, meaning a simple "put everything in context" baseline can beat dedicated memory systems.

The fact that different vendors benchmark on different datasets lets everyone claim leadership simultaneously.

---

## Sources

- [Mem0 GitHub Repository](https://github.com/mem0ai/mem0)
- [Mem0 Paper: arXiv:2504.19413](https://arxiv.org/abs/2504.19413)
- [Mem0 Pricing](https://mem0.ai/pricing)
- [Mem0 Graph Memory Documentation](https://docs.mem0.ai/open-source/features/graph-memory)
- [Mem0 OpenMemory MCP](https://mem0.ai/blog/introducing-openmemory-mcp)
- [Mem0 $24M Series A (TechCrunch)](https://techcrunch.com/2025/10/28/mem0-raises-24m-from-yc-peak-xv-and-basis-set-to-build-the-memory-layer-for-ai-apps/)
- [Graphiti GitHub Repository](https://github.com/getzep/graphiti)
- [Zep Paper: arXiv:2501.13956](https://arxiv.org/abs/2501.13956)
- [Zep Pricing](https://www.getzep.com/pricing/)
- [Zep Blog: "Is Mem0 Really SOTA?"](https://blog.getzep.com/lies-damn-lies-statistics-is-mem0-really-sota-in-agent-memory/)
- [Graphiti Documentation](https://help.getzep.com/graphiti/getting-started/overview)
- [Hindsight GitHub Repository](https://github.com/vectorize-io/hindsight)
- [Hindsight Paper: arXiv:2512.12818](https://arxiv.org/abs/2512.12818)
- [Hindsight VentureBeat Coverage](https://venturebeat.com/data/with-91-accuracy-open-source-hindsight-agentic-memory-provides-20-20-vision)
- [Hindsight vs Mem0 Comparison](https://vectorize.io/articles/hindsight-vs-mem0)
- [Mem0 vs Zep Comparison](https://vectorize.io/articles/mem0-vs-zep)
- [LongMemEval GitHub](https://github.com/xiaowu0162/LongMemEval)
- [5 AI Agent Memory Systems Compared (DEV Community)](https://dev.to/varun_pratapbhardwaj_b13/5-ai-agent-memory-systems-compared-mem0-zep-letta-supermemory-superlocalmemory-2026-benchmark-59p3)
- [FadeMem Paper: arXiv:2601.18642](https://arxiv.org/abs/2601.18642)
- [Letta/MemGPT Memory Management Docs](https://docs.letta.com/advanced/memory-management/)
- [Cognee AI Memory Tools Evaluation](https://www.cognee.ai/blog/deep-dives/ai-memory-tools-evaluation)
