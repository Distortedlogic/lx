# Real-Time Agents and Edge/Offline Agent Deployment

## Sub-100ms Agent Architectures

Real-time agent systems require end-to-end latency below human perception thresholds. Practical deployments achieve this through pipeline decomposition and parallel execution.

**Autonomous vehicle agents** decompose into perception (15ms), planning (20ms), and control (10ms) for 45ms total. Gaming NPC agents running Whisper ASR (20ms) + Llama-3.2 dialogue (30ms) + TTS (25ms) achieve 75ms total. Enterprise RAG pipelines hit 140ms: embedding (10ms), retrieval (30ms), LLM inference (100ms).

**Trading agents** require deterministic sub-millisecond paths for order execution, with ML-based signal generation running separately at 10-50ms cadence. The agent decides; the execution path is pre-compiled and non-ML.

**Robotics agents** use DiMA-style knowledge distillation, compressing GPT-4V-class planners by 100x parameters into edge-deployable models while preserving reasoning logic. NitroGen (trained on 40K+ hours of gameplay) demonstrates that robotics-oriented architectures (GROOT N1.5) can transfer from gaming to real-world navigation.

**Key constraint**: anything involving a cloud round-trip adds 200-500ms before first token, making local inference mandatory for true sub-100ms systems.

## Speculative Execution and Pre-Computation

The Speculative Actions framework (ICLR 2026) applies microprocessor-style speculation to agent pipelines. A fast "guessing model" predicts the next action while the slower ground-truth executor catches up. When predictions match (up to 55% accuracy across gaming, e-commerce, web search, and OS environments), progress is already made; when they disagree, execution proceeds normally. This is lossless -- correctness is never sacrificed.

**Speculative decoding** for LLM inference uses a small draft model to propose token sequences that the large model verifies in a single forward pass, achieving 2-3x speedup for autoregressive generation. Medusa and EAGLE variants achieve 2.2-3.6x speedup.

**KV cache strategies** reduce redundant computation. RadixAttention stores prompt/generation caches in a radix tree for prefix reuse. InfiniGen prefetches KV entries based on predicted access patterns. KV caches can be quantized to 3 bits with negligible quality loss. StreamingLLM enables infinite-length generation with fixed memory.

**Semantic caching** stores query embeddings alongside LLM responses, returning cached answers for semantically similar queries in sub-millisecond time versus seconds for fresh inference. High-repetition workloads see up to 73% cost reduction.

## Small Language Models for Agents

The SLM landscape as of early 2026:

| Model | Params | Context | Key Strength |
|-------|--------|---------|-------------|
| Gemma 3n | ~5B (2B effective) | 32K | Multimodal (text/image/video/audio), 140+ languages |
| Phi-4 mini | 3.8B | 128K | Reasoning competitive with larger models |
| Qwen3-8B | 8B | 32K | Dual-mode: thinking for reasoning, non-thinking for speed |
| Llama 3.2 | 1B/3B | 128K | Broad ecosystem, robust conversational ability |
| SmolLM2 | 135M-1.7B | 8K | Trained on 11T tokens, tiny footprint |
| LFM2.5 | 1.2B | -- | Beats Llama 3.2 1B on GPQA (38.9 vs 16.6) and AIME25 (14.0 vs 0.3) |
| MobileLLM | 125M | -- | ~50 tok/s on iPhone, deep-thin architecture |

**LFM2.5-1.2B benchmarks on device**: AMD Ryzen AI 9 decodes at 116 tok/s (856MB RAM). Snapdragon Gen4 phone decodes at 82 tok/s (0.9GB). Galaxy S25 Ultra decodes at 70 tok/s (719MB).

**FunctionGemma** (Gemma 3 270M) is fine-tuned specifically for function/tool calling, enabling local agent tool use at minimal cost. MobileLLM-R1.5 shows 2-5x better reasoning than models twice its size.

**Quantization trade-offs**: train at FP16, deploy at INT4 for 4x memory reduction with 1-3% quality loss. Sub-4-bit quantization pushes to 4-8x reduction at ~3% loss. BitNet 1.58-bit fits a 2B model in 400MB but requires training from scratch.

## On-Device and Edge Agent Deployment

