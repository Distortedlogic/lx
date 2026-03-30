# WU-15: Icon Size Convention Sweep

## Fixes
- Fix 1: Establish convention -- `text-sm` (14px) default for standard UI icons
- Fix 2: `text-xs` (12px) for inline/compact contexts (filter chips, nested sub-items, priority badges in cards)
- Fix 3: `text-base` (16px) for sidebar nav, card action buttons, section header icons
- Fix 4: `text-lg` (18px) for dialog close buttons, icon picker current icon, agent detail icon
- Fix 5: `text-xl` (20px) for onboarding step hero icons only
- Fix 6: `text-3xl` / `text-4xl` for empty-state illustrations only
- Fix 7: Add missing `text-base` size class to bare `material-symbols-outlined` in `pages/company_skills/mod.rs` line 33
- Fix 8: Add missing `text-base` size class to bare `material-symbols-outlined` in `pages/inbox/mod.rs` line 79
- Fix 9: Add missing `text-base` size class to bare `material-symbols-outlined` in `pages/settings/instance_experimental.rs` line 31
- Fix 10: Add missing `text-base` size class to bare `material-symbols-outlined` in `pages/settings/instance_general.rs` line 11
- Fix 11: Add missing `text-base` size class to bare `material-symbols-outlined` in `pages/plugins/mod.rs` lines 25, 54
- Fix 12: Add missing `text-base` size class to bare `material-symbols-outlined` in `pages/plugins/plugin_settings.rs` line 42
- Fix 13: Add missing `text-base` size class to bare `material-symbols-outlined` in `pages/settings/instance_settings.rs` line 42, `pages/settings/company_settings.rs` line 16

## Files Modified
- `crates/lx-desktop/src/pages/company_skills/mod.rs` (138 lines)
- `crates/lx-desktop/src/pages/inbox/mod.rs` (189 lines)
- `crates/lx-desktop/src/pages/settings/instance_experimental.rs` (54 lines)
- `crates/lx-desktop/src/pages/settings/instance_general.rs` (42 lines)
- `crates/lx-desktop/src/pages/plugins/mod.rs` (160 lines)
- `crates/lx-desktop/src/pages/plugins/plugin_settings.rs` (182 lines)
- `crates/lx-desktop/src/pages/settings/instance_settings.rs` (122 lines)
- `crates/lx-desktop/src/pages/settings/company_settings.rs` (149 lines)

## Preconditions
- All files listed above exist and contain `material-symbols-outlined` spans without an explicit text size class
- The pattern in every case is: `"material-symbols-outlined text-[var(--outline)]"` where the color class is present but no size class like `text-sm`, `text-base`, `text-lg`, etc.
- These are all page/section header icons (paired with `<h1>` or `<h2>` headings), so `text-base` is the correct convention size

## Convention Reference

This is the size convention. All existing usages that already follow this convention do NOT need changes. Only the bare (no-size-class) instances need fixing.

| Size | Class | Usage | Pixel size |
|------|-------|-------|-----------|
| Default | `text-sm` | Toolbar buttons, list item icons, tree expand/collapse, close buttons in small contexts, status/priority icons | 14px |
| Inline | `text-xs` | Filter chip close, nested sub-item icons (transcript group sub-items), kanban card priority badges, property dropdown check | 12px |
| Section | `text-base` | Sidebar nav icons, page/section header icons, card action buttons (delete/power), metric card icons, inbox row icons | 16px |
| Dialog | `text-lg` | Dialog close buttons, icon picker current value, agent detail header icon, command palette result icons | 18px |
| Hero | `text-xl` | Onboarding step hero icons only | 20px |
| Empty state | `text-3xl` / `text-4xl` | Empty state illustrations, loading spinners in full-page contexts | 30px/36px |

## Full Audit of Every `material-symbols-outlined` Usage

### Already correct -- no changes needed

These files have explicit size classes matching the convention:

