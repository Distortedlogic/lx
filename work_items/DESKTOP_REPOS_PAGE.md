# Goal

Make the Repos page interactive: FileTree browses the actual filesystem, AstConfig controls update state, the RUN ANALYSIS ENGINE button triggers analysis, and ChunksPanel renders results reactively.

# Why

Every element on the Repos page is hardcoded. The file tree is a `const ENTRIES` slice of fake paths. The AstConfig mode buttons, slider, and dropdown have no handlers. The ChunksPanel displays hardcoded chunks. The RUN ANALYSIS ENGINE button has no onclick. The page is entirely non-functional.

# Architecture

A shared `ReposState` context struct provided at the Repos page level holds the selected directory path, analysis mode, analysis results, and loading state. FileTree populates from the filesystem via `use_resource`. AstConfig mode buttons and controls write to the state. The RUN button triggers an async analysis task. ChunksPanel renders from the analysis results signal.

The analysis itself reads the selected directory, counts files by extension, and produces mock chunks summarizing the file distribution. This is a placeholder analysis — the architecture is designed so a real AST analysis engine can be plugged in by replacing the `run_analysis` function.

# Files Affected

| File | Change |
|------|--------|
| `src/pages/repos/state.rs` | New file — ReposState context + analysis types |
| `src/pages/repos/mod.rs` | Provide ReposState, wire RUN button |
| `src/pages/repos/file_tree.rs` | Rewrite with use_resource for filesystem browsing |
| `src/pages/repos/ast_config.rs` | Wire mode buttons and controls to state |
| `src/pages/repos/chunks_panel.rs` | Render from analysis results signal |

# Task List

### Task 1: Create ReposState context and analysis types

**Subject:** Define shared state for the Repos page

**Description:** Create `crates/lx-desktop/src/pages/repos/state.rs`:

```rust
use dioxus::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AnalysisMode {
    Syntactic,
    Semantic,
    Hybrid,
}

impl std::fmt::Display for AnalysisMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Syntactic => write!(f, "SYNTACTIC"),
            Self::Semantic => write!(f, "SEMANTIC"),
            Self::Hybrid => write!(f, "HYBRID"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ChunkResult {
    pub id: String,
    pub score: f64,
    pub description: String,
    pub tags: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AnalysisResults {
    pub chunks: Vec<ChunkResult>,
    pub total_tokens: usize,
    pub latency_ms: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TreeNode {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub depth: u8,
}

#[derive(Clone, Copy)]
pub struct ReposState {
    pub root_path: Signal<String>,
    pub selected_file: Signal<Option<String>>,
    pub mode: Signal<AnalysisMode>,
    pub tree_depth: Signal<f64>,
    pub results: Signal<Option<AnalysisResults>>,
    pub analyzing: Signal<bool>,
}

impl ReposState {
    pub fn provide() -> Self {
        let cwd = std::env::current_dir()
            .ok()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| ".".into());
        let ctx = Self {
            root_path: Signal::new(cwd),
            selected_file: Signal::new(None),
            mode: Signal::new(AnalysisMode::Syntactic),
            tree_depth: Signal::new(3.0),
            results: Signal::new(None),
            analyzing: Signal::new(false),
        };
        use_context_provider(|| ctx);
        ctx
    }
}

pub async fn read_dir_tree(root: &str, max_depth: u8) -> Vec<TreeNode> {
    let mut nodes = Vec::new();
    let mut stack: Vec<(String, u8)> = vec![(root.to_string(), 0)];
    while let Some((path, depth)) = stack.pop() {
        if depth > max_depth {
            continue;
        }
        let Ok(mut entries) = tokio::fs::read_dir(&path).await else { continue };
        let mut children = Vec::new();
        while let Ok(Some(entry)) = entries.next_entry().await {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') {
                continue;
            }
            let full_path = entry.path().display().to_string();
            let is_dir = entry.file_type().await.map(|t| t.is_dir()).unwrap_or(false);
            children.push((name, full_path, is_dir));
        }
        children.sort_by(|a, b| match (a.2, b.2) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.0.cmp(&b.0),
        });
        for (name, full_path, is_dir) in children {
            nodes.push(TreeNode { name, path: full_path.clone(), is_dir, depth });
            if is_dir {
                stack.push((full_path, depth + 1));
            }
        }
    }
    nodes
}

pub async fn run_analysis(root: &str) -> AnalysisResults {
    let start = std::time::Instant::now();
    let mut file_count = 0usize;
    let mut total_bytes = 0usize;
    let mut ext_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut stack = vec![root.to_string()];
    while let Some(path) = stack.pop() {
        let Ok(mut entries) = tokio::fs::read_dir(&path).await else { continue };
        while let Ok(Some(entry)) = entries.next_entry().await {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') { continue; }
            let is_dir = entry.metadata().await.map(|m| m.is_dir()).unwrap_or(false);
            if is_dir {
                stack.push(entry.path().display().to_string());
            } else {
                file_count += 1;
                let size = entry.metadata().await.map(|m| m.len() as usize).unwrap_or(0);
                total_bytes += size;
                let ext = entry.path().extension().and_then(|e| e.to_str()).unwrap_or("other").to_string();
                *ext_counts.entry(ext).or_default() += 1;
            }
        }
    }
    let latency = start.elapsed().as_millis() as u64;
    let mut chunks: Vec<ChunkResult> = ext_counts
        .iter()
        .enumerate()
        .map(|(i, (ext, count))| {
            let score = (*count as f64) / (file_count.max(1) as f64);
            ChunkResult {
                id: format!("#CHUNK_{:04}", i),
                score,
                description: format!("{count} .{ext} files found in repository"),
                tags: vec![ext.to_uppercase()],
            }
        })
        .collect();
    chunks.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    let token_estimate = total_bytes / 4;
    AnalysisResults { chunks, total_tokens: token_estimate, latency_ms: latency }
}
```

