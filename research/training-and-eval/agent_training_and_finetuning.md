# Training and Fine-Tuning Models for Agentic Behavior (2025-2026)

This document surveys the latest techniques for training and fine-tuning language models for agentic behavior, covering reinforcement learning approaches, tool-use training, alignment methods, synthetic data, distillation, open-source models, and curriculum learning.

## 1. Reinforcement Learning for Agent Training

### RLHF (Reinforcement Learning from Human Feedback)

RLHF remains the default alignment strategy for LLMs. The standard pipeline: (1) supervised fine-tuning on demonstrations, (2) training a reward model on human preference comparisons between response pairs, (3) optimizing the policy with PPO against that reward model. The key bottleneck is the cost and speed of collecting high-quality human preference annotations.

### RLAIF (Reinforcement Learning from AI Feedback)

RLAIF replaces human annotators with an LLM that generates preference labels. Research shows RLAIF achieves comparable performance to RLHF across summarization, helpful dialogue, and harmless dialogue generation. Direct-RLAIF (d-RLAIF) skips reward model training entirely by obtaining rewards directly from an LLM during RL, achieving superior performance to canonical RLAIF.

### RLVR (Reinforcement Learning with Verifiable Rewards)

RLVR uses computational checks and code execution to produce objective, repeatable reward signals. This is particularly effective for scientific and coding agents where correctness is verifiable. NVIDIA's NeMo framework uses RLVR to train scientific agents that design experiments, evaluate outcomes, and optimize toward domain-specific metrics. A key finding: RLVR training often shows little learning early, followed by a steep learning curve as the model discovers effective strategies.

### GRPO (Group Relative Policy Optimization)

DeepSeek-R1 popularized GRPO as a lightweight alternative to PPO that eliminates the critic model entirely. Instead of training a separate value function, GRPO scores outputs by comparing individual performance against group averages. This reduces training infrastructure complexity while maintaining effectiveness. GRPO is now used across multiple frameworks including AgentRL and Agent-R1.

### DeepSeek-R1 Training Pipeline

DeepSeek-R1 demonstrated that reasoning abilities can emerge from pure RL without supervised fine-tuning. The full pipeline has four stages:

1. **Cold-start SFT**: Thousands of curated examples establish structured reasoning patterns in the base model (DeepSeek-V3-Base).
2. **Reasoning RL**: Pure RL training using GRPO develops reasoning capabilities. Emergent behaviors include self-reflection, verification, and dynamic strategy adaptation.
3. **Rejection sampling + SFT**: Near RL convergence, the best outputs are selected as synthetic training data, combined with supervised data from diverse domains (writing, QA, self-cognition).
4. **Final RL**: Another RL phase across diverse prompts ensures generalization.

For reward design, mathematical tasks receive rewards for logical consistency even without knowing the exact answer. A distilled 14B variant outperformed QwQ-32B, and distilled 32B/70B models set records on reasoning benchmarks among dense models.

### AgentRL Framework

AgentRL (from THUDM) provides a fully-asynchronous generation-training pipeline for multi-turn, multi-task agentic RL. Two key algorithmic innovations:

- **Cross-policy sampling**: Samples actions from a pool of models to enhance diversity and exploration in multi-turn settings.
- **Task advantage normalization**: Normalizes advantage estimates at the task level to stabilize training across heterogeneous tasks.

Applied to Qwen2.5 and GLM-4-9B across five agentic tasks (ALFWorld, database manipulation, knowledge graphs, OS interaction, web shopping), a single jointly-trained model matches the best performance of five separately-trained models while generalizing to unseen tasks like BFCL-v3. The framework is open-sourced at github.com/THUDM/AgentRL.

### Agent-R1

Agent-R1 extends single-turn RL to multi-turn agent interactions with a formal MDP formulation. Key design choices:

- **Advantage alignment**: Process rewards integrated into advantage calculations so credit assignment reflects intermediate successes, not just final outcomes.
- **Masked policy optimization**: Gradients computed only over agent-generated tokens, preventing credit misassignment to prompts or environment responses.
- Tested on multi-hop QA (HotpotQA, 2WikiMultihopQA, MuSiQue) using Qwen2.5-3B, all RL methods (GRPO, PPO, RLOO) achieved roughly 2.5x the performance of retrieval-augmented generation baselines. Disabling the advantage mask reduced PPO performance from 0.3719 to 0.3136 average EM.