| File | Line | Current class | Status |
|------|------|--------------|--------|
| `components/filter_bar.rs` | 33 | `text-xs` | Correct (inline close) |
| `components/toast_viewport.rs` | 56 | `text-sm` | Correct (close button) |
| `components/command_palette.rs` | 85, 114 | `text-lg` | Correct (palette result icons) |
| `components/markdown_editor.rs` | 198 | `text-sm` | Correct (toolbar) |
| `components/empty_state.rs` | 8 | `text-4xl` | Correct (empty state) |
| `components/ui/select.rs` | 82, 215 | `text-sm` | Correct (dropdown) |
| `components/metric_card.rs` | 32 | `text-base` | Correct (section) |
| `components/inline_entity_selector.rs` | 67 | `text-sm` | Correct (list item) |
| `components/onboarding/wizard.rs` | 117 | `text-lg` | Correct (dialog close) |
| `components/onboarding/wizard.rs` | 167, 194, 214 | `text-sm` | Correct (toolbar) |
| `components/onboarding/step_agent.rs` | 21 | `text-xl` | Correct (hero) |
| `components/onboarding/step_company.rs` | 9 | `text-xl` | Correct (hero) |
| `components/onboarding/step_launch.rs` | 9 | `text-xl` | Correct (hero) |
| `components/onboarding/step_launch.rs` | 40, 49 | `text-base` | Correct (section) |
| `components/onboarding/step_task.rs` | 9 | `text-xl` | Correct (hero) |
| `components/file_tree.rs` | 148, 160, 215 | `text-sm` | Correct (tree items) |
| `components/company_switcher.rs` | 45 | `text-sm` | Correct (list item) |
| `components/company_switcher.rs` | 80, 86 | `text-base` | Correct (action buttons) |
| `layout/menu_bar.rs` | 130, 137, 144 | `text-sm` | Correct (toolbar) |
| `layout/sidebar.rs` | 103 | `text-base` | Correct (nav) |
| `layout/properties_panel.rs` | 29 | `text-sm` | Correct (close) |
| `pages/company_import.rs` | 27 | `text-base` | Correct (section header) |
| `pages/company_import.rs` | 53 | `text-lg` | Correct (dialog icon) |
| `pages/company_import.rs` | 63 | `text-4xl` | Correct (empty state) |
| `pages/company_import.rs` | 120 | `text-4xl` | Correct (loading spinner) |
| `pages/company_export.rs` | 20 | `text-base` | Correct (section header) |
| `pages/company_export.rs` | 27, 78 | `text-sm` | Correct (list item) |
| `pages/company_export.rs` | 97 | `text-4xl` | Correct (empty state) |
| `pages/agents/new_agent.rs` | 29 | `text-lg` | Correct (dialog close) |
| `pages/agents/transcript_groups.rs` | 38, 43, 77, 82, 125 | `text-sm` | Correct (list items) |
| `pages/agents/transcript_groups.rs` | 53, 92 | `text-xs` | Correct (nested sub-items) |
| `pages/agents/config_form.rs` | 37, 74 | `text-sm` | Correct (list items) |
| `pages/agents/config_form.rs` | 93 | `text-xs` | Correct (inline tag) |
| `pages/agents/icon_picker.rs` | 48 | `text-lg` | Correct (current icon display) |
| `pages/agents/icon_picker.rs` | 77 | `text-base` | Correct (picker grid) |
| `pages/agents/detail.rs` | 181 | `text-lg` | Correct (agent header icon) |
| `pages/agents/transcript_blocks.rs` | all | `text-sm` | Correct (list items) |
| `pages/settings/mod.rs` | 58 | `text-base` | Correct (section) |
| `pages/settings/mod.rs` | 88 | `text-base` | Correct (section) |
| `pages/company_skills/skill_tree.rs` | 54, 65, 101 | `text-sm` | Correct (tree items) |
| `pages/company_skills/mod.rs` | 46, 51 | `text-sm` | Correct (toolbar/list) |
| `pages/company_skills/mod.rs` | 74 | `text-3xl` | Correct (empty state) |
| `pages/company_skills/mod.rs` | 131 | `text-4xl` | Correct (empty state) |
| `pages/tools/mcp_panel.rs` | 29 | `text-sm` | Correct (toolbar) |
| `pages/issues/new_issue.rs` | 91 | `text-lg` | Correct (dialog close) |
| `pages/issues/list.rs` | 28, 33, 128, 138 | `text-sm` | Correct (list items) |
| `pages/issues/kanban.rs` | 124 | `text-sm` | Correct (column header) |
| `pages/issues/kanban.rs` | 201, 235 | `text-xs` | Correct (card badge) |
| `pages/issues/workspace_card.rs` | 19 | `text-sm` | Correct (card item) |
| `pages/issues/properties.rs` | 91 | `text-sm` | Correct (list item) |
| `pages/issues/properties.rs` | 108 | `text-xs` | Correct (dropdown item) |
| `pages/issues/documents.rs` | 35 | `text-sm` | Correct (list item) |
| `pages/issues/documents.rs` | 42 | `text-xs` | Correct (inline sub-item) |
| `pages/settings/env_vars.rs` | 38 | `text-sm` | Correct (action button) |
| `pages/goals/tree.rs` | 48 | `text-sm` | Correct (tree expand) |
| `pages/approvals/list.rs` | 101 | `text-3xl` | Correct (empty state) |
| `pages/approvals/card.rs` | 29 | `text-base` | Correct (section) |
| `pages/approvals/card.rs` | 37 | `text-sm` | Correct (status) |
| `pages/approvals/detail.rs` | 56 | `text-base` | Correct (section) |
| `pages/approvals/detail.rs` | 69, 83 | `text-sm` | Correct (status) |
| `pages/inbox/inbox_rows.rs` | 26, 57, 74, 125 | `text-base` | Correct (row icons) |
| `pages/inbox/mod.rs` | 99 | `text-4xl` | Correct (empty state) |
| `pages/routines/list.rs` | 189 | `text-base` | Correct (section) |
| `pages/plugins/plugin_settings.rs` | 39, 108, 124, 166 | `text-base` | Correct (section) |
| `pages/plugins/mod.rs` | 33 | `text-sm` | Correct (toolbar button) |
| `pages/plugins/mod.rs` | 40 | `text-base` | Correct (warning icon) |
| `pages/plugins/mod.rs` | 64 | `text-4xl` | Correct (empty state) |
| `pages/plugins/plugin_page.rs` | 14 | `text-sm` | Correct (back nav) |
| `pages/plugins/plugin_card.rs` | 63 | `text-sm` | Correct (warning badge) |
| `pages/plugins/plugin_card.rs` | 89, 95 | `text-base` | Correct (action buttons) |
| `pages/companies/mod.rs` | 16 | `text-sm` | Correct (toolbar) |
| `pages/companies/mod.rs` | 22 | `text-4xl` | Correct (empty state) |
| `pages/companies/company_card.rs` | 64, 69, 93, 97, 101, 112 | `text-sm` | Correct (card detail items) |
| `pages/costs/budget_card.rs` | 22, 88 | `text-sm` | Correct (card items) |
| `pages/costs/accounting_card.rs` | 47 | `text-base` | Correct (section) |
| `pages/org/tree_view.rs` | 65 | `text-xs` | Correct (tree expand) |
| `pages/settings/instance_settings.rs` | 78 | `text-4xl` | Correct (empty state) |

