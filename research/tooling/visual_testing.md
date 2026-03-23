# Visual Testing with Claude Code CLI

Research on testing Dioxus UI by capturing screenshots and sending them to Claude Code CLI for review against specs.

## How Dioxus Tests Today

Dioxus uses a multi-layer testing approach:

| Layer | Tool | Location |
|---|---|---|
| Unit (VirtualDom) | Rust `#[test]` | `packages/core/tests/` (~60 files) |
| Hooks/signals | `tokio::test` | `packages/hooks/tests/`, `packages/signals/tests/` |
| SSR | `dioxus_ssr::render()` | `packages/ssr/tests/` |
| Desktop headless | Rust + JS eval | `packages/desktop/headless_tests/` |
| E2E | Playwright | `packages/playwright-tests/` (25+ spec files) |

Playwright config: `packages/playwright-tests/playwright.config.js` — 50-minute timeouts for Rust compilation, single worker on CI with 2 retries, HTML report generation.

No custom test framework — standard Rust tests + Playwright. No mocking libraries; tests use real components and VirtualDom.

### VirtualDom Unit Test Pattern

```rust
#[test]
fn test_component() {
    let mut dom = VirtualDom::new(|| rsx! { div { "hello!" } });
    let edits = dom.rebuild_to_vec();
    // assert on mutations
}
```

### Async/Hook Test Pattern

```rust
#[tokio::test]
async fn effects_rerun() {
    let mut dom = VirtualDom::new_with_props(app, props);
    dom.rebuild_in_place();
    dom.wait_for_work().await;
    // assert on state
}
```

### Playwright E2E Pattern

```javascript
test("button click", async ({ page }) => {
    await page.goto("http://localhost:9990");
    const button = page.locator("button.increment-button");
    await button.click();
    await expect(page.locator("#main")).toContainText("hello axum! 1");
});
```

## Visual Testing Architecture

Use Playwright for screenshot capture, Rust for orchestration, Claude Code CLI for visual review.

```
Rust test binary (orchestrator)
  ├── spawns dioxus app (dx serve)
  ├── runs playwright to capture screenshots → saves to disk
  └── for each screenshot:
        spawns: claude -p "Read /tmp/screenshots/dashboard.png and ..."
              --max-turns 3
        parses stdout for pass/fail
```

### Why Claude Code CLI instead of Claude Vision API

- Claude Code's `Read` tool natively views images (PNG, JPG, etc.)
- Already have subprocess patterns in `apps/workflow/mcp/src/audit/pipeline.rs`
- No API key management needed — uses existing claude auth
- Can leverage Claude Code's file system access to read specs alongside screenshots

### Why Playwright instead of Rust browser crates

Rust crates exist (`chromiumoxide`, `headless_chrome`, `fantoccini`, `thirtyfour`) but:
- None are in the `reference/` directory
- Less mature than Playwright
- Playwright is purpose-built for this and already used by Dioxus

Use Playwright for capture only (thin JS script), Rust for everything else.

## Implementation

### 1. Playwright Screenshot Capture Script

```javascript
// tests/visual/capture.js
const { chromium } = require("@playwright/test");

const routes = JSON.parse(process.argv[2]);
const outDir = process.argv[3];

(async () => {
  const browser = await chromium.launch();
  const page = await browser.newPage({ viewport: { width: 1280, height: 720 } });

  for (const route of routes) {
    await page.goto(route.url);
    if (route.setup) await page.evaluate(route.setup);
    await page.waitForLoadState("networkidle");
    await page.screenshot({
      path: `${outDir}/${route.name}.png`,
      fullPage: route.fullPage ?? false,
    });
  }

  await browser.close();
})();
```

### 2. Specs as TOML Files

One per screen, stored in `specs/`:

