# Examples — Extended

Additional worked examples. See [examples.md](examples.md) for core examples.

## Git Branch Cleanup

```
+main = () {
  current = $^git branch --show-current | trim
  merged = $^git branch --merged
    | lines
    | map trim
    | filter (b) b != current && b != "main" && b != "master"

  merged | empty? ? {
    true  -> $echo "no branches to clean"
    false -> {
      $echo "branches merged into {current}:"
      merged | each (b) $echo "  {b}"
      $echo "\ndelete {merged | len} branches? [y/N]"
      io.read_line ^ | trim | lower ? {
        "y" -> merged | each (b) $^git branch -d {b}
        _   -> $echo "cancelled"
      }
    }
  }
}
```

Uses: `$^` for pipeline-friendly shell, `lines`/`trim`/`filter` for parsing shell output, pattern matching on user input.

## Data Deduplication with Sets

```
use std/fs
use std/crypto

+main = () {
  seen := #{}
  dupes := []

  fs.walk "data/"
    | filter (ends? ".json")
    | each (path) {
      content = fs.read path ^
      hash = crypto.sha256 content
      contains? hash seen ? {
        true  -> dupes <- [..dupes path]
        false -> seen <- #{..seen hash}
      }
    }

  dupes | empty? ? $echo "no duplicates" : {
    $echo "found {dupes | len} duplicates:"
    dupes | each (p) $echo "  {p}"
  }
}
```

Uses: mutable set for tracking, `crypto.sha256`, `fs.walk`, set spread `#{..s val}`.

## CSV Report Generator

```
use std/fs
use std/csv
use std/fmt

+main = () {
  data = fs.read "sales.csv" ^ | csv.parse_with {delimiter: ","  header: true} ^

  by_region = data | group_by (row) row."region"

  by_region | entries | sort_by (.0) | each (region rows) {
    total = rows | map (r) { r."amount" | parse_int ^ } | sum
    count = rows | len
    avg = count > 0 ? total / count : 0
    $echo "{region | fmt.pad_right 15} total: {total | fmt.pad_left 8}  avg: {avg | fmt.pad_left 6}  count: {count}"
  }
}
```

Uses: `csv.parse_with` for header mode, `group_by`, `entries` for map iteration, `fmt` for aligned output. Note: `each (region rows)` uses tuple auto-spread — each entry is a `(Str [Row])` tuple that spreads into the two parameters.

## Parallel Health Checker

```
use std/env
use std/fmt
use std/net/http
use std/time

+main = () {
  services = [
    {name: "api"  url: "https://api.example.com/health"}
    {name: "auth" url: "https://auth.example.com/health"}
    {name: "db"   url: "https://db.example.com/health"}
  ]

  results = services | pmap (svc) {
    start = time.now ()
    status = sel {
      http.get svc.url -> it.status == 200 ? "ok" : "error ({it.status})"
      timeout 5 -> "timeout"
    }
    elapsed = time.elapsed start
    {name: svc.name  status  ms: elapsed | time.to_ms}
  }

  results | each (r) {
    icon = r.status == "ok" ? "+" : "!"
    $echo "[{icon}] {r.name | fmt.pad_right 10} {r.status | fmt.pad_right 15} {r.ms}ms"
  }

  failures = results | filter (r) r.status != "ok"
  failures | empty? ? () : env.exit 1
}
```

Uses: `pmap` for parallel checks, `sel`/`timeout` for deadline, `time.elapsed` for latency.

## Log File Analyzer

```
use std/fs
use std/math

+main = () {
  env.args ? {
    [path] -> analyze path
    _      -> $echo "usage: logstat <file>"
  }
}

analyze = (path) {
  levels = fs.read_lines path ^
    | filter (contains? r/\[(ERROR|WARN|INFO|DEBUG)\]/)
    | map (line) {
      line | match r/\[(ERROR|WARN|INFO|DEBUG)\]/ ? {
        Some groups -> groups.1
        None        -> "UNKNOWN"
      }
    }
    | fold %{} (counts level) {
      n = get level counts ?? 0
      %{..counts  level: n + 1}
    }

  ["ERROR" "WARN" "INFO" "DEBUG"] | each (level) {
    n = get level levels ?? 0
    capped = math.clamp 0 50 n
    bar = repeat capped "#"
    $echo "{level | fmt.pad_right 6} {n | fmt.pad_left 5} {bar}"
  }
}
```

Uses: `fs.read_lines` for streaming, regex matching in pipeline, `fold` to build frequency map, `repeat`/`take`/`join` for ASCII bar chart.

## Config Merger

```
use std/env
use std/fs
use std/json
use std/toml

+main = () {
  defaults = fs.read "defaults.toml" ^ | toml.parse ^
  overrides = fs.exists? "local.toml" ? {
    true  -> fs.read "local.toml" ^ | toml.parse ^
    false -> %{}
  }
  env_overrides = env.vars ()
    | entries
    | filter (kv) { starts? "APP_" kv.0 }
    | map (kv) { (kv.0 | drop 4 | lower  kv.1) }
    | to_map

  config = merge defaults (merge overrides env_overrides)
  config | json.encode_pretty | (out) fs.write "config.json" out ^
  $echo "wrote config.json with {config | keys | len} keys"
}
```

Uses: `toml.parse`, `merge` for layered config, `env.vars` for env-based overrides, `json.encode_pretty` for output.