### Needs fixing -- bare `material-symbols-outlined` without size class

| File | Line | Current | Fix to | Context |
|------|------|---------|--------|---------|
| `pages/company_skills/mod.rs` | 33 | `"material-symbols-outlined text-[var(--outline)]"` | `"material-symbols-outlined text-base text-[var(--outline)]"` | Page header icon next to "Skills" h1 |
| `pages/inbox/mod.rs` | 79 | `"material-symbols-outlined text-[var(--outline)]"` | `"material-symbols-outlined text-base text-[var(--outline)]"` | Page header icon next to "Inbox" h1 |
| `pages/settings/instance_experimental.rs` | 31 | `"material-symbols-outlined text-[var(--outline)]"` | `"material-symbols-outlined text-base text-[var(--outline)]"` | Page header icon next to "Experimental" h1 |
| `pages/settings/instance_general.rs` | 11 | `"material-symbols-outlined text-[var(--outline)]"` | `"material-symbols-outlined text-base text-[var(--outline)]"` | Page header icon next to "General" h1 |
| `pages/plugins/mod.rs` | 25 | `"material-symbols-outlined text-[var(--outline)]"` | `"material-symbols-outlined text-base text-[var(--outline)]"` | Page header icon next to "Plugin Manager" h1 |
| `pages/plugins/mod.rs` | 54 | `"material-symbols-outlined text-[var(--outline)]"` | `"material-symbols-outlined text-base text-[var(--outline)]"` | Section header icon next to "Installed Plugins" h2 |
| `pages/plugins/plugin_settings.rs` | 42 | `"material-symbols-outlined text-[var(--outline)]"` | `"material-symbols-outlined text-base text-[var(--outline)]"` | Page header icon next to plugin display name h1 |
| `pages/settings/instance_settings.rs` | 42 | `"material-symbols-outlined text-[var(--outline)]"` | `"material-symbols-outlined text-base text-[var(--outline)]"` | Page header icon next to "Scheduler Heartbeats" h1 |
| `pages/settings/company_settings.rs` | 16 | `"material-symbols-outlined text-[var(--outline)]"` | `"material-symbols-outlined text-base text-[var(--outline)]"` | Page header icon next to "Company Settings" h1 |

## Steps

### Step 1: Fix `pages/company_skills/mod.rs` line 33
- Open `crates/lx-desktop/src/pages/company_skills/mod.rs`
- At line 33, find:
```
            span { class: "material-symbols-outlined text-[var(--outline)]",
```
- Replace with:
```
            span { class: "material-symbols-outlined text-base text-[var(--outline)]",
```
- Why: page header icon beside "Skills" h1 should be `text-base` per convention

