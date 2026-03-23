# REF: Towards a Science of Scaling Agent Systems (Google Research / MIT)

Full extraction from https://arxiv.org/abs/2512.08296
Blog post: https://research.google/blog/towards-a-science-of-scaling-agent-systems-when-and-why-agent-systems-work/
Published December 2025 (blog January 28, 2026)

Authors: Yubin Kim (Google Research, MIT), Ken Gu (Google Research), Chanwoo Park (MIT), Chunjong Park (Google DeepMind), Samuel Schmidgall (Google DeepMind), A. Ali Heydari, Yao Yan, Zhihan Zhang, Yuchen Zhuang (Google DeepMind), Mark Malhotra, Paul Pu Liang (MIT), Hae Won Park (MIT), Yuzhe Yang, Xuhai Xu, Yilun Du, Shwetak Patel, Tim Althoff, Daniel McDuff, Xin Liu (Google Research)

## Core Thesis

Multi-agent systems do not provide universal benefits. Performance ranges from +81% improvement to -70% degradation depending on task structure and architecture alignment. Through 180 controlled configurations and 14,742 total instance runs, the study derives the first quantitative scaling principles for agent systems.

## Five Architectures Tested

**Single-Agent System (SAS):** One reasoning locus, sequential execution. Complexity O(k). No communication overhead. Success rate: 0.466.

**Independent MAS:** Parallel sub-task execution, no inter-agent communication, results aggregated at end. Success rate: 0.370.

**Centralized MAS:** Hub-and-spoke with orchestrator delegating to workers. Success rate: 0.463.

**Decentralized MAS:** Peer-to-peer mesh with direct agent-to-agent information sharing. Success rate: 0.477.

**Hybrid MAS:** Combines hierarchical oversight with peer-to-peer coordination. Success rate: 0.452.

## Four Benchmarks

| Benchmark | Domain | Complexity Score | Key Property |
|-----------|--------|-----------------|--------------|
| BrowseComp-Plus | Web navigation/retrieval | D=0.839 | Dynamic state evolution |
| Finance-Agent | Financial analysis (SEC filings, revenue, cash flow) | D=0.407 | Parallelizable subtasks |
| PlanCraft | Minecraft environment planning | D=0.419 | Strict sequential dependencies |
| Workbench | Business tasks (email, scheduling, project mgmt) | D=0.000 | Minimal sequential constraints |

Performance variability (coefficient of variation): BrowseComp-Plus 0.32 (highest), PlanCraft 0.21, Finance-Agent 0.18, Workbench 0.12 (most stable).

## Model Families Tested

- OpenAI: GPT-5-nano, GPT-5-mini, GPT-5
- Google: Gemini 2.0 Flash, 2.5 Flash, 2.5 Pro
- Anthropic: Claude Sonnet 3.7, 4.0, 4.5

Intelligence Index range: 34-66 (composite of reasoning, coding, knowledge). Anchors: Gemini-2.0-Flash (34) to GPT-5 (66). Cross-family consistency: maximum scaling slope difference 0.023, CV < 0.02.

## Headline Results by Domain

**Finance-Agent (parallelizable tasks) -- multi-agent wins big:**
- SAS baseline: 0.349
- Centralized: +80.9% (0.631)
- Decentralized: +74.5% (0.609)
- Hybrid: +73.2% (0.604)

**PlanCraft (sequential tasks) -- multi-agent destroys performance:**
- SAS baseline: 0.568
- Independent: -70.0% (0.170)
- Centralized: -50.4% (0.282)
- Decentralized: -41.4% (0.332)
- Hybrid: -39.0% (0.346)

**BrowseComp-Plus (web navigation):**
- Independent: -35% relative to SAS
- Decentralized: +9.2% (0.347 vs SAS 0.318)
- Centralized: +0.2% (essentially flat)

**Workbench (tool-heavy business tasks):**
- Decentralized: +5.7% (0.664)
- Centralized: -1.2%
- Hybrid: -1.2%

**Aggregate across all benchmarks:** Mean MAS improvement -3.5% (95% CI: [-18.6%, +25.7%]), standard deviation 45.2%.

## Coordination Overhead and Efficiency

