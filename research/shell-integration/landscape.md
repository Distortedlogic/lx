# Shell Execution Across Programming Languages

Research survey covering shell/subprocess integration in general-purpose languages and shell scripting languages. Context: lx has first-class shell integration with `$cmd` (execute, returns record with `out`/`err`/`code`), `$^cmd` (execute with error propagation, returns stdout string on success), and `${}` (interpolated block).

---

## 1. Perl

### 1.1 Execution Primitives

Perl provides four core mechanisms for shell execution, each with distinct behavior:

**Backticks / qx//**: Capture stdout as a string. The command is passed to `/bin/sh -c`. In list context, returns lines; in scalar context, returns a single string.

```perl
my $output = `ls -la`;           # backtick syntax
my $output = qx(ls -la);         # qx// with parens
my @lines  = qx{cat /etc/hosts}; # qx// with braces, list context
```

**system()**: Executes a command and returns the exit status (shifted -- `$? >> 8` gives the actual exit code). Two forms exist:

```perl
system("ls -la");                # string form: invokes /bin/sh -c
system("ls", "-la", "/tmp");     # list form: bypasses shell entirely
```

The list form is the secure variant -- it calls `execvp()` directly, preventing shell metacharacter injection.

**exec()**: Replaces the current process with the given command. Never returns on success. Same two forms as `system()`.

```perl
exec("ls", "-la") or die "exec failed: $!";
```

**open() with pipes**: Opens a filehandle connected to a subprocess's stdin or stdout.

```perl
open(my $fh, "-|", "ls", "-la") or die $!;  # read from command
open(my $fh, "|-", "sort")      or die $!;  # write to command
```

The three-argument form with explicit mode (`-|` or `|-`) and argument list bypasses the shell.

### 1.2 Exit Status: $?

After backticks, `system()`, or `close()` on a pipe, `$?` contains the child's wait status:

- `$? >> 8`: exit code
- `$? & 127`: signal number that killed the process
- `$? & 128`: whether a core dump was produced

### 1.3 Taint Mode

Perl's `-T` flag enables taint mode, a compile-time + runtime security mechanism. All data from external sources (command line args, environment variables, file input, `readdir()`, `readlink()`, locale info) is marked "tainted." Tainted values cannot be used in:

- `system()`, `exec()`, backticks, `qx//`
- `open()` with pipes
- Any command that modifies files, directories, or processes

The only way to untaint is via regex capture groups:

```perl
if ($input =~ /^([-\w.]+)$/) {
    $clean = $1;  # untainted
} else {
    die "Bad input: $input";
}
```

Taint mode activates automatically when setuid/setgid bits are detected. When active, `PERL5LIB` and `PERLLIB` are ignored.

