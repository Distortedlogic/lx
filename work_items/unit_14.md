# Unit 14: Company Management & Skills

## Scope

Port the company list page, company export/import pages, company switcher sidebar component, company skills page with file tree browser, and company pattern icon component from Paperclip React into Dioxus 0.7.3 components in lx-desktop.

## Paperclip Source Files

| Source | What it contains |
|--------|-----------------|
| `reference/paperclip/ui/src/pages/Companies.tsx` (297 lines) | Company list with inline rename, delete confirmation, stats row (agents/issues/budget/created), new company button |
| `reference/paperclip/ui/src/pages/CompanyExport.tsx` (1018 lines) | Export preview: file tree with checkboxes, YAML filtering, search, task pagination, preview pane with markdown/image rendering |
| `reference/paperclip/ui/src/pages/CompanyImport.tsx` (1354 lines) | Import wizard: file upload/URL/GitHub source, preview with action badges (create/update/skip), conflict resolution, adapter override, apply mutation |
| `reference/paperclip/ui/src/components/CompanySwitcher.tsx` (81 lines) | Dropdown menu listing companies with status dots, links to company settings and manage companies |
| `reference/paperclip/ui/src/pages/CompanySkills.tsx` (1170 lines) | Skills list, skill detail with file tree browser, markdown editor, file viewer, new skill form, project scan |
| `reference/paperclip/ui/src/components/PackageFileTree.tsx` (318 lines) | Reusable recursive file tree component with checkboxes, expand/collapse, file icons, frontmatter parsing |
| `reference/paperclip/ui/src/components/CompanyPatternIcon.tsx` (212 lines) | Procedural pattern icon generator using Bayer dithering, HSL color from company name hash |

## Target Directory Structure

```
crates/lx-desktop/src/
  components/
    mod.rs                   (new — component module root)
    company_switcher.rs      (new — dropdown company selector)
    company_pattern_icon.rs  (new — procedural icon component)
    file_tree.rs             (new — reusable file tree component)
  pages/
    companies/
      mod.rs            (new — company list page)
      company_card.rs   (new — individual company card with inline edit)
    company_export.rs   (new — export preview page)
    company_import.rs   (new — import wizard page)
    company_skills/
      mod.rs            (new — skills list + detail page)
      skill_tree.rs     (new — skill file tree browser)
      new_skill_form.rs (new — new skill creation form)
    mod.rs              (existing — add new modules)
  routes.rs             (existing — do NOT modify, Unit 3 owns it)
  lib.rs                (existing — add components module)
```

## Preconditions

- **Unit 1 is complete:** `src/components/mod.rs` already exists with `pub mod ui;` and other module declarations. `src/lib.rs` already contains `pub mod components;`.
- Unit 13 complete (pages/mod.rs has inbox module)
- `crates/lx-desktop/src/routes.rs` exists with `Route` enum including `Companies`, `CompanyExport`, `CompanyImport`, `CompanySkills` variants (from Unit 3, do NOT modify)
- `crates/lx-desktop/src/pages/mod.rs` exists
- `crates/lx-desktop/src/lib.rs` exists with module declarations

## Tasks

### Task 1: Edit `crates/lx-desktop/src/components/mod.rs` (already exists)

Add these module declarations to the existing `components/mod.rs`:

```rust
pub mod company_pattern_icon;
pub mod company_switcher;
pub mod file_tree;
```

### Task 2: Note on lib.rs

`pub mod components;` was already added to `lib.rs` by Unit 1. Do NOT re-add it.

### Task 3: Create `crates/lx-desktop/src/components/company_pattern_icon.rs`

Port `CompanyPatternIcon` from `CompanyPatternIcon.tsx`. The React version uses canvas to generate a Bayer-dithered pattern. In Dioxus, generate a simple deterministic visual using the company name hash to pick a background color, and display the first letter initial.

Reference: `CompanyPatternIcon.tsx` lines 1-212. Key functions: `hashString`, `mulberry32`, `hslToRgb`, `hexToHue`. The React version renders a canvas-based pattern as a data URL. The Dioxus version uses a simpler CSS-based approach since canvas is not available in Dioxus desktop.

```rust
use dioxus::prelude::*;

fn hash_string(value: &str) -> u32 {
    let mut hash: u32 = 2166136261;
    for byte in value.bytes() {
        hash ^= byte as u32;
        hash = hash.wrapping_mul(16777619);
    }
    hash
}

fn hash_to_hsl(hash: u32) -> (u16, u8, u8) {
    let hue = (hash % 360) as u16;
    let sat = 50 + (hash / 360 % 20) as u8;
    let light = 35 + (hash / 7200 % 15) as u8;
    (hue, sat, light)
}

#[component]
pub fn CompanyPatternIcon(
    company_name: String,
    brand_color: Option<String>,
    class: Option<String>,
) -> Element {
    let trimmed = company_name.trim();
    let initial = trimmed
        .chars()
        .next()
        .map(|c| c.to_uppercase().to_string())
        .unwrap_or_else(|| "?".to_string());

    let hash = hash_string(&trimmed.to_lowercase());
    let (hue, sat, light) = if let Some(ref color) = brand_color {
        if color.starts_with('#') && color.len() == 7 {
            let r = u8::from_str_radix(&color[1..3], 16).unwrap_or(100);
            let g = u8::from_str_radix(&color[3..5], 16).unwrap_or(100);
            let b = u8::from_str_radix(&color[5..7], 16).unwrap_or(200);
            let max = r.max(g).max(b);
            let min = r.min(g).min(b);
            let d = max - min;
            let h = if d == 0 {
                0u16
            } else if max == r {
                (60.0 * (((g as f32 - b as f32) / d as f32) % 6.0)) as u16
            } else if max == g {
                (60.0 * (((b as f32 - r as f32) / d as f32) + 2.0)) as u16
            } else {
                (60.0 * (((r as f32 - g as f32) / d as f32) + 4.0)) as u16
            };
            (h, 60, 40)
        } else {
            hash_to_hsl(hash)
        }
    } else {
        hash_to_hsl(hash)
    };

    let bg_style = format!("background: hsl({hue}, {sat}%, {light}%)");
    let extra_class = class.unwrap_or_default();

    rsx! {
        div {
            class: "relative flex items-center justify-center w-11 h-11 text-base font-semibold text-white overflow-hidden {extra_class}",
            style: "{bg_style}",
            span { class: "relative z-10 drop-shadow-[0_1px_2px_rgba(0,0,0,0.65)]",
                "{initial}"
            }
        }
    }
}
```

