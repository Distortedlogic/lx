# Multi-Modal and Computer-Use Agents: State of the Field (Early 2026)

## 1. Computer Use Agents

The computer use agent landscape has matured rapidly from research prototypes (late 2024) to commercially deployed systems with millions of users (early 2026). Two primary architectural patterns have emerged: browser-based agents operating in sandboxed virtual environments, and OS-level agents controlling entire desktops via mouse/keyboard simulation.

### Anthropic Claude Computer Use

Anthropic's approach is vision-based: the AI sees a virtual screen and controls it via mouse and keyboard simulation. Claude Opus 4.5 (November 2025) was positioned as the leading model for coding, agents, and computer use. Opus 4.6 (February 2026) extended the task completion horizon to 14.5 hours, the longest of any AI model at that time.

The computer use tool expanded to include `hold_key`, `left_mouse_down`, `left_mouse_up`, `scroll`, `triple_click`, and `wait` commands. Performance on the OSWorld benchmark reached approximately 14.9% success (screenshot-only), improving to 22.0% with multiple attempts, which nearly doubled the nearest competitor at 7.8%.

In February 2026, Anthropic acquired Vercept, an AI startup focused on enhancing computer use capabilities. Many of these capabilities are surfaced through Claude Cowork, a desktop application (launched January 2026 as research preview) that can interact with files on the host machine, browse the web, and run user-created plugins. Microsoft integrated Claude into Copilot Cowork, a cloud-based agent for multi-step tasks across Microsoft 365.

### OpenAI Operator / Computer-Using Agent (CUA)

OpenAI's Operator launched January 2025, powered by the Computer-Using Agent (CUA) model that combines GPT-4o's vision with advanced reinforcement learning trained to interact with GUIs. CUA operates by taking screenshots and performing mouse/keyboard actions in a remote sandboxed browser on OpenAI's servers, without requiring custom API integrations.

Benchmark performance: 38.1% on OSWorld (full computer use), 58.1% on WebArena, and 87% on WebVoyager. By July 2025, Operator was fully integrated into ChatGPT as "ChatGPT agent mode," available to Pro, Plus, and Team users. OpenAI reported approximately 32.6% success on 50-step web task benchmarks, setting the state of the art for single-agent systems.

### Google Gemini / Project Mariner

Google's Project Mariner provides multiplatform capability covering web interfaces and Android mobile apps via a screenshot-based interaction model. It demonstrated superior performance on WebVoyager, Online-Mind2Web, and custom Android task suites with lower latency than competitors. It includes "teach and repeat" functionality where users demonstrate a task once and the agent learns plans for similar future tasks.

The updated Project Mariner runs on cloud VMs, handling up to 10 tasks simultaneously. Computer use capabilities are being brought into the Gemini API and Vertex AI. In January 2026, Chrome received an "auto browse" feature for Google AI Pro/Ultra subscribers, representing the largest agentic browser deployment to date.

### Amazon Nova Act

Amazon's Nova Act is a web-focused browser automation agent integrated into AWS Bedrock. It is positioned as cost-efficient (approximately 75% cheaper than comparable alternatives) with scheduled task capability for persistent autonomous operation. It achieved 0.939 on the ScreenSpot benchmark and is integrated with the upgraded Alexa digital assistant.

### Microsoft Windows Integration

Microsoft embedded agentic features directly into Windows 11, introducing "Agent Workspace" (sandboxed environment for parallel agent operations) and adopted the Model Context Protocol (MCP) for safe app connections. "Windows 365 for Agents" provides cloud PC infrastructure for enterprise deployments, and Copilot Studio allows developers to build custom computer-use automations.

### Manus AI

A multi-agent orchestration system with specialized sub-agents for planning, web execution, and coding. Claimed approximately 86.5% success on GAIA Level 1 tasks versus OpenAI's 74.3% (human baseline approximately 92%). Reached $100M ARR within 8 months of launch.

### Key Benchmarks