### Evolution Strategies as an Alternative to RL

Cognizant's AI Lab introduced the first successful use of evolution strategies (ES) to fine-tune full LLM parameters at scale. ES explores by sampling perturbations in parameter space and evaluating perturbed models using outcome-only rewards -- no backpropagation or actor-critic architectures needed. Results on the Countdown task show ES consistently outperforms PPO and GRPO across 0.5B to 8B parameter models, with low variance across seeds, minimal hyperparameter sensitivity, no observable reward hacking, and no need for KL penalties. A 10x speedup was achieved in February 2026 via faster vLLM inference engines.

## 2. Tool-Use Fine-Tuning and Instruction Tuning

### ToolLLM and ToolBench

ToolLLM is the foundational framework for tool-use training. ToolBench collects 16,464 real-world RESTful APIs spanning 49 categories from RapidAPI Hub, then uses ChatGPT to generate diverse instructions and search for valid solution paths (chains of API calls). Key components:

- **DFSDT (Depth-First Search Decision Tree)**: Enhances planning and reasoning during data annotation, successfully handling complex multi-tool instructions that simpler methods (CoT, ReACT) cannot solve.
- **API Retriever**: A neural retriever that recommends appropriate APIs for each instruction, enabling open-domain tool use.
- **ToolEval**: Automatic evaluator for tool-use capabilities.

The successor, Tool-MVR (Meta-Verified, Reflection-Augmented), achieves +23.9% over ToolLLM and +15.3% over GPT-4 while reducing API call volume by 31.4%.

### Parameter-Efficient Fine-Tuning for Agents

LoRA and QLoRA remain the dominant parameter-efficient methods for adapting models to agentic tasks. LoRA injects low-rank trainable matrices into transformer layers, cutting trainable parameters dramatically. QLoRA extends this with base model quantization to reduce memory. These enable fine-tuning 70B+ models on consumer hardware.

### Reinforcement Fine-Tuning (RFT)

Launched at AWS re:Invent 2025, RFT teaches models to understand response quality without large pre-labeled datasets. Combined with Continued Pre-training (CPT) for domain-specific knowledge injection, this provides a streamlined path from foundation model to domain-specialized agent.

### Alternative Preference Optimization Methods

Beyond PPO-based RLHF, several methods have emerged:

- **DPO (Direct Preference Optimization)**: Eliminates the reward model by directly optimizing preferences.
- **ORPO (Odds Ratio Policy Optimization)**: Combines SFT and alignment in a single stage.
- **KTO (Kahneman-Tversky Optimization)**: Uses prospect theory-inspired loss functions.
- **IPO (Identity Preference Optimization)**: Incorporates identity-based preference structures.

These are computationally cheaper and more interpretable than full PPO pipelines.

## 3. Constitutional AI and Principle-Guided Agent Behavior

### Two-Phase Training

Constitutional AI (CAI), developed by Anthropic, trains aligned models using a written set of principles (the "constitution") rather than per-example human feedback.

**Phase 1 -- SL-CAI (Supervised Learning)**: The model generates responses, then critiques and revises its own outputs using sampled principles from the constitution. Through iterative refinement, a series of progressively improved completions is produced. The final revision is paired with the original prompt for supervised fine-tuning.

**Phase 2 -- RL-CAI (Reinforcement Learning)**: Pairwise preference data is generated by presenting an LLM-as-judge with two completions plus principles as context. The judge selects the better-aligned completion. This synthetic preference data trains a reward model, which then guides RL. Modern implementations use generative reward models where the judge explains its reasoning before selecting.

### Scaling Properties

CAI substitutes synthetic preference data (low-noise, high-bias) for human feedback, reducing annotation costs from roughly $1+ per preference to roughly $0.01. This made RLHF-style alignment accessible to teams without large annotation budgets. Anthropic published an updated constitution for Claude on January 21, 2026.

### Principle-Following Reward Models

Ongoing 2025-2026 research explores rubric-based reward models that evaluate outputs against explicit principle sets, enabling more granular and customizable alignment than binary preference labels.

## 4. Synthetic Data Generation for Agent Training

### Current State

By 2026, 75% of businesses are expected to use GenAI to create synthetic customer data (up from less than 5% in 2023). Gartner forecasts synthetic data will surpass real-world data for AI training by 2030.

### Generation Methods

Three primary engines produce synthetic agent training data:

