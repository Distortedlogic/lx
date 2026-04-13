# Unit 04: CLI and Desktop Fallback Cleanup

## Goal

Remove the verified CLI and desktop fallbacks that currently turn manifest discovery, upload I/O, and HTTP error decoding failures into silent `Option`/default behavior. User-facing paths should preserve the actual reason for failure instead of quietly continuing with missing data.

## Preconditions

- Unit 01 should be complete first.

## Verified Findings

- `crates/lx-cli/src/manifest.rs`
  - `find_workspace_root(...)` uses `.ok()?` for manifest reads and TOML parsing, which silently skips malformed workspace manifests
  - `load_dep_dirs_filtered(...)` uses `.unwrap_or_default()` for `.dev-deps`
  - `load_dep_dirs_filtered(...)` and related helpers use `filter_map(|e| e.ok())`, silently dropping unreadable directory entries
- `crates/lx-desktop/src/components/drag_drop.rs`
  - `save_dropped_file(...) -> Option<String>` hides directory-creation, base64-decode, and file-write failures
  - `read_dropped_files()` uses `serde_json::from_str(...).unwrap_or_default()`, which turns invalid browser payloads into an empty file list
- `crates/lx-desktop/src/api/client.rs`
  - on non-success HTTP responses, `resp.json().await.ok()` silently drops non-JSON bodies and replaces them with `"Unknown error"`

## Files to Modify

- `crates/lx-cli/src/manifest.rs`
- `crates/lx-cli/src/check.rs`
- `crates/lx-cli/src/install.rs`
- `crates/lx-cli/src/listing.rs`
- `crates/lx-cli/src/main.rs`
- `crates/lx-cli/src/testing.rs`
- `crates/lx-desktop/src/components/drag_drop.rs`
- `crates/lx-desktop/src/api/client.rs`

## Steps

### Step 1: Add detailed manifest/workspace discovery helpers

In `crates/lx-cli/src/manifest.rs`, keep the existing lightweight helpers only where callers truly need best-effort behavior, but add explicit detailed variants for user-facing CLI commands:

- a detailed workspace-root loader that distinguishes:
  - no `lx.toml` found
  - unreadable `lx.toml`
  - invalid TOML
  - manifest exists but is not a workspace root
- a detailed dependency-directory loader that does not silently skip malformed marker files or unreadable directory entries

The public CLI commands in:

- `crates/lx-cli/src/check.rs`
- `crates/lx-cli/src/install.rs`
- `crates/lx-cli/src/listing.rs`
- `crates/lx-cli/src/main.rs`
- `crates/lx-cli/src/testing.rs`

must switch to the detailed helpers anywhere the command currently prints a generic “failed to load workspace” or quietly continues after a bad manifest.

Do not change every helper to `Result` if the caller is intentionally best-effort. Only the user-facing command paths need the stronger error surface.

### Step 2: Stop hiding upload failures behind `Option<String>`

In `crates/lx-desktop/src/components/drag_drop.rs`:

- replace `save_dropped_file(...) -> Option<String>` with a typed or stringly `Result<String, ...>`
- preserve the exact failure cause for:
  - cache-dir creation
  - base64 decode
  - file write
- update `build_markdown_links(...)` so a failed upload includes the real failure message in the generated placeholder text instead of the generic `upload failed`

Keep the current successful markdown output format unchanged for both image and non-image drops.

### Step 3: Treat malformed browser drop payloads as errors, not empty lists

Still in `crates/lx-desktop/src/components/drag_drop.rs`:

- remove `serde_json::from_str(&unescaped).unwrap_or_default()`
- if the browser payload cannot be decoded, surface that failure in a visible way for the caller instead of returning `vec![]`

The fix can be a `Result<Vec<DroppedFile>, String>` return shape or a local error log plus empty list only if the caller explicitly handles the error. Do not silently swallow malformed payloads at the parsing boundary.

### Step 4: Preserve raw HTTP error bodies in the desktop API client

In `crates/lx-desktop/src/api/client.rs`:

- stop using `resp.json().await.ok()` on error responses
- read the response body text first
- if the body is JSON and has an `error` field, keep the current structured `ApiError::Http` behavior
- if the body is non-JSON, preserve the raw text in the `ApiError::Http` payload and message instead of replacing it with `"Unknown error"`

Keep success-path decoding behavior unchanged.

## Verification

1. Run `just test`.
2. Run `just rust-diagnose`.
3. Run `rg -n 'ok\\(\\)\\?|unwrap_or_default\\(|filter_map\\(\\|e\\| e\\.ok\\(\\)\\)' crates/lx-cli/src/manifest.rs crates/lx-desktop/src/components/drag_drop.rs crates/lx-desktop/src/api/client.rs -g '*.rs'`.
4. Manually verify one malformed `lx.toml` path and one non-JSON HTTP error response now produce a concrete error message instead of a generic fallback.
5. Manually verify one failed dropped-file write produces a markdown placeholder that includes the actual failure reason.

