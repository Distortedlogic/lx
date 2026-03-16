# Refinement Loops — Reference

## `refine` Expression

First-class keyword for the try-grade-revise-regrade pattern.

```
result = refine initial_work {
  grade: (work) -> grader ~>? {task: "evaluate" work} ^
  revise: (work feedback) -> worker ~>? {task: "revise" work feedback} ^
  threshold: 85
  max_rounds: 3
}
```

### Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `grade` | `(work) -> {score: Int feedback: Any}` | Yes | Evaluate current work |
| `revise` | `(work feedback) -> work'` | Yes | Produce improved work |
| `threshold` | Int | Yes | Minimum `score` to pass |
| `max_rounds` | Int | Yes | Maximum revision attempts |
| `on_round` | `(round work score) -> ()` | No | Callback after each round (logging/tracing) |

### Execution Order

1. `grade(initial_work)`
2. If `score >= threshold` -> return `Ok` immediately
3. `revise(work, feedback)` -> produce improved version
4. `grade(revised_work)`
5. If `score >= threshold` -> return `Ok`
6. If `round >= max_rounds` -> return `Err`
7. Repeat from step 3

### Return Value

- **Success**: `Ok {work rounds final_score}`
- **Exhausted**: `Err {work rounds final_score reason: "max_rounds"}`

### Example

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
