# Claude Code CLI: `--output-format stream-json`

Research date: 2026-03-26
CLI version tested: 2.1.84 (Claude Code)

## 1. Is `stream-json` a valid `--output-format` value?

Yes. The CLI help (`claude --help`) states:

```
--output-format <format>  Output format (only works with --print):
                          "text" (default), "json" (single result),
                          or "stream-json" (realtime streaming)
                          (choices: "text", "json", "stream-json")
```

Requirements:
- Must be used with `-p` / `--print`
- **Requires `--verbose`** -- without it, you get: `Error: When using --print, --output-format=stream-json requires --verbose`

Minimal invocation:
```bash
claude -p --output-format stream-json --verbose "your prompt"
```

Optional flag `--include-partial-messages` adds Anthropic SSE-style streaming deltas (message_start, content_block_delta, etc.) between the coarser assistant/result events.

## 2. JSON Event Types and Exact Structures

Each line of stdout is a self-contained JSON object with a `"type"` field. The stream follows this sequence:

### Event 1: `system` (subtype `init`)

Emitted once at the start. Contains session metadata, available tools, MCP server status, model info.

```json
{
  "type": "system",
  "subtype": "init",
  "cwd": "/home/user/project",
  "session_id": "bf2e322f-da46-4c48-90b1-b281a3e96fb8",
  "tools": ["Bash", "Edit", "Read", "Glob", "Grep", "Write", "..."],
  "mcp_servers": [
    {"name": "postgresql", "status": "connected"},
    {"name": "valkey", "status": "failed"}
  ],
  "model": "claude-opus-4-6[1m]",
  "permissionMode": "default",
  "apiKeySource": "none",
  "claude_code_version": "2.1.84",
  "output_style": "default",
  "agents": ["general-purpose", "Explore", "Plan"],
  "skills": ["compact", "review", "..."],
  "plugins": [
    {"name": "rust-analyzer-lsp", "path": "/...", "source": "..."}
  ],
  "uuid": "642876ef-...",
  "fast_mode_state": "off"
}
```

### Event 2+: `assistant` (one per model turn)

Contains the full Anthropic Messages API response for one turn. The `message.content` array holds `text` and/or `tool_use` blocks.

**Text-only response:**
```json
{
  "type": "assistant",
  "message": {
    "model": "claude-opus-4-6",
    "id": "msg_01UMqUpGoH9yrNKwux4tJR8i",
    "type": "message",
    "role": "assistant",
    "content": [
      {"type": "text", "text": "Hello world"}
    ],
    "stop_reason": null,
    "stop_sequence": null,
    "usage": {
      "input_tokens": 2,
      "cache_creation_input_tokens": 7689,
      "cache_read_input_tokens": 11367,
      "output_tokens": 1,
      "service_tier": "standard"
    },
    "context_management": null
  },
  "parent_tool_use_id": null,
  "session_id": "bf2e322f-...",
  "uuid": "121fb1f0-..."
}
```

**Tool-use response (assistant decides to call a tool):**
```json
{
  "type": "assistant",
  "message": {
    "model": "claude-opus-4-6",
    "id": "msg_01H2C959tgo9kzfycwiU2ceQ",
    "type": "message",
    "role": "assistant",
    "content": [
      {
        "type": "tool_use",
        "id": "toolu_01EStYjAWVCdCaPoE2w1GaJf",
        "name": "Read",
        "input": {"file_path": "/tmp/test_stream_json.txt"},
        "caller": {"type": "direct"}
      }
    ],
    "stop_reason": null,
    "stop_sequence": null,
    "usage": {
      "input_tokens": 2,
      "cache_creation_input_tokens": 7701,
      "cache_read_input_tokens": 11367,
      "output_tokens": 46,
      "service_tier": "standard"
    },
    "context_management": null
  },
  "parent_tool_use_id": null,
  "session_id": "592840fe-...",
  "uuid": "8be8b177-..."
}
```

