# Primitive Audit Unit 05: Identity Avatar Primitive Migration

## Goal

Replace the custom avatar rendering logic inside `crates/lx-desktop/src/components/identity.rs` with the shared `dioxus-primitives` avatar components while keeping the public `Identity` API and existing call sites unchanged.

## Why

The primitive-audit discovery pass found a live custom avatar implementation in `crates/lx-desktop/src/components/identity.rs`: it manually derives initials, conditionally renders an `img`, and renders a fallback badge when no avatar URL exists. The desktop source tree still uses `Identity` in:

- `crates/lx-desktop/src/components/comment_thread.rs`
- `crates/lx-desktop/src/pages/dashboard/active_agents_panel.rs`

The local primitive library at `../dioxus-common/crates/dioxus-primitives/src/avatar.rs` provides `Avatar`, `AvatarImage`, and `AvatarFallback`, so this is a verified duplicate of an available primitive rather than a speculative design preference.

The validated fix is to keep `Identity` as the app-level compound component that pairs avatar plus name text, but delegate the avatar behavior itself to `dioxus-primitives`. That removes the custom avatar implementation without forcing unrelated call-site churn.

## Changes

- Add `dioxus-primitives` as a dependency of `crates/lx-desktop`.
- Update `crates/lx-desktop/src/components/identity.rs` so the avatar circle is built from `Avatar`, `AvatarImage`, and `AvatarFallback` instead of a hand-rolled `img`/initials branch.
- Preserve the existing `IdentityProps`, size mapping, fallback initial derivation, text label rendering, and external class prop behavior.

## Files Affected

- `work_items/primitives_audit_unit_05_identity_avatar_primitive_migration.md`
- `crates/lx-desktop/Cargo.toml`
- `crates/lx-desktop/src/components/identity.rs`

## Task List

1. Add the local path dependency for `dioxus-primitives` to `crates/lx-desktop/Cargo.toml` with the desktop feature enabled.
2. Import `Avatar`, `AvatarFallback`, and `AvatarImage` into `crates/lx-desktop/src/components/identity.rs`.
3. Replace the custom avatar branch in `Identity` with primitive-based markup while preserving the existing wrapper layout, size classes, text classes, and `derive_initials` behavior.
4. Re-audit `Identity` to ensure it no longer performs custom avatar state/render branching outside the primitive composition.

## Verification

- `rg 'if let Some\\(ref url\\) = props\\.avatar_url|<img|img \\{' crates/lx-desktop/src/components/identity.rs`
- `rg 'Avatar(Image|Fallback)?' crates/lx-desktop/src/components/identity.rs`
- `just fmt`
- `just rust-diagnose`
