# GSD-2: Token Optimization and Cost Management

## Token Profiles

Three profiles coordinate model selection, phase skipping, and context compression:

| Profile | Savings | Models | Phases Skipped | Context |
|---------|---------|--------|----------------|---------|
| `budget` | 40-60% | Sonnet/Haiku | research, reassess, slice-research | Minimal |
| `balanced` | 10-20% | User defaults | slice-research | Standard |
| `quality` | 0% | All phases, full context | None | Full |

### Context Compression Levels

| Level | Includes | Drops |
|-------|----------|-------|
| Minimal | Task plan only | Decisions, requirements, templates, supplements |
| Standard | Task + prior + slice context | Supplements |
| Full | Everything inlined | Nothing |

## Complexity-Based Task Routing

Tasks are classified by heuristic (sub-millisecond, no LLM calls):

| Tier | Criteria | Model |
|------|----------|-------|
| Light | ≤3 steps/files, <500 chars description | Haiku-class |
| Standard | 4-7 steps/files | Sonnet-class |
| Heavy | ≥8 steps/files, complexity keywords | Opus-class |

Budget pressure auto-downgrades tiers:

| Budget Used | Adjustment |
|-------------|-----------|
| 50% | Downgrade heavy → standard |
| 75% | Downgrade standard → light |
| 90% | All units → light tier |

## Dynamic Model Routing (v2.19)

Three tiers with **downgrade-only** semantics (user config is ceiling, never upgrades beyond):

| Tier | Default Models | Unit Types |
|------|---------------|------------|
| Light | Haiku-class | complete-slice, run-uat |
| Standard | Sonnet-class | research, plan, execute, complete |
| Heavy | Opus-class | replan, reassess |

### Task Plan Analysis for execute-task

Classifies based on: step count, file count, description length, code blocks, complexity keywords.

### Adaptive Learning

Rolling 50-entry window in `.gsd/routing-history.json`:
- Automatic outcomes: success/failure with context
- User feedback: "over"/"under"/"ok" with 2× weighting vs automatic outcomes
- History informs future tier selection

### Built-in Cost Table

Per-model input/output pricing per million tokens. Enables cost-optimal routing when `cross_provider: true`.

### Interaction with Token Profiles

Profiles set baselines; dynamic routing optimizes within those constraints.

## Context Optimization Techniques

### Prompt Compression (v2.29.0)

Deterministic compression before section-boundary truncation. Applied at budget/balanced levels.

### Smart Context Selection

Two modes for files over 3KB:
- `full` — Entire file inlined
- `smart` — TF-IDF semantic chunking includes only task-relevant portions (saves ~20-40% tokens)

### Summary Distillation

For 3+ dependency summaries: compress by section boundary, distill key information, respect budget constraints.

### Structured Data Compression

At budget/balanced levels, structured data (JSON, YAML) in context is compressed.

### Cache Hit Rate Tracking

Per-unit tracking of prompt cache effectiveness (Anthropic cache read/write tokens).

## Cost Management

### Tracking

Per-unit metrics captured:
- Tokens: input, output, cache read, cache write
- USD cost
- Duration
- Tool calls
- Message counts

Stored in `.gsd/metrics.json`.

### Dashboard

`Ctrl+Alt+G` or `/gsd status` shows:
- Aggregations by phase, slice, model
- Project totals
- Cost projections after 2+ slices

### Budget Enforcement

| Mode | Behavior |
|------|----------|
| `warn` | Log warning, continue |
| `pause` (default) | Stop auto mode |
| `halt` | Refuse to dispatch |

### Cost Projections

After 2+ completed slices: per-slice average × remaining slices = projected total cost.

## Configuration

```yaml
token_profile: balanced          # budget / balanced / quality
budget_ceiling: 50.00            # USD
budget_enforcement: pause        # warn / pause / halt

models:
  research: claude-sonnet-4-6
  planning:
    model: claude-opus-4-6
    fallbacks:
      - openrouter/z-ai/glm-5
  execution: claude-sonnet-4-6
  execution_simple: claude-haiku-4-5  # Light tasks
  completion: claude-sonnet-4-6
  subagent: claude-sonnet-4-6

dynamic_routing:
  enabled: false
  tier_models:
    light: [claude-haiku-4-5]
    standard: [claude-sonnet-4-6]
    heavy: [claude-opus-4-6]
  escalate_on_failure: true
  budget_pressure: true
  cross_provider: false
```

## Model Selection Priority

1. Token profile → model defaults + phase skipping
2. Explicit preferences override profile defaults
3. Complexity classification → tier selection
4. Dynamic routing → cheapest model for tier
5. Budget pressure → downgrade overrides
6. Fallback chain → try next model on failure
