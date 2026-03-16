# Standard Library — Reference

## Conventions

**Data-last** — primary data argument last: `map f xs` so `xs | map f` works.
**Predicate `?` suffix** — `empty?`, `contains?`, `starts?`, `even?`, `ok?`, `some?`, `any?`, `all?`, `none?`, `has_key?`

## Built-in Functions (no `use` required)

```
dbg x              -- prints [file:line] x = <value>, returns x
tap f x            -- (f x) for side effects, returns x
collect xs         -- iterable to list
identity x         -- returns x
not x              -- logical negation
defer f            -- scope-exit execution (LIFO)
require msg m      -- Maybe -> Result. None becomes Err msg
timeout n          -- n seconds (for sel blocks)
log                -- log.info, log.warn, log.err, log.debug
retry n f          -- retry up to n times
retry_with opts f  -- {n: Int  delay: Duration  backoff: Float}
len xs             -- element count (codepoints for strings)
empty? xs          -- true if length zero
contains? val xs   -- membership test
get key coll       -- safe access, Maybe a
first xs           -- Maybe a
last xs            -- Maybe a
```

## List Functions (all data-last)

```
map f xs            flat_map f xs        scan init f xs
filter pred xs      take n xs            drop n xs
take_while pred xs  drop_while pred xs
find pred xs        find_index pred xs   any? pred xs
all? pred xs        none? pred xs        count pred xs
sort xs             sort_by key xs       rev xs
min xs              max xs               min_by f xs          max_by f xs
zip ys xs           zip_with f ys xs     enumerate xs
partition pred xs   group_by key xs      chunks n xs          windows n xs
fold init f xs      sum xs               product xs
flatten xss         intersperse val xs   uniq xs              uniq_by key xs
each f xs
```

## String Functions

```
trim s              trim_start s         trim_end s
lines s             split sep s          join sep xs
upper s             lower s
starts? prefix s    ends? suffix s       contains? sub s
replace old new s   replace_all old new s
repeat n s          chars s              byte_len s
pad_left width s    pad_right width s
```

## Map: `keys`, `values`, `entries`, `has_key?`, `remove`, `merge`, `to_map`, `to_list`

## Conversion: `to_map`, `to_record`, `to_list`, `parse_int`, `parse_float`, `to_str`, `encode`, `decode`

## Concurrent: `pmap f xs`, `pmap_n limit f xs`

## Sequences: `cycle xs`, `step n xs`

## Regex (std/re)
```
use std/re
re.is_match r/\d+/ text                 re.match r/(\w+)/ text
re.replace r/old/ "new" text            re.replace_all r/old/ "new" text
re.split r/\s+/ text                    re.find_all r/\d+/ text
```

## Core Modules

| Module | Provides |
|---|---|
| `std/fs` | read, write, exists, mkdir, rm, copy, move, glob, walk |
| `std/http` | get, post, put, delete |
| `std/json` | parse, encode, encode_pretty |
| `std/env` | args, get, set, cwd |
| `std/re` | is_match, match, replace, split, find_all |
| `std/time` | now, format, parse, sleep, sec, ms, min |
| `std/math` | abs, sqrt, pow, log, sin, cos, pi, e, floor, ceil, round, clamp, safe_div |
| `std/ctx` | load, save, get, set, empty, merge |
| `std/md` | parse, sections, code_blocks, headings, render, doc, h1-h2, para, code, list |
| `std/ai` | prompt, prompt_with |
| `std/agent` | spawn, kill, capability attenuation |
| `std/mcp` | connect, list_tools, call, close (stdio + HTTP) |
| `std/blackboard` | create, read, write, watch, keys |
| `std/events` | create, publish, subscribe, unsubscribe |
| `std/knowledge` | create, store, get, query, expire |
| `std/introspect` | self, budget, actions, is_stuck, strategy_shift |
| `std/plan` | run, replan, continue, abort, skip, insert_after |
| `std/context` | create, add, usage, pressure, evict, compact, pin |
| `std/prompt` | create, system, section, example, compose, render, render_within |
| `std/strategy` | create, record, best_for, rank, suggest, adapt |