```toml
# specs/dashboard.toml
name = "dashboard"
route = "http://localhost:8080/"
reference = "specs/reference/dashboard.png"

expectation = """
- Nav bar at top with logo left-aligned, user avatar right-aligned
- Sidebar with links: Home, Tasks, Settings (Home should be active/highlighted)
- Main area shows a data table with columns: Name, Status, Updated
- Status column uses colored badges (green=done, yellow=pending, red=error)
- At least 3 rows visible
"""
```

### 3. Rust Orchestrator

```rust
use std::process::Command;
use std::path::{Path, PathBuf};

struct VisualSpec {
    name: String,
    route: String,
    expectation: String,
    reference: Option<PathBuf>,
}

struct ReviewResult {
    pass: bool,
    issues: Vec<String>,
}

fn capture_screenshots(specs: &[VisualSpec], out_dir: &Path) -> Result<(), anyhow::Error> {
    let routes: Vec<serde_json::Value> = specs
        .iter()
        .map(|s| serde_json::json!({ "name": s.name, "url": s.route }))
        .collect();

    let status = Command::new("node")
        .arg("tests/visual/capture.js")
        .arg(serde_json::to_string(&routes)?)
        .arg(out_dir)
        .status()?;

    anyhow::ensure!(status.success(), "playwright capture failed");
    Ok(())
}

fn review_screenshot(screenshot: &Path, spec: &VisualSpec) -> Result<ReviewResult, anyhow::Error> {
    let mut prompt = format!(
        "Read the image at {} and evaluate it against this spec:\n\n{}\n\n",
        screenshot.display(),
        spec.expectation
    );

    if let Some(ref reference) = spec.reference {
        prompt.push_str(&format!(
            "Also read the reference image at {} and compare.\n\n",
            reference.display()
        ));
    }

    prompt.push_str(
        "Respond ONLY with JSON: {\"pass\": bool, \"issues\": [\"...\"]}\n\
         Only fail for meaningful visual/functional differences."
    );

    let output = Command::new("claude")
        .arg("-p")
        .arg(&prompt)
        .arg("--max-turns")
        .arg("3")
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;
    let result: serde_json::Value = extract_json(&stdout)?;

    Ok(ReviewResult {
        pass: result["pass"].as_bool().unwrap_or(false),
        issues: result["issues"]
            .as_array()
            .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default(),
    })
}
```

### 4. Test Entry Point

```rust
#[test]
fn visual_review() {
    let specs = load_specs("specs/");
    let out_dir = tempfile::tempdir().unwrap();

    capture_screenshots(&specs, out_dir.path()).unwrap();

    let mut failures = vec![];
    for spec in &specs {
        let screenshot = out_dir.path().join(format!("{}.png", spec.name));
        let result = review_screenshot(&screenshot, spec).unwrap();
        if !result.pass {
            failures.push((spec.name.clone(), result.issues));
        }
    }

    assert!(failures.is_empty(), "Visual failures: {failures:#?}");
}
```

## Existing Claude CLI Patterns in This Repo

From `apps/workflow/mcp/src/audit/pipeline.rs`:

```rust
let mut child = tokio::process::Command::new("claude")
    .arg("-p")
    .arg(&prompt)
    .arg("--mcp-config")
    .arg(&config_path)
    .env("PIPELINE_ID", pipeline_id)
    .spawn()?;
```

Known CLI flags: `-p` (non-interactive prompt), `--mcp-config`, `--system-prompt`, `--max-turns`, `--allowedTools`, `--model`.

## Cost and Speed

| Lever | Approach |
|---|---|
| Cheap dev runs | `--model haiku` on the claude invocation |
| CI gate | Default (sonnet) for accuracy |
| Speed | Run reviews in parallel with `tokio::process::Command` |
| Token cost | Crop to component under test, not full page |
| Skip unchanged | Hash screenshots, skip if unchanged from last run |

## Spec Format Options

| Format | When to use |
|---|---|
| Text description | Early development, no designs yet |
| Reference screenshot | Regression testing against known-good state |
| Figma export / design image | Comparing implementation vs design |
| All three combined | Maximum coverage |