| Benchmark | What It Measures | Leading Scores |
|-----------|-----------------|----------------|
| OSWorld | Full computer use via screenshots | CUA 38.1%, Claude 22% (multi-attempt) |
| WebArena | Multi-step web navigation | CUA 58.1% |
| WebVoyager | Web browsing task completion | CUA 87%, Browser-Use 89.1% |
| GAIA | General reasoning + tool use | Manus 86.5%, OpenAI 74.3% |
| Online-Mind2Web | Web navigation and form-filling | Gemini 2.5 leading |
| ScreenSpot | UI element detection | Nova Act 0.939 |

### Common Architectural Pattern

All major agents follow an iterative action loop: observe environment (screenshot) -> reason about next action -> execute action -> observe result -> repeat. Sandboxing is a critical safety pattern: agents operate in isolated cloud environments (virtual browsers, Cloud PCs, virtual desktops) with explicit permission gates for risky actions like purchases or deletions.

### Remaining Challenges

- Long-horizon error accumulation across 50+ step sequences
- CAPTCHAs and dynamic content requiring human intervention
- Cross-platform coordination beyond single-application workflows
- Trust, auditability, and detailed logging for enterprise adoption
- Cost-effectiveness boundaries for automation API expenses

## 2. Vision-Language Agents for GUI Interaction

Visual agents have moved from theoretical possibility to practical reality, representing one of the most active areas at CVPR 2025. These systems are built upon or adapted from Vision Language Models (VLMs) with the goal of perceiving visual environments like GUIs and acting within them, often called Vision-Language-Action (VLA) models.

### The Grounding Problem

The fundamental bottleneck for GUI agents is element grounding: accurately identifying and locating specific interactive elements within a GUI screenshot. Even the most powerful general VLMs significantly struggle with element grounding from visual input alone. Desktop applications are particularly challenging due to high-resolution displays (2K+) containing dense layouts and visually similar elements, which produce very long token sequences that are computationally expensive.

### Notable Models and Approaches

**ShowUI** (CVPR 2025): A VLA model built on Qwen2-VL-2B, trained on 256K instances with 2.7M element annotations across web, mobile, and desktop platforms. Achieved 75.1% accuracy in zero-shot screenshot grounding. Its "UI-Guided Visual Token Selection" reduces computational costs by 33% while maintaining detail, demonstrating that quality of data matters more than quantity.

**SpiritSight Agent** (CVPR 2025): Uses the GUI-Lasagne dataset of 5.73M hierarchically structured samples and Universal Block Parsing (UBP) technique that replaces global coordinates with block-specific coordinates for clearer element location mapping. Available in 2B, 8B, and 26B parameter sizes with cross-platform (web and mobile) capability.

**GUICourse** (ACL 2025): A series of datasets for training visual-based GUI agents using general VLMs. Enhances OCR and grounding capabilities using the GUIEnv dataset and enriches GUI knowledge using GUIAct and GUIChat datasets.

**GroundNext**: Vision-language models at 3B and 7B scales designed for precise grounding across desktop applications, trained in two stages: supervised fine-tuning on 700K curated datapoints followed by reinforcement learning.

**GUI-Actor** (Microsoft): Grounds target elements by attending to the most relevant visual regions, addressing visual grounding as the principal challenge in building VLM-powered GUI agents.

**SE-GUI**: Incorporates a reinforcement learning framework with seed data curation, dense policy gradients, and self-evolutionary reinforcement finetuning. Achieves state-of-the-art results with only 3K training samples.

**UIPro** (ICCV 2025): Focused on unleashing superior interaction capability for GUI agents.

**GEA (Generalist Embodied Agent)**: Built on LLaVA-OneVision with a multi-embodiment action tokenizer based on Residual VQ-VAE. Two-stage training (supervised fine-tuning + RL on 2.2M trajectories) achieved 90% success in CALVIN manipulation tasks.

### Training Paradigms

Current approaches emphasize hierarchical dataset construction (foundation skills to specialized capabilities), cross-domain training for generalization, reinforcement learning for error recovery, and strong pre-trained MLLM foundations as a critical starting point. The field is moving from sparse action-oriented datasets that annotate only the single relevant element per step toward comprehensive screen parsing supervision.

