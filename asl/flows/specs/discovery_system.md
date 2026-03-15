# Discovery System

Automated pipeline for finding useful repos, tools, and patterns.

## Target Goal

Search multiple sources (GitHub API, curated lists, MCP registries, crates.io) across 5 research axes. Score candidates with a weighted composite formula. Auto-integrate high-scoring repos (≥8.0), flag mid-range (6.0-7.9) for human review, skip low-scoring (<6.0). Integration: git submodule → context engine reindex → research doc generation → PostgreSQL tracking. The system improves itself through 5 dogfooding loops.

## Scenarios

### Scenario 1: Fill MCP Audit Gap (Docker Engine)

Weekly search: `topic:mcp-server "docker" language:rust stars:>10`. Finds `moondrop/docker-mcp` (200 stars, active, implements container management). Scores: axis_relevance=10 (fills critical gap from audit), integration_difficulty=9, quality=8, activity=9. Composite: 8.9.

**Success:** Auto-integrated into `reference/docker-mcp-rust`. Symlinked to context-engine, indexed with TEI embeddings. Next debugging scenario has container tools available.

**Edge case:** High-scoring repo turns out broken on ARM. Added to blocklist, skipped next cycle.

### Scenario 2: Language Fit Veto

Search finds `openai/openai-java` (10K stars, excellent quality). Axis relevance=6 (Java, not our stack). Integration difficulty=2 (port ideas only). Composite: 4.2.

**Success:** Logged, not integrated. Human review decision: "interesting approach but not actionable."

### Scenario 3: Self-Reinforcing Discovery Loop

Early discovery found "awesome-mcp-servers" list → 14 new repos added. One of those repos has search hints leading to better queries. Next cycle's searches yield higher-quality results.

**Success:** Discovery velocity increases. Average composite score of integrated repos drifts upward over cycles.

### Scenario 4: Duplicate Detection

Search returns a repo already in `reference/`. System detects the existing submodule and skips re-integration. If the repo has been updated since last integration, it notes the update.

**Success:** No duplicate submodules. Updated repos flagged for potential re-evaluation.

### Scenario 5: API Rate Limiting

GitHub API returns 403 (rate limited) after 10 search queries. Pipeline pauses, waits for rate limit reset, continues with remaining axes.

**Success:** Pipeline completes all axes despite rate limiting. No queries lost. Results from completed axes not discarded.

### Scenario 6: Research Doc Generation

After integrating a repo, generate a 200-400 line research doc: what it does, architecture, patterns, integration opportunities, comparisons with our code, risks. Doc identifies SPECIFIC code locations in our codebase where the patterns apply.

**Success:** Research doc is actionable — a developer can read it and know exactly where and how to use the discovered repo.