The `read_dir_tree` function does a depth-first directory scan skipping dotfiles, returning `TreeNode` entries sorted directories-first within each level. The `run_analysis` function counts files by extension, producing `ChunkResult` entries ordered by frequency. This is a placeholder analysis — replace `run_analysis` with a real AST engine when one exists.

**ActiveForm:** Creating ReposState context and analysis functions

---

### Task 2: Rewrite FileTree with use_resource for filesystem browsing

**Subject:** Replace hardcoded file tree with live directory listing

**Description:** Rewrite `crates/lx-desktop/src/pages/repos/file_tree.rs`. Remove `const ENTRIES` and the `TreeEntry` struct.

```rust
use dioxus::prelude::*;
use super::state::{ReposState, TreeNode, read_dir_tree};

#[component]
pub fn FileTree() -> Element {
    let repos = use_context::<ReposState>();
    let root = (repos.root_path)();
    let depth = (repos.tree_depth)() as u8;
    let selected = (repos.selected_file)();

    let tree = use_resource(move || {
        let root = root.clone();
        async move { read_dir_tree(&root, depth).await }
    });

    rsx! {
        div { class: "w-64 bg-[var(--surface-container-low)] border-r border-[var(--outline-variant)]/15 p-4 flex flex-col shrink-0 overflow-auto",
            div { class: "flex items-center justify-between mb-4",
                span { class: "text-xs font-semibold uppercase tracking-wider text-[var(--on-surface)]", "REPOSITORY HUB" }
            }
            match &*tree.value().read() {
                Some(nodes) => rsx! {
                    div { class: "flex flex-col gap-0.5",
                        for node in nodes.iter() {
                            {
                                let path = node.path.clone();
                                let is_selected = selected.as_deref() == Some(path.as_str());
                                let pad = format!("padding-left: {}rem;", node.depth as f32 * 1.0 + 0.5);
                                let icon = if node.is_dir { "\u{1F4C1}" } else { "\u{2192}" };
                                let color = if node.is_dir { "text-[var(--primary)]" } else { "text-[var(--on-surface-variant)]" };
                                let bg = if is_selected { " bg-[var(--surface-container-high)]" } else { "" };
                                rsx! {
                                    div {
                                        class: "flex items-center gap-2 py-1.5 px-2 text-xs rounded cursor-pointer hover:bg-[var(--surface-container-high)] transition-colors duration-150 {color}{bg}",
                                        style: "{pad}",
                                        onclick: move |_| {
                                            repos.selected_file.set(Some(path.clone()));
                                        },
                                        span { "{icon}" }
                                        span { "{node.name}" }
                                    }
                                }
                            }
                        }
                    }
                },
                None => rsx! {
                    div { class: "text-xs text-[var(--outline)] py-4 text-center", "Loading..." }
                },
            }
        }
    }
}
```

The `use_resource` re-runs when `root` or `depth` signals change. Clicking a node sets `selected_file`. Directories-first sort happens in `read_dir_tree`.

**ActiveForm:** Rewriting FileTree with use_resource and filesystem browsing

---

### Task 3: Wire AstConfig interactive controls

**Subject:** Connect mode buttons, slider, and display fields to ReposState

**Description:** Rewrite `crates/lx-desktop/src/pages/repos/ast_config.rs`:

```rust
use dioxus::prelude::*;
use super::state::{AnalysisMode, ReposState};

#[component]
pub fn AstConfig() -> Element {
    let repos = use_context::<ReposState>();
    let current_mode = (repos.mode)();
    let depth = (repos.tree_depth)();
    let selected = (repos.selected_file)();
    let results = repos.results.read();
    let node_count = results.as_ref().map(|r| r.total_tokens).unwrap_or(0);

    let file_content = use_resource(move || {
        let sel = selected.clone();
        async move {
            match sel {
                Some(path) if !path.is_empty() => tokio::fs::read_to_string(&path).await.ok(),
                _ => None,
            }
        }
    });

    let modes = [AnalysisMode::Syntactic, AnalysisMode::Semantic, AnalysisMode::Hybrid];

    rsx! {
        div { class: "flex-1 flex flex-col p-4 overflow-auto min-w-0",
            div { class: "flex items-center justify-between mb-4",
                span { class: "text-sm font-semibold uppercase tracking-wider text-[var(--on-surface)]", "AST CONFIGURATION & ANALYSIS" }
                span { class: "text-xs text-[var(--outline)] uppercase tracking-wider", "TOKEN_EST: {node_count}" }
            }
            span { class: "text-[10px] uppercase tracking-wider text-[var(--outline)] mb-1", "ANALYSIS MODE" }
            div { class: "flex gap-0 mb-4",
                for mode in modes {
                    {
                        let is_active = mode == current_mode;
                        let cls = if is_active {
                            "bg-[var(--primary)] text-[var(--on-primary)] px-6 py-2 text-xs uppercase tracking-wider font-semibold cursor-pointer"
                        } else {
                            "bg-[var(--surface-container)] text-[var(--outline)] px-6 py-2 text-xs uppercase tracking-wider border border-[var(--outline-variant)]/30 cursor-pointer hover:text-[var(--on-surface)] transition-colors duration-150"
                        };
                        rsx! {
                            span {
                                class: "{cls}",
                                onclick: move |_| repos.mode.set(mode),
                                "{mode}"
                            }
                        }
                    }
                }
            }
            div { class: "bg-[var(--surface-container)] border border-[var(--outline-variant)]/30 rounded-lg p-4 mb-4",
                div { class: "flex items-center justify-between mb-2",
                    span { class: "text-xs font-semibold uppercase tracking-wider text-[var(--on-surface)]", "TREE_DEPTH" }
                    span { class: "text-[var(--warning)]", "{depth:.0}" }
                }
                input {
                    r#type: "range",
                    min: "1",
                    max: "10",
                    step: "1",
                    value: "{depth}",
                    class: "w-full accent-[var(--primary)] mb-3",
                    oninput: move |evt| {
                        if let Ok(v) = evt.value().parse::<f64>() {
                            repos.tree_depth.set(v);
                        }
                    },
                }
            }
            div { class: "bg-[var(--surface-container)] border border-[var(--outline-variant)]/30 rounded-lg p-4 flex-1 overflow-auto",
                p { class: "text-xs font-semibold uppercase tracking-wider text-[var(--on-surface)] mb-3", "FILE_PREVIEW" }
                match &*file_content.value().read() {
                    Some(Some(content)) => rsx! {
                        pre { class: "text-xs font-mono text-[var(--on-surface-variant)] whitespace-pre leading-relaxed max-h-64 overflow-auto", "{content}" }
                    },
                    _ => rsx! {
                        p { class: "text-xs text-[var(--outline)]", "Select a file to preview" }
                    },
                }
            }
        }
    }
}
```

