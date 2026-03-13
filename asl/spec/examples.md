# Examples

Complete worked examples demonstrating lx features in realistic scenarios.

## Find Largest Source Files

```
use std/fs
use std/fmt

+main = () {
  dir = $^pwd | trim
  fs.walk dir
    | filter (ends? ".rs")
    | pmap (path) {
      content = fs.read path ^
      {path  lines: content | lines | len}
    }
    | sort_by (.lines) | rev | take 10
    | each (f) {
      padded = f.lines | fmt.pad_left 6
      $echo "{padded}  {f.path}"
    }
}
```

Uses: shell (`$^pwd`), pipes, `pmap` for parallel file reads, record literals, `sort_by` with field section.

## HTTP API Client

```
use std/net/http
use std/json

User = {name: Str  email: Str}
ApiErr = {status: Int  body: Str}

+fetch_users = (base_url: Str) -> [User] ^ ApiErr {
  resp = http.get "{base_url}/users" ^
  resp.status != 200 ? {
    true  -> Err {status: resp.status  body: resp.body}
    false -> resp.body | json.parse ^ | map (obj) {
      {name: obj."name"  email: obj."email"}
    }
  }
}

+main = () {
  fetch_users "https://api.example.com" ?? []
    | filter (u) { u.email | ends? "@company.com" }
    | map (.name)
    | each (name) $echo "employee: {name}"
}
```

Uses: type annotations, `^` propagation, `??` coalescing, pipeline filtering, sections.

## Concurrent API Aggregation

```
use std/net/http
use std/json

+main = () {
  urls = [
    "https://api.a.com/data"
    "https://api.b.com/data"
    "https://api.c.com/data"
  ]

  results = urls | pmap (url) http.get url ^

  all_items = results
    | flat_map (r) { r.body | json.parse ^ | (.items) }
    | sort_by (.date) | rev

  slow_url = "https://slow.example.com/data"
  fast = sel {
    http.get slow_url -> it.body | json.parse ^
    timeout 5         -> Err "too slow"
  }
}
```

Uses: `pmap` for parallel fetching, `flat_map`, `sel` for timeout racing.

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
