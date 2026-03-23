# Heretic: Fully Automatic Directional Ablation for Language Models

Heretic demonstrates that **automated hyperparameter optimization applied to mechanistic interventions** can match or exceed hand-tuned model modifications, achieving near-zero refusal rates while preserving 84% lower KL divergence than manual approaches. The project's architecture -- combining LoRA-based weight intervention with Optuna's TPE sampler -- provides a template for any system that needs to search over parametrized model transformations without retraining.

## Repository Overview

| Metric | Value |
|--------|-------|
| **GitHub** | [p-e-w/heretic](https://github.com/p-e-w/heretic) |
| **Stars** | 12,601 |
| **Forks** | 1,292 |
| **Language** | Python |
| **License** | AGPL-3.0 |
| **Created** | September 21, 2025 |
| **Latest Version** | v1.2.0 (February 14, 2026) |
| **PyPI Package** | `heretic-llm` |
| **Python** | >= 3.10 |
| **Primary Author** | Philipp Emanuel Weidmann (86/104 commits) |
| **Open Issues** | 80 |
| **HuggingFace Models** | 90+ community-created, 1,000+ total claimed |

## What It Does

Heretic performs **abliteration** -- the systematic removal of safety-alignment refusal behavior from transformer language models through directional ablation of internal weight matrices. Unlike fine-tuning or RLHF reversal, abliteration works by identifying **refusal direction vectors** in the model's residual stream and orthogonalizing weight matrices against those directions.

The key innovation is **full automation**: the user provides only a model identifier (e.g., `heretic Qwen/Qwen3-4B-Instruct-2507`), and Heretic handles hardware detection, batch size optimization, refusal direction computation, multi-objective hyperparameter search, and model export. This contrasts with earlier abliteration tools that required manual parameter selection and transformer internals knowledge.

## Architecture

The codebase is compact (7 Python modules in `src/heretic/`) with clear separation of concerns:

| Module | Responsibility |
|--------|---------------|
| `main.py` | CLI entry point, optimization loop, Optuna study management, post-processing actions |
| `model.py` | Model loading, LoRA application, abliteration algorithm, residual extraction, weight reset |
| `evaluator.py` | Refusal detection via pattern matching, KL divergence measurement, composite scoring |
| `config.py` | Pydantic-based settings with 5-layer config source hierarchy (CLI > env > TOML) |
| `analyzer.py` | Residual geometry analysis, PaCMAP visualization, silhouette coefficient computation |
| `utils.py` | Dataset loading, memory management, interactive prompts, batch utilities |
| `__init__.py` | Empty |

### Control Flow

1. **Settings resolution**: CLI args > environment variables > TOML file > defaults
2. **Device detection**: CUDA, XPU, MLU, SDAA, MUSA, NPU, MPS, CPU fallback
3. **Batch size auto-tuning**: Iteratively doubles from 1, measures tokens/second, selects peak throughput
4. **Residual extraction**: Forward pass on harmful/harmless prompt sets, capture hidden states at each layer's last token position
5. **Refusal direction computation**: `refusal_direction = normalize(mean(bad_residuals) - mean(good_residuals))` per layer
6. **Optuna optimization loop**: 200 trials (60 startup/exploration), TPE sampler with 128 EI candidates, multi-objective minimization of (refusal_count, kl_divergence)
7. **Pareto front selection**: Sort by refusal count ascending, filter for monotonically decreasing KL divergence
8. **Export**: Save LoRA adapter, merged model, or upload to HuggingFace with automatic model card generation

### The Abliteration Algorithm

The core mathematical operation modifies transformer weights through rank-1 LoRA adapters rather than direct weight mutation:

For each abliterable module (attention output projections, MLP down projections) at layer `l`:

- Compute a **layer-specific weight** using a kernel function parameterized by `max_weight`, `max_weight_position`, `min_weight`, and `min_weight_distance`
- **LoRA B** = `-weight * refusal_direction`
- **LoRA A** = `refusal_direction^T @ W` (where W is the base weight matrix, optionally dequantized from 4-bit)
- The effective transformation is: `delta_W = -weight * v * (v^T @ W)`, which projects out the refusal direction component

This LoRA-based approach enables **fast iteration** -- resetting between optimization trials requires only zeroing adapter weights rather than reloading the full model from disk.

### Direction Interpolation

A distinctive design choice: `direction_index` is **float-valued**, not integer. When set to e.g. 7.3, the refusal direction is linearly interpolated between layer 7 and layer 8's computed directions. This enables the optimizer to explore a continuous space of refusal vectors rather than being constrained to discrete layer choices.

### Row Normalization Strategies

Three approaches handle the magnitude distortion caused by directional ablation:

| Strategy | Method | Tradeoff |
|----------|--------|----------|
| **NONE** | Direct ablation | Fast, but may distort weight norms |
| **PRE** | Multiply LoRA B by original row norms | Preserves magnitudes before ablation |
| **FULL** | SVD-based low-rank approximation after normalization | Best magnitude preservation, higher compute cost |

## Key Design Decisions

**LoRA over direct weight modification.** Earlier abliteration tools modified weights in-place, requiring full model reload between optimization trials. Heretic's LoRA approach reduces per-trial overhead from minutes to milliseconds. The rank-1 adapter exactly represents the rank-1 directional projection.

**Multi-objective optimization over scalarization.** Rather than combining refusal rate and KL divergence into a single score, Heretic uses Optuna's multi-objective support to maintain a Pareto front. This lets users choose their preferred compliance-quality tradeoff after optimization completes.

**Pattern-based refusal detection over classifier-based.** The evaluator uses 30+ string markers ("sorry", "cannot", "harmful", "violat", etc.) rather than a trained classifier. This is faster but less robust -- issue #224 proposes BERT-based classification as an improvement.

**Automated batch size discovery.** Rather than requiring users to guess VRAM-appropriate batch sizes, Heretic iteratively doubles batch sizes while measuring throughput, catching OOM errors to find the optimal point. This is essential for a "fully automatic" UX.

**Configuration source hierarchy.** Five-layer precedence (init override > CLI > env > dotenv > TOML) via Pydantic BaseSettings enables both interactive and programmatic use.

**Thinking model support.** Prefix detection identifies common response beginnings (including `<think>`, `<|channel|>` tags) and strips them before refusal classification, preventing thinking-model artifacts from corrupting evaluation.

## Optimization Parameters

Each trial explores a 10+ dimensional space:

| Parameter | Range | Purpose |
|-----------|-------|---------|
| `direction_scope` | "global" or "per_layer" | Single vs. layer-specific refusal directions |
| `direction_index` | 0.4-0.9 * num_layers | Which layer(s) provide the refusal direction |
| `max_weight` | 0.8-1.5 | Peak ablation strength |
| `max_weight_position` | 0.6-1.0 * num_layers | Layer where ablation peaks |
| `min_weight` | 0.0-1.0 * max_weight | Minimum ablation strength |
| `min_weight_distance` | 1.0-0.6 * num_layers | Kernel width |

These parameters are duplicated for attention vs. MLP components, allowing the optimizer to treat them independently.

## Performance and Results

**Gemma-3-12B benchmark** (from README):

| Method | Refusals (out of 100) | KL Divergence |
|--------|----------------------|---------------|
| Heretic (automated) | 3 | 0.16 |
| Manual abliteration A | comparable | 0.45 |
| Manual abliteration B | comparable | 1.04 |

Heretic achieves **comparable refusal suppression with 64-84% lower KL divergence**, meaning the abliterated model deviates less from the original's output distribution. This is the central quality claim: automated search finds better compliance-quality tradeoffs than human intuition.

**Timing**: ~45 minutes for Llama-3.1-8B on RTX 3090 with default settings (200 trials).

## Strengths

- **Zero-configuration UX**: Single command with just a model name produces an abliterated model
- **Pareto-optimal results**: Multi-objective optimization systematically outperforms hand-tuned parameters
- **Memory efficient**: bitsandbytes 4-bit quantization + LoRA adapters keep VRAM requirements manageable
- **Resumable**: Optuna study checkpointing via JSONL allows stopping and resuming optimization
- **Multi-GPU support**: Accelerate's device_map distributes model across available GPUs
- **Research tooling**: PaCMAP residual visualization and geometric analysis provide interpretability
- **Broad model support**: Works with vision-language models, MoE architectures, thinking models

## Weaknesses

- **Refusal detection is brittle**: Pattern matching against 30 string markers misses nuanced refusals and produces false positives. Issue #224 proposes BERT-based classification
- **MoE model limitations**: Qwen3.5 hybrid attention and sparse MoE architectures cause failures (issues #218, #219, #221, #222)
- **OOM on large models**: Even H100 80GB GPUs report OOM on 27B+ parameter models (issue #220), suggesting batch size auto-tuning underestimates later-stage memory needs
- **Single-direction assumption**: The rank-1 ablation assumes refusal behavior is captured by a single direction per layer. Issue #211 (Arbitrary-Rank Ablation) and #217 (Optimal Transport) explore multi-directional alternatives
- **No capability benchmarks**: KL divergence is a proxy for capability preservation, but no downstream task benchmarks (MMLU, HumanEval, etc.) validate that model intelligence is actually preserved
- **AGPL license**: Copyleft requirement may limit commercial adoption and integration

## The "NoSlop" Configuration

Beyond safety abliteration, `config.noslop.toml` demonstrates the framework's generality: it abliterates **purple prose** patterns from writing models. Using the same optimization machinery with different prompt sets (clean vs. flowery writing), it suppresses tendencies toward "ethereal", "celestial", "tapestry" and ~100 other cliched markers. This shows the directional ablation framework is a general tool for suppressing any identifiable behavioral direction, not just safety alignment.

## Relation to Agentic AI Patterns

### Harnesses and Orchestration

Heretic's architecture is a **search harness over model transformations** -- a pattern directly applicable to agentic AI systems. The optimization loop (propose parameters -> apply transformation -> evaluate -> update search) mirrors how agent harnesses iterate over tool selections, prompt strategies, or planning steps. Key transferable patterns:

- **Fast rollback via adapters**: LoRA zeroing instead of model reload is analogous to lightweight state rollback in agent systems. Any agentic system that needs to explore multiple strategies benefits from cheap undo operations
- **Multi-objective Pareto selection**: Rather than forcing a single objective, maintaining a frontier of tradeoffs and letting the user/system choose afterward. This applies to any agent that must balance competing goals (cost vs. quality, speed vs. accuracy)
- **Checkpoint-and-resume**: The JSONL-based study persistence pattern applies to long-running agent workflows that may be interrupted

### Context Management

The residual extraction pipeline demonstrates a specific approach to **model-internal context analysis**: capturing per-layer hidden states to identify behavioral directions. For agentic AI, this suggests:

- **Behavioral steering via internal representations**: Rather than prompt engineering, directly modifying model internals based on identified feature directions
- **Interpretability as a feedback signal**: Using geometric analysis (cosine similarity, silhouette coefficients) of internal representations as optimization targets

### Tool Orchestration

Heretic orchestrates multiple subsystems (PyTorch, Transformers, PEFT, Optuna, bitsandbytes, Accelerate) through a unified interface. The configuration hierarchy (CLI > env > TOML) and automatic hardware detection are patterns for any multi-tool agent that must adapt to heterogeneous environments.

### Multi-Agent Coordination

While Heretic itself is single-agent, its **automated pipeline** replaces what was previously a multi-step human workflow (extract directions -> select layers -> tune weights -> evaluate -> iterate). This compression of multi-step expert workflows into automated search is a core agentic AI pattern.

## Practical Takeaways

1. **LoRA as a reversible intervention mechanism** enables fast iteration in any optimization loop over model modifications. Zeroing adapter weights is orders of magnitude cheaper than model reload
2. **Float-valued interpolation over discrete choices** (layer indices, direction blending) converts combinatorial search into continuous optimization, dramatically improving sample efficiency
3. **Multi-objective optimization with deferred selection** is strictly better than premature scalarization when the tradeoff preferences are unknown at search time
4. **Pattern-based evaluation is a bootstrap strategy**, not a final solution. Fast but brittle evaluation enables rapid iteration; more robust evaluation (classifiers, downstream benchmarks) should follow
5. **The directional ablation framework generalizes beyond safety**: any behavioral tendency that manifests as a direction in residual space can be suppressed. This makes it a general-purpose model behavior editor
6. **Automated search over mechanistic interventions** consistently outperforms expert intuition on compliance-quality tradeoffs. The 64-84% KL divergence reduction over manual methods is a strong argument for optimization over hand-tuning
7. **Compact, well-factored codebases scale**: 7 modules with clear responsibilities support 12,600+ stars and 1,000+ derivative models. The architecture's clarity enables community contribution despite a single dominant author

## Release History

| Version | Date | Highlights |
|---------|------|------------|
| v1.0.1 | Nov 16, 2025 | First public release |
| v1.1.0 | Dec 10, 2025 | Apple Silicon (MPS), multi-GPU, notebook support, thinking models, early stopping |
| v1.2.0 | Feb 14, 2026 | LoRA-based abliteration engine, 4-bit quantization, vision-language models, magnitude-preserving orthogonal ablation, study resume |

## Dependencies

| Package | Version | Purpose |
|---------|---------|---------|
| transformers | ~4.57 | Model loading and inference |
| accelerate | ~1.10 | Multi-device distribution |
| peft | ~0.14 | LoRA adapter management |
| huggingface-hub | ~0.34 | Model upload/download |
| optuna | ~4.5 | Bayesian hyperparameter optimization |
| datasets | ~4.0 | Prompt dataset loading |
| bitsandbytes | ~0.45 | 4-bit quantization |
| pydantic-settings | ~2.10 | Configuration management |
| rich | ~14.1 | Terminal UI and progress display |
| questionary | ~2.1 | Interactive prompts |

Optional research dependencies: geom-median, matplotlib, scikit-learn, numpy, PaCMAP, imageio.

## Sources

- [GitHub Repository](https://github.com/p-e-w/heretic)
- [PyPI Package](https://pypi.org/project/heretic-llm/)
- [HuggingFace Models (heretic tag)](https://huggingface.co/models?search=heretic+abliterated)
- [v1.2.0 Release Notes](https://github.com/p-e-w/heretic/releases/tag/v1.2.0)
- [v1.1.0 Release Notes](https://github.com/p-e-w/heretic/releases/tag/v1.1.0)
- [Issue #211: Arbitrary-Rank Ablation](https://github.com/p-e-w/heretic/issues/211)
- [Issue #217: Optimal Transport Refusal Ablation](https://github.com/p-e-w/heretic/issues/217)
- [Issue #224: BERT-based Refusal Classification](https://github.com/p-e-w/heretic/issues/224)
- [config.default.toml](https://github.com/p-e-w/heretic/blob/master/config.default.toml)
- [config.noslop.toml](https://github.com/p-e-w/heretic/blob/master/config.noslop.toml)
