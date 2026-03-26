# Karpathy's autoresearch: Deep Dive

## Identity

autoresearch is an experiment in fully autonomous AI-driven ML research. Created by Andrej Karpathy, March 2026. ~57k GitHub stars, MIT license. Written in Python. Tagline: "let coding agents do ML research overnight."

Core thesis: **give a coding agent a single GPU, a training script, and a strict protocol — let it run 100+ experiments autonomously while you sleep.** The agent modifies code, trains for 5 minutes, checks the metric, keeps or discards the change, repeats indefinitely.

## Architecture

The repo is deliberately minimal — 3 files that matter:

| File | Role | Mutability |
|------|------|------------|
| `prepare.py` (~400 lines) | Data prep, BPE tokenizer training (`rustbpe`), dataloader, evaluation function | **Read-only** — agent cannot touch |
| `train.py` (~500 lines) | Full GPT model, MuonAdamW optimizer, training loop, all hyperparameters | **Agent edits this** — sole mutable file |
| `program.md` (~130 lines) | Protocol for the agent — the "agent program" | **Human edits this** — the meta-research program |

Supporting: `pyproject.toml`, `analysis.ipynb` (experiment visualization), `progress.png`.

## The Experiment Loop

`program.md` defines a strict two-phase protocol:

### Setup Phase (human + agent, once)

1. Agree on a run tag (e.g., `mar5`)
2. Create branch `autoresearch/<tag>` from master
3. Agent reads all in-scope files for full context
4. Verify data in `~/.cache/autoresearch/`
5. Initialize `results.tsv` with header row
6. Run baseline (unmodified `train.py`) to establish starting metric

### Autonomous Loop (runs indefinitely)

```
loop {
    check git state
    hack train.py with experimental idea
    git commit
    run: uv run train.py > run.log 2>&1
    extract: grep "^val_bpb:\|^peak_vram_mb:" run.log
    if empty (crash):
        read tail -n 50 run.log
        attempt fix or log crash and move on
    log to results.tsv (commit, val_bpb, memory_gb, status, description)
    if improved: keep commit, advance branch
    if worse or equal: git reset
    NEVER STOP — run until manually interrupted
}
```

Five columns in `results.tsv`: commit hash, val_bpb, memory_gb, status, description.

### Constraints

- **Fixed 5-minute wall-clock budget** per experiment (~12/hour, ~100 overnight)
- **Single metric:** `val_bpb` (validation bits per byte) — lower is better, vocab-size-independent
- **Single mutable file** — only `train.py`
- **No new packages** — cannot install dependencies
- **Immutable evaluation** — cannot modify the evaluation harness in `prepare.py`
- **Simplicity criterion:** "A 0.001 bpb improvement that adds 20 lines of hacky code? Probably not worth it. Deleting code for equal results? Definitely keep."

## The Training Code (What the Agent Iterates On)

The baseline `train.py` is a sophisticated single-GPU GPT:

- Configurable depth/width with aspect ratio (model_dim = depth * 64)
- Grouped query attention (GQA) with configurable n_kv_head
- Value Embeddings (ResFormer-style) with input-dependent gating on alternating layers
- RoPE (rotary position embeddings)
- Sliding window attention pattern (SSSL — 3 short windows then 1 long)
- RMSNorm, ReluSquared MLP activation
- Logit softcapping (tanh at 15)
- Per-layer residual and skip-connection lambdas (learnable scalars)
- MuonAdamW optimizer: polar express orthogonalization (Newton-Schulz), NorMuon variance reduction, cautious weight decay, dimension-aware LR scaling
- Warmup/warmdown LR schedule based on wall-clock progress
- GC disabled after step 0 to avoid 500ms stalls

Tech stack: PyTorch 2.9.1 + CUDA 12.8, `torch.compile`, bfloat16 autocast, Flash Attention 3 via `kernels`, `rustbpe` for tokenizer, `tiktoken` for runtime encoding, `uv` as package manager. Data: `karpathy/climbmix-400b-shuffle` (HuggingFace parquet shards).

## Key Design Decisions

### 1. Markdown as Agent Programming Language

`program.md` IS the program. Karpathy explicitly states: "you are programming the `program.md` Markdown files... not touching any of the Python files like you normally would as a researcher." The human iterates on the markdown instructions; the agent iterates on the research code. Two levels of programming happening simultaneously.

### 2. Single File Mutation Surface

Only `train.py` is mutable. Architecture, optimizer, hyperparameters, batch size, model size — all live in one file. This keeps diffs reviewable, limits blast radius, and makes the search space manageable while remaining broad.

### 3. Wall-Clock Budget as Universal Normalizer

Every experiment runs for exactly 5 minutes regardless of what changes. A larger model with fewer steps is directly comparable to a smaller model with more steps on the same hardware. No need for step-count normalization.

### 4. Git as Experiment Tracking

Each experiment = one git commit. Successful experiments advance the branch. Failed ones are reset. No MLflow, no W&B, no experiment tracking framework. Git IS the tracker. `results.tsv` is the human-readable log.

### 5. Stdout Isolation

`uv run train.py > run.log 2>&1` redirects all output to a file. The agent reads results via `grep` and only reads full logs on crash. Prevents training output from flooding the agent's context window.

### 6. Crash Resilience in Protocol

