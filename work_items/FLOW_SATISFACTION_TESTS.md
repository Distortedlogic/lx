# Goal

Build satisfaction test suites for all 14 flow specs — making every prose scenario in `flows/specs/` executable via `just test-flows`.

**Prerequisite:** Execute `work_items/LX_LANGUAGE_FIXES.md` first. That work item fixes the parser/runtime issues (record field values, spread binding power, missing field → None, empty records, multi-level `..` imports, `invoke_flow` closure env, source-relative flow paths) and updates the flow libs + existing test flows to use clean syntax.

# Why

- 14 flow specs in `flows/specs/` define 83 scenarios in prose. Zero are executable.
- `std/test` is implemented and proven (security_audit has 3 passing scenarios, defense_layers has 4) but only 2 of 14 flows have tests.
- After the language fixes in `LX_LANGUAGE_FIXES.md`, test flows can directly import `flows/lib/` modules — no more self-contained wrappers with inlined logic. Tests can exercise the real library code.
- 11 of 14 flows are DETERMINISTIC or MOCKABLE — testable with fixture data.

# What Changes

## Phase 1: Rebuild existing test suites with clean imports (Tasks 1-2)

After `LX_LANGUAGE_FIXES.md` Task 8 updates the existing security_audit and defense_layers flows, verify they still pass and now import libs directly.

## Phase 2: Deterministic + mockable flow suites (Tasks 3-11)

For each remaining flow, create:
- `flows/tests/{name}_flow.lx` — wrapper that imports `../lib/` modules and takes fixture input
- `flows/tests/{name}/main.lx` — satisfaction spec with scenarios + grader
- `flows/tests/{name}/fixtures/` — synthetic data per scenario

Since the language fixes make lib imports work in `test.run`, test flows directly import and call `guard.full_scan`, `transcript.parse`, `scoring.rank`, `react.run` (with mock data), etc. — testing the actual library code, not replicated patterns.

## Phase 3: Live-only flow stubs (Task 12)

Three flows require live MCP servers. Create gated stubs.

# Files Affected

**New files (per flow):**
- `flows/tests/{flow_name}_flow.lx` — test wrapper importing flow libs
- `flows/tests/{flow_name}/main.lx` — satisfaction spec
- `flows/tests/{flow_name}/fixtures/` — synthetic data per scenario

# Task List

### Task 1: Verify security_audit suite uses lib imports

**Subject:** Confirm security_audit test flow imports guard/transcript libs directly

**Description:** After `LX_LANGUAGE_FIXES.md` Task 8 converted `security_audit_flow.lx` from self-contained to importing `../lib/guard` and `../lib/transcript`, verify the 3 scenarios still pass.

Run `just test-flows`. If any failures, debug and fix.

**ActiveForm:** Verifying security_audit lib imports

---

### Task 2: Verify defense_layers suite uses lib imports

**Subject:** Confirm defense_layers test flow imports guard/transcript libs directly

**Description:** Same as Task 1 for defense_layers — verify the 4 scenarios still pass with direct lib imports.

Run `just test-flows`.

**ActiveForm:** Verifying defense_layers lib imports

---

### Task 3: Create agentic_loop satisfaction test suite

**Subject:** Build test wrapper, fixtures, and spec for agentic_loop

**Description:** The agentic_loop flow implements Thought→Action→Observation with doom loop detection and circuit breakers. Import `../lib/react` for the ReAct engine logic.

Create `flows/tests/agentic_loop_flow.lx` — imports `../lib/react`, simulates the loop deterministically:
- Takes `{task, actions}` where `actions` is a list of pre-planned `{thought, action, observation, done}` turns
- Iterates, checking for doom loops (3 identical consecutive actions) and circuit breaker (max turns)
- Returns `{completed, turns, reasoning, doom_loop_detected, circuit_breaker_fired}`

Create fixtures:
- `fixtures/simple_completion/actions.json` — 3 turns, completes
- `fixtures/doom_loop/actions.json` — repeating Read→Edit→Test
- `fixtures/stagnating/actions.json` — oscillating edit→revert
- `fixtures/circuit_breaker/actions.json` — 26+ turns

Create `flows/tests/agentic_loop/main.lx` — 4 scenarios, threshold 0.70.

Run `just test-flows`.

**ActiveForm:** Building agentic_loop test suite

---

### Task 4: Create post_hoc_review satisfaction test suite

**Subject:** Build test wrapper, fixtures, and spec for post_hoc_review

**Description:** Import `../lib/transcript` for parsing. Takes a transcript path and pre-computed review, extracts patterns/mistakes, merges with review, returns `{patterns, mistakes, recommendations, summary}`.

Create fixtures:
- `fixtures/successful_session/` — efficient completion transcript
- `fixtures/failed_session/` — retries and failure transcript
- `fixtures/mixed_session/` — mixed success/failure transcript

Create `flows/tests/post_hoc_review/main.lx` — 3 scenarios, threshold 0.65.

Run `just test-flows`.

**ActiveForm:** Building post_hoc_review test suite

---

### Task 5: Create research satisfaction test suite

**Subject:** Build test wrapper, fixtures, and spec for research

**Description:** Takes `{request, web_results, codebase_results, github_results}` as fixture data. Merges using scoring/ranking logic from `../lib/scoring`. Returns `{sources, findings, recommendation, confidence}`.

Create fixtures:
- `fixtures/rate_limiting/` — 3 sources agree
- `fixtures/no_oss_match/` — GitHub empty
- `fixtures/conflicting/` — contradictory sources

