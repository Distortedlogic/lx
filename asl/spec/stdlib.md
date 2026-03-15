# Standard Library

Conventions, built-in functions, and core module reference.

## Conventions

**Data-last arguments** — every function takes the primary data argument last, enabling pipes:

```
map f xs            -- xs | map f
filter pred xs      -- xs | filter pred
fold init f xs      -- xs | fold init f
replace old new str -- str | replace old new
sort_by key xs      -- xs | sort_by key
take n xs           -- xs | take n
flat_map f xs       -- xs | flat_map f
partition pred xs   -- xs | partition pred
```

**Predicate suffix `?`** — functions returning `Bool` end with `?`:

```
empty? xs     -- is the collection empty?
contains? v xs-- does it contain v? (substring for strings, element for lists/sets)
starts? p s   -- does the string start with prefix p?
ends? p s     -- does the string end with suffix s?
even? n       -- is the number even?
odd? n        -- is the number odd?
ok? r         -- is the Result an Ok?
err? r        -- is the Result an Err?
some? m       -- is the Maybe a Some?
sorted? xs    -- is the list sorted?
```

## Built-in Functions

Always in scope. No `use` required.

### Pipeline Utilities

```
dbg x          -- prints [file:line] x = <value> to stderr, returns x
tap f x        -- applies (f x) for side effects, returns x
collect xs     -- converts iterable into list
identity x     -- returns x unchanged
not x          -- logical negation: not true = false
defer f        -- registers f for execution on scope exit (LIFO order)
require msg m  -- Maybe a -> Result a Str. None becomes Err msg
timeout n      -- completes after n seconds; for use in `sel` blocks
log            -- record with info/warn/err/debug fields, each a logging function
```

`dbg` captures the source expression at compile time. `dbg (add 3 4)` prints `[src/main.lx:5] add 3 4 = 7`. Drop it anywhere in a pipeline without changing behavior.

`tap` runs a function for side effects and passes through the original value: `data | tap (d) log.debug "count: {d | len}" | process`.

`defer` takes a zero-argument function: `defer () fs.close handle`. Multiple `defer` calls in a scope run in reverse order when the scope exits (normal return, error propagation, or break).

`require` bridges `Maybe` and `Result`: `env.get "PATH" | require "PATH not set" ^`.

```
retry n f          -- retry f up to n times, returns last Result
retry_with opts f  -- retry with backoff: {n: Int  delay: Duration  backoff: Float}
```

`retry 3 () http.get url` calls `f` up to 3 times, returning the first `Ok` or the last `Err`. `retry_with {n: 5  delay: time.sec 1  backoff: 2.0} () http.get url` waits 1s, 2s, 4s, 8s between attempts.

### Collection Functions

These work on lists, strings, sets, and other iterables where applicable.

```
len xs             -- element count (codepoint count for strings)
empty? xs          -- true if length is zero
contains? val xs   -- membership test
get key coll       -- safe access, returns Maybe a
first xs           -- Maybe a: first element
last xs            -- Maybe a: last element
```

### Map Functions

```
keys m             -- [key] in insertion order
values m           -- [value] in insertion order
entries m          -- [(key value)] in insertion order
has_key? key m     -- Bool
remove key m       -- new map without key
merge m1 m2        -- new map, m2 wins on conflict
to_map xs          -- convert [(k v)] or record to map
to_list m          -- convert map to [(k v)]
```

Maps are iterable — `map`, `filter`, `each` iterate over `(key value)` tuples.

### Sequence Constructors

```
cycle xs           -- repeating list of xs
step n xs          -- take every nth element from list
```

### Concurrent Functions

```
pmap f xs          -- parallel map (all elements concurrently)
pmap_n limit f xs  -- parallel map with concurrency limit
```

`pmap` spawns all elements concurrently. `pmap_n` limits to `limit` concurrent tasks (for rate-limited APIs). See [concurrency.md](concurrency.md).

### Conversion Functions

```
to_map x           -- record or [(k v)] to map
to_record m        -- map to record (keys must be valid identifiers)
to_list x          -- map or iterable to list
parse_int s        -- Str -> Result Int ParseErr
parse_float s      -- Str -> Result Float ParseErr
to_str x           -- any value to Str
encode encoding s  -- Str -> Bytes (encode string to bytes, e.g. "utf-8")
decode encoding b  -- Bytes -> Str ^ DecodeErr (decode bytes to string)
```

### List Functions

