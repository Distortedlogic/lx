# Iterative Refinement: Design Patterns and Implementation

Practical patterns for generate-critique-revise loops, grading rubrics, termination strategies, and their implementation in lx's `refine` construct.

## 1. The Generate-Critique-Revise Loop

### 1.1 Basic Pattern

The fundamental loop has three phases:

```
generate -> critique -> revise -> critique -> revise -> ... -> done
```

**In lx syntax:**
```lx
result = refine initial_draft {
  grade: (work) { score: evaluate(work)  feedback: explain_gaps(work) }
  revise: (work feedback) improve(work, feedback)
  threshold: 80
  max_rounds: 5
}
```

**Phase responsibilities:**
- **Generate** (`initial` expression): Produce the first attempt. Quality here matters -- a better initial draft means fewer refinement rounds and a higher ceiling.
- **Critique** (`grade` function): Evaluate the current work and produce structured feedback. Must return `{score: Int, feedback: ...}`. The quality of the critique is the single most important factor in the refinement loop's effectiveness (Huang et al., 2023).
- **Revise** (`revise` function): Accept current work and feedback, produce improved version. Must actually address the feedback, not just regenerate.

### 1.2 Variations on the Basic Loop

**Single-model (Self-Refine pattern):**
One LLM plays all three roles. Cheapest, simplest, but limited by the self-correction paradox -- the model that made the mistake may not be able to detect or fix it.

**Dual-model (critic separation):**
Use a stronger/different model for grading than for generation. This breaks the self-correction paradox because the evaluator has capabilities the generator lacks. Cost-effective variant: use a large model for critique (easier task) and a smaller model for generation.

**Multi-model (CRITIC pattern):**
Separate generator, tool-using critic, and reviser. The critic invokes external tools (compilers, search engines, test suites) to ground its evaluation in objective reality.

**Multi-agent (lx native pattern):**
Separate agents for each role, communicating via message passing:
```lx
-- from pkg/kit/grading.lx
grade_draft = (worker task draft opts) {
  refine draft {
    grade: (work) grader.grade {work  task}
    revise: (work feedback) worker ~>? {action: "revise"  work  feedback} ^
    threshold: opts.threshold ?? 95
    max_rounds: opts.max_rounds ?? 5
  }
}
```

---

## 2. Grading Rubric Design

### 2.1 Rubric Structure

A grading rubric decomposes evaluation into weighted dimensions. Each dimension has a name, description, weight, and scoring criteria.

**Response rubric (from `pkg/ai/quality.lx`):**
| Dimension | Description | Weight |
|-----------|-------------|--------|
| accuracy | Factually correct, no hallucinations | 30% |
| relevance | Directly answers what was asked | 25% |
| completeness | Covers all aspects of the question | 20% |
| clarity | Clear, well-structured, easy to follow | 15% |
| conciseness | No unnecessary filler or repetition | 10% |

**Code rubric (from `pkg/ai/quality.lx`):**
| Dimension | Description | Weight |
|-----------|-------------|--------|
| correctness | Code works as intended | 35% |
| safety | No security vulnerabilities | 25% |
| style | Clean, idiomatic, no unnecessary complexity | 15% |
| completeness | Handles edge cases, error paths | 15% |
| scope | Only changes what was asked, nothing extra | 10% |

### 2.2 Scoring Approaches

**Likert scale (1-5):**
Each dimension gets a 1-5 integer score with explicit descriptions per level.
- 1: Completely fails the criterion
- 2: Major deficiencies
- 3: Acceptable with notable gaps
- 4: Good with minor issues
- 5: Excellent, no meaningful improvements possible

Prometheus (Kim et al., 2023) demonstrated that fine-grained rubrics with explicit per-level descriptions achieve 0.897 Pearson correlation with human evaluators.

**Pass/fail with weighted scoring:**
Critical dimensions (correctness, safety) are binary pass/fail gates. Non-critical dimensions (style, conciseness) use weighted scoring. Overall pass requires all gates passed AND weighted score above threshold.

**Continuous (0-100):**
G-Eval (Liu et al., 2023) uses probability-weighted continuous scores extracted from token logprobs. More granular than Likert, but requires access to model logprobs.

### 2.3 Rubric Design Principles

**From G-Eval and Prometheus research:**

