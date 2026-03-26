# Goal

Replace the 67-line `ClaudeCodeLlmBackend` stub in `crates/lx-cli/src/llm_backend.rs` with a proper implementation using the `claude-cli-sdk` crate (pomdotdev). The current stub spawns `claude --print --output-format json`, ignores `json_schema` entirely, has no streaming support, no tool use protocol, and returns only raw text. The new implementation uses `claude-cli-sdk`'s typed client for NDJSON streaming over the Claude Code CLI subprocess protocol, properly handles structured output via `--json-schema`, returns rich response records (text, cost, tokens, turns, session_id, structured output), and uses the crate's `MockTransport` for testing. The `LlmBackend` trait stays unchanged — alternative backends remain swappable via the existing manifest `[backends]` configuration.

# Why

- `llm.prompt_structured` is broken — it creates `LlmOpts` with `json_schema: Some(schema)` but `ClaudeCodeLlmBackend.prompt_with` ignores the `json_schema` field. It never passes it to the claude CLI or attempts to parse the response as JSON. Every `think_structured` call in every lx program gets back raw text, not parsed structured output.
- The current stub spawns a new subprocess per call with `std::process::Command` (blocking), waits for full output, and parses a single JSON object. It cannot stream, cannot do multi-turn, cannot handle tool use responses, and cannot report cost/token usage back to lx programs.
- BAML's research shows Schema-Aligned Parsing recovers 92-94% of structured outputs that function calling misses. The `claude-cli-sdk` gives us access to the CLI's `--json-schema` flag for native structured output, and the `ResultMessage.result` field carries the validated output. SAP-style recovery can be applied on top as a safety net.
- The `claude-cli-sdk` crate has the cleanest API of the four evaluated Rust CLI wrappers: `typed-builder` with compile-time required fields, proper `futures::Stream` trait on streaming responses, `#[non_exhaustive]` error types with `is_retriable()`, `MockTransport` + `ScenarioBuilder` for testing, lean dependencies (no reqwest, no bloat), and no red flags in the source code.

# What Changes

**1. Add `claude-cli-sdk` dependency to `lx-cli`**

Add `claude-cli-sdk` to `crates/lx-cli/Cargo.toml` dependencies. Add `process` and `io-util` features to the workspace `tokio` dependency in the root `Cargo.toml` since `claude-cli-sdk` requires them and the workspace currently only has `macros`, `rt-multi-thread`, `sync`, `time`.

**2. Rewrite `ClaudeCodeLlmBackend` in `crates/lx-cli/src/llm_backend.rs`**

Replace the current 67-line stub with an implementation that uses `claude-cli-sdk`. The struct stays named `ClaudeCodeLlmBackend` and continues to implement `lx::runtime::LlmBackend`. The two trait methods (`prompt` and `prompt_with`) are the only public interface — callers do not change.

The `prompt` method delegates to `prompt_with` with default `LlmOpts` (same as now).

The `prompt_with` method:
- Builds a `claude_cli_sdk::ClientConfig` from `LlmOpts`: sets `prompt` from `opts.prompt`, sets `allowed_tools` from `opts.tools` if non-empty, sets `max_turns` from `opts.max_turns` if `Some`, sets `json_schema` from `opts.json_schema` if `Some` (this is the critical fix — the stub ignores this), sets `permission_mode` to `BypassPermissions` (lx agents handle their own permissions).
- Calls `claude_cli_sdk::query(config).await` to get a `Vec<Message>`.
- Iterates the messages to find the `Message::Result` variant. Extracts: `result` (the response text or structured output), `total_cost_usd`, `num_turns`, `usage` (input_tokens, output_tokens), `is_error`, `session_id`.
- Also collects all `Message::Assistant` variants and joins their text content blocks to get the full assistant text (since `result` on `ResultMessage` may be `None` if the response was conversational rather than one-shot).
- When `opts.json_schema` is `Some` and the response contains a `result` string, attempts `serde_json::from_str::<serde_json::Value>` on the result. If that fails, applies SAP-style recovery (strip markdown fences, find first `{`/`[`, remove trailing commas, remove single-line comments) and retries the parse. If JSON parsing succeeds, converts the `serde_json::Value` to `LxVal` using the existing `impl From<serde_json::Value> for LxVal` at `crates/lx/src/value/serde_impl.rs:73`. Returns `Ok(LxVal::ok(parsed_value))`.
- When `opts.json_schema` is `None` or JSON parsing fails, returns the same format as the current stub: `Ok(LxVal::ok(record! { "text" => LxVal::str(&text), "cost_usd" => ..., "turns" => ..., "input_tokens" => ..., "output_tokens" => ..., "session_id" => ... }))`. The additional fields are new but backward-compatible — existing code accesses `.text` and ignores unknown fields.
- On error (CLI not found, process exit, etc.), returns `Ok(LxVal::err_str(format!("llm: {error}")))` matching the current convention. Does NOT return `Err(LxError)` for LLM failures — lx programs handle LLM errors via `Ok`/`Err` pattern matching on the result value, not via Rust error propagation.

