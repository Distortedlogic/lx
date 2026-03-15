# MCP Tool Audit

Assess all MCP servers against 10 engineering scenarios.

## Target Goal

Connect to every MCP server in the toolbelt, list its tools, and assess coverage against 10 concrete engineering scenarios. Classify each server as pass/partial/fail. Identify dead weight tools (serve no scenario) and critical gaps (missing tools that block scenarios). Produce a prioritized report with fix order.

## Scenarios

### Scenario 1: Feature Development (Scenario 1 from Audit)

Agent works on "add invite-by-email." Reads issue ✓. Searches codebase ✓. Writes code ✓. Opens PR — `forgejo:create_pull` doesn't exist ✗. Agent stuck, human must manually open PR.

**Success post-fix:** `create_pull` tool exists. Agent goes end-to-end without human intervention.

### Scenario 2: Debugging Production Bug (Scenario 2)

Service is down. Agent checks Traefik routes ✓. No Docker MCP — can't check if container is running ✗. No Systemd MCP — can't check host services ✗. Finds error in Signoz logs ✓. Needs to restart service — no way to do it ✗.

**Success post-fix:** Docker Engine MCP added. Agent can `docker restart`, verify container status, read logs.

### Scenario 3: Code Review (Scenario 4 — Most Broken)

PR exists. Agent lists PRs ✓. Can't read PR title/body ✗. Can't see diff ✗. Falls back to manual file-by-file comparison. Can't comment ✗. Can't approve/merge ✗.

**Success post-fix:** All 6 PR tools exist. Agent reads, reviews, comments, approves, merges entirely in-context.

### Scenario 4: Server Connection Failure

MCP server binary not found or crashes on startup. Audit should catch this gracefully and report "connection failed" rather than crashing.

**Success:** Server classified as "fail" with issue "connection failed." Audit continues with remaining servers.

### Scenario 5: Dead Weight Detection

PictShare MCP has 6 tools. None are used in any of the 10 scenarios. ByteStash MCP has 6 tools, also unused.

**Success:** Both classified as dead weight. Report recommends removal: "12 tools serving no engineering scenario."

### Scenario 6: Partial Coverage

PostgreSQL MCP has query/execute tools (covers data investigation ✓) but is missing schema introspection tools (can't list tables, describe columns). Classified as "partial."

**Success:** Report shows which scenarios pass and which specific tools are missing. Priority fix: add `list_tables`, `describe_table`.

### Scenario 7: Full Audit Summary

22 servers audited. 10 pass, 4 partial, 6 fail (2 dead weight, 4 broken). 204 total tools, 44 dead weight (22%), 42 missing. Report prioritizes fixes by scenario frequency × impact.

**Success:** Report is actionable. Top 5 fixes would unblock the most scenarios. Priority order: fix broken servers first, add missing tools second, remove dead weight last.
