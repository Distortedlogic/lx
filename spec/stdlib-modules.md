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
