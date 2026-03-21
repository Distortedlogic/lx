# Ghostty: The Terminal Emulator That Became the Agent-Era Default

Ghostty proves that **the terminal emulator is the missing infrastructure layer for agentic coding — and that GPU-accelerated rendering, native notifications, and an embeddable library (libghostty) make the difference between a terminal that survives agent workloads and one that chokes on them**. Built by Mitchell Hashimoto (co-founder of HashiCorp / Terraform / Vault), written in Zig, and now operating as a nonprofit under Hack Club's 501(c)(3), Ghostty went from zero to ~47K GitHub stars in 15 months by becoming the default terminal for Claude Code, Codex CLI, and the broader agentic coding community.

## Overview

| Metric | Value |
|--------|-------|
| **GitHub** | [ghostty-org/ghostty](https://github.com/ghostty-org/ghostty) |
| **Stars** | ~45,000-48,000 |
| **Language** | Zig |
| **Creator** | Mitchell Hashimoto (co-founder, HashiCorp) |
| **Public Launch** | December 2024 |
| **Latest Release** | v1.3.1 (March 13, 2026) |
| **License** | Open source (nonprofit under Hack Club 501(c)(3)) |
| **Platforms** | macOS, Linux (Windows planned) |
| **GUI** | macOS: Swift/AppKit + Metal; Linux: GTK4 + OpenGL |

## Architecture

### libghostty

The critical differentiator. Ghostty is not a monolithic terminal app — it's built on **libghostty**, a cross-platform, C-ABI compatible library that handles:

- Terminal emulation (VT sequence parsing)
- Font handling and discovery
- GPU rendering

Both the macOS and Linux GUIs are thin native shells built on top of this shared core. This architecture enables third-party projects to build purpose-built terminal UIs without reimplementing terminal emulation.

### libghostty-vt

The first standalone component being extracted — a zero-dependency library (doesn't require libc) for terminal sequence parsing and state management:
- SIMD-optimized parsing
- Unicode support
- Kitty Graphics Protocol
- Tmux Control Mode
- Neovim is considering switching from libvterm to libghostty-vt

### Platform-Native Rendering

| Platform | GUI Framework | GPU Backend | Font System |
|----------|--------------|-------------|-------------|
| **macOS** | Swift/AppKit/SwiftUI | Metal | CoreText |
| **Linux** | Zig + GTK4 C API | OpenGL | fontconfig |

Not Electron. Not a cross-platform toolkit pretending to be native. Actually native on each platform.

## Performance

| Metric | Ghostty | Alacritty | Kitty | iTerm2 | Warp |
|--------|---------|-----------|-------|--------|------|
| **Input Latency** | ~2ms | ~3ms | ~3ms | ~12ms | ~8ms |
| **Memory (idle)** | 60-100MB | ~30MB | 60-100MB | ~150MB+ | ~200MB+ |
| **Memory (heavy AI session)** | ~500MB | N/A | N/A | N/A | N/A |
| **IO (100K lines)** | ~0.7s | Slightly slower | 4x slower | 4x slower | N/A |
| **Startup** | <100ms | <100ms | ~200ms | ~500ms | ~300ms |
| **Frame Rate** | ~60fps under load | ~60fps | ~60fps | Drops under load | ~60fps |

The ~500MB under heavy AI agent sessions vs ~8GB for two VS Code terminal sessions is the number that matters for agentic workflows.

## Why Agents Love Ghostty

### 1. Claude Code Officially Recommends It

Anthropic's Claude Code documentation (`code.claude.com/docs/en/terminal-config`) lists Ghostty as a terminal where **Shift+Enter and desktop notifications work natively with zero configuration**. The `/terminal-setup` command doesn't even appear because everything just works.

### 2. Performance Under Agent Output Floods

AI agents produce enormous terminal output — streaming LLM responses, tool call results, file diffs, test output, build logs. Ghostty's GPU-accelerated rendering handles this without lag or memory bloat. Agents like Claude Code run sessions averaging 23 minutes with continuous multi-file output.

### 3. Native Notifications for Fire-and-Forget Workflows

Ghostty supports OSC 9/99/777 escape sequences for desktop notifications with zero configuration. When Claude Code finishes a task and waits for input, it fires a notification that surfaces as a native macOS/Linux notification automatically. This enables the "start agent, switch to other work, get notified when done" pattern.

### 4. Splits and Tabs for Multi-Agent Parallel Workflows

"Agentmaxxing" — running multiple AI coding agents in parallel — is a major workflow pattern. Ghostty's native splits and tabs, combined with tmux, allow running parallel Claude Code sessions with full visibility. The typical workflow: Ghostty + tmux + git worktrees, each worktree getting its own agent session in a split/tab.

### 5. libghostty Enables Purpose-Built Agent Terminals

**cmux** (7,700 GitHub stars in its first month, launched February 2026) is a native macOS terminal built on libghostty specifically for parallel AI agent workflows:
- Vertical tabs showing git branch/PR status/working directory
- Notification rings when agents need attention
- Centralized notification panel
- Built-in scriptable browser
- Socket API for automation

cmux demonstrates the strategic advantage: libghostty as an embeddable library means purpose-built agent UIs can be built without reimplementing terminal emulation from scratch.

### 6. Shell Integration and Hooks

Ghostty's shell integration (OSC 133) marks command boundaries, enabling "command finished" notifications. Combined with Claude Code's hook system, you can trigger custom actions (sounds, Slack messages, scripts) when agents complete tasks.

## The Ghostty Stack

The emerging best-practice stack for agentic development (early 2026):

1. **Ghostty** (or cmux on libghostty) — terminal
2. **tmux** — session/pane management
3. **git worktrees** — parallel branches (one per agent)
4. **Claude Code / Codex CLI / Gemini CLI** — the agent
5. **Notification hooks** — alert when agents finish

## Comparison to Other Terminals

| Feature | Ghostty | Alacritty | Kitty | WezTerm | iTerm2 | Warp |
|---------|---------|-----------|-------|---------|--------|------|
| **Language** | Zig | Rust | C/Python | Rust | Obj-C | Rust |
| **GPU** | Metal/OpenGL | OpenGL | OpenGL | WebGPU/OpenGL | Metal | Metal |
| **Ligatures** | Yes | No | Yes | Yes | Yes | Yes |
| **Images** | Kitty protocol | No | Kitty protocol | Yes | Inline | Yes |
| **Native UI** | Yes (per-platform) | No | No | No | Yes (macOS) | Yes (macOS) |
| **Notifications** | Native (zero-config) | No | Native | Limited | Requires setup | Native |
| **Embeddable lib** | libghostty | No | No | No | No | No |
| **Scriptable** | AppleScript | No | Kittens/Python | Lua | AppleScript | AI-native |
| **Agent-friendly** | Excellent | Minimal | Good | Good | Fair | Good (different approach) |
| **Stars** | ~47K | ~57K | ~25K | ~18K | ~15K | Closed |

**Why not the alternatives:**
- **Alacritty** — Too minimal: no ligatures, no images, no splits, no notifications
- **Kitty** — Strong feature set but no embeddable library and less native feel
- **WezTerm** — Most programmable (Lua) but slower rendering; maintainer stepped back from active development in 2024
- **iTerm2** — macOS-only, 12ms input latency (6x Ghostty), higher memory, notifications require manual setup
- **Warp** — AI built into the terminal itself (different approach), but closed-source, requires account creation, not embeddable
