# GSD-2: Headless Mode and Parallel Orchestration

## Headless Mode

`gsd headless` runs any `/gsd` command without a TUI. Designed for CI pipelines, cron jobs, and scripted automation.

```bash
# Auto mode in CI
gsd headless --timeout 600000

# Create and execute milestone
gsd headless new-milestone --context spec.md --auto

# One unit at a time (cron-friendly)
gsd headless next

# Instant JSON snapshot (no LLM, ~50ms)
gsd headless query

# Force a specific pipeline phase
gsd headless dispatch plan
```

### Architecture

```
headless.ts (parent process)
  ├── parseHeadlessArgs()
  ├── loadContext() (file, stdin, inline text)
  ├── bootstrapGsdProject() (create .gsd/ structure)
  ├── RpcClient (spawns child process in RPC mode)
  │   ├── Child runs gsd --mode rpc
  │   ├── Event stream: JSONL events
  │   └── Stdin: commands and responses
  ├── Event listener (classify events)
  ├── Auto-responder (handle UI requests)
  ├── Completion detection
  └── Exit code
```

### HeadlessOptions

```typescript
interface HeadlessOptions {
  timeout: number;                    // default 300s
  json: boolean;                      // output as JSONL
  model?: string;                     // LLM override
  command: string;                    // "auto", "new-milestone", "status", etc.
  commandArgs: string[];
  context?: string;                   // file path or '-' for stdin
  contextText?: string;               // inline text
  auto?: boolean;                     // chain into auto after creation
  verbose?: boolean;                  // show tool calls
  maxRestarts?: number;               // auto-restart on crash (default 3)
  supervised?: boolean;               // forward requests to orchestrator
  responseTimeout?: number;           // supervised response timeout (default 30s)
  answers?: string;                   // path to pre-supplied answers JSON
  eventFilter?: Set<string>;          // JSONL output event type filter
}
```

### Auto-Response

UI requests are automatically handled:
- `select` → first option
- `confirm` → true
- `input` → empty string
- `editor` → prefill content (if provided)
- `notify`, `setStatus`, `setWidget`, `setTitle` → empty response

### Completion Detection

- Terminal notifications: "Auto-mode stopped..." or "Step-mode stopped..."
- Blocked detection: "Auto-mode stopped (Blocked: ...)"
- Milestone-ready: regex "milestone M\d+.*ready"
- Quick commands: complete on `agent_end` (status, queue, history, hooks, export, stop, etc.)
- Idle timeouts: 15s normal, 120s for new-milestone

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Complete |
| 1 | Error or timeout |
| 2 | Blocked |

### Auto-Restart on Crash

Exponential backoff: 5s → 10s → 15s... capped at 30s. Default max 3 attempts.

### Answer Injection

Pre-supply responses to interactive prompts via `--answers <file>`:

```typescript
interface AnswerFile {
  questions?: Record<string, string | string[]>;  // question ID → answer(s)
  secrets?: Record<string, string>;                // env var keys → values
  defaults?: { strategy?: 'first_option' | 'cancel' };
}
```

The `AnswerInjector`:
1. Observes `ask_user_questions` tool execution to extract question metadata
2. Intercepts `extension_ui_request` events before auto-responder
3. Maps question headers/titles to pre-supplied answers
4. Validates against options
5. Tracks stats: answered, defaulted, secrets provided, unused warnings

### Supervised Mode

For external orchestrators: forward interactive requests via stdin/stdout.

Reads JSON lines from stdin:
- `extension_ui_response` → forwards to child
- `prompt`, `steer`, `follow_up` → sends to RPC client methods

### Headless Query

`gsd headless query` — instant state export without LLM session (~50ms):

```json
{
  "state": { /* GSDState from disk */ },
  "next": { /* dry-run dispatch preview */ },
  "cost": { /* aggregated parallel worker costs */ }
}
```

### MCP Server Mode

`gsd --mode mcp` exposes GSD tools over stdin/stdout via Model Context Protocol. External AI clients can drive GSD programmatically.

## Parallel Milestone Orchestration

Run multiple milestones simultaneously. Each gets its own worker process and worktree.

### Configuration

```yaml
parallel:
  enabled: false              # Master toggle (off by default)
  max_workers: 2              # 1-4
  budget_ceiling: 100.00      # Aggregate budget
  merge_strategy: per-slice   # per-slice or per-milestone
  auto_merge: confirm         # auto / confirm / manual
```

### Eligibility Analysis

Before launching parallel workers, checks:
- Milestone completion status (not already done)
- Dependency satisfaction (declared dependencies met)
- File overlap analysis (independent milestones only)

### Worker Isolation

Each worker gets:
- Separate process
- Separate git worktree (`.gsd/worktrees/<MID>/`)
- Separate branch (`milestone/<MID>`)
- Separate context windows
- Separate metrics tracking
- Independent crash recovery

### File-Based IPC

Coordination via `.gsd/parallel/`:
- `<MID>.status.json` — worker state (running, paused, complete, error)
- `<MID>.signal.json` — control signals (stop, pause, resume)
- Atomic writes prevent corruption

### Merge Strategy

- `per-slice` — merge to main after each slice completes
- `per-milestone` — merge to main only after entire milestone completes
- Sequential merge order by default
- Auto-resolve `.gsd/` files; stop on code conflicts

### Budget Management

Aggregate cost tracking across all parallel workers. Budget enforcement applies to combined spending.

### Commands

| Command | Action |
|---------|--------|
| `/gsd parallel start` | Launch parallel workers |
| `/gsd parallel status` | Worker health dashboard |
| `/gsd parallel stop` | Graceful shutdown all workers |
| `/gsd parallel pause` | Pause all workers |
| `/gsd parallel resume` | Resume paused workers |
| `/gsd parallel merge` | Trigger merge for completed workers |

### Doctor Integration

`/gsd doctor` detects:
- Stale parallel sessions (no heartbeat)
- Orphaned worktrees
- Budget overruns across workers

## CI/CD Pipeline (ci-cd-pipeline.md)

### Three-Stage Promotion

```
Dev (every PR) → Test (auto if green) → Prod (manual approval)
```

### CI Gates (ci.yml)

1. `no-gsd-dir` — prevents .gsd/ from being committed
2. `build` (ubuntu) — npm ci, build, typecheck, validate-pack, unit + integration tests
3. `windows-portability` — cross-platform build verification

### Dev Build

Every merged PR → `npx gsd-pi@dev` with `-next.{commit}` version stamp.

### Platform Binaries (build-native.yml)

5-platform matrix (macOS ARM64/x64, Linux x64/ARM64, Windows x64):
1. Compile Rust to target
2. Upload as artifact
3. Publish to npm (`@gsd-build/engine-{platform}`)
4. Post-publish smoke test

### Version Strategy

- Dev: `2.27.0-dev.a3f2c1b`
- Test: `2.27.0-next.1`
- Prod: `2.27.0`
- Old dev versions cleaned weekly (30-day retention)

### Test Tiers

| Tier | Trigger | Method |
|------|---------|--------|
| Smoke | Every CI run | `test-help.ts`, `test-init.ts`, `test-version.ts` |
| Fixture | Every CI run | Replay-based testing against fixed LLM responses |
| Live | Manual (`GSD_LIVE_TESTS=1`) | Real API calls (Anthropic, OpenAI) |
| Unit/Integration | Every CI run | Extension unit tests, native module tests |
