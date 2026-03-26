# Agent Benchmarks Deep Dive

Research date: March 2026. Comprehensive analysis of how AI agent performance is measured, what benchmark-winning patterns look like, and what this means for an agentic workflow DSL.

---

## 1. SWE-bench / SWE-bench Verified

### What It Tests

SWE-bench evaluates whether language models can resolve real-world GitHub issues from popular Python repositories. The original dataset collects 2,294 Issue-Pull Request pairs from 12 popular Python repositories spanning ML frameworks, data processing, and web frameworks (Django, Astropy, Matplotlib, scikit-learn, etc.). SWE-bench Verified is a human-validated subset of 500 tasks, curated by OpenAI in August 2024.

### Task Structure

Per task instance, an AI system receives the issue text description. The system must modify the codebase to resolve the described issue, producing a code patch. Each task executes within an isolated Docker container. The benchmark tests end-to-end bug fixing and feature implementation -- not just syntax generation but context understanding, dependency handling, and correctness validation.

### Scoring Methodology

Success is determined by running "Fail-to-Pass" unit tests against the generated patch. Models must cause previously failing tests to pass while maintaining all existing functionality (no regressions). Binary pass/fail per task, aggregated as a percentage resolution rate.

### Current Leaderboard (March 2026)

**SWE-bench Verified (self-reported scores):**
- Claude Opus 4.5: 80.9%
- Claude Opus 4.6: 80.8%
- Gemini 3.1 Pro: 80.6%
- MiniMax M2.5: 80.2%
- GPT-5.2: 80.0%
- Claude Sonnet 4.6: 79.6%
- GLM-5 (Zhipu AI): 77.8%
- Claude Sonnet 4.5: 77.2%
- Kimi K2.5: 76.8%

