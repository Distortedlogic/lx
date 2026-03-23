# Nanochat: Full-Stack LLM Training in a Single Codebase

Nanochat proves that GPT-2-level language models can be trained from scratch, fine-tuned, and deployed with tool use for under $50 on commodity GPU hardware. Created by Andrej Karpathy, it compresses the entire LLM pipeline (tokenizer training, pretraining, SFT, RL, inference with KV cache, tool execution, and a ChatGPT-style web UI) into ~4,000 lines of readable Python. The project's thesis: a single `--depth` integer should be the only knob needed to scale a language model, with all other hyperparameters derived automatically.

## Repository Metrics

| Metric | Value |
|---|---|
| Stars | ~47,800 |
| Forks | ~6,300 |
| Language | Python |
| License | MIT |
| Created | October 13, 2025 |
| Last Push | March 10, 2026 |
| Repo Size | ~1.7 MB |
| Open Issues | 80 |

## Time-to-GPT-2 Leaderboard

The primary benchmark is wall-clock time to reach a DCLM CORE score exceeding 0.256525 on an 8xH100 node.

| Entry | Time | Val BPB | CORE Score | Key Change | Date |
|---|---|---|---|---|---|
| 0 | 168 hrs | -- | 0.2565 | Original OpenAI GPT-2 (2019) | 2019 |
| 1 | 3.04 hrs | 0.74833 | 0.2585 | d24 baseline | Jan 29 2026 |
| 2 | 2.91 hrs | 0.74504 | 0.2578 | d26 + FP8 | Feb 2 2026 |
| 3 | 2.76 hrs | 0.74645 | 0.2602 | 1M token batch | Feb 5 2026 |
| 4 | 2.02 hrs | 0.71854 | 0.2571 | NVIDIA ClimbMix dataset | Mar 4 2026 |
| 5 | 1.80 hrs | 0.71808 | 0.2690 | Autoresearch round 1 | Mar 9 2026 |

At ~$24/hr for an 8xH100 node, the latest entry costs roughly $43 in compute. The original GPT-2 cost $43,000 in 2019 -- a 1000x reduction in 7 years.

## Architecture

### Transformer Model (`nanochat/gpt.py`)

The model is a decoder-only transformer with several modern modifications stacked on top of the GPT-2 skeleton:

- **Group-Query Attention (GQA)** with configurable `n_kv_head` for memory-efficient inference
- **Rotary Position Embeddings (RoPE)** instead of learned absolute positions
- **RMSNorm** (parameterless `F.rms_norm`) instead of LayerNorm
- **ReLU squared** activation in the MLP instead of GELU
- **Sliding Window Attention** via per-layer `window_pattern` string ("L" for long-range, "S" for short-range)
- **Value Residuals (ResFormer-style)** with alternating layers using input-dependent gating
- **Logit Softcapping** via `tanh` to bound output magnitudes
- **Untied Embeddings** -- separate token embedding and lm_head weight matrices
- **Flash Attention 3** for GPU-accelerated attention computation
- **FP8 training** for Linear layers with dimensions >= 128 and divisible by 16

### The Depth Dial

The `--depth` parameter (transformer layer count) drives all other hyperparameters through deterministic formulas:

```
base_dim = depth * aspect_ratio  (aspect_ratio=64)
model_dim = ceil(base_dim / head_dim) * head_dim  (head_dim=128)
num_heads = model_dim / head_dim
```

Training duration, batch size, learning rates, and weight decay are all derived from scaling laws anchored to a reference d12 model:

- **Batch size**: `B_ref * (target_tokens / D_ref)^0.383`, clamped to nearest power-of-2
- **Learning rates**: scaled by `sqrt(batch_size_ratio)` following AdamW theory
- **Weight decay**: `lambda_base * sqrt(B/B_ref) * (D_ref/D)` for consistent T_epoch dynamics
- **Training tokens**: `target_param_data_ratio * scaling_params` (default ratio: 10.5)

GPT-2 capability emerges around depth 24-26.

### Optimizer: MuonAdamW (`nanochat/optim.py`)

A hybrid optimizer that routes parameters to two different update rules:

| Parameter Type | Optimizer | Default LR |
|---|---|---|
| Embedding matrices | AdamW | 0.3 |
| Unembedding (lm_head) | AdamW | 0.008 |
| Scalar parameters | AdamW | 0.5 |
| 2D weight matrices | Muon | 0.02 |

