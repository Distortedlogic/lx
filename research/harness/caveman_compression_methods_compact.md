# Caveman Compression Methods for Agent Context (April 2026)

## Executive Summary

Not one standard method. Three patterns:

1. Prompt-only semantic compression.
2. Deterministic NLP or MLM stripping.
3. Prompt plus validator plus repair.

For lx, use pattern 3 on prose-heavy markdown dumps. Best tradeoff: strong compression, lower risk of breaking structure.

## Verified Repos

### 1. `wilpel/caveman-compression`

Repo: <https://github.com/wilpel/caveman-compression>

Best public semantic-compression reference.

Ships:

- LLM prompt in `prompts/compression.txt`
- written spec in `SPEC.md`
- three implementations:
  - `caveman_compress.py` - LLM
  - `caveman_compress_nlp.py` - offline NLP
  - `caveman_compress_mlm.py` - masked-language model

Method:

- cut predictable grammar, filler, redundant scaffolding
- keep nouns, main verbs, numbers, names, negation, uncertainty, technical terms
- shorten phrasing without changing facts

Important repo lesson: good caveman compression is not random telegraph prose. It preserves factual equality and logical continuity.

### 2. `JuliusBrussee/caveman`

Repo: <https://github.com/JuliusBrussee/caveman>

Broader repo. Includes a speaking-style skill and `caveman-compress` for markdown memory files like `CLAUDE.md`.

As of April 12, 2026, README shows latest release `v1.5.1` on April 11, 2026.

What matters for lx inside `caveman-compress`:

- markdown-safe compression prompt in `caveman-compress/scripts/compress.py`
- validator and repair workflow
- file-type detection before compression

Best pattern in the repo:

1. detect compressible prose
2. compress with LLM
3. save `.original.md` backup
4. validate headings, code fences, URLs, file paths, bullet drift
5. run targeted repair if needed
6. restore original if repair fails

This is the strongest public pattern for agent memory files and markdown dumps, especially when files mix prose with code/config.

The speaking-style prompt is useful for terse chat replies or agents without a hook/plugin system. It is not enough by itself for safe file rewriting.

### 3. `jwiegley/claude-prompts/skills/caveman`

Repo: <https://github.com/jwiegley/claude-prompts/tree/main/skills/caveman>

Minimal prompt-only form. Good for core idea. Not enough for safe markdown-file rewriting because it lacks detection, validation, and repair.

## Method Comparison

| Pattern | Repo | Strengths | Weaknesses | Best Use |
|---|---|---|---|---|
| Prompt-only | `jwiegley/claude-prompts`, parts of `wilpel` | Fast, simple | Easy to over-compress or break structure | Ad hoc prose compression |
| Offline rule/NLP/MLM | `wilpel/caveman-compression` | Cheap, reproducible, offline | Lower compression, brittle around nuance | Bulk preprocessing |
| Prompt + validate + repair | `JuliusBrussee/caveman` | Best balance of safety and compression for markdown | More moving parts | Memory files, markdown dumps |

## Clear Reusable Compression Rules

Distilled from the verified repos, especially `wilpel` and `JuliusBrussee`:

### Remove Aggressively

- articles
- auxiliary verbs
- filler adverbs
- pleasantries
- redundant connectives
- indirect wording when direct wording keeps the same meaning

### Preserve Exactly

- names, numbers, dates, quantities
- negation
- uncertainty
- constraints
- technical terms
- code blocks
- inline code
- URLs
- headings
- file paths
- commands

### Preserve Semantically

- causal chains
- temporal ordering
- requirement strength
- exceptions
- comparisons such as "at least", "more than", "same", "different"

### Avoid These Failure Modes

- telegraphic ambiguity
- over-compression
- added information
- structural drift

## Implications for lx

Best targets:

- research dumps
- session summaries
- memory files
- scratchpads
- imported verbose notes
- long status logs

Bad default targets:

- work item specs
- prompts and system instructions
- API references
- legal or policy text
- onboarding docs
- markdown dominated by code or config

## Recommended lx First Iteration

1. Keep nuanced research notes in normal prose.
2. Create a skill for prose-heavy `.md` dump files, not all markdown.
3. Use prompt plus validation, not prompt-only compression.
4. Preserve headings, fenced code blocks, inline code, URLs, commands, and file paths exactly.
5. Compress only natural-language sections.
6. Treat failed validation as targeted repair, not recursive recompression.

## Sources

- `wilpel/caveman-compression` README: <https://github.com/wilpel/caveman-compression>
- `wilpel/caveman-compression` compression prompt: <https://raw.githubusercontent.com/wilpel/caveman-compression/main/prompts/compression.txt>
- `wilpel/caveman-compression` spec: <https://raw.githubusercontent.com/wilpel/caveman-compression/main/SPEC.md>
- `JuliusBrussee/caveman` README: <https://github.com/JuliusBrussee/caveman>
- `JuliusBrussee/caveman` markdown compressor: <https://raw.githubusercontent.com/JuliusBrussee/caveman/main/caveman-compress/scripts/compress.py>
- `JuliusBrussee/caveman` markdown validator: <https://raw.githubusercontent.com/JuliusBrussee/caveman/main/caveman-compress/scripts/validate.py>
- `JuliusBrussee/caveman` file-type detector: <https://raw.githubusercontent.com/JuliusBrussee/caveman/main/caveman-compress/scripts/detect.py>
- `JuliusBrussee/caveman` base speaking-style skill: <https://raw.githubusercontent.com/JuliusBrussee/caveman/main/skills/caveman/SKILL.md>
- `jwiegley/claude-prompts` caveman skill: <https://raw.githubusercontent.com/jwiegley/claude-prompts/main/skills/caveman/SKILL.md>
