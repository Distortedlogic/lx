# Runtime Monitoring, Auditing, and Observability for Sandboxed AI Agents

## System Call Tracing

### eBPF (Dominant Production Approach)

**AgentSight** (eunomia-bpf): Open-source eBPF framework for agent monitoring.
- Non-intrusive probes capturing decrypted "Intent Stream" (LLM prompts/responses intercepted from SSL) and "Action Stream" (syscalls, process events)
- Triple correlation: process lineage tracking (fork/execve), temporal correlation, content matching
- Under 3% overhead across developer workflows monitoring 6 collaborating Claude Code subagents
- https://github.com/eunomia-bpf/agentsight

### Linux audit subsystem (auditd)

Compliance-oriented logging via `/etc/audit/rules.d/`:
```bash
auditctl -w /workspace -p rwxa -k agent-workspace
auditctl -a always,exit -F arch=b64 -S connect -k agent-network
auditctl -a always,exit -F arch=b64 -S execve -k agent-exec
```

Mature, well-understood, but lacks eBPF's programmability and low overhead.

### strace

2-5x slowdown. Use only for development profiling (determine which syscalls agent needs → inform seccomp profiles).

---

## Real-Time Anomaly Detection

### Behavioral Baselines

Run agent through representative tasks, record: files accessed, syscalls made, network connections, processes spawned. Alert on deviations.

### Anomaly Categories

