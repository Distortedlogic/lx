# Goal

Fix the `ClaudeCodeLlmBackend` stub in `crates/lx-cli/src/llm_backend.rs` to properly handle structured output, NDJSON streaming protocol, cost/token reporting, and SAP-style JSON recovery. No new crate dependencies — the fix extends the existing 67-line stub to ~150 lines using `std::process::Command` and `serde_json`, both already available. Add a ToolSearch tool in `pkg/connectors/tool_search.lx` for on-demand tool discovery.

# Why

- `llm.prompt_structured` is broken — it creates `LlmOpts` with `json_schema: Some(schema)` but `ClaudeCodeLlmBackend.prompt_with` ignores the `json_schema` field entirely. It never passes `--json-schema` to the `claude` CLI. Every `think_structured` call returns raw text, not parsed structured output.
- The stub uses `--output-format json` which returns a single JSON blob after the entire response completes. The `claude` CLI also supports `--output-format stream-json` which returns NDJSON lines including a `Result` message with cost, token usage, turn count, and session ID. The stub discards all of this metadata.
- The `claude` CLI supports `--json-schema <schema>` (verified via `claude --help`). When this flag is set, Claude validates its output against the schema. The stub never passes this flag.
- BAML's research shows SAP-style recovery (strip markdown fences, find JSON in chain-of-thought, fix trailing commas) recovers 92-94% of structured outputs in under 10ms. This is a pure string transformation safety net on top of proper structured output support.
- Adding a 79-download, 0-star community crate (`claude-cli-sdk`) for what amounts to ~80 lines of additional code is over-engineering. The NDJSON protocol is simple — JSON lines over stdin/stdout. The existing stub already spawns the subprocess and reads output. It just needs the right CLI flags and proper line parsing.

# What Changes

**1. Pass `--json-schema` and use `--output-format stream-json`**

In `prompt_with`, change `--output-format json` to `--output-format stream-json`. When `opts.json_schema` is `Some(ref schema)`, add `--json-schema` with the schema string as an argument. This is the critical fix — the stub currently ignores `json_schema` entirely.

**2. Parse NDJSON lines instead of single JSON blob**

The current stub reads `child.wait_with_output()` and parses the entire stdout as a single JSON object. With `--output-format stream-json`, stdout contains one JSON object per line. Each line has a `type` field: `"system"`, `"assistant"`, `"result"`. The stub needs to read lines, find the line with `"type": "result"`, and extract the response from it.

**3. Extract rich metadata from the Result message**

The `result` NDJSON line contains: `result` (the response text), `cost_usd`, `total_cost_usd`, `duration_ms`, `num_turns`, `is_error`, `session_id`, and `usage` (with `input_tokens`, `output_tokens`). The new implementation extracts these and returns them in the response record alongside `text`. Also collects all `assistant` message text blocks for the full response text.

**4. SAP-style JSON recovery for structured output**

When `json_schema` is set and the response `result` string is not valid JSON, apply recovery: strip markdown fences, find first `{`/`[` and trim before, find last `}`/`]` and trim after, remove trailing commas, remove single-line comments. If recovery produces valid JSON, convert to LxVal via the existing `impl From<serde_json::Value> for LxVal` at `crates/lx/src/value/serde_impl.rs:73`.

**5. ToolSearch tool**

A new lx Tool in `pkg/connectors/tool_search.lx` for on-demand tool discovery by keyword matching against tool name and description.

# How It Works

The `prompt_with` method continues to use `std::process::Command` inside `tokio::task::block_in_place` — same pattern as the current stub. The subprocess is spawned with `--print --output-format stream-json --verbose`. The prompt is written to stdin, stdin is closed, then stdout is read line by line.

Each line is parsed as `serde_json::Value`. Lines are categorized by their `type` field. `assistant` lines have their text content blocks collected. The `result` line provides the final response text, cost, tokens, and session ID.

When `json_schema` is `Some`:
1. The CLI receives `--json-schema <schema>` which instructs Claude to format output per the schema
2. The `result` field in the Result message should contain JSON
3. First attempt: `serde_json::from_str` on the result string
4. If that fails: apply `recover_json()` and retry
5. If parsing succeeds: return `Ok(LxVal::ok(LxVal::from(json_value)))` — a real lx Record/List
6. If parsing fails: fall through to the text return path (backward compatible)

