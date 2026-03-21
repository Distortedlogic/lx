# Iterative Refinement in AI Systems: Research Landscape

Research survey covering self-refinement, evaluation frameworks, convergence theory, and their relationship to lx's `refine` construct.

## 1. Foundational Papers on LLM Self-Refinement

### 1.1 Self-Refine (Madaan et al., 2023)

The foundational paper on iterative self-refinement without additional training.

**Citation:** Madaan, A., Tandon, N., et al. "Self-Refine: Iterative Refinement with Self-Feedback." NeurIPS 2023. [arXiv:2303.17651](https://arxiv.org/abs/2303.17651)

**Core loop:**
```
output_0 = LLM(prompt)
loop:
    feedback = LLM(feedback_prompt, output_t)
    output_{t+1} = LLM(refine_prompt, output_t, feedback)
    if is_refinement_sufficient(output_{t+1}): break
```

**Key properties:**
- Single model serves as generator, critic, and refiner (no separate models needed)
- No supervised training data or RL required
- History of past outputs is appended to prompts, enabling learning from prior attempts
- Stopping condition is task-dependent (`is_refinement_sufficient` varies per domain)

**Evaluated across 7 tasks:**
1. Review Rewriting (sentiment reversal)
2. Acronym Generation
3. Story Generation
4. Code Rewriting (optimization)
5. Response Generation (dialogue)
6. Constrained Generation (commonsense)
7. Toxicity Removal

**Results:** 5-40% improvement across tasks, averaging ~20% absolute improvement over single-pass generation with GPT-3.5/GPT-4.

**Relevance to lx `refine`:** Self-Refine uses a single LLM for all three roles. lx's `refine` construct separates `grade` and `revise` into distinct functions, which allows mixing LLM-based and tool-based evaluation. Self-Refine's task-dependent stopping maps to lx's `threshold` parameter.

---

### 1.2 Reflexion (Shinn et al., 2023)

Verbal reinforcement learning: agents reflect on failures and maintain episodic memory.

**Citation:** Shinn, N., Cassano, F., Gopinath, A., Narasimhan, K., Yao, S. "Reflexion: Language Agents with Verbal Reinforcement Learning." NeurIPS 2023. [arXiv:2303.11366](https://arxiv.org/abs/2303.11366)

**Core mechanism:**
- Agent attempts a task, receives binary or scalar feedback
- Agent generates a **verbal reflection** analyzing why it failed
- Reflection is stored in an **episodic memory buffer**
- On the next trial, the agent conditions on its accumulated reflections
- No weight updates; the "learning" is entirely in-context via the memory buffer

**Key distinction from Self-Refine:** Reflexion operates across *trials* (complete re-attempts), not within a single generation pass. The reflection is a meta-cognitive summary of failure, not line-by-line feedback.

**Results:**
- HumanEval coding: 91% pass@1 (vs GPT-4's 80%)
- Also evaluated on AlfWorld (sequential decision-making) and HotpotQA (reasoning)

**Relevance to lx `refine`:** Reflexion's episodic memory maps to a pattern where the `revise` function accumulates context across rounds. lx's `on_round` callback could record reflections that feed into subsequent `revise` calls.

---

### 1.3 CRITIC (Gou et al., 2023)

LLMs verify and correct themselves using external tools, not just self-evaluation.

**Citation:** Gou, Z., Shao, Z., et al. "CRITIC: Large Language Models Can Self-Correct with Tool-Interactive Critiquing." [arXiv:2305.11738](https://arxiv.org/abs/2305.11738)

**Core insight:** LLMs cannot reliably critique their own outputs without external grounding. CRITIC provides that grounding through tool interaction.

**Tool categories used:**
- **Search engines** for fact-checking (free-form QA)
- **Code interpreters** for verifying generated programs (mathematical program synthesis)
- **Calculators** for numerical validation
- **Toxicity classifiers** for content safety

**Loop:**
```
output = LLM(prompt)
loop:
    critique = LLM(output) + tool_results(output)
    output = LLM(refine_prompt, output, critique)
```

**Results:** Consistent improvement across free-form QA, mathematical program synthesis, and toxicity reduction. The key finding is that tool-grounded critique significantly outperforms pure self-critique.

**Relevance to lx `refine`:** CRITIC validates lx's design decision to separate `grade` from `revise`. The `grade` function in lx can invoke external tools (compilers, test suites, search APIs) rather than relying on LLM self-assessment. This is the "grounded refinement" pattern -- see `design-patterns.md`.

---

### 1.4 The Self-Correction Paradox (Huang et al., 2023)

The critical counterpoint: LLMs struggle to self-correct without external signal.

**Citation:** Huang, J., et al. "Large Language Models Cannot Self-Correct Reasoning Yet." ICLR 2024. [arXiv:2310.01798](https://arxiv.org/abs/2310.01798)

**Key findings:**
- **Intrinsic self-correction** (no external feedback) frequently **degrades** performance on reasoning tasks
- The fundamental paradox: if an LLM could recognize its own errors, why didn't it produce the correct answer initially?
- Self-correction works only when the critique signal contains information the model didn't have during initial generation (external tools, human feedback, additional context)
- Performance degradation occurs because the model sometimes "corrects" correct answers into wrong ones

**Implications for system design:**
- Pure self-evaluation is unreliable for reasoning tasks
- External feedback (tool outputs, test results, human judgment) is essential
- The quality of the critique signal determines the ceiling of iterative refinement

**Relevance to lx `refine`:** This paper is the strongest argument for lx's `grade` function being a first-class, user-supplied component rather than an implicit self-evaluation. The `grade` function should incorporate external signals (compiler output, test results, API responses) to avoid the self-correction paradox.

---

## 2. Reasoning and Search Approaches

### 2.1 Chain-of-Thought Prompting (Wei et al., 2022)

Step-by-step reasoning as a prerequisite for effective self-evaluation.

**Citation:** Wei, J., Wang, X., et al. "Chain-of-Thought Prompting Elicits Reasoning in Large Language Models." NeurIPS 2022. [arXiv:2201.11903](https://arxiv.org/abs/2201.11903)

**Core finding:** Providing a few chain-of-thought demonstrations (intermediate reasoning steps) in the prompt significantly improves reasoning on arithmetic, commonsense, and symbolic tasks. A 540B-parameter model with 8 CoT exemplars achieved state-of-the-art on GSM8K, surpassing fine-tuned GPT-3 with a verifier.

**Model size dependency:** CoT reasoning emerges only in models above ~100B parameters. In smaller models, CoT can actually reduce performance.

**Relevance to refinement:** CoT is foundational to evaluation -- G-Eval (Section 3.2) uses CoT to generate evaluation steps. Effective `grade` functions should elicit chain-of-thought reasoning from the evaluating model to produce calibrated scores and actionable feedback.

---

### 2.2 Tree of Thoughts (Yao et al., 2023)

Deliberate problem solving with backtracking and multiple reasoning paths.

**Citation:** Yao, S., Yu, D., Zhao, J., et al. "Tree of Thoughts: Deliberate Problem Solving with Large Language Models." NeurIPS 2023. [arXiv:2305.10601](https://arxiv.org/abs/2305.10601)

**Core mechanism:**
- Generalizes CoT from a single chain to a **tree** of reasoning paths
- At each step, the model generates multiple candidate "thoughts" (intermediate steps)
- A value function evaluates each candidate
- Search algorithms (BFS, DFS) explore the tree with **backtracking**
- The model can abandon unpromising paths and try alternatives

**Results:** Game of 24: 74% success (vs 4% with CoT). Also evaluated on Creative Writing and Mini Crosswords with large improvements.

**Relevance to lx `refine`:** ToT represents a different iteration topology -- tree-structured rather than linear. lx's `refine` is a linear loop (try -> grade -> revise -> repeat), while ToT branches. A future `explore` construct could support tree-shaped search. However, `refine` with `max_rounds` and threshold-based stopping is the right default for most agentic workflows where you want focused improvement rather than broad exploration.

---

### 2.3 OpenAI's Reasoning Models (o1, o3)

Internal chain-of-thought with learned self-correction via RL.

**Sources:**
- [Learning to Reason with LLMs](https://openai.com/index/learning-to-reason-with-llms/) (OpenAI, Sept 2024)
- [o1 Technical Primer](https://www.lesswrong.com/posts/byNYzsfFmb2TpYFPW/o1-a-technical-primer) (LessWrong)

**How o1/o3 work:**
- Models generate hidden "reasoning tokens" (chain-of-thought that is not exposed in the API response but is billed as output tokens)
- Trained via large-scale RL to produce productive chains of thought
- The model learns to: recognize mistakes, break hard steps into simpler ones, try alternative approaches when stuck
- Performance scales with both train-time compute (more RL) and test-time compute (more reasoning tokens)

**o1 results:** 83% on AIME 2024 (single sample), 93% with reranking 1000 samples via learned scoring. Equivalent to top-500 US math students.

**o3 results:** 96.7% on AIME 2024, matching gold-medal IMO competitors.

**Key insight:** o1/o3 represent the "internalized refinement" approach -- the generate-evaluate-revise loop is baked into the model's reasoning process via RL, rather than being an explicit outer loop. This is complementary to lx's `refine` construct, which provides an *explicit* outer loop that works with any model.

---

### 2.4 Constitutional AI (Anthropic)

Self-improvement through constitutional principles during training.

**Citation:** Bai, Y., et al. "Constitutional AI: Harmlessness from AI Feedback." Anthropic, 2022. [arXiv:2212.08073](https://arxiv.org/abs/2212.08073)

**Two-phase process:**
1. **Supervised Learning phase:** Sample from initial model, generate self-critiques against constitutional principles, revise, fine-tune on revised responses
2. **RL phase (RLAIF):** Sample from finetuned model, use a model to evaluate which sample is better according to the constitution, train a preference model, use it for RL

**The constitution:** A set of natural language principles (e.g., "choose the response that is least harmful," "choose the response that is most helpful") that replace human preference labels. Principles are randomly sampled per critique round.

**Key innovation:** The critique-revise loop happens during *training*, not inference. The model internalizes the constitutional principles so it doesn't need explicit refinement at inference time.

**Relevance to lx `refine`:** Constitutional AI's principle-based critique maps to lx's rubric pattern (see `pkg/ai/quality.lx`). The `grade` function can encode constitutional principles as rubric dimensions. The random principle sampling is analogous to evaluating different rubric categories per round.

---

## 3. Grading and Evaluation Frameworks

### 3.1 LLM-as-Judge

Using LLMs to evaluate other LLM outputs.

**Survey:** Gu, Y., et al. "LLMs-as-Judges: A Comprehensive Survey on LLM-based Evaluation Methods." 2024. [arXiv:2412.05579](https://arxiv.org/abs/2412.05579)

**Evaluation paradigms:**
- **Pointwise:** Score a single output on a rubric (1-5 Likert scale, 0-100 continuous)
- **Pairwise:** Compare two outputs and select the better one
- **Listwise:** Rank multiple outputs simultaneously

**Calibration techniques:**
- Adding a grading rubric with score-level descriptions
- Few-shot examples of scored outputs for calibration
- Measuring logprobs of each possible score for probability-weighted evaluation
- Position debiasing (randomizing the order of pairwise comparisons)

**Known biases:**
- **Self-preference bias:** LLMs prefer their own outputs or outputs from similar models
- **Position bias:** Models tend to favor the first or last option in pairwise comparisons
- **Verbosity bias:** Longer outputs tend to receive higher scores regardless of quality
- **Anchoring:** Prior scores or examples influence subsequent judgments

**Best practices (2024-2025):**
- Use rubrics with explicit score-level descriptions
- Employ multi-dimensional evaluation (not a single score)
- Include few-shot calibration examples
- Run multiple evaluations and aggregate
- Use different models for generation and evaluation when possible

---

### 3.2 G-Eval (Liu et al., 2023)

Framework for using GPT-4 as a structured evaluator.

**Citation:** Liu, Y., Iter, D., et al. "G-Eval: NLG Evaluation using GPT-4 with Better Human Alignment." EMNLP 2023. [arXiv:2303.16634](https://arxiv.org/abs/2303.16634)

**Methodology:**
1. Input Task Introduction and Evaluation Criteria to LLM
2. LLM generates a **chain-of-thought** of detailed Evaluation Steps
3. Use the prompt + generated CoT to evaluate NLG outputs in a **form-filling paradigm**
4. Extract probability-weighted scores from token logprobs

**Results:** Spearman correlation of 0.514 with human judgments on summarization (previous best was significantly lower). Evaluated on summarization and dialogue generation.

**Critical finding:** LLM-based evaluators show bias toward LLM-generated text. GPT-4 tends to rate GPT-4 outputs higher than human outputs, even when human outputs are objectively better.

**Relevance to lx:** G-Eval's CoT-based evaluation steps map to how a sophisticated `grade` function should work -- generate evaluation criteria, reason through each criterion, then assign scores. The `response_rubric` and `code_rubric` in `pkg/ai/quality.lx` implement this pattern.

---

### 3.3 Prometheus (Kim et al., 2023)

Open-source LLM trained specifically for fine-grained evaluation.

**Citation:** Kim, S., Shin, J., et al. "Prometheus: Inducing Fine-grained Evaluation Capability in Language Models." ICLR 2024. [arXiv:2310.08491](https://arxiv.org/abs/2310.08491)

**Training data (Feedback Collection):**
- 1,000 fine-grained score rubrics
- 20,000 instructions
- 100,000 responses with language feedback, generated by GPT-4

**Architecture:** 13B parameter model (based on Llama), fine-tuned specifically for evaluation tasks.

**Results:**
- Pearson correlation 0.897 with human evaluators (45 custom rubrics)
- GPT-4: 0.882 correlation (comparable)
- ChatGPT: 0.392 correlation (far worse)
- Outperforms open-source reward models on HHH Alignment and MT Bench Human Judgment benchmarks

**Key innovation:** Customizable score rubrics. Users provide their own evaluation criteria, and Prometheus evaluates against them. This makes it a general-purpose evaluator, not task-specific.

**Prometheus 2** (2024, [arXiv:2405.01535](https://arxiv.org/abs/2405.01535)): Extended to support both pointwise and pairwise evaluation, with improved correlation numbers.

**Relevance to lx:** Prometheus validates the rubric-based grading approach used in `pkg/ai/quality.lx`. A `grade` function could use Prometheus as its backbone model for evaluation, providing open-source, customizable assessment.

---

### 3.4 RAGAS (Retrieval Augmented Generation Assessment)

Evaluation framework specifically for RAG pipelines.

**Citation:** Es, S., James, J., et al. "RAGAS: Automated Evaluation of Retrieval Augmented Generation." 2023. [arXiv:2309.15217](https://arxiv.org/abs/2309.15217)

**Documentation:** [docs.ragas.io](https://docs.ragas.io/en/stable/)

**Core metrics:**
| Metric | Measures | Reference-free? |
|--------|----------|-----------------|
| Faithfulness | Factual consistency of answer with retrieved context | Yes |
| Context Precision | Relevance of retrieved context to the question | Yes |
| Context Recall | Whether retriever found all necessary information | Needs ground truth |
| Answer Relevancy | How well the answer addresses the question | Yes |
| Noise Sensitivity | Robustness to irrelevant retrieved context | Yes |

**Additional metric categories:**
- Natural language comparison: factual correctness, semantic similarity
- Traditional NLP: BLEU, ROUGE, chrF
- Agent/tool use: tool call accuracy, agent goal accuracy
- General purpose: aspect critic, rubrics-based evaluation

**Relevance to lx:** RAGAS demonstrates domain-specific evaluation frameworks. lx's `refine` construct could use RAGAS-style metrics as the `grade` function for RAG workflow refinement.

---

### 3.5 Automated Evaluation Frameworks

**DeepEval** ([deepeval.com](https://deepeval.com))
- "Pytest for LLMs" -- unit-test-like interface for validating model outputs
- 60+ built-in metrics across prompt, RAG, chatbot, and safety testing
- Open-source, Python-native
- Custom metric definition supported

**promptfoo** ([promptfoo.dev](https://promptfoo.dev))
- YAML-based test configuration (declarative evaluation)
- Strong built-in red-teaming and security scanning
- Zero cloud dependencies
- Best for security-focused evaluation

**Braintrust** ([braintrust.dev](https://www.braintrust.dev))
- End-to-end platform connecting evaluation to production monitoring
- Closes the loop between production data and CI testing
- Collaboration features for teams
- Best for full evaluation lifecycle management

**Relevance to lx:** These frameworks validate the pattern of automated, programmatic evaluation. lx's `grade` function is the language-level primitive that these frameworks build on top of.

---

## 4. Convergence and Termination Theory

### 4.1 Mathematical Model of Refinement Convergence

Yang et al. (2025) formalized iterative refinement as a Markov chain.

**Source:** [Iterative review-fix loops remove LLM hallucinations](https://dev.to/yannick555/iterative-review-fix-loops-remove-llm-hallucinations-and-there-is-a-formula-for-it-4ee8), based on "A Probabilistic Inference Scaling Theory for LLM Self-Correction."

**Recurrence relation:**
```
Acc_t = Acc_{t-1} * CL + (1 - Acc_{t-1}) * CS
```

Where:
- `Acc_t` = probability of correct output after round t
- `CL` (Confidence Level) = probability the model preserves correct content (keeps right things right)
- `CS` (Critique Score) = probability the model fixes an error (converts wrong to right)

**Closed-form convergence:**
```
Acc_t = Upp - alpha^t * (Upp - Acc_0)
```

Where:
- `Upp = CS / (1 - CL + CS)` -- the theoretical accuracy ceiling
- `alpha = CL - CS` -- convergence rate (smaller = faster convergence)
- `Acc_0` = initial accuracy

**Critical conditions:**
- **Convergent** when `0 < alpha < 1` (CL > CS, both in (0,1))
- **Oscillating** when `alpha < 0` (CS > CL) -- model fixes errors but also breaks correct content
- **Divergent** when CL is very low -- model degrades its own correct outputs faster than it fixes errors

**Practical stopping rules:**
1. Two review passes capture ~75% of achievable improvement
2. Track findings-per-round; stop when findings plateau or increase
3. Hard cap at 5-6 rounds maximum
4. Monitor for stochastic drift (prior fixes getting broken); if detected, use previous round's output

---

### 4.2 Test-Time Compute Scaling (Snell et al., 2024)

How to optimally allocate inference-time computation.

**Citation:** Snell, C., Lee, J., Xu, K., Kumar, A. "Scaling LLM Test-Time Compute Optimally can be More Effective than Scaling Model Parameters." 2024. [arXiv:2408.03314](https://arxiv.org/abs/2408.03314)

**Two mechanisms for scaling test-time compute:**
1. **Search against verifier reward models** -- generate candidates, score with a process-based reward model (PRM), select best
2. **Sequential revision** -- iteratively refine a single output

**Compute-optimal strategy depends on problem difficulty:**
- **Easy problems** (model already has reasonable initial output): Sequential revision is more effective. The model's first attempt is on the right track and benefits from focused refinement.
- **Hard problems** (model needs to explore fundamentally different approaches): Best-of-N sampling or tree-search against a PRM is better. You need diverse exploration, not focused refinement.

**Key result:** Compute-optimal scaling achieves 2-4x efficiency improvement over naive best-of-N. A small model with optimal test-time compute allocation can outperform a 14x larger model in FLOPs-matched comparisons.

**Relevance to lx `refine`:** This research directly validates lx's `refine` as the right construct for "easy-to-medium" problems where the initial attempt is reasonable and needs focused improvement. For "hard" problems requiring exploration, a `fan_out` + `best_of` construct (best-of-N sampling) would be more appropriate.

---

### 4.3 Quality Metrics for Convergence Assessment

**Traditional NLP metrics:**
| Metric | What it Measures | Use Case |
|--------|-----------------|----------|
| BLEU | N-gram overlap with reference | Translation |
| ROUGE | Recall-oriented n-gram overlap | Summarization |
| BERTScore | Semantic similarity via embeddings | General text |
| Exact Match | Binary correct/incorrect | QA, code |
| Pass@k | Code passes k test cases | Code generation |

**Task-specific composite metrics:**
- Weighted rubric scores (accuracy 30%, relevance 25%, completeness 20%, clarity 15%, conciseness 10%) -- as implemented in `pkg/ai/quality.lx`
- Likert-scale ratings (1-5) per dimension, averaged or weighted
- Binary pass/fail on critical criteria with weighted scoring on non-critical ones

**Feedback quality determines the refinement ceiling:**
- High-quality external feedback (compiler errors, test failures, search results): ceiling approaches 100%
- LLM self-evaluation only: ceiling is bounded by the model's ability to detect its own errors (the Huang et al. paradox)
- Human evaluation: highest ceiling but highest cost; typically used as a meta-evaluation of automated metrics

---

## 5. Related Constructs in Programming and Mathematics

### 5.1 Fixpoint Iteration

**Mathematical foundation:** Apply a function f repeatedly until f(x) = x (the fixpoint).

**Knaster-Tarski theorem:** Every monotone function on a complete lattice has a least fixpoint, computable by iterated application from bottom. This is the theoretical foundation for dataflow analysis, abstract interpretation, and recursive type definitions in programming languages.

**Banach fixed-point theorem:** If f is a contraction mapping (|f(x) - f(y)| <= k|x-y| for k < 1), then f has a unique fixpoint and iteration converges to it from any starting point. The convergence rate is geometric with ratio k.

**Connection to `refine`:** lx's `refine` is a fixpoint iteration where `revise(grade(work))` is the function being iterated. The `threshold` parameter defines the "close enough to fixpoint" criterion. The `max_rounds` parameter provides the finite-lattice guarantee -- even if convergence is slow, we stop after bounded iterations.

### 5.2 Do-While / Repeat-Until

`refine` most closely resembles a `repeat-until` loop with structured state:
```
do {
    work = revise(work, grade(work).feedback)
} until (grade(work).score >= threshold || round > max_rounds)
```

The key difference: `refine` is *semantically richer* than a plain loop. It carries the `work`, `score`, and `feedback` as structured state, returns a Result type (`Ok`/`Err`) indicating whether the threshold was met, and reports `rounds` taken.

### 5.3 Simulated Annealing

Start broad, narrow down. In the refinement context:
- Early rounds: large revisions, exploring different approaches
- Later rounds: small, targeted fixes
- "Temperature" decreases as score approaches threshold

This pattern emerges naturally when the `revise` function receives a high-scoring work product -- there's less to change, so revisions become smaller.

### 5.4 Genetic Algorithms

Generate-evaluate-select-mutate as a population-based refinement:
- **Generate:** Multiple initial candidates (parallel `refine` calls)
- **Evaluate:** `grade` function scores each
- **Select:** Keep the best
- **Mutate:** `revise` introduces variations

lx's `refine` is the single-individual variant (hill climbing). A population-based variant would combine `refine` with `pmap` and selection.
