# Caveman Compression Methods for Agent Context (April 2026)

## Executive Summary

As of April 12, 2026, "caveman compression" is not one single standard technique. The verified public repos split into three practical patterns:

1. Prompt-only semantic compression: tell an LLM to remove predictable grammar while preserving facts, constraints, negation, and technical terms.
2. Deterministic or semi-deterministic compression: use NLP or masked-language-model heuristics to remove highly predictable words offline.
3. Prompt plus validator plus repair: use an LLM to compress prose-heavy markdown, then run local structure-preservation checks and only fix the broken sections.

For lx, the third pattern is the most immediately useful for markdown dump files. It gives higher compression than simple rule stripping while reducing the risk of breaking headings, code fences, URLs, and file paths.

## Verified Repos

### 1. `wilpel/caveman-compression`

Repo: <https://github.com/wilpel/caveman-compression>

This is the clearest "semantic compression" repo. It explicitly frames the method as removing only the linguistic material an LLM can reconstruct reliably from context.

What it ships:

- An LLM prompt in `prompts/compression.txt`
- A written spec in `SPEC.md`
- Three implementations:
  - `caveman_compress.py` - LLM-based
  - `caveman_compress_nlp.py` - offline NLP-based
  - `caveman_compress_mlm.py` - masked-language-model-based

Core idea:

- Remove predictable grammar, filler, and redundant scaffolding
- Keep nouns, main verbs, numbers, names, negation, uncertainty, and technical terms
- Prefer shorter direct phrasing without adding or deleting facts

The clearest reusable prompt pattern comes from this repo. The raw prompt says to aggressively remove stop words and grammatical scaffolding, always keep core semantic carriers, and output only the compressed text.

Important details from the repo:

- The README positions the methods by tradeoff:
  - NLP: stable, offline, about 15-30% reduction
  - MLM: offline and predictability-aware, about 20-30% reduction
  - LLM: strongest compression, about 40-58% reduction
- The spec is stricter than the README:
  - preserve temporal distinctions
  - preserve numbers exactly
  - remove only intensifiers, not meaningful qualifiers
  - keep pronouns when replacing them would increase ambiguity
  - keep full logical chains explicit

Most important design lesson: "caveman compression" is not supposed to be random telegraphese. The good versions protect factual equality and causal completeness.

### 2. `JuliusBrussee/caveman`

Repo: <https://github.com/JuliusBrussee/caveman>

This repo is broader. It includes:

- A persistent caveman speaking style skill
- Several derivative skills such as terse review comments
- `caveman-compress`, which targets markdown memory files like `CLAUDE.md`

As of April 12, 2026, the repo README shows latest release `v1.5.1` dated April 11, 2026.

What matters for lx is not the speaking-style gimmick. It is the markdown compression workflow inside `caveman-compress`.

The compressor prompt in `caveman-compress/scripts/compress.py` is pragmatic rather than theoretical:

- compress markdown into caveman format
- do not modify fenced code blocks
- do not modify inline backticks
- preserve all URLs exactly
- preserve all headings exactly
- preserve file paths and commands
- return only the compressed markdown body
- compress only natural language

That repo also ships the best public "safety loop" I found for markdown compression:

1. Detect whether a file looks compressible natural language or code/config.
2. Compress it with an LLM.
3. Save a `.original.md` backup.
4. Validate structural preservation locally.
5. If validation fails, run a targeted fix prompt instead of recompressing the whole file.
6. Restore the original if repeated repair attempts fail.

The validator checks:

- headings
- fenced code blocks
- URLs
- file paths
- bullet-count drift

This is the most useful production pattern for agent memory files and markdown dumps because it acknowledges that LLM compression is high quality but not trustworthy enough without a structural backstop.

The repo also contains a short always-on prompt for agents without a hook/plugin system. Distilled, it says:

- speak tersely
- keep technical substance exact
- drop filler, pleasantries, hedging, and articles
- short fragments are fine
- code stays normal

That is useful for response style, but it is not sufficient by itself for file rewriting.

### 3. `jwiegley/claude-prompts/skills/caveman`

Repo: <https://github.com/jwiegley/claude-prompts/tree/main/skills/caveman>

This is the minimal prompt-only form. It tells the model to aggressively remove stop words and grammatical scaffolding while preserving meaning.

It is useful because it shows the irreducible core of the method. It is not enough for safe markdown-file rewriting because it does not add detection, validation, or repair.

## Method Comparison

| Pattern | Repo | Strengths | Weaknesses | Best Use |
|---|---|---|---|---|
| Prompt-only | `jwiegley/claude-prompts`, parts of `wilpel` | Fast, simple, flexible | Easy to over-compress or damage structure | Ad hoc prose compression, chat mode |
| Offline rule/NLP/MLM | `wilpel/caveman-compression` | Cheap, reproducible, no API dependency | Lower compression, brittle around nuance | Bulk preprocessing, experiments |
| Prompt + validate + repair | `JuliusBrussee/caveman` | Best balance of compression and safety for markdown | More moving parts | Memory files, session notes, markdown dumps |

## Clear Reusable Compression Rules

Distilled from the verified repos, especially `wilpel` and `JuliusBrussee`:

### Remove Aggressively

- articles
- auxiliary verbs
- filler adverbs
- pleasantries
- redundant connectives
- indirect wording when a direct phrase says the same thing

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

- Telegraphic ambiguity: compressed text becomes hard to parse
- Over-compression: important intermediate reasoning disappears
- Information addition: compressed version invents rationale not present in original
- Structural drift: headings, code fences, links, or paths get mutated

## Implications for lx

For lx, "caveman compression" is most useful as a workflow for markdown dump files written by agents:

- research dumps
- session summaries
- memory files
- scratchpads
- imported verbose notes
- long status logs

It is a poor default for:

- work item specs
- prompts and system instructions
- API references
- legal or policy text
- onboarding docs
- anything where exact prose nuance matters more than token savings

## Recommended lx First Iteration

1. Keep the full nuanced research note in normal prose.
2. Create a skill that targets prose-heavy `.md` dump files, not every markdown file.
3. Base the skill on prompt-plus-validation, not prompt-only compression.
4. Require preservation of headings, fenced code blocks, inline code, URLs, commands, and file paths.
5. Instruct the agent to compress only natural-language sections.
6. Treat failed validation as a targeted repair problem, not a re-compression problem.

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
