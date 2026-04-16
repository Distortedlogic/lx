# Flow Editor Theme Alignment

## Goal

Replace the flow editor's hardcoded scene palette with graph-specific semantic theme tokens so the DAG editor inherits the desktop app's visual language while retaining a distinct graph-focused surface.

## Why

The current flow editor mixes app-level semantic colors with many literal hex and `rgba(...)` values in the scene renderer. That makes the editor look visually detached from the rest of the desktop shell and makes future tuning expensive because graph colors are scattered through Rust render code instead of centralized in the theme.

## Changes

- Add graph semantic tokens to `crates/lx-desktop/tailwind.css` for:
  - canvas background, tint, and grid
  - overlay surfaces and overlay text
  - node surfaces, node headers, node borders, and selected state
  - edge colors and edge labels
  - port badge colors for input and output
  - diagnostic and status accents for error, warning, info, and selection
- Derive those tokens from the existing desktop theme instead of introducing an unrelated palette.
- Replace hardcoded scene colors in `crates/lx-desktop/src/pages/flows/workspace.rs` with `var(--graph-...)` references.
- Keep the current interaction model and layout; this work is theme alignment and editor configuration cleanup, not a structural UI redesign.

## Files Affected

- `crates/lx-desktop/tailwind.css`
- `crates/lx-desktop/src/pages/flows/workspace.rs`
- `crates/lx-desktop/assets/tailwind.css`

## Task List

1. Define graph semantic tokens in `crates/lx-desktop/tailwind.css` under both dark and light themes.
2. Cover every graph-scene literal color currently used by `FlowWorkspace`, `FlowEditorCanvas`, `PortBadge`, `node_style`, and validation rows.
3. Update `workspace.rs` so editor-specific buttons, palette overlay, canvas badges, grid, edges, nodes, ports, empty-state surfaces, fit-view controls, and diagnostic rows consume the new tokens.
4. Preserve the graph editor's visual hierarchy:
   - canvas darker than shell panels
   - nodes elevated above canvas
   - selected state stronger than default state
   - input and output ports visually distinct
   - error and warning states obvious without clashing with the shell theme
5. Regenerate `crates/lx-desktop/assets/tailwind.css` from the root Tailwind entrypoint.
6. Run desktop diagnostics and verify there are no remaining scene-level hardcoded colors in `workspace.rs` except where geometry-only inline styles still need literals for measurements.

## Verification

- `pnpm exec tailwindcss -i crates/lx-desktop/tailwind.css -o crates/lx-desktop/assets/tailwind.css`
- `cargo test -p lx-desktop --no-run`
- `rg -n "#|rgba\\(|rgb\\(" crates/lx-desktop/src/pages/flows/workspace.rs`
