# Linux Pipe Buffering Behavior

Empirically verified on Fedora 43, Linux 6.19.9, glibc 2.42. All claims below were tested
with instrumented programs, not sourced from documentation alone.

---

## 1. Default Buffering When stdout Is a Pipe

**glibc uses full buffering (\_IOFBF) when stdout is not a TTY.** The default buffer size is
**4096 bytes**.

The C standard requires that stdout be line-buffered only when it "can be determined to refer
to an interactive device." glibc implements this by checking `isatty(1)` during the first
write to stdout. When stdout is a pipe, `isatty()` returns 0, and glibc selects full buffering
with a 4096-byte buffer.

Verified by inspecting glibc internals after the first `fprintf(stdout, ...)`:

```
stdout->_flags = 0xfbad2884    (no _IO_LINE_BUF flag set)
buffer size = 4096
mode: fully-buffered
```

When stdout IS a TTY, glibc sets the `_IO_LINE_BUF` (0x200) flag and uses line buffering.

## 2. Line-Terminated Output Through Pipes

**No. Lines ending in `\n` are NOT flushed immediately when stdout is fully buffered.**

A C program that writes 5 short lines with `fprintf(stdout, "line %d\n", i)` followed by
`sleep(1)` between each -- all 5 lines arrive at the pipe reader simultaneously after the
process exits (when the stdio buffer is flushed at `exit()`). The `\n` character has no
special meaning to the full-buffering logic.

The data sits in glibc's 4096-byte userspace buffer until one of:

1. The buffer fills (4096 bytes accumulated)
2. `fflush(stdout)` is called explicitly
3. The process calls `exit()` (which flushes all stdio streams)
4. The stream is closed via `fclose(stdout)`

## 3. Kernel Pipe Buffer vs. PIPE_BUF vs. stdio Buffer

Three distinct concepts that are often confused:

| Concept | Value | What it is |
|---------|-------|------------|
| **stdio buffer** (glibc) | 4096 bytes | Userspace buffer in the process. Data accumulates here before any `write()` syscall. |
| **PIPE_BUF** | 4096 bytes | Kernel atomicity guarantee. Writes of PIPE_BUF bytes or fewer to a pipe are guaranteed atomic (won't interleave with other writers). |
| **Pipe capacity** | 65536 bytes (64 KiB) | Total kernel buffer for the pipe. `fcntl(fd, F_GETPIPE_SZ)`. Can be increased up to `pipe-max-size` (1 MiB default). |

The stdio buffer is entirely separate from the kernel pipe buffer. Data must first leave the
stdio buffer (via `fflush`/`write()`) before it enters the kernel pipe buffer.

## 4. Kernel read() Behavior

**`read()` on a pipe returns immediately when ANY data is available.** It does not wait for a
minimum amount or for the buffer to fill.

Verified: a parent process calling `read(fd, buf, 4096)` on a pipe receives each 8-byte
`write()` from the child individually, with 1-second gaps matching the child's `sleep(1)`.
The kernel pipe is not buffering or batching -- data written by `write()` is immediately
available to `read()` on the other end.

This means the bottleneck for streaming is always the **writer's stdio buffering**, never the
kernel pipe itself.

## 5. glibc Specifics

glibc (2.42, as shipped on Fedora 43) implements the ISO C / POSIX rules:

- `stderr`: always unbuffered (`_IONBF`)
- `stdout`: line-buffered (`_IOLBF`) if `isatty(1)` is true, otherwise fully-buffered (`_IOFBF`)
- `stdin`: line-buffered if `isatty(0)` is true, otherwise fully-buffered

The `isatty()` check happens lazily on first I/O operation, not at program startup.

musl libc follows the same POSIX rules but with different buffer sizes (1024 bytes default for
full buffering). The behavior is functionally identical.

## 6. Forcing Line Buffering on Pipes

### stdbuf -oL

```bash
stdbuf -oL ./my_program | reader
```

Works by setting `LD_PRELOAD=/usr/libexec/coreutils/libstdbuf.so` and environment variables
(`_STDBUF_O=L`). The preloaded library intercepts stdio initialization and calls
`setvbuf(stdout, NULL, _IOLBF, 0)` before `main()` runs.

**Limitations:**
- Only works on dynamically-linked programs that use glibc stdio
- Does NOT work on statically-linked binaries
- Does NOT work on programs that explicitly call `setvbuf()` themselves (overrides stdbuf)
- Does NOT work on programs that bypass stdio (e.g., raw `write()` syscalls, Node.js, Go)

### unbuffer (from expect package)

```bash
unbuffer ./my_program | reader
```

Creates a pseudo-TTY (pty) for the child's stdout. The child sees `isatty(1) == true` and
glibc automatically uses line buffering. This is the most reliable approach because it works
regardless of how the program does I/O -- the program genuinely believes it's writing to a
terminal.

**Limitations:**
- Adds pty overhead
- May cause programs to emit terminal escape codes (colors, cursor movement)
- Requires the `expect` package

### script

```bash
script -qc "./my_program" /dev/null | reader
```

Similar to `unbuffer` -- uses a pty. Available in `util-linux` (always installed on Fedora).

### Program-internal solutions

- C: `setlinebuf(stdout)` or `setvbuf(stdout, NULL, _IOLBF, 0)` or `fflush(stdout)` after each line
- Python: `python3 -u` or `PYTHONUNBUFFERED=1` or `sys.stdout.reconfigure(line_buffering=True)`
- Perl: `$| = 1`

## 7. Rust-Specific Behavior

### Rust's Stdout is ALWAYS Line-Buffered

**Rust's `std::io::Stdout` uses `LineWriter<StdoutRaw>` unconditionally**, regardless of
whether stdout is a TTY or a pipe. This is different from C.

From the Rust standard library source (`library/std/src/io/stdio.rs`, line 608-612):

```rust
pub struct Stdout {
    // FIXME: this should be LineWriter or BufWriter depending on the state of
    //        stdout (tty or not). Note that if this is not line buffered it
    //        should also flush-on-panic or some form of flush-on-abort.
    inner: &'static ReentrantLock<RefCell<LineWriter<StdoutRaw>>>,
}
```

The FIXME comment acknowledges this deviates from the C convention, but it has been this way
since Rust 1.0 and is unlikely to change (it would break existing programs that rely on
line-buffered pipe output).

**Consequence:** `println!()` and `write!(stdout(), "...\n")` both flush at every newline,
even through pipes. Verified empirically -- each line arrives at the pipe reader with the
expected 1-second delay.

### BufWriter Overrides This

Wrapping stdout in `BufWriter` switches to full buffering (8 KiB default):

```rust
let mut writer = BufWriter::new(std::io::stdout().lock());
writeln!(writer, "...")?;  // buffered, NOT flushed at newline
```

All 5 lines arrive simultaneously at process exit. Use this when throughput matters more than
latency (e.g., writing large files).

### tokio::process and Stdio::piped()

When spawning a child with `tokio::process::Command` and `Stdio::piped()`:

1. The child process gets a pipe fd for its stdout (fd 1)
2. The child's buffering depends on the child's runtime (C/glibc = full, Rust = line, Node = none)
3. On the parent side, `read()` on the pipe returns data as soon as any bytes are available in the kernel pipe buffer
4. `tokio::io::AsyncReadExt::read()` and `BufReader::read_line()` behave the same as synchronous `read()` -- they return as soon as the kernel has data

**The parent never adds buffering delay.** If you read line-by-line with `BufReader::read_line()`,
you get each line as soon as it enters the kernel pipe buffer. The only question is when the
child's stdio layer decides to `write()` the data to the kernel.

## 8. JSON-Lines Streaming Through Pipes

For the pattern `child --output-format stream-json | parent_reader`:

**Whether each JSON line arrives immediately depends entirely on the child's stdout buffering:**

| Child runtime | Default pipe behavior | Each line arrives immediately? |
|---------------|----------------------|-------------------------------|
| C (glibc) | Full buffering, 4 KiB | NO -- batched until buffer fills or process exits |
| C (glibc) + `fflush()` | Explicit flush | YES |
| C (glibc) + `stdbuf -oL` | Forced line buffering | YES |
| Rust (`println!`) | Line buffering (always) | YES |
| Rust (`BufWriter`) | Full buffering, 8 KiB | NO |
| Node.js | No stdio buffering (direct fd writes via libuv) | YES |
| Python | Full buffering, 8 KiB | NO |
| Python + `-u` | Unbuffered | YES |
| Go | No stdio buffering (direct syscalls) | YES |

### Claude CLI Specifically

The Claude CLI (`~/.local/share/claude/versions/2.1.84`) is a **statically-compiled Node.js
binary** (ELF with Node API symbols, linked against glibc). Node.js does not use C stdio for
`process.stdout` -- it uses libuv which writes directly to the fd. Therefore:

**`claude --print --output-format stream-json` delivers each JSON line through a pipe
immediately as it's generated**, without any buffering delay. No `stdbuf` or `unbuffer` is
needed.

### Reading JSON-Lines in Rust

The recommended pattern for a Rust parent reading streaming JSON lines from a piped child:

```rust
let mut child = tokio::process::Command::new("child")
    .stdout(Stdio::piped())
    .spawn()?;

let stdout = child.stdout.take().unwrap();
let reader = tokio::io::BufReader::new(stdout);
let mut lines = reader.lines();

while let Some(line) = lines.next_line().await? {
    let event: serde_json::Value = serde_json::from_str(&line)?;
    // Process event immediately -- no buffering delay
}
```

`BufReader` on the read side does NOT add latency. It buffers reads for efficiency (fewer
`read()` syscalls) but returns a line as soon as one is available. The key insight: read-side
`BufReader` only buffers what's already in the kernel pipe buffer. It doesn't introduce
artificial delays.

---

## Summary of Key Facts

1. **C/glibc pipes are fully buffered at 4096 bytes** -- `\n` does NOT trigger a flush
2. **Rust stdout is line-buffered always** -- even on pipes, unlike C
3. **Node.js bypasses stdio entirely** -- writes go directly to the fd via libuv
4. **Kernel `read()` returns immediately** when any data is available -- the kernel pipe never adds delay
5. **The 65536-byte pipe capacity** is unrelated to buffering behavior -- it's just the total buffer before `write()` blocks
6. **`stdbuf -oL` only works on dynamically-linked glibc programs** -- useless for Go, Rust, Node.js, or static binaries
7. **For JSON-lines streaming, the writer must flush after each line** -- the reader side is never the bottleneck
