---
unit: 1
title: Full Audit Sweep — lx-mobile
type: read-only verification
depends_on: none
---

## Goal

Verify all 9 applicable dioxus-audit.md rules against `crates/lx-mobile/` (11 files, ~370 lines). Produce a pass/fail verdict per rule with evidence.

## Scope

Target: `crates/lx-mobile/src/` — all .rs files.

## Rules to Check

### 1. dioxus-primitives (no hand-rolled duplicates)

Run: `rg '#\[component\]' --type rust crates/lx-mobile/`

Components defined: `PulseIndicator`, `Status`, `Events`, `Approvals`, `ConfirmPrompt`, `ChoosePrompt`, `AskPrompt`, `MobileShell`, `BottomNav`, `NavTab`, `App`.

For each, determine if a dioxus-primitive covers the same use case. `PulseIndicator` is a domain-specific status indicator (enum-driven color/animation/label) — no primitive equivalent. `BottomNav`/`NavTab` are 3 hardcoded route links — not a generic `Navbar`. The prompt components are domain-specific approval UI. **Expected: clean.**

### 2. Dioxus Re-exports (no `use dioxus_*`)

Run: `rg 'use dioxus_' --type rust crates/lx-mobile/`

All files use `dioxus::prelude::*` or `dioxus::fullstack::*`. **Expected: clean.**

### 3. Logging (built-in Dioxus logger only)

Run: `rg 'tracing::|log::|env_logger|tracing_subscriber' --type rust crates/lx-mobile/`

No logging calls in the crate. **Expected: clean.**

### 4. Component Design

#### 4a. Wrapper forwarding

For each `#[component]`: does the body render exactly one child component and forward all props? `MobileShell` wraps `Outlet` + `BottomNav` with layout div — not a pure passthrough. **Expected: clean.**

#### 4b. Single-use

For each `#[component]`, count call sites:
- `PulseIndicator` — 1 call site (`status.rs:26`). Inlining would make status.rs ~55 lines (under 300). **FINDING: inline.**
- `BottomNav` — 1 call site (`shell.rs:16`). But it is a layout module boundary component — inlining degrades structure. **No action.**
- `NavTab` — 3 call sites. Not single-use.
- `ConfirmPrompt`, `ChoosePrompt`, `AskPrompt` — 1 each, but they are match arms. Inlining creates a ~90-line match block. **No action.**
- `MobileShell`, `App`, `Status`, `Events`, `Approvals` — page/layout/app components. Structural, not inlineable.

### 5. Frontend Structure

- Router: `routes.rs` — own file. Clean.
- Layout: `layout/shell.rs` — own file. Clean.
- App: `app.rs` — own file. Clean.

### 6. Hooks (use_action/use_loader correctness)

Run: `rg 'use_action|use_future|use_loader|spawn' --type rust crates/lx-mobile/`

- `status.rs`: `use_loader(get_run_status)` — correct for data loading.
- `approvals.rs`: `use_loader(get_pending_prompts)` + `use_action(respond_to_prompt)` — correct.
- `events.rs`: `use_future` for WebSocket receive loop — correct (persistent connection, not a server fn call).

**Expected: clean.**

### 7. Hook Misuse

Run: `rg 'use_resource|use_effect' --type rust crates/lx-mobile/`

Neither hook is used anywhere. **Expected: clean.**

### 8. RSX Class Attributes

Run: `rg 'class:\s*"[^"]*\{[^}]+\}[^"]*"' --type rust crates/lx-mobile/`

`components.rs` uses `class: "{color}"` and `class: "{animation}"` — each interpolated value is its own attribute, separate from static classes. **Expected: clean.**

### 9. Ecosystem Utilization

Run: `rg 'web_sys|gloo' --type rust crates/lx-mobile/`

No raw web_sys or gloo usage. WebSocket uses `dioxus::fullstack::use_websocket`. **Expected: clean.**

## Expected Output

| Rule | Verdict |
|------|---------|
| 1. dioxus-primitives | Clean |
| 2. Dioxus re-exports | Clean |
| 3. Logging | Clean |
| 4a. Wrapper forwarding | Clean |
| 4b. Single-use | **FINDING: PulseIndicator** |
| 5. Frontend structure | Clean |
| 6. Hooks | Clean |
| 7. Hook misuse | Clean |
| 8. RSX class attributes | Clean |
| 9. Ecosystem utilization | Clean |

Single actionable finding: inline `PulseIndicator` into `status.rs` (see Unit 2).