### Event: `user` (tool result, injected by the CLI)

After a tool runs, the CLI emits the tool result as a synthetic user message:

```json
{
  "type": "user",
  "message": {
    "role": "user",
    "content": [
      {
        "type": "tool_result",
        "content": "     1\thello from test file\n",
        "is_error": false,
        "tool_use_id": "toolu_01EStYjAWVCdCaPoE2w1GaJf"
      }
    ]
  },
  "parent_tool_use_id": null,
  "session_id": "592840fe-...",
  "uuid": "31ce5c8a-...",
  "timestamp": "2026-03-26T13:53:13.511Z",
  "tool_use_result": "     1\thello from test file\n"
}
```

Error case:
```json
{
  "type": "user",
  "message": {
    "role": "user",
    "content": [
      {
        "type": "tool_result",
        "content": "File does not exist. Note: your current working directory is /home/user/project.",
        "is_error": true,
        "tool_use_id": "toolu_01EStYjAWVCdCaPoE2w1GaJf"
      }
    ]
  },
  "tool_use_result": "Error: File does not exist. ..."
}
```

### Event: `rate_limit_event`

Emitted after each API call completes:

```json
{
  "type": "rate_limit_event",
  "rate_limit_info": {
    "status": "allowed",
    "resetsAt": 1774544400,
    "rateLimitType": "five_hour",
    "overageStatus": "rejected",
    "overageDisabledReason": "org_level_disabled",
    "isUsingOverage": false
  },
  "uuid": "59f29be9-...",
  "session_id": "bf2e322f-..."
}
```

### Event: `result` (always the final line)

Contains the final assembled text, cost, timing, and aggregated usage:

```json
{
  "type": "result",
  "subtype": "success",
  "is_error": false,
  "duration_ms": 2450,
  "duration_api_ms": 2441,
  "num_turns": 1,
  "result": "Hello world",
  "stop_reason": "end_turn",
  "session_id": "bf2e322f-...",
  "total_cost_usd": 0.05387475,
  "usage": {
    "input_tokens": 2,
    "cache_creation_input_tokens": 7689,
    "cache_read_input_tokens": 11367,
    "output_tokens": 5,
    "server_tool_use": {"web_search_requests": 0, "web_fetch_requests": 0},
    "service_tier": "standard"
  },
  "modelUsage": {
    "claude-opus-4-6[1m]": {
      "inputTokens": 2,
      "outputTokens": 5,
      "cacheReadInputTokens": 11367,
      "cacheCreationInputTokens": 7689,
      "costUSD": 0.05387475,
      "contextWindow": 1000000,
      "maxOutputTokens": 64000
    }
  },
  "permission_denials": [],
  "fast_mode_state": "off",
  "uuid": "bafb2768-..."
}
```

### Streaming delta events (only with `--include-partial-messages`)

When `--include-partial-messages` is added, you also get `stream_event` lines wrapping the Anthropic streaming API events. These appear *before* the consolidated `assistant` event for each turn:

```json
{"type":"stream_event","event":{"type":"message_start","message":{"model":"claude-opus-4-6","id":"msg_016Qf...","type":"message","role":"assistant","content":[],"stop_reason":null,...}},"session_id":"...","parent_tool_use_id":null,"uuid":"..."}

{"type":"stream_event","event":{"type":"content_block_start","index":0,"content_block":{"type":"text","text":""}},"session_id":"...","parent_tool_use_id":null,"uuid":"..."}

{"type":"stream_event","event":{"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"1\n2\n3\n4\n5"}},"session_id":"...","parent_tool_use_id":null,"uuid":"..."}

{"type":"stream_event","event":{"type":"content_block_stop","index":0},"session_id":"...","parent_tool_use_id":null,"uuid":"..."}

{"type":"stream_event","event":{"type":"message_delta","delta":{"stop_reason":"end_turn","stop_sequence":null},"usage":{"output_tokens":13},"context_management":{"applied_edits":[]}},"session_id":"...","parent_tool_use_id":null,"uuid":"..."}

{"type":"stream_event","event":{"type":"message_stop"},"session_id":"...","parent_tool_use_id":null,"uuid":"..."}
```

