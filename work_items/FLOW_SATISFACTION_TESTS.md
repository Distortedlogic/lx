# Goal

Fix the `Protocol +Name` parse bug that blocks all flow lib imports, then fix the existing flow lib files to use correct syntax, then build satisfaction test suites for all 14 flow specs — making every prose scenario in `flows/specs/` executable via `just test-flows`.

# Why

- 14 flow specs in `flows/specs/` define 83 scenarios in prose. Zero are executable. The specs are dead documentation.
- `std/test` is implemented and proven (security_audit has 3 passing scenarios) but only 1 of 14 flows has tests.
- The `Protocol +Name` parse bug blocks importing any flow lib file (`guard.lx`, `transcript.lx`, `catalog.lx`, `react.lx`, `scoring.lx`). Every test flow that imports libs must inline the logic instead — defeating the purpose of testing the actual flow code.
- The flow lib files themselves use `Protocol +Name` syntax (export marker between keyword and name) but the parser expects `+Protocol Name` (export marker before keyword). Fixing this unblocks direct import of all 15 lib modules.
- 11 of 14 flows are DETERMINISTIC or MOCKABLE — they can be tested with fixture data and/or mocked backends without any live external services.

# What Changes

## Phase 1: Fix Protocol +Name parsing (Tasks 1-2)

The lexer emits `TokenKind::Plus` for `+` mid-line (only `+` at line-start becomes `Export`). The Protocol parser calls `expect_type_name` after advancing past `Protocol`, and chokes on the `Plus` token.

Fix: in `parse_protocol`, after advancing past the `Protocol` keyword, check if the next token is `Plus`. If so, consume it and set `exported = true`, then continue to `expect_type_name`. Same fix for `parse_mcp_decl`, `parse_trait_decl`, `parse_agent_decl` — all four declaration types should accept `+` between keyword and name as an alternative export syntax.

This makes both `+Protocol Foo = {...}` and `Protocol +Foo = {...}` valid and equivalent.

## Phase 2: Fix flow lib syntax (Task 3)

The flow lib files use `Protocol +Name` which will now parse. But some also have other syntax issues. Audit all 15 lib files, fix any that don't parse, and verify each one runs through `lx run` without errors. The libs are:

`catalog.lx`, `dispatch.lx`, `github.lx`, `grading.lx`, `guard.lx`, `guidance.lx`, `mcp_session.lx`, `memory.lx`, `react.lx`, `report.lx`, `scoring.lx`, `specialists.lx`, `training.lx`, `transcript.lx`, `workflow.lx`

## Phase 3: Deterministic flow test suites (Tasks 4-5)

Two flows need no mocking — they transform file data without external calls:

**defense_layers** (7 spec scenarios): Reads transcripts, runs guard scans, checks for drift. Uses `guard`, `transcript`, `report`, `std/cron`, `std/trace`, `std/agents/monitor`. The `monitor.check` call is the only external dependency — mock it or use a fixture-only subset.

**security_audit** (already done — 3 scenarios passing). Verify it still works after the Protocol fix.

## Phase 4: Mockable flow test suites (Tasks 6-14)

Nine flows use AI/agent calls but can be tested with self-contained wrapper flows that replicate the core logic with fixture data instead of live calls:

| Flow | Spec scenarios | Key deps to mock/inline |
|------|---------------|------------------------|
| agentic_loop | 7 | `react.run` (spawns worker agent) |
| agent_lifecycle | 6 | `dispatch.run_one` (spawns seeder), `memory` |
| discovery_system | 6 | `github.search_axes` (HTTP calls) |
| post_hoc_review | 6 | `reviewer.review` (AI call) |
| research | 5 | 3 parallel search agents |
| perf_analysis | 5 | `perf_specialist.analyze` (AI call) |
| software_diffusion | 6* | 5 agent spawns (elicitor, structurer, typer, type_fixer, implementer) |
| tool_generation | 6 | `tool_maker.*` agents + MCP smoke test |
| full_pipeline | 7 | `router.route`, `verifier.verify`, `grading.run`, gritql MCP |