**Muon** performs SGD with momentum, then replaces the update with the nearest orthogonal matrix via the **Polar Express Sign Method** (5 iterations). It includes **NorMuon variance reduction** for per-neuron adaptive scaling and **cautious weight decay** with sign-matching masks.

The distributed variant (`DistMuonAdamW`) implements ZeRO-2-style optimizer state sharding with a three-phase asynchronous pipeline: launch all-reduce, compute updates during communication, launch all-gather.

### Learning Rate Schedule

Three-phase warmup-constant-warmdown:

- **Warmup**: 40 steps linear ramp
- **Constant**: holds at peak LR
- **Warmdown**: 65% of total iterations, linear decay to 5% of peak (pretraining) or 0% (SFT)

## Pipeline Stages

### 1. Tokenizer Training (`scripts/tok_train.py`)

BPE tokenizer trained on ~2B characters with vocabulary size 32,768. Uses a modified GPT-4 split pattern (`\p{N}{1,2}` instead of `\p{N}{1,3}`) optimized for smaller vocabularies. Nine special tokens define the conversation and tool-use protocol:

`<|bos|>`, `<|user_start|>`, `<|user_end|>`, `<|assistant_start|>`, `<|assistant_end|>`, `<|python_start|>`, `<|python_end|>`, `<|output_start|>`, `<|output_end|>`

### 2. Pretraining (`scripts/base_train.py`)

Distributed training across 8 GPUs using DDP with NCCL backend. The dataloader implements **BOS-aligned best-fit packing**: documents are packed into fixed-length sequences starting with BOS tokens, using a best-fit algorithm that searches the buffer for the largest document fitting the remaining capacity. Achieves ~100% token utilization with the tradeoff of ~35% of documents being cropped.

Data sources include FineWeb, SmolTalk, and NVIDIA ClimbMix (parquet format via HuggingFace).

### 3. Supervised Fine-Tuning (`scripts/chat_sft.py`)

Trains on a task mixture combining:

| Dataset | Size | Purpose |
|---|---|---|
| SmolTalk | 460K conversations | General chat |
| Custom Identity JSON | 1K conversations | Model identity/personality |
| MMLU (auxiliary_train) | 100K rows/epoch | Multiple choice reasoning |
| GSM8K | 8K rows/epoch | Math with tool use |
| SimpleSpelling | 200K examples | Character-level tasks |
| SpellingBee | 80K examples | Advanced spelling |

Loss is cross-entropy with **selective masking**: only assistant completion tokens contribute gradients. User prompts, special tokens, and padding are masked with `ignore_index=-1`. The optimizer warm-starts from pretrained momentum buffers but resets learning rates.

### 4. Reinforcement Learning (`scripts/chat_rl.py`)

Simplified GRPO/REINFORCE on GSM8K math problems:

- Generates `num_samples` completions per question with temperature sampling
- Computes scalar rewards via exact-match against ground truth answers
- Advantages: `reward - mean_reward` (no z-score normalization)
- Token-level policy gradient: `loss = -(logp * advantages).sum()`
- No KL regularization, no importance sampling, no PPO clipping

This is explicitly on-policy: each batch is generated from the current model, used once, then discarded.

### 5. Inference Engine (`nanochat/engine.py`)

**KVCache** pre-allocates tensors in shape `(B, T, H, D)` with per-batch position tracking via `cache_seqlens`. Two-phase inference:

1. **Prefill**: batch=1 forward pass over prompt, populating KV cache
2. **Decode**: clone cache to `batch_size=num_samples`, sample tokens autoregressively

**Sampling**: temperature scaling, top-k filtering, softmax + multinomial. Temperature=0 falls back to argmax.

**Tool execution state machine**: when the model emits `<|python_start|>`, the engine accumulates expression tokens until `<|python_end|>`, evaluates the expression, and force-injects the result wrapped in `<|output_start|>` / `<|output_end|>` tokens. The `RowState` class tracks per-sample generation state including forced token queues and Python block context.

### 6. Code Execution (`nanochat/execution.py`)

Two execution modes:

**Calculator** (`use_calculator`): restricted to characters `0-9*+-/() ` plus `.count()`. Blocks `**`, `__`, `import`, `exec`, `eval`. 3-second timeout.

**Full sandbox** (`execute_code`): subprocess isolation with `multiprocessing.Process`, 5-second timeout via `signal.ITIMER_REAL`, 256MB memory cap via `resource.setrlimit`, disabled destructive OS functions (`os.remove`, `os.kill`, `os.fork`, `subprocess.Popen`, etc.). Explicitly documented as "not a true security sandbox" -- network access remains open, `ctypes` can circumvent restrictions, no kernel-level containment.

### 7. Web Interface (`scripts/chat_web.py`)

FastAPI + uvicorn serving a single-page HTML UI (`nanochat/ui.html`). Key design:

- **Worker pool**: each GPU hosts a model replica; workers queue in `asyncio.Queue` and are acquired/released per request
- **Stateless sessions**: every request includes full conversation history (max 500 messages, 8K chars/message, 32K total)
- **SSE streaming**: tokens stream via Server-Sent Events with UTF-8 multi-byte accumulation
- **Validation**: temperature 0.0-2.0, top-k 0-200, max tokens 1-4096

## Evaluation System

### DCLM CORE Metric (`nanochat/core_eval.py`)

Implements the DCLM paper's evaluation suite across three task types:

- **Multiple Choice**: renders prompts, computes per-option cross-entropy, selects minimum loss
- **Schema**: similar to MC but with shared prefix/suffix identification
- **Language Modeling**: exact-match of argmax predictions against continuations

Few-shot examples sampled with seeded RNG (1234 + idx). Supports distributed evaluation with result synchronization.

### Chat Evaluation (`scripts/chat_eval.py`)

Post-SFT task-specific evaluation: ARC-Easy/Challenge, MMLU, GSM8K, HumanEval, SpellingBee. Reports pass@k metrics for generative tasks.

## Relation to Agentic AI Patterns

### What Nanochat Does

**Tool orchestration (minimal)**: The model learns to emit `<|python_start|>` tokens during training on GSM8K examples that contain calculator expressions. The inference engine intercepts these tokens, evaluates the expression, and injects results back into the generation stream. This is a hardcoded single-tool state machine, not a general tool-calling framework.

**Context management (basic)**: Stateless request handling where the full conversation history is re-tokenized and prefilled on every request. No persistent memory, no retrieval, no context window management beyond truncation.

**Reward-driven behavior (emergent)**: The RL stage on GSM8K teaches the model when to use the calculator tool by rewarding correct final answers. Tool-use behavior emerges from the reward signal rather than explicit tool-calling instructions.

### What Nanochat Does Not Do

- **No multi-agent coordination**: single model, single inference path
- **No agent harness or orchestration loop**: no retry logic, no planning, no self-reflection
- **No dynamic tool registration**: the Python calculator is the only tool, hardcoded in the engine
- **No structured output parsing**: tool calls use special tokens, not JSON schemas or function-calling protocols
- **No memory or state persistence**: each request is independent
- **No routing or delegation**: no model selection, no task decomposition

### Relevance to Agentic Systems

Nanochat is valuable as a **reference implementation of the substrate layer** that agentic systems build on top of. It demonstrates:

1. **How tool use gets baked into a model** -- through SFT data with tool-call tokens followed by RL that rewards correct tool usage
2. **How special tokens create protocol boundaries** -- the `<|python_start|>` / `<|python_end|>` / `<|output_start|>` / `<|output_end|>` pattern is the same pattern used by production function-calling models, just with a single tool
3. **How inference engines manage tool execution** -- the `RowState` / forced-token-queue pattern shows how generation pauses for tool execution and resumes with injected results
4. **The cost floor** -- at $43-48 for a GPT-2-class model with basic tool use, this establishes what minimal capable agents cost to train from scratch

## Strengths

- **Complete pipeline in one repo**: tokenizer through web UI, no external training frameworks
- **Radical simplicity**: single `--depth` parameter controls the entire model scale
- **Reproducible speedrun**: `bash runs/speedrun.sh` trains everything end-to-end in ~2 hours
- **State-of-the-art optimizers**: Muon + AdamW hybrid with ZeRO-2 sharding and Polar Express orthogonalization
- **Modern architecture choices**: GQA, RoPE, RMSNorm, Flash Attention 3, FP8, sliding window attention
- **Scaling law infrastructure**: built-in support for sweeps via `runs/scaling_laws.sh`
- **Strong community**: 47K+ stars, active leaderboard competition driving efficiency improvements

