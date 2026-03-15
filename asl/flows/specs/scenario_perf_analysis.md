# Performance Analysis

5-domain parallel specialist analysis with conflict resolution.

## Target Goal

Fan out performance analysis to 5 domain specialists (db, memory, compute, threading, network). Each analyzes its domain independently. Orchestrator collects findings, detects conflicts (same location, different recommendations), and produces a prioritized report.

## Scenarios

### Scenario 1: Database Bottleneck

Concern: "API endpoint /users takes 3 seconds." perf-db finds: missing index on users.email, N+1 query in user loader. Other 4 specialists find nothing significant.

**Success:** Report shows 2 findings, both from db domain, both high severity. Other domains report clean.

### Scenario 2: Conflicting Recommendations

Concern: "high memory usage." perf-memory says: "cache is too large, reduce TTL." perf-db says: "increase cache size to reduce query load." Same component (cache), opposite recommendations.

**Success:** Conflict detected and reported. Both specialists' reasoning preserved. Human must decide trade-off.

### Scenario 3: Cross-Domain Issue

Concern: "service occasionally hangs." perf-threading finds: mutex contention in request handler. perf-network finds: DNS resolution blocking the event loop. Both contribute to the same symptom.

**Success:** Both findings reported. Orchestrator notes they may be related (same symptom, different root causes). Fix priority: threading first (higher severity).

### Scenario 4: All Clear

Concern: "is our service performant enough?" All 5 specialists analyze and find no issues above threshold.

**Success:** Report says "no significant performance issues found" with the metrics each specialist checked.

### Scenario 5: One Specialist Slow

perf-network specialist takes 60 seconds (running actual network benchmarks). Other 4 complete in 5 seconds.

**Success:** Results from fast specialists available immediately. Slow specialist's results appended when ready. Report not blocked by one slow domain.