Create `flows/tests/research/main.lx` — 3 scenarios, threshold 0.65.

Run `just test-flows`.

**ActiveForm:** Building research test suite

---

### Task 6: Create perf_analysis satisfaction test suite

**Subject:** Build test wrapper, fixtures, and spec for perf_analysis

**Description:** Takes `{path, concern, specialist_results}`. Aggregates, deduplicates, sorts by severity. Returns `{findings, categories, top_issue, total_findings}`.

Create fixtures:
- `fixtures/memory_leak/` — memory specialist finds leak
- `fixtures/clean/` — no issues
- `fixtures/multi_category/` — CPU + memory + algorithm findings

Create `flows/tests/perf_analysis/main.lx` — 3 scenarios, threshold 0.65.

Run `just test-flows`.

**ActiveForm:** Building perf_analysis test suite

---

### Task 7: Create discovery_system satisfaction test suite

**Subject:** Build test wrapper, fixtures, and spec for discovery_system

**Description:** Import `../lib/scoring` for ranking logic. Takes `{query, candidates}`. Filters, ranks. Returns `{ranked, top_pick, total_candidates, query}`.

Create fixtures:
- `fixtures/clear_winner/` — one dominant candidate
- `fixtures/close_race/` — two within 5%
- `fixtures/no_match/` — all below threshold

Create `flows/tests/discovery_system/main.lx` — 3 scenarios, threshold 0.65.

Run `just test-flows`.

**ActiveForm:** Building discovery_system test suite

---

### Task 8: Create agent_lifecycle satisfaction test suite

**Subject:** Build test wrapper, fixtures, and spec for agent_lifecycle

**Description:** Import `../lib/catalog`, `../lib/memory`. Simulates lifecycle phases: create → seed → maintain → retire. Returns `{phases_completed, final_state, memory_entries, errors}`.

Create fixtures:
- `fixtures/full_lifecycle/` — all phases succeed
- `fixtures/seed_failure/` — seeding fails
- `fixtures/maintenance_drift/` — drift detected, re-seed triggered

Create `flows/tests/agent_lifecycle/main.lx` — 3 scenarios, threshold 0.65.

Run `just test-flows`.

**ActiveForm:** Building agent_lifecycle test suite

---

### Task 9: Create tool_generation satisfaction test suite

**Subject:** Build test wrapper, fixtures, and spec for tool_generation

**Description:** Takes `{gap, design, scaffold, code}` as pre-computed stage results. Pipeline: analyze → design → scaffold → codegen. Returns `{tool_name, files_created, design_summary, passed_smoke}`.

Create fixtures:
- `fixtures/simple_tool/` — 2-function tool
- `fixtures/complex_tool/` — 5-function tool
- `fixtures/compile_error/` — codegen error, fix loop runs

Create `flows/tests/tool_generation/main.lx` — 3 scenarios, threshold 0.65.

Run `just test-flows`.

**ActiveForm:** Building tool_generation test suite

---

### Task 10: Create software_diffusion satisfaction test suite

**Subject:** Build test wrapper, fixtures, and spec for software_diffusion

**Description:** Import `../lib/dispatch`, `../lib/guidance`. 5-stage pipeline: elicitor → structurer → typer → type_fixer → implementer. Returns `{stages_completed, final_output, type_errors_fixed, implementation}`.

Create fixtures:
- `fixtures/clean_pipeline/` — all 5 stages succeed
- `fixtures/type_error/` — typer finds errors, fixer corrects
- `fixtures/elicitation_incomplete/` — incomplete spec

Create `flows/tests/software_diffusion/main.lx` — 3 scenarios, threshold 0.65.

Run `just test-flows`.

**ActiveForm:** Building software_diffusion test suite

---

### Task 11: Create full_pipeline satisfaction test suite

**Subject:** Build test wrapper, fixtures, and spec for full_pipeline

**Description:** Import `../lib/grading`. Two entry points: audit and manual. Simulates: classify → dispatch → collect → grade → iterate. Uses `grader.quick_grade` (deterministic). Returns `{mode, findings, grade_score, iterations, passed}`.

Create fixtures:
- `fixtures/audit_pass/` — passes on first grade
- `fixtures/manual_list/` — 3-item parallel dispatch
- `fixtures/grading_loop/` — fails first grade, passes second

Create `flows/tests/full_pipeline/main.lx` — 3 scenarios, threshold 0.65.

Run `just test-flows`.

**ActiveForm:** Building full_pipeline test suite

---

### Task 12: Create live-only flow test stubs

**Subject:** Add gated test stubs for mcp_tool_audit, fine_tuning, project_setup

**Description:** These 3 flows require live MCP servers. Create stub spec files gated behind `LX_TEST_LIVE` env var, tagged `["live"]`.

Create:
- `flows/tests/mcp_tool_audit/main.lx` — 2 scenario stubs
- `flows/tests/fine_tuning/main.lx` — 2 scenario stubs
- `flows/tests/project_setup/main.lx` — 2 scenario stubs

Each checks `env.get "LX_TEST_LIVE"`, emits "SKIPPED: requires LX_TEST_LIVE", and returns early when unset.

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

**Execute `work_items/LX_LANGUAGE_FIXES.md` FIRST.** Then load this work item:

```
mcp__workflow__load_work_item({ path: "work_items/FLOW_SATISFACTION_TESTS.md" })
```

Then call `next_task` to begin.
