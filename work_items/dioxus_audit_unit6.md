# Unit 6: mcp_panel.rs — use_resource → use_loader

## Violation

Rule: "No use_resource in fullstack apps" — `use_resource` on line 16 loads MCP server names. Must use `use_loader`.

File: `crates/lx-desktop/src/pages/tools/mcp_panel.rs`, line 16.

## Current Code

```rust
#[component]
pub fn McpPanel() -> Element {
  let mut servers = use_resource(|| async { load_mcp_servers().await });

  rsx! {
    // ... header with refresh button that calls servers.restart() ...
    match &*servers.value().read() {
        Some(names) => rsx! { /* render servers */ },
        None => rsx! { /* loading text */ },
    }
  }
}
```

## Context

`load_mcp_servers()` (lines 3-12 of same file) reads `.mcp.json` from the local filesystem using `tokio::fs::read_to_string`. Same situation as Unit 5 — this is a local async operation, not a server function.

The component also has a refresh button (line 27) that calls `servers.restart()`. If converting to `use_loader`, the refresh mechanism needs a replacement.

## Required Changes

### Option A: If use_loader works with local async

Replace lines 15-52 (entire McpPanel body):

```rust
#[component]
pub fn McpPanel() -> Element {
  let servers = use_loader(|| async { load_mcp_servers().await })?;

  rsx! {
    div { class: "flex items-center gap-3 mb-4",
      div { class: "h-px flex-1 bg-[var(--outline-variant)]" }
      span { class: "text-xs uppercase tracking-wider font-semibold text-[var(--on-surface)]",
        "MCP_EXTENSIONS"
      }
      div { class: "h-px flex-1 bg-[var(--outline-variant)]" }
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
- Remove the `match` on `Some`/`None` — `use_loader` suspends until data is ready, so `servers` is always populated
- Remove the refresh button — `use_loader` doesn't have `.restart()`. If refresh is needed, use a signal to trigger re-render or accept the loss of manual refresh.

### Option B: If use_loader requires a server function

Keep `use_resource` (this is a desktop-only local filesystem read, not a server function). The `use_resource` usage is valid for client-only reactive computations per the audit exception: "use_resource is only acceptable for client-only reactive computations that genuinely cannot use use_loader."

In this case, reading `.mcp.json` from the local filesystem IS a client-only operation that cannot be a server function. Mark as **exception — no change needed**.

### Determine which option

Check whether `load_mcp_servers` could be converted to a server function. If `McpPanel` runs on a desktop app (not fullstack with a server), then local file reads cannot use server functions, and `use_resource` is the correct pattern. If the desktop app IS fullstack (has a server component), then `load_mcp_servers` should become a `#[get]` server function and `use_loader` should be used.

Check `crates/lx-desktop/Cargo.toml` for `dioxus` features — if `fullstack` is enabled, Option A applies. If only `desktop` is enabled, Option B (exception) applies.

## Files Modified

- `crates/lx-desktop/src/pages/tools/mcp_panel.rs` — entire `McpPanel` component

## Verification

Run `just diagnose` and confirm no errors in `mcp_panel.rs`.