**Deployment runtimes**:
- ExecuTorch (Meta): 50KB base footprint, 12+ hardware backends, GA since October 2025, powers Instagram/WhatsApp/Messenger
- llama.cpp: CPU inference standard, GGUF format, broad hardware support
- MLX (Apple): Optimized for Apple Silicon unified memory
- LiteRT (Google): 1.4x faster GPU than TFLite, NPU acceleration, cross-platform (Android/iOS/macOS/Windows/Linux/Web)
- MLC-LLM: Cross-platform compilation, WebGPU support for browsers

**Browser-local agents** are now feasible via WebGPU + MLC-LLM or MediaPipe LLM Inference API, running sub-1B models entirely client-side with no server round-trip.

**Embedded systems** use ExecuTorch's minimal footprint on microcontrollers. SINTRONES demonstrated IEC 62443-aligned edge AI for mission-critical industrial applications at Embedded World 2026.

**Hybrid edge-cloud routing**: simple queries route to device (50ms), complex queries to cloud. The agent itself decides routing based on query complexity estimation.

## Offline-Capable Agents

**Architecture pattern**: tiered model stack with local-first execution. The agent maintains a small on-device model for core capabilities and escalates to cloud when available and needed.

**Practical offline stacks**:
- Ollama + Qwen3-8B or LFM2.5-1.2B for terminal/desktop agents
- ExecuTorch + Llama 3.2 1B for mobile
- llama.cpp + quantized 3B model for embedded

**Graceful degradation strategies**:
1. Cache recent cloud model responses for common queries
2. Fall back to smaller local model with explicit capability disclosure
3. Queue complex requests for when connectivity returns
4. Use orchestrator agents that delegate to specialist agents with fallback awareness

**Key gap**: most agent frameworks (LangChain, CrewAI) assume always-on cloud APIs. Building offline-first requires explicit architecture decisions -- local tool execution, embedded vector stores (SQLite-backed), and pre-loaded context windows.

Hyperlink AI demonstrates a fully offline agent running on-device with file understanding. Nexa AI's approach runs complete agent loops locally in the terminal.

## Streaming and Incremental Processing

**Voice agent pipeline** (the most latency-sensitive streaming use case):
- Traditional sequential: STT (500ms) -> LLM -> TTS (400ms) = 900ms+ before first audio
- Parallel streaming: Audio -> Realtime LLM -> Audio, collapsing six stages to three

