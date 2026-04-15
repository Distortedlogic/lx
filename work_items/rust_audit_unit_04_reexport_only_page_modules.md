# Rust Audit Unit 04: Re-export-Only Page Modules

## Goal

Remove the verified page modules that exist only to re-export settings pages.

## Why

`./rules/rust-audit.md` flags module files that contain only re-exports and no logic. `crates/lx-desktop/src/pages/company_settings.rs` and `crates/lx-desktop/src/pages/instance_settings.rs` are exact matches. The real page components already exist under `crates/lx-desktop/src/pages/settings/`, and `pages/settings/mod.rs` already re-exports them.

## Changes

- Delete the two re-export-only page files.
- Remove their module declarations from `crates/lx-desktop/src/pages/mod.rs`.
- Update `crates/lx-desktop/src/routes.rs` to import the same components from `crate::pages::settings`.

## Files Affected

- `crates/lx-desktop/src/pages/company_settings.rs`
- `crates/lx-desktop/src/pages/instance_settings.rs`
- `crates/lx-desktop/src/pages/mod.rs`
- `crates/lx-desktop/src/routes.rs`

## Task List

1. Delete the two re-export-only page files.
2. Remove their module declarations from `pages/mod.rs`.
3. Update `routes.rs` imports so the routes still resolve to the same settings page components through `crate::pages::settings`.
4. Run formatting and Rust diagnostics.

## Verification

- `rg -n 'pages::company_settings::CompanySettings|pages::instance_settings::InstanceSettings|use crate::pages::company_settings::CompanySettings|use crate::pages::instance_settings::InstanceSettings' crates/lx-desktop/src --type rust`
- `just fmt`
- `just rust-diagnose`