1. **LLM-based generation**: Models like GPT-4, Llama, and DeepSeek generate synthetic instructions, dialogues, rationales, and tool traces using prompts, templates, and domain rules.
2. **Multimodal generative models**: Diffusion models, GANs, and video models create edited scenes and UI variants for visual agent training.
3. **Simulators and RL environments**: Produce state-action-reward sequences for control workflows and embodied agents.

### The Human-in-the-Loop Flywheel

The dominant paradigm for production synthetic data follows a four-step cycle:

1. **Curate**: Assemble high-quality human data anchored to real workflows.
2. **Generate**: Create synthetic variants at scale around known gaps.
3. **Filter**: Human reviewers rapidly accept/reject/edit candidates; every click becomes supervision feeding into RLHF.
4. **Train and validate**: Fine-tune on hybrid corpora, validate against held-out real data.

### Quality Assurance

Critical guardrails include maintaining a "Golden Corpus" of human-verified data for high-risk decisions, tagging all data by source (real vs. synthetic vs. external), validating on real-world performance (not synthetic benchmarks), and capping synthetic data proportion per use case based on domain risk.

### Reusable Synthetic Datasets

By early 2026, an ecosystem of reusable synthetic datasets has emerged: Nemotron-Synth, SYNTH, and Toucan (IBM). In 2025, major models (Minimax, Trinity, K2/K2.5, Nemotron-3) used extensive synthetic datasets during pre-training.

## 5. Distillation of Agentic Capabilities

### Reasoning Distillation

The dominant pattern in 2025-2026 is distilling reasoning chains from large teacher models into smaller student models. DeepSeek-R1 demonstrated that a distilled 14B model outperforms QwQ-32B, and distilled 32B/70B models set benchmarks among dense models. The approach captures intermediate reasoning steps and creates synthetic datasets where teachers provide both problems and structured solution paths.

### Multi-Agent Distillation (Chain-of-Agents)

Chain-of-Agents (CoAM) distills multi-agent system behavior into a single model. The process converts sophisticated multi-agent interactions into learnable chain-of-agents trajectories for "agentic supervised fine-tuning." The resulting model dynamically activates specialized sub-agents (tool agents, role-playing agents) to simulate multi-agent collaboration within a unified architecture, achieving state-of-the-art performance across search, math, and code benchmarks.

### Inference-Time Compute as an Alternative

Rather than massive parameter counts, smaller models "thinking longer" can match larger models. This means intelligence becomes a tunable parameter -- adjustable for speed versus accuracy. Google DeepMind's Gemini 3 Flash exemplifies this: distilled from Gemini Pro but enhanced with agentic RL, achieving frontier quality at reduced cost.

### LLM-to-SLM Agent Conversion

Research argues that small language models (SLMs) are sufficiently powerful, inherently more suitable, and more economical for many agentic invocations. A general LLM-to-SLM conversion algorithm enables transitioning existing agent systems to smaller models. For tasks requiring conversational abilities, heterogeneous agentic systems (agents invoking multiple different models) are recommended.

### Cost Implications

Distillation makes advanced reasoning economically viable for applications where frontier model pricing is prohibitive. Smaller distilled models run faster, making them practical for real-time agentic scenarios requiring immediate responses.

## 6. Open-Source Agentic Models

### Landscape Overview

By late 2025, five independent open model families simultaneously reached frontier quality: DeepSeek, Qwen, Kimi, GLM, and Mistral. This made the open-source surge structural rather than a one-off event.

### Key Models and Training Approaches

**DeepSeek-V3/R1**: Emphasis on optimized training pipelines to reduce compute cost. R1 demonstrated pure-RL reasoning emergence. GRPO algorithm eliminates the critic model. Open weights with full training methodology published.

**Qwen3.5-397B-A17B**: Alibaba's flagship MoE model combining multimodal reasoning with ultra-long context support. One of the most capable open models for agentic and multimodal workloads.

**GLM-4.6/GLM-5**: ~355B parameter open-source LLM from Tsinghua/THUDM emphasizing reasoning, coding, and agentic abilities. GLM-5 focuses on long-horizon agent tasks (web browsing, tool orchestration, terminal-based coding). Trained using Slime, an asynchronous RL framework that also supports Qwen3 and DeepSeek-V3.

**Mistral (MoE Architecture)**: Activates only a subset of experts per token, providing large-model capability at small-model inference speed. Function calling, web browsing, and structured outputs are built-in.

