# Agent Observability, Feedback Loops, and Self-Correction Patterns

## 1. Trace Architecture for Agents

OpenTelemetry's GenAI semantic conventions define a hierarchical span structure for agent systems. The two primary span types are `create_agent` and `invoke_agent`, with span names formatted as `invoke_agent {gen_ai.agent.name}`. Span kind is CLIENT for remote agents (OpenAI Assistants, AWS Bedrock) and INTERNAL for in-process agents (LangChain, CrewAI).

Required attributes per span: `gen_ai.operation.name`, `gen_ai.provider.name`. Recommended attributes include `gen_ai.conversation.id` (session correlation across sub-agents), `gen_ai.usage.input_tokens`, `gen_ai.usage.output_tokens`, `gen_ai.request.model`, `gen_ai.response.model`, and `gen_ai.agent.id`. Token counts attach at the span level, enabling per-step cost attribution.

For multi-agent correlation, `gen_ai.conversation.id` threads through parent and child spans. Each tool invocation gets its own `execute_tool` child span. A proposal (semantic-conventions #2664) extends this to cover tasks, actions, teams, artifacts, and memory for complex agentic workflows.

Industry adoption is high: 89% of organizations have agent observability, 71.5% have full tracing (LangChain State of Agent Engineering survey, n=1,340, Dec 2025).

## 2. Self-Verification Patterns

Verification is the single highest-leverage capability for autonomous systems. The core insight: "The quality of an autonomous system is determined not by its generation capabilities but by its verification capabilities." Google's 2025 DORA Report found 90% AI adoption increase correlates with 91% increase in code review time -- more generation without verification creates negative net value.

**Calibrated Confidence Scoring**: Every verification produces a 0.0-1.0 score. A score of 0.95 means thorough checking with no issues found; 0.6 means partial verification with unknowns. Scores propagate to downstream decision-making systems.

**Four-Verdict System**: ACCEPT (commit), ACCEPT_WITH_WARNINGS (proceed with logging), REJECT (requires revision), PARTIAL (accept passing, revise failures). Single critical issues force rejection -- false negatives are costlier than false positives.

**Trajectory Analysis (TrajAD)**: Evaluates improvement cycles holistically, detecting when fixes create new instances of the same problem pattern rather than addressing root causes.

**Feedback Loop Closure**: Agents configured to self-verify ("Run tests yourself. If tests fail, read output, fix, re-run") eliminate the manual bottleneck of Agent-Human-Terminal-Human-Agent. Reflection loops show diminishing returns after 2-3 iterations for code generation, though analytical tasks sometimes benefit from a fourth pass.

## 3. Doom Loop Detection

Analysis of 220 agent loops found 55% productive, 45% problematic -- nearly half exhibited unresolved issues despite agents claiming progress. Only 6 of 12 automated responses successfully reduced their target signal rates (50% fix effectiveness).

**Detection Thresholds**: Monitor last 3 actions for repetition. Hard caps at 25 agent turns OR 300 seconds, whichever comes first. Use embedding similarity or Jaccard similarity for semantically equivalent (not just identical) actions. One detection mechanism generated 13x more signals than it suppressed, creating amplification instead of resolution.

**Circuit Breaker Pattern**: External enforcement where the system -- not the agent -- controls termination. Components: hard iteration caps, output similarity detection, token budget exhaustion tracking. The agent's self-assessment cannot be trusted; mechanical pattern counts regularly contradict agent self-reports.

**Gear Reduction**: When a step exceeds the model's distribution, switch to read-only planning mode and decompose into simpler sub-steps. This avoids both premature termination and infinite retry.

**Loop Classification**: Each iteration is tagged as productive, stagnating, stuck, failing, or recovering based on signal distribution, enabling automated routing to appropriate intervention.

## 4. Behavioral Drift Monitoring

The Agent Stability Index (ASI) quantifies drift across 12 dimensions in four weighted categories: Response Consistency (30%), Tool Usage Patterns (25%), Inter-Agent Coordination (25%), Behavioral Boundaries (20%). Alert threshold: ASI below 0.75 for three consecutive measurement windows.

Impact of unchecked drift (ASI <0.70): 42% task success rate decline, 24.9% accuracy reduction, 63.2% completion time increase, 216.1% human intervention surge. Detectable drift emerges after median 73 interactions, accelerating thereafter.

Three drift types: **Semantic** (outputs deviate from intent while remaining syntactically valid), **Coordination** (multi-agent consensus degrades), **Behavioral** (unintended strategies emerge). Combined mitigation (Adaptive Behavioral Anchoring + Episodic Memory Consolidation + Drift-Aware Routing) achieves 81.5% drift reduction.

The key finding: changes are individually minor and imperceptible in isolated evaluations, yet collectively degrade performance by double-digit percentages. Offline eval datasets only validate pre-deployment; running evals against live production traffic is the only way to catch drift in real time. Segment-level breakdowns by intent type, user segment, or workflow catch regressions invisible in aggregate metrics.

## 5. Quality Metrics for Agents

**pass@k vs pass^k**: At k=1 they are identical. At k=10 they tell opposite stories. pass@k (probability at least one of k attempts succeeds) approaches 100%; pass^k (probability all k attempts succeed) approaches 0%. Example: 75% per-trial success, k=3 -- pass@3 is ~97%, pass^3 is 34.3%. Use pass@k when you can check outputs pre-deployment (coding with tests). Use pass^k for customer-facing agents where consistency matters every time. Tau-Bench (ICLR 2025) demonstrated that agents looking good on average become wildly unstable under repeated runs.

**Core production metrics**: Task completion rate (target: 90%+ simple, 70-80% medium, 50-60% complex), cost per transaction (token + infra + tool invocations), p95 latency per task type, regression detection rate in CI/CD, deployment frequency vs rollback rate.

**Drift-specific metrics**: Embedding distance for concept drift, semantic similarity checks week-over-week, human intervention rate as a lagging indicator, loop rate (percentage of tasks entering retry cycles).

## 6. Observability Platform Comparison

| Platform | Pricing | Overhead | Key Differentiator |
|---|---|---|---|
| LangSmith | Free 5k traces/mo, $39/user/mo paid | Near-zero | Native LangChain integration, ~30min setup |
| Braintrust | Free 1M spans/mo, $249/mo pro | Low | Eval-driven with deployment blocking, 80x query perf |
| Langfuse | Free self-hosted, $29/mo cloud | ~15% | Open-source, SQL access, OTel-native |
| Arize Phoenix | Free OSS, $50/mo managed | Low | Drift detection, embedding monitoring |
| AgentOps | Free 1k events, $40/mo pro | ~12% | Time-travel debugging, Python-only |
| Helicone | Free 100k req/mo, $25/mo flat | 50-80ms added | Proxy-based, 15min setup, semantic caching |
| Galileo | Free 5k traces/mo, $100/50k traces | Low | Luna-2 SLM at sub-200ms, ~$0.02/M tokens |
| Fiddler | Enterprise only ($20k-100k+/yr) | Sub-100ms guardrails | Compliance, bias detection, regulated industries |

Integration methods ranked by setup time: Proxy (15min, 50-80ms overhead) < SDK (hours, deeper context) < OpenTelemetry (days, maximum flexibility and vendor independence).

## 7. Guardrail Execution Patterns

**Parallel execution** (OpenAI Agents SDK default, `run_in_parallel=True`): Guardrails run concurrently with agent execution. Best latency since both start simultaneously. Tradeoff: if the guardrail trips, the agent may have already consumed tokens and executed tools.

**Blocking execution** (`run_in_parallel=False`): Guardrail completes before agent starts. No wasted tokens on rejected inputs. Ideal for cost optimization and high-risk tool invocations (refunds, database writes, API calls).

**Constitutional Classifiers++ (Anthropic)**: Two-stage cascade. Stage 1: linear probe on model's internal activations screens all traffic with ~1% compute overhead (down from 23% in v1). Stage 2: full exchange classifier activated only when probe flags suspicion. Result: jailbreak success rate drops from 86% to 4.4%, false positive refusals reduced 87% vs v1.

**Tool-level guardrails**: Input checks pre-execution, output checks post-execution, enabling fine-grained control per tool invocation rather than blanket agent-level filtering.

Latency-sensitive operations (financial transactions, database mutations) require synchronous guardrails that inspect, validate, and block before the tool fires. A "too late" safety check is architecturally useless.

## 8. Feedback-to-Improvement Loops

**Eval-Driven Development (EDD)**: Evals precede code. Define correctness specifications before writing prompts or selecting models. Tier evaluations: fast smoke tests on every commit, comprehensive suites nightly. Evals run in CI/CD alongside lint and type-check -- quarterly notebook reviews do not count. Version-control eval definitions, datasets, and thresholds with changelogs.

**Production trace to eval conversion**: Braintrust and similar platforms convert production traces directly into test cases. Sample production traffic continuously, compare quality scores week-over-week, and use downward trends as signals before users notice degradation.

**Progressive deployment**: Shadow mode first (compare agent decisions to human decisions), then 1-5% canary traffic with guardrail metrics monitored, then gradual ramp based on task completion rate, satisfaction scores, and resolution rates. Ramp's approach: "AI as suggestions before graduating to autonomous actions" with an autonomy slider per team.

**The evaluator-optimizer pattern**: Generator LLM produces output, evaluator LLM scores it with feedback. Cycle repeats until quality threshold is met. Practical ceiling of 2-3 iterations for most tasks before diminishing returns.

## 9. Cost Observability

Only 44% of organizations have financial guardrails for AI (Gartner, expected to double by end of 2026). A $400M collective leak in unbudgeted cloud spend hit Fortune 500 from autonomous agent resource exhaustion.

**Per-step token tracking**: Attach metadata (user_id, feature_name, team) to every API request. Track prompt tokens and completion tokens attributed to specific dimensions. A single agent request creates 8-15 telemetry spans vs 2-3 for a traditional API call.

**Budget alerting**: Dollar-based thresholds per user (e.g., $50 in 24 hours triggers investigation). Human-in-the-loop approval for any process exceeding a compute threshold. Automated enforcement: throttle traffic or block requests at hard caps.

**Unit economics**: Track cost-per-insight and cost-per-outcome rather than total cloud spend. Include token spend, latency, and compute as eval dimensions alongside quality metrics. Minor token increases compound significantly at production scale -- a 10% increase per step across a 15-step workflow is 4.2x total cost.

## 10. The Swiss Cheese Model

No single layer catches everything. The model stacks imperfect defenses so failures passing through one layer are caught by the next.

**Layer 1 -- Data Quality**: Vector databases, data versioning, automated freshness checks. Catches stale information and broken documentation. Cannot prevent logical reasoning errors.

**Layer 2 -- Prompt Constraints**: Precise system prompts defining capabilities and boundaries, escalation triggers, confirmation requirements. Catches ambiguous instructions and out-of-scope actions. Cannot guarantee correct interpretation of complex scenarios.

**Layer 3 -- Reasoning and Grounding**: RAG with multi-stage retrieval, chain-of-thought with extended thinking. Catches logical inconsistencies and false assumptions. Still misses edge cases and operational realities.

**Layer 4 -- Runtime Guardrails**: Transaction limits, rate limiting, approval chains, anomaly detection, rollback capability. Catches high-impact errors before execution. Reactive by nature, requires human oversight for resolution.

**Layer 5 -- Production Monitoring**: Continuous automated quality evals on sampled traffic, week-over-week comparison, segment-level breakdowns. Catches drift invisible in aggregate metrics.

**Layer 6 -- Human Review**: Transcript sampling, user feedback triage, systematic human studies for calibrating automated graders. Catches subjective degradation that metrics miss.

Anthropic's recommended combination: automated evals (pre-launch, CI/CD) + production monitoring (post-launch) + A/B testing (with sufficient traffic) + user feedback and transcript review (ongoing) + systematic human studies (for calibration). The 5 required pillars: Traces, Evaluations, Human Review, Alerts, Data Engine.

## Sources

- https://opentelemetry.io/docs/specs/semconv/gen-ai/gen-ai-agent-spans/
- https://opentelemetry.io/blog/2025/ai-agent-observability/
- https://github.com/open-telemetry/semantic-conventions/issues/2664
- https://www.anthropic.com/engineering/demystifying-evals-for-ai-agents
- https://www.philschmid.de/agents-pass-at-k-pass-power-k
- https://www.langchain.com/state-of-agent-engineering
- https://arxiv.org/html/2601.04170
- https://vadim.blog/verification-gate-research-to-practice
- https://dev.to/boucle2026/how-to-tell-if-your-ai-agent-is-stuck-with-real-data-from-220-loops-4d4h
- https://www.fixbrokenaiapps.com/blog/ai-agents-infinite-loops
- https://gantz.ai/blog/post/agent-loops/
- https://www.superteams.ai/blog/how-to-use-the-swiss-cheese-model-for-ai-agent-accuracy
- https://www.braintrust.dev/articles/best-ai-observability-tools-2026
- https://softcery.com/lab/top-8-observability-platforms-for-ai-agents-in-2025
- https://alignment.anthropic.com/2025/cheap-monitors/
- https://www.anthropic.com/research/next-generation-constitutional-classifiers
- https://openai.github.io/openai-agents-python/guardrails/
- https://evaldriven.org/
- https://analyticsweek.com/finops-for-agentic-ai-cloud-cost-2026/
- https://www.traceloop.com/blog/from-bills-to-budgets-how-to-track-llm-token-usage-and-cost-per-user
- https://www.getmaxim.ai/articles/a-comprehensive-guide-to-preventing-ai-agent-drift-over-time/
- https://dev.to/navyabuilds/from-prototype-to-production-10-metrics-for-reliable-ai-agents-4ha6
- https://newsletter.owainlewis.com/p/the-10x-skill-for-ai-engineers-in
- https://www.zenml.io/blog/what-1200-production-deployments-reveal-about-llmops-in-2025
- https://www.braintrust.dev/articles/ab-testing-llm-prompts
