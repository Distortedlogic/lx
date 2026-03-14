# Examples — Extended

Additional worked examples. See [examples.md](examples.md) for core agentic examples.

## Continuous Monitoring Agent

```
use std/agent
use std/cron
use std/ctx

+main = () {
  state = ctx.load ".monitor-state.json" ?? ctx.empty ()

  cron.every (time.min 5) () {
    checks = ["api" "db" "cache"] | pmap (svc) {
      result = sel {
        agent.ask (agent.spawn {name: "health-{svc}" prompt: "Check {svc} health"} ^)
          {service: svc} -> it
        timeout 30 -> {status: "timeout" service: svc}
      }
      {service: svc  status: result.status  ts: time.now () | to_str}
    }

    failures = checks | filter (c) c.status != "ok"
    failures | empty? ? () : {
      agent.send (agent.connect "alerter" ^) {
        severity: failures | len > 1 ? "critical" : "warn"
        services: failures | map (.service)
        details: checks
      }
    }

    ctx.set "last_check" checks state
      | ctx.set "history" [..(ctx.get "history" state ?? []) ..checks]
      | (c) ctx.save ".monitor-state.json" c ^
  }
}
```

Uses: `cron.every` for scheduling, `pmap` for parallel checks, `sel`/`timeout` for deadlines, context persistence, agent messaging.

## Agent Memory with Markdown

```
use std/agent
use std/md
use std/ctx
use std/fs

+main = () {
  doc = md.parse (fs.read "AGENT_MEMORY.md" ^)
  prev_tasks = md.sections doc | find (s) s.title == "Active Tasks"
  prev_findings = md.code_blocks doc
    | filter (b) b.lang == Some "json")
    | map (b) json.parse b.code ^

  new_results = agent.ask (agent.spawn {name: "researcher" prompt: "Continue research"} ^)
    {prior_findings: prev_findings action: "extend"} ^

  updated = md.doc [
    md.h1 "Agent Memory"
    md.para "Last updated: {time.now () | time.format "%Y-%m-%d %H:%M"}"
    md.h2 "Active Tasks"
    md.list (new_results.tasks | map (.summary))
    md.h2 "Findings"
    md.code "json" (new_results.findings | json.encode_pretty)
    md.h2 "History"
    md.list (prev_findings | map (f) "{f.date}: {f.summary}")
  ]
  md.render updated | (out) fs.write "AGENT_MEMORY.md" out ^
}
```

Uses: markdown parsing for agent memory, structured extraction, markdown generation, agent delegation.

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