Source: [perlsec](https://perldoc.perl.org/perlsec)

### 1.4 Modern Subprocess Management

**IPC::Open3**: Core module. Provides filehandles for stdin, stdout, and stderr of a subprocess. Requires manual `select()` loops and `waitpid()` to avoid zombie processes.

```perl
use IPC::Open3;
my $pid = open3(\*CHLD_IN, \*CHLD_OUT, \*CHLD_ERR, "command", "arg1");
waitpid($pid, 0);
```

**IPC::Run**: CPAN module. Wraps `select()` and `waitpid()`. Supports piping between subprocesses, timeouts, pty allocation (Unix), and killing running processes.

**IPC::Run3**: Modern replacement for `system()`, `qx//`, and `open3()`. Automatically handles buffering, prevents deadlocks, provides a simple Perlish interface. Recommended default choice.

Source: [IPC::Run on CPAN](https://metacpan.org/pod/IPC::Run), [IPC::Open3](https://perldoc.perl.org/IPC::Open3)

---

## 2. Ruby

### 2.1 Nine Ways to Execute Shell Commands

Ruby provides a remarkably large number of shell execution methods:

**Backticks**: Capture stdout as a string. Invokes `/bin/sh -c`.

```ruby
output = `ls -la`
```

**%x{}**: Equivalent to backticks with alternate delimiters.

```ruby
output = %x{ls -la}
output = %x[echo "hello"]
```

**system()**: Returns `true` on zero exit, `false` on nonzero, `nil` if command not found. Two forms:

```ruby
system("ls -la")            # string: uses shell
system("ls", "-la", "/tmp") # array: bypasses shell
```

**exec()**: Replaces current process. Never returns on success.

**IO.popen**: Returns an IO object connected to the subprocess's stdin/stdout.

```ruby
IO.popen("ls -la") { |io| puts io.read }
IO.popen(["ls", "-la"]) { |io| puts io.read }  # array form, no shell
```

**Open3**: The `open3` module provides simultaneous access to stdin, stdout, and stderr.

```ruby
require 'open3'
stdout, stderr, status = Open3.capture3("ls", "-la")
# status is a Process::Status object
```

`Open3.popen3` gives streaming access; `Open3.capture3` is the convenience wrapper that collects everything.

**spawn()**: Low-level process creation. Returns PID immediately. Requires `Process.wait` to reap.

```ruby
pid = spawn("long_running_command")
Process.wait(pid)
```

**Process.spawn**: Same as `spawn()` with additional options for redirecting file descriptors, setting environment variables, and process group management.

### 2.2 Exit Status: $?

After backticks, `system()`, or `%x{}`, Ruby sets `$?` to a `Process::Status` object with methods: `.exitstatus`, `.success?`, `.signaled?`, `.termsig`, `.pid`.

### 2.3 Shellwords and Escaping

The `Shellwords` module provides escaping for safe shell string construction:

```ruby
require 'shellwords'
safe = Shellwords.escape("file with spaces.txt")
# => "file\\ with\\ spaces.txt"
```

Limitation: `Shellwords.escape` does not protect against argument injection (leading `-` characters). The array form of `system()`/`spawn()` is always safer than escaping.

Source: [Shellwords](https://ruby-doc.org/stdlib-2.5.1/libdoc/shellwords/rdoc/Shellwords.html), [Open3 guide](https://readysteadycode.com/howto-execute-shell-commands-with-ruby-open3), [Ruby subprocess patterns](https://mattbrictson.com/blog/run-shell-commands-in-ruby)

---

## 3. Python

### 3.1 Evolution of Subprocess APIs

Python's subprocess story is a progression from dangerous to safe:

**os.system()** (deprecated): Passes a string to `/bin/sh -c`. Returns the exit status. No output capture. Vulnerable to injection.

**os.popen()** (deprecated): Returns a file-like object for reading stdout. Also vulnerable.

**subprocess.run()** (Python 3.5+): The recommended high-level API.

```python
import subprocess

result = subprocess.run(["ls", "-la"], capture_output=True, text=True)
print(result.stdout)
print(result.returncode)
```

Key parameters:
- `capture_output=True`: captures stdout and stderr
- `text=True` (or `encoding="utf-8"`): decode bytes to strings
- `check=True`: raises `CalledProcessError` on nonzero exit
- `timeout=30`: kills and raises `TimeoutExpired` after N seconds
- `shell=True`: passes command string to `/bin/sh -c` (dangerous)
- `env={"KEY": "val"}`: replaces environment entirely
- `cwd="/path"`: sets working directory
- `input="data"`: sends data to stdin

**subprocess.Popen**: Low-level API for streaming and interactive subprocesses.

```python
proc = subprocess.Popen(["sort"], stdin=subprocess.PIPE,
                        stdout=subprocess.PIPE, text=True)
out, err = proc.communicate(input="banana\napple\ncherry\n")
```

### 3.2 shell=True: Why It's Dangerous

When `shell=True`, the command is passed as a string to `/bin/sh -c`. This enables:
- Shell metacharacters: `;`, `|`, `&&`, `||`, `$()`, backticks
- Environment variable expansion
- Glob expansion
- Redirections

An attacker controlling any part of the string can inject arbitrary commands. The list syntax with `shell=False` (default) passes arguments directly to `execvp()`, making injection impossible.

### 3.3 shlex.quote()

`shlex.quote(s)` wraps a string in single quotes with proper escaping for POSIX shells. It is a fallback -- the list syntax is always preferred.

```python
import shlex
cmd = f"echo {shlex.quote(user_input)}"
subprocess.run(cmd, shell=True)  # still not ideal
```

### 3.4 CalledProcessError

When `check=True`, nonzero exit raises `CalledProcessError` with attributes: `.returncode`, `.cmd`, `.stdout`, `.stderr`, `.output`.

Source: [subprocess docs](https://docs.python.org/3/library/subprocess.html), [Semgrep Python command injection](https://semgrep.dev/docs/cheat-sheets/python-command-injection)

---

## 4. JavaScript / Node.js

### 4.1 child_process Module

Node.js provides four functions in `child_process`:

**exec()**: Runs command in a shell (`/bin/sh -c`). Buffers output. Returns via callback.

```javascript
const { exec } = require('child_process');
exec('ls -la', (error, stdout, stderr) => {
    if (error) { console.error(`error: ${error.message}`); return; }
    console.log(stdout);
});
```

**execFile()**: Like `exec()` but does NOT use a shell by default. Arguments passed as array.

```javascript
execFile('ls', ['-la'], (error, stdout, stderr) => { ... });
```

**spawn()**: Streams-based. Does NOT use a shell by default. Returns a `ChildProcess` object with `.stdin`, `.stdout`, `.stderr` streams.

```javascript
const child = spawn('ls', ['-la']);
child.stdout.on('data', (data) => console.log(data.toString()));
child.on('close', (code) => console.log(`exited with ${code}`));
```

**fork()**: Specialized `spawn()` for Node.js modules. Establishes an IPC channel for `process.send()` / `process.on('message')` communication.

| Feature | exec() | execFile() | spawn() | fork() |
|---|---|---|---|---|
| Shell by default | yes | no | no | no |
| Output delivery | callback | callback | streams | IPC |
| Buffer limit | yes | yes | no | no |

### 4.2 stdio Configuration

The `stdio` option controls file descriptor mapping:

- `'pipe'`: creates accessible streams (`.stdin`, `.stdout`, `.stderr`)
- `'inherit'`: connects directly to parent process streams
- `'ignore'`: discards I/O

### 4.3 Execa

Execa wraps `child_process` with a modern, promise-based API. Its template literal syntax (`$`) automatically escapes interpolated values:

```javascript
import { $ } from 'execa';

const name = 'foo bar';
await $`mkdir /tmp/${name}`;  // space handled safely, no injection

const { stdout } = await $`cat package.json`.pipe`grep name`;
```

Key features:
- Template literal syntax with automatic argument escaping
- Pipeline support with intermediate result access (`pipedFrom`)
- Generator-based stream transforms
- Detailed error messages with exit code, stdout, stderr
- IPC between parent and child via `sendMessage()` / `getOneMessage()`
- Cross-platform: Windows shebang handling, graceful signal emulation

Source: [Execa on GitHub](https://github.com/sindresorhus/execa)

### 4.4 Google's zx

zx provides a shell scripting experience in JavaScript using tagged template literals:

```javascript
import { $ } from 'zx';

await $`cat package.json | grep name`;

const branch = await $`git branch --show-current`;
await $`dep deploy --branch=${branch}`;

// Parallel execution
await Promise.all([$`sleep 1; echo 1`, $`sleep 2; echo 2`]);
```

zx's `$` returns a `ProcessPromise` that resolves to a `ProcessOutput` with `.stdout`, `.stderr`, `.exitCode`, `.signal`. Arguments are auto-escaped. Built-in helpers: `cd()`, `glob()`, `fetch()`, `question()`, `retry()`, `spinner()`.

Unlike execa (which avoids the shell), zx uses `/bin/sh` by default, providing shell features like pipes and redirections while escaping interpolated values.

Source: [zx on GitHub](https://github.com/google/zx)

---

## 5. Go

### 5.1 os/exec Package

Go's `os/exec` package intentionally does NOT invoke the system shell. No glob expansion, no environment variable expansion, no pipes, no redirections. Arguments are passed directly to `execve()`.

```go
cmd := exec.Command("ls", "-la", "/tmp")
output, err := cmd.Output()
```

**Command creation**:

```go
func Command(name string, arg ...string) *Cmd
func CommandContext(ctx context.Context, name string, arg ...string) *Cmd
```

`CommandContext` adds timeout/cancellation via Go's context system.

**Execution methods**:

| Method | Waits? | Returns |
|---|---|---|
| `Run()` | yes | `error` |
| `Output()` | yes | `([]byte, error)` -- captures stdout |
| `CombinedOutput()` | yes | `([]byte, error)` -- captures stdout+stderr |
| `Start()` | no | `error` -- must call `Wait()` |

**I/O piping**:

```go
cmd := exec.Command("cat")
stdin, _ := cmd.StdinPipe()   // io.WriteCloser
stdout, _ := cmd.StdoutPipe() // io.ReadCloser
cmd.Start()
io.WriteString(stdin, "data")
stdin.Close()
result, _ := io.ReadAll(stdout)
cmd.Wait()
```

`StdinPipe`/`StdoutPipe`/`StderrPipe` must NOT be used with `Run()` or `Output()` -- use `Start()` + `Wait()`.

### 5.2 Environment and Directory

```go
cmd.Env = append(os.Environ(), "FOO=bar")  // inherit + add
cmd.Dir = "/some/path"                      // working directory
cmd.Environ()                               // read computed env
```

If `Env` is nil, the child inherits the parent's environment. Duplicate keys: last value wins.

### 5.3 LookPath and ErrDot

`LookPath(file)` searches PATH for an executable. Go 1.19+ returns `ErrDot` if the resolved path is relative to the current directory -- a security measure preventing accidental execution of local files that shadow system commands.

### 5.4 Error Types

- `*exec.Error`: returned by `LookPath` when the command is not found
- `*exec.ExitError`: returned when the command exits with nonzero status; wraps `os.ProcessState` and includes captured stderr

Source: [os/exec package](https://pkg.go.dev/os/exec)

---

## 6. Rust

### 6.1 std::process::Command

Rust's `Command` uses a builder pattern. Arguments are passed directly to `execve()` -- no shell expansion, no glob patterns, no variable substitution.

```rust
use std::process::Command;

let output = Command::new("ls")
    .arg("-la")
    .arg("/tmp")
    .output()?;
println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
```

**Execution methods**:

| Method | Returns | Defaults |
|---|---|---|
| `output()` | `io::Result<Output>` | stdout/stderr piped, stdin null |
| `status()` | `io::Result<ExitStatus>` | all inherited |
| `spawn()` | `io::Result<Child>` | all inherited |

`Output` contains `.status: ExitStatus`, `.stdout: Vec<u8>`, `.stderr: Vec<u8>`.

### 6.2 I/O Configuration

```rust
use std::process::Stdio;

let child = Command::new("sort")
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::null())
    .spawn()?;

let stdin = child.stdin.take().unwrap();
// write to stdin
let output = child.wait_with_output()?;
```

Options: `Stdio::inherit()`, `Stdio::piped()`, `Stdio::null()`.

### 6.3 Environment Control

```rust
Command::new("prog")
    .env("KEY", "value")         // set one
    .env_remove("SECRET")        // remove one
    .env_clear()                 // remove all inherited
    .envs(filtered_env)          // set multiple
    .current_dir("/path")
    .spawn()?;
```

### 6.4 Platform Extensions

**Unix** (`std::os::unix::process::CommandExt`): `.uid()`, `.gid()`, `.pre_exec()` (runs closure in child after fork, before exec).

**Windows** (`std::os::windows::process::CommandExt`): `.creation_flags()`, `.raw_arg()` (bypass standard argument encoding).

**Linux** (`std::os::linux::process::CommandExt`): `.create_pidfd()` for pidfd-based process management.

### 6.5 Windows Security Caveat

On Windows, `Command` uses `CreateProcessW`. When given a `.bat` file, it auto-converts to `cmd.exe /c`, which uses non-standard argument parsing. Untrusted input passed to `cmd.exe` or `.bat` files can result in arbitrary shell command execution.

### 6.6 Reusability

A `Command` can be reused to spawn multiple processes (unlike Go's `Cmd`):

```rust
let mut cmd = Command::new("echo");
cmd.arg("hello");
let out1 = cmd.output()?;
let out2 = cmd.output()?;  // valid
```

Source: [std::process::Command](https://doc.rust-lang.org/std/process/struct.Command.html), [Rust command injection prevention](https://www.stackhawk.com/blog/rust-command-injection-examples-and-prevention/)

---

## 7. Lua

### 7.1 Limited Subprocess Support

Lua provides only two functions for shell execution, both minimal:

**os.execute(command)**: Passes a string to the system shell. Returns three values: `true`/`nil` (success/failure), `"exit"`/`"signal"` (how it ended), and the exit code or signal number.

```lua
local ok, how, code = os.execute("ls -la")
if not ok then print("failed with " .. how .. " " .. code) end
```

**io.popen(command, mode)**: Opens a pipe to a shell command. Returns a file handle for reading stdout (`"r"`) or writing stdin (`"w"`). Cannot capture stderr.

```lua
local f = io.popen("ls -la", "r")
local output = f:read("*a")
f:close()
```

### 7.2 Limitations

- `io.popen` only captures stdout; no access to stderr without shell redirects (`2>&1`)
- No simultaneous read/write (no bidirectional pipes)
- No process control (signals, kill, timeout)
- No argument array form -- everything goes through the shell
- `io.popen` may not be available on all platforms (not supported on some embedded Lua builds)
- GUI applications using `os.execute` on Windows get a console window flash

### 7.3 External Libraries

For more advanced subprocess management, Lua relies on external libraries:

- **luaposix**: Provides `posix.unistd.exec()`, `posix.unistd.fork()`, `posix.sys.wait.wait()`
- **lua-subprocess**: Cross-platform subprocess with pipes for stdin/stdout/stderr
- **luasocket**: For network-based IPC

The minimal standard library is intentional -- Lua targets embedding, and advanced process management depends on the host application.

Source: [Lua 5.4 Reference Manual](https://www.lua.org/manual/5.4/)

---

## 8. Nushell

### 8.1 Structured Data Pipelines

Nushell's core innovation is that pipelines carry structured data (tables, records, lists) rather than text streams. Internal commands produce and consume typed values:

```nu
ls | where size > 1kb | sort-by modified | first 5
```

`ls` returns a table with columns: `name`, `type`, `size`, `modified`. `where` filters rows. `sort-by` sorts by column. No parsing required.

### 8.2 Type System

Nushell has first-class types: `int`, `float`, `string`, `bool`, `date`, `duration`, `filesize`, `list`, `record`, `table`, `binary`, `nothing`, `closure`, `error`.

### 8.3 External Commands

External commands produce "raw streams" (byte sequences). Nushell auto-converts to UTF-8 text where possible:

```nu
^git log --oneline | lines | first 5
```

The `^` prefix explicitly runs an external command. Without it, Nushell checks internal commands first.

### 8.4 Exit Code and Error Handling

Two mechanisms for accessing exit information:

**$env.LAST_EXIT_CODE**: Set after each external command completes.

**complete command**: Returns a record with `stdout`, `stderr`, and `exit_code`:

```nu
let result = do { ^git status } | complete
$result.exit_code    # 0
$result.stdout       # "On branch main\n..."
```

### 8.5 Stderr Handling

By default, stderr passes through to the terminal. To capture or redirect:

```nu
^cmd e>| lines              # pipe stderr to next command
^cmd e> error.log           # redirect stderr to file
do -i { ^cmd } | complete   # capture stderr in record
```

Source: [Nushell book: stdout, stderr, exit codes](https://www.nushell.sh/book/stdout_stderr_exit_codes.html), [Nushell types](https://www.nushell.sh/book/types_of_data.html)

---

## 9. PowerShell

### 9.1 Object-Based Pipelines

PowerShell pipelines pass .NET objects, not text. Every cmdlet outputs objects with typed properties and methods:

```powershell
Get-ChildItem | Where-Object { $_.Length -gt 1KB } | Sort-Object LastWriteTime
```

`Get-ChildItem` returns `FileInfo` / `DirectoryInfo` objects. Properties (`.Length`, `.LastWriteTime`) are accessed directly -- no text parsing.

### 9.2 Cmdlets

Cmdlets are specialized .NET classes following the Verb-Noun naming convention (`Get-Process`, `Set-Content`, `Invoke-WebRequest`). They implement `BeginProcessing()`, `ProcessRecord()`, `EndProcessing()` methods.

### 9.3 External Commands

PowerShell runs external commands but treats their output as text. The `$LASTEXITCODE` variable holds the exit code:

```powershell
git status
if ($LASTEXITCODE -ne 0) { Write-Error "git failed" }
```

### 9.4 Comparison with Unix Shells

| Feature | Unix shells | PowerShell |
|---|---|---|
| Pipeline data | text streams | .NET objects |
| Filtering | `grep`, `awk`, `sed` | `Where-Object`, properties |
| Parsing needed | always | rarely |
| Cross-command types | strings | structured objects |
| Command naming | arbitrary | Verb-Noun convention |

### 9.5 Invoke-Expression

`Invoke-Expression` is PowerShell's equivalent of `eval` -- it parses and executes a string as PowerShell code. It has the same injection risks as `shell=True` in other languages.

Source: [PowerShell pipelines](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_pipelines), [PowerShell vs Bash](https://www.techtarget.com/searchitoperations/tip/On-Windows-PowerShell-vs-Bash-comparison-gets-interesting)

---

## 10. Fish

### 10.1 Command Substitution

Fish uses `$(command)` or bare `(command)` -- backticks are not supported:

```fish
set files (ls)
echo "Current dir: $(pwd)"
```

Output is split on newlines only -- no `$IFS`-based word splitting. This prevents an entire class of bash bugs.

### 10.2 No Word Splitting

Fish does not perform word splitting on variable expansion. A variable expands to all its elements, with each element as its own argument. This means double-quoting variables is unnecessary for safety:

```fish
set name "foo bar"
mkdir $name   # creates ONE directory named "foo bar", not two
```

### 10.3 Quoting

- Single quotes: no expansions whatsoever
- Double quotes: only `$variable` expansion and `$(command)` substitution
- No `$'...'` syntax; escape sequences work only when unquoted

### 10.4 Process Substitution

Fish replaces bash's `<(command)` with `(command | psub)`:

```fish
diff (sort file1 | psub) (sort file2 | psub)
```

### 10.5 Safety Improvements Over Bash

- Globs that match nothing produce an error (like bash's `failglob`)
- No implicit subshells -- variables set in pipelines persist
- No word splitting eliminates the #1 source of bash bugs
- Built-in `string` command replaces `sed`/`awk` for string manipulation

### 10.6 Special Variables

`$status` (not `$?`), `$fish_pid` (not `$$`), `$argv` (not `$@`), `$last_pid` (not `$!`).

Source: [Fish for bash users](https://fishshell.com/docs/current/fish_for_bash_users.html), [Fish language docs](https://fishshell.com/docs/current/language.html)

---

## 11. Bash / POSIX sh

### 11.1 Command Substitution

Two forms: `$(command)` (preferred, nestable) and `` `command` `` (legacy, cannot nest):

```bash
files=$(ls -la)
count=$(wc -l < $(find . -name "*.txt" | head -1))  # nested
```

### 11.2 Process Substitution (Bash-only, not POSIX)

`<(command)` creates a named pipe (FIFO) or `/dev/fd/*` entry:

```bash
diff <(sort file1) <(sort file2)
paste <(cut -f1 data.tsv) <(cut -f2 data.tsv)
```

`>(command)` provides write-side process substitution.

### 11.3 Here Documents

```bash
cat <<EOF
Hello $name
Today is $(date)
EOF

cat <<'EOF'
No $expansion here
EOF
```

Quoted delimiter (`'EOF'`) prevents all expansion.

### 11.4 Error Handling with set Options

**set -e** (errexit): Exit immediately on nonzero exit status. Exceptions: commands in `if`, `while`, `until` conditions, and commands followed by `&&` or `||`.

**set -o pipefail**: Exit code of a pipeline is the rightmost nonzero, not just the last command.

**set -u** (nounset): Treat unset variables as errors.

**set -o errtrace**: ERR trap inherited by functions and subshells.

Common idiom:

```bash
set -euo pipefail
```

Caveat: `set -e` does NOT apply inside command substitution `$(...)` by default. In Bash 4.4+, `shopt -s inherit_errexit` fixes this.

### 11.5 Exit Codes

- `$?`: exit status of last command
- `${PIPESTATUS[@]}`: array of exit statuses from last pipeline (Bash-only)

Source: [Greg's Wiki on pipefail](https://mywiki.wooledge.org/BashFAQ/105), [Safer bash scripts](https://coderwall.com/p/fkfaqq/safer-bash-scripts-with-set-euxo-pipefail)

---

## 12. Zsh

### 12.1 Extended Globbing

With `setopt EXTENDED_GLOB`:

```zsh
ls ^*.txt          # anything NOT matching *.txt
ls *.txt~bad.txt   # *.txt except bad.txt
ls (#i)*.TXT       # case-insensitive glob
ls **/*.rs(.)      # glob qualifiers: (.) means regular files only
ls *(om[1,5])      # 5 most recently modified files
```

Glob qualifiers (the `(...)` after the pattern) are unique to Zsh and enable filtering by file type, permissions, size, and modification time.

### 12.2 Differences from Bash

| Feature | Bash | Zsh |
|---|---|---|
| Array indexing | 0-based | 1-based |
| Word splitting on `$var` | yes (unquoted) | no (by default) |
| Glob no-match | silent (returns pattern) | error |
| Associative arrays | `declare -A` | `typeset -A` (richer API) |
| Extended globs | `shopt -s extglob` | `setopt EXTENDED_GLOB` |
| Prompt expansion | `PS1` | `PROMPT` with `%`-escapes |

Zsh does NOT perform word splitting on unquoted parameter expansions by default (unlike Bash). This is the single biggest compatibility difference.

### 12.3 Associative Arrays

```zsh
typeset -A map
map[key1]=value1
map[key2]=value2
echo ${(k)map}     # print keys
echo ${(v)map}     # print values
```

Zsh allows unsetting individual elements of associative arrays (Bash does too, but Zsh's implementation is more consistent).

---

## 13. Oil / Oils (YSH)

### 13.1 Fixing Bash's Quoting Problems

YSH (the expression language of Oils) separates the command language from the expression language. In YSH, `$` interpolation happens at the language level, not the shell level, eliminating the need for defensive quoting:

```ysh
var name = 'foo bar'
echo $name           # passes "foo bar" as ONE argument
```

No word splitting. No glob expansion on variable values. These are opt-in, not opt-out.

### 13.2 Expressions vs Commands

YSH has three interleaved sub-languages:

- **Command language**: lines that start with a command word
- **Expression language**: inside `$[...]` or after `=`
- **Word language**: how arguments are constructed

```ysh
echo "flag=$[1 + a[i] * 3]"    # expression inside string
var x = len(mylist)             # expression after =
```

### 13.3 String Types

- Double-quoted: `"hello $name"` with `$` interpolation
- Single-quoted: `'literal'` with no interpolation
- J8 strings: JSON-compatible with C-style escapes (`\n`, `\t`)
- Multi-line strings with `'''` and `"""`

### 13.4 Error Handling

YSH provides `try` / `_error` for command error handling, replacing the fragile `set -e`:

```ysh
try {
    ls /nonexistent
}
if (_error.code !== 0) {
    echo "ls failed with $[_error.code]"
}
```

Source: [YSH vs Shell Idioms](https://www.oilshell.org/release/latest/doc/idioms.html), [YSH Language Influences](https://oils.pub/release/latest/doc/language-influences.html)

---

## 14. Xonsh

### 14.1 Python + Shell Hybrid

Xonsh is a superset of Python 3 with shell features. Mode switching is implicit:

```xonsh
# Python mode (recognized Python syntax)
x = 1 + 2
for f in ['a', 'b']:
    print(f)

# Subprocess mode (line starts with unknown name)
ls -la
git status
```

A line enters subprocess mode when it begins with a name that doesn't exist in the current Python scope.

### 14.2 Captured vs Uncaptured Subprocesses

Four subprocess operators:

| Operator | Captures stdout? | Returns |
|---|---|---|
| `$()` | yes | string (stdout) |
| `!()` | yes | `CommandPipeline` object |
| `$[]` | no (passes through) | `None` |
| `![]` | no (passes through) | `CommandPipeline` object |

`CommandPipeline` contains: `.returncode`, `.pid`, `.out`, `.err`, `.rtn`.

### 14.3 Environment Variables

Xonsh uses `$` prefix for environment variables, shared between Python and subprocess mode:

```xonsh
$PATH.append('/custom/bin')
$MY_VAR = 'hello'
echo $MY_VAR           # works in subprocess mode
print($MY_VAR)         # works in Python mode
```

### 14.4 Python Evaluation in Commands

```xonsh
echo @(2 + 3)          # evaluates Python expression, passes result as argument
ls @(glob('*.py'))     # Python glob result used as arguments
```

Source: [Xonsh tutorial](https://xon.sh/tutorial.html), [Xonsh on GitHub](https://github.com/xonsh/xonsh)

---

## 15. Amber

### 15.1 Type-Safe Bash Transpiler

Amber compiles to Bash, adding type safety and structured error handling. Commands use `$...$` delimiters:

```amber
$ mv file.txt dest.txt $
let result = $ cat file.txt | grep "READY" $
```

### 15.2 Error Handling

Amber enforces error handling at compile time -- unhandled failures prevent compilation.

**failed block** (recommended):

```amber
$ cat file.txt $ failed(code) {
    echo "Exited with code {code}."
}
```

**succeeded block**: runs only on success.

**? operator**: propagates failure to the calling function.

**status keyword**: holds exit code of the previous command.

**trust modifier**: disables failure handling (opt-in to Bash-like behavior).

### 15.3 Command Modifiers

- `silent`: suppresses all output (stdout and stderr)
- `trust`: disables failure handling
- `sudo`: runtime-detected privilege escalation

### 15.4 Interpolation

Variables and expressions interpolate with `{...}`:

```amber
let path = "/tmp/data"
$ cat {path} $ failed {
    echo "Could not open '{path}'"
}
```

### 15.5 Compilation Target

Everything compiles to Bash. The output is a valid `.sh` script. This means Amber scripts run anywhere Bash runs, but gain compile-time type checking and enforced error handling.

Source: [Amber docs: commands](https://docs.amber-lang.com/basic_syntax/commands), [Amber on GitHub](https://github.com/amber-lang/amber)
