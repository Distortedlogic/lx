# Tool Generation

Detect tool gaps, build MCP servers, register, and smoke test.

## Target Goal

When an agent encounters a missing tool capability, the tool-maker agent runs a 7-phase pipeline: analyze the target API, design tool interfaces, scaffold a Rust MCP project, generate code, compile (hard gate — zero warnings), register in the toolbelt, and smoke test every tool. The tool-maker learns from its successes and failures via L0-L3 memory.

## Scenarios

### Scenario 1: Build Docker Engine MCP

Gap signal: "capability_dead_end" — cannot manage containers, blocks 5 audit scenarios. Tool-maker analyzes Docker Engine API docs. Designs 6 tools: list_containers, get_container, start, stop, restart, get_logs. Scaffolds Rust project with reqwest client. Generates handler code. Compiles clean. Registers in .mcp.json. Smoke tests each tool against a running Docker daemon.

**Success:** All 6 tools callable. `list_containers` returns structured data. `get_logs` returns container output. Full pipeline completes in <5 minutes.

### Scenario 2: Compilation Failure and Recovery

Tool-maker generates code for a Prometheus MCP. First compilation fails: missing `prometheus-rs` dependency. Tool-maker reads the error, adds the dependency to Cargo.toml, retries. Second compilation succeeds.

**Success:** Compilation failure caught and auto-fixed. Final binary has zero warnings. Pipeline continues to registration and testing.

**Edge case:** Second compilation also fails (native dependency not installed). Tool-maker proposes fallback: pure HTTP client instead of tonic. User approves, retries.

### Scenario 3: Fill PR Workflow Gap (Forgejo)

Audit found: "code review 90% broken — no PR tools." Tool-maker analyzes Forgejo API. Designs 6 tools: create_pull, get_pull, comment_on_pull, approve_pull, request_changes, merge_pull. Each tool maps to a Forgejo REST endpoint.

**Success:** All 6 tools pass smoke test. Code review scenario (Scenario 4 from audit) goes from 90% broken to fully functional.

### Scenario 4: Smoke Test Catches Bug

Tool-maker generates an MCP, compilation passes, but smoke test of `list_metrics` returns an empty list when metrics exist. Tool-maker inspects the response parsing, finds it's not handling the Prometheus JSON format correctly. Fixes the parser.

**Success:** Smoke test catches the bug before registration. Fixed tool returns correct data. User never sees the broken version.

### Scenario 5: Agent-Fronted MCP (Learning Wrapper)

Static PostgreSQL MCP exists but agents keep writing slow queries. Tool-maker creates a wrapper agent that sits between caller and static MCP. Wrapper learns L1 patterns: "when query >1s, run EXPLAIN first." After 2 weeks, 80% of suggestions improve query performance.

**Success:** Wrapper agent's L1 entries confirm effectiveness. L1→L2 promotions happen for reliable patterns.

### Scenario 6: Updating Existing MCP

Gap signal: "missing Valkey commands — SMEMBERS, ZRANGE, XRANGE not implemented." Tool-maker adds 3 new tools to the existing Valkey MCP rather than creating a new server. Recompiles, re-registers, re-tests.

**Success:** Existing tools still work. New tools pass smoke test. No regression in existing functionality.
