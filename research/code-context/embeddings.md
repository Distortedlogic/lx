# Code Embeddings: Models & Local Inference

## Why Specialized Code Embeddings

General-purpose text embeddings miss code-specific semantics: variable naming conventions, control flow patterns, type relationships, and API usage patterns.

## Model Comparison (2025-2026)

### Voyage-Code-3 (Recommended for Quality)

- **Provider**: Voyage AI (acquired by Google)
- **Context length**: 32K tokens
- **Dimensions**: 2048, 1024 (default), 512, 256 (Matryoshka learning)
- **Output types**: float32, int8, uint8, binary, ubinary
- **Performance**: Outperforms OpenAI-v3-large by 13.80% and CodeSage-large by 16.81%
- **Cost**: $0.22 per 1M tokens
- **Blog**: [voyage-code-3 announcement](https://blog.voyageai.com/2024/12/04/voyage-code-3/)

### CodeXEmbed / SFR-Embedding-Code (Salesforce)

- #1 on CoIR benchmark, 12 programming languages, 7B parameter model
- [MarkTechPost coverage](https://www.marktechpost.com/2025/01/18/salesforce-ai-research-introduced-codexembed-sfr-embedding-code-a-code-retrieval-model-family-achieving-1-rank-on-coir-benchmark-and-supporting-12-programming-languages/)

### Local/Open-Source Options

- **EmbeddingGemma** -- used by Code Context MCP server for local inference
- **Gemini embedding-001** -- used by Roo Code for codebase indexing
- **CodeSage-large-v2** -- open-source, but only 1K context length

### Selection Criteria

| Factor | Consideration |
|---|---|
| Local vs API | Local = privacy + no cost; API = higher quality |
| Context length | 32K (Voyage) vs 8K (OpenAI) vs 1K (CodeSage) |
| Dimensions | Lower dims = less storage, slightly less recall |
| Matryoshka support | Truncating dims with graceful degradation |
| Quantization | int8/binary reduces storage 4-32x with small quality loss |

## Embedding Enrichment

Raw code chunks produce weaker embeddings than enriched chunks. Prepend contextual metadata:

```
Scope: module::submodule::ClassName
Imports: use std::collections::HashMap, use crate::utils
Defines: fn process_data(input: &str) -> Result<Data>
Siblings: [previous: fn validate_input, next: fn save_data]

<actual code here>
```

## Local Inference Frameworks (Rust)

### fastembed-rs (Recommended)

- **Repo**: [github.com/Anush008/fastembed-rs](https://github.com/Anush008/fastembed-rs)
- **Backend**: ONNX Runtime via `ort` crate
- **Maintained by**: Qdrant team
- Synchronous API, auto-downloads models from HuggingFace, supports quantized models
- Pre-built support for BGE, sentence-transformers, and other popular models
- Includes reranking support

### ort (ONNX Runtime Bindings)

- **Repo**: [github.com/pykeio/ort](https://github.com/pykeio/ort)
- Direct ONNX model control, run any ONNX model
- Hardware acceleration: CPU, CUDA, TensorRT, CoreML, DirectML

### Candle (HuggingFace)

- **Repo**: [github.com/huggingface/candle](https://github.com/huggingface/candle)
- Pure Rust, no C/C++ dependencies, small binaries
- Works with `tokenizers` crate, loads from HuggingFace Hub

### EmbedAnything

- **Repo**: [github.com/StarlightSearch/EmbedAnything](https://github.com/StarlightSearch/EmbedAnything)
- **Backend**: Candle
- Streaming embeddings via Rust MPSC channels -- memory-efficient, handles 10GB+
- Dense, sparse, late-interaction, reranker, ModernBERT

### Decision Matrix

| Factor | fastembed | ort | candle | embed_anything |
|---|---|---|---|---|
| Ease of use | Best | Medium | Medium | Good |
| Model variety | Good (curated) | Best (any ONNX) | Good (HF) | Good |
| Dependencies | ONNX Runtime (C++) | ONNX Runtime (C++) | Pure Rust | Candle (Rust) |
| GPU support | Via ONNX EP | Full | CUDA/Metal | CUDA/Metal |
| Streaming | No | No | No | Yes (MPSC) |

## Recommendation

**Primary**: `fastembed-rs` -- simplest integration, pairs with Qdrant, ONNX-backed.
**Alternative**: `embed_anything` for streaming during bulk indexing.
**Fallback**: Direct `ort` for custom ONNX models not in fastembed.

Quantized models (int8) reduce memory ~4x with minimal quality loss. Batch embedding significantly improves throughput. Indexing latency is less critical than retrieval since it runs in the background.

## References

- [6 Best Code Embedding Models (Modal)](https://modal.com/blog/6-best-code-embedding-models-compared)
- [Voyage-Code-3 Blog](https://blog.voyageai.com/2024/12/04/voyage-code-3/)
- [fastembed-rs GitHub](https://github.com/Anush008/fastembed-rs)
- [ort GitHub](https://github.com/pykeio/ort)
- [Candle GitHub](https://github.com/huggingface/candle)
- [EmbedAnything GitHub](https://github.com/StarlightSearch/EmbedAnything)
- [LoRACode: LoRA Adapters for Code Embeddings](https://arxiv.org/html/2503.05315)