## 3. Web Browsing Agents and Browser Automation

Three factors converged to make browser agents viable in 2026: LLMs became capable of reasoning about web pages, browser automation infrastructure matured, and standardized protocols (MCP, A2A) enabled interoperability.

### Open-Source Frameworks

**Browser-Use**: The leading open-source project (21,000+ GitHub stars) created by Magnus Muller and Gregor Zunic. It restructures messy DOM for LLMs, strips irrelevant elements, labels interactive components, and provides control interfaces. Achieved 89.1% on WebVoyager. Supports multiple LLM providers including OpenAI, Google, Anthropic, and local models via Ollama. Has processed over 600,000 tasks in testing and released an open-source benchmark for model comparison.

**Stagehand v3** (Browserbase): An AI-native rewrite achieving 44% speed improvement with natural language commands and self-healing capabilities.

**Skyvern**: Enterprise-focused automation platform.

**AgentQL and Notte**: Query language and research agent frameworks for web interaction.

**Lightpanda**: A purpose-built headless browser written in Zig, claiming 11x faster execution than Chrome.

### Consumer Agentic Browsers

**Perplexity Comet**: Full Chromium-based browser launched July 2025 (desktop), November 2025 (Android), March 2026 (iOS). Combines search AI with form-filling and transaction completion.

**ChatGPT Atlas**: OpenAI's subscription browser with Agent Mode for multi-step autonomous tasks, launched October 2025.

**Dia** (The Browser Company): AI-first browser design, acquired by Atlassian in September 2025.

**Opera Neon**: Dedicated agentic browser with specialized agents (Neon Do, Make, ODRA). Added Intelligent Mode in February 2026 for automatic agent selection.

**Genspark**: On-device AI models (169+ open-weight models), $530M valuation, MCP Store with 700+ integrations.

### Cloud Infrastructure

Browserbase, Browserless, Steel Browser, and Hyperbrowser provide managed headless browsers as cloud services, enabling scalable agent deployment without local browser management.

### Protocol Standardization

**Model Context Protocol (MCP)**: Released by Anthropic in November 2024, donated to the Linux Foundation in December 2025. Standardizes how AI systems interface with tools. Over 97 million monthly downloads by late 2025.

**Google A2A (Agent-to-Agent Protocol)**: Establishes interoperability standards for agent-to-agent communication.

**WebMCP**: Google's early preview (February 2026) for structured agent interactions, being developed with Microsoft through W3C.

### Website Optimization for Agents

Agents work best with semantic HTML, clear labels, logical structure, accessible design, and server-rendered content. Breaking patterns include aggressive CAPTCHAs, hover-dependent interactions, infinite scroll without pagination, and heavy client-side rendering.

## 4. Multi-Modal Reasoning in Agentic Systems

### Conceptual Framework

A comprehensive survey (arXiv 2510.10991) organizes agentic Multimodal Large Language Models along three dimensions:

1. **Internal intelligence**: reasoning, reflection, and memory enabling long-horizon planning
2. **External tool invocation**: proactively leveraging external tools beyond intrinsic knowledge
3. **Environment interaction**: taking actions in virtual or physical environments, adapting strategies

This represents a transition from traditional static, passive, domain-specific AI agents toward dynamic, proactive, generalizable agentic AI.

### Microsoft's Argos Framework

Microsoft Research introduced Argos, an agentic verification framework for improving reliability of reinforcement learning in multimodal models. Rather than rewarding only correct outputs, Argos evaluates how behaviors were produced by verifying that referenced objects and events actually exist in the input and that reasoning aligns with visual evidence.

Key results: models trained with Argos show stronger spatial reasoning, substantially fewer visual hallucinations, more stable learning dynamics, and better performance on robotics and real-world tasks. Without Argos, models learned to game the system by producing plausible answers divorced from visual evidence.

### Microsoft Phi-4-reasoning-vision-15B