### Task 4: Create `crates/lx-desktop/src/components/company_switcher.rs`

Port `CompanySwitcher` from `CompanySwitcher.tsx` lines 1-81. Dropdown listing companies with status dot indicators, links to settings and manage companies.

```rust
use dioxus::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct CompanySwitcherEntry {
    pub id: String,
    pub name: String,
    pub status: String,
}

fn status_dot_color(status: &str) -> &'static str {
    match status {
        "active" => "bg-green-400",
        "paused" => "bg-yellow-400",
        "archived" => "bg-neutral-400",
        _ => "bg-green-400",
    }
}

#[component]
pub fn CompanySwitcher(
    companies: Vec<CompanySwitcherEntry>,
    selected_id: Option<String>,
    on_select: EventHandler<String>,
) -> Element {
    let mut open = use_signal(|| false);
    let selected = companies
        .iter()
        .find(|c| Some(&c.id) == selected_id.as_ref());
    let sidebar_companies: Vec<_> = companies
        .iter()
        .filter(|c| c.status != "archived")
        .collect();

    rsx! {
        div { class: "relative",
            button {
                class: "w-full flex items-center justify-between px-2 py-1.5 text-left hover:bg-[var(--surface-container)] rounded",
                onclick: move |_| {
                    let current = open();
                    open.set(!current);
                },
                div { class: "flex items-center gap-2 min-w-0",
                    if let Some(company) = selected {
                        span { class: "h-2 w-2 rounded-full shrink-0 {status_dot_color(&company.status)}" }
                    }
                    span { class: "text-sm font-medium truncate text-[var(--on-surface)]",
                        if let Some(company) = selected {
                            "{company.name}"
                        } else {
                            "Select company"
                        }
                    }
                }
                span { class: "material-symbols-outlined text-sm text-[var(--outline)]",
                    "unfold_more"
                }
            }
            if open() {
                div { class: "absolute left-0 top-full mt-1 w-[220px] rounded-md border border-[var(--outline-variant)] bg-[var(--surface-container-lowest)] shadow-lg z-50",
                    div { class: "px-3 py-1.5 text-xs font-semibold text-[var(--outline)] uppercase",
                        "Companies"
                    }
                    div { class: "border-t border-[var(--outline-variant)]" }
                    for company in sidebar_companies.iter() {
                        {
                            let id = company.id.clone();
                            let is_selected = Some(&company.id) == selected_id.as_ref();
                            let bg = if is_selected { " bg-[var(--surface-container)]" } else { "" };
                            rsx! {
                                button {
                                    class: "w-full flex items-center gap-2 px-3 py-1.5 text-sm text-left hover:bg-[var(--surface-container)]{bg}",
                                    onclick: move |_| {
                                        on_select.call(id.clone());
                                        open.set(false);
                                    },
                                    span { class: "h-2 w-2 rounded-full shrink-0 {status_dot_color(&company.status)}" }
                                    span { class: "truncate text-[var(--on-surface)]", "{company.name}" }
                                }
                            }
                        }
                    }
                    if sidebar_companies.is_empty() {
                        div { class: "px-3 py-1.5 text-sm text-[var(--outline)]",
                            "No companies"
                        }
                    }
                    div { class: "border-t border-[var(--outline-variant)]" }
                    button {
                        class: "w-full flex items-center gap-2 px-3 py-1.5 text-sm text-left hover:bg-[var(--surface-container)]",
                        onclick: move |_| open.set(false),
                        span { class: "material-symbols-outlined text-base", "settings" }
                        span { class: "text-[var(--on-surface)]", "Company Settings" }
                    }
                    button {
                        class: "w-full flex items-center gap-2 px-3 py-1.5 text-sm text-left hover:bg-[var(--surface-container)]",
                        onclick: move |_| open.set(false),
                        span { class: "material-symbols-outlined text-base", "add" }
                        span { class: "text-[var(--on-surface)]", "Manage Companies" }
                    }
                }
            }
        }
    }
}
```

### Task 5: Create `crates/lx-desktop/src/components/file_tree.rs`

Port `PackageFileTree` from `PackageFileTree.tsx` lines 1-318. Reusable recursive file tree with checkboxes, expand/collapse, file type icons.

