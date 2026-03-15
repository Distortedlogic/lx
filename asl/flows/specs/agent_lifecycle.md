# Agent Lifecycle

Tiered memory management with seeding, consolidation reviews, and developmental stages.

## Target Goal

Manage an agent's knowledge across its entire lifetime using an LSM-tree-inspired 4-tier memory hierarchy. L0 (episodic raw transcripts) feeds L1 (working patterns with confidence scores) via daily review. L1 patterns that prove reliable promote to L2 (consolidated, low plasticity). L2 patterns that reach near-certainty promote to L3 (procedural — baked into the system prompt). Contradictions trigger demotion. Seeding transfers a mentor's knowledge to new agents at birth.

## Scenarios

### Scenario 1: Infant Agent Learning from Failures

An "issue classifier" agent is deployed at embryonic stage. First 5 calls misclassify issues. Daily review clusters the failures and creates an L1 entry at confidence 0.2: "When issue mentions 'slow query', ask about table size first." By conversation 15, the pattern has 8 confirmations and 0 contradictions. Promoted to L2.

**Success:** Agent accuracy improves week-over-week. Pattern reaches L2 after ≥5 confirmations at confidence ≥0.7.

### Scenario 2: Mentor Seeding New Agent

A mature "PostgreSQL optimizer" agent (500+ conversations) has high-confidence L2/L3 knowledge. A new "index recommendation" agent is created. Seeder session runs: mentor's L2 patterns transfer as L1 at halved confidence (0.3-0.5). New agent starts with 10-30 L1 entries.

**Success:** Seeded agent reaches target accuracy 2-3 weeks faster than an unseeded agent.

### Scenario 3: Contradiction Detection and Demotion

Classifier has L2 pattern: "Issues tagged 'urgent' go to Sprint-1" (confidence 0.85, 20 confirmations). User moves 3 urgent issues to Sprint-2. Weekly review detects the contradiction spike. Pattern demoted to L1 with confidence reset to 0.5.

**Success:** Contradiction increments; demotion happens automatically. Agent now flags urgent→Sprint-2 assignments for human confirmation.

**Edge case:** False positive contradiction (user made a mistake). Manual review window + reversal option needed.

### Scenario 4: Context Assembly Under Token Budget

Verifier agent has L3 (600 tokens), 15 L2 entries (20-100 tokens each), 40 L1 entries. Prompt is "write a migration for the users table." Context assembly: load L3 (always) → score L2 against prompt → load top-3 full L2 → fill remaining budget with L1 summaries.

**Success:** Context fits within budget (3000 tokens). Agent has relevant knowledge without truncation.

**Edge case:** 30 L2 entries match but budget fits 5. Scoring must rank by relevance; dropped knowledge re-queryable on demand.

### Scenario 5: L3 Promotion (Weekly Review)

Agent has an L2 entry with confidence 0.95 that has been stable for 4 weeks. Weekly review identifies it as L3-eligible. Entry is written into the system prompt markdown file.

**Success:** System prompt grows by the promoted entry. Next session loads the knowledge automatically.

### Scenario 6: Developmental Stage Transition

Agent crosses 20 conversations threshold: embryonic → infant. First L1 entries appear. At 100 conversations: infant → juvenile. First L2 promotions happen. At 500 conversations: juvenile → competent. Token cost per task drops as L3 grows.

**Success:** Each stage transition is logged. Token cost per task decreases as knowledge consolidates upward.
