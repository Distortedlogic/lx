# DSPy: Deep Dive

## Identity

DSPy is a framework from Stanford NLP for "programming -- not prompting -- language models." Created by Omar Khattab (now at Databricks). 32.9k GitHub stars, MIT license, v3.1.3 (February 2026). Published at ICLR 2024. Core thesis: prompt engineering should be automated via compilation, not hand-crafted. The framework draws directly from PyTorch's nn.Module abstraction.

## The Six Pieces

DSPy has six core abstractions: LMs (language model clients), Signatures (declarative I/O specs), Modules (composable strategy units), Examples (training data), Optimizers (prompt/weight compilers), and Adapters (provider-specific formatters).

## Signatures: Typed I/O Specs for LLM Calls

Signatures are DSPy's foundational abstraction -- declarative specifications of input/output behavior that replace prompt templates. In compiler terms, they are the type signatures of LM calls.

**Two syntactic forms:**

Inline: `"context: list[str], question: str -> answer: str"`

Class-based (Pydantic-style):
```python
class Emotion(dspy.Signature):
    """Classify emotion."""
    sentence: str = dspy.InputField()
    sentiment: Literal['sadness', 'joy', 'fear'] = dspy.OutputField()
```

Fields accept `desc` (natural-language hint/constraint), `prefix` (prompt label), and `format` (serialization callable). The class docstring becomes the task instruction.

**Type system:** `str`, `int`, `bool`, `float`, `list[str]`, `dict[str, int]`, `Optional[T]`, `Union[T1, T2]`, `Literal[...]`, Pydantic models, `dspy.Image`, `dspy.Audio`, `dspy.History`.

**Critical insight:** Signatures are not prompts. They are intermediate representations that adapters compile into provider-specific message formats. The `Adapter.format()` method converts a signature + inputs into the actual messages array sent to the LM. This indirection makes signatures portable across models and optimizable by compilers.

## Modules: Composable Strategy Units

All modules descend from `dspy.Module` (parallel to `nn.Module`). Modules have learnable parameters (prompt instructions, demonstrations, LM weights) and are callable objects.

| Module | What it does |
|--------|-------------|
| `Predict` | Foundational. Takes signature, stores instructions/demos, invokes LM. |
| `ChainOfThought` | Wraps Predict by prepending a `reasoning: str` OutputField. Forces step-by-step thinking before answering. |
| `ReAct` | Reasoning+Acting agent loop. Tools registered as `dspy.Tool`. Trajectory stores `thought/tool_name/tool_args/observation` tuples. `max_iters=20`. |
| `ProgramOfThought` | Routes through code generation and execution. |
| `MultiChainComparison` | Runs multiple ChainOfThought instances, compares outputs. |
| `Parallel` | Concurrent execution of multiple modules or batched calls. |
| `BestOfN` | Runs N times, returns highest-reward prediction. |
| `Refine` | Extends BestOfN with automatic feedback loops -- generates detailed feedback after each failed attempt, uses as hints for next run. Replaces deprecated Assert/Suggest as of DSPy 2.6. |
| `Reasoning` | (v3.1.0) Captures native reasoning from reasoning models (o3, DeepSeek R1). Surfaces thinking tokens. |
| `CodeAct` | Executes code using provided tools directly. |

Custom modules implement `forward()` (and optionally `aforward()` for async). Standard Python control flow orchestrates sub-module calls. There is no graph DSL -- the computational graph IS the Python control flow.

```python
class MultiHop(dspy.Module):
    def __init__(self, num_hops=4):
        self.generate_query = dspy.ChainOfThought("context, question -> search_query")
        self.append_notes = dspy.ChainOfThought("context, question, notes -> updated_notes")

    def forward(self, question: str) -> str:
        context = []
        for _ in range(self.num_hops):
            query = self.generate_query(context=context, question=question)
            context += dspy.Retrieve(k=3)(query.search_query).passages
        return self.append_notes(context=context, question=question, notes="")
```

## Optimizers: Prompt/Weight Compilers

An optimizer takes your program, a metric function, and training examples (as few as 5-10). Returns an optimized copy with the same structure but better-tuned parameters.