```rust
use dioxus::prelude::*;
use std::collections::{BTreeSet, HashSet};

#[derive(Clone, Debug, PartialEq)]
pub struct FileTreeNode {
    pub name: String,
    pub path: String,
    pub kind: FileNodeKind,
    pub children: Vec<FileTreeNode>,
    pub action: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum FileNodeKind {
    Dir,
    File,
}

pub fn build_file_tree(
    files: &[String],
    action_map: Option<&std::collections::HashMap<String, String>>,
) -> Vec<FileTreeNode> {
    let mut root = FileTreeNode {
        name: String::new(),
        path: String::new(),
        kind: FileNodeKind::Dir,
        children: vec![],
        action: None,
    };

    for file_path in files {
        let segments: Vec<&str> = file_path.split('/').filter(|s| !s.is_empty()).collect();
        let mut current = &mut root;
        let mut current_path = String::new();

        for (i, segment) in segments.iter().enumerate() {
            if !current_path.is_empty() {
                current_path.push('/');
            }
            current_path.push_str(segment);
            let is_leaf = i == segments.len() - 1;

            let pos = current.children.iter().position(|c| c.name == *segment);
            if let Some(idx) = pos {
                current = &mut current.children[idx];
            } else {
                let node = FileTreeNode {
                    name: segment.to_string(),
                    path: current_path.clone(),
                    kind: if is_leaf {
                        FileNodeKind::File
                    } else {
                        FileNodeKind::Dir
                    },
                    children: vec![],
                    action: if is_leaf {
                        action_map.and_then(|m| m.get(file_path).cloned())
                    } else {
                        None
                    },
                };
                current.children.push(node);
                let last = current.children.len() - 1;
                current = &mut current.children[last];
            }
        }
    }

    fn sort_node(node: &mut FileTreeNode) {
        node.children.sort_by(|a, b| {
            if a.kind != b.kind {
                return if a.kind == FileNodeKind::File {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Greater
                };
            }
            a.name.cmp(&b.name)
        });
        for child in &mut node.children {
            sort_node(child);
        }
    }

    sort_node(&mut root);
    root.children
}

pub fn collect_file_paths(nodes: &[FileTreeNode]) -> HashSet<String> {
    let mut paths = HashSet::new();
    for node in nodes {
        if node.kind == FileNodeKind::File {
            paths.insert(node.path.clone());
        }
        paths.extend(collect_file_paths(&node.children));
    }
    paths
}

pub fn count_files(nodes: &[FileTreeNode]) -> usize {
    let mut count = 0;
    for node in nodes {
        if node.kind == FileNodeKind::File {
            count += 1;
        } else {
            count += count_files(&node.children);
        }
    }
    count
}

fn file_icon(name: &str) -> &'static str {
    if name.ends_with(".yaml") || name.ends_with(".yml") {
        "code"
    } else {
        "description"
    }
}

#[component]
pub fn FileTree(
    nodes: Vec<FileTreeNode>,
    selected_file: Option<String>,
    expanded_dirs: HashSet<String>,
    checked_files: Option<HashSet<String>>,
    on_toggle_dir: EventHandler<String>,
    on_select_file: EventHandler<String>,
    on_toggle_check: Option<EventHandler<(String, FileNodeKind)>>,
    show_checkboxes: Option<bool>,
    depth: Option<usize>,
) -> Element {
    let show_cb = show_checkboxes.unwrap_or(true);
    let d = depth.unwrap_or(0);
    let effective_checked = checked_files.unwrap_or_default();
    let base_indent = 16;
    let step_indent = 24;

    rsx! {
        div {
            for node in nodes.iter() {
                {
                    let indent = base_indent + d * step_indent;
                    let path = node.path.clone();

                    if node.kind == FileNodeKind::Dir {
                        let expanded = expanded_dirs.contains(&node.path);
                        let child_files = collect_file_paths(&node.children);
                        let all_checked = child_files.iter().all(|p| effective_checked.contains(p));
                        let some_checked = child_files.iter().any(|p| effective_checked.contains(p));
                        let dir_path = node.path.clone();
                        let dir_path2 = node.path.clone();
                        let dir_path3 = node.path.clone();
                        rsx! {
                            div { key: "{node.path}",
                                div {
                                    class: "group flex w-full items-center gap-1 pr-3 text-left text-sm text-[var(--outline)] hover:bg-[var(--surface-container)]/30 hover:text-[var(--on-surface)] min-h-9",
                                    style: "padding-left: {indent}px",
                                    if show_cb {
                                        label { class: "flex items-center pl-2",
                                            input {
                                                r#type: "checkbox",
                                                checked: all_checked,
                                                onchange: move |_| {
                                                    if let Some(ref handler) = on_toggle_check {
                                                        handler.call((dir_path.clone(), FileNodeKind::Dir));
                                                    }
                                                },
                                                class: "mr-2",
                                            }
                                        }
                                    }
                                    button {
                                        class: "flex min-w-0 items-center gap-2 py-1 text-left",
                                        onclick: move |_| on_toggle_dir.call(dir_path2.clone()),
                                        span { class: "material-symbols-outlined text-sm",
                                            if expanded { "folder_open" } else { "folder" }
                                        }
                                        span { class: "truncate", "{node.name}" }
                                    }
                                    button {
                                        class: "ml-auto flex h-9 w-9 items-center justify-center rounded-sm text-[var(--outline)] hover:bg-[var(--surface-container)]",
                                        onclick: move |_| on_toggle_dir.call(dir_path3.clone()),
                                        span { class: "material-symbols-outlined text-sm",
                                            if expanded { "expand_more" } else { "chevron_right" }
                                        }
                                    }
                                }
                                if expanded {
                                    FileTree {
                                        nodes: node.children.clone(),
                                        selected_file: selected_file.clone(),
                                        expanded_dirs: expanded_dirs.clone(),
                                        checked_files: Some(effective_checked.clone()),
                                        on_toggle_dir: on_toggle_dir.clone(),
                                        on_select_file: on_select_file.clone(),
                                        on_toggle_check: on_toggle_check.clone(),
                                        show_checkboxes: Some(show_cb),
                                        depth: Some(d + 1),
                                    }
                                }
                            }
                        }
                    } else {
                        let checked = effective_checked.contains(&node.path);
                        let is_selected = selected_file.as_deref() == Some(&node.path);
                        let file_path = node.path.clone();
                        let file_path2 = node.path.clone();
                        let icon_name = file_icon(&node.name);
                        let sel_class = if is_selected {
                            " text-[var(--on-surface)] bg-[var(--surface-container)]/20"
                        } else {
                            ""
                        };
                        rsx! {
                            div { key: "{node.path}",
                                div {
                                    class: "flex w-full items-center gap-1 pr-3 text-left text-sm text-[var(--outline)] hover:bg-[var(--surface-container)]/30 hover:text-[var(--on-surface)] cursor-pointer min-h-9{sel_class}",
                                    style: "padding-left: {indent}px",
                                    onclick: move |_| on_select_file.call(file_path.clone()),
                                    if show_cb {
                                        label { class: "flex items-center pl-2",
                                            input {
                                                r#type: "checkbox",
                                                checked: checked,
                                                onchange: move |_| {
                                                    if let Some(ref handler) = on_toggle_check {
                                                        handler.call((file_path2.clone(), FileNodeKind::File));
                                                    }
                                                },
                                                class: "mr-2",
                                            }
                                        }
                                    }
                                    span { class: "material-symbols-outlined text-sm", "{icon_name}" }
                                    span { class: "truncate", "{node.name}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
```

### Task 6: Create `crates/lx-desktop/src/pages/companies/company_card.rs`

Port the per-company card from `Companies.tsx` lines 105-293. Each card shows name (with inline edit), status badge, stats row, dropdown menu, delete confirmation.

