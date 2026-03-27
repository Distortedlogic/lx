# Goal

Perform eight mechanical structural cleanups across the workspace.

# Violations

## 1. Remove dead trait: `UserBackend`

**File:** `crates/lx/src/runtime/mod.rs`

Delete the following block:

```rust
pub trait UserBackend: Send + Sync {
  fn confirm(&self, message: &str) -> Result<bool, String>;
  fn choose(&self, message: &str, options: &[String]) -> Result<usize, String>;
  fn ask(&self, message: &str, default: Option<&str>) -> Result<String, String>;
  fn progress(&self, current: usize, total: usize, message: &str);
  fn progress_pct(&self, pct: f64, message: &str);
  fn status(&self, level: &str, message: &str);
  fn table(&self, headers: &[String], rows: &[Vec<String>]);
  fn check_signal(&self) -> Option<LxVal>;
}
```

---

## 2. Flatten `components/mod.rs` intermediary in lx-mobile

1. Copy the contents of `crates/lx-mobile/src/components/pulse_indicator.rs` into a new file `crates/lx-mobile/src/components.rs` (at the same level as `main.rs`).

2. Delete the entire directory `crates/lx-mobile/src/components/` (both `mod.rs` and `pulse_indicator.rs`).

3. The `mod components;` declaration in `crates/lx-mobile/src/main.rs` stays unchanged.

4. In `crates/lx-mobile/src/pages/status.rs`, find:

```rust
use crate::components::pulse_indicator::{ExecutionState, PulseIndicator};
```

Replace with:

```rust
use crate::components::{ExecutionState, PulseIndicator};
```

---

## 3. Free function to method: `workspace_member_map`

**File:** `crates/lx-cli/src/manifest.rs`

**Step 1:** Find the closing brace of the `Workspace` struct. Immediately after:

```rust
pub struct Workspace {
  pub members: Vec<Member>,
}
```

Insert:

```rust

impl Workspace {
  pub fn member_map(&self) -> HashMap<String, PathBuf> {
    self.members.iter().map(|m| (m.name.clone(), m.dir.clone())).collect()
  }
}
```

**Step 2:** Delete the standalone function:

```rust
pub fn workspace_member_map(ws: &Workspace) -> HashMap<String, PathBuf> {
  ws.members.iter().map(|m| (m.name.clone(), m.dir.clone())).collect()
}
```

**Step 3:** In `crates/lx-cli/src/manifest.rs`, inside `try_load_workspace_members()`, find:

```rust
  workspace_member_map(&ws)
```

Replace with:

```rust
  ws.member_map()
```

**Step 4:** In `crates/lx-cli/src/testing.rs`, find:

```rust
  let ws_members = manifest::workspace_member_map(&ws);
```

Replace with:

```rust
  let ws_members = ws.member_map();
```

---

## 4. Inline single-call-site function: `h_level`

**File:** `crates/lx/src/stdlib/md/mod.rs`

**Step 1:** Delete this function:

```rust
fn h_level(level: HeadingLevel) -> i64 {
  match level {
    HeadingLevel::H1 => 1,
    HeadingLevel::H2 => 2,
    HeadingLevel::H3 => 3,
    HeadingLevel::H4 => 4,
    HeadingLevel::H5 => 5,
    HeadingLevel::H6 => 6,
  }
}
```

**Step 2:** Find:

```rust
            nodes.push(node_rec("heading", vec![("level", LxVal::int(h_level(level))), ("text", LxVal::str(text.trim()))]));
```

Replace with:

```rust
            let h = match level {
              HeadingLevel::H1 => 1,
              HeadingLevel::H2 => 2,
              HeadingLevel::H3 => 3,
              HeadingLevel::H4 => 4,
              HeadingLevel::H5 => 5,
              HeadingLevel::H6 => 6,
            };
            nodes.push(node_rec("heading", vec![("level", LxVal::int(h)), ("text", LxVal::str(text.trim()))]));
```

---

## 5. Single-call-site functions to `FromStr` impls

**Step A:** At the end of `crates/lx/src/stdlib/diag/diag_types.rs`, after the `Graph` struct closing brace, append:

```rust

use std::str::FromStr;

impl FromStr for NodeKind {
  type Err = String;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "agent" => Ok(Self::Agent),
      "tool" => Ok(Self::Tool),
      "decision" => Ok(Self::Decision),
      "fork" => Ok(Self::Fork),
      "join" => Ok(Self::Join),
      "loop" => Ok(Self::Loop),
      "resource" => Ok(Self::Resource),
      "user" => Ok(Self::User),
      "io" => Ok(Self::Io),
      "type" => Ok(Self::Type),
      other => Err(format!("unknown node kind: {other}")),
    }
  }
}

impl FromStr for EdgeStyle {
  type Err = String;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "solid" => Ok(Self::Solid),
      "dashed" => Ok(Self::Dashed),
      "double" => Ok(Self::Double),
      other => Err(format!("unknown edge style: {other}")),
    }
  }
}

impl FromStr for EdgeType {
  type Err = String;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "agent" => Ok(Self::Agent),
      "stream" => Ok(Self::Stream),
      "data" => Ok(Self::Data),
      "io" => Ok(Self::Io),
      "exec" => Ok(Self::Exec),
      other => Err(format!("unknown edge type: {other}")),
    }
  }
}
```