An open-source 15B parameter multimodal reasoning model capable of deducing the function of interface elements from screenshots and analyzing complex visual assets like scientific charts. Designed to enable developers to build AI agents that interact with applications via their user interfaces.

### Reasoning Distillation

A major trend is transferring inference-time compute capabilities to smaller models. OpenAI's o3-mini matched original o1 performance at 15x cost reduction and 5x faster speeds. By mid-2026, reasoning is expected to become a dial users adjust rather than requiring separate model categories.

### Continual Learning

Google's Nested Learning paradigm treats models as interconnected optimization problems operating at different speeds. The HOPE implementation demonstrated unbounded in-context learning without performance degradation. The Titans architecture expanded context windows beyond 2 million tokens while introducing learned long-term memory modules.

### Beyond Transformers

State Space Models like Mamba and Mamba-2 achieve linear-time sequence modeling. Mamba-3B matches transformers twice its size while delivering 5x inference throughput. Hybrid architectures combining attention with Mamba blocks represent the emerging standard.

### Long-Horizon Agent Planning

AI task duration is doubling every seven months, from one-hour tasks in early 2025 to eight-hour workstreams by late 2026. METR research shows 50% success rates on extended tasks. A key transition is moving from reasoning about tasks to reasoning within environments, demanding breakthroughs in continual learning, world models, and architectural efficiency.

### Enterprise Adoption Projections

Gartner projects 40% of enterprise applications will embed AI agents by mid-2026, but warns over 40% of these projects will be canceled by 2027 due to escalating costs. Multi-agent orchestration frameworks are replacing isolated task handlers, with agent-to-agent communication protocols becoming standardized infrastructure.

## 5. Screen Understanding and UI Element Detection

### Core Technical Challenge

Screen understanding requires models to process high-resolution screenshots (often 2K+), identify interactive elements, understand their function, and determine precise click coordinates. This is harder than it appears because GUI elements are visually dense, contextually dependent, and require understanding of spatial relationships.

### Approaches

**Coordinate-based grounding**: Models output (x, y) pixel coordinates for target elements. GroundNext and SE-GUI use this approach with supervised fine-tuning followed by reinforcement learning.

**Block-based grounding**: SpiritSight's Universal Block Parsing divides the screen into blocks and uses block-specific coordinates, reducing ambiguity in dense layouts.

**Coordinate-free grounding**: GUI-Actor attends to the most relevant visual regions without requiring explicit coordinate prediction.

**Comprehensive screen parsing**: Moving beyond sparse annotation of single elements per step to full-screen understanding. The GUI-Lasagne dataset (5.73M samples) is structured in layers that build from foundational skills to complex navigation, enabling complete screen comprehension.

### Data and Training

Quality over quantity is a consistent finding. ShowUI achieved state-of-the-art performance with 256K carefully curated instances, while SE-GUI reached top performance with just 3K training samples using reinforcement learning. Cross-platform datasets spanning web, mobile, and desktop are essential for generalization.

### InternVL3

An advanced multimodal LLM that excels in multimodal perception and reasoning, with enhanced capabilities including tool usage, GUI agents, industrial image analysis, and 3D vision perception.

## 6. Voice and Audio Agents

### Architecture Patterns

Three main architecture patterns exist for voice agents:

**Cascading pipeline**: Sequential audio -> STT -> LLM -> TTS processing. Simple but introduces 800-2000ms latency, unsuitable for real-time conversation.

**Streaming architecture**: Parallel processing where data flows continuously between components. STT transcribes incrementally, LLM begins generating as text arrives, TTS starts synthesizing before complete generation. This is the current production standard.

**End-to-end speech-to-speech**: A single model handles the entire pipeline. OpenAI's gpt-realtime model processes and generates audio directly through a single model and API, reducing latency, preserving nuance in speech, and producing more natural, expressive responses.

### Performance Targets

| Metric | Target |
|--------|--------|
| Time to First Byte (TTFB) | < 200ms |
| Total Response Time | < 1500ms |
| Word Error Rate (WER) | < 5% |
| STT Latency | 100-500ms |
| Conversation naturalness threshold | < 800ms total |