```rust
use dioxus::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct CompanyData {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub agent_count: u32,
    pub issue_count: u32,
    pub spent_monthly_cents: u64,
    pub budget_monthly_cents: u64,
    pub created_at: String,
}

fn format_cents(cents: u64) -> String {
    format!("${:.2}", cents as f64 / 100.0)
}

#[component]
pub fn CompanyCard(
    company: CompanyData,
    selected: bool,
    on_select: EventHandler<String>,
    on_rename: EventHandler<(String, String)>,
    on_delete: EventHandler<String>,
) -> Element {
    let mut editing = use_signal(|| false);
    let mut edit_name = use_signal(|| company.name.clone());
    let mut confirming_delete = use_signal(|| false);

    let border = if selected {
        "border-[var(--primary)] ring-1 ring-[var(--primary)]"
    } else {
        "border-[var(--outline-variant)] hover:border-[var(--outline)]"
    };
    let status_class = match company.status.as_str() {
        "active" => "bg-green-500/10 text-green-600",
        "paused" => "bg-yellow-500/10 text-yellow-600",
        _ => "bg-[var(--surface-container)] text-[var(--outline)]",
    };
    let budget_pct = if company.budget_monthly_cents > 0 {
        (company.spent_monthly_cents as f64 / company.budget_monthly_cents as f64 * 100.0) as u32
    } else {
        0
    };
    let id = company.id.clone();
    let id2 = company.id.clone();
    let id3 = company.id.clone();

    rsx! {
        div {
            class: "group text-left bg-[var(--surface-container-lowest)] border rounded-lg p-5 cursor-pointer {border}",
            onclick: move |_| on_select.call(id.clone()),
            div { class: "flex items-start justify-between gap-3",
                div { class: "flex-1 min-w-0",
                    if editing() {
                        div { class: "flex items-center gap-2",
                            onclick: move |evt| evt.stop_propagation(),
                            input {
                                class: "h-7 text-sm border border-[var(--outline-variant)] rounded px-2 bg-transparent text-[var(--on-surface)]",
                                value: "{edit_name}",
                                oninput: move |evt| edit_name.set(evt.value()),
                            }
                            button {
                                class: "text-green-500 text-sm",
                                onclick: move |_| {
                                    on_rename.call((id2.clone(), edit_name().trim().to_string()));
                                    editing.set(false);
                                },
                                span { class: "material-symbols-outlined text-sm", "check" }
                            }
                            button {
                                class: "text-[var(--outline)] text-sm",
                                onclick: move |_| editing.set(false),
                                span { class: "material-symbols-outlined text-sm", "close" }
                            }
                        }
                    } else {
                        div { class: "flex items-center gap-2",
                            h3 { class: "font-semibold text-base text-[var(--on-surface)]",
                                "{company.name}"
                            }
                            span { class: "inline-flex items-center rounded-full px-2 py-0.5 text-[11px] font-medium {status_class}",
                                "{company.status}"
                            }
                        }
                    }
                    if let Some(ref desc) = company.description {
                        if !editing() {
                            p { class: "text-sm text-[var(--outline)] mt-1 line-clamp-2",
                                "{desc}"
                            }
                        }
                    }
                }
            }
            div { class: "flex items-center gap-3 mt-4 text-sm text-[var(--outline)] flex-wrap",
                div { class: "flex items-center gap-1.5",
                    span { class: "material-symbols-outlined text-sm", "group" }
                    span { "{company.agent_count} agents" }
                }
                div { class: "flex items-center gap-1.5",
                    span { class: "material-symbols-outlined text-sm", "radio_button_checked" }
                    span { "{company.issue_count} issues" }
                }
                div { class: "flex items-center gap-1.5 tabular-nums",
                    span { class: "material-symbols-outlined text-sm", "attach_money" }
                    span {
                        "{format_cents(company.spent_monthly_cents)}"
                        if company.budget_monthly_cents > 0 {
                            " / {format_cents(company.budget_monthly_cents)} ({budget_pct}%)"
                        } else {
                            " Unlimited"
                        }
                    }
                }
                div { class: "flex items-center gap-1.5 ml-auto",
                    span { class: "material-symbols-outlined text-sm", "calendar_today" }
                    span { "Created {company.created_at}" }
                }
            }
            if confirming_delete() {
                div {
                    class: "mt-4 flex items-center justify-between bg-red-500/5 border border-red-500/20 rounded-md px-4 py-3",
                    onclick: move |evt| evt.stop_propagation(),
                    p { class: "text-sm text-red-500 font-medium",
                        "Delete this company? This cannot be undone."
                    }
                    div { class: "flex items-center gap-2 ml-4 shrink-0",
                        button {
                            class: "px-3 py-1 text-sm rounded hover:bg-[var(--surface-container)]",
                            onclick: move |_| confirming_delete.set(false),
                            "Cancel"
                        }
                        button {
                            class: "bg-red-600 text-white px-3 py-1 text-sm rounded",
                            onclick: move |_| {
                                on_delete.call(id3.clone());
                                confirming_delete.set(false);
                            },
                            "Delete"
                        }
                    }
                }
            }
        }
    }
}
```

### Task 7: Create `crates/lx-desktop/src/pages/companies/mod.rs`

Port `Companies` from `Companies.tsx`. Company list page with new company button and grid of company cards.

```rust
mod company_card;

use dioxus::prelude::*;
use self::company_card::{CompanyCard, CompanyData};

#[component]
pub fn Companies() -> Element {
    let mut selected_id: Signal<Option<String>> = use_signal(|| None);

    let companies: Vec<CompanyData> = vec![];

    rsx! {
        div { class: "space-y-6 p-4 overflow-auto",
            div { class: "flex items-center justify-end",
                button {
                    class: "flex items-center gap-1.5 bg-[var(--primary)] text-[var(--on-primary)] rounded px-3 py-1.5 text-xs font-semibold",
                    span { class: "material-symbols-outlined text-sm", "add" }
                    "New Company"
                }
            }
            if companies.is_empty() {
                div { class: "flex flex-col items-center justify-center py-16 text-[var(--outline)]",
                    span { class: "material-symbols-outlined text-4xl mb-4", "business" }
                    p { class: "text-sm", "No companies yet." }
                }
            }
            div { class: "grid gap-4",
                for company in companies.iter() {
                    CompanyCard {
                        key: "{company.id}",
                        company: company.clone(),
                        selected: selected_id() == Some(company.id.clone()),
                        on_select: move |id: String| selected_id.set(Some(id)),
                        on_rename: move |(_id, _name): (String, String)| {},
                        on_delete: move |_id: String| {},
                    }
                }
            }
        }
    }
}
```