After these streaming events, the consolidated `assistant` event still appears (full message with complete content).

## 3. Buffering Behavior

**Line-buffered through pipes.** Verified empirically by timestamping each line as it arrives:

```
[0.659s] type=system       <-- init arrives immediately
[5.141s] type=assistant     <-- arrives when API response completes
[5.187s] type=rate_limit_event
[5.188s] type=result        <-- final event, ~0ms after rate_limit
```

The `system` init event arrives well before the API call completes (~0.6s vs ~5.1s), proving that each JSON line is flushed immediately to stdout as it is produced. The CLI does **not** buffer the entire output before writing.

## 4. Extracting the Assistant's Response Text

### Method A: Just read the `result` event (simplest)

The final `result` event always has `result` containing the final text response as a plain string. If you only need the answer:

```bash
claude -p --output-format stream-json --verbose "prompt" \
  | jq -r 'select(.type == "result") | .result'
```

### Method B: Read `assistant` events for per-turn content

Each `assistant` event contains `message.content[]`. To extract text from all assistant turns:

```bash
claude -p --output-format stream-json --verbose "prompt" \
  | jq -r 'select(.type == "assistant") | .message.content[] | select(.type == "text") | .text'
```

### Method C: Streaming deltas with `--include-partial-messages`

For real-time character-by-character display:

```bash
claude -p --output-format stream-json --verbose --include-partial-messages "prompt" \
  | jq -r 'select(.type == "stream_event" and .event.type == "content_block_delta") | .event.delta.text // empty'
```

### Method D: Detect tool use

To watch for tool invocations and their results in real time:

```bash
# Tool calls (what the model wants to do)
jq -r 'select(.type == "assistant") | .message.content[] | select(.type == "tool_use") | "\(.name): \(.input)"'

# Tool results (what happened)
jq -r 'select(.type == "user") | .message.content[] | select(.type == "tool_result") | "[\(if .is_error then "ERROR" else "OK" end)] \(.content)"'
```

## 5. Summary of Event Sequence

For a simple single-turn response:
```
system (init) -> assistant (text) -> rate_limit_event -> result
```

For a multi-turn tool-use conversation:
```
system (init) -> assistant (tool_use) -> user (tool_result) -> rate_limit_event -> assistant (text) -> rate_limit_event -> result
```

With `--include-partial-messages`, each assistant turn is preceded by streaming delta events:
```
system (init) -> stream_event (message_start) -> stream_event (content_block_start) -> stream_event (content_block_delta)* -> stream_event (content_block_stop) -> stream_event (message_delta) -> stream_event (message_stop) -> assistant -> rate_limit_event -> result
```

## 6. Comparison: `--output-format json` vs `stream-json`

| Feature | `json` | `stream-json` |
|---|---|---|
| Number of lines | 1 (the result) | Multiple (one per event) |
| Requires `--verbose` | No | Yes |
| Streaming | No (waits until done) | Yes (line-buffered) |
| Tool use visibility | No | Yes (`assistant` + `user` events) |
| Partial messages | N/A | With `--include-partial-messages` |
| Final result shape | Same as `result` event | Same, but also has preceding events |

The `json` format emits a single JSON object identical in shape to the `result` event from `stream-json`.

## 7. Additional Related Flags

From `--help`:

- `--include-partial-messages` -- adds `stream_event` lines wrapping Anthropic SSE deltas (only works with `--print` and `--output-format=stream-json`)
- `--input-format stream-json` -- enables *input* streaming (send events to stdin), choices: `text`, `stream-json`
- `--replay-user-messages` -- re-emits user messages from stdin on stdout (only with both `--input-format=stream-json` and `--output-format=stream-json`)