The total latency budget for natural conversation is approximately 800ms, broken down as: microphone input (40ms), transcription (300ms), LLM inference (400ms), text-to-speech (150ms).

### Key Providers and Technologies

**Speech-to-Text**: AssemblyAI Universal-Streaming (~300ms immutable transcripts at $0.15/hour), Deepgram (deployed across 330+ Cloudflare cities).

**Text-to-Speech**: Cartesia (ultra-low latency), ElevenLabs (voice customization), Rime (emotional expression focus). Production quality requires Mean Opinion Score (MOS) above 4.0.

**LLMs for voice**: Smaller models like Gemini 2.5 Flash-Lite and Claude 4.5 Haiku for low-latency interactions; larger models like Gemini 3 Pro and GPT-5.2 for complex reasoning. Time to First Token optimization is critical.

**Orchestration platforms**: Vapi, LiveKit Agents, Daily/Pipecat for development; Bland, Retell, Synthflow as all-in-one solutions.

**Infrastructure**: Cloudflare Realtime Agents platform uses WebRTC connections to nearest datacenter (330+ locations), with composable AI pipeline components and Durable Objects for state management.

### Emerging Capabilities

- Emotion recognition in speech with adaptive delivery
- Real-time translation across multiple languages
- Multimodal agents spanning text, GUI actions, and voice interactions (e.g., Genspark)
- The speech recognition technology market is projected to reach $29.28 billion by 2026

## 7. Robotics and Embodied Agents Using Foundation Models

### GEN-0 (Generalist AI, November 2025)

A new class of embodied foundation models built for multimodal training directly on high-fidelity raw physical interaction. Key characteristics:

- **Harmonic Reasoning**: Enables simultaneous thinking and acting by training on asynchronous, continuous-time streams of sensing and acting tokens, eliminating the need for explicit System 1/System 2 architectures
- **Training scale**: Over 270,000 hours of real-world manipulation data, growing at 10,000 hours per week, collected from homes, warehouses, bakeries, laundromats, and factories
- **Scaling laws**: Predictable power-law relationships between pretraining data and performance. 1B models ossify under data overload; 6B models show multi-task improvement; 7B+ models internalize large-scale data and transfer to downstream tasks
- **Cross-embodiment**: Works across 6DoF, 7DoF, and 16+ DoF semi-humanoid robots without redesign
- **Peak performance**: Up to 99% success when combining large-scale pretraining with task-specific fine-tuning

### World Models

**V-JEPA 2** (LeCun/Advanced Machine Intelligence Labs): Achieved 65-80% success on robotics pick-and-place tasks with unfamiliar objects, supporting the argument that language-only systems are a dead end for general intelligence lacking physical grounding.

**Genie 3**: Real-time 3D environment generation at 24fps maintaining consistency for minutes.

**Marble**: First commercially available world model generating persistent 3D environments from text and images.

### Foundation Model Approaches for Robotics

Large Language Models support embodied agents for goal interpretation, subgoal decomposition, action sequencing, and transition modeling. The field advances through three model families:

1. **Large Language Models (LLMs)**: High-level planning and natural language instruction parsing
2. **Vision-Language Models (VLMs)**: Scene understanding and visual reasoning
3. **Vision-Language-Action Models (VLAs)**: End-to-end perception-to-action, including Google DeepMind RT-X (trained on 13M+ trajectories from 22 robot embodiments) and OpenAI VPT-R

### Embodied Intelligence Framework

A three-layer architecture is emerging: multimodal perception, world modeling, and structured strategies. Recent breakthroughs in Multimodal Large Models and World Models are providing tools for semantic understanding and robust generalization.

### Current Challenges

- Translating natural language instructions into executable robot actions
- Multimodal perception in human-centered environments
- Uncertainty estimation for safe decision-making
- Computational constraints for real-time onboard deployment
- Moving from foundation models to embodied agents requires understanding lower-level visual details and long-horizon reasoning

### Investment and Growth