### Task 8: Create `crates/lx-desktop/src/pages/company_export.rs`

Port the export page from `CompanyExport.tsx`. Two-column layout: left panel is the file tree with checkboxes and search, right panel is a file preview pane.

Reference: `CompanyExport.tsx` lines 1-1018. Key sections: `checkedSlugs` helper, `filterPaperclipYaml` helper (lines 54-208), `filterTree` (lines 211-227), `CompanyExport` component (starts around line 370).

```rust
use dioxus::prelude::*;
use std::collections::HashSet;
use crate::components::file_tree::{
    build_file_tree, collect_file_paths, count_files, FileTree, FileTreeNode, FileNodeKind,
};

#[component]
pub fn CompanyExport() -> Element {
    let mut search_query = use_signal(String::new);
    let mut selected_file: Signal<Option<String>> = use_signal(|| None);
    let mut expanded_dirs: Signal<HashSet<String>> = use_signal(HashSet::new);
    let mut checked_files: Signal<HashSet<String>> = use_signal(HashSet::new);

    let demo_files: Vec<String> = vec![];
    let tree = build_file_tree(&demo_files, None);
    let total_files = count_files(&tree);
    let checked_count = checked_files().len();

    rsx! {
        div { class: "flex flex-col h-full",
            div { class: "flex items-center gap-2 px-4 py-3 border-b border-[var(--outline-variant)]",
                span { class: "material-symbols-outlined text-[var(--outline)]", "inventory_2" }
                h1 { class: "text-lg font-semibold text-[var(--on-surface)]",
                    "Export Company Package"
                }
            }
            div { class: "flex flex-1 overflow-hidden",
                // Left panel: file tree
                div { class: "w-80 border-r border-[var(--outline-variant)] flex flex-col",
                    div { class: "px-3 py-2 border-b border-[var(--outline-variant)]",
                        div { class: "flex items-center gap-2",
                            span { class: "material-symbols-outlined text-sm text-[var(--outline)]",
                                "search"
                            }
                            input {
                                class: "flex-1 bg-transparent text-sm outline-none text-[var(--on-surface)] placeholder-[var(--outline)]",
                                placeholder: "Search files...",
                                value: "{search_query}",
                                oninput: move |evt| search_query.set(evt.value()),
                            }
                        }
                    }
                    div { class: "px-3 py-1.5 text-xs text-[var(--outline)] border-b border-[var(--outline-variant)]",
                        "{checked_count} / {total_files} files selected"
                    }
                    div { class: "flex-1 overflow-auto",
                        FileTree {
                            nodes: tree,
                            selected_file: selected_file(),
                            expanded_dirs: expanded_dirs(),
                            checked_files: Some(checked_files()),
                            on_toggle_dir: move |path: String| {
                                let mut dirs = expanded_dirs();
                                if dirs.contains(&path) {
                                    dirs.remove(&path);
                                } else {
                                    dirs.insert(path);
                                }
                                expanded_dirs.set(dirs);
                            },
                            on_select_file: move |path: String| {
                                selected_file.set(Some(path));
                            },
                            on_toggle_check: Some(EventHandler::new(move |(path, kind): (String, FileNodeKind)| {
                                let mut files = checked_files();
                                if kind == FileNodeKind::File {
                                    if files.contains(&path) {
                                        files.remove(&path);
                                    } else {
                                        files.insert(path);
                                    }
                                }
                                checked_files.set(files);
                            })),
                        }
                    }
                    div { class: "px-3 py-2 border-t border-[var(--outline-variant)]",
                        button {
                            class: "w-full flex items-center justify-center gap-2 bg-[var(--primary)] text-[var(--on-primary)] rounded px-4 py-2 text-sm font-semibold",
                            disabled: checked_count == 0,
                            span { class: "material-symbols-outlined text-sm", "download" }
                            "Export Package"
                        }
                    }
                }
                // Right panel: preview
                div { class: "flex-1 overflow-auto",
                    if selected_file().is_some() {
                        div { class: "p-5",
                            div { class: "border-b border-[var(--outline-variant)] pb-3 mb-4",
                                span { class: "font-mono text-sm text-[var(--on-surface)]",
                                    "{selected_file().unwrap_or_default()}"
                                }
                            }
                            p { class: "text-sm text-[var(--outline)]",
                                "File preview content would appear here."
                            }
                        }
                    } else {
                        div { class: "flex flex-col items-center justify-center h-full text-[var(--outline)]",
                            span { class: "material-symbols-outlined text-4xl mb-4", "inventory_2" }
                            p { class: "text-sm", "Select a file to preview its contents." }
                        }
                    }
                }
            }
        }
    }
}
```

### Task 9: Create `crates/lx-desktop/src/pages/company_import.rs`

Port import page from `CompanyImport.tsx`. Multi-step wizard: source selection (file upload/URL/GitHub), preview with action badges, conflict resolution, apply.

Reference: `CompanyImport.tsx` lines 1-1354. Key sections: `FrontmatterCard` (line 114), `ImportPreviewPane` (line 180), `ConflictItem` (line 267), `CompanyImport` main component (starts around line 400).

