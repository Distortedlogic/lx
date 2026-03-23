# GSD-2: Configuration Reference

## Preferences System

Preferences live in `~/.gsd/preferences.md` (global) or `.gsd/preferences.md` (project-local). YAML frontmatter format. Managed interactively with `/gsd prefs`.

### Full Preferences Schema

```yaml
---
version: 1

# Model Selection (per-phase)
models:
  research: claude-sonnet-4-6
  planning:
    model: claude-opus-4-6
    fallbacks:
      - openrouter/z-ai/glm-5
  execution: claude-sonnet-4-6
  execution_simple: claude-haiku-4-5     # Light tasks
  completion: claude-sonnet-4-6
  subagent: claude-sonnet-4-6

# Token Optimization
token_profile: balanced          # budget / balanced / quality

# Phase Skipping
skip_research: false
skip_reassess: false
skip_slice_research: true        # balanced default
require_slice_discussion: false
skip_validation: false

# Skill Discovery
skill_discovery: suggest         # auto / suggest / off
always_use_skills: []
prefer_skills: []
avoid_skills: []
skill_rules: []

# Auto Supervisor Timeouts
auto_supervisor:
  soft_timeout_minutes: 20       # Warns LLM to wrap up
  idle_timeout_minutes: 10       # Detects stalls
  hard_timeout_minutes: 30       # Pauses auto mode

# Budget
budget_ceiling: 50.00            # USD
budget_enforcement: pause        # warn / pause / halt

# Verification
verification_commands:
  - npm run lint
  - npm run test
verification_auto_fix: true
verification_max_retries: 2

# Git
git:
  isolation: worktree            # worktree / branch / none
  manage_gitignore: true
  auto_push: false
  push_branches: false
  pre_merge_check: false
  commit_type: conventional      # conventional commit format
  main_branch: main
  merge_strategy: squash         # squash / merge
  commit_docs: true
  worktree_post_create: ""       # shell command after worktree creation
  auto_pr: false                 # auto-create PR on milestone completion
  pr_target_branch: main

# Notifications
notifications:
  completion: true
  error: true
  budget: true
  milestone: true
  attention: true

# Remote Questions (headless integration)
remote:
  provider: discord              # discord / slack / telegram
  channel: "#gsd-headless"
  channel_id: "123456789"
  timeout_minutes: 15            # 1-30
  poll_interval_seconds: 10      # 2-30

# Post-Unit Hooks
post_unit_hooks:
  - name: "run-tests"
    trigger: after_task
    command: "npm run test"
    on_failure: pause            # pause / warn / ignore

# Pre-Dispatch Hooks
pre_dispatch_hooks:
  - name: "check-deps"
    trigger: before_execute
    action: modify               # modify / skip / replace

# Dynamic Model Routing
dynamic_routing:
  enabled: false
  tier_models:
    light: [claude-haiku-4-5]
    standard: [claude-sonnet-4-6]
    heavy: [claude-opus-4-6]
  escalate_on_failure: true
  budget_pressure: true
  cross_provider: false

# Parallel Orchestration
parallel:
  enabled: false
  max_workers: 2                 # 1-4
  budget_ceiling: 100.00
  merge_strategy: per-slice      # per-slice / per-milestone
  auto_merge: confirm            # auto / confirm / manual

# Reports
auto_report: true                # HTML generation after milestone

# Context
context_selection: full          # full / smart (TF-IDF chunking for >3KB)

# Workflow Mode
mode: solo                       # solo / team
---
```

### Merge Hierarchy

Later overrides earlier:

```
Token profile defaults
  → Explicit preferences (scalars override, arrays/objects merge)
    → Global (~/.gsd/preferences.md)
      → Project (.gsd/preferences.md)
        → Mode defaults (solo/team)
          → Environment variables
```

### Workflow Mode Defaults

| Setting | Solo | Team |
|---------|------|------|
| auto_push | true | false |
| merge_strategy | squash | squash |
| milestone IDs | simple (M001) | unique |
| push_branches | false | true |
| pre_merge_check | false | true |
| commit_docs | true | true |

## Key Configuration Patterns

### Per-Phase Model Selection

Research can use a cheap model (Haiku) while planning uses an expensive one (Opus). Execution gets the default (Sonnet). Each phase independently configurable.

### Model Fallbacks

```yaml
models:
  planning:
    model: claude-opus-4-6
    fallbacks:
      - openrouter/z-ai/glm-5
      - claude-sonnet-4-6
```

On failure, tries each fallback in order.

### Verification Commands

```yaml
verification_commands:
  - npm run lint
  - npm run test
  - npm run typecheck
verification_auto_fix: true
verification_max_retries: 2
```

Run automatically after each task. On failure, auto-fix retries up to max. If still failing, pause.

### Git Settings by Use Case

**Solo developer:**
```yaml
git:
  isolation: worktree
  auto_push: true
  merge_strategy: squash
```

**Team with PR workflow:**
```yaml
git:
  isolation: worktree
  auto_push: false
  push_branches: true
  pre_merge_check: true
  auto_pr: true
  pr_target_branch: main
```

**Minimal (no isolation):**
```yaml
git:
  isolation: none
  auto_push: false
```

### Remote Questions Setup

For headless auto-mode with human interaction via chat platforms:

```yaml
remote:
  provider: discord
  channel_id: "123456789"
  timeout_minutes: 15
  poll_interval_seconds: 10
```

Discord: rich embeds. Slack: Block Kit. Telegram: formatted messages.
Response formats: reactions (single choice), replies (number/text/semicolons for multi).

### .gitignore for Teams

Track planning artifacts, ignore runtime state:

```gitignore
# Track these
!.gsd/milestones/
!.gsd/PROJECT.md
!.gsd/DECISIONS.md
!.gsd/REQUIREMENTS.md
!.gsd/QUEUE.md
!.gsd/preferences.md

# Ignore these
.gsd/STATE.md
.gsd/auto.lock
.gsd/completed-units.json
.gsd/metrics.json
.gsd/routing-history.json
.gsd/gsd.db
.gsd/worktrees/
.gsd/activity/
.gsd/runtime/
.gsd/parallel/
.gsd/reports/
```

## Environment Variables

| Variable | Purpose |
|----------|---------|
| `PI_PACKAGE_DIR` | Pi SDK package directory (set by loader.ts) |
| `PI_SKIP_VERSION_CHECK` | Disable Pi's update check |
| `GSD_CODING_AGENT_DIR` | Path to ~/.gsd/agent/ |
| `GSD_VERSION` | Current GSD version |
| `GSD_BIN_PATH` | Absolute path to loader |
| `GSD_WORKFLOW_PATH` | Path to GSD-WORKFLOW.md |
| `GSD_BUNDLED_EXTENSION_PATHS` | Discovered extension entry points |
| `GSD_MILESTONE_LOCK` | Lock parallel worker to specific milestone |
| `GSD_LIVE_TESTS` | Enable live API tests (1 to enable) |
| `GSD_FIXTURE_MODE` | Fixture test mode (record/playback) |
| `HTTP_PROXY` / `HTTPS_PROXY` | Proxy configuration |

## Directory Constants

```
~/.gsd/                    Application root
~/.gsd/agent/              Agent resources (extensions, agents, skills)
~/.gsd/agent/auth.json     API credentials
~/.gsd/agent/models.json   Model registry
~/.gsd/sessions/           Session JSONL files (per-directory scoped)
~/.gsd/preferences.md      Global preferences
```
