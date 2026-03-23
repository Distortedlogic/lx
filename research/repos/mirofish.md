# MiroFish: Multi-Agent Swarm Intelligence Prediction Engine

MiroFish demonstrates that **emergent social dynamics from thousands of LLM-powered agents interacting on simulated social platforms** can serve as a prediction mechanism for real-world outcomes. The system extracts entities and relationships from seed documents, generates agent personas with distinct personality profiles and memory, runs parallel Twitter/Reddit simulations via the OASIS framework, then synthesizes prediction reports through a ReACT-based report agent. It represents one of the most complete implementations of an end-to-end multi-agent simulation pipeline available as open source, backed by Shanda Group and reaching 20.9k stars within months of release.

## Repository Overview

| Metric | Value |
|--------|-------|
| **GitHub** | [666ghj/MiroFish](https://github.com/666ghj/MiroFish) |
| **Stars** | 20,900+ |
| **Forks** | 2,300+ |
| **Language** | Python 57.8%, Vue 41.1% |
| **License** | AGPL-3.0 |
| **Backend** | Flask + Python 3.11-3.12 |
| **Frontend** | Vue 3 + Vite |
| **LLM Integration** | OpenAI SDK format (default: Alibaba Qwen-plus) |
| **Memory System** | Zep Cloud (GraphRAG) |
| **Simulation Engine** | CAMEL-AI OASIS (camel-oasis 0.2.5) |
| **Package Management** | uv (backend), npm (frontend) |
| **Deployment** | Docker single-container or source |

## Core Thesis

Traditional prediction approaches rely on statistical models or expert panels. MiroFish takes a fundamentally different approach: **construct a miniature digital society populated by AI agents that mirror real-world actors, let them interact freely, then observe what emerges**. The system positions this as "swarm intelligence" -- macro-level patterns arising from micro-level agent interactions. Users upload seed materials (news reports, analysis documents, even novel manuscripts), describe their prediction question in natural language, and receive both a structured report and an interactive simulation world they can probe further.

## Architecture

The system follows a **five-stage pipeline** with clear service boundaries:

### Stage 1: Graph Construction

Seed documents (PDF, Markdown, TXT) are uploaded and processed through `TextProcessor` for normalization and chunking (default 500-char chunks with 50-char overlap). The `OntologyGenerator` uses an LLM to analyze the documents and produce exactly 10 entity types and 6-10 relationship types suited for social media simulation. Entities must represent "real actors capable of speaking on social media" -- individuals, organizations, media outlets -- not abstract concepts. The `GraphBuilderService` then constructs a knowledge graph in **Zep Cloud**, sending text batches asynchronously and polling for episode completion.

### Stage 2: Environment Setup

`ZepEntityReader` reads back the constructed graph, filtering nodes to retain only those matching the ontology's custom entity types (excluding generic "Entity"/"Node" labels). Each entity is enriched with its relationship edges and connected nodes. `OasisProfileGenerator` transforms these entities into agent profiles with LLM-generated personas (up to 2000 characters each), including personality traits (MBTI type, age, gender, profession), platform-specific metrics, behavioral tendencies, and stance information. Profiles are generated in **parallel batches** with real-time saving. `SimulationConfigGenerator` produces detailed simulation parameters: agent activity levels, time-zone-aware activity patterns (following Chinese work-hour patterns), initial seed posts, scheduled events, hot topics, and platform-specific recommendation algorithm weights.

### Stage 3: Simulation Execution

The `SimulationRunner` launches OASIS simulation processes for **both Twitter and Reddit platforms in parallel** as background subprocesses. Each platform runs its own action loop where agents perform actions from a defined action space:

| Platform | Available Actions |
|----------|------------------|
| **Twitter** | CREATE_POST, LIKE_POST, REPOST, FOLLOW, DO_NOTHING, QUOTE_POST |
| **Reddit** | LIKE_POST, DISLIKE_POST, CREATE_POST, CREATE_COMMENT, LIKE_COMMENT, DISLIKE_COMMENT, SEARCH_POSTS, SEARCH_USER, TREND, REFRESH, DO_NOTHING, FOLLOW, MUTE |

Actions are logged to JSONL files per platform. The `ZepGraphMemoryUpdater` runs a background thread that monitors these action logs and **feeds agent activities back into the Zep knowledge graph** in batches of 5, with retry logic and exponential backoff. Each activity is converted to natural language descriptions (e.g., "Alice: Liked Bob's post: 'Market analysis shows...'") so Zep can extract evolving entity relationships. This creates a **feedback loop** where the knowledge graph evolves as the simulation progresses.

### Stage 4: Report Generation

The `ReportAgent` implements a **three-phase ReACT loop**: planning (outline generation), chapter-by-chapter content generation, and reflection/validation. Each chapter requires minimum 3 tool calls (max 5) before content can be written, enforcing thorough research. Four specialized tools access simulation data:

| Tool | Purpose |
|------|---------|
| **insight_forge** | Deep analysis via automatic question decomposition and multi-dimensional retrieval |
| **panorama_search** | Complete event timeline and relationship network retrieval |
| **quick_search** | Lightweight fact lookup |
| **interview_agents** | Live interviews with running OASIS agents via IPC |

Reports are saved section-by-section with real-time progress streaming to the frontend.

### Stage 5: Deep Interaction

Users can chat with the `ReportAgent` post-generation (it automatically invokes tools to synthesize answers) and directly interview individual simulation agents. The interview system uses a **file-based IPC mechanism** -- the Flask backend writes JSON command files to a shared directory, the OASIS simulation process polls for commands, executes them, and writes response files back. This avoids complex socket/RPC infrastructure but introduces polling latency (0.5s intervals, 60s timeout).

## Key Design Decisions

**Zep Cloud as the knowledge backbone**: Rather than building a custom graph database, MiroFish relies entirely on Zep's hosted GraphRAG service for entity storage, relationship tracking, and memory management. This reduces infrastructure complexity but creates a hard dependency on a third-party SaaS (free tier available but limited).

**OASIS as the simulation engine**: Instead of building custom agent interaction logic, MiroFish uses CAMEL-AI's OASIS framework (3,100+ stars, published at arXiv Nov 2024) which provides tested social media environment simulation supporting up to 1M agents. MiroFish wraps OASIS with its own process management, IPC, and memory feedback layers.

**Dual-platform parallel simulation**: Running both Twitter and Reddit simultaneously creates two "parallel worlds" with different interaction dynamics (Twitter's follow-graph vs Reddit's subreddit structure), allowing the report agent to cross-reference divergent outcomes.

**File-based IPC over sockets/RPC**: The inter-process communication between Flask and OASIS uses filesystem polling (JSON files in `ipc_commands/` and `ipc_responses/` directories). This is architecturally simple and debuggable but introduces latency and doesn't scale to high-frequency communication.

**LLM-generated ontologies constrained to 10 entity types**: Hard-capping entity types prevents ontology explosion and keeps the simulation manageable. The last two slots are always reserved for `Person` and `Organization` as fallbacks.

**Chinese-localized activity patterns**: The simulation config generator hardcodes Chinese timezone activity multipliers (peak 19:00-22:00 at 1.5x, dead hours 00:00-05:00 at 0.05x), reflecting the primary target audience.

## Codebase Structure

| Layer | File | Role |
|-------|------|------|
| **API** | `graph.py` | Project CRUD, ontology generation, graph building endpoints |
| **API** | `simulation.py` | Entity reading, simulation management, agent interviews |
| **API** | `report.py` | Report generation, chat, progress monitoring, logs |
| **Services** | `graph_builder.py` | Zep graph creation and text ingestion |
| **Services** | `ontology_generator.py` | LLM-based entity/relationship type generation |
| **Services** | `oasis_profile_generator.py` | Entity-to-agent profile transformation (~900 lines) |
| **Services** | `simulation_config_generator.py` | LLM-generated simulation parameters |
| **Services** | `simulation_manager.py` | Simulation lifecycle orchestration |
| **Services** | `simulation_runner.py` | OASIS process management and monitoring |
| **Services** | `simulation_ipc.py` | File-based IPC client/server |
| **Services** | `report_agent.py` | ReACT report generation with tool calling |
| **Services** | `text_processor.py` | Text normalization and chunking |
| **Services** | `zep_entity_reader.py` | Zep graph node filtering and enrichment |
| **Services** | `zep_graph_memory_updater.py` | Real-time simulation-to-graph feedback |
| **Services** | `zep_tools.py` | Zep utility functions |
| **Models** | `project.py` | Project state persistence |
| **Models** | `task.py` | Async task tracking |
| **Utils** | `llm_client.py` | OpenAI-format LLM wrapper |
| **Utils** | `file_parser.py` | PDF/MD/TXT extraction |
| **Utils** | `logger.py` | Logging configuration |
| **Utils** | `retry.py` | Retry utilities |
| **Utils** | `zep_paging.py` | Zep pagination helpers |
| **Scripts** | `run_parallel_simulation.py` | Dual-platform simulation launcher |
| **Scripts** | `run_twitter_simulation.py` | Twitter-only simulation |
| **Scripts** | `run_reddit_simulation.py` | Reddit-only simulation |
| **Scripts** | `action_logger.py` | JSONL action logging |
| **Scripts** | `test_profile_format.py` | Profile format validation |
| **Frontend Views** | `Home.vue` | Landing page |
| **Frontend Views** | `MainView.vue` | Primary workspace |
| **Frontend Views** | `Process.vue` | Pipeline progress visualization |
| **Frontend Views** | `SimulationView.vue` | Simulation configuration |
| **Frontend Views** | `SimulationRunView.vue` | Live simulation monitoring |
| **Frontend Views** | `ReportView.vue` | Report display |
| **Frontend Views** | `InteractionView.vue` | Agent interview interface |

## Dependency Analysis

| Dependency | Version | Role |
|------------|---------|------|
| **flask** | >= 3.0.0 | HTTP API server |
| **openai** | >= 1.0.0 | LLM client (OpenAI SDK format) |
| **zep-cloud** | 3.13.0 (pinned) | Knowledge graph and memory |
| **camel-oasis** | 0.2.5 (pinned) | Social media simulation engine |
| **camel-ai** | 0.2.78 (pinned) | Agent framework underlying OASIS |
| **PyMuPDF** | >= 1.24.0 | PDF text extraction |
| **pydantic** | >= 2.0.0 | Data validation |
| **python-dotenv** | >= 1.0.0 | Environment configuration |
| **charset-normalizer** | >= 3.0.0 | Encoding detection |

The pinned versions of `camel-oasis` and `zep-cloud` indicate tight coupling to specific API surfaces.

## Strengths

**End-to-end pipeline completeness**: Very few open-source projects connect document ingestion, knowledge graph construction, agent generation, multi-platform simulation, report synthesis, and interactive querying into a single coherent workflow. MiroFish does all of this.

**Knowledge graph feedback loop**: The `ZepGraphMemoryUpdater` feeding simulation actions back into the graph in real-time means the knowledge base evolves as agents interact, enabling the report agent to query an enriched, post-simulation state rather than just the original seed data.

**Practical agent interview system**: The ability to interview individual agents mid-simulation or post-simulation adds genuine interactive value. Users can probe specific actors' reasoning and memory, creating a dialogue-based exploration of the simulated world.

**Low infrastructure requirements**: Single Docker container, two environment variables (LLM API key + Zep key), and the system runs. No Kubernetes, no message queues, no custom databases.

**LLM-provider agnostic**: Any OpenAI SDK-compatible API works (Qwen, GPT-4, Claude, local models via compatible servers).

## Weaknesses

**Hard dependency on Zep Cloud**: The entire knowledge graph layer relies on a hosted third-party service. If Zep's API changes, rate-limits, or shuts down, the system breaks. No local graph database fallback exists.

**File-based IPC is fragile**: Using filesystem polling for inter-process communication works for demos but introduces race conditions, doesn't handle concurrent interviews well, and adds 0.5-1s latency per command. A proper message queue or Unix socket approach would be more robust.

**No validation of prediction accuracy**: The system generates predictions but provides no mechanism to evaluate whether those predictions are actually correct. There's no backtesting framework, no calibration metrics, no comparison against baselines.

**Token cost is substantial**: Each simulation round involves LLM calls for every agent's decision-making, plus ontology generation, profile generation, config generation, and report generation. The README warns about high consumption and suggests starting with fewer than 40 rounds.

**Single-threaded Flask with threading**: The backend uses Flask's built-in threaded server (`app.run(threaded=True)`) rather than a production WSGI/ASGI server. Background tasks use Python threading, not async/await or proper task queues.

**Chinese-language bias**: Most prompts, UI text, logging, and configuration defaults are in Chinese. The ontology generator's prompts, the activity description templates, and the report agent's instructions all assume Chinese-language operation.

**Large service files**: The `oasis_profile_generator.py` spans roughly 900 lines, suggesting the codebase would benefit from further decomposition.

## Relation to Agentic AI Patterns

### Agent Harnesses

MiroFish implements a **layered harness** pattern. The OASIS framework provides the low-level agent execution harness (action selection, environment interaction), while MiroFish wraps it with a higher-level orchestration harness that handles lifecycle management, memory injection, and cross-simulation coordination. The `SimulationManager` acts as the outer harness, managing state transitions (created -> preparing -> ready -> running -> completed) while delegating actual agent execution to OASIS.

### Context Management

The Zep GraphRAG integration represents a sophisticated **shared persistent context** approach. Unlike typical RAG systems where context is read-only, MiroFish's context evolves bidirectionally: seed documents populate the initial graph, simulation actions update it in real-time, and the report agent reads back the enriched state. This is closer to a **world model** pattern than a retrieval pattern.

### Tool Orchestration

The ReportAgent's four-tool setup (insight_forge, panorama_search, quick_search, interview_agents) with enforced minimum/maximum call counts per section demonstrates **constrained tool orchestration** -- the agent cannot produce content without first performing structured research. This prevents the common failure mode of agents generating plausible-sounding but ungrounded text.

### Multi-Agent Coordination

MiroFish operates at two distinct multi-agent levels:
1. **Simulation-level**: Thousands of OASIS agents coordinating through simulated social platforms (emergent coordination via environment, not direct communication)
2. **System-level**: Multiple specialized service agents (ontology generator, profile generator, config generator, report agent) coordinating through a shared knowledge graph and file-based state

The simulation agents exhibit **stigmergic coordination** -- they influence each other indirectly through the shared environment (posts, likes, follows) rather than through direct agent-to-agent messaging.

## Practical Takeaways

**GraphRAG as a living world model**: The pattern of using a knowledge graph that is both populated from seed data and continuously updated from simulation outputs is powerful. It transforms static RAG into a dynamic world state that agents can both read from and write to.

**Dual-environment simulation for robustness**: Running the same scenario across two different platform dynamics (Twitter vs Reddit) provides a form of ensemble prediction. Divergent outcomes across platforms signal uncertainty; convergent outcomes increase confidence.

**Constrained tool use in report generation**: Requiring minimum tool calls before allowing content generation is a transferable pattern for any agentic system that needs grounded outputs. It prevents the agent from "winging it" and forces evidence gathering.

**File-based IPC as a prototyping strategy**: While not production-ready, the filesystem-based command/response pattern between the Flask server and OASIS processes is remarkably simple to implement and debug. For early-stage multi-process agent systems, this approach gets something working quickly before investing in proper message infrastructure.

**Ontology constraints prevent scope explosion**: Hard-limiting entity types to 10 and relationship types to 6-10 keeps the simulation tractable. This is a useful heuristic for any system that needs to convert unstructured documents into structured agent configurations.

## Sources

- [MiroFish GitHub Repository](https://github.com/666ghj/MiroFish)
- [MiroFish English README](https://github.com/666ghj/MiroFish/blob/main/README-EN.md)
- [CAMEL-AI OASIS Repository](https://github.com/camel-ai/oasis)
- [Zep Cloud Platform](https://app.getzep.com/)
- [OASIS arXiv Paper (November 2024)](https://arxiv.org/abs/2411.11581)
- [Alibaba Qwen Model Platform](https://bailian.console.aliyun.com/)