*subagent_lifecycle.md covers 6 scenarios for the lifecycle pattern used in software_diffusion

For each mockable flow, the test wrapper inlines the data-transformation logic (dispatch, aggregation, reconciliation, reporting) but replaces AI/agent/MCP calls with deterministic fixture data. The grader scores whether the transformation logic produces correct structure, correct field values, and correct aggregation.

## Phase 5: Live-only flow stubs (Task 15)

Three flows fundamentally require live services: `mcp_tool_audit` (live MCP servers), `fine_tuning` (Langfuse MCP), `project_setup` (Forgejo + PostgreSQL + Uptime Kuma MCPs). For these, create stub test specs that are gated behind `LX_TEST_LIVE` env var and tagged `["live"]`. They won't run in CI but document what a live test would look like.

# Files Affected

**Modified files:**
- `crates/lx/src/parser/stmt_protocol.rs` — accept `+` between `Protocol` and name
- `crates/lx/src/parser/statements.rs` — accept `+` in MCP/Trait/Agent decls
- `flows/lib/*.lx` — fix any syntax issues blocking parsing
- `flows/tests/security_audit/main.lx` — verify still works after Protocol fix

**New files (per flow):**
- `flows/tests/{flow_name}_flow.lx` — self-contained test wrapper
- `flows/tests/{flow_name}/main.lx` — satisfaction spec (scenarios + grader)
- `flows/tests/{flow_name}/fixtures/` — synthetic data per scenario

# Task List

### Task 1: Fix Protocol +Name parsing in stmt_protocol.rs

**Subject:** Accept + export marker between Protocol keyword and type name

**Description:** In `crates/lx/src/parser/stmt_protocol.rs`, in `parse_protocol`, after `self.advance()` (consuming the `Protocol` keyword), add a check: if `*self.peek() == TokenKind::Plus`, consume it and set `exported = true` (overriding the `exported` parameter — OR it if both `+Protocol +Name` somehow occurs). Then proceed to `self.expect_type_name`. This makes `Protocol +Foo = {x: Int}` equivalent to `+Protocol Foo = {x: Int}`.

Apply the same fix to `parse_mcp_decl`, `parse_trait_decl`, and `parse_agent_decl` in their respective parser files — each should accept `+` after the keyword as an export marker.

Run `just diagnose`.

**ActiveForm:** Fixing Protocol +Name parsing

---

### Task 2: Add parser tests for Protocol +Name syntax

**Subject:** Verify both export syntaxes parse correctly

**Description:** Add a test file `tests/16_edge_cases.lx` (append to existing) or a new section that verifies:
1. `Protocol +Foo = {x: Int}` parses and creates a protocol
2. `+Protocol Bar = {y: Int}` also works (existing behavior)
3. Protocols with `+` export can be imported and validated

Run `just test`.

**ActiveForm:** Adding Protocol +Name parser tests

---

### Task 3: Fix and verify all flow lib files