```rust
use dioxus::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ImportStep {
    SelectSource,
    Preview,
    Applying,
    Done,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ImportSourceKind {
    FileUpload,
    Url,
    GitHub,
}

#[component]
pub fn CompanyImport() -> Element {
    let mut step = use_signal(|| ImportStep::SelectSource);
    let mut source_kind = use_signal(|| ImportSourceKind::FileUpload);
    let mut url_input = use_signal(String::new);
    let mut github_input = use_signal(String::new);

    rsx! {
        div { class: "flex flex-col h-full",
            div { class: "flex items-center gap-2 px-4 py-3 border-b border-[var(--outline-variant)]",
                span { class: "material-symbols-outlined text-[var(--outline)]", "upload" }
                h1 { class: "text-lg font-semibold text-[var(--on-surface)]",
                    "Import Company Package"
                }
            }
            div { class: "flex-1 overflow-auto p-6",
                match step() {
                    ImportStep::SelectSource => rsx! {
                        div { class: "max-w-2xl mx-auto space-y-6",
                            h2 { class: "text-base font-semibold text-[var(--on-surface)]",
                                "Select Import Source"
                            }
                            div { class: "grid grid-cols-3 gap-4",
                                for (kind, label, icon) in [
                                    (ImportSourceKind::FileUpload, "Upload File", "upload_file"),
                                    (ImportSourceKind::Url, "From URL", "link"),
                                    (ImportSourceKind::GitHub, "From GitHub", "code"),
                                ] {
                                    {
                                        let is_selected = source_kind() == kind;
                                        let border = if is_selected {
                                            "border-[var(--primary)] ring-1 ring-[var(--primary)]"
                                        } else {
                                            "border-[var(--outline-variant)] hover:border-[var(--outline)]"
                                        };
                                        rsx! {
                                            button {
                                                class: "flex flex-col items-center gap-2 p-4 rounded-lg border cursor-pointer {border}",
                                                onclick: move |_| source_kind.set(kind),
                                                span { class: "material-symbols-outlined text-2xl text-[var(--outline)]",
                                                    "{icon}"
                                                }
                                                span { class: "text-sm font-medium text-[var(--on-surface)]",
                                                    "{label}"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            match source_kind() {
                                ImportSourceKind::FileUpload => rsx! {
                                    div { class: "rounded-lg border border-dashed border-[var(--outline-variant)] p-8 text-center",
                                        span { class: "material-symbols-outlined text-4xl text-[var(--outline)] mb-2",
                                            "cloud_upload"
                                        }
                                        p { class: "text-sm text-[var(--outline)]",
                                            "Drop a .zip file here or click to browse"
                                        }
                                        input {
                                            r#type: "file",
                                            accept: ".zip",
                                            class: "mt-2",
                                        }
                                    }
                                },
                                ImportSourceKind::Url => rsx! {
                                    div { class: "space-y-2",
                                        label { class: "text-xs font-medium text-[var(--on-surface)]",
                                            "Package URL"
                                        }
                                        input {
                                            class: "w-full rounded-md border border-[var(--outline-variant)] bg-transparent px-3 py-2 text-sm outline-none text-[var(--on-surface)]",
                                            placeholder: "https://example.com/company-package.zip",
                                            value: "{url_input}",
                                            oninput: move |evt| url_input.set(evt.value()),
                                        }
                                    }
                                },
                                ImportSourceKind::GitHub => rsx! {
                                    div { class: "space-y-2",
                                        label { class: "text-xs font-medium text-[var(--on-surface)]",
                                            "GitHub Repository"
                                        }
                                        input {
                                            class: "w-full rounded-md border border-[var(--outline-variant)] bg-transparent px-3 py-2 text-sm outline-none text-[var(--on-surface)]",
                                            placeholder: "owner/repo",
                                            value: "{github_input}",
                                            oninput: move |evt| github_input.set(evt.value()),
                                        }
                                    }
                                },
                            }
                            div { class: "flex justify-end",
                                button {
                                    class: "bg-[var(--primary)] text-[var(--on-primary)] rounded px-4 py-2 text-sm font-semibold",
                                    onclick: move |_| step.set(ImportStep::Preview),
                                    "Continue"
                                }
                            }
                        }
                    },
                    ImportStep::Preview => rsx! {
                        div { class: "max-w-4xl mx-auto space-y-6",
                            h2 { class: "text-base font-semibold text-[var(--on-surface)]",
                                "Import Preview"
                            }
                            p { class: "text-sm text-[var(--outline)]",
                                "Review the contents before importing."
                            }
                            div { class: "flex justify-between",
                                button {
                                    class: "border border-[var(--outline-variant)] rounded px-4 py-2 text-sm",
                                    onclick: move |_| step.set(ImportStep::SelectSource),
                                    "Back"
                                }
                                button {
                                    class: "bg-[var(--primary)] text-[var(--on-primary)] rounded px-4 py-2 text-sm font-semibold",
                                    onclick: move |_| step.set(ImportStep::Applying),
                                    "Apply Import"
                                }
                            }
                        }
                    },
                    ImportStep::Applying => rsx! {
                        div { class: "flex flex-col items-center justify-center py-16",
                            span { class: "material-symbols-outlined text-4xl text-[var(--primary)] animate-spin mb-4",
                                "progress_activity"
                            }
                            p { class: "text-sm text-[var(--outline)]", "Importing..." }
                        }
                    },
                    ImportStep::Done => rsx! {
                        div { class: "flex flex-col items-center justify-center py-16",
                            span { class: "material-symbols-outlined text-4xl text-green-500 mb-4",
                                "check_circle"
                            }
                            p { class: "text-sm text-[var(--on-surface)]", "Import complete." }
                        }
                    },
                }
            }
        }
    }
}
```

### Task 10: Create `crates/lx-desktop/src/pages/company_skills/new_skill_form.rs`

Port `NewSkillForm` from `CompanySkills.tsx` lines 243-291. Form with name, slug, description inputs.