## Weaknesses

- **Single tool only**: the calculator is the only supported tool; no extensible tool framework
- **No security sandbox**: code execution explicitly documented as unsafe for adversarial inputs
- **GPT-2 scale ceiling**: 124M-parameter models are far below the capability threshold for useful agentic behavior
- **No persistent state**: stateless sessions mean no long-running agent workflows
- **No structured generation**: no constrained decoding, no JSON mode, no function-calling schema
- **CUDA-centric**: CPU/MPS support exists but requires significant capability reduction
- **No quantization for deployment**: inference runs in bf16/fp32 with no GGUF/GPTQ/AWQ export

## Key Takeaways

1. **The depth dial is an elegant abstraction** for scaling experiments. Deriving batch size, LR, weight decay, and training duration from a single integer via scaling laws eliminates hyperparameter search for the common case.

2. **Tool use through special tokens + RL** is the minimal viable pattern. Nanochat shows that even a 124M-parameter model can learn when to use a calculator if the training data includes tool-call examples and RL rewards correct usage.

3. **Muon optimizer is a meaningful advance** over pure AdamW for matrix parameters. The orthogonalization step and distributed three-phase pipeline are production-worthy ideas applicable to larger training runs.

4. **Best-fit document packing** with BOS alignment achieves near-100% token utilization during pretraining, a technique directly applicable to any LLM training pipeline.

5. **The 1000x cost reduction** from $43,000 (2019) to $43 (2026) for GPT-2-level capability is driven by hardware (H100 vs V100), software (Flash Attention, FP8, Muon), and data (curated datasets like ClimbMix).

## File Structure

```
nanochat/
  checkpoint_manager.py    Checkpoint I/O
  common.py                Constants, distributed utilities, dtype detection
  core_eval.py             DCLM CORE metric evaluation
  dataloader.py            Distributed tokenizing loader with best-fit packing
  dataset.py               Pretraining data utilities
  engine.py                Inference with KV cache and tool-use state machine
  execution.py             Python code execution (calculator + subprocess sandbox)
  gpt.py                   Transformer model (GQA, RoPE, RMSNorm, sliding window)
  loss_eval.py             Bits-per-byte evaluation
  optim.py                 MuonAdamW optimizer with ZeRO-2 distributed variant
  report.py                Result reporting utilities
  tokenizer.py             BPE tokenizer with conversation rendering
  ui.html                  ChatGPT-style web interface

scripts/
  base_train.py            Pretraining with depth-driven hyperparameters
  base_eval.py             Base model evaluation
  chat_sft.py              Supervised fine-tuning on task mixtures
  chat_rl.py               GRPO/REINFORCE on GSM8K
  chat_cli.py              CLI chat interface
  chat_web.py              FastAPI web server with worker pool
  chat_eval.py             Post-SFT task evaluation
  tok_train.py             Tokenizer training
  tok_eval.py              Tokenizer evaluation

tasks/
  arc.py                   ARC science questions
  gsm8k.py                 Grade-school math with calculator tool use
  humaneval.py             Code generation
  mmlu.py                  Multiple choice reasoning
  smoltalk.py              General conversation data
  spellingbee.py           Spelling and character counting
  customjson.py            Custom identity conversations
  common.py                Task mixture and shared utilities

runs/
  speedrun.sh              Full pipeline (~2 hours on 8xH100)
  miniseries.sh            Multi-model training series
  scaling_laws.sh           Scaling experiments
  runcpu.sh                CPU/MPS examples
```

## Sources

- [GitHub Repository](https://github.com/karpathy/nanochat)
- [DCLM Paper (arXiv:2406.11794)](https://arxiv.org/abs/2406.11794)
- [Muon Optimizer](https://github.com/KellerJordan/Muon)
- [modded-nanogpt (predecessor)](https://github.com/KellerJordan/modded-nanogpt)
- [Flash Attention](https://github.com/Dao-AILab/flash-attention)
- [FineWeb Dataset](https://huggingface.co/datasets/HuggingFaceFW/fineweb)