### Common Technical Patterns

- **Mixture of Experts (MoE)**: Dominant architecture for balancing capability with efficiency.
- **Scaled RL post-training**: All frontier open models use RL stages after SFT.
- **Built-in agentic capabilities**: Function calling, code execution, and structured outputs are standard features, not afterthoughts.
- **Asynchronous RL frameworks**: Slime (THUDM) and similar frameworks enable efficient large-scale RL training across model families.

## 7. Curriculum Learning and Progressive Skill Building

### E2H Reasoner (Easy-to-Hard Curriculum RL)

E2H Reasoner schedules reinforcement learning tasks from easy to hard, allowing models to build reasoning skills gradually. Tasks are decomposed into difficulty levels (trivial, easy, medium, hard), addressing two problems:

- **Distribution gap**: Large shifts between pre-training data and target tasks create sparse rewards. Intermediate difficulties provide smoother transitions.
- **Natural reward shaping**: Task decomposition breaks complex learning into steps without engineering task-specific intermediate rewards.

Two scheduling strategies stand out:

- **Cosine scheduling (E2H-C)**: Interpolates difficulty probabilities using a cosine function, performing better with dense reward landscapes.
- **Gaussian scheduling (E2H-G)**: Uses Gaussian distributions with tunable concentration and progression speed, performing better with sparse rewards.

Theoretical results prove curriculum learning requires fewer total samples than direct training when tasks are appropriately sequenced. Experiments on 1.5B-3B models across five reasoning domains show small models previously considered incapable of reasoning achieve meaningful improvements.

### WebRL: Self-Evolving Online Curriculum

WebRL generates training tasks dynamically from the agent's own failures rather than using fixed datasets. The mechanism:

1. Select instructions the model failed to complete as seeds.
2. Generate new task variants from those seeds.
3. Filter tasks using a critic that evaluates initial states, retaining only instructions scoring between 0.05 and 0.75 on difficulty (neither trivially easy nor impossible).
4. Manual verification removes infeasible instructions.

An outcome-supervised reward model (ORM) receives instruction text, action history, and final HTML state, outputting binary success/failure signals (~80% accuracy vs. ~70% for baselines). KL-divergence constraints between current and previous-phase policies prevent catastrophic drift during curriculum shifts.

Results: Llama-3.1-8B improved from 4.8% to 42.4% success rate on WebArena-Lite; GLM-4-9B from 6.1% to 43%. Both surpass GPT-4-Turbo (17.6%) and GPT-4o (13.9%) by over 160%.

### Actor-Curator: Co-Adaptive Curriculum

The Actor-Curator framework uses policy-improvement bandits to adaptively select training problems. The curator learns to choose problems that maximize the actor's improvement rate, co-adapting as the agent's capabilities change. This addresses the finding that choice, ordering, and frequency of training problems critically determine convergence speed, training stability, and generalization.

### Curriculum-Guided Multi-Agent Systems

Research on curriculum-guided massive multi-agent systems applies progressive complexity to cooperative settings, where agents first learn basic coordination before advancing to complex multi-agent scenarios.

### Key Insight: Long-Horizon Task Growth

Research from METR indicates agent task duration roughly doubles every seven months. Systems must handle 8+ hour autonomous workflows with graceful failure recovery and self-correction. This makes curriculum approaches essential -- agents that cannot handle 30-minute tasks will fail catastrophically at 8-hour tasks. Progressive training from short to long horizons is an emerging best practice.

## Emerging Trends

### Beyond Transformers for Agents

State Space Models (Mamba architecture) achieve linear-time sequence modeling, with 3B-parameter variants matching larger transformers at 5x inference throughput. Hybrid systems combining attention layers with Mamba blocks are entering production for agent workloads where long context is critical.

### Continual Learning

Google's nested learning paradigm treats single models as interconnected optimization problems at different timescales. Fast-updating modules handle immediate context; slow-updating modules preserve foundational capabilities, preventing catastrophic forgetting during continual agent deployment.

### World Models

V-JEPA 2 achieves 65-80% success on unfamiliar robotics tasks with 62 hours of training data. Real-time 3D generation (Genie 3) maintains environmental coherence at 24 FPS, providing spatial reasoning that language-only agents cannot.

### Research-to-Production Convergence

Anthropic, OpenAI, and Google DeepMind are simultaneously developing long-horizon agents. Q2 2026 is projected as the inflection point where research-grade agentic capabilities transition to reliable, deployable production systems.