### Few-Shot Learning

- **LabeledFewShot:** Random k examples as demos. No LM calls.
- **BootstrapFewShot:** Executes teacher program on training data, validates with metric, keeps passing traces as demonstrations. Multi-step handling: random sampling from early/final traces with 50/50 probability.
- **BootstrapRS (RandomSearch):** Runs BootstrapFewShot multiple times with special seeds (seed -3 = zero-shot, -2 = labeled-only, -1 = unshuffled, >=0 = shuffled with random demo sizes). Selects best on validation.
- **KNNFewShot:** At inference, uses k-NN with embeddings to dynamically select most relevant demos per input.

### Instruction Optimization

- **COPRO:** Coordinate ascent (hill-climbing) to generate and refine natural-language instructions.
- **MIPROv2 (flagship):** Three phases: (1) Bootstrap candidate demo sets, (2) Grounded Proposal via LM-powered module using four context signals (program-aware, data-aware, tip-aware, few-shot-aware), (3) Bayesian Optimization via Optuna's TPE -- instructions and demos as categorical parameters in a joint search space. Trial count: `max(2 * num_vars * log2(N), 1.5 * N)`.
- **SIMBA:** Self-reflective. Samples mini-batches, creates diverse traces at multiple temperatures, measures output variability, generates introspective rules for challenging examples. Softmax sampling balances exploration vs exploitation.
- **GEPA (Genetic-Pareto):** Maintains a Pareto frontier of candidates (highest score on at least one validation instance). Reflective mutation: sample from frontier, capture traces, LM-reflect on what worked/failed, propose mutations. Accepts `ScoreWithFeedback` dicts -- textual feedback, not just scalar scores.

### Weight Optimization

- **BootstrapFinetune:** Distills prompt-based programs into model weight updates via bootstrapped traces.
- **BetterTogether:** Meta-optimizer combining prompt and weight optimization in configurable sequences.

**Cost profile:** Typical MIPROv2: ~$2-3, 6-10 minutes, ~3,200 API calls, ~2.7M input tokens. Range: $0.01 to $100+ depending on scale.

## Adapters: Provider-Specific Formatters

Convert (signature, inputs, demos) into provider-specific message formats and parse responses back.

- **ChatAdapter (default):** Uses `[[ ## field_name ## ]]` delimiters. Falls back to JSONAdapter on parse failure.
- **JSONAdapter:** Prompts LM to return JSON object. Lower latency, more reliable parsing. Requires structured output support.
- **XMLAdapter:** XML tags. Documented as particularly suited for Claude models.

Message structure: system message (field definitions + task objective) + alternating user/assistant messages for demos + final user message with actual input.

## Assertions → Refine Evolution

The original DSPy Assertions (published separately: arXiv:2312.13382) provided `dspy.Assert` (hard constraint, backtrack on failure) and `dspy.Suggest` (soft constraint, log and continue). Self-refinement via a "Retry" meta-module that adds `past_output` and `instruction` fields to the signature.

Empirical results: on HotPotQA, suggestions improved constraint satisfaction by 35.7%, retrieval scores by 4.2-13.3%.

**As of DSPy 2.6+, assertions are deprecated** in favor of `dspy.Refine` -- reward-function-based iterative improvement with configurable `fail_count` and temperature escalation. The evolution reflects maturation from imperative constraints to declarative quality objectives.

## Compile vs Execute

The "compile" step IS optimization -- it doesn't produce a different executable artifact. It mutates the program's parameters (instructions, demos, weights) in-place. A "compiled" program has the same structure but better-tuned prompts. Analogous to training a neural network: architecture stays fixed, weights change.

The compiler "simulates versions of the program on the inputs and bootstraps example traces of each module for self-improvement."

## Traces

`dspy.settings.trace` stores execution history as `(predictor, inputs, outputs)` tuples. Enables debugging (step-through), optimizer feedback (bootstrapping from high-scoring traces), and metric computation (the `trace` parameter in metrics lets you inspect intermediate predictions).

## Metrics

`def metric(example, pred, trace=None) -> float | int | bool`

