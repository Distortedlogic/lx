# Goal

Wire the PaneToolbar address bar, back, forward, and refresh buttons to the BrowserView CDP session so the browser pane is fully navigable. Update the address bar to reflect the actual URL after each navigation (click-through, address bar entry, back/forward). Normalize bare domains to https:// URLs.

# Why

- The PaneToolbar renders back/forward/refresh buttons and an address bar for Browser panes, but `on_navigate` is `None` — clicking the buttons and pressing Enter in the address bar does nothing
- The address bar shows the initial URL ("about:blank") and never updates when the headless browser navigates
- Users cannot browse the web because there is no way to tell the CDP session to navigate
- Bare domain input like "google.com" needs to become "https://google.com" or Chrome will fail to navigate

# What changes

**Convert `render_pane_item` to `PaneItem` component (terminals.rs):** The current `render_pane_item` is a plain function, which cannot use hooks. Convert it to a `#[component] fn PaneItem`. Create a `tokio::sync::mpsc::unbounded_channel::<String>()` for navigation commands and a `Signal<String>` for `current_url`. Provide both via `provide_context` using a `BrowserNavCtx` struct — this avoids threading props through `render_pane_view` (6 of 7 match arms would ignore them) and avoids needing a PartialEq newtype wrapper for the receiver. Wire `on_navigate` for Browser panes to send commands through the channel sender.

**Add channel recv branch to BrowserView (view.rs):** BrowserView calls `use_context::<BrowserNavCtx>()` to get the receiver and current_url signal. Takes the receiver once at the start of the `use_future` async block. Adds a third `tokio::select!` branch: `cmd = rx.recv()` — wakes instantly when PaneToolbar sends a command, zero polling. Dispatches the command using the session already in scope. Updates `current_url` after every navigation.

**Add current_url sync to PaneToolbar (toolbar.rs):** Add `current_url: ReadOnlySignal<String>` prop (this IS reactive display state — a prop is appropriate). Add a `use_effect` that syncs it to the address bar input.

# How it works

1. User types "google.com" in address bar, presses Enter
2. PaneToolbar calls `on_navigate` with "google.com"
3. `on_navigate` handler calls `tx.send("google.com")` on the mpsc sender
4. BrowserView's `rx.recv()` branch in `tokio::select!` wakes instantly
5. Normalizes to "https://google.com", calls `session.navigate("https://google.com")` using the session already in scope
6. `navigate` returns `("https://www.google.com/", "Google")` — writes "https://www.google.com/" to `current_url` signal
7. PaneToolbar's `use_effect` fires, updates `url_input` to "https://www.google.com/"
8. Address bar shows the actual URL
9. The 500ms screenshot branch captures the Google homepage and renders it on the canvas

Back/forward/refresh follow the same path but with command strings "back"/"forward"/"refresh" instead of URLs.

# Files affected

- `crates/lx-desktop/src/pages/terminals.rs` — convert `render_pane_item` to `PaneItem` component, add `BrowserNavCtx` struct, `provide_context`, wire `on_navigate`
- `crates/lx-desktop/src/terminal/view.rs` — `use_context` in `BrowserView`, add channel recv branch to select loop, update `current_url` after navigation
- `crates/lx-desktop/src/terminal/toolbar.rs` — add `current_url` prop to `PaneToolbar`, add `use_effect` to sync address bar

# Task List

### Task 1: Convert render_pane_item to PaneItem component with BrowserNavCtx

**Subject:** Convert render_pane_item to a Dioxus component with context-based navigation channel

**Description:** In `crates/lx-desktop/src/pages/terminals.rs`:

(A) Add imports at the top: `use std::sync::{Arc, Mutex};` and `use tokio::sync::mpsc;`.

(B) `BrowserNavCtx` is defined in `crates/lx-desktop/src/terminal/view.rs` (see Task 2), not here. Import it: `use crate::terminal::view::BrowserNavCtx;`.

(C) Rename `render_pane_item` to `PaneItem` and add `#[component]` attribute. Change the signature from `fn render_pane_item(mut tabs_state: Signal<TabsState<DesktopPane>>, pane: &DesktopPane, rect: &Rect, focused_pane_id: &Option<String>) -> Element` to `fn PaneItem(tabs_state: Signal<TabsState<DesktopPane>>, pane: DesktopPane, rect: Rect, focused_pane_id: Option<String>) -> Element`. Props are now owned. `let pid = pane.pane_id().to_owned()` becomes `let pid = pane.pane_id().to_string()`. Keep the `pane_toolbar` and `pane_view` clones since pane is consumed by the rsx block.

(D) Inside PaneItem, after the existing variable declarations and before the `rsx!` block, create the channel, signal, and provide context:
```
let current_url: Signal<String> = use_signal(|| {
    match &pane { DesktopPane::Browser { url, .. } => url.clone(), _ => String::new() }
});
let nav_ctx = use_hook(|| {
    let (tx, rx) = mpsc::unbounded_channel::<String>();
    BrowserNavCtx { tx, rx: Arc::new(Mutex::new(Some(rx))), current_url }
});
provide_context(nav_ctx.clone());
```
`use_hook` stores the `BrowserNavCtx` (Clone satisfied). The channel is created once on first render. `current_url` signal is captured by value (Signal is Copy).