1. **Explicit criteria per score level.** Don't just name the dimension; describe what a 1, 3, and 5 look like. This calibrates the evaluator and produces reproducible scores.

2. **Chain-of-thought evaluation.** The evaluator should reason through each criterion before assigning a score (G-Eval pattern). This produces better-calibrated scores and more actionable feedback.

3. **Separate score from feedback.** The score is for the convergence check (`score >= threshold`). The feedback is for the reviser. These serve different consumers and should be generated independently.

4. **Weight critical dimensions higher.** In code evaluation, correctness (35%) matters more than style (15%). Weight allocation should reflect the task's priorities.

5. **Include negative criteria.** "No hallucinations" is more actionable than "be accurate." Negative criteria create clear failure modes that are easier to detect and fix.

6. **Task-specific rubrics over generic ones.** A rubric for "summarize this document" should evaluate coverage, faithfulness, and compression ratio. A generic "quality" rubric misses domain-specific requirements.

### 2.4 Known Biases and Mitigations

**From LLM-as-Judge research (Gu et al., 2025):**

| Bias | Description | Mitigation |
|------|-------------|------------|
| Self-preference | LLMs rate their own outputs higher | Use different model for evaluation vs generation |
| Position | First/last options favored in pairwise comparison | Randomize order, average across orderings |
| Verbosity | Longer outputs get higher scores | Include conciseness in rubric, normalize by length |
| Anchoring | Prior scores influence subsequent ones | Evaluate each sample independently, reset context |
| Leniency | LLMs tend to give generous scores | Include calibration examples with harsh scoring |

---

## 3. Termination Strategies

### 3.1 Threshold-Based Termination

The simplest strategy: stop when `score >= threshold`.

```lx
refine draft {
  grade: grader
  revise: reviser
  threshold: 80    -- stop when score hits 80
  max_rounds: 5    -- hard cap for safety
}
```

**Choosing the threshold:**
- Too low (< 60): Accepts mediocre output, wastes the refinement opportunity
- Sweet spot (70-85): Allows meaningful improvement without demanding perfection
- Too high (> 90): May never converge, exhausting `max_rounds` on diminishing returns
- Domain-dependent: Code correctness may need 95+ (compilation is binary); creative writing may be fine at 70

**lx implementation detail:** The `refine` construct returns `Ok {...}` when threshold is met, `Err {reason: "max_rounds" ...}` when rounds are exhausted. The caller can pattern-match on the result to handle both cases:
```lx
result ? { Ok r -> r.work; Err r -> fallback(r.work) }
```

### 3.2 Round-Limited Termination

Hard cap on iteration count, independent of score.

**Why you always need `max_rounds`:**
- Cost control (each round costs API calls / compute)
- Prevents infinite loops when the grade function has bugs
- Bounds worst-case latency
- Prevents oscillation from consuming unbounded resources

**Empirical guidance:**
- 2 rounds capture ~75% of achievable improvement (Yang et al., 2025)
- 3-5 rounds is the typical sweet spot for most tasks
- Beyond 5-6 rounds, diminishing returns dominate
- For code compilation loops, up to 10 rounds may be appropriate (see `software_diffusion.lx` stage_refine)

### 3.3 Diminishing Returns Detection

Stop when the improvement per round drops below a threshold.

**Implementation pattern using `on_round`:**
```lx
prev_score := 0
min_delta = 5

result = refine draft {
  grade: grader
  revise: reviser
  threshold: 90
  max_rounds: 10
  on_round: (round work score) {
    delta = score - prev_score
    prev_score <- score
    -- Could signal early termination if delta < min_delta
    -- Currently tracked; future: support early exit from on_round
  }
}
```

**The `should_stop` pattern from `pkg/ai/quality.lx`:**
```lx
revise: (work feedback) {
  stop = trace.should_stop {min_delta: 2.0  window: 3}
  stop ? work : improve(work, feedback)
}
```
When the reviser detects diminishing returns via trace history, it returns the work unchanged, causing the next `grade` call to produce the same score, which terminates the loop.

### 3.4 Oscillation Detection

Stop when scores oscillate rather than converge.

**Mathematical basis (Yang et al., 2025):**
Oscillation occurs when `alpha = CL - CS < 0`, meaning the model fixes errors (high CS) but also breaks correct content (low CL). The result: scores bounce between rounds rather than monotonically improving.