The `block_in_place` wrapper stays — `LlmBackend` methods are sync (not async), so the implementation uses `tokio::task::block_in_place` with `tokio::runtime::Handle::current().block_on()` to call the async `claude_cli_sdk::query()` from the sync trait method. This is the same pattern the current stub uses.

**3. SAP-style JSON recovery as a helper function**

Add a `recover_json(raw: &str) -> String` function in `llm_backend.rs`. Applied when structured output parsing fails on the first attempt. Transformations in order:
- If the string contains triple backticks, extract content between first opening and next closing triple-backtick lines (strip language tag if present).
- Find first `{` or `[`, trim everything before. Find last `}` or `]`, trim everything after.
- Remove trailing commas before `}` or `]` using simple string replacement in a loop (no regex crate needed).
- Remove single-line `//` comments: for each line, count `"` characters before the `//` position; if even, truncate at `//`.
- Return the cleaned string.

**4. Add ToolSearch tool in `pkg/connectors/tool_search.lx`**

A new lx Tool that takes a query string and a list of tool records (each with `name` and `description` fields), scores each tool by keyword overlap between the query and the tool's name+description, and returns the top 5 matches with `name`, `description`, and `relevance` fields. Agents include this in their tools list to discover other tools on demand instead of loading all tool definitions into the system prompt.

# How It Works

The `LlmBackend` trait in `crates/lx/src/runtime/mod.rs` is unchanged — it has two methods: `prompt(&self, text, span)` and `prompt_with(&self, opts, span)`. Both return `Result<LxVal, LxError>`. The trait is implemented by `ClaudeCodeLlmBackend` in `lx-cli` and by `NoopLlmBackend` in the core `lx` crate. The core crate has no dependency on `claude-cli-sdk` — only `lx-cli` depends on it. Alternative backends (e.g., a direct Anthropic API client, a local model runner) can implement `LlmBackend` and be wired in via `apply_manifest_backends` in `main.rs`.

The `claude-cli-sdk` crate communicates with the Claude Code CLI via NDJSON over stdin/stdout. It spawns the `claude` binary as a subprocess, sends the prompt, and reads a stream of typed `Message` objects. The `query()` free function collects all messages and returns `Vec<Message>`. The `Message::Result` variant carries cost, usage, turn count, and optionally the structured output. The `Message::Assistant` variants carry the text content blocks.

The `claude-cli-sdk`'s `ClientConfig::builder().prompt("...").build()` requires only the `prompt` field at compile time (enforced by `typed-builder`). All other fields (`model`, `max_turns`, `allowed_tools`, `permission_mode`, etc.) are optional with sensible defaults. The `ClientConfig` does not have a dedicated `json_schema` field, but it has `extra_args: BTreeMap<String, Option<String>>` for passing arbitrary CLI flags. Inserting `("json-schema", Some(schema_string))` produces `--json-schema <schema>` on the CLI invocation, which instructs Claude to return output conforming to the schema. The `ResultMessage.result` field carries the response text (which will be the structured JSON when `--json-schema` is used).

The SAP recovery function is a pure string transformation applied only when structured output is requested and initial JSON parsing fails. It handles the common cases: Claude wrapping JSON in markdown fences, chain-of-thought text before the JSON, trailing commas in generated JSON, and JavaScript-style comments. The `impl From<serde_json::Value> for LxVal` at `serde_impl.rs:73` handles the conversion from parsed JSON to lx values (null→None, bool→Bool, number→Int/Float, string→Str, array→List, object→Record).

The return value convention: the current stub returns `Ok(LxVal::ok(record! { "text" => ... }))`. The new implementation adds optional fields (`cost_usd`, `turns`, `input_tokens`, `output_tokens`, `session_id`) to the record. When structured output is requested and parsed successfully, the return value is `Ok(LxVal::ok(parsed_lx_val))` instead of a text-wrapping record. Existing lx code that accesses `.text` on the result continues to work for non-structured calls. Code using `think_structured` gets a real lx Record/List instead of a text string.

# Gotchas

