# Agent-Driven Pane Spawn Requests

## Goal

Extend the existing terminal spawn request protocol to support spawning any pane type. MCP tools, agents, and external automation can request the desktop app open a browser, editor, agent session, or canvas pane — not just terminals. The desktop client claims requests via WebSocket and creates the appropriate PaneNode variant.

## Why

- The existing TerminalSpawnRequest mechanism (POST to /api/terminal-requests, claimed via WebSocket) only supports terminals
- Agents need to open browsers to verify web app changes, editors to show code, canvas panes to display logs — all programmatically
- This is the foundation for autonomous agent workflows: agent spawns terminal → runs tests → opens browser to verify → shows diff in editor

## How it works

The existing flow is: external caller POSTs a TerminalSpawnRequest to /api/terminal-requests → server broadcasts via WebSocket → desktop client in terminal_ws_loop claims the request → creates a terminal tab. The new flow replaces TerminalSpawnRequest with PaneSpawnRequest which carries a kind field. The server broadcast and claim mechanism stays the same. The desktop client reads the kind and constructs the matching PaneNode variant.

## Files affected

| File | Change |
|------|--------|
| `crates/workflow-types/src/terminal.rs` | Add PaneSpawnRequest with kind field and variant-specific optional fields; keep TerminalSpawnRequest as alias |
| `apps/desktop/src/server/workflow/terminals.rs` | Update endpoint to accept PaneSpawnRequest |
| `apps/desktop/src/layout/shell.rs` | Update terminal_ws_loop to create PaneNode by kind |

## Task List

### Task 1: Update spawn request types

Edit `crates/workflow-types/src/terminal.rs`. Add a PaneSpawnKind enum with variants Terminal, Browser, Editor, Agent, Canvas — derive Clone, Debug, Serialize, Deserialize, PartialEq. Add a PaneSpawnRequest struct with fields: id (String), kind (PaneSpawnKind), title (Option of String), command (Option of String), working_directory (Option of String), url (Option of String), file_path (Option of String), language (Option of String), model (Option of String), widget_type (Option of String), config (Option of serde_json::Value), env (HashMap of String to String). Derive Clone, Debug, Serialize, Deserialize. Add a From impl converting TerminalSpawnRequest to PaneSpawnRequest (set kind to Terminal, map command, working_directory, id, env fields, set the rest to None). Update TerminalWsServerMsg::NewRequest to carry PaneSpawnRequest instead of TerminalRequestEntry (or whichever type it currently carries). Keep the existing TerminalSpawnRequest struct unchanged for backward compatibility.

### Task 2: Update server endpoint

Edit `apps/desktop/src/server/workflow/terminals.rs`. Update the POST handler to accept PaneSpawnRequest as the JSON body. If deserialization fails (old-format TerminalSpawnRequest), try deserializing as TerminalSpawnRequest and convert via the From impl. Update the storage (however pending requests are stored) to use PaneSpawnRequest. Update the GET handler response type. Update the WebSocket broadcast to send PaneSpawnRequest in the NewRequest message.

### Task 3: Update desktop claim handler

Edit `apps/desktop/src/layout/shell.rs`. In the terminal_ws_loop function, update the message handling for NewRequest. After claiming a request, match on the PaneSpawnRequest kind field to construct the appropriate PaneNode: Terminal creates PaneNode::Terminal with working_dir from working_directory (defaulting to ".") and command; Browser creates PaneNode::Browser with url (defaulting to "about:blank"); Editor creates PaneNode::Editor with file_path (defaulting to empty string) and language; Agent creates PaneNode::Agent with a new UUID session_id and model (defaulting to "claude-sonnet-4-6"); Canvas creates PaneNode::Canvas with widget_type (defaulting to "markdown") and config (defaulting to empty object). Use the request title if provided, otherwise generate one from the kind name. Call add_tab with the constructed PaneNode.

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.
