# Goal

Improve the reliability of what comes back from LLM calls through two changes: add a ToolSearch tool that indexes available tools and returns matches on demand (avoiding the 75k token cost of eager tool loading), and fix `llm.prompt_structured` to actually parse structured output as JSON with SAP-style edit-distance recovery (recovering malformed JSON in microseconds instead of re-prompting). Currently `llm.prompt_structured` ignores the `json_schema` field entirely — the `ClaudeCodeLlmBackend` never passes it to the claude CLI or to its response parser.

# Why

- The MCP token cost problem is real: 10 MCP servers x 15 tools x 500 tokens = 75,000 tokens of tool definitions before any user input. Cursor enforces a hard limit of 40 tools due to output degradation. Claude Code's tool search achieves 85% token reduction. A ToolSearch tool that indexes descriptions and returns matches on demand is the standard pattern — it is just a Tool, not a runtime feature.
- BAML's Schema-Aligned Parsing benchmarks show 92-94% structured output accuracy vs 57-87% for native function calling. SAP applies edit-distance recovery (strip markdown fences, fix trailing commas, coerce types, find JSON in chain-of-thought text) in under 10ms. Every `llm.prompt_structured` call benefits. The research consistently shows structured output reliability is a multiplier on agent success.

# What Changes

**1. ToolSearch tool in `pkg/connectors/tool_search.lx`**

A new Tool that takes a query string and returns the top matching tools from a registry. The tool maintains a list of all available tool descriptions (loaded from MCP servers, declared tools, etc.) and scores matches using keyword overlap between the query and each tool's name + description. Returns the top 5 matches with name, description, and relevance score. Agents include ToolSearch in their tools list alongside their core tools, and the LLM calls it to discover additional tools on demand instead of having all tool definitions in the system prompt.

**2. SAP-style parsing in the Rust structured output path**

The structured output path works as follows: `llm.prompt_structured` in `crates/lx/src/builtins/llm.rs` (function `bi_prompt_structured`, line 29) creates an `LlmOpts` with `json_schema: Some(schema)` and calls `ctx.llm.prompt_with(&opts, span)`. The actual LLM backend is `ClaudeCodeLlmBackend` in `crates/lx-cli/src/llm_backend.rs`. Its `prompt_with` method (line 17) spawns the `claude` CLI, gets the response, and calls `parse_response` (line 51) to extract the text. Two problems: (a) `prompt_with` ignores the `json_schema` field entirely — it never passes it to the `claude` CLI or to `parse_response`, and (b) `parse_response` only extracts the raw text string, never attempting to parse it as JSON per the schema.

The fix has two parts. First, pass `json_schema` from `LlmOpts` through to `parse_response`. Second, when `json_schema` is present, after extracting the text from Claude's response, apply SAP-style recovery transformations on the text before attempting `serde_json::from_str`: strip markdown code fences, find the first `{` or `[` and trim before it, remove trailing commas before `}` or `]`, remove single-line comments. If JSON parsing succeeds, return the parsed value as a Record/List LxVal. If it fails, return the raw text as before (no behavior change for non-structured calls).

# How It Works

**ToolSearch** is a pure lx Tool. It reads `registry.list_tools ()` (or a passed-in tools list) at initialization, builds an in-memory index of name + description strings, and on each `run` call scores the query against each tool's description using keyword overlap (split query into words, count how many appear in the tool description, normalize by query word count). No vector embeddings, no external dependencies. This matches the "BM25-lite" approach the Anthropic Agent SDK uses for its regex-based tool search variant.

**SAP recovery** is a pure Rust function (`recover_json`) in `crates/lx-cli/src/llm_backend.rs` that preprocesses the raw response text before `serde_json::from_str`. It is called inside `parse_response` only when `json_schema` is `Some`. The function is pure (no side effects, no LLM calls) and fast (string operations only). If JSON parsing succeeds after recovery, the parsed value is converted to an `LxVal` and returned as `Ok(LxVal::ok(parsed))`. If parsing fails after recovery, the raw text is returned exactly as before — no behavior change for existing callers. The `json_schema` string itself is not used for validation (it was already sent to the LLM as a prompt instruction) — it only serves as a flag indicating that structured output was requested and JSON parsing should be attempted.

# Files Affected

| File | Change |
|------|--------|
| `pkg/connectors/tool_search.lx` | New file — ToolSearch Tool (~50 lines) |
| `crates/lx-cli/src/llm_backend.rs` | Pass `json_schema` to `parse_response`; add `recover_json` helper; parse structured output as JSON with SAP recovery |

---

## Task List

### Task 1: Create the ToolSearch tool

Create a new file `pkg/connectors/tool_search.lx`. Add a header comment: `-- Tool search — indexes available tools and returns matches on demand to avoid eager loading.` Define a `Tool +ToolSearch` with description `"Search available tools by capability. Returns top matches with name and description."`. The `run` method takes a record with fields `query` (Str) and `tools` (List of records with `name` and `description` fields). Split the query into words using `split " "` and lowercase each. For each tool in the list, split its description into words and lowercase, count how many query words appear in the tool's description or name (using `contains?`), compute a relevance score as hits divided by query word count. Filter tools with relevance above 0.0, sort by relevance descending (using `sort_by` and `reverse`), take the top 5, and return them as a list of records with `name`, `description`, and `relevance` fields.

### Task 2: Format and commit

Run the following command verbatim, exactly as written, with no modifications: `just fmt`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way.

### Task 3: Commit ToolSearch tool

