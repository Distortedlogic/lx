# Feedback Loops (Refinement Primitive)

First-class `refine` construct for the try-grade-revise-regrade pattern. This is the most common pattern in every agentic workflow — currently hand-rolled as 15-20 lines of mutable bindings, loops, grader calls, and conditionals every time.

## Problem

Every flow in `flows/` contains this pattern:

```
with result := initial_work {
  with round := 0 {
    with done := false {
      while (round < 3 && !done) {
        grade = grader ~>? {task result} ^
        grade.score >= 85 ? {
          true  -> done <- true
          false -> {
            result <- worker ~>? {revise: result feedback: grade.feedback} ^
            round <- round + 1
          }
        }
      }
      result
    }
  }
}
```

This is boilerplate. The intent is simple: refine work until it passes. The implementation obscures it.

## `refine` Expression

```
result = refine initial_work {
  grade: (work) -> grader ~>? {task: "evaluate" work} ^
  revise: (work feedback) -> worker ~>? {task: "revise" work feedback} ^
  threshold: 85
  max_rounds: 3
}
```

`refine` is a new keyword — an expression that returns `Result`:
- `Ok {work rounds final_score}` on passing
- `Err {work rounds final_score reason: "max_rounds"}` on exhaustion

### Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `grade` | `(work) -> {score: Int feedback: Any}` | Yes | Evaluate current work. Must return record with `score` and `feedback`. |
| `revise` | `(work feedback) -> work'` | Yes | Produce improved work from current work + feedback. |
| `threshold` | Int | Yes | Minimum `score` to pass. |
| `max_rounds` | Int | Yes | Maximum revision attempts before giving up. |
| `on_round` | `(round work score) -> ()` | No | Callback after each round. For logging/tracing. |

### Execution

1. `grade(initial_work)` — evaluate starting quality
2. If `score >= threshold`, return `Ok` immediately (work already good enough)
3. `revise(work, feedback)` — produce improved version
4. `grade(revised_work)` — evaluate again
5. If `score >= threshold`, return `Ok`
6. If `round >= max_rounds`, return `Err`
7. Repeat from step 3

### Incremental Grading

The `grade` function can return per-category scores. On revision, only failed categories need re-evaluation:

```
result = refine draft {
  grade: (work) -> {
    cats = rubric | map (cat) {name: cat.name score: (evaluate cat work)}
    failed = cats | filter (c) c.score < cat.threshold
    {score: (cats | map (.score) | avg) feedback: failed categories: cats}
  }
  revise: (work feedback) -> worker ~>? {
    task: "fix only these categories"
    work
    failed: feedback
  } ^
  threshold: 85
  max_rounds: 3
}
```

### Composition

`refine` composes with pipes and error propagation:

```
final = generate_draft task ^
  | refine {
    grade: quality_check
    revise: improve
    threshold: 90
    max_rounds: 5
  } ^
  | (.work)
```

### With Diminishing Returns Detection

```
result = refine draft {
  grade: evaluate
  revise: improve
  threshold: 90
  max_rounds: 5
  on_round: (round work score) {
    round > 1 && score - prev_score < 2 ? {
      true  -> emit "diminishing returns at round {round}"
      false -> ()
    }
  }
}
```

## Implementation

`refine` is syntactic sugar over a loop. The interpreter desugars it to the mutable-binding pattern shown in the Problem section. No new runtime machinery needed — just AST support and interpreter evaluation.

### AST Node

```
Refine {
  initial: Expr
  grade: Expr
  revise: Expr
  threshold: Expr
  max_rounds: Expr
  on_round: Option<Expr>
}
```

### Parser

`refine` keyword followed by expression (initial work), then `{` with named fields `}`. Uses the same record-like syntax but with known field names.

## Cross-References

- Task state machine: [agents-plans.md](agents-plans.md) (plan steps use refine internally)
- Grader agent: stdlib_roadmap (`std/agents/grader`)
- Auditor agent: stdlib_roadmap (`std/agents/auditor`)
- Circuit breakers: [stdlib-introspect.md](stdlib-introspect.md) (max_rounds is a circuit breaker)
- Diminishing returns: [agents-progress.md](agents-progress.md)
