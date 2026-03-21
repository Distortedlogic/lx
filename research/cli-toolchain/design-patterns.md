# CLI Design Patterns for Language Toolchains

Research date: 2026-03-20

## 1. Subcommand Organization

### Verb-Noun vs Noun-Verb

Two dominant patterns:

**Verb-first** (most common for simple CLIs): `git commit`, `cargo build`, `go test`, `deno run`. The verb is the subcommand. Works when there is one object type per action.

**Noun-verb** (for complex CLIs with many object types): `docker container create`, `kubectl get pods`, `aws s3 cp`. The noun groups related verbs. Used when the same verbs apply to different objects.

**Hybrid**: Some tools mix both. `go mod init` is noun-verb (mod is the noun group), while `go build` is verb-only.

Microsoft's CLI design guidance recommends: use verbs for commands that perform actions, nouns for grouping subcommands. If a command has subcommands, the command should function as a grouping identifier, not an action.

Source: [System.CommandLine design guidance](https://learn.microsoft.com/en-us/dotnet/standard/commandline/design-guidance), [CLI Proposal: verb-noun structure](https://github.com/fnproject/cli/wiki/CLI-Proposal:--verb--noun--structure)

### When to Use Flags vs Subcommands

**Use subcommands** when the behavior fundamentally changes (different arguments, different output, different side effects). Example: `cargo build` vs `cargo test` have different semantics.

**Use flags** when modifying the same core behavior. Example: `cargo build --release` vs `cargo build` (same action, different profile).

**Rule of thumb from clig.dev**: subcommands divide the CLI into two parts -- global options that apply to everything, and per-command options. Options are optional by definition; requiring a flag is a design smell.

Source: [clig.dev](https://clig.dev/), [Julio Merino on subcommand-based interfaces](https://jmmv.dev/2013/09/cli-design-subcommand-based-interfaces.html)

### Grouping Strategies

**By workflow phase** (Cargo): Build commands, Manifest commands, Package commands, Publishing commands. Users find commands based on what stage of development they're in.

**By object type** (Go): `go mod *`, `go work *`, `go tool *`. Groups commands that operate on the same thing.

**Flat with conventions** (Deno): All commands at top level, but named consistently (`deno fmt`, `deno lint`, `deno test`). Works when command count is manageable (~25 or fewer).

**Task-based** (Mix): Every command is a task. `mix deps.get`, `mix phx.new`. Dot-separated namespaces provide implicit grouping without formal nesting.

### Consistency Rules

From [clig.dev](https://clig.dev/) and [Heroku CLI Style Guide](https://devcenter.heroku.com/articles/cli-style-guide):

- Use the same flag names across subcommands for the same concept (`--verbose`, `--quiet`, `--json`)
- Use consistent output formatting across all subcommands
- Never allow arbitrary prefix abbreviation of commands (explicitly define aliases only)
- Display the most common commands first in help text
- Avoid ambiguous near-synonyms (`update` vs `upgrade`)


## 2. Help and Documentation

### Help Flag Design

The convention from [clig.dev](https://clig.dev/):

- `-h` / `--help`: show full help
- No arguments when arguments are required: show concise usage + hint to use `--help`
- `help <subcommand>`: alternative to `<subcommand> --help`

### Help Content Structure

```
<tool> <command> - one-line description

USAGE:
    <tool> <command> [OPTIONS] <ARGS>

EXAMPLES:
    $ <tool> <command> foo         # Common case first
    $ <tool> <command> -v foo bar  # Then complex cases

OPTIONS:
    -v, --verbose    Increase output verbosity
    -q, --quiet      Suppress non-essential output
    -h, --help       Print help

SUBCOMMANDS:
    sub1    Description of sub1
    sub2    Description of sub2

See '<tool> help <subcommand>' for more information.
Report bugs at https://github.com/...
```

### How Real Tools Format Help

**Cargo**: Groups flags by purpose (Package Selection, Feature Selection, Compilation Options, Output Options, Display Options, Manifest Options). Uses uppercase section headers.

**Go**: Terse by default. `go help <topic>` for extended docs. `go help buildconstraint` documents build tags. Help text is plain ASCII, no formatting.

**Mix**: `mix help` lists all tasks with `@shortdoc`. `mix help <task>` shows `@moduledoc`. Tasks without `@shortdoc` are hidden from listings but still executable.

**Deno**: Colored output with bold section headers. Groups commands by category in `--help` output. Links to web documentation.

### Man Pages

Cargo generates man pages via `cargo install cargo-man` or ships them with the Rust toolchain. Go includes man-style help via `go doc`. Most modern tools skip man pages in favor of `--help` and web docs.

Source: [clig.dev](https://clig.dev/)


## 3. Error Messages

### Principles

From [clig.dev](https://clig.dev/):

1. **Rewrite errors for humans**: Catch internal errors and translate them into actionable messages
2. **Put critical info last**: Terminal users' eyes naturally focus on the bottom of output
3. **Use color sparingly**: Red for errors, yellow for warnings. High signal-to-noise ratio
4. **Group related errors**: Don't spam 100 lines for 100 instances of the same problem

### "Did You Mean..." Pattern

Git pioneered this. When a user types `git poll`, Git responds:

```
git: 'poll' is not a git command. Did you mean this?
    pull
```

Implementation: compute Levenshtein distance or similar string similarity against known commands. Suggest when distance is below a threshold.

Cargo does this for subcommands and feature names. clap provides this automatically for subcommands and flag names.

### Exit Codes

Standard convention:
- `0` -- success
- `1` -- general failure
- `2` -- misuse of command (bad arguments)
- Higher codes for specific error categories (some tools use `101` for panics, etc.)

Cargo uses `101` for compilation errors, allowing scripts to distinguish "cargo failed" from "the code failed to compile."

### Structured Error Output

Rust's compiler pioneered rich error formatting:
```
error[E0308]: mismatched types
 --> src/main.rs:3:24
  |
3 |     let x: i32 = "hello";
  |            ---   ^^^^^^^ expected `i32`, found `&str`
  |            |
  |            expected due to this
```

Elements: error code, source location, inline annotations, explanation. This pattern has been adopted by many modern tools (elm, deno, zig).

Source: [clig.dev](https://clig.dev/), [Make Your CLI a Joy to Use](https://www.caduh.com/blog/make-your-cli-a-joy-to-use)


## 4. Configuration Cascade

### Standard Precedence Order

Highest to lowest priority:

1. **Command-line flags** -- explicit, per-invocation
2. **Environment variables** -- per-shell/session
3. **Project-level config** (`.env`, tool-specific config in project root)
4. **User-level config** (`~/.config/<tool>/config.toml` per XDG)
5. **System-wide config** (`/etc/<tool>/config`)
6. **Hardcoded defaults**

This is the pattern used by Cargo (`.cargo/config.toml` at multiple directory levels), Go (env vars + `go.env`), npm (`.npmrc` at project/user/global levels), and AWS CLI.

Source: [clig.dev](https://clig.dev/), [ConfigArgParse](https://pypi.org/project/ConfigArgParse/)

### Cargo's Config Cascade

Cargo searches for `.cargo/config.toml` from the current directory upward to the filesystem root. Each level can override settings from parent levels. Environment variables prefixed with `CARGO_` override config file values. CLI flags override everything.

Source: [Cargo Configuration](https://doc.rust-lang.org/cargo/reference/config.html)

### Config File Locations (XDG)

Follow the [XDG Base Directory Specification](https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html):

- Config: `$XDG_CONFIG_HOME/<tool>/` (default `~/.config/<tool>/`)
- Data: `$XDG_DATA_HOME/<tool>/` (default `~/.local/share/<tool>/`)
- Cache: `$XDG_CACHE_HOME/<tool>/` (default `~/.cache/<tool>/`)
- State: `$XDG_STATE_HOME/<tool>/` (default `~/.local/state/<tool>/`)

### Environment Variable Naming

Conventions:
- Uppercase with underscores: `CARGO_TARGET_DIR`, `GOPATH`, `DENO_DIR`
- Tool prefix to avoid collision: `LX_CONFIG`, not just `CONFIG`
- Check standard variables: `NO_COLOR`, `TERM`, `HOME`, `EDITOR`, `PAGER`, `HTTP_PROXY`, `TMPDIR`
- Single-line values only (keeps `env` output readable)
- Never store secrets in env vars (use files, pipes, or secret managers)


## 5. Output Formats

### Human-Readable vs Machine-Readable

From [clig.dev](https://clig.dev/) and the [Heroku CLI Style Guide](https://devcenter.heroku.com/articles/cli-style-guide):

**Human-readable** (default when stdout is a TTY):
- Color-coded output
- Tables with alignment
- Progress indicators
- Friendly messages ("Compiled 42 crates in 3.2s")

**Machine-readable** (when piped or explicitly requested):
- `--json` flag for structured JSON output
- `--plain` flag for tab-separated, one-record-per-line output
- Stable field names and types across versions

### TTY Detection

Detect whether stdout is a terminal:
- Rust: `std::io::stdout().is_terminal()` (or `atty` crate)
- Go: `os.Stdout.Stat()` and check for `ModeCharDevice`
- Node: `process.stdout.isTTY`

When not a TTY: disable color, disable progress bars, use simpler formatting.

### Color Control

Standard signals to disable color:
1. `NO_COLOR` environment variable (any value) -- see [no-color.org](https://no-color.org/)
2. `TERM=dumb`
3. `--no-color` flag
4. stdout is not a TTY

Tool-specific override: `MYAPP_NO_COLOR` or `MYAPP_COLOR=always/never/auto`.

Cargo uses `--color=auto|always|never`. Go tools produce no color by default.

### Verbosity Levels

Common pattern:
- `-q` / `--quiet` -- suppress non-essential output
- Default -- normal output
- `-v` / `--verbose` -- extra detail
- `-vv` or `-v -v` -- debug-level output (cargo, some tools)

### Streaming and Progress

**Progress bars**: Use for operations > 2 seconds. Cargo shows a progress bar during dependency downloads. Libraries: `indicatif` (Rust), `tqdm` (Python), `progress` (Go), `ora` (Node.js).

**Spinners**: Use for indeterminate operations. Show activity without percentage.

**Multi-progress**: `indicatif::MultiProgress` for parallel operations (downloading multiple crates simultaneously).

**Key rule**: Never show animations when stdout is not a TTY. Write progress to stderr so stdout remains pipeable.

Source: [indicatif](https://github.com/console-rs/indicatif), [clig.dev](https://clig.dev/), [Heroku CLI Style Guide](https://devcenter.heroku.com/articles/cli-style-guide)


## 6. Shell Completions

### Generation Approaches

**Static generation**: Generate completion scripts at build time. Ship them with the package or let users generate via `<tool> completions <shell>`.

**Dynamic generation**: Generate completions at shell startup. Self-updating as the tool evolves:
```bash
# .bashrc
source <(COMPLETE=bash my-tool)

# .zshrc
source <(COMPLETE=zsh my-tool)
```

### clap_complete (Rust)

The `clap_complete` crate generates completions for bash, zsh, fish, elvish, PowerShell from the clap command definition. Two modes:

1. **Build-time**: Generate scripts during `cargo build` via a build script
2. **Runtime**: `my-tool completions bash` prints the script to stdout

Dynamic completions (clap_complete::env) provide context-aware completions that query the tool at completion time for valid values.

Source: [clap_complete docs](https://docs.rs/clap_complete/latest/clap_complete/env/index.html), [Kevin Knapp's blog on shell completions](https://kbknapp.dev/shell-completions/)

### Cobra (Go)

Cobra auto-generates completions for bash, zsh, fish, PowerShell. `rootCmd.GenBashCompletionV2(os.Stdout, true)` or via a `completion` subcommand. Supports custom completion functions for dynamic values.

Source: [cobra completions](https://github.com/spf13/cobra/blob/main/completions.go)

### What to Complete

- Subcommand names
- Flag names (long and short)
- Flag values (from enum-like sets)
- File paths (with extension filtering)
- Dynamic values (package names, test names, available targets)


## 7. Init / Scaffolding Commands

### Approaches by Tool

**Cargo** (`cargo new` / `cargo init`):
- `cargo new <name>` -- creates new directory with `Cargo.toml`, `src/main.rs` or `src/lib.rs`
- `cargo init` -- initializes in existing directory
- `--lib` vs `--bin` flag for library or binary
- `--vcs git|hg|pijul|fossil|none`
- `--edition 2021` for Rust edition
- `--name` to override package name
- Minimal: creates 2-3 files, no prompts

**Go** (`go mod init`):
- `go mod init <module-path>` -- creates `go.mod` with module path
- Minimal: creates 1 file, no prompts, no scaffolding
- Module path follows URL convention: `github.com/user/repo`

**Mix** (`mix new`):
- Creates project directory with `mix.exs`, `lib/`, `test/`, `.formatter.exs`, `.gitignore`, `README.md`
- `--sup` flag to include a supervision tree
- `--umbrella` flag for umbrella project structure
- `--app <name>` to set OTP application name
- `--module <Name>` to set main module name

**Deno** (`deno init`):
- Creates `main.ts`, `main_test.ts`, `deno.json`
- Minimal scaffolding, quick start

**npm** (`npm init`):
- Interactive prompts for name, version, description, entry point, test command, git repo, keywords, license
- `npm init -y` skips prompts with defaults
- `npm init <initializer>` runs `create-<initializer>` package (e.g., `npm init react-app`)

### Design Considerations

- **Non-interactive by default**: Cargo, Go, Deno create files without prompts. Fastest path to working code
- **Interactive when complex**: npm prompts because `package.json` has many fields
- **Template support**: Some tools support project templates (`cargo-generate`, `npm init <template>`)
- **Idempotent**: `cargo init` works in existing directories, `go mod init` can be re-run


## 8. Watch Mode

### Built-in vs External

**Built-in**: Deno (`--watch`), Mix (`mix test --stale`), Node.js (`--watch`)

**External**: Cargo (`cargo-watch`), Node.js (`nodemon`), general-purpose (`watchexec`, `entr`)

### How Watch Mode Works

1. **File system monitoring**: Use OS-level APIs (inotify on Linux, FSEvents on macOS, ReadDirectoryChangesW on Windows). Rust crate: `notify`.
2. **Debouncing**: Delay execution after last file change to batch rapid saves. Typical defaults: 50ms (watchexec), 500ms (cargo-watch), 1000ms (nodemon).
3. **Re-execution**: Kill previous process, re-run command. Some tools support HMR (hot module replacement) instead of full restart.
4. **Filtering**: Ignore patterns for `target/`, `node_modules/`, `.git/`, build artifacts.

### cargo-watch

```bash
cargo watch -x check                    # re-check on changes
cargo watch -x test                     # re-test on changes
cargo watch -x 'run -- --some-arg'      # re-run with args
cargo watch -x check -x test -x run     # chain commands
cargo watch -w src -w tests             # watch specific dirs
cargo watch --delay 2                   # 2 second debounce
```

Source: [cargo-watch](https://crates.io/crates/cargo-watch)

### Deno Watch

```bash
deno run --watch main.ts                # restart on change
deno run --watch-hmr main.ts            # hot module replacement
deno test --watch                       # re-test on change
deno fmt --watch                        # re-format on change
```

Source: [Deno CLI reference](https://docs.deno.com/runtime/getting_started/command_line_interface/)

### watchexec

General-purpose file watcher, written in Rust. Powers cargo-watch internally. Supports any command, any file types, configurable debouncing, signal handling.

Source: [watchexec](https://github.com/watchexec/watchexec)


## 9. Plugin / Extension Systems

### Discovery Patterns

**PATH-based (Cargo)**: Any binary named `cargo-<name>` in `$PATH` becomes `cargo <name>`. Zero registration. Install with `cargo install cargo-<name>`. Pros: dead simple, works with any language. Cons: no discovery mechanism, no metadata.

**Registry-based (npm scripts)**: Commands defined in `package.json` `scripts`. `npm run <name>` executes them. Lifecycle hooks (`pre<name>`, `post<name>`) wrap execution. Not true plugins, but serves the same purpose.

**Framework-based (oclif)**: Plugins are npm packages following oclif conventions. `heroku plugins:install <name>`. The framework manages plugin lifecycle, provides hook points, handles conflicts. Pros: structured, discoverable. Cons: framework lock-in.

**Task-based (Mix)**: Any module implementing `Mix.Task` behaviour in the code path becomes a command. Install via `mix deps.get` of a hex package that defines tasks. Tasks are discovered at runtime by scanning loaded modules.

**Built-in task runner (Deno, Just)**: Tasks defined in config file (`deno.json`, `justfile`). No plugin system needed; the task runner is the extension point.

Source: [Extending Cargo](https://doc.rust-lang.org/book/ch14-05-extending-cargo.html), [oclif](https://oclif.io/), [Heroku CLI plugins](https://devcenter.heroku.com/articles/developing-cli-plugins)

### Comparison

| Approach | Discovery | Isolation | Metadata | Examples |
|----------|-----------|-----------|----------|----------|
| PATH convention | Automatic | Process-level | None | Cargo, Git |
| Config scripts | Explicit | Shell | Defined in config | npm, Just, Deno |
| Framework plugins | Registry | Framework-managed | Structured | oclif, Heroku CLI |
| Behaviour/trait | Code scan | In-process | Module attributes | Mix |
| Build-system steps | Build file | Build graph | Build API | Zig build.zig |


## 10. CLI Libraries

### Rust

**clap** (Command Line Argument Parser):
The dominant Rust CLI library. Powers ripgrep, bat, fd, cargo-nextest, and hundreds of production tools.

Two APIs:
- **Derive API**: Define CLI via `#[derive(Parser)]` on structs and enums. Subcommands via `#[derive(Subcommand)]`. Less boilerplate, compile-time checked.
- **Builder API**: Construct commands programmatically via `Command::new()`. More flexible, runtime construction.

Key features:
- Subcommands via enum variants with `#[derive(Subcommand)]`
- `#[derive(ValueEnum)]` for flag values from a fixed set
- Argument groups for mutually exclusive options
- Custom validators on arguments
- Automatic `--help`, `--version` generation
- Environment variable fallbacks via `#[arg(env = "MY_VAR")]`
- Shell completions via `clap_complete`

Source: [clap docs](https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html), [clap derive reference](https://docs.rs/clap/latest/clap/_derive/index.html)

**argh** (Google):
Minimal CLI parser. Derive-based like clap but with much smaller binary size overhead. Follows Fuchsia OS conventions rather than POSIX. Missing features: no `--flag=value` syntax, no combined short flags (`-abc`), no automatic suggestions. Best for internal tools where binary size matters and POSIX compliance doesn't.

Source: [Rain's Rust CLI recommendations](https://rust-cli-recommendations.sunshowers.io/cli-parser.html)

### Go

**Cobra + pflag**:
The de facto Go CLI library. Used by Kubernetes, Hugo, GitHub CLI, Docker CLI, Helm.

Pattern: `APPNAME COMMAND ARG --FLAG`.

Architecture:
- `cobra.Command` struct: `Use`, `Short`, `Long`, `Run`, `RunE` (with error return), `PersistentPreRun`, `PreRun`, `PostRun`, `PersistentPostRun`
- Persistent flags: inherited by all child commands (`rootCmd.PersistentFlags()`)
- Local flags: per-command only (`cmd.Flags()`)
- Auto-generated completions for bash, zsh, fish, PowerShell
- `cobra-cli` scaffold tool for generating command files

Best practice: pair with **Viper** for configuration (env vars, config files, remote config).

Source: [cobra](https://github.com/spf13/cobra), [cobra.dev](https://cobra.dev/)

**urfave/cli**:
Simpler alternative to Cobra. Single-struct definition instead of command tree. Good for small tools.

### Python

**argparse** (stdlib):
Built into Python. Positional args, optional args, subcommands via `add_subparsers()`. Verbose but no dependencies.

**Click**:
Decorator-based. `@click.command()`, `@click.group()`, `@click.option()`, `@click.argument()`. Context object passes state between commands. Supports command groups (nested subcommands), prompts, password input, file handling, color output.

```python
@click.group()
def cli():
    pass

@cli.command()
@click.option('--name', prompt='Your name')
def hello(name):
    click.echo(f'Hello {name}!')
```

**Typer**:
Built on Click. Uses Python type hints instead of decorators. Function signature becomes the CLI interface:

```python
import typer
app = typer.Typer()

@app.command()
def hello(name: str, formal: bool = False):
    greeting = "Good day" if formal else "Hello"
    print(f"{greeting} {name}")
```

Source: [Click docs](https://click.palletsprojects.com/), [Typer docs](https://typer.tiangolo.com/)

### JavaScript/TypeScript

**commander**:
Simple, declarative. `.command()`, `.option()`, `.action()`. Widely used.

**yargs**:
Feature-rich. Chaining API. Built-in completion generation. Good for complex CLIs.

**oclif** (Salesforce/Heroku):
Full framework, not just a parser. Class-based commands, plugin system, auto-generated help, testing infrastructure, TypeScript-first. Used for Heroku CLI, Salesforce CLI.

Source: [oclif](https://oclif.io/)


## 11. Caching and Incremental Builds

### Fingerprinting (Cargo)

Cargo stores fingerprints in `target/{debug,release}/.fingerprint/`. Each compilation unit gets a directory with a hash file. Inputs to the hash:
- Source file contents
- Dependency versions and features
- Compiler version and flags
- Profile settings (opt-level, debug, etc.)
- Environment variables used by the build
- Build script outputs

If fingerprint matches, the unit is skipped. If the unit compiles successfully, the fingerprint is updated. If compilation fails, the old fingerprint is preserved (so the next build retries).

Source: [cargo fingerprint module](https://doc.rust-lang.org/beta/nightly-rustc/cargo/core/compiler/fingerprint/index.html)

### Go Build Cache

Go caches compiled packages in `$GOMODCACHE` (default `~/go/pkg/mod`). Build cache lives in `$GOCACHE` (default `~/.cache/go-build`). Cache key: source file hash + import path + build flags. `go clean -cache` clears it.

Go test results are also cached. If inputs (source, test files, env) haven't changed, `go test` prints `(cached)` and skips execution. Force re-run with `go test -count=1`.

Source: [Inside the Go Build Cache](https://medium.com/cloud-native-daily/inside-the-go-build-cache-and-the-incremental-build-mechanism-52f0da94f457)

### General Principles

- Cache by content hash, not by timestamp (timestamps break with git checkout, CI, etc.)
- Store cache alongside build artifacts or in a user-level cache directory
- Provide `clean` command to reset cache
- Show cache hit/miss in verbose mode
- Consider remote/shared caches for CI (sccache for Rust, Go's module proxy)


## 12. Parallel Execution

### Test Parallelism

**Cargo test**: Tests within a binary run in parallel (controlled by `--test-threads`). Multiple test binaries run sequentially by default. `cargo-nextest` runs each test as a separate process, in parallel, with configurable concurrency.

**Go test**: Tests within a package run serially by default. `t.Parallel()` opts individual tests into parallel execution. Multiple packages run in parallel (controlled by `-p`, default GOMAXPROCS). `-parallel` flag controls per-package concurrency.

**Mix test**: `mix test --max-cases 8` controls parallelism. Tests are async by default.

**npm/pnpm**: pnpm runs workspace scripts in parallel respecting the dependency graph. Independent packages run simultaneously; dependent packages wait.

### Build Parallelism

**Cargo**: `-j` / `--jobs` flag controls parallel rustc invocations (default: number of CPUs). Parallelism is at the crate level; individual crate compilation uses codegen units for intra-crate parallelism.

**Go**: `-p` flag for package-level parallelism. `GOMAXPROCS` for goroutine scheduling.

**Zig**: Build DAG enables concurrent step execution automatically.

Source: [cargo-nextest](https://nexte.st/docs/design/how-it-works/), [Go test parallelism](https://threedots.tech/post/go-test-parallelism/)


## 13. Cross-Platform Considerations

### Path Handling

- Use `std::path::Path` (Rust), `filepath.Join` (Go), `path.join` (Node), `pathlib.Path` (Python)
- Never hardcode `/` or `\`
- Handle case-insensitive filesystems (macOS, Windows)
- Handle long paths on Windows (> 260 chars) with `\\?\` prefix

### Terminal Capabilities

- Check `TERM` environment variable (`dumb` means no capabilities)
- Enable `ENABLE_VIRTUAL_TERMINAL_PROCESSING` on Windows for ANSI color support
- Use `COLUMNS` and `LINES` for terminal dimensions (or ioctl)
- Handle UTF-8 encoding: Python needs `PYTHONUTF8=1` on Windows

### Shell Differences

- Unix: `/bin/sh`, `/bin/bash`, `/bin/zsh`
- Windows: `cmd.exe`, PowerShell, Git Bash
- Just handles this via `set shell := [...]` in the justfile
- Cargo runs build scripts with `sh` on Unix, `cmd` on Windows

### Process Model

- Unix: `fork()` + `exec()`, signal handling (SIGINT, SIGTERM)
- Windows: `CreateProcess()`, no fork, different signal model
- Ctrl+C: Should terminate gracefully. Second Ctrl+C should force-quit.
- Use process groups to kill child processes on parent exit

Source: [Five Considerations for Cross-Platform Tools](https://semgrep.dev/blog/2025/five-considerations-when-building-cross-platform-tools-for-windows-and-macos/), [clig.dev](https://clig.dev/)


## 14. Standard Flags and Arguments

### Widely Recognized Flags

From [clig.dev](https://clig.dev/):

| Flag | Purpose |
|------|---------|
| `-h`, `--help` | Show help |
| `--version` | Show version |
| `-v`, `--verbose` | Increase output detail |
| `-q`, `--quiet` | Suppress non-essential output |
| `-f`, `--force` | Skip confirmations |
| `-n`, `--dry-run` | Preview without executing |
| `-o`, `--output` | Output file/directory |
| `--json` | JSON output |
| `--no-color` | Disable color |
| `--no-input` | Disable prompts |
| `-a`, `--all` | Apply to all items |
| `-d`, `--debug` | Debug output |
| `-p`, `--port` | Port number |

### Argument Design Rules

- Prefer flags to bare arguments for clarity and future flexibility
- Provide both short (`-v`) and long (`--verbose`) forms for common flags
- Reserve single-letter flags for frequently used options
- Make arguments order-independent where possible
- Support `--` to separate tool flags from passthrough arguments
- Support `-` to mean stdin/stdout for file arguments
- Never read secrets from flags (visible in `ps` output). Use `--password-file` or stdin.


## 15. Summary: Patterns for lx

Based on this research, key patterns relevant to lx's CLI (with subcommands: run, test, check, init, install, update, agent, diagram, list):

**Subcommand design**: lx's commands are verb-first (run, test, check) and noun-first (agent, diagram) -- a pragmatic mix. This matches Go's approach.

**Help**: Lead with examples in `--help`. Show most common usage first. Keep it scannable.

**Errors**: Suggest corrections for typos. Show source locations for parse/runtime errors. Use color sparingly but effectively.

**Configuration**: CLI flags > env vars (`LX_*`) > project config (`lx.toml`) > defaults.

**Output**: Detect TTY. Default to human-readable. Support `--json` for scriptability. Respect `NO_COLOR`.

**Completions**: Use `clap_complete` for bash/zsh/fish. Add a `completions` subcommand.

**Watch**: Consider built-in `--watch` on `run` and `test` (like Deno) rather than requiring external tools.

**Extensibility**: lx's `agent` subcommand is inherently extensible (agents are programs). Consider allowing user-defined subcommands via `lx.toml` tasks (like Deno tasks) or PATH-based discovery (like Cargo plugins).

Sources:
- [Command Line Interface Guidelines (clig.dev)](https://clig.dev/)
- [Heroku CLI Style Guide](https://devcenter.heroku.com/articles/cli-style-guide)
- [System.CommandLine design guidance](https://learn.microsoft.com/en-us/dotnet/standard/commandline/design-guidance)
- [clap docs](https://docs.rs/clap/latest/clap/)
- [clap_complete](https://docs.rs/clap_complete/latest/clap_complete/)
- [cobra](https://github.com/spf13/cobra)
- [indicatif](https://github.com/console-rs/indicatif)
- [cargo-watch](https://crates.io/crates/cargo-watch)
- [watchexec](https://github.com/watchexec/watchexec)
- [oclif](https://oclif.io/)
- [no-color.org](https://no-color.org/)
- [cargo-xtask](https://github.com/matklad/cargo-xtask)
- [cargo fingerprint module](https://doc.rust-lang.org/beta/nightly-rustc/cargo/core/compiler/fingerprint/index.html)
- [cargo-nextest](https://nexte.st/docs/design/how-it-works/)
- [Zig Build System](https://ziglang.org/learn/build-system/)
- [Deno CLI](https://docs.deno.com/runtime/getting_started/command_line_interface/)
- [Go command](https://pkg.go.dev/cmd/go)
- [Mix.Task](https://hexdocs.pm/mix/Mix.Task.html)
- [Poetry CLI](https://python-poetry.org/docs/cli/)
- [uv docs](https://docs.astral.sh/uv/)
- [npm scripts](https://docs.npmjs.com/cli/v11/using-npm/scripts/)
- [pnpm filtering](https://pnpm.io/filtering)
- [just manual](https://just.systems/man/en/)
