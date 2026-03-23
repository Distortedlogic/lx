# Goal

Wire the PaneToolbar address bar, back, forward, and refresh buttons to the BrowserView CDP session so the browser pane is fully navigable. Update the address bar to reflect the actual URL after each navigation (click-through, address bar entry, back/forward). Normalize bare domains to https:// URLs.

# Why

- The PaneToolbar renders back/forward/refresh buttons and an address bar for Browser panes, but `on_navigate` is `None` — clicking the buttons and pressing Enter in the address bar does nothing
- The address bar shows the initial URL ("about:blank") and never updates when the headless browser navigates
- Users cannot browse the web because there is no way to tell the CDP session to navigate
- Bare domain input like "google.com" needs to become "https://google.com" or Chrome will fail to navigate

# What changes

**Convert `render_pane_item` to `PaneItem` component (terminals.rs):** The current `render_pane_item` is a plain function, which cannot use hooks. Convert it to a `#[component] fn PaneItem` so we can create hooks inside it. Create a `tokio::sync::mpsc::unbounded_channel::<String>()` for navigation commands (event-driven, zero-latency, no polling). The `UnboundedSender` goes to the `on_navigate` EventHandler. The `UnboundedReceiver` is stored in an `Arc<std::sync::Mutex<Option<...>>>` so it can be passed as a component prop (BrowserView `.take()`s it once on mount). Create a `Signal<String>` for `current_url` (BrowserView writes after navigation, PaneToolbar reads to sync address bar). Pass both to BrowserView and PaneToolbar.

**Add navigation channel and current_url to BrowserView (view.rs):** Add `nav_rx: Arc<std::sync::Mutex<Option<UnboundedReceiver<String>>>>` and `current_url: Signal<String>` props. In the existing `use_future`, take the receiver once at the start of the async block. Add a third `tokio::select!` branch: `cmd = rx.recv() => { ... }` — this wakes instantly when PaneToolbar sends a command (no polling). Match on the command: "back" → `go_back()`, "forward" → `go_forward()`, "refresh" → `reload()`, anything else → normalize URL and `navigate()`. After successful navigate, write `final_url` to `current_url`. Also update the initial navigation block and the `Some("navigate")` widget recv arm to write `final_url` to `current_url`.

**Add current_url prop to PaneToolbar (toolbar.rs):** Add `current_url: ReadOnlySignal<String>` prop. Add a `use_effect` that syncs `current_url` to `url_input` when it changes.

**Thread props through render_pane_view (terminals.rs):** Add the nav_rx and current_url parameters. Pass to BrowserView in the Browser match arm. Other arms ignore them.

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

- `crates/lx-desktop/src/pages/terminals.rs` — convert `render_pane_item` to `PaneItem` component, create channel + signal, wire `on_navigate`, thread through `render_pane_view`
- `crates/lx-desktop/src/terminal/view.rs` — add `nav_rx` and `current_url` props to `BrowserView`, add channel recv branch to select loop
- `crates/lx-desktop/src/terminal/toolbar.rs` — add `current_url` prop to `PaneToolbar`, add `use_effect` to sync address bar

# Task List

### Task 1: Convert render_pane_item to PaneItem component with navigation channel

**Subject:** Convert render_pane_item to a Dioxus component with mpsc channel for navigation

**Description:** In `crates/lx-desktop/src/pages/terminals.rs`:

(A) Add imports at the top: `use std::sync::{Arc, Mutex};` and `use tokio::sync::mpsc;`.

(B) Rename `render_pane_item` to `PaneItem` and add `#[component]` attribute. Change the signature from `fn render_pane_item(mut tabs_state: Signal<TabsState<DesktopPane>>, pane: &DesktopPane, rect: &Rect, focused_pane_id: &Option<String>) -> Element` to `fn PaneItem(tabs_state: Signal<TabsState<DesktopPane>>, pane: DesktopPane, rect: Rect, focused_pane_id: Option<String>) -> Element`. Props are now owned. `let pid = pane.pane_id().to_owned()` becomes `let pid = pane.pane_id().to_string()`. Keep the `pane_toolbar` and `pane_view` clones since pane is consumed by the rsx block.

(C) Inside PaneItem, after the existing variable declarations and before the `rsx!` block, create the channel and signals:
```
let (nav_tx, nav_rx) = use_hook(|| {
    let (tx, rx) = mpsc::unbounded_channel::<String>();
    (tx, Arc::new(Mutex::new(Some(rx))))
});
let mut current_url: Signal<String> = use_signal(|| {
    match &pane { DesktopPane::Browser { url, .. } => url.clone(), _ => String::new() }
});
```
`use_hook` stores the `(UnboundedSender, Arc<Mutex<Option<UnboundedReceiver>>>)` pair. `UnboundedSender` is Clone, `Arc<Mutex<...>>` is Clone, so the tuple is Clone — satisfying `use_hook`'s requirement. The channel is created once on first render. The Arc+Mutex wrapper lets BrowserView take the receiver once via `.lock().unwrap().take()`.

