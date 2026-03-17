# Goal

Fix the two root-cause defects that prevent the workgen verify/refine loop from functioning: parse_llm_json crashes the entire program on empty LLM responses, and RULES_FILE defaults to a relative path that only resolves from the lx repo. These are the only blockers — the workgen lx program itself (main.lx) is already rewritten with the correct agentic flow (AI-driven investigation with tools, append_system instead of system, verify loop called, proper error propagation).

# Why

- grader.grade and auditor.audit crash with an unrecoverable LxError::runtime when the LLM returns empty text, because parse_llm_json passes an empty string to serde_json::from_str — the verify loop cannot execute a single iteration. Empty text occurs when the model chooses a tool call over text generation (e.g., in grader/auditor calls with tools: [] which the Claude CLI treats as full tool access), exhausts max_turns, and returns result: null. The Claude CLI has no mechanism to disable tools via --allowedTools — passing an empty string is equivalent to omitting the flag, both give full tool access.
- RULES_FILE defaults to "workgen/rules/work-item.md" which is a relative path that only exists when CWD is the lx repo — running from any other directory without the justfile produces a file-not-found error deep in the program instead of a clear message at startup

# What changes

**crates/lx/src/stdlib/ai.rs — parse_llm_json empty-text guard:**

In the parse_llm_json function, after extract_llm_text successfully returns the text string, add a check for empty text before attempting JSON parse. When text.trim() is empty, return Ok(Err(format!("{context}: empty LLM response"))) — the same return type as when extract_llm_text itself returns an error message. This flows through the existing fallback paths in every caller (grader builds a zero-score result, auditor builds a zero-score result) without any caller changes.

**workgen/main.lx — RULES_FILE validation:**

In the +main function, read RULES_FILE the same way AUDIT_FILE is read (env.get with ?? "" fallback). Check both together: if either is empty, emit a single error message naming both required env vars with a usage example, then return Err. Remove the "workgen/rules/work-item.md" default.

# How it works

The parse_llm_json guard is a three-line addition at the chokepoint where all AI-backed stdlib agents extract text before JSON parsing. The function signature already returns a nested Result (Ok(Ok(json)) for success, Ok(Err(msg)) for soft errors, Err(LxError) for hard errors). Returning Ok(Err(msg)) for empty text uses the soft-error path that every caller already handles — grader's parse_llm_result builds {score: 0, passed: false, feedback: msg}, auditor's parse_llm_result does the same. The verify loop receives a failing grade, attempts revision, and continues iterating rather than crashing. This also handles the case where the model chooses a tool call despite append_system instructions — the tool call exhausts max_turns, result is null, parse_ai_response yields empty text, and the guard catches it cleanly.

The RULES_FILE change is a startup validation. The justfile recipes always set both env vars to absolute paths — AUDIT_FILE via the RULES variable and RULES_FILE via WORK_ITEM_RULES, both derived from home_directory(). Normal operation through the justfile is unaffected. Direct invocation fails fast with a clear error.

# Files affected

- `crates/lx/src/stdlib/ai.rs` — Add empty-text check in parse_llm_json between the extract_llm_text match and the serde_json::from_str call
- `workgen/main.lx` — Rewrite +main to validate both AUDIT_FILE and RULES_FILE, remove RULES_FILE default

# Task List

## Task 1: Add empty-text guard in parse_llm_json

**Subject:** Guard against empty LLM text in parse_llm_json
**ActiveForm:** Adding empty-text guard in parse_llm_json

Edit crates/lx/src/stdlib/ai.rs. In the parse_llm_json function, after the match on extract_llm_text that binds the text variable and before the serde_json::from_str call, add a guard: if text.trim().is_empty(), return Ok(Err(format!("{context}: empty LLM response"))). This uses the same soft-error return path as the Err(msg) arm of the extract_llm_text match on the line above.

Verify: run `just diagnose` and confirm it passes.

## Task 2: Require RULES_FILE env var in workgen main.lx

**Subject:** Make RULES_FILE required with clear error message
**ActiveForm:** Updating workgen env var validation

Edit workgen/main.lx. In the +main function, change the env var handling: read rules_path with env.get "RULES_FILE" ?? "" (removing the "workgen/rules/work-item.md" default). Check both audit_path and rules_path — if either is empty, emit an error message that names both AUDIT_FILE and RULES_FILE as required, shows a usage example referencing the justfile (e.g., just -f workgen/justfile audit-rust), and return Err. Only call run when both are non-empty.

Verify: run `cargo run -p lx-cli -- run workgen/run.lx` with no env vars set and confirm the error message names both AUDIT_FILE and RULES_FILE.

## Task 3: Run workgen smoke test end-to-end

**Subject:** Verify workgen verify loop works with smoke test
**ActiveForm:** Running workgen smoke test

Run `TEST_TAG=smoke cargo run -p lx-cli -- run workgen/tests/run.lx` and confirm the rust-audit scenario completes without crashing. The key verification is that the verify_and_revise function executes (grader.grade returns a result instead of crashing, the loop iterates or passes, auditor.audit runs). The scenario does not need to score above threshold — the goal is that the pipeline runs end-to-end without hard errors.

If the test fails with a non-crash error (e.g., low grading score), that is acceptable — the verify loop is functioning. If it crashes with a JSON parse error or other LxError, investigate whether task 1 was applied correctly by checking that parse_llm_json has the empty-text guard.

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.
