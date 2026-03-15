# Project Setup

Dependency-ordered service scaffolding via MCP tools.

## Target Goal

Scaffold a new service with proper infrastructure: project files, git repo, database, health monitoring. Steps must execute in dependency order (can't create DB before repo exists, can't monitor before service exists). Each step uses a different MCP server.

## Scenarios

### Scenario 1: Full Setup (Rust Service with DB and Monitoring)

Config: name="billing-service", lang="rust", db=true, monitoring=true. Steps: scaffold → create repo → create database → add health monitor → verify.

**Success:** All 5 steps complete. Project exists at expected path. Repo created in Forgejo. Database exists in PostgreSQL. Health monitor pinging the service URL.

### Scenario 2: Minimal Setup (No DB, No Monitoring)

Config: name="webhook-proxy", lang="rust", db=false, monitoring=false. Steps: scaffold → create repo → verify. DB and monitoring steps skipped.

**Success:** Only 3 steps execute. No DB created. No monitor added. Verification only checks scaffold + repo.

### Scenario 3: Repo Creation Fails

Forgejo MCP returns error (duplicate repo name). Setup should stop and report the failure.

**Success:** Error propagated with clear message: "repo 'billing-service' already exists." Database and monitoring steps not attempted.

### Scenario 4: Verification Catches Missing Resource

Scaffold succeeds, repo succeeds, DB creation succeeds, but monitoring fails (Uptime Kuma MCP unreachable). Final verification reports 1 failure.

**Success:** Verification lists: repo ✓, scaffold ✓, db ✓, monitoring ✗. Partial success reported — user can manually add monitoring.

### Scenario 5: Idempotent Re-run

User runs setup again for the same service name. Scaffold directory exists, repo exists, DB exists.

**Success:** Each step detects the resource already exists and skips or reports "already exists." No duplicate resources created.
