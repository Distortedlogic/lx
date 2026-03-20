# Goal

Add `pkg/connectors/cdp` — a Chrome DevTools Protocol client for browser automation. Agents can navigate URLs, take screenshots, extract DOM content, click elements, type text, and evaluate JavaScript. Built on `std/ws` (WebSocket) + `std/json`. Pure lx, no Rust changes.

**Depends on: STD_WS work item must be completed first.**

# Why

- Devin's differentiator is autonomous browsing — navigate, screenshot, analyze, act. No other agentic workflow language has browser automation as a composable primitive.
- CDP is the standard protocol for browser control. Every browser automation tool (Puppeteer, Playwright, Selenium 4) uses it under the hood. CDP is JSON-RPC over WebSocket — with `std/ws`, the entire protocol client can be written in lx.
- Agents need browser access for: documentation lookup, web scraping, testing web UIs, filling forms, visual verification via screenshots.

# What Changes

**New file `pkg/connectors/cdp.lx`:** CDP client with connection management, page navigation, DOM interaction, and screenshot capture.

Core API:
- `cdp.launch opts` — launch Chrome with `--remote-debugging-port`, connect via WebSocket
- `cdp.connect url` — connect to existing Chrome DevTools WebSocket
- `cdp.navigate conn url` — navigate to URL, wait for load
- `cdp.screenshot conn opts` — capture page screenshot as base64
- `cdp.evaluate conn expr` — evaluate JavaScript expression, return result
- `cdp.click conn selector` — click a DOM element by CSS selector
- `cdp.type conn selector text` — type text into an input element
- `cdp.wait_for conn selector timeout_ms` — wait for element to appear
- `cdp.content conn` — get page HTML content
- `cdp.close conn` — close connection

# Files Affected

- `pkg/connectors/cdp.lx` — New file
- `tests/106_cdp.lx` — New test file

# Task List

### Task 1: Create pkg/connectors/cdp.lx with CDP protocol client

**Subject:** Create CDP client with connection, navigation, and DOM interaction

**Description:** Create `pkg/connectors/cdp.lx`:

```
-- CDP client -- Chrome DevTools Protocol over WebSocket.
-- Requires std/ws. Requires Chrome/Chromium installed with --remote-debugging-port.

use std/ws
use std/json
use std/time

msg_id = 0

next_id = () {
  msg_id <- msg_id + 1
  msg_id
}

send_cmd = (conn method params) {
  id = next_id ()
  msg = json.encode {id: id  method: method  params: params ?? {}}
  ws.send conn msg ^
  recv_response conn id
}

recv_response = (conn expected_id) {
  raw = ws.recv_json conn ^
  raw.id == expected_id ? raw : {
    -- Skip events, keep reading until we get our response
    recv_response conn expected_id
  }
}

+launch = (opts) {
  port = opts.port ?? 9222
  chrome = opts.chrome ?? "google-chrome-stable"
  headless = opts.headless ?? true

  headless_flag = headless ? "--headless=new" : ""
  $^{chrome} {headless_flag} --remote-debugging-port={port} --no-first-run --no-default-browser-check --disable-gpu &

  -- Wait for Chrome to start
  time.sleep 1000

  -- Get WebSocket URL from /json/version endpoint
  version = std/http.get "http://localhost:{port}/json/version" ^
  ws_url = version.body.webSocketDebuggerUrl

  conn = ws.connect ws_url ^
  {..conn  port: port  __cdp: true}
}

+connect = (url) {
  conn = ws.connect url ^
  {..conn  __cdp: true}
}

+navigate = (conn url) {
  send_cmd conn "Page.navigate" {url: url} ^
  -- Wait for load
  time.sleep 500
  -- Could use Page.loadEventFired for proper wait
  Ok ()
}

+screenshot = (conn opts) {
  format = opts.format ?? "png"
  quality = opts.quality ?? 80
  result = send_cmd conn "Page.captureScreenshot" {
    format: format
    quality: quality
  } ^
  result.result.data
}

+evaluate = (conn expression) {
  result = send_cmd conn "Runtime.evaluate" {
    expression: expression
    returnByValue: true
  } ^
  result.result.result.value ?? result.result.result
}

+click = (conn selector) {
  -- Find element via DOM query
  doc = send_cmd conn "DOM.getDocument" {} ^
  node = send_cmd conn "DOM.querySelector" {
    nodeId: doc.result.root.nodeId
    selector: selector
  } ^
  node_id = node.result.nodeId
  node_id == 0 ? (Err "element not found: {selector}") : {
    -- Get element box model for click coordinates
    box = send_cmd conn "DOM.getBoxModel" {nodeId: node_id} ^
    content = box.result.model.content
    x = (content.[0] + content.[2]) / 2
    y = (content.[1] + content.[5]) / 2

    send_cmd conn "Input.dispatchMouseEvent" {type: "mousePressed"  x: x  y: y  button: "left"  clickCount: 1} ^
    send_cmd conn "Input.dispatchMouseEvent" {type: "mouseReleased"  x: x  y: y  button: "left"  clickCount: 1} ^
    Ok ()
  }
}

+type_text = (conn selector text) {
  -- Focus the element first
  cdp.click conn selector ^
  -- Type each character
  text | chars | each (ch) {
    send_cmd conn "Input.dispatchKeyEvent" {type: "keyDown"  text: ch} ^
    send_cmd conn "Input.dispatchKeyEvent" {type: "keyUp"  text: ch} ^
  }
  Ok ()
}

+wait_for = (conn selector timeout_ms) {
  deadline = time.now().ms + timeout_ms
  poll = () {
    doc = send_cmd conn "DOM.getDocument" {} ^
    node = send_cmd conn "DOM.querySelector" {
      nodeId: doc.result.root.nodeId
      selector: selector
    } ^
    node.result.nodeId != 0 ? (Ok node.result.nodeId) : {
      time.now().ms > deadline ?
        (Err "timeout waiting for {selector}") :
        { time.sleep 100; poll () }
    }
  }
  poll ()
}

+content = (conn) {
  doc = send_cmd conn "DOM.getDocument" {} ^
  html = send_cmd conn "DOM.getOuterHTML" {
    nodeId: doc.result.root.nodeId
  } ^
  html.result.outerHTML
}

+close = (conn) {
  ws.close conn ^
  conn.port ? {
    Ok port -> { $kill -9 $(lsof -ti :{port}) 2>/dev/null; Ok () }
    _ -> Ok ()
  }
}
```

This is a first-pass implementation. The CDP protocol has many more methods — this covers the core browser automation primitives. The `send_cmd`/`recv_response` pattern handles CDP's JSON-RPC-over-WebSocket protocol. Event handling (skipping events while waiting for responses) is simplified — a production version would use a message queue.

Adjust based on actual CDP response shapes and lx parser constraints (e.g., `? { }` match block gotcha from GOTCHAS.md).

**ActiveForm:** Creating CDP client with browser automation primitives

---

### Task 2: Write tests for pkg/connectors/cdp

**Subject:** Write tests that verify CDP module loads and functions exist

**Description:** Create `tests/106_cdp.lx`. Since CDP requires Chrome installed and running, tests verify the module loads and provide graceful skip:

```
use pkg/connectors/cdp

-- Verify API surface exists
assert (type_of cdp.launch == "Fn") "launch exists"
assert (type_of cdp.connect == "Fn") "connect exists"
assert (type_of cdp.navigate == "Fn") "navigate exists"
assert (type_of cdp.screenshot == "Fn") "screenshot exists"
assert (type_of cdp.evaluate == "Fn") "evaluate exists"
assert (type_of cdp.click == "Fn") "click exists"
assert (type_of cdp.type_text == "Fn") "type_text exists"
assert (type_of cdp.wait_for == "Fn") "wait_for exists"
assert (type_of cdp.content == "Fn") "content exists"
assert (type_of cdp.close == "Fn") "close exists"

log.info "106_cdp: all passed"
```

Integration tests with actual Chrome should go in `flows/tests/` where MCP servers and external dependencies are expected.

Run `just test` to verify.

**ActiveForm:** Writing tests for CDP package

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
mcp__workflow__load_work_item({ path: "work_items/PKG_CDP.md" })
```

Then call `next_task` to begin.
