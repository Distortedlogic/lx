# Concurrency — Reference

Structured concurrency only. Every concurrent op has a clear scope and lifetime.

## `pmap` / `pmap_n`

```
results = urls | pmap fetch                     -- parallel map, all concurrent
results = urls | pmap_n 10 fetch                -- at most 10 concurrent
```

Same signature as `map`. Result order matches input. Any failure cancels remaining and propagates.

## `par` — Parallel Block

```
(a b c) = par {
  fetch url1 ^
  fetch url2 ^
  fetch url3 ^
}
```

Runs all statements concurrently, returns tuple in order. First error cancels siblings.

## `sel` — Select (Race)

```
result = sel {
  fetch url   -> Ok it
  timeout 5   -> Err "timed out"
}
```

Arms run concurrently. First to complete wins; `it` binds to result. Others cancelled.

## Cancellation

Shell commands get SIGTERM, HTTP requests aborted, nested `par`/`sel` recursively cancelled.

## Mutable State Restriction

Capturing mutable bindings (`:=`) in `par`/`sel`/`pmap` bodies is a **compile error**. Mutables defined *inside* the body are fine (local to each task). Collect results first, process sequentially:

```
results = xs | pmap process
total = results | sum
```

## Patterns

```
urls | pmap fetch | filter ok? | map (?? ())           -- fan-out/fan-in

(users posts) = par {                                    -- parallel + timeout
  sel { fetch_users ^ -> it; timeout 5 -> Err "timeout" }
  sel { fetch_posts ^ -> it; timeout 5 -> Err "timeout" }
}

items | chunks 10 | each (batch) { batch | pmap process } -- batch

results = urls | pmap (url) fetch url                    -- no ^ = Err values kept
successes = results | filter ok? | map (?? ())
```