When `json_schema` is `None`:
- Return `Ok(LxVal::ok(record! { "text" => ..., "cost_usd" => ..., "turns" => ..., "input_tokens" => ..., "output_tokens" => ..., "session_id" => ... }))`
- The `text` field matches the current stub's return format for backward compatibility
- The additional fields are new but harmless — existing code accesses `.text` and ignores unknown fields

# Gotchas

- **`LlmBackend` methods are sync.** The `block_in_place` pattern is required and already used by the current stub. Do not make the trait async.
- **`std::process::Command` is sufficient.** No need for tokio `process` feature. The current stub already uses sync subprocess spawning inside `block_in_place`. This works fine since the LLM call is the blocking operation, not the process management.
- **NDJSON lines may contain `assistant` messages with partial content.** Collect ALL assistant text blocks across ALL assistant messages, not just the last one. Join with newlines.
- **The `result` line's `result` field is `Option<String>`.** It may be null if the response was conversational. Fall back to the collected assistant text.
- **The `result` line's `usage` is a nested object.** Access `usage.input_tokens` and `usage.output_tokens` via `json.get("usage").and_then(|u| u.get("input_tokens")).and_then(|v| v.as_u64())`.
- **`LxVal::from(serde_json::Value)` exists at `serde_impl.rs:73`.** It handles null→None, bool→Bool, i64→Int, f64→Float, string→Str, array→List, object→Record recursively. No conversion code needed.
- **`LxVal::int()` takes `impl Into<BigInt>` and `i64` has a `From<i64> for BigInt` impl.** Use `LxVal::int(tokens as i64)` for integer fields. No `num-bigint` dependency needed in `lx-cli`.
- **`record!` macro is exported from the `lx` crate.** Used as `record! { "key" => LxVal::str("val") }`. Already used by the current stub.
- **`recover_json` must not add a regex dependency.** Use simple `str::replace` loops and line-by-line processing. The `lx-cli` crate currently has zero regex dependencies.
- **The `--verbose` flag is needed with `--output-format stream-json`** to get the full NDJSON output including system init and result messages. Without it, some message types may be suppressed.

# Files Affected

| File | Change |
|------|--------|
| `crates/lx-cli/src/llm_backend.rs` | Rewrite: add `--json-schema`, switch to `--output-format stream-json`, parse NDJSON lines, extract rich metadata, add `recover_json` helper, structured output parsing |
| `pkg/connectors/tool_search.lx` | New file: ToolSearch tool for on-demand tool discovery (~30 lines) |

---

## Task List

### Task 1: Add recover_json helper function

Edit `crates/lx-cli/src/llm_backend.rs`. Add a new function `fn recover_json(raw: &str) -> String` above the existing `parse_response` function. The function creates a `String` from the input and applies these transformations in order:

1. Triple backtick extraction: if the string contains ` ``` `, split on ` ``` `. If there are at least 2 occurrences, take the content between the first and second occurrence. Trim the first line of that content (it may be a language tag like `json`). If no triple backticks found, keep the original.
2. JSON bracket extraction: use `find` to locate the first `{` or `[` character (whichever has the smaller index). Use `rfind` to locate the last `}` or `]` character (whichever has the larger index). If both found, slice the string to include only that range (inclusive of both bracket characters).
3. Trailing comma removal: in a loop, replace `,}` with `}` and `,]` with `]`. Also replace `, }` with `}` and `, ]` with `]`. Continue looping until no more replacements are made (check that the string length didn't change).
4. Single-line comment removal: split by `\n`. For each line, find the index of `//`. If found, count the number of `"` characters in the substring before that index. If the count is even (the `//` is not inside a string literal), truncate the line at that index. Rejoin with `\n`.
5. Return the cleaned string.

### Task 2: Rewrite prompt_with and parse_response

Edit `crates/lx-cli/src/llm_backend.rs`. Replace the `prompt_with` method body and the `parse_response` function entirely.

**New `prompt_with` body** (inside the existing `tokio::task::block_in_place` closure):

Build the Command: `Command::new("claude")` with args `--print`, `--output-format`, `stream-json`, `--verbose`. If `opts.tools` is non-empty, add `--allowedTools` with `opts.tools.join(",")`. If `opts.max_turns` is `Some(n)`, add `--max-turns` with `n.to_string()`. If `opts.json_schema` is `Some(ref schema)`, add `--json-schema` with `schema` as the argument. Set stdin/stdout/stderr to piped.

Spawn the child. Write `opts.prompt` to stdin. Drop stdin (close it). Read stdout to string via `child.wait_with_output()`. Check exit status — if non-zero, return `Ok(LxVal::err_str(...))` with the stderr content (same as current).

Call `parse_ndjson(&stdout, opts.json_schema.is_some(), span)` to process the output.

**New `parse_ndjson` function** with signature `fn parse_ndjson(raw: &str, structured: bool, _span: SourceSpan) -> Result<LxVal, LxError>`:

Split `raw` by newlines. For each non-empty line, attempt `serde_json::from_str::<serde_json::Value>(&line)`. Skip lines that fail to parse (the CLI may emit non-JSON debug output).

Maintain mutable state: `full_text: Vec<String>` for collecting assistant text, and `result_msg: Option<serde_json::Value>` for the result line.

For each parsed JSON object, check the `type` field:
- If `"assistant"`: get the `message.content` array. For each element where `type` is `"text"`, push the `text` field value to `full_text`.
- If `"result"`: store the entire object as `result_msg`.
- Otherwise: skip.

After all lines processed, extract from `result_msg` (if Some):
- `response_text`: `result_msg["result"]` as string, or if null, join `full_text` with `"\n"`
- `cost`: `result_msg["total_cost_usd"]` as f64, default 0.0
- `turns`: `result_msg["num_turns"]` as i64, default 0
- `input_tokens`: `result_msg["usage"]["input_tokens"]` as i64, default 0
- `output_tokens`: `result_msg["usage"]["output_tokens"]` as i64, default 0
- `is_error`: `result_msg["is_error"]` as bool, default false
- `session_id`: `result_msg["session_id"]` as string, optional

If `result_msg` is None (no result line found), set `response_text` to `full_text.join("\n")` and all metrics to defaults.

If `is_error` is true, return `Ok(LxVal::err_str(&response_text))`.

If `structured` is true: attempt `serde_json::from_str::<serde_json::Value>(&response_text)`. If that fails, call `recover_json(&response_text)` and retry. If either parse succeeds, return `Ok(LxVal::ok(LxVal::from(json_val)))`. If both fail, fall through.

Return `Ok(LxVal::ok(record! { "text" => LxVal::str(&response_text), "cost_usd" => LxVal::Float(cost), "turns" => LxVal::int(turns), "input_tokens" => LxVal::int(input_tokens), "output_tokens" => LxVal::int(output_tokens), "session_id" => session_id.map(|s| LxVal::str(s)).unwrap_or(LxVal::None) }))`.

Remove the old `parse_response` function entirely — it is replaced by `parse_ndjson`.

### Task 3: Format and commit

Run the following command verbatim, exactly as written, with no modifications: `just fmt`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way.

### Task 4: Commit backend rewrite

Run the following command verbatim, exactly as written, with no modifications: `git add -A && git commit -m "feat: fix ClaudeCodeLlmBackend with json-schema, NDJSON parsing, SAP recovery, and cost reporting"`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way. CRITICAL: Do NOT use `$()`, heredocs, `cat <<EOF`, or any subshell substitution. Do NOT append `--trailer`, `Co-authored-by`, `Signed-off-by`, or any other trailer/metadata.

### Task 5: Create the ToolSearch tool

Create a new file `pkg/connectors/tool_search.lx`. Add a header comment: `-- Tool search — indexes available tools and returns matches on demand to avoid eager loading.`

Define a `Tool +ToolSearch` with description `"Search available tools by capability. Returns top matches with name and description."`. The `run` method takes a record with fields `query` (Str) and `tools` (List of records with `name` and `description` fields). Split the query into words using `split " "` and lowercase each with `map lower`. For each tool in the tools list, lowercase the tool name and split the tool description into words and lowercase. Count how many query words appear in either the tool name (using `contains?`) or the tool description words (using `any?`). Compute relevance as hits divided by total query word count (as Float, add `+ 0.0` to force Float division). Filter tools where relevance is above 0.0. Sort by relevance descending using `sort_by (t) { 0.0 - t.relevance }` (negate for descending). Take the top 5. Return as a list of records with `name`, `description`, and `relevance` fields.

### Task 6: Format and commit

Run the following command verbatim, exactly as written, with no modifications: `just fmt`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way.

### Task 7: Commit ToolSearch tool

Run the following command verbatim, exactly as written, with no modifications: `git add -A && git commit -m "feat: add ToolSearch tool for on-demand tool discovery"`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way. CRITICAL: Do NOT use `$()`, heredocs, `cat <<EOF`, or any subshell substitution. Do NOT append `--trailer`, `Co-authored-by`, `Signed-off-by`, or any other trailer/metadata.

### Task 8: Run tests

Run the following command verbatim, exactly as written, with no modifications: `just test`. Do NOT pipe, redirect, or append shell operators. If any tests fail, fix all failures and re-run until all tests pass.

### Task 9: Run diagnostics

Run the following command verbatim, exactly as written, with no modifications: `just diagnose`. Do NOT pipe, redirect, or append shell operators. Fix ALL reported errors AND warnings. Re-run `just diagnose` until the output is completely clean with zero errors and zero warnings.

### Task 10: Final format

Run the following command verbatim, exactly as written, with no modifications: `just fmt`. Do NOT pipe, redirect, or append shell operators.

### Task 11: Final commit

Run the following command verbatim, exactly as written, with no modifications: `git add -A && git commit -m "fix: final verification cleanup"`. Do NOT pipe, redirect, or append shell operators. CRITICAL: Do NOT use `$()`, heredocs, `cat <<EOF`, or any subshell substitution — just a plain `-m "message"` flag. Do NOT append `--trailer`, `Co-authored-by`, `Signed-off-by`, or any other trailer/metadata.

### Task 12: Remove work item file

Run the following command verbatim, exactly as written, with no modifications: `rm work_items/OUTPUT_RELIABILITY.md && git add -A && git commit -m "chore: remove completed work item"`. Do NOT pipe, redirect, or append shell operators. Do NOT append `--trailer`, `Co-authored-by`, `Signed-off-by`, or any other trailer/metadata.

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

These are rules you (the executor) are known to violate. Re-read before starting each task:

1. **NEVER run raw cargo commands.** No `cargo check`, `cargo test`, `cargo clippy`, `cargo fmt`. ALWAYS use `just fmt`, `just test`, `just diagnose`, `just fix`. The `just` recipes include additional steps that raw `cargo` skips.
2. **Between implementation tasks, ONLY run `just fmt` + `git commit`.** Do NOT run `just test`, `just diagnose`, or any compilation/verification command between implementation tasks. These are expensive and only belong in the FINAL VERIFICATION section at the end.
3. **Run commands VERBATIM.** Copy-paste the exact command from the task description. Do not modify, "improve," or substitute commands. Do NOT append `--trailer`, `Co-authored-by`, or any metadata to commit commands.
4. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
5. **`just fmt` is not `cargo fmt`.** `just fmt` runs `dx fmt` + `cargo fmt` + `eclint`. Running `cargo fmt` alone is wrong.
6. **Do NOT modify verbatim commands.** No piping (`| tail`, `| head`, `| grep`), no redirects (`2>&1`, `> /dev/null`), no subshells. Run the exact command string as written — nothing appended, nothing prepended.

## Task Loading Instructions

Read each `### Task N:` entry from the `## Task List` section above. For each task, call `TaskCreate` with:
- `subject`: The task heading text (after `### Task N:`) — copied VERBATIM, not paraphrased
- `description`: The full body text under that heading — copied VERBATIM, not paraphrased, summarized, or reworded. Every sentence, every command, every instruction must be transferred exactly as written. Do NOT omit lines, rephrase instructions, drop the "verbatim" language from command instructions, or inject your own wording.
- `activeForm`: A present-continuous form of the subject (e.g., "Rewriting prompt_with and parse_response")

After creating all tasks, use `TaskUpdate` to set `addBlockedBy` on each task N (N > 1) pointing to task N-1, enforcing strict sequential execution.

Execute tasks strictly in order — mark each `in_progress` before starting and `completed` when done. Run commands EXACTLY as written in the task description — do not substitute `cargo` for `just` or vice versa. Do not run any command not specified in the current task. Do not "pre-check" compilation between implementation tasks. If a task says "Run the following command verbatim" then copy-paste that exact command — do not modify it. Do NOT append `--trailer`, `Co-authored-by`, `Signed-off-by`, or any other trailer/metadata to git commit commands. Do NOT paraphrase, summarize, reword, combine, split, reorder, skip, or add tasks beyond what is in the Task List section.