**Streaming optimizations**:
- Process partial transcripts for early LLM reasoning (don't wait for complete utterance)
- Synthesize audio in chunks as response tokens generate (streaming TTS achieves <50ms time-to-first-audio)
- Use WebRTC for bidirectional continuous connections (30-50ms transport latency)
- MCP enables continuous message flow instead of discrete API calls

**Incremental tool execution**: stream tool outputs back to the agent as they become available rather than waiting for completion. The agent begins reasoning on partial results, reducing wall-clock time for multi-tool workflows.

**Token streaming**: partial LLM outputs stream as chunks, critical for chatbot time-to-first-token. 4-bit quantization achieves up to 40% latency reduction. On-device token generation runs under 20ms each for short contexts.

## Hardware Acceleration for Local Inference

### NPU Comparison (2026)

| Platform | TOPS | Memory BW | LLM Decode | Max RAM |
|----------|------|-----------|------------|---------|
| Qualcomm Snapdragon X2 Elite | 80-85 | ~136 GB/s | ~63 tok/s (1.2B Q4) | 48 GB |
| AMD Ryzen AI 400 | 60 | ~136 GB/s | 116 tok/s (1.2B Q4) | 64 GB |
| Intel Lunar Lake | 48 | ~102 GB/s | 18.5 tok/s (decode) | 32 GB |
| Apple M4 Max | 38 | 546 GB/s | -- | 128 GB |
| Apple M5 | -- | 153 GB/s | 19-27% faster than M4 | -- |
| Qualcomm Snapdragon 8 Elite Gen 5 | ~60 | 50-90 GB/s | 82 tok/s (1.2B Q4) | -- |

**Memory bandwidth is the decisive bottleneck**, not TOPS. Mobile devices have 50-90 GB/s versus datacenter GPUs at 2-3 TB/s (a 30-50x gap). LLM decode is memory-bound, so Apple's M4 Max at 546 GB/s outperforms higher-TOPS competitors for large models.

**GPU inference**: NVIDIA H100 achieves 7.1ms first-token latency on GPT-J 6B, 12K tok/s on Llama2-13B. Jetson edge modules hit 30-80ms for optimized models. DRIVE AGX targets <50ms multi-sensor autonomous driving.

**Power efficiency**: NPU workloads use 30-40% less battery than CPU/GPU equivalents. ARM-based Qualcomm laptops achieve 40% better battery life versus x86. Always-on edge inference requires single-digit milliwatts.

**Diffusion LLMs** promise 4-6x speedups over autoregressive decoding by generating multiple tokens simultaneously, particularly relevant for NPU architectures.

## Latency Budgets and Optimization

### Where Time Is Spent in Agent Pipelines

1. **LLM inference** (40-70% of total): prefill + decode. Prefill scales with input length; decode is sequential and memory-bound
2. **Tool execution** (10-30%): API calls, database queries, file I/O
3. **Context assembly** (5-15%): embedding generation, retrieval, prompt construction
4. **Network round-trips** (10-30%): 200-500ms per cloud API call, eliminated by local inference

### Optimization Priority Order

1. **Eliminate network round-trips**: move inference on-device (biggest single win: 200-500ms saved)
2. **Reduce model size**: 70B -> 7B -> 1B with distillation (10-100x latency reduction)
3. **Quantize aggressively**: FP16 -> INT4 gives 4x memory reduction, up to 40% latency reduction
4. **Cache everything**: semantic caching for repeated queries, KV cache for context reuse
5. **Parallelize tool calls**: execute independent tools concurrently, not sequentially
6. **Speculative execution**: predict likely next actions, execute in parallel with verification
7. **Stream outputs**: return partial results immediately instead of waiting for completion

### Context Growth Problem

Unregulated context is the primary driver of agentic cost and latency inflation. Multi-turn agents concatenate history, causing quadratic cost scaling with turn count. Mitigation: rolling summarization, selective history, ChunkKV semantic compression (26% throughput improvement).

### Model Selection as Optimization

A 2-3 model stack (small for routing/simple tasks, medium for most work, large for complex reasoning) with explicit latency budgets outperforms single-model approaches. AgentBalance imposes explicit token-cost and latency budgets using Pareto-optimal LLM pools via profiling.

### Quantized Inference Speedups (Llama2-70B)

| Precision | Relative Speed | Notes |
|-----------|---------------|-------|
| FP32 | 1x (baseline) | 2000ms |
| FP16 | ~2x | Standard training precision |
| FP8 | ~4x | H100 optimized |
| INT4 | ~13x | 150ms achieved |

## Sources

- https://www.humai.blog/real-time-ai-inference-2026-complete-guide-to-sub-100ms-models/
- https://arxiv.org/abs/2510.04371
- https://iclr.cc/virtual/2026/poster/10009726
- https://v-chandra.github.io/on-device-llms/
- https://getstream.io/blog/realtime-ai-agents-latency/
- https://localaimaster.com/blog/npu-comparison-2026
- https://localaimaster.com/blog/small-language-models-guide-2026
- https://www.liquid.ai/blog/introducing-lfm2-5-the-next-generation-of-on-device-ai
- https://www.edge-ai-vision.com/2026/01/on-device-llms-in-2026-what-changed-what-matters-whats-next/
- https://redis.io/blog/llm-token-optimization-speed-up-apps/
- https://developers.googleblog.com/google-ai-edge-small-language-models-multimodality-rag-function-calling/
- https://developers.googleblog.com/litert-the-universal-framework-for-on-device-ai/
- https://machinelearning.apple.com/research/exploring-llms-mlx-m5
- https://hyperlink.nexa.ai/
- https://arxiv.org/html/2508.04721v1
- https://www.bentoml.com/blog/the-best-open-source-small-language-models
- https://www.siliconflow.com/articles/en/best-small-llms-for-edge-devices
- https://blog.karanbalaji.com/100-days-of-ai-day-4-offline-on-device-ai-in-2025-and-beyond
