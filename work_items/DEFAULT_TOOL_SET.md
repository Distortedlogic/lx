# Goal

Ship lx with a default set of tools that agents can `uses` out of the box: Bash, Read, Write, Edit, Glob, Grep, WebSearch, WebFetch. These mirror the tools LLM coding agents expect to have available immediately.

# Why

An agent running an lx program needs basic capabilities without the user installing or configuring anything. Every LLM coding agent (Claude Code, Cursor, Copilot) ships with these tools built in.

# Depends On

- `TOOL_TRAIT_UNIFICATION.md` — the merged Tool trait must exist first
- `KEYWORD_DESUGAR_4_MCP_CLI.md` — the CLI keyword desugarer must work (generates the subprocess `run` method for CLI-backed tools)

# Default Tools

All defined as lx files in `crates/lx/std/tools/`. No new Rust builtins — tools use the existing extension mechanisms (CLI keyword desugaring provides subprocess execution via the already-implemented `std::process::Command` code in `desugar_mcp_cli.rs`, `std/fs` provides file I/O).

### Bash
CLI-backed via the `CLI` keyword. The desugarer auto-generates `run` that calls `std::process::Command`.

```lx
CLI Bash = {
  command: "bash"
  name: "bash"
  description: "Execute shell commands"
  params: {command: "Str"}
}
```

Returns `Ok {stdout: Str, stderr: Str, code: Int}` always — non-zero exit is a value, not an error.

### Read
Wraps existing `std/fs` read. Pure lx.

```lx
use std/fs

Tool Read = {
  name: "read"
  description: "Read file contents"
  params: {path: Str}
  run = (args) { fs.read (args.path) }
}
```

`fs.read` already returns `Ok content` or `Err message`.

### Write
Wraps existing `std/fs` write. Pure lx.

```lx
use std/fs

Tool Write = {
  name: "write"
  description: "Write content to file"
  params: {path: Str  content: Str}
  run = (args) { fs.write (args.path) (args.content) }
}
```

### Edit
Pure lx. Reads file, replaces first occurrence, writes back.

```lx
use std/fs

Tool Edit = {
  name: "edit"
  description: "Replace text in a file"
  params: {path: Str  old: Str  new: Str}
  run = (args) {
    content = fs.read (args.path) ^
    (content | contains? (args.old)) ? {
      true -> {
        updated = content | replace (args.old) (args.new)
        fs.write (args.path) updated ^
        Ok {path: args.path}
      }
      false -> Err "old string not found in {args.path}"
    }
  }
}
```

### Glob
Uses Bash tool to call `find`. Pure lx composition.

```lx
Tool Glob = {
  name: "glob"
  description: "Find files matching a pattern"
  params: {pattern: Str  path: Str = "."}
  run = (args) {
    result = Bash.run {command: "find {args.path} -name '{args.pattern}' -type f 2>/dev/null"} ^
    result ? {
      Ok r -> Ok (r.stdout | trim | split "\n" | filter (s) { (s | len) > 0 })
      Err e -> Err e
    }
  }
}
```

### Grep
Uses Bash tool to call `grep -rn`. Pure lx composition.

```lx
Tool Grep = {
  name: "grep"
  description: "Search file contents for a pattern"
  params: {pattern: Str  path: Str = "."}
  run = (args) {
    result = Bash.run {command: "grep -rn '{args.pattern}' {args.path} 2>/dev/null"} ^
    result ? {
      Ok r -> r.code == 0 ? (Ok r.stdout) : (Ok "")
      Err e -> Err e
    }
  }
}
```

Note: `grep` returns exit code 1 for no matches — handled as `Ok ""`, not an error.

### WebSearch
Uses Bash tool to call `curl` + DuckDuckGo lite. Pure lx composition. Returns raw HTML — parsing into structured results is a future WASM plugin.

```lx
Tool WebSearch = {
  name: "web_search"
  description: "Search the web"
  params: {query: Str}
  run = (args) {
    encoded = args.query | replace " " "+"
    result = Bash.run {command: "curl -sL 'https://lite.duckduckgo.com/lite?q={encoded}'"} ^
    result ? {
      Ok r -> Ok r.stdout
      Err e -> Err e
    }
  }
}
```

### WebFetch
Uses Bash tool to call `curl`. Pure lx composition. Returns raw HTML — markdown conversion is a future WASM plugin.

```lx
Tool WebFetch = {
  name: "web_fetch"
  description: "Fetch a URL and return its content"
  params: {url: Str}
  run = (args) {
    result = Bash.run {command: "curl -sL '{args.url}'"} ^
    result ? {
      Ok r -> r.code == 0 ? (Ok r.stdout) : (Err "fetch failed: HTTP error")
      Err e -> Err e
    }
  }
}
```

# Registration

Default tools are loaded as lx source modules from `crates/lx/std/tools/`. They're registered in the interpreter's environment during initialization so agents can `uses` them without explicit `use` imports.

In `crates/lx/src/interpreter/mod.rs`, after `builtins::register(&env)`:
- Load each default tool definition from `std/tools/*.lx` via `lx_std_module_source()`
- Bind each tool name (Bash, Read, Write, etc.) in the global environment

When an agent declares `uses Bash`, the `uses` wiring (from AGENT_USES_WIRING work item) resolves `Bash` from the environment.

# Gotchas

- **Glob uses `find`:** On macOS, `find` has different flag semantics than GNU find. The `-name` + `-type f` flags are POSIX-compatible and work on both.
- **Grep exit code 1:** `grep` returns 1 for "no matches" — normal, not an error. The tool handles this by checking `.code`.
- **WebSearch returns raw HTML:** Parsing search results into structured data is left for a future WASM plugin. The v1 is functional but crude.
- **WebFetch returns raw HTML:** Same — markdown conversion is a future WASM plugin.
- **Shell injection:** Commands built with string interpolation (`"find {args.path}"`) are vulnerable to injection if args contain shell metacharacters. Proper escaping is tracked in STD_SANDBOX.md.
- **Bash tool must be loaded before Glob/Grep/WebSearch/WebFetch:** These tools call `Bash.run` — Bash must be in scope. Load order in registration matters.
- **CLI keyword desugarer generates `call` not `run`:** The current desugarer in `desugar_mcp_cli.rs` generates `connect`/`disconnect`/`call`/`tools` methods for the Connector trait. After TOOL_TRAIT_UNIFICATION, it generates `run`. This work item depends on that change being done first.

# Task List

### Task 1: Create tool definition files
Create `crates/lx/std/tools/bash.lx`, `read.lx`, `write.lx`, `edit.lx`, `glob.lx`, `grep.lx`, `web_search.lx`, `web_fetch.lx` with the exact definitions above.

### Task 2: Register default tools in interpreter
Update `lx_std_module_source()` to serve `tools/*` files. Update interpreter initialization to load and bind default tool names in the global environment. Ensure Bash loads first (Glob/Grep/WebSearch/WebFetch depend on it).

### Task 3: Write tests
Create `tests/default_tools.lx`. Test Bash (echo), Read/Write (create temp file, read it back), Edit (replace string), Glob (find test files), Grep (search for known pattern). WebSearch and WebFetch are not tested in CI (require network) — test manually.

---

## Task Loading Instructions

```
mcp__workflow__load_work_item({ path: "work_items/DEFAULT_TOOL_SET.md" })
```
