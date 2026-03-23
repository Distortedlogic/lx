# Filesystem Sandboxing and Isolation for AI Agent Workspaces

## OverlayFS for Disposable Agent Workspaces

OverlayFS layers a writable upper directory over a read-only lower directory:

```
Merged View (what agent sees)
    ├── Upper Layer (writable, agent changes go here)
    └── Lower Layer (read-only, base workspace)
```

- Reads: fall through to lower if not in upper
- Writes: copy-on-write to upper
- Deletes: whiteout file in upper
- Reset: unmount, delete upper, remount

```bash
mount -t overlay overlay \
  -o lowerdir=/base/workspace,upperdir=/tmp/agent-1/upper,workdir=/tmp/agent-1/work \
  /mnt/agent-1-workspace
```

Multiple agents share one base with separate uppers. OpenHands supports this via `SANDBOX_VOLUME_OVERLAYS` config.

Limitation: requires `CAP_SYS_ADMIN` or user namespace with mount capabilities.

---

## Read-Only Bind Mounts, tmpfs, and CoW

### Read-Only Bind Mounts

Bubblewrap approach: system dirs (`/usr`, `/lib`, `/bin`) bind-mounted read-only, only project working dir gets read-write.

### tmpfs

In-memory filesystem, auto-cleaned on unmount:
```bash
mount -t tmpfs -o size=512M,noexec,nosuid tmpfs /sandbox/tmp
```

Key use: mount tmpfs over `~/.ssh` to make SSH keys invisible to the agent.

### Copy-on-Write (Block/File Level)

**Btrfs**: `cp --reflink=always` creates instant CoW copies. Snapshots are metadata-only operations.

**ZFS**: `zfs snapshot` + `zfs clone` for instant writable copies. Built-in checksumming.

---

## How Coding Agents Isolate File Access

### Claude Code

- Linux: Bubblewrap via `sandbox-runtime` with bind mounts (ro/rw), seccomp BPF, network namespace isolation
- macOS: Seatbelt (sandbox-exec) with dynamically generated profiles
- Bash tool restricted to read/write within CWD and subdirectories
- `enableWeakerNestedSandbox` mode for running inside Docker without privileged namespaces

### OpenHands

- Per-session Docker container, torn down after session
- Workspace bind-mounted (optional overlay CoW mode)
- Agent connects via SSH/REST API
- Pluggable backends: local Docker, remote Docker, K8s pods, Daytona

### Devin

- Cloud-hosted "Devbox" -- isolated VM with shell, editor, Chrome
- Bidirectional file sync with host
- Per-Devbox scoped credentials

### SWE-agent

- Docker container per task
- Repository cloned inside container
- Custom shell interface (open, edit, create, find_file, search_dir)

### Docker Sandboxes (v4.60+)

- Dedicated microVM with private Docker daemon per agent
- Workspace bind-mounted at same absolute path
- HTTP/HTTPS filtering proxy for egress

---

## Snapshot and Rollback Mechanisms

### Btrfs Snapshots

```bash
btrfs subvolume snapshot /workspaces/base /workspaces/agent-1
# Agent works in /workspaces/agent-1
# Rollback:
btrfs subvolume delete /workspaces/agent-1
btrfs subvolume snapshot /workspaces/base /workspaces/agent-1
```

Instant, space-efficient (only modified blocks use space), sub-second for any size.

### ZFS

```bash
zfs snapshot pool/workspace@clean
zfs clone pool/workspace@clean pool/workspace/agent-1
# Rollback:
zfs rollback pool/workspace@checkpoint
```

### AgentFS (Turso) -- SQLite-Backed Overlay

FUSE overlay where writes go to SQLite database (delta layer):
- Every change is structured, queryable, supports time-travel
- Can fork sessions, diff changes, replay agent actions
- On macOS uses NFS instead of FUSE

https://github.com/tursodatabase/agentfs

### Git-Based Snapshots

Most common lightweight approach:
- `git stash` before risky operations
- `git commit` at checkpoints
- `git diff` to review changes
- `git reset --hard` to rollback

---

## Git Worktrees as Lightweight Isolation

```bash
git worktree add /tmp/agent-workspace feature-branch
# Agent works in /tmp/agent-workspace on feature-branch
# Changes don't affect main workspace
git worktree remove /tmp/agent-workspace
```

Advantages:
- Instant creation (shared .git objects)
- Branch isolation (each worktree on own branch)
- Safe merging (review before merge)
- Near-zero overhead

**Container Use by Dagger**: MCP server creating Docker container + Git worktree per agent session. Container provides runtime isolation, worktree provides code isolation.

**ccswarm**: Orchestrates multiple Claude Code agents in worktree-isolated environments.

Limitation: worktrees isolate code but not runtime (shared host ports, databases, services). Not a security boundary.

---

## Landlock LSM for Filesystem Access Control