| Metric | SAS | Independent | Decentralized | Centralized | Hybrid |
|--------|-----|-------------|---------------|-------------|--------|
| Turns | 7.2 +/- 2.1 | 11.4 +/- 3.2 | 26.1 +/- 7.5 | 27.7 +/- 8.1 | 44.3 +/- 12.4 |
| Overhead (O%) | 0 | 58 | 263 | 285 | 515 |
| Message Density | 0.00 | 0.00 | 0.41 | 0.39 | 0.24 |
| Redundancy (R) | 0.00 | 0.48 +/- 0.09 | 0.50 +/- 0.06 | 0.41 +/- 0.06 | 0.46 +/- 0.04 |
| Efficiency (Ec) | 0.466 | 0.234 | 0.132 | 0.120 | 0.074 |
| Error Amplification (Ae) | 1.0 | 17.2 | 7.8 | 4.4 | 5.1 |
| Success/1K tokens | 67.7 | 42.4 | 23.9 | 21.5 | 13.6 |

Hybrid requires 6.2x more reasoning turns than SAS (44.3 vs 7.2; t(178)=16.8, p<0.001).

## Turn Scaling Power Law

T = 2.72 * (n + 0.5)^1.724, R^2 = 0.974, exponent 95% CI [1.685, 1.763], p < 0.001. Super-linear growth with agent count.

## Error Amplification Analysis

Error amplification factors (relative to SAS baseline of 1.0):
- Independent: 17.2x (95% CI: [14.3, 20.1]) -- catastrophic, no correction mechanism
- Decentralized: 7.8x
- Hybrid: 5.1x
- Centralized: 4.4x (95% CI: [3.8, 5.0]) -- orchestrator acts as validation checkpoint

Error absorption by type (centralized vs baseline):
- Context Omission: 66.8% reduction (from 15.8-25.2% to 8.3%)
- Logical Contradiction: 36.4% reduction (from 12.3-18.7% to 9.1%)
- Numerical Drift: 24% reduction (from 20.9-24.1% to 18.3%)

Coordination-specific failures: Independent 0%, Centralized 1.8%, Decentralized 3.2%, Hybrid 12.4%.

Average error correction rate: 22.7% (95% CI: [20.1%, 25.3%]). Peak: 31.4% on Finance-Agent. Independent MAS shows +4.6% error amplification instead of correction.

## The Five Scaling Principles

### Principle 1: Tool-Coordination Trade-off
Beta = -0.330 (95% CI: [-0.432, -0.228], p < 0.001). Tool-rich environments amplify coordination tax by 6.3x. For 16-tool tasks: SAS efficiency penalty -2.46, MAS efficiency penalty -0.39.

### Principle 2: Capability Saturation (Baseline Paradox)
Beta = -0.408 (95% CI: [-0.564, -0.251], p < 0.001). Critical threshold: PSA* ~ 0.45. Tasks where a single agent already achieves >45% success rate get negative returns from adding agents. Strongest negative predictor in the model.

### Principle 3: Topology-Dependent Error Amplification
Hierarchy: Single (1.0x) < Centralized (4.4x) < Hybrid (5.1x) < Decentralized (7.8x) < Independent (17.2x). Absence of validation mechanisms allows unchecked error propagation.

### Principle 4: Overhead Non-Linearity
Beta = -0.141 (95% CI: [-0.213, -0.069], p < 0.001). Critical overhead threshold: O%max(T) ~ 150% for T=16 tools. This rules out all MAS except potentially Decentralized at high tool counts.

### Principle 5: Redundancy at Scale
Beta = 0.041 (95% CI: [0.002, 0.081], p = 0.040). For 4-agent system with R=0.50: ~8% boost. Marginal compared to efficiency losses (8x smaller than the Ec*T interaction).

## Predictive Model

Mixed-effects model achieving R^2_CV = 0.513 (5-fold cross-validation with experiment-level holdout). MAE: 0.089 +/- 0.011. RMSE: 0.112 +/- 0.014.

Correctly identifies optimal architecture for 87% of unseen task configurations (vs 20% random baseline, 54% capability-only models).

Model equation: P = B0 + B1*I + B2*I^2 + B3*log(1+T) + B4*log(1+na) + B5*log(1+O%) + B6*c + B7*R + B8*Ec + B9*log(1+Ae) + B10*PSA + interaction terms + epsilon

Three dominant factors: tool-coordination trade-off (Beta=-0.330), capability saturation (Beta=-0.408), error amplification topology.