**Detection heuristic:**
```
If score[t] < score[t-1] for 2 consecutive rounds, stop and return best-scoring version.
```

**Prevention strategies:**
- Conservative revision: Instruct the reviser to make minimal changes, preserving what works
- Diff-based revision: Instead of regenerating the entire work, produce a diff of targeted fixes
- Checkpoint best: Track the highest-scoring version across all rounds, return that instead of the final version

### 3.5 Convergence Ceiling

**From the mathematical model (Section 4.1 of landscape.md):**

The theoretical accuracy ceiling is:
```
Upp = CS / (1 - CL + CS)
```

If `Upp < threshold`, the refinement loop will **never** converge. The system should detect this condition early and either:
- Lower the threshold
- Improve the `revise` function (increase CS)
- Improve content preservation (increase CL)
- Switch to a fundamentally different approach (human intervention, different model, different strategy)

---

## 4. Implementation Patterns

### 4.1 Single-Pass Generation + Refinement

The most common pattern. Generate once, then refine.

```lx
draft = agent.ask "Write a function that sorts a list"

result = refine draft {
  grade: grade_code
  revise: (code feedback) agent.ask "Fix this code:\n{code}\n\nIssues:\n{feedback}"
  threshold: 85
  max_rounds: 3
}
```

**When to use:** Most agentic tasks. The initial generation provides a strong starting point, and refinement handles edge cases and quality issues.

### 4.2 Multi-Agent Refinement

Separate generator, critic, and reviser agents with different capabilities.

```lx
-- from pkg/kit/grading.lx
grade_run = (script init_msg task opts) {
  worker = agent.spawn {command: "lx" args: ["run" script]} ^
  draft = worker ~>? init_msg ^
  result = grade_draft worker task draft opts
  agent.kill worker ^
  result
}
```

**Advantages:**
- Critic can be a stronger model than generator (cost-effective: critique is easier than creation)
- Reviser can be specialized for the domain
- Agents can maintain persistent state across rounds
- Natural parallelism: critic and reviser can run on different models/machines

**When to use:** Complex tasks where a single model's self-evaluation is insufficient, or when the critique requires tool use that the generator doesn't support.

### 4.3 Grounded Refinement

Use external tools/validators as the critic instead of (or in addition to) LLM self-evaluation.

**Examples of grounding signals:**
- **Compiler output** for code refinement: `cargo check` errors as feedback
- **Test suite results** for implementation refinement: which tests pass/fail
- **Search engine results** for factual accuracy: verify claims against sources
- **Linter output** for style refinement: clippy warnings as feedback
- **Type checker** for API design refinement: type errors as feedback

**Software Diffusion example (from `flows/examples/software_diffusion.lx`):**
```lx
stage_refine = (stubs skeleton) {
  code := stubs | map (.code) | join "\n"
  loop {
    signal = guidance.check_build "."   -- external compiler signal
    signal.proceed ? break (Ok {code  verified: true})
    fix = dispatch.run_one "subagents/type_fixer.lx" {action: "fix" errors: signal.feedback code}
    code <- fix.code
  }
}
```

This is the most reliable refinement pattern because the feedback signal is objective and external. The compiler doesn't have self-preference bias or verbosity bias. It either compiles or it doesn't.

**Key insight from CRITIC (Gou et al., 2023):** LLMs alone cannot reliably critique their own work. External tools provide the grounding that makes self-correction actually work.

### 4.4 Progressive Refinement

Start with coarse improvements, then focus on specific aspects.

**Round-based focus:**
```lx
grade_progressive = (work round) {
  round <= 2 ? grade_structure(work)   -- first: get the structure right
    : round <= 4 ? grade_details(work)  -- then: fix details
    : grade_polish(work)                -- finally: polish
}
```

**Aspect-by-aspect:**
```
Round 1: Fix factual accuracy
Round 2: Improve completeness
Round 3: Polish clarity and conciseness
```

This mirrors how humans revise: first get the big picture right, then refine details.

### 4.5 Best-of-N Sampling

Generate N candidates in parallel, pick the best. Complementary to iterative refinement.

```lx
candidates = [1..N] | pmap (_) generate(prompt)
scores = candidates | pmap (c) grade(c).score
best = candidates | zip scores | max_by snd | fst
```