Unprivileged, near-zero overhead, self-sandboxing (Linux 5.13+).

Three syscalls:
1. `landlock_create_ruleset()` -- define handled access rights
2. `landlock_add_rule()` -- grant specific access to specific paths
3. `landlock_restrict_self()` -- enforce (irreversible, can only tighten further)

```bash
# Using landrun CLI:
landrun --rw /project --ro /usr --ro /lib -- python agent.py
```

Access rights: `READ_FILE`, `WRITE_FILE`, `EXECUTE`, `READ_DIR`, `MAKE_DIR`, `REMOVE_FILE`, etc.
ABI 4 (Linux 6.7): TCP bind/connect restrictions.

Rust crate: `landlock`

Tooling:
- `landrun` -- Go CLI wrapping Landlock
- `ai-sandbox-landlock` -- Rust binary with YAML profiles
- `sandboxec` -- minimal single-binary wrapper

---

## FUSE-Based Approaches

FUSE (Filesystem in Userspace): kernel forwards VFS calls through /dev/fuse to userspace daemon. Enables complete interception, audit, and control of every file operation.

### AgentFS Architecture

1. FUSE server launches CoW overlay before agent starts
2. Host filesystem = read-only base layer; SQLite = writable delta layer
3. FUSE filesystem bind-mounted onto working directory
4. All writes captured in SQLite, reads pass through to host
5. Agent spawned inside sandbox with overlay mounted

Forensic queries: "what files modified?", "diff of file X at time T", "fork session state"

### sandboxfs (Google/Bazel)

FUSE filesystem exposing arbitrary host filesystem view without symlinks. Instant sandbox setup, FUSE overhead on individual I/O. Net win for CPU-bound workloads.

### Performance

FUSE adds 10-30% overhead on I/O-heavy workloads due to kernel-userspace context switches. Acceptable for agent workloads where LLM inference latency dominates.

---

## Protecting Agent Access to Secrets

### Sensitive Paths to Protect

| Path | Risk |
|------|------|
| `~/.ssh/` | Server access, git push |
| `~/.aws/credentials` | Cloud resource access |
| `~/.config/gcloud/` | GCP access |
| `~/.kube/config` | K8s cluster access |
| `~/.docker/config.json` | Registry creds |
| `.env`, `.env.*` | API keys, passwords |
| `*.pem`, `*.key` | TLS/SSL keys |
| `~/.gnupg/` | GPG signing |
| `~/.claude/` | Claude API keys |

### Protection Strategies

1. **Mount filtering**: Only mount what agent needs, never full home directory
2. **tmpfs over sensitive paths**: Mount tmpfs over ~/.ssh, ~/.aws to make invisible
3. **Landlock/AppArmor deny rules**: Explicitly deny sensitive paths
4. **Environment sanitization**: `env -i PATH=/usr/bin:/bin HOME=/sandbox agent-process`
5. **Secret injection at runtime**: Via secrets manager (Vault, Infisical), memory-only, never on disk
6. **Secret scanning**: trufflehog, gitleaks on agent workspace and output

### NVIDIA Red Team Guidance

OS-level enforcement over application-layer restrictions because it catches indirect execution paths (agent spawning subprocess that reads creds).

---

## Isolation Spectrum (Weakest to Strongest)

| Approach | Root Required | Isolation Strength | Overhead |
|---|---|---|---|
| Git worktrees | No | Code only (no runtime) | Near zero |
| Landlock | No | Filesystem + network ACLs | Negligible |
| Bubblewrap | No (user namespaces) | FS + network + PID | Low |
| FUSE overlay (AgentFS) | No | FS + audit trail | Moderate (I/O) |
| OverlayFS | Yes (or user ns) | Filesystem CoW | Low |
| Docker containers | No (daemon) | FS + network + PID + cgroups | Low-moderate |
| gVisor | No (runtime swap) | User-space kernel | Moderate |
| MicroVMs (Firecracker/Kata) | Yes | Separate kernel per agent | Higher (sub-200ms) |

---

## Sources

- https://docs.kernel.org/filesystems/overlayfs.html
- https://docs.openhands.dev/openhands/usage/architecture/runtime
- https://github.com/anthropic-experimental/sandbox-runtime
- https://turso.tech/blog/agentfs-overlay
- https://turso.tech/blog/agentfs-fuse
- https://github.com/dagger/container-use
- https://landlock.io/
- https://github.com/Zouuup/landrun
- https://blog.bazel.build/2017/08/25/introducing-sandboxfs.html
- https://developer.nvidia.com/blog/practical-security-guidance-for-sandboxing-agentic-workflows-and-managing-execution-risk/
- https://cheatsheetseries.owasp.org/cheatsheets/AI_Agent_Security_Cheat_Sheet.html
- https://2k-or-nothing.com/posts/Sandbox-Coding-Agents-Securely-With-Bubblewrap
- https://docs.docker.com/ai/sandboxes/architecture/