Key coefficients:
- Intelligence quadratic (I^2): 0.256 (p=0.010) -- accelerating returns to model capability
- log(1+T) tool diversity: 0.535 (p<0.001) -- strongest positive main effect
- PSA baseline proxy: 0.319 (p<0.001)
- PSA * log(1+na): -0.408 (p<0.001) -- baseline paradox, strongest negative interaction
- Ec * T: -0.330 (p<0.001) -- tool-coordination trade-off
- O% * T: -0.141 (p<0.001) -- overhead scales with task complexity

Leave-one-domain-out CV: R^2 = 0.89 (strong cross-domain generalization).

## Model Intelligence Scaling

Top-quartile models (Intelligence Index > 60): 23% performance advantage vs linear prediction due to quadratic term. Marginal benefit: dP/dI = -0.180 + 0.512*I. More capable models benefit disproportionately.

Sonnet upgrade > doubling token budget on less capable model (consistent with Anthropic findings).

## Heterogeneous Agent Performance

Mixing capability levels across agents:

BrowseComp-Plus results:
- Anthropic: Low-capability orchestrator + high-capability subagents (Centralized): 0.42 vs homogeneous 0.32 (+31% from heterogeneity)
- OpenAI: Heterogeneous underperforms homogeneous
- Decentralized mixed-capability: OpenAI 0.53 vs homogeneous 0.50; Anthropic 0.47 vs 0.37; Gemini 0.42 vs 0.43

Decentralized enables emergent collaboration despite capability asymmetry.

## Agent Count Scaling

Optimal agent count depends on both model capacity and coordination strategy. Gemini-2.0 Flash peaks at 7 agents then degrades. Gemini-2.5 Pro shows diminishing returns beyond 5 agents. Centralized architecture scales more stably than decentralized.

## Family-Specific Patterns

Finance-Agent by family (centralized):
- Google: +164.3% (0.740 vs 0.280 SAS)
- Anthropic: +127.5% (0.636 vs 0.280 SAS)
- OpenAI: +71.2% (0.79 vs 0.465 SAS)

Workbench efficiency: Anthropic MAS-Decentralized +10.8% (highest), Google +9.5%, OpenAI +8.6%.

PlanCraft degradation: All families degrade. Anthropic worst at -54.5% (Hybrid), Google best at -25.3%.

## Architecture Selection Decision Rules

Type 1 -- Planning Tasks (T=4, PSA=0.57): Use single-agent. Baseline paradox dominates, low tool count.

Type 2 -- Analysis Tasks (T=5, PSA=0.35): Use centralized multi-agent. Moderate efficiency penalties balanced by error control (Ae=4.4).

Type 3 -- Tool-Heavy Tasks (T=16, PSA=0.63): Use decentralized multi-agent. Despite 263% overhead, parallelization and redundancy outweigh efficiency losses.

## Message Density Saturation

S = 0.73 + 0.28*ln(c), R^2 = 0.68, p < 0.001. Saturation at c* = 0.39 messages/turn.

Information gain correlations: Finance-Agent r=0.71 (p<0.001, strong), PlanCraft r=0.18 (p=0.22, non-significant). Sequential interdependence, not complexity alone, determines coordination viability.

Token overlap: successful runs 2.3% contradictory mass vs failures 8.1%.

## Practical Recommendations

1. Use single-agent for sequential reasoning, high baseline performance (>45%), or planning tasks
2. Use centralized coordination for structured analysis, error-sensitive domains, moderate tool complexity
3. Use decentralized coordination for parallelizable tasks, tool-heavy environments, high-entropy search spaces
4. Never use independent multi-agent -- error amplification (17.2x) overwhelms any diversity benefits
5. Use the scaling equation with five measurable inputs (I, T, na, O%, PSA) to predict optimal architecture with 87% accuracy
6. Model capability matters more than agent count -- upgrading the model beats adding agents
7. Start with single agents; switch to multi-agent only when tasks divide into independent pieces AND single-agent success remains below 45%

## Statistical Rigor