When `trace is None` (evaluation): return float scores. When `trace is not None` (bootstrapping): return strict booleans for demo filtering. Four complexity levels: exact match, multi-property validation, LLM-as-judge (itself a DSPy program, optimizable), trace-based.

Key insight: "If your metric is itself a DSPy program, one of the most powerful ways to iterate is to compile (optimize) your metric itself."

## The "Programming Not Prompting" Philosophy

Khattab: "the hand-coded prompt approach, while pervasive, can be brittle and unscalable -- conceptually akin to hand-tuning the weights for a classifier." DSPy "pushes building new LM pipelines away from manipulating free-form strings and closer to programming (composing modular operators to build text transformation graphs) where a compiler automatically generates optimized LM invocation strategies."

The Bitter Lesson applied to prompting: general methods leveraging computation ultimately beat hand-engineered solutions.

## Empirical Results

"Within minutes of compiling, a few lines of DSPy allow GPT-3.5 and llama2-13b-chat to self-bootstrap pipelines that outperform standard few-shot prompting (generally by over 25% and 65%, respectively) and pipelines with expert-created demonstrations (by up to 5-46% and 16-40%, respectively)."

Multi-use case study: jailbreak detection manual 59% → DSPy 93.18% (+34 points). Hallucination detection 64% → 82% (+18). Prompt evaluation 46.2% → 76.9% (+30.7).

## Production Usage

JetBlue (chatbot), Replit (code diff synthesis), Databricks (LM judges, RAG, classification), Sephora (agents), VMware (RAG), Moody's (financial RAG), Haize Labs (red-teaming: CodeAttack success 75% → 5%), Relevance AI (self-improving email agents matching human quality 80% of time).

Research: STORM (Wikipedia articles), WangLab @ MEDIQA (outperformed next-best by 20 points), UMD Suicide Detection (40% improvement over 20-hour expert prompting).

## Criticisms

**Adoption gap.** 4.7M monthly downloads vs LangChain's 222M. "DSPy's problem isn't that it's wrong. It's that it's hard."

**Compilation cost.** $2-3 typical, $20-50 for complex pipelines, 10-30 minutes.

**Black box.** "Optimizers and compilers can be difficult to understand, seem non-intuitive, and mysterious." Compiled prompts not always human-interpretable.

**Formatting overhead.** DSPy-formatted prompts sometimes decrease performance on already-strong baselines (code generation: 87.5% baseline → 83.0% with DSPy formatting).

**Metric dependency.** "Effectiveness relies heavily on the chosen metrics for optimization" -- garbage metrics produce garbage optimization.

## Relevance to lx

**Signatures as the unit of abstraction.** The most portable idea in DSPy. Separating WHAT (I/O spec) from HOW (prompt/model/optimization) is the key design move. lx should have first-class typed task specifications that are independent of execution strategy. An lx `task` block with typed inputs and outputs is essentially a signature.

**Optimization as compilation.** The separation of program structure from tunable parameters enables systematic improvement without changing user code. lx's runtime could optimize agent prompts, tool selection, or routing strategies independently of the workflow definition. This is a post-v1 feature but the architecture should support it.

**Traces as first-class data.** DSPy's trace mechanism enables both debugging and self-improvement. lx workflow execution traces should serve the same dual purpose -- observable for debugging, usable for optimization.

**Metrics drive everything.** Without a metric, there's no optimization. lx should build evaluation primitives in from day one. An `eval` block or `metric` declaration that defines how to score agent outputs.

**The assertion-to-refinement evolution.** Deprecating imperative Assert/Suggest in favor of reward-function-based Refine reflects maturation from runtime halts to declarative quality objectives. lx should express quality constraints as optimization targets, not as exceptions.

**Khattab's Law.** Complex AI systems inevitably contain "an ad hoc, informally-specified, bug-ridden implementation of half of DSPy." lx embedding these patterns (typed signatures, composable modules, metric-driven optimization) at the language level eliminates the framework learning curve while preserving the benefits.

**The adoption paradox as opportunity.** DSPy is technically superior but harder to adopt because its abstractions are unfamiliar Python library patterns. A purpose-built language makes the abstractions THE language -- no framework to learn, just syntax.