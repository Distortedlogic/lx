# Standard Library — Module Details

Detailed API for each core module. See [stdlib.md](stdlib.md) for conventions, built-in functions, and the module overview table. Agent ecosystem modules (std/agent, std/mcp, std/ctx, std/md) are in [stdlib-agents.md](stdlib-agents.md).

## std/fs

```
read path              -- Str ^ IoErr (errors on non-UTF-8)
read_bytes path        -- Bytes ^ IoErr
read_lossy path        -- Str ^ IoErr (U+FFFD for invalid bytes)
write path content     -- () ^ IoErr
write_bytes path bytes -- () ^ IoErr
append path content    -- () ^ IoErr
walk dir               -- lazy [Str] of file paths (recursive)
stat path              -- {size: Int  modified: Time  is_dir: Bool} ^ IoErr
exists? path           -- Bool
mkdir path             -- () ^ IoErr (creates parents)
rm path                -- () ^ IoErr
copy src dst           -- () ^ IoErr
move src dst           -- () ^ IoErr
tmp_dir ()             -- Str (path to new temp directory)
open path              -- Handle ^ IoErr (for streaming reads/writes)
close handle           -- () ^ IoErr
read_lines path        -- lazy [Str] ^ IoErr (line-by-line streaming)
glob pattern           -- lazy [Str] of matching file paths
```

`read_lines` returns a list of lines. `glob` matches shell-style patterns: `fs.glob "src/**/*.lx"`.

`Handle` is an opaque type — it cannot be destructured or inspected by user code. Created by `open`, consumed by `close` and streaming read/write operations.

## std/http

```
get url                -- {status: Int  headers: %{Str: Str}  body: Str} ^ HttpErr
post url body          -- same response type
put url body           -- same response type
delete url             -- same response type
request opts           -- full control via options record:
                       --   {method: Str  url: Str  headers: %{}  body: Str  timeout: Int}
```

## std/json

```
parse str              -- a ^ ParseErr (returns dynamic value)
encode val             -- Str
encode_pretty val      -- Str (indented)
```

Parsed JSON values support field access with `."key"`:

```
data = json.parse raw ^
data."users".0."name"
```

JSON types map to lx types: objects become maps (`%{}`), arrays become lists, strings/numbers/booleans map directly, `null` becomes `None`.

## std/time

```
now ()                 -- Time (current timestamp)
elapsed start          -- Duration since start
sleep dur              -- () (pause execution)
sec n                  -- Duration: n seconds
ms n                   -- Duration: n milliseconds
min n                  -- Duration: n minutes
format fmt time        -- Str
parse fmt str          -- Time ^ ParseErr
timeout dur            -- () (completes after dur; for use in `sel` blocks)
to_ms dur              -- Int: duration in milliseconds
to_sec dur             -- Int: duration in seconds (truncated)
to_min dur             -- Int: duration in minutes (truncated)
```

Duration values are created by `sec`, `ms`, `min`. They compose with arithmetic: `time.sec 5 + time.ms 500`. `Duration` is an opaque type — created by `sec`/`ms`/`min`, consumed by `sleep`, `timeout`, and arithmetic. Cannot be destructured.

`time.timeout` takes a `Duration`. The built-in `timeout n` takes seconds directly (shorthand for `time.timeout (time.sec n)`):

```
sel {
  http.get url -> it
  timeout 5    -> Err "timed out"
}
```

Use `time.timeout` when you need sub-second precision: `time.timeout (time.ms 500)`.

## std/math

```
abs n          -- absolute value (Int or Float)
sqrt x         -- Float
pow base exp   -- Float
log x          -- natural log
log2 x         -- base-2 log
log10 x        -- base-10 log
sin x          -- radians
cos x          -- radians
tan x          -- radians
pi             -- 3.14159...
e              -- 2.71828...
floor x        -- Float -> Int
ceil x         -- Float -> Int
round x        -- Float -> Int (round half to even)
trunc x        -- Float -> Int (truncate toward zero)
clamp lo hi x  -- constrain x to [lo hi]
safe_div a b   -- Result Int Str: returns Err on zero divisor
safe_mod a b   -- Result Int Str: returns Err on zero divisor
min a b        -- smaller of two values (works on any comparable type)
max a b        -- larger of two values
```