**vs Iterative Refinement (from Snell et al., 2024):**
| Factor | Best-of-N | Iterative Refinement |
|--------|-----------|---------------------|
| Problem difficulty | Better for hard problems | Better for easy-medium problems |
| Diversity | High (independent samples) | Low (focused improvement) |
| Compute pattern | Parallel, fixed cost | Sequential, variable cost |
| When initial attempt is close | Wasteful (most samples similar) | Efficient (small targeted fixes) |
| When initial attempt is far off | Explores diverse strategies | May get stuck in local optimum |

**Hybrid approach:** Best-of-N for initial generation, then refine the best candidate:
```lx
candidates = [1..5] | pmap (_) generate(prompt)
best = candidates | max_by (c) grade(c).score
result = refine best { grade  revise  threshold: 90  max_rounds: 3 }
```

### 4.6 Constitutional Prompting During Refinement

Apply principles/rules during the revision step, inspired by Constitutional AI (Bai et al., 2022).

**Pattern:**
```lx
principles = [
  "The response must not contain unverified claims"
  "The response must directly answer the question asked"
  "The response must acknowledge uncertainty where appropriate"
]

revise = (work feedback) {
  -- Sample a random subset of principles per round (CAI pattern)
  active = principles | shuffle | take 2
  agent.ask "Revise this work:\n{work}\n\nFeedback:\n{feedback}\n\nPrinciples to apply:\n{active | join '\n'}"
}
```

**Random principle sampling** (the CAI innovation): Instead of applying all principles every round, sample a subset. This prevents the reviser from being overwhelmed by conflicting instructions and produces more diverse revisions.

### 4.7 Refinement with Memory

Accumulate lessons across rounds to avoid repeated mistakes.

**Reflexion-inspired pattern:**
```lx
lessons := []

result = refine draft {
  grade: grader
  revise: (work feedback) {
    lesson = "Round failed because: {feedback}"
    lessons <- [..lessons lesson]
    agent.ask "Improve:\n{work}\n\nCurrent feedback:\n{feedback}\n\nLessons from prior rounds:\n{lessons | join '\n'}"
  }
  threshold: 80
  max_rounds: 5
}
```

**Key considerations:**
- Lesson accumulation increases prompt length; summarize if rounds > 3
- Avoid recording low-information lessons ("needs improvement" is not actionable)
- Track what was tried and failed to prevent the reviser from reverting to previously-rejected approaches

---

## 5. The `refine` Construct in lx: Design Rationale

### 5.1 Why a First-Class Construct

`refine` is a language-level keyword, not a library function, because:

1. **Structured result type:** Returns `Ok {work, rounds, final_score}` or `Err {work, rounds, final_score, reason}`. This is richer than a plain loop's return value and enables downstream pattern matching.

2. **Semantic clarity:** `refine draft { grade: ... revise: ... threshold: 80 max_rounds: 5 }` is self-documenting. A `while` loop with manual state management obscures the intent.

3. **Composability:** `refine ... ^` propagates errors naturally. The `^` operator unwraps `Ok`, and the caller can handle `Err` at the appropriate level.

4. **Observability:** The `on_round` callback provides a hook for logging, tracing, and monitoring without cluttering the core loop logic.

### 5.2 Current Implementation

**Parser** (`crates/lx/src/parser/refine.rs`):
- Parses `refine <initial> { grade: ... revise: ... threshold: ... max_rounds: ... on_round: ... }`
- `grade`, `revise`, `threshold`, `max_rounds` are required; `on_round` is optional
- Produces `Expr::Refine { initial, grade, revise, threshold, max_rounds, on_round }`

**Interpreter** (`crates/lx/src/interpreter/refine.rs`):
- Evaluates `initial` to get the first work product
- Evaluates `grade`, `revise` to get callable values
- Calls `grade(work)` to get `{score, feedback}`
- If score >= threshold before any rounds: returns `Ok` with rounds=0
- Loops up to `max_rounds`: calls `revise(work)(feedback)` (curried), then `grade(result)`
- Calls `on_round(round, work, score)` after each round if present
- Returns `Ok` if threshold met, `Err {reason: "max_rounds"}` if exhausted