The mode buttons write to `repos.mode`. The slider writes to `repos.tree_depth` (which triggers FileTree's `use_resource` to re-run). The preview pane reads the selected file via `use_resource`. The token count comes from analysis results.

**ActiveForm:** Wiring AstConfig interactive controls to ReposState

---

### Task 4: Wire RUN ANALYSIS ENGINE button and provide ReposState

**Subject:** Connect the analysis button and provide the shared context

**Description:** Edit `crates/lx-desktop/src/pages/repos/mod.rs`. Add `mod state;` to the module declarations.

Rewrite the `Repos` component:

```rust
use dioxus::prelude::*;
use self::ast_config::AstConfig;
use self::chunks_panel::ChunksPanel;
use self::file_tree::FileTree;
use self::state::{ReposState, run_analysis};

#[component]
pub fn Repos() -> Element {
    let repos = ReposState::provide();
    let analyzing = (repos.analyzing)();

    rsx! {
        div { class: "flex h-full",
            FileTree {}
            div { class: "flex-1 flex flex-col min-h-0 min-w-0",
                AstConfig {}
                div { class: "p-4 border-t border-[var(--outline-variant)]/15",
                    button {
                        class: "w-full bg-[var(--success)] text-[var(--on-primary)] rounded py-3 text-sm uppercase tracking-wider font-semibold hover:brightness-110 transition-all duration-150",
                        disabled: analyzing,
                        onclick: move |_| {
                            let root = (repos.root_path)();
                            spawn(async move {
                                repos.analyzing.set(true);
                                let results = run_analysis(&root).await;
                                repos.results.set(Some(results));
                                repos.analyzing.set(false);
                            });
                        },
                        if analyzing { "\u{23F3} ANALYZING..." } else { "\u{26A1} RUN ANALYSIS ENGINE" }
                    }
                }
            }
            ChunksPanel {}
        }
    }
}
```

The button spawns an async task that runs the analysis and writes results to `repos.results`. While running, the button shows a spinner text and is disabled.

**ActiveForm:** Wiring RUN ANALYSIS ENGINE button with async execution

---

### Task 5: Wire ChunksPanel to reactive analysis results

**Subject:** Replace hardcoded chunks with signal-driven rendering from analysis results

**Description:** Rewrite `crates/lx-desktop/src/pages/repos/chunks_panel.rs`. Remove `const CHUNKS` and the `Chunk` struct.

```rust
use dioxus::prelude::*;
use super::state::ReposState;

#[component]
pub fn ChunksPanel() -> Element {
    let repos = use_context::<ReposState>();
    let results = repos.results.read();

    rsx! {
        div { class: "w-72 bg-[var(--surface-container-low)] border-l border-[var(--outline-variant)]/15 p-4 flex flex-col shrink-0 overflow-auto",
            div { class: "flex items-center justify-between mb-4",
                span { class: "text-xs font-semibold uppercase tracking-wider text-[var(--on-surface)]", "EXTRACTED CHUNKS" }
            }
            match results.as_ref() {
                Some(analysis) => rsx! {
                    div { class: "flex flex-col gap-3 flex-1",
                        for chunk in analysis.chunks.iter() {
                            {
                                let score_color = if chunk.score > 0.5 { "bg-[var(--success)]" } else if chunk.score > 0.2 { "bg-[var(--warning)]" } else { "bg-[var(--error)]" };
                                let score_text = format!("{:.3}", chunk.score);
                                rsx! {
                                    div { class: "bg-[var(--surface-container)] border border-[var(--outline-variant)]/30 rounded-lg p-3",
                                        div { class: "flex items-center justify-between mb-2",
                                            span { class: "text-xs text-[var(--primary)] font-semibold", "{chunk.id}" }
                                            span { class: "{score_color} text-[var(--on-primary)] text-[10px] px-2 py-0.5 rounded font-semibold", "{score_text}" }
                                        }
                                        p { class: "text-[10px] text-[var(--on-surface-variant)] leading-relaxed mb-2", "{chunk.description}" }
                                        if !chunk.tags.is_empty() {
                                            div { class: "flex gap-1",
                                                for tag in chunk.tags.iter() {
                                                    span { class: "bg-[var(--surface-container-high)] text-[10px] text-[var(--outline)] px-2 py-0.5 rounded uppercase tracking-wider", "{tag}" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    div { class: "mt-4 pt-3 border-t border-[var(--outline-variant)]/15",
                        p { class: "text-[10px] uppercase tracking-wider text-[var(--outline)] mb-1", "ANALYSIS STATS" }
                        p { class: "text-xs text-[var(--on-surface-variant)]", "TOTAL_TOKENS: {analysis.total_tokens}" }
                        p { class: "text-xs text-[var(--on-surface-variant)]", "LATENCY: {analysis.latency_ms}ms" }
                    }
                },
                None => rsx! {
                    div { class: "flex-1 flex items-center justify-center",
                        p { class: "text-xs text-[var(--outline)] text-center", "Run analysis to see results" }
                    }
                },
            }
        }
    }
}
```

When no analysis has been run, shows a placeholder message. After analysis, renders chunks from `repos.results` with dynamic score coloring and live stats.

**ActiveForm:** Wiring ChunksPanel to reactive analysis results

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

1. **Call `complete_task` after each task.**
2. **Call `next_task` to get the next task.**
3. **Do not add, skip, reorder, or combine tasks.**
4. **Tasks are implementation-only.**

---

## Task Loading Instructions

```
mcp__workflow__load_work_item({ path: "work_items/DESKTOP_REPOS_PAGE.md" })
```