- **`LlmBackend` methods are sync, `claude-cli-sdk` is async.** The `block_in_place` + `block_on` pattern is required. The current stub already does this. Do not make the trait async — it would require changing every call site in the interpreter.
- **Tokio `process` feature required.** The workspace `Cargo.toml` currently has `tokio = { features = ["macros", "rt-multi-thread", "sync", "time"] }`. Adding `process` and `io-util` is necessary for `claude-cli-sdk` to spawn and communicate with the CLI subprocess.
- **`claude-cli-sdk` requires the `claude` binary on PATH.** This is already the case for the current stub (it also spawns `claude`). The `claude-cli-sdk` crate provides a clear `Error::CliNotFound` when the binary is missing.
- **The current `parse_response` function returns `Ok(LxVal::ok(record! { "text" => ... }))`.** The new implementation must preserve this return shape for non-structured calls so existing lx programs that access `.text` on the result don't break.
- **`opts.tools` in `LlmOpts` is `Vec<String>` — tool names, not definitions.** These map to `--allowedTools` on the CLI, which is a comma-separated list of tool names (e.g., "Bash", "Read", "Write"). The `claude-cli-sdk`'s `ClientConfig` accepts `allowed_tools: Vec<String>`.
- **`ClientConfig` has no `json_schema` field.** Pass it via `extra_args: BTreeMap<String, Option<String>>` with key `"json-schema"` and value `Some(schema_string)`. The `extra_args` BTreeMap is appended last in `to_cli_args()`, producing `--json-schema <schema>`. This was verified against the `claude` CLI's `--help` output which shows `--json-schema <schema>` as a supported flag.
- **`ResultMessage.usage` fields are `u32`, not `u64`.** The `Usage` struct has `input_tokens: u32`, `output_tokens: u32`, `cache_read_input_tokens: u32`, `cache_creation_input_tokens: u32`. Use `as i64` when converting to `LxVal::int()`.
- **`ResultMessage.session_id` is `Option<String>`, a direct field — not a method.** Access it as `result_msg.session_id.as_deref()` for `Option<&str>`.
- **`serde_json` is already a workspace dependency.** No new serde dependencies needed.
- **The `record!` macro is exported from `lx` crate via `#[macro_export]`.** It creates `LxVal::Record` from key-value pairs. Used as `record! { "key" => LxVal::str("val") }`.
- **`LxVal::from(serde_json::Value)` exists at `crates/lx/src/value/serde_impl.rs:73`.** It handles null, bool, number (i64 → Int, f64 → Float), string, array, and object recursively. Import it in `llm_backend.rs` via `use lx::value::LxVal` (the From impl is available wherever LxVal is in scope).

# Files Affected

| File | Change |
|------|--------|
| `Cargo.toml` (workspace root) | Add `process`, `io-util` to tokio features |
| `crates/lx-cli/Cargo.toml` | Add `claude-cli-sdk` dependency |
| `crates/lx-cli/src/llm_backend.rs` | Full rewrite: `ClaudeCodeLlmBackend` using `claude-cli-sdk`, add `recover_json` helper |
| `pkg/connectors/tool_search.lx` | New file: ToolSearch tool for on-demand tool discovery |

---

## Task List

### Task 1: Add tokio process and io-util features to workspace

Edit the root `Cargo.toml`. Find the tokio workspace dependency at line 63: `tokio = { version = "1.50.0", features = ["macros", "rt-multi-thread", "sync", "time"] }`. Add `"process"` and `"io-util"` to the features array. The line should become: `tokio = { version = "1.50.0", features = ["macros", "rt-multi-thread", "sync", "time", "process", "io-util"] }`.

### Task 2: Add claude-cli-sdk dependency to lx-cli

Edit `crates/lx-cli/Cargo.toml`. Add `claude-cli-sdk = "0.5"` to the `[dependencies]` section, after the existing `tokio.workspace = true` line. Also add `futures = "0.3"` since the `claude-cli-sdk` streaming response requires `StreamExt` from futures.

### Task 3: Format and commit

Run the following command verbatim, exactly as written, with no modifications: `just fmt`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way.

### Task 4: Commit dependency additions

Run the following command verbatim, exactly as written, with no modifications: `git add -A && git commit -m "feat: add claude-cli-sdk and tokio process features for LLM backend"`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way. CRITICAL: Do NOT use `$()`, heredocs, `cat <<EOF`, or any subshell substitution. Do NOT append `--trailer`, `Co-authored-by`, `Signed-off-by`, or any other trailer/metadata.

### Task 5: Implement recover_json helper function

