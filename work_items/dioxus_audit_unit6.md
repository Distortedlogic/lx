# Unit 6: mcp_panel.rs — use_resource → use_loader

## Violation

Rule: "No use_resource in fullstack apps" — `use_resource` on line 16 loads MCP server names.

File: `crates/lx-desktop/src/pages/tools/mcp_panel.rs`.

## Current Code

```rust
async fn load_mcp_servers() -> Vec<String> {
  let Ok(content) = tokio::fs::read_to_string(".mcp.json").await else {
    return Vec::new();
  };
  let json: serde_json::Value = match serde_json::from_str(&content) {
    Ok(v) => v,
    Err(_) => return Vec::new(),
  };
  json.get("mcpServers").and_then(|v| v.as_object()).map(|obj| obj.keys().cloned().collect()).unwrap_or_default()
}

#[component]
pub fn McpPanel() -> Element {
  let mut servers = use_resource(|| async { load_mcp_servers().await });
  // ... rsx with match &*servers.value().read() { Some(names) => ..., None => ... }
}
```

## Approach

`use_loader` requires `Future<Output = Result<T, E>>`. Change `load_mcp_servers` to return `Result<Vec<String>, std::io::Error>` so it satisfies use_loader's bounds directly. `Vec<String>` implements `PartialEq + Serialize + DeserializeOwned`.

The current refresh button (`servers.restart()`) is replaced by a signal-based trigger: a `refresh` counter signal that the loader subscribes to. Incrementing it forces the loader to re-evaluate.

## Required Changes

### Step 1: Change load_mcp_servers return type

Replace the entire `load_mcp_servers` function (lines 3-12):

Current:
```rust
async fn load_mcp_servers() -> Vec<String> {
  let Ok(content) = tokio::fs::read_to_string(".mcp.json").await else {
    return Vec::new();
  };
  let json: serde_json::Value = match serde_json::from_str(&content) {
    Ok(v) => v,
    Err(_) => return Vec::new(),
  };
  json.get("mcpServers").and_then(|v| v.as_object()).map(|obj| obj.keys().cloned().collect()).unwrap_or_default()
}
```

Replace with:
```rust
async fn load_mcp_servers() -> Result<Vec<String>, std::io::Error> {
  let content = tokio::fs::read_to_string(".mcp.json").await?;
  let json: serde_json::Value = serde_json::from_str(&content)
    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
  Ok(json.get("mcpServers").and_then(|v| v.as_object()).map(|obj| obj.keys().cloned().collect()).unwrap_or_default())
}
```

### Step 2: Replace McpPanel component body

Replace lines 14-52 (entire component) with:

```rust
#[component]
pub fn McpPanel() -> Element {
  let mut refresh = use_signal(|| 0u32);
  let servers = use_loader(move || {
    let _ = refresh();
    load_mcp_servers()
  })?;

  rsx! {
    div { class: "flex items-center gap-3 mb-4",
      div { class: "h-px flex-1 bg-[var(--outline-variant)]" }
      span { class: "text-xs uppercase tracking-wider font-semibold text-[var(--on-surface)]",
        "MCP_EXTENSIONS"
      }
      div { class: "h-px flex-1 bg-[var(--outline-variant)]" }
      button {
        class: "text-xs text-[var(--outline)] hover:text-[var(--primary)] transition-colors duration-150",
        onclick: move |_| refresh += 1,
        span { class: "material-symbols-outlined text-sm", "refresh" }
      }
    }
    div { class: "grid grid-cols-4 gap-3",
      for name in servers.read().iter() {
        div { class: "bg-[var(--surface-container-low)] border border-[var(--outline-variant)]/30 rounded-lg p-4 flex flex-col gap-2",
          span { class: "text-2xl text-[var(--primary)]", "\u{1F5C4}" }
          span { class: "text-xs font-semibold uppercase tracking-wider text-[var(--on-surface)]",
            "{name}"
          }
          span { class: "text-[10px] uppercase tracking-wider text-[var(--outline)]",
            "CONFIGURED"
          }
        }
      }
    }
  }
}
```

Changes:
- `use_resource` → `use_loader` with `?` for suspension
- Added `refresh` signal: the loader closure reads `refresh()` which subscribes it to the signal. The refresh button increments this signal, which triggers the loader to re-run.
- Removed `match &*servers.value().read() { Some(..) => ..., None => ... }` — `use_loader` suspends until data is ready, so `servers` is always populated. Use `servers.read().iter()` directly.
- The "Loading MCP servers..." text is removed because `use_loader`'s `?` suspends the component. The parent must have a `SuspenseBoundary` (or Dioxus's default suspense) to show a fallback while loading. If no `SuspenseBoundary` exists above `McpPanel`, add one at the call site wrapping `McpPanel {}`.

## Files Modified

- `crates/lx-desktop/src/pages/tools/mcp_panel.rs` — entire file

## Verification

Run `just diagnose` and confirm no errors in `mcp_panel.rs`.