**Subject:** Audit all 15 flows/lib/*.lx files for parse/runtime errors

**Description:** For each of the 15 lib files in `flows/lib/`, run `cargo run -p lx-cli -- run flows/lib/{file}.lx` and fix any parse or runtime errors. Common issues expected:
- `Protocol +Name` (now fixed by Task 1)
- `re.is_match (lower p) haystack` body extent issues (use temp bindings)
- Record literal field value parsing (use temp bindings for complex expressions)

After fixing, verify each file parses cleanly. Not all files can fully execute standalone (some require imports or inputs), but they must at least parse without errors.

Run `just diagnose` and `just test`.

**ActiveForm:** Fixing flow lib parse errors

---

### Task 4: Verify security_audit test suite still works

**Subject:** Run existing security_audit satisfaction tests after Protocol fix

**Description:** Run `just test-flows` and verify the security_audit satisfaction tests (3 scenarios) still pass. If the Protocol +Name fix changes how the flow wrapper or libs behave, update accordingly.

Run `just test-flows`.

**ActiveForm:** Verifying security_audit tests

---

### Task 5: Create defense_layers satisfaction test suite

**Subject:** Build test wrapper, fixtures, and spec for defense_layers

**Description:** The defense_layers flow reads transcripts, runs guard scans (injection, loop, resource abuse), checks for agent drift, and produces alerts. It uses `std/cron` for scheduling and `std/agents/monitor` for drift detection.

Create `flows/tests/defense_layers_flow.lx` — a self-contained wrapper that:
- Takes `{project_dir, agents}` as input (agents is a list of mock agent records)
- Parses transcripts from `project_dir` via `transcript.parse` (or inlined logic)
- Runs `guard.full_scan` for injection/loop/resource detection
- Skips `monitor.check` (mock it to return no drift) or inline a drift-check stub
- Returns `{findings, alerts, clean_agents}` record

Create fixtures:
- `fixtures/escalation/` — transcripts showing probe → scan → exploit escalation pattern
- `fixtures/false_positive/` — benign activity that resembles a threat
- `fixtures/clean/` — normal agent activity with no issues
- `fixtures/multi_agent/` — 3 agents: one stuck, one with injection, one clean

Create `flows/tests/defense_layers/main.lx` — spec with 4 scenarios, threshold 0.65, grader using `std/audit` to check output structure and finding counts.

Run `just test-flows`.

**ActiveForm:** Building defense_layers test suite

---

### Task 6: Create agentic_loop satisfaction test suite

**Subject:** Build test wrapper, fixtures, and spec for agentic_loop

**Description:** The agentic_loop flow implements Thought→Action→Observation with doom loop detection and circuit breakers. The core logic from `react.run` spawns a worker agent.

Create `flows/tests/agentic_loop_flow.lx` — a wrapper that simulates the ReAct loop deterministically:
- Takes `{task, actions}` where `actions` is a list of `{thought, action, observation, done}` records representing pre-planned turns
- Iterates through actions, checking for doom loops (3 identical consecutive actions) and circuit breaker (max turns)
- Returns `{completed, turns, reasoning, doom_loop_detected, circuit_breaker_fired}` record

Create fixtures:
- `fixtures/simple_completion/` — 3 turns, completes successfully
- `fixtures/doom_loop/` — repeating Read→Edit→Test pattern
- `fixtures/stagnating/` — oscillating edit→revert pattern
- `fixtures/circuit_breaker/` — 25+ turns without completion

Create `flows/tests/agentic_loop/main.lx` — spec with 4 scenarios from the 7 in `flows/specs/agentic_loop.md` (the deterministic subset), threshold 0.70.

Run `just test-flows`.

**ActiveForm:** Building agentic_loop test suite

---

### Task 7: Create post_hoc_review satisfaction test suite

**Subject:** Build test wrapper, fixtures, and spec for post_hoc_review

**Description:** The post_hoc_review flow reads a completed session transcript, extracts patterns/mistakes, and produces a review report. It uses `reviewer.review` (AI call) for analysis.

Create `flows/tests/post_hoc_review_flow.lx` — a wrapper that:
- Takes `{transcript_path, review_result}` where `review_result` is a pre-computed fixture review
- Reads the transcript, extracts tool call patterns and error patterns
- Merges with the provided review result
- Returns `{patterns, mistakes, recommendations, summary}` record

Create fixtures:
- `fixtures/successful_session/` — transcript where agent completed task efficiently
- `fixtures/failed_session/` — transcript with multiple retries and eventual failure
- `fixtures/mixed_session/` — some successes, some failures, tool misuse

Create `flows/tests/post_hoc_review/main.lx` — spec with 3 scenarios, threshold 0.65.

Run `just test-flows`.

**ActiveForm:** Building post_hoc_review test suite

---

### Task 8: Create research satisfaction test suite

**Subject:** Build test wrapper, fixtures, and spec for research

**Description:** The research flow fans out to 3 parallel specialists (web, codebase, GitHub), then synthesizes findings into a recommendation report.

Create `flows/tests/research_flow.lx` — a wrapper that:
- Takes `{request, web_results, codebase_results, github_results}` as pre-computed fixture data
- Merges the three result sets using the same scoring/ranking logic as the real flow
- Produces a recommendation report with comparison and gap analysis
- Returns `{sources, findings, recommendation, confidence}` record

Create fixtures:
- `fixtures/rate_limiting/` — 3 sources agree on approach, clear recommendation
- `fixtures/no_oss_match/` — GitHub returns nothing, recommend custom build
- `fixtures/conflicting/` — two sources give contradictory advice

Create `flows/tests/research/main.lx` — spec with 3 scenarios, threshold 0.65.

Run `just test-flows`.

**ActiveForm:** Building research test suite

---

### Task 9: Create perf_analysis satisfaction test suite

**Subject:** Build test wrapper, fixtures, and spec for perf_analysis

**Description:** The perf_analysis flow runs 5 parallel performance specialists (cpu, memory, io, concurrency, algorithmic) against a codebase path and concern.

Create `flows/tests/perf_analysis_flow.lx` — a wrapper that:
- Takes `{path, concern, specialist_results}` where `specialist_results` is a list of pre-computed findings per specialist
- Aggregates findings, deduplicates, sorts by severity
- Produces a prioritized report
- Returns `{findings, categories, top_issue, total_findings}` record

Create fixtures:
- `fixtures/memory_leak/` — memory specialist finds leak, others find minor issues
- `fixtures/clean/` — all specialists report no significant issues
- `fixtures/multi_category/` — findings across CPU, memory, and algorithm categories

Create `flows/tests/perf_analysis/main.lx` — spec with 3 scenarios, threshold 0.65.

Run `just test-flows`.

**ActiveForm:** Building perf_analysis test suite

---

### Task 10: Create discovery_system satisfaction test suite

**Subject:** Build test wrapper, fixtures, and spec for discovery_system

**Description:** The discovery_system flow searches GitHub for OSS tools/libraries matching capability gaps, scores candidates, and produces a ranked recommendation list.

Create `flows/tests/discovery_system_flow.lx` — a wrapper that:
- Takes `{query, candidates}` where `candidates` is a list of pre-scored tool records
- Runs the scoring/ranking logic from `flows/lib/scoring.lx`
- Filters, deduplicates, and ranks by composite score
- Returns `{ranked, top_pick, total_candidates, query}` record

Create fixtures:
- `fixtures/clear_winner/` — one candidate dominates all criteria
- `fixtures/close_race/` — two candidates within 5% of each other
- `fixtures/no_match/` — all candidates below threshold

Create `flows/tests/discovery_system/main.lx` — spec with 3 scenarios, threshold 0.65.

Run `just test-flows`.

**ActiveForm:** Building discovery_system test suite

---

### Task 11: Create agent_lifecycle satisfaction test suite

**Subject:** Build test wrapper, fixtures, and spec for agent_lifecycle

**Description:** The agent_lifecycle flow manages agent creation, seeding with initial knowledge, periodic maintenance (cron), and eventual retirement.

Create `flows/tests/agent_lifecycle_flow.lx` — a wrapper that:
- Takes `{agent_name, seed_data, maintenance_results}` as fixture data
- Simulates the lifecycle: create → seed → maintain → retire
- Tracks state transitions and validates each phase completed
- Returns `{phases_completed, final_state, memory_entries, errors}` record

Create fixtures:
- `fixtures/full_lifecycle/` — agent goes through all phases successfully
- `fixtures/seed_failure/` — seeding fails, agent enters error state
- `fixtures/maintenance_drift/` — maintenance detects drift, triggers re-seed

Create `flows/tests/agent_lifecycle/main.lx` — spec with 3 scenarios, threshold 0.65.

Run `just test-flows`.

**ActiveForm:** Building agent_lifecycle test suite

---

### Task 12: Create tool_generation satisfaction test suite

**Subject:** Build test wrapper, fixtures, and spec for tool_generation

**Description:** The tool_generation flow takes a capability gap, designs an MCP tool, scaffolds it, generates code, and optionally runs a smoke test.

Create `flows/tests/tool_generation_flow.lx` — a wrapper that:
- Takes `{gap, design, scaffold, code}` as pre-computed stage results
- Runs the pipeline: analyze gap → design tool → scaffold files → generate code
- Validates each stage output before proceeding
- Returns `{tool_name, files_created, design_summary, passed_smoke}` record

Create fixtures:
- `fixtures/simple_tool/` — straightforward tool with 2 functions
- `fixtures/complex_tool/` — tool with 5 functions and dependencies
- `fixtures/compile_error/` — code generation produces compile error, fix loop runs

Create `flows/tests/tool_generation/main.lx` — spec with 3 scenarios, threshold 0.65.

Run `just test-flows`.

**ActiveForm:** Building tool_generation test suite

---

### Task 13: Create software_diffusion satisfaction test suite

**Subject:** Build test wrapper, fixtures, and spec for software_diffusion

**Description:** The software_diffusion flow runs 5 sequential agents (elicitor → structurer → typer → type_fixer → implementer) to progressively refine a specification into code.

Create `flows/tests/software_diffusion_flow.lx` — a wrapper that:
- Takes `{prompt, stage_outputs}` where `stage_outputs` is a list of pre-computed outputs per stage
- Runs the 5-stage pipeline passing each stage's output to the next
- Validates structural requirements at each stage boundary
- Returns `{stages_completed, final_output, type_errors_fixed, implementation}` record

Create fixtures:
- `fixtures/clean_pipeline/` — all 5 stages succeed first try
- `fixtures/type_error/` — typer finds errors, type_fixer corrects them
- `fixtures/elicitation_incomplete/` — elicitor produces incomplete spec, subsequent stages handle gaps

Create `flows/tests/software_diffusion/main.lx` — spec with 3 scenarios, threshold 0.65.

Run `just test-flows`.

**ActiveForm:** Building software_diffusion test suite

---

### Task 14: Create full_pipeline satisfaction test suite

**Subject:** Build test wrapper, fixtures, and spec for full_pipeline

**Description:** The full_pipeline flow has two entry points (audit, manual), dispatches to parallel subagents, grades output against a rubric, and iterates with feedback.

Create `flows/tests/full_pipeline_flow.lx` — a wrapper that:
- Takes `{mode, task, subagent_results, grade_results}` as fixture data
- Simulates: classify → dispatch → collect → grade → iterate
- The grading loop uses `grader.quick_grade` (deterministic keyword matching)
- Returns `{mode, findings, grade_score, iterations, passed}` record

Create fixtures:
- `fixtures/audit_pass/` — audit mode, findings pass on first grade
- `fixtures/manual_list/` — manual mode with 3-item list, parallel dispatch
- `fixtures/grading_loop/` — first grade fails, revision passes on second attempt

Create `flows/tests/full_pipeline/main.lx` — spec with 3 scenarios, threshold 0.65.

Run `just test-flows`.

**ActiveForm:** Building full_pipeline test suite

---

### Task 15: Create live-only flow test stubs

**Subject:** Add gated test stubs for mcp_tool_audit, fine_tuning, project_setup

**Description:** These 3 flows require live MCP servers and cannot be tested with fixtures. Create minimal spec files that:
- Are gated behind `LX_TEST_LIVE` env var (skip if not set)
- Document what the test would verify if live services were available
- Tagged `["live"]` so they're excluded from normal `just test-flows` runs

Create:
- `flows/tests/mcp_tool_audit/main.lx` — stub with 2 scenarios (audit connected servers, handle missing server)
- `flows/tests/fine_tuning/main.lx` — stub with 2 scenarios (harvest traces, enhance with teacher)
- `flows/tests/project_setup/main.lx` — stub with 2 scenarios (scaffold project, handle MCP failure)

Each stub should `emit "SKIPPED: requires LX_TEST_LIVE"` and return early when the env var is absent.

Run `just test-flows`.

**ActiveForm:** Creating live-only test stubs

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.

---

## Task Loading Instructions

To execute this work item, load it with the workflow MCP:

```
mcp__workflow__load_work_item({ path: "work_items/FLOW_SATISFACTION_TESTS.md" })
```

Then call `next_task` to begin.