Run the following command verbatim, exactly as written, with no modifications: `git add -A && git commit -m "feat: add ToolSearch tool for on-demand tool discovery"`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way. CRITICAL: Do NOT use `$()`, heredocs, `cat <<EOF`, or any subshell substitution. Do NOT append `--trailer`, `Co-authored-by`, `Signed-off-by`, or any other trailer/metadata.

### Task 4: Pass json_schema through to parse_response in llm_backend.rs

Edit `crates/lx-cli/src/llm_backend.rs`. The current `prompt_with` method (line 17) calls `parse_response(&stdout, span)` at line 46 but does not pass `json_schema`. The current `parse_response` signature (line 51) is `fn parse_response(raw: &str, _span: SourceSpan) -> Result<LxVal, LxError>`.

Change `parse_response` signature to `fn parse_response(raw: &str, json_schema: Option<&str>, _span: SourceSpan) -> Result<LxVal, LxError>`. Update the call site in `prompt_with` to pass `opts.json_schema.as_deref()` as the second argument.

### Task 5: Format and commit

Run the following command verbatim, exactly as written, with no modifications: `just fmt`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way.

### Task 6: Commit json_schema passthrough

Run the following command verbatim, exactly as written, with no modifications: `git add -A && git commit -m "refactor: pass json_schema to parse_response in llm_backend"`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way. CRITICAL: Do NOT use `$()`, heredocs, `cat <<EOF`, or any subshell substitution. Do NOT append `--trailer`, `Co-authored-by`, `Signed-off-by`, or any other trailer/metadata.

### Task 7: Add SAP-style JSON recovery and structured output parsing

Edit `crates/lx-cli/src/llm_backend.rs`. Add a `recover_json` function above `parse_response` with signature `fn recover_json(raw: &str) -> String`. The function applies these transformations in order on a mutable `String` copy of the input:

1. If the string contains triple backticks, find the first line starting with three backticks, find the next line starting with three backticks, and extract everything between them. If the opening line has text after the backticks (like a language tag), ignore that text.
2. Find the index of the first `{` or `[` character. If found, remove everything before that index. Find the index of the last `}` or `]` character. If found, remove everything after that index.
3. Replace all occurrences of `,` followed by optional whitespace then `}` with just `}`. Replace all occurrences of `,` followed by optional whitespace then `]` with just `]`. Use simple `str::replace` or a loop — do not add a regex crate dependency.
4. Remove single-line comments: for each line, if it contains `//` that is not inside a quoted string, truncate the line at that position. A simple heuristic: split by lines, for each line find `//` and if the count of `"` characters before it is even, truncate there.
5. Return the cleaned string.

Then modify `parse_response`: after extracting the `text` variable (currently line 63 `raw.to_string()` fallback or the extracted `result`/`text` field), add a new block. If `json_schema` is `Some(_)`, call `recover_json(&text)` to get a cleaned string, then attempt `serde_json::from_str::<serde_json::Value>(&cleaned)`. If that succeeds, convert the `serde_json::Value` to an `LxVal` using the existing `impl From<serde_json::Value> for LxVal` at `crates/lx/src/value/serde_impl.rs` line 73 — call `LxVal::from(json_value)`. Wrap the result in `Ok(LxVal::ok(parsed_value))`. If JSON parsing fails after recovery, fall through to the existing behavior of returning the raw text wrapped in a record.

### Task 8: Format and commit

Run the following command verbatim, exactly as written, with no modifications: `just fmt`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way.

### Task 9: Commit SAP recovery

Run the following command verbatim, exactly as written, with no modifications: `git add -A && git commit -m "feat: add SAP-style JSON recovery for structured output parsing"`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way. CRITICAL: Do NOT use `$()`, heredocs, `cat <<EOF`, or any subshell substitution. Do NOT append `--trailer`, `Co-authored-by`, `Signed-off-by`, or any other trailer/metadata.

### Task 10: Run tests

Run the following command verbatim, exactly as written, with no modifications: `just test`. Do NOT pipe, redirect, or append shell operators. If any tests fail, fix all failures and re-run until all tests pass.

### Task 11: Run diagnostics

Run the following command verbatim, exactly as written, with no modifications: `just diagnose`. Do NOT pipe, redirect, or append shell operators. Fix ALL reported errors AND warnings. Re-run `just diagnose` until the output is completely clean with zero errors and zero warnings.

### Task 12: Final format

Run the following command verbatim, exactly as written, with no modifications: `just fmt`. Do NOT pipe, redirect, or append shell operators.

### Task 13: Final commit

Run the following command verbatim, exactly as written, with no modifications: `git add -A && git commit -m "fix: final verification cleanup"`. Do NOT pipe, redirect, or append shell operators. CRITICAL: Do NOT use `$()`, heredocs, `cat <<EOF`, or any subshell substitution — just a plain `-m "message"` flag. Do NOT append `--trailer`, `Co-authored-by`, `Signed-off-by`, or any other trailer/metadata.

### Task 14: Remove work item file

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
- `activeForm`: A present-continuous form of the subject (e.g., "Creating the ToolSearch tool")

After creating all tasks, use `TaskUpdate` to set `addBlockedBy` on each task N (N > 1) pointing to task N-1, enforcing strict sequential execution.

Execute tasks strictly in order — mark each `in_progress` before starting and `completed` when done. Run commands EXACTLY as written in the task description — do not substitute `cargo` for `just` or vice versa. Do not run any command not specified in the current task. Do not "pre-check" compilation between implementation tasks. If a task says "Run the following command verbatim" then copy-paste that exact command — do not modify it. Do NOT append `--trailer`, `Co-authored-by`, `Signed-off-by`, or any other trailer/metadata to git commit commands. Do NOT paraphrase, summarize, reword, combine, split, reorder, skip, or add tasks beyond what is in the Task List section.