**Step B:** In `crates/lx/src/stdlib/diag/mod.rs`, delete these three functions:

```rust
fn parse_node_kind(s: &str) -> Result<NodeKind, String> {
  match s {
    "agent" => Ok(NodeKind::Agent),
    "tool" => Ok(NodeKind::Tool),
    "decision" => Ok(NodeKind::Decision),
    "fork" => Ok(NodeKind::Fork),
    "join" => Ok(NodeKind::Join),
    "loop" => Ok(NodeKind::Loop),
    "resource" => Ok(NodeKind::Resource),
    "user" => Ok(NodeKind::User),
    "io" => Ok(NodeKind::Io),
    "type" => Ok(NodeKind::Type),
    other => Err(format!("unknown node kind: {other}")),
  }
}

fn parse_edge_style(s: &str) -> Result<EdgeStyle, String> {
  match s {
    "solid" => Ok(EdgeStyle::Solid),
    "dashed" => Ok(EdgeStyle::Dashed),
    "double" => Ok(EdgeStyle::Double),
    other => Err(format!("unknown edge style: {other}")),
  }
}

fn parse_edge_type(s: &str) -> Result<EdgeType, String> {
  match s {
    "agent" => Ok(EdgeType::Agent),
    "stream" => Ok(EdgeType::Stream),
    "data" => Ok(EdgeType::Data),
    "io" => Ok(EdgeType::Io),
    "exec" => Ok(EdgeType::Exec),
    other => Err(format!("unknown edge type: {other}")),
  }
}
```

**Step C:** In `crates/lx/src/stdlib/diag/mod.rs`, find:

```rust
  let kind = parse_node_kind(&kind_str).map_err(|e| LxError::type_err(e, span, None))?;
```

Replace with:

```rust
  let kind = kind_str.parse::<NodeKind>().map_err(|e| LxError::type_err(e, span, None))?;
```

**Step D:** In `crates/lx/src/stdlib/diag/mod.rs`, find:

```rust
  let style = parse_edge_style(&style_str).map_err(|e| LxError::type_err(e, span, None))?;
```

Replace with:

```rust
  let style = style_str.parse::<EdgeStyle>().map_err(|e| LxError::type_err(e, span, None))?;
```

**Step E:** In `crates/lx/src/stdlib/diag/mod.rs`, find:

```rust
  let edge_type = parse_edge_type(edge_type_str).map_err(|e| LxError::type_err(e, span, None))?;
```

Replace with:

```rust
  let edge_type = edge_type_str.parse::<EdgeType>().map_err(|e| LxError::type_err(e, span, None))?;
```

---

## 6. Merge duplicate functions: `build_args_text` / `build_args_stream`

**File:** `crates/lx-desktop/src/voice_backend.rs`

**Step 1:** Find and delete both functions:

```rust
fn build_args_text(text: &str) -> Vec<&str> {
  let mut args = vec!["-p", text, "--output-format", "text", "--system-prompt", SYSTEM_PROMPT];
  if SESSION_CREATED.load(Ordering::Relaxed) {
    args.extend(["--resume", &SESSION_ID]);
  } else {
    args.extend(["--session-id", &SESSION_ID]);
  }
  args
}

fn build_args_stream(text: &str) -> Vec<&str> {
  let mut args = vec!["-p", text, "--output-format", "stream-json", "--verbose", "--include-partial-messages", "--system-prompt", SYSTEM_PROMPT];
  if SESSION_CREATED.load(Ordering::Relaxed) {
    args.extend(["--resume", &SESSION_ID]);
  } else {
    args.extend(["--session-id", &SESSION_ID]);
  }
  args
}
```

Replace with:

```rust
fn build_args<'a>(text: &'a str, format_args: &[&'a str]) -> Vec<&'a str> {
  let mut args = vec!["-p", text];
  args.extend_from_slice(format_args);
  args.extend(["--system-prompt", SYSTEM_PROMPT]);
  if SESSION_CREATED.load(Ordering::Relaxed) {
    args.extend(["--resume", &SESSION_ID]);
  } else {
    args.extend(["--session-id", &SESSION_ID]);
  }
  args
}
```

**Step 2:** Find:

```rust
    let args = build_args_text(text);
```

Replace with:

```rust
    let args = build_args(text, &["--output-format", "text"]);
```

**Step 3:** Find:

```rust
  let args = build_args_stream(text);
```

Replace with:

```rust
  let args = build_args(text, &["--output-format", "stream-json", "--verbose", "--include-partial-messages"]);
```

---

## 7. Field spreading: `PackageSection` into `Member`

**File:** `crates/lx-cli/src/manifest.rs`

**Step 1:** Find:

```rust
pub struct Member {
  pub name: String,
  pub version: Option<String>,
  pub dir: PathBuf,
  pub entry: Option<String>,
  pub description: Option<String>,
  pub license: Option<String>,
  pub authors: Option<Vec<String>>,
  pub lx: Option<String>,
  pub test_dir: String,
  pub test_pattern: String,
}
```

Replace with:

```rust
pub struct Member {
  pub pkg: PackageSection,
  pub dir: PathBuf,
  pub test_dir: String,
  pub test_pattern: String,
}
```

**Step 2:** Find:

```rust
    members.push(Member {
      name: pkg.name,
      version: pkg.version,
      dir: member_dir,
      entry: pkg.entry,
      description: pkg.description,
      license: pkg.license,
      authors: pkg.authors,
      lx: pkg.lx,
      test_dir: test.dir.unwrap_or_else(|| "tests/".into()),
      test_pattern: test.pattern.unwrap_or_else(|| "*.lx".into()),
    });
```

Replace with:

```rust
    members.push(Member {
      pkg,
      dir: member_dir,
      test_dir: test.dir.unwrap_or_else(|| "tests/".into()),
      test_pattern: test.pattern.unwrap_or_else(|| "*.lx".into()),
    });
```

**Step 3:** Update the `member_map` method (created in Violation 3). Find:

```rust
    self.members.iter().map(|m| (m.name.clone(), m.dir.clone())).collect()
```

Replace with:

```rust
    self.members.iter().map(|m| (m.pkg.name.clone(), m.dir.clone())).collect()
```

**Step 4 — `crates/lx-cli/src/listing.rs`:** Find:

```rust
    let entry_display = member.entry.as_deref().unwrap_or("(no entry)");
    let version = member.version.as_deref().unwrap_or("0.0.0");
```

Replace with:

```rust
    let entry_display = member.pkg.entry.as_deref().unwrap_or("(no entry)");
    let version = member.pkg.version.as_deref().unwrap_or("0.0.0");
```

**Step 5 — `crates/lx-cli/src/listing.rs`:** Find:

```rust
    let desc = member.description.as_deref().unwrap_or("");
```

Replace with:

```rust
    let desc = member.pkg.description.as_deref().unwrap_or("");
```

**Step 6 — `crates/lx-cli/src/listing.rs`:** Find:

```rust
    if let Some(ref lic) = member.license {
```

Replace with:

```rust
    if let Some(ref lic) = member.pkg.license {
```

**Step 7 — `crates/lx-cli/src/listing.rs`:** Find:

```rust
    if let Some(ref lx_ver) = member.lx {
```

Replace with:

```rust
    if let Some(ref lx_ver) = member.pkg.lx {
```

**Step 8 — `crates/lx-cli/src/listing.rs`:** Find:

```rust
    if let Some(ref authors) = member.authors
```

Replace with:

```rust
    if let Some(ref authors) = member.pkg.authors
```

**Step 9 — `crates/lx-cli/src/listing.rs`:** Find:

```rust
    println!("  {:<12} {:<7} {:>3} files  {:<14} {:>3} tests  {desc}{extra}", member.name, version, file_count, entry_display, test_count,);
```

Replace with:

```rust
    println!("  {:<12} {:<7} {:>3} files  {:<14} {:>3} tests  {desc}{extra}", member.pkg.name, version, file_count, entry_display, test_count,);
```

**Step 10 — `crates/lx-cli/src/testing.rs`:** Find:

```rust
    let found: Vec<_> = ws.members.iter().filter(|m| m.name == filter).collect();
```

Replace with:

```rust
    let found: Vec<_> = ws.members.iter().filter(|m| m.pkg.name == filter).collect();
```

**Step 11 — `crates/lx-cli/src/testing.rs`:** Find:

```rust
      eprintln!("available: {}", ws.members.iter().map(|m| m.name.as_str()).collect::<Vec<_>>().join(", "));
```

Replace with:

```rust
      eprintln!("available: {}", ws.members.iter().map(|m| m.pkg.name.as_str()).collect::<Vec<_>>().join(", "));
```

**Step 12a — `crates/lx-cli/src/testing.rs`:** Find:

```rust
      member_results.push((member.name.clone(), 0u32, 0u32, true));
```