**Independent bash-only evaluation (standardized harness, Feb 2026 -- [Simon Willison](https://simonwillison.net/2026/Feb/19/swe-bench/)):**
- Claude Opus 4.5: 76.8% (mini-SWE-agent v2.0.0, cost $376.95)
- Gemini 3 Flash: 75.8% (mini-SWE-agent v2.0.0, cost $177.98)
- MiniMax M2.5: 75.8% (mini-SWE-agent v2.0.0, cost $36.64 -- most cost-efficient)

Notable: Chinese models dominated the independent top ten. Claude 4.5 unexpectedly outperformed the newer 4.6 by ~1 percentage point.

The independent benchmark uses "the same system prompt for every model" -- it measures model capability, not harness quality.

### Harness Design Choices That Drive Scores

The official evaluation uses a minimal **bash-tool-only agent harness**. Models get a single tool (bash) and must navigate, search, edit, and solve using standard command-line tools. This puts evaluation burden on the model.

Key harness design choices that matter:
- **Containerized Docker environments** for reproducible evaluation
- **Context management instrumentation** -- SWE Context Bench logs every file-view/code-print action at file, AST-block, and line-span granularities using Tree-Sitter parsing
- **mini-SWE-agent** (reportedly ~100 lines of Python) achieves competitive results, demonstrating that minimal harnesses can be highly effective

### Criticisms

**Training Data Contamination (the fatal flaw):** OpenAI built an automated red-teaming system using GPT-5 to probe competing models for benchmark knowledge. Results: all tested frontier models (GPT-5.2, Claude Opus 4.5, Gemini 3 Flash) could reproduce original human-written solutions or quote verbatim problem details they should never have seen. ([OpenAI, Feb 2026](https://openai.com/index/why-we-no-longer-evaluate-swe-bench-verified/))

**Flawed Test Design:** OpenAI analyzed 138 problematic tasks and found 59.4% of remaining hard problems have flawed test cases -- 49 tests too narrowly defined (rejecting functionally correct submissions) and 26 tests "too wide" (looking for extra features never mentioned in the problem description).

**Solution Leakage:** Manual review found 32.67% of model-marked "successful" cases had the answer in the issue description or comments, leading to spurious scores. ([Runloop analysis](https://runloop.ai/blog/swe-bench-deep-dive-unmasking-the-limitations-of-a-popular-benchmark))

**Insufficient Test Coverage:** 31.08% of accepted patches passed due to inadequate test suites. Filtering these out reduces apparent effectiveness from 12.47% to 3.97%.

**Narrow Scope:** Approximately half of real-world issues require multi-file changes, yet 161 of Verified's 500 tasks require only 1-2 lines of change. Only Python. Only 12 repositories.

**Memorization Evidence:** Diagnostic tasks showed up to 76% accuracy on Verified vs 53% on external repositories, suggesting memorization.

**OpenAI's Recommendation:** They now report SWE-Bench Pro and recommend others do the same. Models scoring 80% on Verified drop to ~23% on Pro.

### SWE-bench Pro

Designed to address Verified's limitations. 1,865 problems from 41 repositories spanning Python, Go, TypeScript, and JavaScript. Reference solutions average 107.4 lines across 4.1 files. Every task requires at least 10 lines of modification.

Claude Opus 4.1 drops from 22.7% to 17.8% on the private subset. GPT-5 drops from 23.1% to 14.9%. Three agent systems running identical Opus 4.5 achieved scores from 50.2% to 55.4% -- a 5.2-point variance driven entirely by how the agent manages context and tool calls.

Critical finding: "coding agents spend 60%+ of their time searching for context," with context overflow comprising 35.6% of Sonnet 4 failures.

---

## 2. Terminal-Bench / Terminal-Bench 2.0

### What It Tests

Terminal-Bench 2.0 is a rigorously curated benchmark of 89 tasks spanning diverse professional command-line domains. Published as a conference paper at ICLR 2026. Tasks require extensive domain knowledge, long chains of interdependent actions, and autonomous problem-solving. ([arXiv](https://arxiv.org/html/2601.11868v1))

### How It Differs from SWE-bench

| Dimension | SWE-bench | Terminal-Bench |
|---|---|---|
| Scope | GitHub software engineering issues | Terminal-based tasks across 10+ domains |
| Languages | Python only | Multi-language + system tools |
| Validation | Test suite pass/fail | Outcome-driven container state |
| Complexity | Often 1-2 line fixes | Hours to days for experts |
| Solution paths | Specific patches | Multiple valid approaches |

Terminal-Bench emphasizes outcome validation rather than specific solution approaches. One task (fixing the OCaml garbage collector) demanded ~10 days for junior engineers despite 24 hours for domain experts.

### Task Types and Difficulty

**Domains:** Software engineering, ML operations, system configuration, scientific computing, debugging, legacy system configuration, research paper implementation.

**Difficulty classification (dual metrics):**
- Human-predicted difficulty via contributor assessments
- Empirical difficulty from model performance: Easy (>=66.7% of frontier models solve), Medium (33.3-66.7%), Hard (<33.3%)
- Correlation between human and empirical difficulty: r=0.436 (p<<0.001)
- 93.3% of human-hard tasks proved empirically hard
- 54.5% of human-medium tasks became empirically hard -- "creative or adversarial reasoning" exceeds pattern-following

**Time estimates:** 48.6% of tasks required under one hour for experts; 71.6% required 1-24 hours for junior engineers.

### Current Leaderboard (March 2026)

**Top performers:**
1. ForgeCode + Claude Opus 4.6: 81.8% (+/-1.7%)
2. ForgeCode + GPT-5.4: 81.8% (+/-2.0%)
3. TongAgents + Gemini 3.1 Pro: 80.2% (+/-2.6%)
4. ForgeCode + Gemini 3.1 Pro: 78.4% (+/-1.8%)
5. SageAgent + GPT-5.3-Codex: 78.4% (+/-2.2%)

**Earlier results from the ICLR paper (32,155 trials):**
- GPT-5.2 + Codex CLI: 62.9% (+/-3.0%)
- Claude Opus 4.5 + Terminus 2: 57.8% (+/-2.5%)
- Gemini 3 Pro + Terminus 2: 56.9% (+/-2.5%)
- Open-weight leader Kimi K2 Thinking: 35.7% (+/-2.8%)
- GPT-5-Nano: 7.9% (dramatic drop for smaller models)

Performance range spans 81.8% down to 3.1% (GPT-OSS-20B), indicating massive agent-model compatibility variance.

### The Terminus 2 Finding

Terminus 2 was designed as a **neutral testbed** -- a minimal agent using only a single tool (headless terminal executing bash commands). Despite its simplicity:

- Gemini 2.5-Pro showed a **17% performance improvement** with Terminus 2 over OpenHands
- The minimal agent performed competitively with complex commercial harnesses
- This proved that scaffold-model interaction effects are substantial

### Error Analysis

**Trajectory-level failures:** Execution errors dominate frontier closed-source models (Opus 4.5, GPT-5.2), with coherence and verification errors at lower rates. Open-weight models show more balanced error patterns.

**Command-level errors:** Missing/inaccessible executables: 24.1% of failures. Execution errors: 9.6%.

**Key negative finding:** No correlation between episode count and success rates. Higher token generation doesn't guarantee better outcomes.

---

## 3. GAIA (General AI Assistants)

### What It Tests

GAIA evaluates general-purpose AI assistant capabilities on real-world tasks requiring reasoning, multi-modality handling, web browsing, and tool-use proficiency. 466 curated questions with unambiguous, factual answers. Proposed by Meta Research. ([arXiv](https://arxiv.org/abs/2311.12983))

### Three Difficulty Levels

**Level 1 (146 questions):** Require ~5 human steps, up to 1 tool. "Should be breakable by very good LLMs." Example: "What was the actual enrollment count of the clinical trial on H. pylori in acne vulgaris patients from Jan-May 2018 as listed on the NIH website?"

**Level 2 (245 questions):** Require 5-10 human steps, any tools. More complex reasoning and proper usage of multiple tools. Example: "According to bls.gov/cps, what was the difference in unemployment (%) in the US in June 2009 between 20+ year men and women?"

**Level 3 (75 questions):** Require up to 50 human steps, any number of tools. Long-term planning and sophisticated integration of diverse tools. "Indicates a strong jump in model capabilities."

### Evaluation Methodology

- Questions split into public validation set and private test set (300 questions)
- Answers are designed to be unambiguous and factual
- Performance measured on two dimensions: **accuracy** and **cost** (USD for total API calls)
- Individual scores reported per difficulty level

### Current Results (GAIA Leaderboard at [HAL Princeton](https://hal.cs.princeton.edu/gaia))

| System | Overall | L1 | L2 | L3 | Cost |
|---|---|---|---|---|---|
| HAL Agent + Claude Sonnet 4.5 | 74.55% | 82.07% | 72.68% | 65.39% | $178.20 |
| HAL Agent + Claude Sonnet 4.5 High | 70.91% | 77.36% | 74.42% | 46.15% | $179.86 |
| HAL Agent + Claude Opus 4.1 High | 68.48% | 71.70% | 70.93% | 53.85% | $562.24 |
| HAL Agent + Claude Opus 4 High | 64.85% | 71.70% | 67.44% | 42.31% | $665.89 |
| HAL Agent + Claude-3.7 Sonnet High | 64.24% | 67.92% | 63.95% | 57.69% | $122.49 |
| Human performance | 92% | -- | -- | -- | -- |

All top performers use the HAL Generalist Agent framework with Anthropic's Claude models. Humans still achieve 92% vs the best AI system at 74.55%.

Key observations:
- Claude Sonnet 4.5 (a mid-tier model) outperforms the more expensive Opus 4.1 -- cost does not linearly correlate with performance
- Level 3 scores are wildly inconsistent (ranging from 42% to 65%), suggesting long-horizon multi-tool tasks remain unreliable
- The HAL framework itself appears critical -- all top entries use it

---

## 4. CORE Benchmark

### What It Tests

CORE-Bench (Computational Reproducibility Agent Benchmark) tests agents' ability to reproduce scientific research results. 270 tasks based on 90 scientific papers across computer science, social science, and medicine, with Python and R codebases curated from CodeOcean.com repositories. ([arXiv](https://arxiv.org/abs/2409.11363))

### Three Difficulty Levels

**Easy:** Agents receive complete code output from successful execution. Must perform information extraction over output to answer questions.

**Medium:** Agents get Dockerfile and README instructions. Must run Docker commands and extract information from output.

**Hard:** Agents receive only the README. Must install all dependencies, determine correct commands, run code, and extract results.

### The Harness Finding That Changed the Conversation

The landmark result: **Claude Opus 4.5 scored 78% with Claude Code's harness but 42% with the generic CORE-Agent scaffold** -- a 36-point gain from switching scaffolds alone, before any model tuning.

Researcher Sayash Kapoor (Princeton, co-author of HAL): "If the same model can score double the accuracy by switching out the scaffold, it's clear the choice of scaffold matters a lot."

### Detailed Results

| Agent + Model | Easy | Medium | Hard |
|---|---|---|---|
| CORE-Agent + GPT-4o | 60.00% | 57.78% | 21.48% |
| CORE-Agent + GPT-4o-mini | 44.44% | 32.59% | 16.30% |
| AutoGPT + GPT-4o | 35.56% | 37.78% | 6.67% |

Additional findings:
- Simple prompting adjustments boosted GPT-4o-mini from 8.9% to 44.44% on Easy tasks
- Vision-based questions much harder: 87.88% on written vs 59.26% on vision (Easy level)
- Python tasks far easier than R tasks
- The 21.48% ceiling on Hard tasks shows massive room for improvement

---

## 5. Other Notable Benchmarks

### HumanEval / MBPP

**What they test:** Single-function code generation. HumanEval (164 problems, OpenAI) and MBPP (Mostly Basic Programming Problems, 974 problems, Google) test whether models can generate correct Python functions from docstrings.

**Current state:** Largely saturated. Frontier models exceed 90% on original versions. o1-mini achieves 96.2% on HumanEval. Codestral: 86.6% HumanEval, 91.2% MBPP.

**Evolution:** HumanEval Pro and MBPP Pro (ACL 2025 Findings) test self-invoking code generation -- models must solve a base problem then use its solution for a harder problem. o1-mini drops from 96.2% to 76.2% on HumanEval Pro. BigCodeBench positions itself as "the next generation of HumanEval." ([arXiv](https://arxiv.org/abs/2412.21199))

**DSL relevance:** These measure code synthesis, not agent capability. The gap between HumanEval (96%) and SWE-bench Pro (23%) reveals how much orchestration, context management, and multi-step planning matter beyond raw generation ability.

### WebArena / VisualWebArena

**What they test:** Web agent task completion in realistic environments. WebArena provides 812 tasks across containerized web applications (shopping, forums, content management, maps). VisualWebArena extends this to multimodal tasks requiring visual understanding.

**Current results:** AI agents leaped from 14% to ~60% success rate in two years. IBM CUGA reached 61.7% (Feb 2025). Gemini 2.5 Pro: 54.8% on WebArena, but only 37.8% on WebChoreArena (harder variant).

**WebArena Verified** (Jan 2026, [pip installable](https://github.com/ServiceNow/webarena-verified)) addresses original's measurement issues: audited all 812 tasks, repaired misaligned evaluations, replaced substring matching with type/normalization-aware comparators, verified backend state for state-changing tasks.

**DSL relevance:** Web navigation requires tool sequencing, state tracking across page transitions, and recovery from unexpected page states -- all workflow primitives. ([WebArena](https://webarena.dev/))

### AssistantBench

**What it tests:** Whether web agents can solve realistic, time-consuming tasks. 214 tasks across 258+ websites. Measures emergent behaviors: tool use, planning, memory, and recovery across dynamic tasks.

**Results:** No model achieved accuracy above 26%. Multi-agent coordination shows +3-4pp gains over monolithic baselines (ACP, Magentic-One). Most difficult tasks (cross-site, memory-intensive) reduce all systems to low double-digit accuracy. Scoring is fully automated. ([AssistantBench](https://assistantbench.github.io/))

### Tau-bench / Tau2-bench

**What it tests:** Customer service agent reliability. Simulates dynamic conversations between users and agents with domain-specific API tools and policy guidelines. Domains: retail, airline, telecom (tau2 adds banking).

**Key innovation -- pass^k metric:** While pass@k measures "at least one of k attempts succeeded," tau-bench introduces pass^k = p^k, measuring "all k attempts succeeded." A model with 90% pass@1 drops to 57% at pass^8. This exponential decay exposes reliability gaps invisible in single-attempt metrics.

**Results:** Even GPT-4o succeeds on <50% of tasks; pass^8 < 25% in retail. In the airline domain, GPT-4o solves only 35.2%. Key finding: "agents built on top of LM function calling lack sufficient consistency and rule-following ability to reliably build real-world applications."

**Tau2-bench evolution:** Introduces dual-control framework where both agent and user have tools -- closer to real-world scenarios like technical support where users actively participate. ([arXiv](https://arxiv.org/abs/2406.12045))

### AgentBench

**What it tests:** LLMs as agents across 8 diverse environments: Operating System, Database, Knowledge Graph, Digital Card Game, Lateral Thinking Puzzles, House-Holding, Web Shopping, Web Browsing. Published ICLR 2024.

**Results:** GPT-4 scored 4.01 vs below 1.00 for many open-source models. API-based models averaged 2.24 vs 0.42 for open-source. GPT-4 achieves 78% on House-Holding. Evaluation of 29 LLMs revealed massive performance gap between commercial and open-source (no larger than 70B). ([arXiv](https://arxiv.org/abs/2308.03688))

**Key finding:** Poor long-term reasoning, decision-making, and instruction following are the main obstacles for developing usable LLM agents.

### BrowseComp

**What it tests:** Whether AI agents can locate hard-to-find, entangled information on the internet. 1,266 questions designed by OpenAI using an inverted process: start with verifiable facts, create questions where "the answer is hard to find but easy to verify." ([OpenAI](https://openai.com/index/browsecomp/))

**Construction methodology:** Trainers performed exactly 5 Google searches per question. Questions solved by humans >40% of the time were refined further. Questions had to be unsolvable by GPT-4o and o1.

**Results:**
- GPT-4.5 (no browsing): 0.9%
- GPT-4o with browsing: 1.9%
- Deep Research (single attempt): 51.5%
- Deep Research (multiple attempts): 78%
- Human trainers: 29.2%

Basic tool access produces minimal gains (0.6% to 1.9%). Specialized agentic architecture achieves 85x improvement. Human baseline of 29.2% means even these deliberately difficult questions are feasible for humans but brutal for most AI systems.

**Key finding:** "Reasoning capability and strategic orchestration fundamentally outweigh simple tool access." Success depends on query reformulation, evidence synthesis, and persistence through dead-ends.

### DeepSearchQA

**What it tests:** 900-prompt benchmark for difficult multi-step information-seeking across 17 fields. Tests ability to execute complex search plans and generate exhaustive answer lists.

**Results:** Gemini Deep Research Agent and GPT-5 Pro High Reasoning achieve state-of-the-art with comparable fully correct rates (66.09% vs 65.18%). Gemini minimizes catastrophic failures more effectively (9.95% vs 14.13% Fully Incorrect rate).

**Key finding:** Even the most advanced models struggle to balance high recall with precision. Distinct failure modes: premature stopping (under-retrieval) vs hedging behaviors (overly wide net of low-confidence answers). ([Google DeepMind](https://storage.googleapis.com/deepmind-media/DeepSearchQA/DeepSearchQA_benchmark_paper.pdf))

---

## 6. Cross-Benchmark Analysis

### Pattern 1: Harness Design Drives Scores More Than Model Choice

This is the single most important finding across benchmarks in 2025-2026. Concrete evidence:

**CORE-Bench:** Same Opus 4.5, scaffold swap: 42% -> 78% (36-point gain)

**Terminal-Bench 2.0:** LangChain improved from 52.8% to 66.5% (13.7 points) by only tweaking the harness with GPT-5.2-Codex fixed. Moved from outside top 30 to top 5. Specific changes:
- Build-verify loop with PreCompletionChecklistMiddleware
- LocalContextMiddleware for environment discovery
- LoopDetectionMiddleware for stuck-agent recovery
- "Reasoning sandwich" -- max compute on planning/verification, moderate on implementation

**SWE-bench Verified:** Switching scaffolds makes up to 11% difference for GPT-5 and up to 15% for Kimi K2 Thinking.

**SWE-bench Pro:** Three agent systems running identical Opus 4.5 achieved 50.2% to 55.4% -- 5.2-point variance from harness alone.

**Vercel case study:** Reducing tools from 17 to 2 with same model (Claude Opus 4.5): accuracy 80% -> 100%, tokens -37%, speed 3.5x.

**Terminal-Bench 2.0:** Gemini 2.5-Pro showed 17% improvement with Terminus 2 over OpenHands.

Quote from [Evangelos Pappas](https://medium.com/@epappas/the-agent-harness-is-the-architecture-and-your-model-is-not-the-bottleneck-5ae5fd067bb2): "Past a capability threshold, improving the harness yields better returns than swapping the model."

### Pattern 2: Context Management Is the Central Design Constraint

Evidence across benchmarks:
- SWE-bench Pro: "coding agents spend 60%+ of their time searching for context"; context overflow comprises 35.6% of Sonnet 4 failures
- Terminal-Bench paper: "Context pressure emerged as the central design constraint"
- BrowseComp: 85x improvement from Deep Research's context orchestration vs basic browsing
- GAIA: Level 3 (50 steps) sees wildly inconsistent scores, suggesting long-context management breaks down

Key strategies that improve scores:
- **Adaptive Context Compaction**: Progressively reduce older observations as token budgets approach exhaustion
- **Dual-memory architecture**: Separate episodic memory (full history) from working memory (recent + active)
- **Event-driven system reminders**: Inject guidance at decision points to counteract instruction fade-out
- **Progressive disclosure**: Load instructions/tools only when contextually relevant
- **Sub-agents as context firewalls**: Prevent intermediate noise accumulating in parent thread

### Pattern 3: Failures Are Execution Failures, Not Knowledge Failures

APEX-Agents study (480 professional tasks): "Failures were predominantly not knowledge failures -- the failures were execution and orchestration problems -- agents getting lost after too many steps, looping back to failed approaches, losing track of their objectives."

Terminal-Bench error analysis: Execution errors dominate frontier models. Missing executables: 24.1%. No correlation between more tokens and better outcomes.

Tau-bench: Agents "lack sufficient consistency and rule-following ability" -- they know the rules, they just fail to reliably execute them.

### Pattern 4: Simplicity Beats Complexity in Tool Design

Three independent teams (OpenAI, Anthropic, Manus) converged on identical principles:
1. **Fewer, more general-purpose tools outperform specialized tool sets** -- Vercel cut from 17 to 2 tools, accuracy went up
2. **Too many tools push agents into "the dumb zone"** -- tool schema tokens crowd out reasoning
3. **CLI preference over MCP servers** for well-known tools (git, docker, etc.) -- saves thousands of tokens from tool definitions
4. **mini-SWE-agent** (~100 lines, bash only) achieves competitive SWE-bench results

ETH Zurich study (138 agentfiles tested): LLM-generated configuration files hurt performance while costing 20%+ more tokens. Human-written files helped only ~4%.

### Pattern 5: Reliability Degrades Exponentially with Steps

Tau-bench's pass^k metric reveals the exponential reliability problem:
- 90% per-step success -> 57% at 8 steps -> 34.9% at 10 steps
- This is why Level 3 GAIA (50 steps) shows wild variance (42-65%)
- AssistantBench: most difficult cross-site, memory-intensive tasks -> low double-digit accuracy
- Terminal-Bench: no correlation between episode count and success

### What Workflow Patterns Are Most Tested

**Well-tested patterns:**
- Single-agent + bash tool (SWE-bench, Terminal-Bench)
- Sequential tool use (GAIA, tau-bench)
- Web navigation sequences (WebArena)
- Code generation + test validation loop (SWE-bench)
- Search-synthesize-answer (BrowseComp, DeepSearchQA)

**Under-tested patterns:**
- Multi-agent coordination and handoff (only AssistantBench, tangentially)
- Error recovery and retry with backoff (tested implicitly, never isolated)
- Dynamic replanning when environment changes mid-execution
- Parallel tool execution (not measured by any major benchmark)
- Budget-aware execution (only GAIA tracks cost)
- Human-in-the-loop escalation (no benchmark tests this)
- Streaming/incremental output patterns
- Inter-agent communication protocols
- Conflict resolution between contradictory agent outputs
- Volatile environment adaptation (environment changes mid-plan)

### What Primitives a DSL Needs to Support Benchmark-Winning Patterns

Based on the cross-benchmark evidence, these are the primitives that matter:

**1. Context management primitives**
- Explicit context window/budget declarations
- Compaction policies (what to keep, what to summarize, what to drop)
- Scoped context (sub-agents get clean windows)
- Progressive disclosure (load instructions on demand)
- Working memory vs episodic memory distinction

**2. Tool interface primitives**
- Minimal tool declaration (name, description, parameters, execute)
- Schema-level restrictions per agent mode (read-only for planning, full for execution)
- Dynamic tool discovery (lazy loading to avoid prompt bloat)
- Tool result truncation policies

**3. Verification loop primitives**
- Build-verify-fix cycles as first-class constructs
- Pre-completion checklists
- Outcome validation (check final state, not intermediate steps)
- Test-gated progression

**4. Error recovery primitives**
- Loop detection with configurable thresholds
- Retry with strategy change (not just retry same approach)
- Graceful degradation / escalation paths
- Rollback capabilities (shadow state for undo)

**5. Reliability primitives**
- pass^k-style consistency requirements
- Budget caps (token, cost, time, step count)
- Multiple-attempt strategies with best-of-k selection
- Deterministic fallback chains

**6. Multi-step orchestration**
- Phase-based execution (plan -> build -> verify -> fix)
- Reasoning budget allocation per phase ("reasoning sandwich")
- Sub-agent delegation with isolated context
- Event-driven reminders to counteract instruction fade-out

**7. Observability primitives**
- Append-only event logs for reproducibility
- Trajectory recording for post-hoc analysis
- Grader interfaces (code-based, model-based, human)
- Cost/token/time accounting per step

### Evaluation-as-Code

How benchmarks express evaluation, and what a language needs:

**Three grader types** (from [Anthropic's eval guide](https://www.anthropic.com/engineering/demystifying-evals-for-ai-agents)):
- Code-based graders (fast, objective): test suites, state comparison, regex matching
- Model-based graders (flexible, nuanced): LLM-as-judge with rubrics
- Human graders (gold standard): manual transcript review

**Evaluation structural components:**
- Tasks: single tests with defined inputs and success criteria
- Trials: multiple attempts per task for non-determinism
- Transcripts: complete records of all API calls and responses
- Outcomes: final environmental state verification

**Evaluation harness frameworks:**
- Harbor: containerized agent evaluation at scale, tasks + graders as standardized format
- HAL: unified CLI for reproducible evaluation across benchmarks
- Promptfoo: declarative YAML configuration for prompt testing
- Three execution modes: expr (run agent), eval (grade outputs), e2e (both)

**What this means for lx:** Agent evaluation is a workflow itself. A DSL that can express "run agent on task, record trajectory, apply grader, aggregate across trials" has a natural advantage. The task-trial-transcript-outcome structure maps directly to workflow primitives. Graders are just agents with specific tool access (file comparison, state inspection, rubric application). Evaluation is orchestration.

---

## Summary: Key Numbers to Remember

| Benchmark | What It Measures | Top Score | Human Baseline | Gap |
|---|---|---|---|---|
| SWE-bench Verified | GitHub issue resolution | 80.9% | ~95%* | 14% |
| SWE-bench Pro | Long-horizon SE tasks | ~46% | ~95%* | 49% |
| Terminal-Bench 2.0 | Terminal tasks, 10 domains | 81.8% | expert-level | ~18% |
| GAIA | General assistant tasks | 74.55% | 92% | 17% |
| GAIA Level 3 | 50-step multi-tool tasks | 65.39% | 92% | 27% |
| CORE-Bench Hard | Scientific reproducibility | 21.48% | manual | 79% |
| WebArena | Web navigation tasks | 61.7% | ~80%+ | 18%+ |
| AssistantBench | Complex web tasks | 26% | higher | 74%+ |
| Tau-bench Retail | Customer service reliability | <50% pass@1 | ~95%+ | 45%+ |
| BrowseComp | Hard web research | 51.5% (DR) | 29.2%** | DR wins |
| HumanEval | Single function gen | 96.2% | 100% | 4% |

*Human estimates for SWE tasks
**BrowseComp was designed to be hard even for humans

The pattern: single-step code generation is solved (~96%). Single-file bug fixes are nearly solved (~81%). Multi-file, multi-step, long-horizon tasks are wide open (~23-46%). Reliability across repeated attempts is abysmal (<25% pass^8). This is where workflow orchestration -- and a DSL designed for it -- can make the difference.

---

Sources:
- [SWE-bench Leaderboard](https://www.swebench.com/)
- [SWE-bench Verified - Epoch AI](https://epoch.ai/benchmarks/swe-bench-verified)
- [Simon Willison - SWE-bench Feb 2026](https://simonwillison.net/2026/Feb/19/swe-bench/)
- [OpenAI - Why We No Longer Evaluate SWE-bench Verified](https://openai.com/index/why-we-no-longer-evaluate-swe-bench-verified/)
- [SWE-Bench Pro - morphllm](https://www.morphllm.com/swe-bench-pro)
- [Runloop - SWE-bench Limitations](https://runloop.ai/blog/swe-bench-deep-dive-unmasking-the-limitations-of-a-popular-benchmark)
- [Terminal-Bench 2.0 Leaderboard](https://www.tbench.ai/leaderboard/terminal-bench/2.0)
- [Terminal-Bench ICLR Paper](https://arxiv.org/html/2601.11868v1)
- [Terminal-Bench 2.0 Announcement](https://www.tbench.ai/news/announcement-2-0)
- [GAIA - HAL Princeton Leaderboard](https://hal.cs.princeton.edu/gaia)
- [GAIA Paper](https://arxiv.org/abs/2311.12983)
- [CORE-Bench Paper](https://arxiv.org/abs/2409.11363)
- [HumanEval Pro / MBPP Pro](https://arxiv.org/abs/2412.21199)
- [WebArena](https://webarena.dev/)
- [WebArena Verified](https://github.com/ServiceNow/webarena-verified)
- [AssistantBench](https://assistantbench.github.io/)
- [Tau-bench Paper](https://arxiv.org/abs/2406.12045)
- [AgentBench Paper](https://arxiv.org/abs/2308.03688)
- [BrowseComp - OpenAI](https://openai.com/index/browsecomp/)
- [BrowseComp - Galileo Analysis](https://galileo.ai/blog/what-is-browsecomp-openai-benchmark-web-browsing-agents)
- [DeepSearchQA - Google DeepMind](https://storage.googleapis.com/deepmind-media/DeepSearchQA/DeepSearchQA_benchmark_paper.pdf)
- [Anthropic - Demystifying Evals for AI Agents](https://www.anthropic.com/engineering/demystifying-evals-for-ai-agents)
- [HumanLayer - Skill Issue: Harness Engineering](https://www.humanlayer.dev/blog/skill-issue-harness-engineering-for-coding-agents)
- [LangChain - Improving Deep Agents with Harness Engineering](https://blog.langchain.com/improving-deep-agents-with-harness-engineering/)
- [Martin Fowler - Harness Engineering](https://martinfowler.com/articles/exploring-gen-ai/harness-engineering.html)
- [OpenAI - Harness Engineering with Codex](https://openai.com/index/harness-engineering/)
- [Building AI Coding Agents for the Terminal (arxiv)](https://arxiv.org/html/2603.05344v1)
- [The Model vs. the Harness - Adam Baitch](https://medium.com/@adambaitch/the-model-vs-the-harness-which-actually-matters-more-59dd3116bb31)
- [The Agent Harness Is the Architecture - Evangelos Pappas](https://medium.com/@epappas/the-agent-harness-is-the-architecture-and-your-model-is-not-the-bottleneck-5ae5fd067bb2)
- [Phil Schmid - Agent Harness 2026](https://www.philschmid.de/agent-harness-2026)
- [SWE-bench Pro Paper](https://arxiv.org/abs/2509.16941)
- [Dan Liden - Harness Engineering](https://www.danliden.com/posts/20260228-harness-engineering.html)