### Step 2: Fix `pages/inbox/mod.rs` line 79
- Open `crates/lx-desktop/src/pages/inbox/mod.rs`
- At line 79, find:
```
        span { class: "material-symbols-outlined text-[var(--outline)]", "inbox" }
```
- Replace with:
```
        span { class: "material-symbols-outlined text-base text-[var(--outline)]", "inbox" }
```
- Why: page header icon beside "Inbox" h1 should be `text-base` per convention

### Step 3: Fix `pages/settings/instance_experimental.rs` line 31
- Open `crates/lx-desktop/src/pages/settings/instance_experimental.rs`
- At line 31, find:
```
          span { class: "material-symbols-outlined text-[var(--outline)]",
```
- Replace with:
```
          span { class: "material-symbols-outlined text-base text-[var(--outline)]",
```
- Why: page header icon beside "Experimental" h1 should be `text-base`

### Step 4: Fix `pages/settings/instance_general.rs` line 11
- Open `crates/lx-desktop/src/pages/settings/instance_general.rs`
- At line 11, find:
```
          span { class: "material-symbols-outlined text-[var(--outline)]",
```
- Replace with:
```
          span { class: "material-symbols-outlined text-base text-[var(--outline)]",
```
- Why: page header icon beside "General" h1 should be `text-base`

### Step 5: Fix `pages/plugins/mod.rs` lines 25 and 54
- Open `crates/lx-desktop/src/pages/plugins/mod.rs`
- At line 25, find:
```
          span { class: "material-symbols-outlined text-[var(--outline)]",
```
- Replace with:
```
          span { class: "material-symbols-outlined text-base text-[var(--outline)]",
```
- At line 54, find:
```
          span { class: "material-symbols-outlined text-[var(--outline)]",
```
- Replace with:
```
          span { class: "material-symbols-outlined text-base text-[var(--outline)]",
```
- Why: page header and section header icons should be `text-base`
- NOTE: There are two occurrences of the identical string in this file. Use line numbers to identify which is which -- line 25 is the page header, line 54 is the section header. Both get the same fix.

### Step 6: Fix `pages/plugins/plugin_settings.rs` line 42
- Open `crates/lx-desktop/src/pages/plugins/plugin_settings.rs`
- At line 42, find:
```
          span { class: "material-symbols-outlined text-[var(--outline)]",
```
- Replace with:
```
          span { class: "material-symbols-outlined text-base text-[var(--outline)]",
```
- Why: page header icon beside plugin name h1 should be `text-base`

### Step 7: Fix `pages/settings/instance_settings.rs` line 42
- Open `crates/lx-desktop/src/pages/settings/instance_settings.rs`
- At line 42, find:
```
          span { class: "material-symbols-outlined text-[var(--outline)]",
```
- Replace with:
```
          span { class: "material-symbols-outlined text-base text-[var(--outline)]",
```
- Why: page header icon beside "Scheduler Heartbeats" h1 should be `text-base`

### Step 8: Fix `pages/settings/company_settings.rs` line 16
- Open `crates/lx-desktop/src/pages/settings/company_settings.rs`
- At line 16, find:
```
        span { class: "material-symbols-outlined text-[var(--outline)]", "settings" }
```
- Replace with:
```
        span { class: "material-symbols-outlined text-base text-[var(--outline)]", "settings" }
```
- Why: page header icon beside "Company Settings" h1 should be `text-base`

## File Size Check
- All 8 files: each gains 0 net lines (only adding `text-base ` within an existing string literal). All remain under 300 lines.
  - `pages/company_skills/mod.rs`: 138 lines, unchanged
  - `pages/inbox/mod.rs`: 189 lines, unchanged
  - `pages/settings/instance_experimental.rs`: 54 lines, unchanged
  - `pages/settings/instance_general.rs`: 42 lines, unchanged
  - `pages/plugins/mod.rs`: 160 lines, unchanged
  - `pages/plugins/plugin_settings.rs`: 182 lines, unchanged
  - `pages/settings/instance_settings.rs`: 122 lines, unchanged
  - `pages/settings/company_settings.rs`: 149 lines, unchanged

## Verification
- Run `just diagnose` to confirm compilation (these are string-only changes, no Rust logic affected)
- Launch the desktop app and visually check each affected page:
  1. Skills page (sidebar) -- "widgets" icon next to "Skills" heading should be 16px, not the browser-default 24px
  2. Inbox page -- "inbox" icon next to "Inbox" heading should be 16px
  3. Settings > Experimental -- "science" icon should be 16px
  4. Settings > General -- "tune" icon should be 16px
  5. Plugin Manager -- "extension" icon next to heading and section should be 16px
  6. Plugin settings detail page -- "extension" icon should be 16px
  7. Settings > Scheduler Heartbeats -- "settings" icon should be 16px
  8. Company Settings -- "settings" icon should be 16px
- All icons should now be visually consistent with other page header icons (e.g., company_import.rs line 27 which already uses `text-base`)
