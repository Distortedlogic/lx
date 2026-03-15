# Research

Multi-source research with parallel specialists and synthesis.

## Target Goal

Fan out to 3 parallel specialists (web research, codebase search, GitHub OSS search), then synthesize their findings into a single recommendation report. The orchestrator plans the research strategy, the specialists execute independently, and the synthesis compares external best practices, our current approach, and available OSS libraries.

## Scenarios

### Scenario 1: Rate Limiting Research

Prompt: "research how other projects handle rate limiting, compare to our codebase, check OSS." Research specialist finds token bucket and sliding window patterns. Context-engine finds our current approach (simple counter in middleware). GitHub finds `governor` crate (Rust, 2K stars).

**Success:** Report compares 3 approaches, identifies our gap (no per-user limits), recommends `governor` crate with integration plan.

### Scenario 2: No OSS Match

Prompt: "research how to implement our custom agent memory tier system." External findings describe general RAG patterns. Codebase shows our current flat ctx.json approach. GitHub finds nothing matching our specific L0-L3 tier model.

**Success:** Report says "no strong OSS match" and recommends building custom based on the external RAG patterns adapted to our tier model.

### Scenario 3: Conflicting External Advice

Prompt: "research database migration strategies." Research specialist finds two contradictory best practices: "always use reversible migrations" vs "irreversible is fine with good backups." Context-engine shows we use irreversible.

**Success:** Report presents both positions with trade-offs. Recommendation accounts for our existing approach and team preference.

### Scenario 4: One Specialist Fails

GitHub search API is rate-limited and returns an error. Research and context-engine succeed.

**Success:** Report is generated from 2 out of 3 sources. OSS section notes the failure and suggests retrying later. Report is not blocked by one specialist failing.

### Scenario 5: Large Codebase Results

Context-engine returns 50 matching files for a broad topic. Research returns 20 articles.

**Success:** Synthesis summarizes (not dumps) the findings. Report is under 500 lines. Key patterns extracted, not every file listed.