The program distinguishes "dumb crashes" (typos, missing imports — fix and retry) from "fundamentally broken ideas" (log crash, move on). Fast-fail in training: NaN loss or loss > 100 exits immediately rather than wasting the 5-minute budget.

### 7. Fire-and-Forget Autonomy

The "NEVER STOP" instruction: the agent runs indefinitely, never asks for permission, never pauses for confirmation. The human may be asleep. Only manual interruption terminates the loop.

## What It Is NOT

- **No multi-agent coordination.** Single agent, single loop. (The `.gitignore` ignoring `AGENTS.md`, `queue/`, `worktrees/` hints at multi-agent ambitions.)
- **No literature search.** Purely empirical hill-climbing. "Read papers referenced in the code" is listed as a last resort.
- **No backtracking strategy.** Greedy local search. Each experiment either advances the frontier or gets discarded. No population-based approach, no Bayesian optimization.
- **No orchestration framework.** No SDK, no agent library. Just a markdown file that a coding agent reads and follows.

## Comparison to Agent Frameworks

| Dimension | autoresearch | CrewAI / LangGraph | Julep | lx |
|-----------|-------------|-------------------|-------|-----|
| Agent program format | Markdown (natural language) | Python code | YAML | .lx DSL |
| Orchestration | None — agent self-orchestrates | Framework-managed | Temporal-backed | Language-native |
| Multi-agent | Not yet | Yes | Yes (via tool calls) | Yes (spawn/message) |
| State management | Git + TSV file | Framework state | Temporal + PostgreSQL | Runtime-managed |
| Iteration model | Infinite greedy loop | Configurable | Step sequence | Workflow constructs |
| Constraint enforcement | Natural language rules | Code validation | Expression sandbox | Type system + budgets |
| Complexity | 3 files, ~1,000 lines | Thousands of lines | Docker Compose stack | Language + runtime |

## Patterns Worth Noting

### Program-as-Prompt

The entire "agentic framework" is a text document. No code framework, no SDK, no orchestration library. This works precisely because the task is simple enough (single-agent, single-metric, single-file) to not need structured coordination. It breaks down the moment you need multi-agent collaboration, structured state, or programmatic constraints.

### Environment-as-State

The agent maintains NO internal state across iterations. All state is external: git (code state), `results.tsv` (experiment history), the filesystem (run logs). The agent reconstructs its understanding from these artifacts each iteration.

### Metric-Gated Greedy Search

The pattern `modify → run → measure → keep_or_discard` with a single scalar metric is the simplest possible autonomous optimization loop. It works because: (1) the metric is immutable and trustworthy, (2) the budget is fixed making experiments comparable, (3) the search space is bounded to one file.

### Natural Language Constraints

Rules like "prefer simplicity" and "don't add hacky code for tiny improvements" are enforced purely through natural language instruction. There's no programmatic enforcement — the agent either follows the instruction or doesn't.

## Relevance to lx

**The experiment loop IS a workflow.** The core pattern is `loop { plan → modify → execute → evaluate → branch(keep/discard) }`. In lx this maps directly to a single-agent workflow with tool invocations (git, shell), conditional branching (improved vs. not), and infinite looping with external termination.

**Demonstrates the ceiling and floor of markdown-as-program.** `program.md` works for a single-agent, single-metric loop. But it has no type safety, no constraint enforcement, no composability, no multi-agent coordination. Everything autoresearch can't do is what lx provides. autoresearch is the motivating example for why a structured workflow language exists.

**Wall-clock budgets belong in the language.** The 5-minute-per-experiment constraint is central to autoresearch's design but enforced via training script internals (`time.time()` checks). lx's budget system should make time-bounded execution a first-class primitive — `budget(wall: 5m) { run("train.py") }` rather than hoping the subprocess respects a timeout.

**Git-as-state is a pattern lx should support.** The `commit → run → maybe reset` pattern is a legitimate workflow state management strategy for code-modifying agents. lx could provide git-aware workflow primitives: checkpoint (commit), rollback (reset), branch (for parallel exploration).

**The "NEVER STOP" pattern needs language support.** An infinite loop with no programmatic exit condition — only human interrupt — is poorly modeled by most workflow systems. lx should support `loop { ... } until interrupted` as a first-class construct for long-running autonomous agents.

**Single-metric gating is the simplest refine loop.** autoresearch's keep/discard decision is a degenerate case of lx's refine loop pattern. The generalization: multi-metric evaluation, partial rollback, Pareto frontier tracking. lx's refine constructs should handle the simple case (scalar gate) as naturally as the complex case (structured evaluation).

**Environment-as-state complements agent-internal state.** autoresearch stores everything in the filesystem rather than agent memory. lx workflows that spawn code-modifying agents should support both patterns — agent-managed state (variables, channels) and environment-managed state (filesystem, git, databases) — with explicit primitives for each.

**Scaling autoresearch would require lx.** Multi-agent swarms (`.gitignore` hints at this), work queues, separate worktrees per agent, result aggregation across parallel experiments, coordinated exploration strategies — all of these need the structured orchestration that lx provides and autoresearch deliberately lacks.

## Sources

- https://github.com/karpathy/autoresearch (repo, README, program.md, train.py, prepare.py)
- Karpathy's description: "research is now entirely the domain of autonomous swarms of AI agents"
- nanochat: https://github.com/karpathy/nanochat (the full training codebase autoresearch simplifies)