(D) Change `on_navigate: None::<EventHandler<String>>,` to:
```
on_navigate: if matches!(&pane_toolbar, DesktopPane::Browser { .. }) {
    let tx = nav_tx.clone();
    Some(EventHandler::new(move |cmd: String| { let _ = tx.send(cmd); }))
} else {
    None
},
```

(E) Add `current_url: current_url.into(),` as a new prop in the PaneToolbar block (after on_navigate).

(F) Change `{render_pane_view(&pane_view)}` to `{render_pane_view(&pane_view, nav_rx.clone(), current_url)}`.

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

(H) Update `render_pane_view` signature to `fn render_pane_view(pane: &DesktopPane, nav_rx: Arc<Mutex<Option<mpsc::UnboundedReceiver<String>>>>, current_url: Signal<String>) -> Element`. In the `DesktopPane::Browser` match arm, add `nav_rx` and `current_url` props to BrowserView. All other match arms are unchanged.

**ActiveForm:** Converting render_pane_item to PaneItem component with mpsc navigation channel

### Task 2: Add channel recv branch and current_url to BrowserView

**Subject:** Wire mpsc receiver to CDP session navigation in BrowserView's select loop

**Description:** In `crates/lx-desktop/src/terminal/view.rs`:

(A) Add imports: `use std::sync::{Arc, Mutex};` and `use tokio::sync::mpsc;`.

(B) Change the BrowserView component signature to: `pub fn BrowserView(browser_id: String, url: String, devtools: bool, nav_rx: Arc<Mutex<Option<mpsc::UnboundedReceiver<String>>>>, current_url: Signal<String>)`. The `Arc<Mutex<Option<...>>>` is Clone + PartialEq (via Arc pointer equality) — wait, Arc does NOT implement PartialEq by default. Add a newtype wrapper:
```
#[derive(Clone)]
pub struct NavReceiver(pub Arc<Mutex<Option<mpsc::UnboundedReceiver<String>>>>);
impl PartialEq for NavReceiver { fn eq(&self, other: &Self) -> bool { Arc::ptr_eq(&self.0, &other.0) } }
```
Use `nav_rx: NavReceiver` as the prop type instead of the raw Arc. Update terminals.rs to wrap: `nav_rx: NavReceiver(nav_rx.clone())` in render_pane_view's Browser arm, and the `render_pane_view` signature to use `NavReceiver`.

(C) At the start of the `use_future` async block (after `let session = ...` succeeds), take the receiver: `let mut nav_rx = nav_rx.0.lock().unwrap().take();`.

(D) Update the initial navigation block. Change:
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
        Ok((final_url, _)) => current_url.set(final_url),
        Err(e) => { error!("browser navigate failed: {e}"); return; }
    }
}
```

(E) Add a third branch to the `tokio::select!` loop. This branch only exists if the receiver was successfully taken (it should always succeed since only one BrowserView instance takes it). Use an `if let` to guard:
```
cmd = async { match nav_rx.as_mut() { Some(rx) => rx.recv().await, None => std::future::pending().await } } => {
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
                    Ok((final_url, _)) => current_url.set(final_url),
                    Err(e) => { error!("browser navigate failed: {e}"); }
                }
            }
        }
    }
}
```
The `async { match nav_rx.as_mut() ... }` pattern handles the case where the receiver wasn't taken (returns `pending()` which never resolves, so the branch is effectively disabled). When the receiver IS present, `rx.recv()` wakes instantly when PaneToolbar sends a command — zero polling.

(F) Update the existing `Some("navigate")` match arm to write final_url to current_url:
```
Some("navigate") => {
    if let Some(nav_url) = msg["url"].as_str() {
        match session.navigate(nav_url).await {
            Ok((final_url, _)) => current_url.set(final_url),
            Err(e) => { error!("browser navigate failed: {e}"); break; }
        }
    }
}
```

**ActiveForm:** Adding mpsc channel recv branch and current_url updates to BrowserView

### Task 3: Add current_url prop to PaneToolbar

**Subject:** Sync PaneToolbar address bar with actual browser URL

**Description:** In `crates/lx-desktop/src/terminal/toolbar.rs`:

(A) Add `current_url: ReadOnlySignal<String>` to the PaneToolbar component signature (after `on_navigate`).

(B) After `let mut url_input = use_signal(|| initial_url);` (line 21), add:
```
use_effect(move || {
    let val = current_url.read().clone();
    if !val.is_empty() {
        url_input.set(val);
    }
});
```

This fires whenever `current_url` changes. When BrowserView navigates and writes the final URL to `current_url`, this effect updates the address bar input to match.

**ActiveForm:** Syncing PaneToolbar address bar with browser current URL

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.