VC investment in robotics reached $7.2B in 2025, up from $3.1B in 2023, directed toward humanoids, foundation models, and autonomous manufacturing. Embodied AI research in healthcare saw publications in 2024 nearly sevenfold higher than in 2019. A CVPR 2025/2026 workshop series on "Foundation Models Meet Embodied Agents" reflects the research community's focus.

## Cross-Cutting Themes

### Multi-Agent Orchestration
Organizations are transitioning from single all-purpose agents to coordinated teams of specialized agents. Orchestrator agents coordinate specialists (researcher, coder, analyst agents), mirroring human team structures. This is described as the "microservices moment" for AI.

### Protocol Standardization
MCP (Anthropic, now Linux Foundation) and A2A (Google) are establishing the foundational protocols for the agent ecosystem, analogous to HTTP for the web. WebMCP (Google + Microsoft via W3C) extends this to structured web interactions.

### The System-Level Intelligence Paradigm
Rather than individual model breakthroughs, 2026 emphasizes integration: systems that reason deeply, learn continuously, and deploy efficiently. The research question has evolved from "how large can we scale?" to "how intelligently can we operate within constraints?"

### Reinforcement Learning as Universal Improvement
RL is applied across every domain covered here: GUI grounding (SE-GUI, GroundNext), computer use (CUA, Argos), robotics (GEN-0, GEA), and multimodal reasoning. It consistently improves action selection, error recovery, and visual grounding beyond supervised fine-tuning alone.

## Sources

- https://o-mega.ai/articles/agentic-computer-use-the-ultimate-deep-guide-2026
- https://openai.com/index/introducing-operator/
- https://openai.com/index/computer-using-agent/
- https://openai.com/index/introducing-chatgpt-agent/
- https://openai.com/index/introducing-gpt-realtime/
- https://www.anthropic.com/news/claude-opus-4-5
- https://platform.claude.com/docs/en/release-notes/overview
- https://siliconangle.com/2026/02/25/anthropic-acquires-ai-startup-vercept-enhance-claudes-computer-use-features/
- https://blog.google/innovation-and-ai/models-and-research/google-deepmind/gemini-universal-ai-assistant/
- https://techcrunch.com/2025/05/20/google-rolls-out-project-mariner-its-web-browsing-ai-agent/
- https://www.nohackspod.com/blog/agentic-browser-landscape-2026
- https://github.com/browser-use/browser-use
- https://www.firecrawl.dev/blog/best-browser-agents
- https://brightdata.com/blog/ai/best-agent-browsers
- https://voxel51.com/blog/visual-agents-at-cvpr-2025
- https://openaccess.thecvf.com/content/CVPR2025/papers/Lin_ShowUI_One_Vision-Language-Action_Model_for_GUI_Visual_Agent_CVPR_2025_paper.pdf
- https://aclanthology.org/2025.acl-long.1065/
- https://microsoft.github.io/GUI-Actor/
- https://openreview.net/forum?id=IbzDaIDyt6
- https://arxiv.org/abs/2510.10991
- https://arxiv.org/html/2602.14276v1
- https://arxiv.org/html/2511.07332v1
- https://www.microsoft.com/en-us/research/blog/multimodal-reinforcement-learning-with-agentic-verifier-for-ai-agents/
- https://siliconangle.com/2026/03/04/microsoft-open-sources-multimodal-reasoning-model-15b-parameters/
- https://labs.adaline.ai/p/the-ai-research-landscape-in-2026
- https://machinelearningmastery.com/7-agentic-ai-trends-to-watch-in-2026/
- https://www.assemblyai.com/blog/ai-voice-agents
- https://www.assemblyai.com/blog/the-voice-ai-stack-for-building-agents
- https://blog.cloudflare.com/cloudflare-realtime-voice-ai/
- https://generalistai.com/blog/nov-04-2025-GEN-0
- https://foundation-models-meet-embodied-agents.github.io/cvpr2025/
- https://www.frontiersin.org/journals/robotics-and-ai/articles/10.3389/frobt.2025.1668910/full
- https://github.com/showlab/Awesome-GUI-Agent
