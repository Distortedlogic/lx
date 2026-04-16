# Primitive Audit Unit 04: Unused Overlay And Navigation Wrapper Removal

## Goal

Remove unused `lx-desktop` UI wrapper modules that duplicate available `dioxus-primitives` concepts and currently have no call sites anywhere in the desktop application.

## Why

The current primitive-audit discovery pass found seven custom wrapper modules in `crates/lx-desktop/src/components/ui/` whose component names and behavior overlap directly with primitives available in the local `../dioxus-common/crates/dioxus-primitives` source tree:

- `dialog.rs`
- `popover.rs`
- `collapsible.rs`
- `tabs.rs`
- `dropdown_menu.rs`
- `scroll_area.rs`
- `sheet.rs`

Independent `rg` verification showed those wrappers are exported from `ui/mod.rs` but have zero imports and zero RSX call sites elsewhere in `crates/lx-desktop/src`. Because the wrappers are dead code, the best validated fix is deletion rather than introducing or preserving abstractions that can drift away from the shared primitive implementations.

The same verification pass also showed their dedicated Tailwind hooks are dead:

- `animate-dialog-overlay-in`
- `animate-dialog-overlay-out`
- `animate-dialog-content-in`
- `animate-dialog-content-out`
- `collapsible-open`
- `collapsible-closed`

Those selectors are referenced only by the wrapper modules being removed, so keeping them would leave unused animation and layout rules behind.

## Changes

- Delete the unused wrapper modules:
  - `crates/lx-desktop/src/components/ui/dialog.rs`
  - `crates/lx-desktop/src/components/ui/popover.rs`
  - `crates/lx-desktop/src/components/ui/collapsible.rs`
  - `crates/lx-desktop/src/components/ui/tabs.rs`
  - `crates/lx-desktop/src/components/ui/dropdown_menu.rs`
  - `crates/lx-desktop/src/components/ui/scroll_area.rs`
  - `crates/lx-desktop/src/components/ui/sheet.rs`
- Remove their module exports from `crates/lx-desktop/src/components/ui/mod.rs`.
- Remove the now-dead dialog animation and collapsible layout selectors from `crates/lx-desktop/src/tailwind.css`.

## Files Affected

- `work_items/primitives_audit_unit_04_unused_overlay_navigation_wrappers.md`
- `crates/lx-desktop/src/components/ui/mod.rs`
- `crates/lx-desktop/src/components/ui/dialog.rs`
- `crates/lx-desktop/src/components/ui/popover.rs`
- `crates/lx-desktop/src/components/ui/collapsible.rs`
- `crates/lx-desktop/src/components/ui/tabs.rs`
- `crates/lx-desktop/src/components/ui/dropdown_menu.rs`
- `crates/lx-desktop/src/components/ui/scroll_area.rs`
- `crates/lx-desktop/src/components/ui/sheet.rs`
- `crates/lx-desktop/src/tailwind.css`

## Task List

1. Re-verify that `Dialog`, `Popover`, `Collapsible`, `Tabs`, `DropdownMenu`, `ScrollArea`, and `Sheet` have no imports or RSX call sites outside their own module files.
2. Delete the seven unused wrapper module files listed above.
3. Remove `collapsible`, `dialog`, `dropdown_menu`, `popover`, `scroll_area`, `sheet`, and `tabs` from `crates/lx-desktop/src/components/ui/mod.rs`.
4. Remove the dead `.animate-dialog-*` and `.collapsible-*` selectors from `crates/lx-desktop/src/tailwind.css`.
5. Re-audit the source tree to confirm the deleted modules, exports, and Tailwind selectors are no longer referenced.

## Verification

- `rg '\\b(Dialog|DialogContent|DialogHeader|DialogFooter|DialogTitle|DialogDescription|Popover|PopoverTrigger|PopoverContent|Collapsible|CollapsibleTrigger|CollapsibleContent|Tabs|TabsList|TabsTrigger|TabsContent|DropdownMenu|DropdownMenuTrigger|DropdownMenuContent|DropdownMenuItem|DropdownMenuSeparator|DropdownMenuLabel|ScrollArea|SheetContent|SheetHeader|SheetFooter|SheetTitle|SheetDescription)\\s*\\{' crates/lx-desktop/src -g '!crates/lx-desktop/src/components/ui/*.rs'`
- `rg 'components::ui::(dialog|popover|collapsible|tabs|dropdown_menu|scroll_area|sheet)|ui::(dialog|popover|collapsible|tabs|dropdown_menu|scroll_area|sheet)' crates/lx-desktop/src`
- `rg 'animate-dialog-overlay-in|animate-dialog-overlay-out|animate-dialog-content-in|animate-dialog-content-out|collapsible-open|collapsible-closed' crates/lx-desktop/src`
- `just fmt`
- `just rust-diagnose`