## Sources

- [RLAIF vs. RLHF: Scaling Reinforcement Learning from Human Feedback with AI Feedback](https://arxiv.org/abs/2309.00267)
- [How to Train Scientific Agents with Reinforcement Learning (NVIDIA)](https://developer.nvidia.com/blog/how-to-train-scientific-agents-with-reinforcement-learning/)
- [The State of Reinforcement Learning in 2025 (Turing Post)](https://www.turingpost.com/p/stateofrl2025)
- [RLAR: An Agentic Reward System for Multi-task RL on LLMs](https://arxiv.org/html/2603.00724)
- [AgentRL: Scaling Agentic Reinforcement Learning](https://arxiv.org/abs/2510.04206)
- [Agent-R1: Training Powerful LLM Agents with End-to-End RL](https://arxiv.org/html/2511.14460v1)
- [AgentGym-RL: Training LLM Agents for Long-Horizon Decision Making](https://arxiv.org/abs/2509.08755)
- [DeepSeek-R1: Incentivizing Reasoning Capability in LLMs via RL](https://arxiv.org/abs/2501.12948)
- [DeepSeek-R1 Training Breakdown (Vellum)](https://www.vellum.ai/blog/the-training-of-deepseek-r1-and-ways-to-use-it)
- [ToolLLM: Facilitating LLMs to Master 16000+ Real-world APIs](https://arxiv.org/abs/2307.16789)
- [ToolBench (GitHub)](https://github.com/OpenBMB/ToolBench)
- [Advanced Fine-Tuning for Multi-Agent Orchestration (AWS)](https://aws.amazon.com/blogs/machine-learning/advanced-fine-tuning-techniques-for-multi-agent-orchestration-patterns-from-amazon-at-scale/)
- [Constitutional AI: Harmlessness from AI Feedback (Anthropic)](https://www.anthropic.com/research/constitutional-ai-harmlessness-from-ai-feedback)
- [Claude's Constitution (Anthropic, Jan 2026)](https://www.anthropic.com/news/claudes-constitution)
- [Constitutional AI & AI Feedback (RLHF Book)](https://rlhfbook.com/c/13-cai)
- [Synthetic Data Generation for Agentic AI (NVIDIA)](https://www.nvidia.com/en-us/use-cases/synthetic-data-generation-for-agentic-ai/)
- [AI Training in 2026: Anchoring Synthetic Data in Human Truth](https://invisibletech.ai/blog/ai-training-in-2026-anchoring-synthetic-data-in-human-truth)
- [Synthetic Pretraining (Vintage Data)](https://vintagedata.org/blog/posts/synthetic-pretraining)
- [Small Language Models are the Future of Agentic AI](https://arxiv.org/abs/2506.02153)
- [Chain-of-Agents: Multi-Agent Distillation and Agentic RL](https://openreview.net/forum?id=VcT9KJeB89)
- [AI Model Distillation 2026 Explained](https://www.aitechboss.com/ai-model-distillation-2026/)
- [The AI Research Landscape in 2026 (Adaline Labs)](https://labs.adaline.ai/p/the-ai-research-landscape-in-2026)
- [Top Open-Source LLMs 2026 (DataCamp)](https://www.datacamp.com/blog/top-open-source-llms)
- [The State of Open Source AI Models in 2025 (Red Hat)](https://developers.redhat.com/articles/2026/01/07/state-open-source-ai-models-2025)
- [Curriculum RL from Easy to Hard Tasks (E2H Reasoner)](https://arxiv.org/abs/2506.06632)
- [WebRL: Training LLM Web Agents via Self-Evolving Online Curriculum RL](https://arxiv.org/abs/2411.02337)
- [Actor-Curator: Co-adaptive Curriculum Learning](https://arxiv.org/html/2602.20532v1)
- [Evolution Strategies at Scale: LLM Fine-Tuning Beyond RL (Cognizant)](https://arxiv.org/abs/2509.24372)
- [Evolution Strategies for LLM Fine-Tuning (Cognizant Blog)](https://www.cognizant.com/us/en/ai-lab/blog/evolution-strategies-fine-tuning-llm)
- [The Landscape of Agentic RL for LLMs: A Survey](https://arxiv.org/abs/2509.02547)
- [RL Meets LLMs: A Survey of Advancements](https://arxiv.org/html/2509.16679v1)