All data-last for piping. Signature shown as `name args data`.

**Transform:**

```
map f xs               -- apply f to each element
flat_map f xs          -- map then flatten one level
scan init f xs         -- fold returning all intermediate values
```

**Filter:**

```
filter pred xs         -- keep elements where pred is true
take n xs              -- first n elements
drop n xs              -- skip first n elements
take_while pred xs     -- take while predicate holds
drop_while pred xs     -- skip while predicate holds
```

**Search:**

```
find pred xs           -- first match, Maybe a
find_index pred xs     -- index of first match, Maybe Int
any? pred xs           -- true if any element matches
all? pred xs           -- true if all elements match
none? pred xs          -- true if no element matches
count pred xs          -- count of matching elements
```

**Order:**

```
sort xs                -- ascending sort (elements must be comparable)
sort_by key xs         -- sort by key function
rev xs                 -- reverse order
min xs / max xs        -- minimum/maximum element (empty list is a runtime panic)
min_by f xs / max_by f xs -- min/max by key function
```

**Grouping:**

```
zip ys xs              -- [(x0 y0) (x1 y1) ...]
zip_with f ys xs       -- combine paired elements with f
enumerate xs           -- [(0 x0) (1 x1) ...]
partition pred xs      -- (matches non_matches)
group_by key xs        -- %{key: [elements]} map
chunks n xs            -- split into sublists of size n
windows n xs           -- sliding windows of size n
```

**Reduction:**

```
fold init f xs         -- reduce to single value
sum xs                 -- sum of numbers (fold 0 (+))
product xs             -- product of numbers (fold 1 (*))
```

**Shape:**

```
flatten xss            -- flatten one level: [[1 2] [3]] -> [1 2 3]
intersperse val xs     -- insert val between elements
uniq xs                -- deduplicate adjacent equal elements
uniq_by key xs         -- deduplicate by key function
```

**Side Effects:**

```
each f xs              -- apply f to each element, returns unit
```

### String Functions

Strings are UTF-8 sequences. All data-last.

```
trim s                 -- strip leading/trailing whitespace
trim_start s           -- strip leading whitespace
trim_end s             -- strip trailing whitespace
lines s                -- split on newlines -> [Str]
split sep s            -- split by string or regex -> [Str]
join sep xs            -- join list with separator -> Str
upper s                -- uppercase
lower s                -- lowercase
starts? prefix s       -- prefix test
ends? suffix s         -- suffix test
contains? sub s        -- substring test
replace old new s      -- replace first occurrence
replace_all old new s  -- replace all (string or regex)
repeat n s             -- repeat n times
chars s                -- list of codepoint strings
byte_len s             -- byte count
pad_left width s       -- pad with spaces
pad_right width s      -- pad with spaces
```

`contains?`, `starts?`, `ends?` are polymorphic — they work on both strings and collections.

### Regex Functions (std/re)

Use `std/re` for pattern matching with string patterns:

```
use std/re
re.is_match "\\d+" text       -- Bool: does it match?
re.match "(\\w+)" text         -- Maybe {match groups}: capture groups
re.replace "old" "new" text    -- replace first match
re.replace_all "old" "new" text -- replace all matches
re.split "\\s+" text           -- split on matches
re.find_all "\\d+" text        -- [Str]: all matches
```

## Core Modules

| Module | Provides |
|---|---|
| `std/fs` | Filesystem: read, write, exists, mkdir, rm, copy, move, glob, walk |
| `std/http` | HTTP client: get, post, put, delete |
| `std/json` | JSON: parse, encode, encode_pretty |
| `std/env` | Environment: args, get, set, cwd |
| `std/re` | Regex: is_match, match, replace, split, find_all |
| `std/time` | Time: now, format, parse, sleep, sec, ms, min |
| `std/math` | Math: abs, sqrt, pow, log, sin, cos, pi, e, floor, ceil, round, clamp, safe_div |
| `std/ctx` | Context: load, save, get, set, empty, merge |
| `std/md` | Markdown: parse, sections, code_blocks, headings, render, doc, h1, h2, para, code, list |
| `std/agent` | Agent: spawn, kill, subprocess communication |
| `std/mcp` | MCP: connect, list_tools, call, close (stdio + HTTP transports) |

Agent ecosystem modules (`std/agent`, `std/mcp`, `std/ctx`, `std/md`) are in [stdlib-agents.md](stdlib-agents.md). Module API details are in [stdlib-modules.md](stdlib-modules.md).