```rust
use dioxus::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct NewSkillPayload {
    pub name: String,
    pub slug: Option<String>,
    pub description: Option<String>,
}

#[component]
pub fn NewSkillForm(
    on_create: EventHandler<NewSkillPayload>,
    on_cancel: EventHandler<()>,
    is_pending: bool,
) -> Element {
    let mut name = use_signal(String::new);
    let mut slug = use_signal(String::new);
    let mut description = use_signal(String::new);

    rsx! {
        div { class: "border-b border-[var(--outline-variant)] px-4 py-4",
            div { class: "space-y-3",
                input {
                    class: "w-full h-9 border-0 border-b border-[var(--outline-variant)] bg-transparent px-0 text-sm outline-none text-[var(--on-surface)] placeholder-[var(--outline)]",
                    placeholder: "Skill name",
                    value: "{name}",
                    oninput: move |evt| name.set(evt.value()),
                }
                input {
                    class: "w-full h-9 border-0 border-b border-[var(--outline-variant)] bg-transparent px-0 text-sm outline-none text-[var(--on-surface)] placeholder-[var(--outline)]",
                    placeholder: "optional-shortname",
                    value: "{slug}",
                    oninput: move |evt| slug.set(evt.value()),
                }
                textarea {
                    class: "w-full min-h-20 border-0 border-b border-[var(--outline-variant)] bg-transparent px-0 text-sm outline-none text-[var(--on-surface)] placeholder-[var(--outline)]",
                    placeholder: "Short description",
                    value: "{description}",
                    oninput: move |evt| description.set(evt.value()),
                }
                div { class: "flex items-center justify-end gap-2",
                    button {
                        class: "px-3 py-1.5 text-xs rounded hover:bg-[var(--surface-container)]",
                        disabled: is_pending,
                        onclick: move |_| on_cancel.call(()),
                        "Cancel"
                    }
                    button {
                        class: "bg-[var(--primary)] text-[var(--on-primary)] px-3 py-1.5 text-xs rounded font-semibold",
                        disabled: is_pending || name().trim().is_empty(),
                        onclick: move |_| {
                            on_create.call(NewSkillPayload {
                                name: name().trim().to_string(),
                                slug: if slug().trim().is_empty() { None } else { Some(slug().trim().to_string()) },
                                description: if description().trim().is_empty() { None } else { Some(description().trim().to_string()) },
                            });
                        },
                        if is_pending { "Creating..." } else { "Create skill" }
                    }
                }
            }
        }
    }
}
```

### Task 11: Create `crates/lx-desktop/src/pages/company_skills/skill_tree.rs`

Port `SkillTree` from `CompanySkills.tsx` lines 294-380. Recursive file tree for skill files with directory expand/collapse.

```rust
use dioxus::prelude::*;
use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq)]
pub struct SkillTreeNode {
    pub name: String,
    pub path: Option<String>,
    pub kind: SkillNodeKind,
    pub file_kind: Option<String>,
    pub children: Vec<SkillTreeNode>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SkillNodeKind {
    Dir,
    File,
}

fn file_icon(kind: Option<&str>) -> &'static str {
    match kind {
        Some("script" | "reference") => "code",
        _ => "description",
    }
}

#[component]
pub fn SkillTree(
    nodes: Vec<SkillTreeNode>,
    selected_path: String,
    expanded_dirs: HashSet<String>,
    on_toggle_dir: EventHandler<String>,
    on_select_path: EventHandler<String>,
    depth: Option<usize>,
) -> Element {
    let d = depth.unwrap_or(0);
    let base_indent = 16;
    let step_indent = 24;

    rsx! {
        div {
            for node in nodes.iter() {
                {
                    let indent = base_indent + d * step_indent;
                    if node.kind == SkillNodeKind::Dir {
                        let expanded = node.path.as_ref().map_or(false, |p| expanded_dirs.contains(p));
                        let dir_path = node.path.clone().unwrap_or_default();
                        let dir_path2 = dir_path.clone();
                        rsx! {
                            div { key: "{node.name}",
                                div {
                                    class: "group flex w-full items-center gap-1 pr-3 text-left text-sm text-[var(--outline)] hover:bg-[var(--surface-container)]/30 hover:text-[var(--on-surface)] min-h-9",
                                    button {
                                        class: "flex min-w-0 items-center gap-2 py-1 text-left",
                                        style: "padding-left: {indent}px",
                                        onclick: move |_| on_toggle_dir.call(dir_path.clone()),
                                        span { class: "material-symbols-outlined text-sm",
                                            if expanded { "folder_open" } else { "folder" }
                                        }
                                        span { class: "truncate", "{node.name}" }
                                    }
                                    button {
                                        class: "ml-auto flex h-9 w-9 items-center justify-center",
                                        onclick: move |_| on_toggle_dir.call(dir_path2.clone()),
                                        span { class: "material-symbols-outlined text-sm",
                                            if expanded { "expand_more" } else { "chevron_right" }
                                        }
                                    }
                                }
                                if expanded {
                                    SkillTree {
                                        nodes: node.children.clone(),
                                        selected_path: selected_path.clone(),
                                        expanded_dirs: expanded_dirs.clone(),
                                        on_toggle_dir: on_toggle_dir.clone(),
                                        on_select_path: on_select_path.clone(),
                                        depth: Some(d + 1),
                                    }
                                }
                            }
                        }
                    } else {
                        let is_selected = node.path.as_deref() == Some(selected_path.as_str());
                        let file_path = node.path.clone().unwrap_or_default();
                        let icon = file_icon(node.file_kind.as_deref());
                        let sel_class = if is_selected {
                            " text-[var(--on-surface)] bg-[var(--surface-container)]/20"
                        } else { "" };
                        rsx! {
                            div { key: "{node.name}",
                                button {
                                    class: "flex w-full min-w-0 items-center gap-2 py-1 text-left text-sm text-[var(--outline)] hover:bg-[var(--surface-container)]/30 hover:text-[var(--on-surface)] min-h-9{sel_class}",
                                    style: "padding-left: {indent}px",
                                    onclick: move |_| on_select_path.call(file_path.clone()),
                                    span { class: "material-symbols-outlined text-sm", "{icon}" }
                                    span { class: "truncate", "{node.name}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
```

### Task 12: Create `crates/lx-desktop/src/pages/company_skills/mod.rs`

Port `CompanySkills` from `CompanySkills.tsx`. Two-panel layout: skill list on left, skill detail (file tree + content viewer) on right.

Reference: `CompanySkills.tsx` lines 380+ for the main `CompanySkills` component.