`safe_div` and `safe_mod` return `Result` instead of panicking on zero. Use these in data pipelines where zero divisors are expected input.

## std/env

```
args                   -- [Str]: command-line arguments
get key                -- Maybe Str
set key val            -- () (blocked in --sandbox mode)
vars ()                -- %{Str: Str}: all environment variables
exit code              -- ! (never returns)
cwd ()                 -- Str: current working directory
```

To require an env var: `env.get "KEY" | require "KEY must be set" ^`.

## std/re

```
is_match pattern s       -- Bool: does it match?
match pattern s          -- Maybe {text start end}: first match
replace pattern new s    -- replace first match
replace_all pattern new s -- replace all matches
split pattern s          -- split on matches -> [Str]
find_all pattern s       -- [Str]: all match texts
```

Patterns can be regex literals (`r/\d+/`) or strings (`"\\d+"`). Regex literals are preferred — no double-escaping. Flags: `r/pattern/i` (case insensitive), `m` (multiline), `s` (dotall), `x` (extended).

## std/blackboard

Concurrent shared workspace for multi-agent collaboration within `par` blocks. Unlike `ctx` (single-owner, immutable), a blackboard supports concurrent reads and writes from multiple agents.

```
create ()                 -- Board (empty blackboard)
read key board            -- Maybe a (read a key)
write key val board       -- () (write a key, last-write-wins)
watch key callback board  -- WatchId (invoke callback on key change)
unwatch id board          -- ()
keys board                -- [Str] (all keys)
snapshot board            -- %{Str: a} (atomic snapshot of all entries)
```

`Board` is an opaque type backed by a concurrent map. Thread-safe for use inside `par`/`pmap` blocks.

## std/events

Topic-based pub/sub event bus. Decouples event producers from consumers — publishers don't know who's listening.

```
create ()                      -- Bus (empty event bus)
publish bus topic msg          -- () (broadcast to all subscribers of topic)
subscribe bus topic handler    -- SubId (register handler for topic)
unsubscribe bus id             -- ()
topics bus                     -- [Str] (all active topics)
```

`Bus` is an opaque type. Handlers are `(msg) -> ()` functions invoked synchronously in subscription order.

## std/budget

Cumulative resource/cost accounting with projection and adaptive strategy.

```
create limits                  -- Budget (e.g. {tokens: 50000 api_calls: 20})
create limits opts             -- Budget with custom thresholds ({tight_at: 60 critical_at: 90})
spend b costs                  -- () ^ BudgetErr (deduct from budget)
remaining b                    -- {tokens: Int  api_calls: Int  ...}
used b                         -- {tokens: Int  api_calls: Int  ...}
used_pct b                     -- {tokens: Float  api_calls: Float  ...}
status b                       -- :comfortable | :tight | :critical | :exceeded
project b opts                 -- {projected_total  will_exceed  headroom}
slice b limits                 -- Budget (sub-budget drawing from parent)
```

`Budget` is a mutable, thread-safe type. `slice` creates child budgets sharing parent counters. Spec: `spec/agents-budget.md`.

## std/reputation

Cross-interaction agent quality tracking with EWMA scoring.

```
load path                      -- Rep ^ IoErr (load or create reputation store)
record rep entry               -- () ^ IoErr (record outcome: {agent task_type passed score})
score rep agent task_type      -- {score ewma total recent trend} ^ RepErr
best_for rep task_type         -- {agent score} ^ RepErr
best_for rep task_type opts    -- with {min_history: 5}
rank rep task_type             -- [{agent score}] (sorted best-first)
```

`Rep` is an opaque file-backed type. EWMA per (agent, task_type) pair. Configurable `decay_half_life`. Spec: `spec/agents-reputation.md`.

## std/trace extensions

**Implemented:** `trace.improvement_rate N store` computes improvement rate over last N progress spans (spans named "progress" with score field). Returns `{avg_delta recent_delta trend samples}`. Trend is one of "improving"/"steady"/"diminishing"/"plateau"/"regressing"/"insufficient". `trace.should_stop {min_delta: Float window: Int} store` returns true if all deltas in the last `window` score changes are at or below `min_delta`. Spec: `spec/agents-progress.md`.

