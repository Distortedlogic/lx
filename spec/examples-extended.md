# Examples — Extended

Additional worked examples. See [examples.md](examples.md) for core agentic examples.

## Continuous Monitoring Agent

```
use std/agent
use std/ctx
use std/time

+main = () {
  state = ctx.load ".monitor-state.json" ?? ctx.empty ()

  loop {
    checks = ["api" "db" "cache"] | pmap (svc) {
      agent_handle = agent.spawn {command: "health-check" args: [svc]} ^
      result = sel {
        agent_handle ~>? {service: svc} -> it
        timeout 30 -> {status: "timeout" service: svc}
      }
      {service: svc  status: result.status  ts: time.now () | to_str}
    }

    failures = checks | filter (c) c.status != "ok"
    failures | empty? ? () : {
      $echo "ALERT: {failures | map (.service) | join ", "} failed"
    }

    ctx.set "last_check" checks state
      | (c) ctx.save ".monitor-state.json" c ^
    time.sleep (time.sec 300)
  }
}
```

Uses: `loop` for continuous monitoring, `pmap` for parallel checks, `sel`/`timeout` for deadlines, context persistence, agent messaging.

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
    | filter (b) b.lang == Some "json"
    | map (b) json.parse b.code ^

  new_results = (agent.spawn {name: "researcher" prompt: "Continue research"} ^)
    ~>? {prior_findings: prev_findings action: "extend"} ^

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

## JSON Report Generator

```
use std/fs
use std/json

+main = () {
  data = fs.read "sales.json" ^ | json.parse ^

  by_region = data | group_by (row) row."region"

  by_region | entries | sort_by (.0) | each (region, rows) {
    total = rows | map (r) r."amount" | sum
    count = rows | len
    avg = count > 0 ? total / count : 0
    $echo "{region} total: {total}  avg: {avg}  count: {count}"
  }
}
```

Uses: `json.parse` for data loading, `group_by`, `entries` for map iteration. Note: `each (region, rows)` uses tuple auto-spread — each entry is a `(Str [Row])` tuple that spreads into the two parameters.

## Parallel Health Checker

```
use std/env
use std/http
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
    $echo "[{icon}] {r.name} {r.status} {r.ms}ms"
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
use std/re

+main = () {
  env.args ? {
    [path] -> analyze path
    _      -> $echo "usage: logstat <file>"
  }
}

analyze = (path) {
  levels = fs.read_lines path ^
    | filter (line) re.is_match "\\[(ERROR|WARN|INFO|DEBUG)\\]" line
    | map (line) {
      re.match "\\[(ERROR|WARN|INFO|DEBUG)\\]" line ? {
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
    $echo "{level} {n} {bar}"
  }
}
```

Uses: `fs.read_lines` for streaming, `re.match`/`re.is_match` for pattern matching in pipeline, `fold` to build frequency map, `repeat` for ASCII bar chart.

## Config Merger

```
use std/env
use std/fs
use std/json

+main = () {
  defaults = fs.read "defaults.json" ^ | json.parse ^
  overrides = fs.exists? "local.json" ? {
    true  -> fs.read "local.json" ^ | json.parse ^
    false -> %{}
  }

  config = merge defaults overrides
  config | json.encode_pretty | (out) fs.write "config.json" out ^
  $echo "wrote config.json with {config | keys | len} keys"
}
```

Uses: `json.parse` for config loading, `merge` for layered config, `json.encode_pretty` for output.