```rust
mod new_skill_form;
mod skill_tree;

use dioxus::prelude::*;
use self::new_skill_form::{NewSkillForm, NewSkillPayload};
use self::skill_tree::{SkillTree, SkillTreeNode};

#[derive(Clone, Debug, PartialEq)]
struct SkillListItem {
    id: String,
    name: String,
    slug: String,
    description: Option<String>,
    source_badge: String,
}

#[component]
pub fn CompanySkills() -> Element {
    let mut selected_skill_id: Signal<Option<String>> = use_signal(|| None);
    let mut show_new_form = use_signal(|| false);
    let mut search_query = use_signal(String::new);
    let mut selected_file = use_signal(|| "SKILL.md".to_string());
    let mut expanded_dirs: Signal<std::collections::HashSet<String>> =
        use_signal(std::collections::HashSet::new);

    let skills: Vec<SkillListItem> = vec![];
    let tree_nodes: Vec<SkillTreeNode> = vec![];

    rsx! {
        div { class: "flex h-full",
            // Left panel: skill list
            div { class: "w-72 border-r border-[var(--outline-variant)] flex flex-col",
                div { class: "flex items-center justify-between px-3 py-2 border-b border-[var(--outline-variant)]",
                    div { class: "flex items-center gap-2",
                        span { class: "material-symbols-outlined text-[var(--outline)]",
                            "widgets"
                        }
                        h1 { class: "text-base font-semibold text-[var(--on-surface)]",
                            "Skills"
                        }
                    }
                    button {
                        class: "p-1 rounded hover:bg-[var(--surface-container)]",
                        onclick: move |_| {
                            let current = show_new_form();
                            show_new_form.set(!current);
                        },
                        span { class: "material-symbols-outlined text-sm", "add" }
                    }
                }
                div { class: "px-3 py-2 border-b border-[var(--outline-variant)]",
                    div { class: "flex items-center gap-2",
                        span { class: "material-symbols-outlined text-sm text-[var(--outline)]",
                            "search"
                        }
                        input {
                            class: "flex-1 bg-transparent text-sm outline-none text-[var(--on-surface)] placeholder-[var(--outline)]",
                            placeholder: "Search skills...",
                            value: "{search_query}",
                            oninput: move |evt| search_query.set(evt.value()),
                        }
                    }
                }
                if show_new_form() {
                    NewSkillForm {
                        on_create: move |_payload: NewSkillPayload| {
                            show_new_form.set(false);
                        },
                        on_cancel: move |_| show_new_form.set(false),
                        is_pending: false,
                    }
                }
                div { class: "flex-1 overflow-auto",
                    if skills.is_empty() {
                        div { class: "flex flex-col items-center justify-center py-12 text-[var(--outline)]",
                            span { class: "material-symbols-outlined text-3xl mb-3",
                                "widgets"
                            }
                            p { class: "text-xs", "No skills yet." }
                        }
                    }
                    for skill in skills.iter() {
                        {
                            let skill_id = skill.id.clone();
                            let is_selected = selected_skill_id() == Some(skill.id.clone());
                            let bg = if is_selected { " bg-[var(--surface-container)]" } else { "" };
                            rsx! {
                                button {
                                    class: "w-full text-left px-3 py-2 border-b border-[var(--outline-variant)]/30 hover:bg-[var(--surface-container)]{bg}",
                                    onclick: move |_| selected_skill_id.set(Some(skill_id.clone())),
                                    div { class: "text-sm font-medium text-[var(--on-surface)]",
                                        "{skill.name}"
                                    }
                                    if let Some(ref desc) = skill.description {
                                        p { class: "text-xs text-[var(--outline)] mt-0.5 truncate",
                                            "{desc}"
                                        }
                                    }
                                    div { class: "text-[10px] text-[var(--outline)] mt-0.5",
                                        "{skill.source_badge}"
                                    }
                                }
                            }
                        }
                    }
                }
            }
            // Right panel: skill detail
            div { class: "flex-1 flex",
                if selected_skill_id().is_some() {
                    // File tree sidebar
                    div { class: "w-56 border-r border-[var(--outline-variant)] overflow-auto",
                        SkillTree {
                            nodes: tree_nodes.clone(),
                            selected_path: selected_file(),
                            expanded_dirs: expanded_dirs(),
                            on_toggle_dir: move |path: String| {
                                let mut dirs = expanded_dirs();
                                if dirs.contains(&path) {
                                    dirs.remove(&path);
                                } else {
                                    dirs.insert(path);
                                }
                                expanded_dirs.set(dirs);
                            },
                            on_select_path: move |path: String| {
                                selected_file.set(path);
                            },
                        }
                    }
                    // Content viewer
                    div { class: "flex-1 overflow-auto p-5",
                        div { class: "border-b border-[var(--outline-variant)] pb-3 mb-4",
                            span { class: "font-mono text-sm text-[var(--on-surface)]",
                                "{selected_file}"
                            }
                        }
                        p { class: "text-sm text-[var(--outline)]",
                            "File content would appear here."
                        }
                    }
                } else {
                    div { class: "flex-1 flex flex-col items-center justify-center text-[var(--outline)]",
                        span { class: "material-symbols-outlined text-4xl mb-4", "widgets" }
                        p { class: "text-sm", "Select a skill to view its files." }
                    }
                }
            }
        }
    }
}
```

### Task 13: Verify `pages/mod.rs` and `routes.rs`

Unit 3 already created stub modules (`pages/companies.rs`, `pages/company_export.rs`, `pages/company_import.rs`, `pages/company_skills.rs`) and declared them in `pages/mod.rs`. Unit 3 also added the corresponding route variants (`Companies`, `CompanyExport`, `CompanyImport`, `CompanySkills`) to the `Route` enum in `routes.rs`. Creating the real directory modules in Tasks 7-12 above replaces those stubs automatically. Do NOT modify `routes.rs` or `pages/mod.rs`.

## Definition of Done

1. `just diagnose` passes with zero warnings
2. All new files exist at the paths listed above
3. All new files are under 300 lines
4. `lib.rs` includes `pub mod components`
5. `components/mod.rs` declares all three component modules
6. `pages/mod.rs` already includes the four page modules (from Unit 3 stubs)
7. `routes.rs` already has the four route variants (from Unit 3) — no modifications made
8. `CompanyPatternIcon` renders a colored square with company initial
9. `CompanySwitcher` renders a dropdown with company list and status dots
10. `FileTree` renders recursive file tree with checkboxes, expand/collapse, and file selection
11. `Companies` page renders company card grid with stats
12. `CompanyExport` renders two-column layout with file tree and preview pane
13. `CompanyImport` renders multi-step import wizard
14. `CompanySkills` renders two-panel layout with skill list and file tree browser