| Category | Normal | Anomalous | Risk |
|----------|--------|-----------|------|
| File access | /workspace/** | /etc/shadow, ~/.ssh/ | Credential theft |
| Network | Package registries | Unknown IPs, DNS tunneling | Exfiltration |
| Process | python, git, npm | curl, wget, nc, nmap | Recon/attack |
| Syscalls | read, write, open | ptrace, mount, kexec | Sandbox escape |
| Resource | Steady CPU/memory | Spikes, fork bombs | DoS |

### Detection Approaches

- **Rule-based**: Simple, fast, low false positive (e.g., alert on access outside /workspace)
- **Cross-stage correlation**: ARMO maps agent escapes across 5 predictable stages using eBPF
- **Sequence-based**: Read creds → encode → network = exfiltration; download binary → chmod +x → execute = malware
- **Rate-of-change alerts**: 3x daily average cost/call volume → catch loops and runaway retries

---

## Audit Logging Requirements

### What to Log

| Event Type | Fields | Retention |
|-----------|--------|-----------|
| Tool calls | tool, parameters, caller_id, timestamp | 90+ days |
| File operations | path, operation, size, timestamp | 90+ days |
| Network requests | destination, port, protocol, payload_size | 90+ days |
| Process execution | command, args, user, exit_code | 90+ days |
| LLM interactions | prompt hash, response hash, model, tokens | 30+ days |
| Auth events | who, what, success/failure | 1+ year |
| Sandbox lifecycle | create, start, stop, destroy | 90+ days |

### Compliance Retention

- EU AI Act Article 12 (Aug 2, 2026): Structured logging, min 6 months
- California ADMT (Sept 2025): 5-year retention for financial/housing/employment/healthcare
- SOC 2: 1 year
- HIPAA: 6 years
- Canada DADM: compliance by June 24, 2026

### Tamper-Proofing

1. **Append-only storage**: Kafka (compaction disabled), S3 Object Lock, Azure Immutable Blob
2. **Hash chaining**: Each entry includes hash of previous (reveals tampering)
3. **Separate log collector**: Logs sent out-of-band, agent cannot access log storage
4. **Digital signatures**: Ed25519 signing per entry (AEGIS framework)
5. **Write-ahead logging**: Logs written before action execution

### Log Pipeline

```
Agent Sandbox → syslog/journald → Fluent Bit → SIEM (ClickHouse/ES) → Alerting
                                              → Cold Storage (S3/MinIO) → Retention
```

---

## Kill Switches and Circuit Breakers

### Kill Switch (Global Hard Stop)

- Revokes all tool permissions, halts queued jobs, locks pipelines
- Must live in control plane the agent cannot modify
- Must be externally accessible (one tap/API call)
- Must be testable (regularly exercised like circuit breakers)
- Use SIGKILL not SIGTERM for emergency stops

```bash
# Kill via cgroup
echo 1 > /sys/fs/cgroup/agent-sandbox/cgroup.kill
# Kill via container
docker kill agent-container --signal KILL
# Freeze (pause without killing)
echo 1 > /sys/fs/cgroup/agent-sandbox/cgroup.freeze
```

### Circuit Breakers

- Kill after N steps, $X cost, or K consecutive errors
- AgentBudget (agentbudget.dev): real-time cost enforcement with hard limits
- Soft limits at 50%/80% (alerts), hard limit at 100% (automatic stop)

### Budget-Based Cutoffs

Monthly bills dropped from $180 to $94 by catching runaway runs.

### Anti-Pattern

"Confirm before acting" dialogs are NOT a safety system. Create alert fatigue, trivially bypassed.

---

## Resource Limits

### cgroups v2

| Resource | Mechanism | Config |
|----------|-----------|--------|
| CPU | cpu.max | 200000 100000 (2 cores) |
| Memory | memory.max / memory.high | 4G hard, 2G throttle |
| Swap | memory.swap.max | 0 (disabled) |
| PIDs | pids.max | 256 (prevent fork bombs) |
| I/O | io.max | rbps=50MB/s, wbps=25MB/s |
| Disk | tmpfs size, XFS quotas | 10G per workspace |
| Time | Timeout wrappers, SIGTERM/SIGKILL | 30s simple, 600s tests |
| Tokens | Application-layer | Hard limits per run |

Critical caveat: cgroups prevent DoS, not escape. Same kernel attack surface.

---

## Runtime Security Tools

### Comparison

| Tool | Engine | Detection | Enforcement | CPU | Memory | CNCF |
|------|--------|-----------|-------------|-----|--------|------|
| Falco | eBPF + kernel module | Yes | No (alert only) | Moderate | Lowest | Graduated |
| Tracee | eBPF | Yes | Partial | Highest | Moderate | Sandbox |
| Tetragon | eBPF + Cilium | Yes | **Yes (SIGKILL)** | **Lowest** | Moderate | Incubating |

### Tetragon (Recommended for Agents)

Only tool that can enforce policy in-kernel -- SIGKILL on violation. Critical because detection-only allows threats to continue.

```yaml
apiVersion: cilium.io/v1alpha1
kind: TracingPolicy
spec:
  kprobes:
    - call: "sys_openat"
      selectors:
        - matchArgs:
            - index: 1
              operator: "Prefix"
              values: ["/etc/shadow", "/root/.ssh"]
      matchActions:
        - action: Sigkill
```

### Falco (Complementary Detection)

93 community rules, YAML format:
```yaml
- rule: Agent Reading Sensitive Files
  condition: >
    open_read and container and
    (fd.name startswith /root/.ssh/ or fd.name endswith /.aws/credentials)
  output: "Sensitive file read (file=%fd.name container=%container.name)"
  priority: WARNING
```

**Recommendation**: Tetragon for enforcement + Falco for broad detection/alerting.

---

## Detecting Sandbox Escape Attempts

### Escape Vectors

1. Privileged containers (never run agents as privileged)
2. Kernel exploits (use gVisor/Firecracker for separate kernel)
3. Sensitive mount leaks (/proc, /sys, Docker socket)
4. runc CVEs (2025: CVE-2025-31133, CVE-2025-52565, CVE-2025-52881)

### ARMO's 5-Stage Detection

Agent escapes follow predictable stages detectable via eBPF:
1. Unexpected mount operations
2. setns calls (namespace manipulation)
3. ptrace usage (process injection)
4. Unusual procfs/sysfs access
5. Capability escalation attempts

### seccomp Blocking

For agents, specifically block: `clone3` (nested namespace escape), `io_uring` (force epoll fallback), `ptrace`, `kexec_load`, `init_module`, `finit_module`.

---

## Checkpoint/Restore (CRIU)

Freezes running process tree, saves complete state to disk (memory, FDs, sockets, timers), restores exactly.

### Agent Use Cases

1. **Pause for human review**: Checkpoint before high-risk action, human inspects, restore or discard
2. **Cost optimization**: Checkpoint idle agents, restore on-demand
3. **Forensic snapshots**: Checkpoint misbehaving agent for post-mortem
4. **Migration**: Move agents between nodes

### Kubernetes Integration

- Alpha in K8s v1.25, beta in v1.30
- Checkpoint/Restore Working Group announced Jan 2026

### CRIUgpu

Transparent GPU container checkpointing (NVIDIA cuda-checkpoint + CRIU). Tested with GPT-2 (1.5B) and LLaMA 3.1 (8B). Integrated in CRIU 4.0+.

### Limitations

- Requires CAP_SYS_ADMIN (only orchestrator should have, not agent)
- Not all processes checkpointable (some kernel threads)
- Checkpoint images contain process memory (may include secrets -- must encrypt)

---

## Human-in-the-Loop Approval

### Risk-Based Strategy

- 100% approval for high-risk (irreversible, regulated, high blast-radius)
- 5-20% sampling for low-risk (monitor drift)
- Auto-approve routine within baselines

### Patterns

| Pattern | Behavior | Best For |
|---------|----------|----------|
| Synchronous | Pause agent, await human | Irreversible actions |
| Asynchronous | Queue action, continue other work | Medium-risk, throughput |
| Budget-based | N approval tokens per session | Long-running sessions |
| Policy-based | Auto-approve/deny by rule, ask when unsure | Production systems |

### Governance-as-Code

Embed approval logic in execution graph:
- LangGraph: explicit human approval nodes
- LangChain: HumanInTheLoopMiddleware
- OPA: version-controlled, testable policy rules

### Scaling Challenge

Human-in-the-loop has hit the wall (SiliconANGLE, Jan 2026). AI-overseeing-AI ("supervisor agents") needed for scale, with humans reviewing supervisors rather than every action.

### EU AI Act Article 14

Human oversight is a legal requirement for high-risk AI systems (effective Aug 2, 2026).

---

## Sources

- https://arxiv.org/html/2508.02736v1
- https://github.com/eunomia-bpf/agentsight
- https://www.armosec.io/blog/ai-agent-escape-detection/
- https://galileo.ai/blog/real-time-anomaly-detection-multi-agent-ai
- https://galileo.ai/blog/ai-agent-compliance-governance-audit-trails-risk-management
- https://www.sakurasky.com/blog/missing-primitives-for-trustworthy-ai-part-6/
- https://arxiv.org/html/2511.13725v3
- https://agentbudget.dev
- https://tetragon.io/docs/concepts/enforcement/
- https://falco.org/docs/reference/rules/default-rules/
- https://www.scitepress.org/Papers/2025/142727/142727.pdf
- https://accuknox.com/technical-papers/container-runtime-security-comparison
- https://blaxel.ai/blog/container-escape
- https://criu.org/Main_Page
- https://arxiv.org/html/2502.16631v1
- https://www.kubernetes.dev/blog/2026/01/21/introducing-checkpoint-restore-wg/
- https://www.stackai.com/insights/human-in-the-loop-ai-agents
- https://www.permit.io/blog/human-in-the-loop-for-ai-agents-best-practices-frameworks-use-cases-and-demo
- https://siliconangle.com/2026/01/18/human-loop-hit-wall-time-ai-oversee-ai/