- 14,742 total instance runs across 180 configurations
- Token budgets matched across all systems (mean 4,800 per trial)
- Shapiro-Wilk normality: p=0.412
- Breusch-Pagan homoscedasticity: p=0.298
- All VIF < 5 (no severe multicollinearity)
- Bootstrap stability (n=1000): mean SE < 0.015 for all |Beta| > 0.1
- Lasso R^2_CV=0.506 (16/20 predictors retained), Ridge R^2_CV=0.509, Full model R^2_CV=0.513
- Inter-rater agreement (Cohen's kappa): Finance 0.91, Workbench 0.89, BrowseComp 0.88, PlanCraft 0.87

---

# REF: Are More LLM Calls All You Need? (Stanford / UC Berkeley / Princeton)

Full extraction from https://arxiv.org/abs/2403.02419
Published March 2024, presented at NeurIPS 2024

Authors: Lingjiao Chen, Jared Quincy Davis (Stanford), Boris Hanin (Princeton), Peter Bailis (Stanford), Ion Stoica, Matei Zaharia (UC Berkeley), James Zou (Stanford)

## Core Thesis

Compound systems using multiple LLM calls with majority voting exhibit non-monotonic scaling: performance first increases then decreases as calls increase. This contradicts the assumption that more compute always helps.

## Key Finding: Non-Monotonic Scaling

Across multiple language tasks, Vote and Filter-Vote systems' performance first increases then decreases as a function of LLM call count K. The non-monotonicity is driven by query difficulty diversity: more calls help "easy" queries but hurt "hard" queries.

## Theoretical Framework

Query difficulty indicator d(x):
- Easy queries (d(x) < 0): performance approaches 1.0 as K grows
- Hard queries (d(x) > 0): performance approaches 0.0 as K grows

Four performance patterns depending on dataset composition:
1. Monotonic increase: when p1+p2 > 1 AND alpha >= 1-1/t (mostly easy queries, individually-correct bias)
2. Monotonic decrease: when p1+p2 < 1 OR alpha <= 1-1/t
3. Inverse U-shape: p1+p2 > 1 AND alpha < 1-1/t (rises then falls -- the surprising case)
4. U-shape: p1+p2 < 1 AND alpha > 1-1/t (falls then rises)

Where alpha = fraction of easy queries, p1 = P(correct|easy), p2 = P(correct|hard), t = threshold function of p1 and p2.

Optimal K formula (for inverse U-shape): K* = 2 * [log(alpha/(1-alpha)) * (2p1-1)/(1-2p2)] / [log(p2(1-p2)/(p1(1-p1)))]

## Systems Analyzed

**Vote:** Sample K independent generations, return majority-voted answer.
**Filter-Vote:** Generate K candidates with explanations, apply LLM filter, majority vote among filtered (or all if none pass).

## Benchmarks

| Dataset | Type | Task |
|---------|------|------|
| MMLU Physics | Real | Multiple-choice physics |
| TruthfulQA | Real | Truthfulness evaluation |
| GPQA | Real | Expert-level Q&A (biology, physics, chemistry) |
| AVERITEC | Real | Fact verification (500 claims) |
| Synthetic D_{alpha,p1,p2} | Controlled | Variable difficulty |

All experiments: GPT-3.5-turbo-0125, 1000 runs per configuration.

## Key Experimental Results

AVERITEC: Both Vote and Filter-Vote show inverse U-shape across K in [2, 1000]. Easy queries (d<0) show monotonic improvement, hard queries (d>0) show monotonic degradation. Optimal K exists and is predictable.

Synthetic validation: (p1,p2)=(0.85, 0.4) at alpha=0.4 produces U-shape. (p1,p2)=(0.85, 0.1) at alpha=0.6 produces monotonic increase. Changing alpha from 0.6 to 0.4 reverses monotonic increase to U-shape.

Theorem 4 predictions for optimal K matched empirical observations exactly across all configurations.

## Practical Implications

When more calls help: Predominantly easy datasets (high alpha), individually-correct bias (p1+p2 > 1).

When more calls hurt: Predominantly hard datasets (low alpha), individual error dominance (p1+p2 < 1), mixed difficulty where hard queries are sufficiently numerous.

Optimization: Fit the analytical model G(K,D) on small query sample to identify optimal K* without exhaustive search.

## Limitations

Restricted to tasks with small answer spaces supporting majority voting. Only objective language tasks tested. Subjective tasks unexplored. Cost/latency tradeoffs not analyzed.

## Source

https://arxiv.org/abs/2403.02419