(E) Change `on_navigate: None::<EventHandler<String>>,` to:
```
on_navigate: if matches!(&pane_toolbar, DesktopPane::Browser { .. }) {
    let tx = nav_ctx.tx.clone();
    Some(EventHandler::new(move |cmd: String| { let _ = tx.send(cmd); }))
} else {
    None
},
```

(F) Add `current_url: current_url.into(),` as a new prop in the PaneToolbar block (after on_navigate).

(G) Update the call site in `render_tab` (line 103). Change `{render_pane_item(tabs_state, pane, rect, focused_pane_id)}` to:
```
PaneItem {
    key: "{pane.pane_id()}",
    tabs_state,
    pane: pane.clone(),
    rect: *rect,
    focused_pane_id: focused_pane_id.clone(),
}
```

`render_pane_view` signature is UNCHANGED — no threading nav_rx or current_url through it. BrowserView gets what it needs from context.

**ActiveForm:** Converting render_pane_item to PaneItem component with context-based navigation

### Task 2: Add channel recv branch and current_url updates to BrowserView

**Subject:** Wire mpsc receiver from context to CDP session navigation in BrowserView's select loop

**Description:** In `crates/lx-desktop/src/terminal/view.rs`:

(A) Add imports at the top of view.rs: `use std::sync::{Arc, Mutex};` and `use tokio::sync::mpsc;`.

(B) Define the context struct in view.rs, before the BrowserView component:
```
#[derive(Clone)]
pub struct BrowserNavCtx {
    pub tx: mpsc::UnboundedSender<String>,
    pub rx: Arc<Mutex<Option<mpsc::UnboundedReceiver<String>>>>,
    pub current_url: Signal<String>,
}
```
This is `Clone` because `UnboundedSender` is Clone, `Arc` is Clone, `Signal` is Copy. No PartialEq needed — `provide_context` only requires `Clone + 'static`. Defined here (next to BrowserView, its primary consumer) so that `terminals.rs` imports it from `view.rs` — same direction as existing imports, no circular dependency.

(C) BrowserView component signature is UNCHANGED — no new props. BrowserView gets the navigation channel from context.

(D) Inside BrowserView, after `let bid_drop = browser_id.clone();` and before the `use_future` block, retrieve the context: `let nav_ctx: BrowserNavCtx = use_context();`.

(E) At the start of the `use_future` async block, after the session is created and initial navigation is done, take the receiver unconditionally:
```
let Some(mut nav_rx) = nav_ctx.rx.lock().unwrap().take() else { return; };
```
This runs once — the receiver is consumed by this future. If the future somehow re-runs and the receiver was already taken, it exits cleanly.

(F) Update the initial navigation block. Change:
```
if !url.is_empty()
    && url != "about:blank"
    && let Err(e) = session.navigate(&url).await
{
    error!("browser navigate failed: {e}");
    return;
}
```
to:
```
if !url.is_empty() && url != "about:blank" {
    match session.navigate(&url).await {
        Ok((final_url, _)) => nav_ctx.current_url.set(final_url),
        Err(e) => { error!("browser navigate failed: {e}"); return; }
    }
}
```

(G) Add a third branch to the `tokio::select!` loop:
```
cmd = nav_rx.recv() => {
    if let Some(cmd) = cmd {
        match cmd.as_str() {
            "back" => { if let Err(e) = session.go_back().await { error!("browser go_back failed: {e}"); } }
            "forward" => { if let Err(e) = session.go_forward().await { error!("browser go_forward failed: {e}"); } }
            "refresh" => { if let Err(e) = session.reload().await { error!("browser reload failed: {e}"); } }
            raw_url => {
                let url = if raw_url.starts_with("http://") || raw_url.starts_with("https://") {
                    raw_url.to_string()
                } else {
                    format!("https://{raw_url}")
                };
                match session.navigate(&url).await {
                    Ok((final_url, _)) => nav_ctx.current_url.set(final_url),
                    Err(e) => { error!("browser navigate failed: {e}"); }
                }
            }
        }
    }
}
```
`nav_rx.recv()` wakes instantly when PaneToolbar sends a command — no polling. The session is already in scope — no redundant lookup.

(H) Update the existing `Some("navigate")` match arm to write final_url to current_url:
```
Some("navigate") => {
    if let Some(nav_url) = msg["url"].as_str() {
        match session.navigate(nav_url).await {
            Ok((final_url, _)) => nav_ctx.current_url.set(final_url),
            Err(e) => { error!("browser navigate failed: {e}"); break; }
        }
    }
}
```

**ActiveForm:** Adding context-based channel recv branch and current_url updates to BrowserView

### Task 3: Add current_url prop to PaneToolbar

**Subject:** Sync PaneToolbar address bar with actual browser URL

**Description:** In `crates/lx-desktop/src/terminal/toolbar.rs`:

(A) Add `current_url: ReadOnlySignal<String>` to the PaneToolbar component signature (after `on_navigate`). This is reactive display state — a prop is appropriate here (the toolbar displays the URL, it doesn't own it).

(B) After `let mut url_input = use_signal(|| initial_url);` (line 21), add:
```
use_effect(move || {
    let val = current_url.read().clone();
    if !val.is_empty() {
        url_input.set(val);
    }
});
```

(C) In `crates/lx-desktop/src/pages/terminals.rs`, in the PaneToolbar block inside PaneItem, add the `current_url` prop. It was already specified in Task 1 step (F): `current_url: current_url.into(),`.

**ActiveForm:** Syncing PaneToolbar address bar with browser current URL

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.
