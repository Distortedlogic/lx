# Goal

Ship lx with a default set of tools that agents can `uses` out of the box: Bash, Read, Write, Edit, Glob, Grep, WebSearch, WebFetch. These mirror the tools LLM coding agents expect to have available immediately.

# Why

An agent running an lx program needs basic capabilities without the user installing or configuring anything. Every LLM coding agent (Claude Code, Cursor, Copilot) ships with these tools built in.

# Depends On

- `TOOL_TRAIT_UNIFICATION.md` — the merged Tool trait must exist first

# Default Tools

All defined as lx files in `crates/lx/std/tools/` using the `Tool` keyword. Each has a fixed `run` implementation — no "TBD" backings.

### Bash
CLI-backed. `run` calls `std::process::Command` with `bash -c`.

```lx
Tool Bash = {
  name: "bash"
  description: "Execute shell commands"
  params: {command: Str}
  run = (args) { bash_exec (args.command) }
}
```

`bash_exec` is a new Rust builtin in `stdlib/tools/cli.rs`:
```rust
fn bi_bash_exec(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
    let cmd = args[0].require_str("bash_exec", span)?;
    let output = std::process::Command::new("bash")
        .arg("-c").arg(cmd).output()
        .map_err(|e| LxError::runtime(format!("bash: {e}"), span))?;
    Ok(LxVal::ok(LxVal::record(indexmap! {
        sym!("stdout") => LxVal::str(String::from_utf8_lossy(&output.stdout)),
        sym!("stderr") => LxVal::str(String::from_utf8_lossy(&output.stderr)),
        sym!("code") => LxVal::int(output.status.code().unwrap_or(-1)),
    })))
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
CLI-backed via `find`. Returns list of matching paths.

```lx
Tool Glob = {
  name: "glob"
  description: "Find files matching a pattern"
  params: {pattern: Str  path: Str = "."}
  run = (args) {
    result = bash_exec "find {args.path} -name '{args.pattern}' -type f 2>/dev/null"
    result ^ ? {
      Ok r -> Ok (r.stdout | trim | split "\n" | filter (s) { (s | len) > 0 })
      Err e -> Err e
    }
  }
}
```

### Grep
CLI-backed via `grep -rn`. Returns matching lines with file paths.

```lx
Tool Grep = {
  name: "grep"
  description: "Search file contents for a pattern"
  params: {pattern: Str  path: Str = "."}
  run = (args) {
    result = bash_exec "grep -rn '{args.pattern}' {args.path} 2>/dev/null"
    result ^ ? {
      Ok r -> r.code == 0 ? (Ok r.stdout) : (Ok "")
      Err e -> Err e
    }
  }
}
```

Note: `grep` returns exit code 1 for no matches — this is handled as `Ok ""`, not an error.

### WebSearch
CLI-backed. Calls a search tool. Initial implementation uses `curl` + DuckDuckGo lite HTML endpoint, parses results. This is functional but basic — can be upgraded later to a proper search API or MCP server.

```lx
Tool WebSearch = {
  name: "web_search"
  description: "Search the web"
  params: {query: Str}
  run = (args) {
    encoded = args.query | replace " " "+"
    result = bash_exec "curl -sL 'https://lite.duckduckgo.com/lite?q={encoded}'"
    result ^ ? {
      Ok r -> Ok r.stdout
      Err e -> Err e
    }
  }
}
```

### WebFetch
CLI-backed. Fetches URL and returns content. Uses `curl` for the fetch. Markdown conversion is not included in v1 — returns raw HTML. Markdown conversion can be added as a WASM plugin (`html_to_md`) or upgraded to use a readability tool later.

```lx
Tool WebFetch = {
  name: "web_fetch"
  description: "Fetch a URL and return its content"
  params: {url: Str}
  run = (args) {
    result = bash_exec "curl -sL '{args.url}'"
    result ^ ? {
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

# New Rust Builtins

One new builtin needed: `bash_exec` — registered globally in `builtins/register.rs`. This is the backing function for CLI tools. It takes a command string and returns `Ok {stdout, stderr, code}`.

No other Rust builtins needed — Read, Write, Edit, Glob, Grep, WebSearch, WebFetch are all pure lx wrapping either `bash_exec` or `std/fs`.

# Gotchas

- **`bash_exec` vs existing `$^` syntax:** `bash_exec` returns `Ok {stdout, stderr, code}` always. `$^` throws uncatchable `LxError`. They coexist until `$^` is removed (Phase 3 of EXTENSIONS.md migration).
- **Glob uses `find`:** On macOS, `find` has different flag semantics than GNU find. The `-name` + `-type f` flags are POSIX-compatible and work on both.
- **Grep exit code 1:** `grep` returns 1 for "no matches" — this is normal, not an error. The tool handles this by checking `.code`.
- **WebSearch returns raw HTML:** Parsing search results into structured data is left for a future WASM plugin or MCP tool. The v1 is functional but crude.
- **WebFetch returns raw HTML:** Same — markdown conversion is a future enhancement. Raw content is still useful for agents.
- **String interpolation in `bash_exec` calls:** Commands built with string interpolation (`"find {args.path}"`) are vulnerable to injection if args contain shell metacharacters. For v1 this matches how `$^` works today. Proper escaping is a future enhancement (tracked in STD_SANDBOX.md).

# Task List

### Task 1: Add `bash_exec` builtin
Create `crates/lx/src/stdlib/tools/cli.rs` with `bi_bash_exec`. Register in `builtins/register.rs` as `bash_exec`. Add `pub mod tools;` to `stdlib/mod.rs` with `pub mod cli;` inside.

### Task 2: Create tool definition files
Create `crates/lx/std/tools/bash.lx`, `read.lx`, `write.lx`, `edit.lx`, `glob.lx`, `grep.lx`, `web_search.lx`, `web_fetch.lx` with the exact definitions above.

### Task 3: Register default tools in interpreter
Update `lx_std_module_source()` to serve `tools/*` files. Update interpreter initialization to bind default tool names in the global environment.

### Task 4: Write tests
Create `tests/default_tools.lx`. Test Bash (echo), Read/Write (create temp file, read it back), Edit (replace string), Glob (find test files), Grep (search for known pattern). WebSearch and WebFetch are not tested in CI (require network) — test manually.

---

## Task Loading Instructions

```
mcp__workflow__load_work_item({ path: "work_items/DEFAULT_TOOL_SET.md" })
```