Replace with:

```rust
      member_results.push((member.pkg.name.clone(), 0u32, 0u32, true));
```

**Step 12b — `crates/lx-cli/src/testing.rs`:** Find:

```rust
    member_results.push((member.name.clone(), result.passed, result.failed, false));
```

Replace with:

```rust
    member_results.push((member.pkg.name.clone(), result.passed, result.failed, false));
```

**Step 13 — `crates/lx-cli/src/main.rs`:** Find:

```rust
    if member.name == target {
      let entry = member.entry.as_deref().unwrap_or("main.lx");
```

Replace with:

```rust
    if member.pkg.name == target {
      let entry = member.pkg.entry.as_deref().unwrap_or("main.lx");
```

**Step 14 — `crates/lx-cli/src/fmt.rs`:** Find:

```rust
  for member in ws.members.iter().filter(|m| member_filter.is_none() || member_filter == Some(m.name.as_str())) {
```

Replace with:

```rust
  for member in ws.members.iter().filter(|m| member_filter.is_none() || member_filter == Some(m.pkg.name.as_str())) {
```

**Step 15 — `crates/lx-cli/src/fmt.rs`:** Find:

```rust
    eprintln!("{:<16} {} checked, {} formatted, {} failed", member.name, total, formatted_count, failed);
```

Replace with:

```rust
    eprintln!("{:<16} {} checked, {} formatted, {} failed", member.pkg.name, total, formatted_count, failed);
```

**Step 16 — `crates/lx-cli/src/check.rs`:** Find:

```rust
    let found: Vec<_> = ws.members.iter().filter(|m| m.name == filter).collect();
```

Replace with:

```rust
    let found: Vec<_> = ws.members.iter().filter(|m| m.pkg.name == filter).collect();
```

**Step 17 — `crates/lx-cli/src/check.rs`:** Find:

```rust
      eprintln!("available: {}", ws.members.iter().map(|m| m.name.as_str()).collect::<Vec<_>>().join(", "));
```

Replace with:

```rust
      eprintln!("available: {}", ws.members.iter().map(|m| m.pkg.name.as_str()).collect::<Vec<_>>().join(", "));
```

**Step 18 — `crates/lx-cli/src/check.rs`:** Find:

```rust
      println!("{:<16} {total_files} checked, {member_fixed} fixed, {member_err} remaining errors — {status}", member.name);
    } else if member_parse_err > 0 {
      println!("{:<16} {total_files} checked, {member_err} type errors, {member_parse_err} parse errors — {status}", member.name);
    } else {
      println!("{:<16} {total_files} checked, {member_err} errors — {status}", member.name);
```

Replace with:

```rust
      println!("{:<16} {total_files} checked, {member_fixed} fixed, {member_err} remaining errors — {status}", member.pkg.name);
    } else if member_parse_err > 0 {
      println!("{:<16} {total_files} checked, {member_err} type errors, {member_parse_err} parse errors — {status}", member.pkg.name);
    } else {
      println!("{:<16} {total_files} checked, {member_err} errors — {status}", member.pkg.name);
```

---

## 8. Merge duplicate extract functions: `extract_mermaid` / `extract_echart_json`

**File:** `crates/lx/src/stdlib/diag/mod.rs`

Find:

```rust
pub fn extract_mermaid<P>(program: &Program<P>) -> String {
  let mut walker = Walker::with_arena(&program.arena);
  let _ = walk_program(&mut walker, program);
  to_mermaid(&walker.into_graph())
}

pub fn extract_echart_json<P>(program: &Program<P>) -> String {
  let mut walker = Walker::with_arena(&program.arena);
  let _ = walk_program(&mut walker, program);
  let graph = walker.into_graph();
  graph_to_echart_json(&graph)
}
```

Replace with:

```rust
fn extract_graph_from_program<P>(program: &Program<P>) -> Graph {
  let mut walker = Walker::with_arena(&program.arena);
  let _ = walk_program(&mut walker, program);
  walker.into_graph()
}

pub fn extract_mermaid<P>(program: &Program<P>) -> String {
  to_mermaid(&extract_graph_from_program(program))
}

pub fn extract_echart_json<P>(program: &Program<P>) -> String {
  graph_to_echart_json(&extract_graph_from_program(program))
}
```

# Execution Order

1. Violation 1 — standalone
2. Violation 4 — standalone
3. Violation 5 — standalone, affects only diag files
4. Violation 8 — affects same file as 5, apply after 5
5. Violation 6 — standalone, affects only voice_backend.rs
6. Violation 2 — standalone, affects only lx-mobile
7. Violation 3 — must apply before Violation 7
8. Violation 7 — depends on Violation 3 being done first

# Verification

After all changes, run `just diagnose`.