Edit `crates/lx-cli/src/llm_backend.rs`. Remove the existing `parse_response` function (lines 51-67). Add a new function `fn recover_json(raw: &str) -> String` that takes a raw string and returns a cleaned version suitable for JSON parsing. The function creates a mutable `String` from the input and applies these transformations in order:

1. Triple backtick extraction: if the string contains "```", find the byte index of the first occurrence of "```". Find the end of that line (next newline). Find the next occurrence of "```" after that. Extract the substring between end-of-first-line and start-of-second-occurrence. If not found, keep the original string.
2. JSON bracket extraction: find the byte index of the first `{` or `[` character (whichever comes first). Find the byte index of the last `}` or `]` character (whichever comes last). If both found, take the substring between them (inclusive of the brackets).
3. Trailing comma removal: loop replacing `,}` with `}` and `,]` with `]` until no more replacements occur. For whitespace between the comma and bracket, also handle `, }` and `, ]` by replacing `, }` with ` }` then `,}` with `}` (or simply use `str::replace` in a loop on the patterns `,}`, `, }`, `,\n}`, `,]`, `, ]`, `,\n]`).
4. Single-line comment removal: split by newlines, for each line find the index of `//`, count the number of `"` characters before that index, if the count is even (meaning the `//` is not inside a string), truncate the line at that index. Rejoin with newlines.
5. Return the cleaned string.

### Task 6: Rewrite ClaudeCodeLlmBackend using claude-cli-sdk

Edit `crates/lx-cli/src/llm_backend.rs`. Replace the existing imports and `ClaudeCodeLlmBackend` implementation entirely. The new file should:

**Imports:** `use std::collections::BTreeMap;` `use claude_cli_sdk::{ClientConfig, Message, ContentBlock, PermissionMode};` `use lx::error::LxError;` `use lx::record;` `use lx::runtime::{LlmBackend, LlmOpts};` `use lx::value::LxVal;` `use miette::SourceSpan;`

**Struct:** Keep `pub struct ClaudeCodeLlmBackend;` unchanged.

**`impl LlmBackend for ClaudeCodeLlmBackend`:**

The `prompt` method delegates to `prompt_with` with default `LlmOpts` (same as current).

The `prompt_with` method wraps its body in `tokio::task::block_in_place(|| { let handle = tokio::runtime::Handle::current(); handle.block_on(async { ... }) })`. Inside the async block:

1. Build `ClientConfig`: start with `ClientConfig::builder().prompt(&opts.prompt)`. If `opts.tools` is non-empty, call `.allowed_tools(opts.tools.clone())`. If `opts.max_turns` is `Some(n)`, call `.max_turns(n)`. Set `.permission_mode(claude_cli_sdk::PermissionMode::BypassPermissions)`. If `opts.json_schema` is `Some(ref schema)`, pass it via `extra_args`: create a `BTreeMap::from([("json-schema".to_string(), Some(schema.clone()))])` and call `.extra_args(btree_map)` — the `ClientConfig` does not have a dedicated `json_schema` field, but `extra_args` passes arbitrary CLI flags, and the `claude` CLI accepts `--json-schema <schema>` (verified). Call `.build()`.

2. Call `claude_cli_sdk::query(config).await`. Map the error to `LxError::runtime(format!("llm: {e}"), span)` — but since the current convention is to return `Ok(LxVal::err_str(...))` for LLM errors (not `Err(LxError)`), catch the error and return `Ok(LxVal::err_str(format!("llm: {e}")))` instead.

3. Process the returned `Vec<Message>`: iterate to find `Message::Result(ref result_msg)`. Also iterate all `Message::Assistant(ref asst)` and for each, iterate `asst.message.content` to collect all `ContentBlock::Text(ref t)` blocks, joining their `.text` fields with newlines into a `full_text: String`.

4. Extract from the `ResultMessage`: `result_msg.result` (Option<String>), `result_msg.total_cost_usd` (Option<f64>), `result_msg.num_turns` (u32), `result_msg.usage.input_tokens` (u32), `result_msg.usage.output_tokens` (u32), `result_msg.is_error` (bool), `result_msg.session_id` (Option<String>, direct field — not a method). If `is_error` is true, return `Ok(LxVal::err_str(full_text))`.

5. If `opts.json_schema.is_some()`: take the `result` string (or fall back to `full_text`). Attempt `serde_json::from_str::<serde_json::Value>(&text)`. If that fails, call `recover_json(&text)` and retry `serde_json::from_str::<serde_json::Value>(&recovered)`. If either parse succeeds, return `Ok(LxVal::ok(LxVal::from(json_val)))`. If both fail, fall through to the text return path.

