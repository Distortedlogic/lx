# Examples

Complete worked examples demonstrating lx features in realistic agentic scenarios.

## Multi-Agent Code Review

```
use std/agent
use std/ctx

+main = () {
  files = $^git diff --name-only HEAD~1 | lines | filter (!= "")

  (security perf style) = par {
    agent.ask (agent.spawn {name: "security" prompt: "Audit for vulnerabilities"} ^)
      {files action: "review"} ^
    agent.ask (agent.spawn {name: "perf" prompt: "Check for performance issues"} ^)
      {files action: "review"} ^
    agent.ask (agent.spawn {name: "style" prompt: "Check code style"} ^)
      {files action: "review"} ^
  }

  findings = [..security.issues ..perf.issues ..style.issues]
    | sort_by (.severity) | rev
  findings | each (f) $echo "[{f.severity}] {f.file}:{f.line} — {f.message}"

  ctx.set "last_review" findings (ctx.empty ())
    | (c) ctx.save ".review-state.json" c ^
}
```

Uses: `par` for parallel agent work, `agent.spawn`/`agent.ask`, context persistence, shell + pipes.

## MCP Tool Orchestration

```
use std/mcp

+main = () {
  client = mcp.connect "stdio:///usr/local/bin/code-tools" ^
  tools = mcp.list_tools client ^
  $echo "available tools: {tools | map (.name) | join ", "}"

  files = mcp.call client "list_files" {pattern: "src/**/*.lx"} ^
  results = files | pmap (f) {
    content = mcp.call client "read_file" {path: f} ^
    analysis = mcp.call client "analyze" {content code: content lang: "lx"} ^
    {file: f  issues: analysis.issues}
  }

  results | filter (r) r.issues | len > 0
    | each (r) {
      $echo "\n{r.file}:"
      r.issues | each (i) $echo "  [{i.severity}] {i.message}"
    }
  mcp.close client
}
```

Uses: MCP tool discovery and invocation, `pmap` for parallel analysis, pipeline filtering.

## Agent Pipeline with Context

```
use std/agent
use std/ctx
use std/md

+main = () {
  state = ctx.load ".pipeline-state.json" ?? ctx.empty ()
  step = ctx.get "step" state ?? "fetch"

  step ? {
    "fetch" -> {
      data = agent.ask (agent.spawn {name: "fetcher" prompt: "Gather data"} ^)
        {sources: ["api" "db" "logs"]} ^
      ctx.set "step" "analyze" state
        | ctx.set "raw_data" data
        | (c) ctx.save ".pipeline-state.json" c ^
    }
    "analyze" -> {
      raw = ctx.get "raw_data" state ^
      result = agent.ask (agent.spawn {name: "analyzer" prompt: "Analyze data"} ^)
        {data: raw} ^
      report = md.doc [
        md.h1 "Analysis Report"
        md.para "Processed {raw | len} sources"
        md.h2 "Findings"
        md.list (result.findings | map (.summary))
      ]
      md.render report | (out) fs.write "report.md" out ^
      ctx.set "step" "done" state | (c) ctx.save ".pipeline-state.json" c ^
    }
    "done" -> $echo "pipeline complete"
  }
}
```

Uses: checkpoint/resume workflow, context persistence, markdown report generation, agent delegation.

## CLI Tool with Pattern Matching

```
use std/env
use std/fs

+main = () {
  env.args ? {
    ["count" path] -> {
      n = fs.read path ^ | lines | len
      $echo "{n} lines in {path}"
    }
    ["find" pattern ..paths] -> {
      paths | each (p) {
        fs.read p ?? ""
          | lines
          | filter (contains? pattern)
          | each (line) $echo "{p}: {line}"
      }
    }
    _ -> $echo "usage: tool <count|find> [args...]"
  }
}
```

Uses: list destructuring with `..rest`, pattern matching on `env.args`, `??` for fallback.

## Multi-Line Shell with Pipeline Debugging

```
use std/fs

+main = () {
  build_result = ${
    cd project/
    make clean
    make -j8
  }

  build_result ? {
    Ok {code: 0 ..} -> {
      fs.walk "build/"
        | filter (ends? ".o")
        | dbg
        | tap (files) $echo "found {files | len} objects"
        | map (f) {path: f  size: fs.stat f ^ | (.size)}
        | sort_by (.size) | rev | take 5
        | each (f) $echo "{f.size}\t{f.path}"
    }
    Ok {err ..} -> $echo "build failed: {err}"
    Err e -> $echo "couldn't run make: {e}"
  }
}
```

Uses: `${ }` multi-line shell, `dbg` for pipeline inspection, `tap` for side effects, record spread matching.

## JSON Transform Pipeline

```
use std/json
use std/fs

+main = () {
  data = fs.read "input.json" ^ | json.parse ^
  data."users"
    | filter (u) { u."active" == true }
    | map (u) {
      name = "{u."first"} {u."last"}"
      {name  email: u."email"  role: u."role" ?? "user"}
    }
    | sort_by (.name)
    | json.encode
    | (out) fs.write "output.json" out ^
}
```

Uses: JSON field access with `."key"`, `??` for default values, pipe to inline function for final write.

## Parallel File Processor

```
use std/fs

+main = () {
  (config data_files) = par {
    fs.read "config.json" ^ | json.parse ^
    fs.walk "data/" | filter (ends? ".csv") | collect
  }

  results = data_files | pmap (path) {
    raw = fs.read path ^
    raw | lines | filter (!= "") | len
  }

  total = results | sum
  $echo "processed {results | len} files, {total} total lines"
}
```

Uses: `par` block for concurrent setup, `pmap` for parallel processing, `collect` to force lazy sequence.

## Iterator Protocol: Fibonacci

```
fib = () {
  a := 0; b := 1
  {next: () { val = a; tmp = a + b; a <- b; b <- tmp; Some val }}
}

+main = () {
  fib () | take 20 | each (n) $echo "{n}"
}
```

Uses: iterator protocol (record with `next`), closures over mutable state, lazy consumption with `take`.

## Interactive Loop with Defer

```
use std/fs
use std/io

+main = () {
  log_file = fs.open "session.log" ^
  defer () fs.close log_file

  loop {
    io.print "> "
    line = io.read_line ^
    line | trim ? {
      "quit" -> break
      ""     -> ()
      cmd    -> {
        result = $sh -c "{cmd}"
        result ? {
          Ok {out ..} -> {
            $echo "{out}"
            fs.append "session.log" "{cmd}: {out}\n" ^
          }
          Err e -> $echo "error: {e}"
        }
      }
    }
  }
}
```

Uses: `defer` for cleanup, `loop`/`break`, pattern matching for control flow, shell execution.

## Retry with Backoff

```
use std/time
use std/net/http

with_retry = (n delay f) {
  attempt := 0
  loop {
    f () ? {
      Ok val -> break (Ok val)
      Err e  -> {
        attempt <- attempt + 1
        attempt >= n ? break (Err e)
        wait = delay * attempt
        log.warn "attempt {attempt}/{n} failed: {e}, retrying in {wait}"
        time.sleep wait
      }
    }
  }
}

+main = () {
  f = () http.get "https://api.example.com/data"
  data = with_retry 3 (time.sec 1) f
  data ? {
    Ok resp -> resp.body | json.parse ^ | (.items) | each (i) $echo "{i}"
    Err e   -> log.err "all attempts failed: {e}"
  }
}
```

Uses: mutable state in `loop`, `time.sleep` with stdlib durations, higher-order function pattern for retry logic.

More examples in [examples-extended.md](examples-extended.md).
