# Agent Pane — Chat Interface

## Goal

Implement the Agent pane surface as a chat interface for interactive agent conversations. Uses claude-agent-sdk-rs for bidirectional streaming, renders messages with markdown, displays tool call cards with approve/deny actions, and provides a text input for user messages. This replaces the stub AgentView with a functional agent conversation pane.

## Why

- Parallel agent sessions (agentmaxxing) are a Tier 2 developer value feature — require agent conversations as first-class panes alongside terminals
- Agent conversations have distinct backend plumbing: LLM streaming, tool call approval flow, message history — not reducible to a Canvas widget
- claude-agent-sdk-rs is already available in reference/claude-agent-sdk-rs as a submodule

## How it works

AgentView calls use_ts_widget("agent", config) with session and model info. The Rust side creates or resumes an agent session via claude-agent-sdk-rs, connected through a WebSocket endpoint. Messages flow bidirectionally: user input from the TS widget goes to the agent session, agent responses stream back as chunks. Tool calls are rendered as cards with approve/deny buttons. The TS widget renders three sections: a scrollable message list (flex 1), a streaming indicator, and a fixed input bar at bottom. Messages are typed as user bubbles (right-aligned, primary accent), assistant bubbles (left-aligned, surface-container-low background), or tool call cards (full-width, surface-container-high with action buttons).

## Files affected

| File | Change |
|------|--------|
| `ts/desktop/package.json` | Add markdown-it dependency |
| `ts/desktop/src/widgets/agent.ts` | New file: agent chat widget |
| `ts/desktop/src/index.ts` | Side-effect import to register agent widget |
| `apps/desktop/Cargo.toml` | Add claude-agent-sdk path dependency under server feature |
| `apps/desktop/src/server/agent.rs` | New file: agent session management and WebSocket endpoint |
| `apps/desktop/src/server/mod.rs` | Export agent module |
| `apps/desktop/src/terminal/view.rs` or new `agent_view.rs` | Replace stub AgentView with real implementation |

## Task List

### Task 1: Add markdown-it dependency

Run `pnpm add markdown-it` and `pnpm add -D @types/markdown-it` in the `ts/desktop` directory.

### Task 2: Create agent widget TypeScript implementation

Create `ts/desktop/src/widgets/agent.ts` implementing the Widget interface. The mount function creates a container div (flex column, full height). Inside: a messages div (flex 1, overflow-y auto, padding 16px, display flex, flex-direction column, gap 12px) and an input bar div (display flex, padding 8px, gap 8px, border-top 1px solid --color-border, background --surface-container-low). The input bar contains a textarea (flex 1, resize none, background --surface-lowest, border 1px solid --color-border, color --color-on-surface, padding 8px, border-radius 4px, font-size 14px, rows 1) and a send button (gradient-primary background, padding 8px 16px, border-radius 4px, font-weight 600). On send click or Ctrl+Enter in textarea, call dx.send with type "user_message" and content from textarea value, then clear the textarea. Store a reference to the current assistant bubble div (null when no streaming). The update function handles message types via a type field: "assistant_chunk" appends text to the current assistant bubble (create a new left-aligned bubble if none active, render content via markdown-it), "assistant_done" finalizes the current bubble and sets current to null, "tool_call" creates a tool card div (full width, background --surface-container-high, padding 12px, border-radius 8px, ghost-border) showing tool name bold, arguments as formatted text, and two buttons — Approve (gradient-primary) sending dx.send with type "tool_decision", callId from data, and decision "approve", and Deny (bg-red-500/20 text) sending decision "deny". "error" creates a red-tinted bubble. Auto-scroll the messages div to bottom on new content unless user has scrolled up. Import registerWidget and register as "agent".

### Task 3: Register agent widget in exports

Edit `ts/desktop/src/index.ts`. Add a side-effect import for the agent widget file. Run `just ts-build`.

### Task 4: Add claude-agent-sdk dependency

Edit `apps/desktop/Cargo.toml`. Read `reference/claude-agent-sdk-rs/Cargo.toml` to determine the exact crate name and available features. Add it as a path dependency pointing to reference/claude-agent-sdk-rs, gated on the server feature flag.

### Task 5: Create agent session server endpoint

Create `apps/desktop/src/server/agent.rs`. Define an AgentSession struct held in a global DashMap keyed by session_id (same pattern as terminal PtySession in session.rs). The session wraps a claude-agent-sdk client instance. Define message enums: ClientToAgent with variants UserMessage (content: String) and ToolDecision (call_id: String, decision: String); AgentToClient with variants AssistantChunk (text: String), AssistantDone, ToolCall (call_id: String, name: String, arguments: String), and Error (message: String). Derive Serialize and Deserialize on both. Create a WebSocket endpoint at /api/agent/ws with query params session_id and model. The handler gets or creates an AgentSession, then loops: receive ClientToAgent messages from the socket and forward to the SDK, receive streaming events from the SDK and send as AgentToClient messages. Register the endpoint in the server router. Export the module from server/mod.rs.

### Task 6: Implement AgentView Rust component

Replace the stub AgentView. The component takes agent_id (String), session_id (String), and model (String) as props. Call use_ts_widget("agent", serde_json::json!({ "sessionId": session_id, "model": model })). Connect to the agent WebSocket endpoint using the same WebSocket pattern as TerminalView (use_websocket to /api/agent/ws with session_id and model as query params). Spawn a bidirectional forwarding loop using tokio::select: messages from widget.recv (user_message, tool_decision) are serialized and sent to the WebSocket; messages from the WebSocket (assistant_chunk, tool_call, assistant_done, error) are forwarded to the widget via widget.send_update. The component renders a div with id element_id and class "w-full h-full".

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.