6. Default text return: build a record with all available fields. Use `record!` macro: `"text" => LxVal::str(&response_text)`, `"cost_usd" => LxVal::Float(cost)`, `"turns" => LxVal::int(turns as i64)`, `"input_tokens" => LxVal::int(input_tokens as i64)`, `"output_tokens" => LxVal::int(output_tokens as i64)`. For the session_id, if available: `"session_id" => LxVal::str(sid)`. Wrap in `Ok(LxVal::ok(record))`.

Use `LxVal::int()` for integer fields — it takes `impl Into<BigInt>` and `i64` has a `From` impl, so no direct `num-bigint` dependency needed in `lx-cli`.

### Task 7: Format and commit

Run the following command verbatim, exactly as written, with no modifications: `just fmt`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way.

### Task 8: Commit backend rewrite

Run the following command verbatim, exactly as written, with no modifications: `git add -A && git commit -m "feat: rewrite ClaudeCodeLlmBackend using claude-cli-sdk with structured output and SAP recovery"`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way. CRITICAL: Do NOT use `$()`, heredocs, `cat <<EOF`, or any subshell substitution. Do NOT append `--trailer`, `Co-authored-by`, `Signed-off-by`, or any other trailer/metadata.

### Task 9: Create the ToolSearch tool

Create a new file `pkg/connectors/tool_search.lx`. Add a header comment: `-- Tool search — indexes available tools and returns matches on demand to avoid eager loading.` Define a `Tool +ToolSearch` with description `"Search available tools by capability. Returns top matches with name and description."`. The `run` method takes a record with fields `query` (Str) and `tools` (List of records with `name` and `description` fields). Split the query into words using `split " "` and lowercase each with `map lower`. For each tool in the tools list, split its description into words and lowercase, also lowercase the tool name, count how many query words appear in either the tool description words or the tool name (using `contains?`), compute a relevance score as hits divided by total query word count. Filter tools with relevance above 0.0, sort by relevance descending (negate relevance for sort or use `sort_by` then `reverse`), take the top 5, and return them as a list of records with `name`, `description`, and `relevance` fields.

### Task 10: Format and commit

Run the following command verbatim, exactly as written, with no modifications: `just fmt`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way.

### Task 11: Commit ToolSearch tool

Run the following command verbatim, exactly as written, with no modifications: `git add -A && git commit -m "feat: add ToolSearch tool for on-demand tool discovery"`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way. CRITICAL: Do NOT use `$()`, heredocs, `cat <<EOF`, or any subshell substitution. Do NOT append `--trailer`, `Co-authored-by`, `Signed-off-by`, or any other trailer/metadata.

### Task 12: Run tests

Run the following command verbatim, exactly as written, with no modifications: `just test`. Do NOT pipe, redirect, or append shell operators. If any tests fail, fix all failures and re-run until all tests pass.

### Task 13: Run diagnostics

Run the following command verbatim, exactly as written, with no modifications: `just diagnose`. Do NOT pipe, redirect, or append shell operators. Fix ALL reported errors AND warnings. Re-run `just diagnose` until the output is completely clean with zero errors and zero warnings.

### Task 14: Final format

Run the following command verbatim, exactly as written, with no modifications: `just fmt`. Do NOT pipe, redirect, or append shell operators.

### Task 15: Final commit

Run the following command verbatim, exactly as written, with no modifications: `git add -A && git commit -m "fix: final verification cleanup"`. Do NOT pipe, redirect, or append shell operators. CRITICAL: Do NOT use `$()`, heredocs, `cat <<EOF`, or any subshell substitution — just a plain `-m "message"` flag. Do NOT append `--trailer`, `Co-authored-by`, `Signed-off-by`, or any other trailer/metadata.

### Task 16: Remove work item file

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
- `activeForm`: A present-continuous form of the subject (e.g., "Rewriting ClaudeCodeLlmBackend")

After creating all tasks, use `TaskUpdate` to set `addBlockedBy` on each task N (N > 1) pointing to task N-1, enforcing strict sequential execution.

Execute tasks strictly in order — mark each `in_progress` before starting and `completed` when done. Run commands EXACTLY as written in the task description — do not substitute `cargo` for `just` or vice versa. Do not run any command not specified in the current task. Do not "pre-check" compilation between implementation tasks. If a task says "Run the following command verbatim" then copy-paste that exact command — do not modify it. Do NOT append `--trailer`, `Co-authored-by`, `Signed-off-by`, or any other trailer/metadata to git commit commands. Do NOT paraphrase, summarize, reword, combine, split, reorder, skip, or add tasks beyond what is in the Task List section.