**Key design decisions:**
- `revise` is curried: `revise(work)(feedback)` -- allows partial application
- `grade` returns a record, not just a score -- feedback must accompany the score
- `threshold` and `max_rounds` are expressions, evaluated once at the start
- Initial grade happens before the loop -- if the initial work already passes, zero rounds are consumed

### 5.3 Patterns Mapped to lx Constructs

| Research Pattern | lx Implementation |
|-----------------|-------------------|
| Self-Refine (Madaan) | `refine` with LLM-based `grade` and `revise` |
| CRITIC (Gou) | `refine` with tool-invoking `grade` function |
| Reflexion (Shinn) | `refine` + `on_round` accumulating lessons into revise context |
| Constitutional AI | `revise` function that samples from principle list |
| Best-of-N + refinement | `pmap` + `max_by` + `refine` |
| Grounded refinement | `grade` calls `guidance.check_build` or test runners |
| Progressive refinement | `grade` that varies criteria based on round number |
| Diminishing returns detection | `revise` that calls `trace.should_stop` |

### 5.4 Open Design Questions

**Early exit from `on_round`:**
Currently `on_round` is observational only -- it cannot terminate the loop early. Adding early exit would enable the callback to detect oscillation or diminishing returns and stop the loop before `max_rounds`.

**Multiple grade functions:**
Some patterns call for different grading criteria per round (progressive refinement). Currently this requires the `grade` function to internally track round numbers. A `grade_per_round` parameter or round number passed to grade would make this more ergonomic.

**Best-version tracking:**
The current implementation returns the final version, which may not be the best if oscillation occurred. Tracking and returning the highest-scoring version across all rounds would be more robust.

**Parallel refinement (population-based):**
`refine` operates on a single work product. A `refine_many` or `evolve` construct could maintain a population, select the best, and breed variations -- mapping the genetic algorithm pattern.

**Adaptive threshold:**
Some workflows want to *increase* the threshold across rounds (start lenient, get stricter). Currently `threshold` is fixed at loop entry.

---

## 6. Evaluation Framework Integration

### 6.1 Mapping External Frameworks to lx

**DeepEval-style assertions in `grade`:**
```lx
grade = (work) {
  checks = [
    {name: "length" pass: (len work) > 100}
    {name: "no_hedging" pass: !(work |> contains "I think")}
    {name: "addresses_task" pass: llm_judge(work, task)}
  ]
  failures = checks | filter (c) !c.pass
  score = ((checks | len) - (failures | len)) * 100 / (checks | len)
  feedback = failures | map (.name) | join ", "
  {score  feedback}
}
```

**RAGAS-style metrics for RAG refinement:**
```lx
grade_rag = (response) {
  faithfulness = check_faithfulness response context
  relevancy = check_relevancy response question
  score = (faithfulness * 50 + relevancy * 50) |> to_int
  feedback = "faithfulness: {faithfulness}, relevancy: {relevancy}"
  {score  feedback}
}
```

**Prometheus-style rubric evaluation:**
```lx
grade = (work) {
  rubric = [
    {name: "helpfulness" desc: "Is the response helpful?" weight: 40}
    {name: "accuracy" desc: "Is the response accurate?" weight: 60}
  ]
  scores = rubric | map (r) {
    s = prometheus.evaluate work r.desc  -- returns 1-5
    {name: r.name  score: s * 20  weight: r.weight}  -- normalize to 0-100
  }
  total = scores | map (s) s.score * s.weight | sum | / 100
  worst = scores | min_by (.score)
  {score: to_int total  feedback: "Weakest dimension: {worst.name} ({worst.score})"}
}
```

### 6.2 When to Use Which Pattern

| Scenario | Recommended Pattern |
|----------|-------------------|
| Code generation | Grounded refinement (compiler/tests as critic) |
| Text quality improvement | Single-model Self-Refine or dual-model with rubric |
| Factual accuracy | CRITIC-style with search engine grounding |
| Safety/alignment | Constitutional prompting with principle sampling |
| Creative writing | Best-of-N sampling (diversity matters more than refinement) |
| API response formatting | Pass/fail assertions (structure is binary) |
| Multi-step reasoning | Tree of Thoughts (exploration > refinement) |
| RAG pipeline tuning | RAGAS-style composite metrics |
| General agent output | Weighted rubric with threshold ~80, max_rounds 3-5 |
