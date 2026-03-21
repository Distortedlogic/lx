# Shell Integration Design Patterns: Security, Error Handling, I/O, and Cross-Platform

Cross-cutting concerns for shell/subprocess execution in programming languages. Context: lx uses `$cmd` (returns `Result<{out, err, code}, {cmd, msg}>`), `$^cmd` (returns stdout string on success, propagates error on failure), and `${expr}` for interpolation inside shell commands.

---

## 1. Security: Shell Injection Prevention

### 1.1 The Fundamental Problem

Shell injection occurs when user-controlled input reaches a shell interpreter. The core issue is that POSIX `execve()` takes an argument array, but many languages wrap it with a shell layer that re-parses a single string. Metacharacters (`;`, `|`, `&&`, `$()`, backticks, `>`, `<`) in that string can inject arbitrary commands.

Example (Node.js):

```javascript
// UNSAFE: user controls `filename`
exec(`cat ${filename}`);
// If filename = "; rm -rf /", the shell executes both commands
```

Source: [matklad: Shell Injection](https://matklad.github.io/2021/07/30/shell-injection.html)

### 1.2 Defense Hierarchy

From strongest to weakest:

**1. Avoid shell execution entirely.** Use language-native APIs (filesystem, HTTP, etc.) instead of shelling out. OWASP: "Built-in library functions are a very good alternative to OS commands, as they cannot be manipulated to perform tasks other than those intended."

**2. Argument arrays (no shell).** Pass command + arguments as separate elements. The process is exec'd directly via `execve()`, bypassing shell parsing entirely:

| Language | Safe API |
|---|---|
| Python | `subprocess.run(["ls", "-la", path])` |
| Ruby | `system("ls", "-la", path)` |
| Node.js | `spawn("ls", ["-la", path])` / `execFile` |
| Go | `exec.Command("ls", "-la", path)` |
| Rust | `Command::new("ls").arg("-la").arg(path)` |
| Perl | `system("ls", "-la", $path)` |
| Java | `Runtime.exec(new String[]{"ls", "-la", path})` |

**3. Escaping/quoting.** Language-specific escaping functions wrap strings for safe inclusion in shell commands. This is a fallback, not a primary defense:

| Language | Escaping Function |
|---|---|
| Python | `shlex.quote(s)` |
| Ruby | `Shellwords.escape(s)` / `String#shellescape` |
| Perl | manual regex validation in taint mode |
| PHP | `escapeshellarg(s)` / `escapeshellcmd(s)` |
| Bash | quoting (`"$var"`) |

Limitation: escaping protects against metacharacter injection but NOT argument injection (e.g., a value like `--delete` being interpreted as a flag).

**4. Input validation (allowlist).** Validate input against a known-good pattern before using it in any command:

```python
import re
if not re.match(r'^[a-zA-Z0-9_.-]+$', filename):
    raise ValueError("Invalid filename")
```

Source: [OWASP Command Injection Defense](https://cheatsheetseries.owasp.org/cheatsheets/OS_Command_Injection_Defense_Cheat_Sheet.html), [CISA: Eliminating OS Command Injection](https://www.cisa.gov/resources-tools/resources/secure-design-alert-eliminating-os-command-injection-vulnerabilities)

### 1.3 How Languages Default

Languages split into two camps on API design:

**Safe by default** (argument arrays, no shell):

- **Go**: `os/exec` has no shell mode at all. You must explicitly invoke `sh -c` to get shell features.
- **Rust**: `std::process::Command` passes arguments directly. No shell expansion.
- **Node.js spawn/execFile**: default `shell: false`.
- **Python subprocess.run**: default `shell=False`.

**Unsafe by default** (string to shell):

- **Perl backticks/qx//**: always invoke `/bin/sh -c`.
- **Ruby backticks/%x{}**: always invoke `/bin/sh -c`.
- **Node.js exec()**: always uses shell.
- **Python os.system()**: always uses shell.
- **Lua os.execute/io.popen**: always use shell.
- **C system()**: always uses shell.

### 1.4 Template Literal Safety (Execa/zx Pattern)

JavaScript's execa and zx use tagged template literals to achieve automatic escaping:

```javascript
const userInput = "file; rm -rf /";
await $`cat ${userInput}`;  // userInput treated as single argument
```

The template tag receives the static string parts and dynamic values separately, allowing the library to escape values before constructing the command. This is a language-level solution: the separation of template and values happens at parse time, not runtime string manipulation.

This pattern is relevant to lx's `${}` interpolation blocks.

### 1.5 Perl Taint Mode: Compile-Time Tracking

Perl's taint mode tracks data provenance at runtime. Variables derived from external input are "tainted" and cannot be used in shell commands, `eval`, or file operations until explicitly untainted via regex capture. This is unique among mainstream languages -- no other language has a comparable built-in mechanism for tracking data flow to shell execution points.

### 1.6 Windows-Specific Injection Risks

On Windows, even "safe" argument-array APIs have risks:

- Rust's `Command` auto-converts `.bat` file execution into `cmd.exe /c`, which re-parses arguments with `cmd.exe` metacharacter rules.
- Go's `exec.Command` on Windows uses `CreateProcess`, which concatenates arguments into a command line string with Windows escaping rules.
- Python's `subprocess.run` on Windows with `shell=True` uses `cmd.exe /c`, where `%VAR%` expansion, `&`, `|`, `^` are all dangerous.

The argument array defense is weaker on Windows because `CreateProcess` ultimately takes a single command-line string, and each program is responsible for its own argument parsing (via `CommandLineToArgvW` or custom logic).

Source: [Rust Command docs: Windows caveat](https://doc.rust-lang.org/std/process/struct.Command.html)

---

## 2. Error Handling

### 2.1 Exit Code Conventions

POSIX conventions:

- `0`: success
- `1`: general errors
- `2`: misuse of shell command
- `126`: command not executable
- `127`: command not found
- `128+N`: killed by signal N (e.g., `137` = killed by SIGKILL/signal 9)
- `255`: exit status out of range

### 2.2 How Languages Surface Errors

**Return-code inspection** (caller must check):

| Language | Mechanism |
|---|---|
| Bash | `$?`, `${PIPESTATUS[@]}` |
| Perl | `$? >> 8` |
| Ruby | `$?.exitstatus` |
| Python `run()` | `result.returncode` |
| Go | `err.(*exec.ExitError)` |
| Rust | `output.status.code()` |
| Nushell | `$env.LAST_EXIT_CODE` |
| PowerShell | `$LASTEXITCODE` |

**Exception/error on failure** (opt-in or default):

| Language | Mechanism |
|---|---|
| Python | `check=True` raises `CalledProcessError` |
| Go | `Run()` returns `*ExitError` for nonzero |
| Rust | manual check on `ExitStatus.success()` |
| Amber | `failed` block is mandatory (compile-time enforced) |
| lx `$^` | propagates error value on nonzero exit |

**Structured error capture**:

| Language | Returns |
|---|---|
| Nushell `complete` | record: `{stdout, stderr, exit_code}` |
| lx `$cmd` | `Ok({out, err, code})` or `Err({cmd, msg})` |
| Ruby `Open3.capture3` | `[stdout, stderr, status]` |
| Python `run()` | `CompletedProcess(returncode, stdout, stderr)` |

### 2.3 Bash Error Handling: The set -e Problem

`set -e` (errexit) is bash's attempt at "fail on error." It has well-documented problems:

- Does not apply inside command substitution `$(...)` (until `inherit_errexit` in Bash 4.4)
- Does not apply in commands tested by `if`, `while`, `||`, `&&`
- Subshells may or may not inherit it
- Process substitution `<(cmd)` exit codes are invisible (until Bash 4.4 / Zsh 5.6)

`set -o pipefail` addresses pipeline exit codes (rightmost nonzero) but introduces its own complexity: `cmd1 | cmd2` fails if `cmd1` exits nonzero, even if `cmd2` succeeds.

YSH fixes this with explicit `try` / `_error`:

```ysh
try { ls /nonexistent }
if (_error.code !== 0) { echo "failed" }
```

Amber fixes it by making error handling mandatory at compile time.

Source: [BashFAQ/105](https://mywiki.wooledge.org/BashFAQ/105), [Robust error handling in Bash](https://dev.to/banks/stop-ignoring-errors-in-bash-3co5)

### 2.4 Timeout and Kill

| Language | Timeout Mechanism |
|---|---|
| Python | `subprocess.run(..., timeout=30)` -- raises `TimeoutExpired`, kills child |
| Go | `exec.CommandContext(ctx, ...)` -- context cancellation kills process |
| Rust | manual: `child.wait_timeout()` or tokio's `timeout()` |
| Node.js | `exec({timeout: 30000})` or `AbortController` signal |
| Perl | `IPC::Run` supports `timeout()` |
| zx | `$`cmd`.timeout('30s')` |
| Bash | `timeout 30 command` (coreutils) |

Python's timeout story:

```python
try:
    result = subprocess.run(["slow_cmd"], timeout=30, capture_output=True)
except subprocess.TimeoutExpired as e:
    # e.stdout and e.stderr contain partial output
    print(f"timed out after {e.timeout}s")
```

### 2.5 Stderr Capture Patterns

Three approaches across languages:

**Separate capture** (stdout and stderr as distinct values):

```python
result = subprocess.run(cmd, capture_output=True)
# result.stdout, result.stderr are separate
```

**Combined capture** (stdout and stderr merged):

```go
out, err := cmd.CombinedOutput()  // interleaved
```

**Structured capture** (exit code + stdout + stderr as a record):

```nu
let r = do { ^cmd } | complete
# r.stdout, r.stderr, r.exit_code
```

lx's `$cmd` returns `Ok({out, err, code})`, which is the structured capture pattern.

---

## 3. Interpolation and Escaping

### 3.1 Variable Interpolation Strategies

Languages use different strategies to interpolate values into shell commands:

**String concatenation** (most dangerous):

```python
os.system("cat " + filename)  # injection via filename
```

**Format strings** (still dangerous if shell=True):

```python
subprocess.run(f"cat {filename}", shell=True)  # still injectable
```

**Argument arrays** (safe by construction):

```python
subprocess.run(["cat", filename])  # filename is a single argument
```

**Tagged templates** (safe, preserves ergonomics):

```javascript
await $`cat ${filename}`;  // filename auto-escaped
```

**Language-level interpolation** (depends on implementation):

```
-- lx
$cat ${filename}   -- interpolated into shell string
```

lx's `${}` interpolation builds a shell command string via string concatenation, then passes it to `sh -c`. This is the string-concatenation approach, which means the value of the interpolated expression is embedded in the shell string without escaping.

### 3.2 Quoting Rules Across Shells

| Shell | Single quotes | Double quotes | Escape char |
|---|---|---|---|
| Bash | literal (no expansion) | `$`, `` ` ``, `"`, `\` expanded | `\` |
| Zsh | literal | same as bash | `\` |
| Fish | literal | only `$var` and `$(cmd)` | `\` (unquoted only) |
| PowerShell | literal | `$var` expanded | `` ` `` (backtick) |
| cmd.exe | not special | groups arguments | `^` |

Fish's simplified model (only `$` in double quotes) eliminates many bash quoting bugs. YSH takes a similar approach.

### 3.3 Word Splitting and Globbing

The interaction of variable expansion, word splitting, and globbing is the #1 source of shell scripting bugs:

**Bash** (dangerous default): Unquoted `$var` undergoes word splitting (on `$IFS`) and then glob expansion. `"$var"` prevents both.

**Zsh** (safer default): Unquoted `$var` does NOT undergo word splitting by default. Glob expansion still applies.

**Fish** (safest): No word splitting, no glob expansion on variable values. Variables always expand to their exact value as a single argument.

**YSH**: No word splitting on variable expansion. Glob expansion is opt-in.

The shellharden project documents safe bash patterns exhaustively.

Source: [shellharden: How to do things safely in bash](https://github.com/anordal/shellharden/blob/master/how_to_do_things_safely_in_bash.md), [ShellCheck SC2086](https://www.shellcheck.net/wiki/SC2086)

### 3.4 OS-Specific Escaping Differences

Unix shells and Windows cmd.exe use fundamentally different escaping:

| Aspect | Unix (/bin/sh) | Windows (cmd.exe) |
|---|---|---|
| Quote style | single quotes prevent all expansion | double quotes group args |
| Escape character | `\` | `^` |
| Variable expansion | `$VAR` | `%VAR%` |
| Dangerous chars | `; | & $ ( ) ` ` > <` | `& | ( ) < > ^ %` |
| Command separator | `;` or `&&` | `&` or `&&` |

PowerShell uses yet another escaping model (backtick `` ` `` as escape, `$` for variables). Languages that abstract over shells must handle all three.

---

## 4. Streaming I/O

### 4.1 Pipe Architecture

Subprocess I/O uses OS-level pipes:

```
Parent Process                  Child Process
  write end  ──────pipe──────>  stdin  (fd 0)
  read end   <─────pipe──────  stdout (fd 1)
  read end   <─────pipe──────  stderr (fd 2)
```

Each pipe has a kernel buffer (typically 64KB on Linux, 16KB on macOS). If the buffer fills and the reader doesn't consume, the writer blocks. This is the source of deadlocks.

### 4.2 Deadlock Prevention

The classic deadlock scenario: reading stdout, but the child fills the stderr buffer first. The child blocks writing stderr, the parent blocks reading stdout -- deadlock.

Solutions:

**communicate() / wait_with_output()**: Read stdout and stderr simultaneously (using threads or poll/select internally):

```python
out, err = proc.communicate()  # reads both, avoids deadlock
```

**Separate threads**: Read stdout and stderr on different threads:

```go
var stdout, stderr bytes.Buffer
cmd.Stdout = &stdout
cmd.Stderr = &stderr
cmd.Run()  // Go's Run() handles concurrent reads internally
```

**Merge streams**: Redirect stderr to stdout (`2>&1` or `stderr=subprocess.STDOUT`):

```python
subprocess.run(cmd, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
```

**Non-blocking I/O / poll**: Use OS-level multiplexing (epoll, kqueue, poll) to read whichever stream has data available.

### 4.3 Real-Time Output Capture

Many languages buffer subprocess output until completion. For real-time processing:

**Python**: Use `Popen` with line-by-line reading:

```python
proc = subprocess.Popen(cmd, stdout=subprocess.PIPE, text=True)
for line in proc.stdout:
    process(line)
proc.wait()
```

Caveat: many programs buffer stdout when it's not a TTY. Fix with `stdbuf -oL` or the program's own unbuffering option.

**Node.js**: Use `spawn()` with stream events:

```javascript
const child = spawn('cmd', ['arg']);
child.stdout.on('data', (chunk) => process(chunk));
```

**Go**: Use `StdoutPipe()` + `bufio.Scanner`:

```go
stdout, _ := cmd.StdoutPipe()
cmd.Start()
scanner := bufio.NewScanner(stdout)
for scanner.Scan() {
    process(scanner.Text())
}
cmd.Wait()
```

**Rust**: Use `spawn()` + `BufReader`:

```rust
let child = Command::new("cmd").stdout(Stdio::piped()).spawn()?;
let reader = BufReader::new(child.stdout.take().unwrap());
for line in reader.lines() {
    process(line?);
}
child.wait()?;
```

### 4.4 Buffering Modes

Programs typically use three buffering modes:

| Mode | When | Buffer size |
|---|---|---|
| Unbuffered | stderr (by convention) | 0 |
| Line-buffered | stdout to TTY | line |
| Fully buffered | stdout to pipe | 4KB--8KB |

When a parent captures stdout via pipe, the child's C library detects non-TTY and switches to full buffering. This means output arrives in chunks, not lines. Workarounds:

- `stdbuf -oL command`: force line buffering (GNU coreutils)
- `unbuffer command`: allocate a pty (expect package)
- `script -q /dev/null command`: similar pty trick (macOS/BSD)

### 4.5 Interactive Subprocess Communication

For bidirectional communication (write to stdin, read from stdout):

- **Perl IPC::Run**: supports pty allocation for interactive programs
- **Python pexpect**: pty-based expect library
- **Node.js node-pty**: pty allocation for terminal emulation
- **Go creack/pty**: pty package for interactive subprocesses
- **Rust portable-pty**: cross-platform pty support

PTY allocation is necessary when the child program detects whether it's running interactively (isatty check) and changes behavior accordingly.

---

## 5. Cross-Platform Concerns

### 5.1 Shell Differences

| Aspect | Unix | Windows |
|---|---|---|
| Default shell | `/bin/sh` (POSIX) | `cmd.exe` |
| Modern shell | bash, zsh, fish | PowerShell |
| Path separator | `/` | `\` (also accepts `/`) |
| Path list separator | `:` | `;` |
| Executable extension | none (uses `+x` bit) | `.exe`, `.bat`, `.cmd`, `.com` |
| Line ending | `\n` | `\r\n` |
| Null device | `/dev/null` | `NUL` |

### 5.2 How Languages Abstract

**Python subprocess**: On Windows with `shell=True`, uses `cmd.exe` via `COMSPEC`. With `shell=False`, uses `CreateProcess` directly. The `shlex` module is POSIX-only; on Windows it may produce incorrect results.

**Go os/exec**: Uses `execve` on Unix, `CreateProcess` on Windows. `LookPath` searches `PATH` and checks Windows executable extensions. The `ErrDot` security check applies on both platforms.

**Rust Command**: Uses `posix_spawn`/`fork`+`exec` on Unix, `CreateProcessW` on Windows. Windows-specific `.creation_flags()` and `.raw_arg()` methods. The `.bat` file auto-conversion to `cmd.exe /c` is Windows-only.

**Node.js**: Uses `posix_spawn` on Unix, `CreateProcess` on Windows. The `shell` option defaults to `/bin/sh` on Unix, `cmd.exe` on Windows. On Windows, `.bat` and `.cmd` files require `shell: true`.

### 5.3 Signal Handling Differences

| Signal | Unix | Windows |
|---|---|---|
| SIGTERM | graceful termination request | emulated by Node.js; Rust uses `TerminateProcess` |
| SIGKILL | immediate kill, no cleanup | `TerminateProcess` (always immediate) |
| SIGINT | Ctrl+C | `GenerateConsoleCtrlEvent` |
| SIGSTOP | suspend process | not supported |
| SIGHUP | hangup / terminal close | not supported |

Windows has no real signal mechanism. `TerminateProcess` is always immediate (like SIGKILL). Graceful shutdown on Windows requires:
- Console control events (Ctrl+C, Ctrl+Break)
- Window messages (WM_CLOSE, WM_QUIT)
- Named events or IPC

### 5.4 Process Creation Models

**Unix (fork + exec)**: The parent process is duplicated (`fork`), then the child replaces itself with the new program (`exec`). This enables `pre_exec` hooks (Rust), `preexec_fn` (Python), and arbitrary setup between fork and exec.

**Windows (CreateProcess)**: Creates a new process from scratch. No fork. Arguments are concatenated into a single command-line string, and each program parses it with its own logic (usually `CommandLineToArgvW`). This is why argument-array safety is weaker on Windows.

### 5.5 Portable Patterns

For cross-platform subprocess execution:

1. Use argument arrays (not shell strings) everywhere
2. Avoid shell-specific features (pipes, redirections) -- implement them in the host language
3. Use `/` for paths (Windows accepts it in most contexts)
4. Test on both platforms if targeting cross-platform
5. Be aware that exit code semantics differ (Windows uses 32-bit DWORD, Unix uses 8-bit + signal info)

---

## 6. Environment Handling

### 6.1 Inheritance Model

By default, child processes inherit the parent's environment:

| Language | Default | Override |
|---|---|---|
| Python | inherit all | `env={"KEY": "val"}` replaces entirely |
| Go | inherit all | `cmd.Env = [...]` replaces entirely |
| Rust | inherit all | `.env_clear()` + `.env("K", "V")` |
| Node.js | inherit all | `{env: {...}}` replaces entirely |
| Perl | inherit all | `local %ENV; $ENV{KEY} = "val"` |

Important: most languages' `env` parameter REPLACES the entire environment, not merges. To add to the existing environment:

```python
import os
env = os.environ.copy()
env["NEW_VAR"] = "value"
subprocess.run(cmd, env=env)
```

```rust
Command::new("prog")
    .env("NEW_VAR", "value")  // adds to inherited env
    .spawn()?;

// vs
Command::new("prog")
    .env_clear()
    .env("ONLY_VAR", "value")  // minimal env
    .spawn()?;
```

### 6.2 Security-Sensitive Variables

Variables that affect program loading and should be sanitized or cleared:

| Variable | Risk |
|---|---|
| `PATH` | attacker-controlled directory loaded first |
| `LD_PRELOAD` | arbitrary shared library injection (Linux) |
| `LD_LIBRARY_PATH` | library search path manipulation |
| `DYLD_LIBRARY_PATH` | macOS equivalent |
| `PYTHONPATH` | Python module injection |
| `NODE_PATH` | Node.js module injection |
| `PERL5LIB` | Perl module injection (blocked by taint mode) |
| `RUBYLIB` | Ruby library injection |

For security-critical subprocesses, use `env_clear()` (Rust) or an explicit minimal environment.

### 6.3 Working Directory

All languages support setting the child's working directory:

```python
subprocess.run(cmd, cwd="/path")
```

```go
cmd.Dir = "/path"
```

```rust
Command::new("prog").current_dir("/path")
```

The child's working directory is independent of the parent's. Relative paths in the command's arguments resolve against the child's cwd.

---

## 7. Process Management

### 7.1 PID Tracking

After spawning a child, languages provide access to its PID:

| Language | PID Access |
|---|---|
| Python | `proc.pid` |
| Go | `cmd.Process.Pid` |
| Rust | `child.id()` |
| Node.js | `child.pid` |
| Ruby | `spawn()` returns PID |
| Perl | return value of `fork()` or `open()` |

### 7.2 Process Groups

On Unix, related processes can be grouped under a process group ID (PGID). Killing the group kills all members:

```bash
kill -- -$PGID    # negative PID = process group
```

In languages:

```python
proc = subprocess.Popen(cmd, start_new_session=True)
# creates new session + process group
os.killpg(proc.pid, signal.SIGTERM)  # kill entire group
```

```go
cmd.SysProcAttr = &syscall.SysProcAttr{Setpgid: true}
// later: syscall.Kill(-cmd.Process.Pid, syscall.SIGTERM)
```

```rust
// Unix-specific: pre_exec to call setpgid
use std::os::unix::process::CommandExt;
unsafe {
    cmd.pre_exec(|| { libc::setpgid(0, 0); Ok(()) });
}
```

### 7.3 Zombie Processes

A zombie process has exited but hasn't been reaped (its exit status hasn't been read by the parent via `wait`). It retains its PID and process table entry.

Prevention:

- Always call `wait()` / `waitpid()` after `start()` / `spawn()` / `fork()`
- Python: `proc.communicate()` or `proc.wait()` -- also `proc.poll()` for non-blocking check
- Go: `cmd.Wait()` after `cmd.Start()`
- Rust: `child.wait()` or `child.wait_with_output()`
- Perl: `waitpid($pid, 0)` -- IPC::Open3 explicitly warns about this

If the parent exits without reaping, init (PID 1) adopts the zombie and reaps it. But long-running parent processes that spawn many children without reaping will leak PIDs.

### 7.4 Signal Handling

Graceful termination pattern:

```
1. Send SIGTERM (request graceful shutdown)
2. Wait with timeout
3. If still alive, send SIGKILL (force kill)
4. Wait to reap
```

```python
proc.terminate()  # SIGTERM
try:
    proc.wait(timeout=5)
except subprocess.TimeoutExpired:
    proc.kill()    # SIGKILL
    proc.wait()
```

```rust
child.kill()?;     // sends SIGKILL on Unix
child.wait()?;     // reaps
```

### 7.5 Timeout with Process Group Kill

When a subprocess spawns its own children, killing only the parent leaves orphans. The correct pattern uses process groups:

```python
proc = subprocess.Popen(cmd, start_new_session=True)
try:
    proc.wait(timeout=30)
except subprocess.TimeoutExpired:
    os.killpg(proc.pid, signal.SIGTERM)
    try:
        proc.wait(timeout=5)
    except subprocess.TimeoutExpired:
        os.killpg(proc.pid, signal.SIGKILL)
        proc.wait()
```

### 7.6 Linux pidfd

Linux 5.3+ provides `pidfd_open()` for race-free process management. A pidfd is a file descriptor referring to a process, avoiding PID reuse races (where a PID is recycled before the signal is delivered).

Rust exposes this via `CommandExt::create_pidfd(true)` on Linux.

Go's `os.Process` uses pidfd internally when available (Go 1.23+).

---

## 8. Design Patterns Summary for lx

### 8.1 What lx Already Does Well

- **Structured return value**: `$cmd` returns `Ok({out, err, code})`, matching the structured capture pattern used by Nushell's `complete` and Python's `CompletedProcess`.
- **Error propagation mode**: `$^cmd` is analogous to Python's `check=True` or Amber's mandatory `failed` handling -- the programmer opts into automatic error propagation.
- **Interpolation syntax**: `${}` blocks for embedding expressions in shell commands.

### 8.2 Patterns to Consider

From this research, several patterns stand out:

**Argument array safety**: lx currently passes the entire command string to `sh -c`. This is the same approach as Perl backticks, Ruby backticks, and Python `shell=True`. The safer approach (used by Go, Rust, and safe modes of Python/Ruby/Node.js) is argument arrays that bypass the shell. A hybrid approach (like execa/zx's tagged templates) could maintain lx's ergonomic `$cmd` syntax while providing injection safety.

**Timeout support**: Most languages provide timeout mechanisms. lx could add a timeout modifier: `$cmd timeout:30s` or similar.

**Process group management**: For agentic workflows where lx spawns subprocesses that may spawn their own children, process group kill is important for clean shutdown.

**Stderr separation**: lx already separates `out` and `err` in the return record, which is the most capable pattern.

**Environment control**: lx could expose environment manipulation for shell commands, following Rust's `.env()` / `.env_clear()` / `.env_remove()` model.

**Windows support**: If lx targets Windows, the `sh -c` invocation needs to be abstracted. The cross-platform pattern is to use the host language's process spawning API directly rather than assuming `/bin/sh`.