**Planned:** Causal chain queries via parent-child span trees. `trace.chain` walks from failure to root cause.

## std/skill

Runtime registry and discovery for `Skill` declarations.

```
registry skills                -- Registry (from list of Skill values)
list registry                  -- [{name description input output tags}] (metadata only)
get registry name              -- {name description input output requires tags} ^ SkillErr
match registry prompt          -- {name score reason} ^ SkillErr (keyword matching)
match_semantic registry prompt -- {name score reason} ^ SkillErr (LLM-based matching)
run registry name input        -- Result output SkillErr (validated execution)
compose registry names         -- Fn (chained pipeline, output->input type-checked)
```

`Registry` is an opaque type holding Skill values. `list` returns metadata only (LLM-safe). `compose` chains skills into a pipeline. Spec: `spec/agents-skill.md`.

## std/durable

Workflow persistence management. Functions: `status`, `resume`, `cancel`, `list`, `cleanup`. Manages workflows created by `durable` expression. Storage via `DurableBackend` trait on RuntimeCtx. Default: filesystem JSON. Spec: `spec/agents-durable.md`.

## std/context

Context capacity management. Tracks working memory, pressure, eviction, compaction. Distinct from `std/ctx` (persistent storage) and `std/memory` (tiered facts). Spec: `spec/agents-context-capacity.md`.

```
create opts                    -- Window (e.g. {capacity: 200000})
add win item                   -- () (item: {key content tokens priority?})
usage win                      -- {used capacity available pct}
pressure win                   -- :low | :moderate | :high | :critical
estimate content               -- Int (approximate token count)
on_pressure win level callback -- () (fire callback when pressure >= level)
pin win key / unpin win key    -- ()
evict win strategy             -- () (:oldest | :lowest_priority | :largest)
evict_until win strategy opts  -- () (repeat until {target_pct} reached)
compact win strategy           -- () (:summarize | :drop_examples | :truncate)
items win                      -- [{key tokens priority pinned added_at}]
get win key                    -- Maybe {key content tokens priority pinned}
remove win key / clear win     -- ()
```

## std/prompt

Typed composable prompt assembly. Immutable builder — all functions return new Prompt. Spec: `spec/agents-prompt.md`.

```
create ()                      -- Prompt (empty builder)
system text p                  -- Prompt (set system section)
section name content p         -- Prompt (add named section)
constraint text p              -- Prompt (add constraint)
instruction text p             -- Prompt (add instruction)
example pair p                 -- Prompt (add {input output} few-shot example)
compose [p1 p2 ...]           -- Prompt (merge prompts, sections concatenate)
render p                       -- Str (render to final string)
render_within p budget         -- Str (trim to fit token budget)
estimate p                     -- Int (approximate token count)
sections p                     -- [Symbol] (list section names)
without p name                 -- Prompt (remove section)
```

## std/strategy

Strategy memory — approach outcomes per problem type, cross-session learning. File-backed. Spec: `spec/agents-strategy.md`.

```
create path                    -- Store ^ IoErr (file-backed JSON)
record store entry             -- () ^ IoErr ({problem approach score context?})
best_for store problem         -- {approach avg_score count trend} ^ StratErr
rank store problem             -- [{approach avg_score count trend}]
suggest store query            -- {approach confidence reason} ({problem context})
history store problem approach -- [{score context timestamp}]
adapt store problem            -- {approach mode} (:exploit or :explore)
prune store opts               -- () ({older_than: days} or {min_count below_score})
export store / import store data -- Record / ()
```

## Eliminated Modules (merged into existing features)

- **std/decide** → Decision metadata stored as trace spans with structured metadata fields. Query via `trace.query {type: "decision"}`.
- **std/causal** → Parent-child span trees in `std/trace`. `trace.chain` walks from failure to root cause.
- **std/agent_test** → `agent.mock` + `agent.mock_calls` + `agent.mock_assert_called` helpers in `std/agent`. Test scenarios are regular lx code.
