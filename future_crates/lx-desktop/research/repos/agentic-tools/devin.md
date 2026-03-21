# Devin: Cognition's Fully Autonomous VM-Based Coding Agent

Devin proves that **a fully sandboxed cloud VM with terminal, editor, and browser — operated autonomously by an AI agent — can achieve a 67% PR merge rate on well-scoped tasks, positioning autonomous agents as viable junior engineers for backlogs**. Built by Cognition AI, Devin 2.0 dropped the price from $500/month to $20/month, and the company's acquisition of Windsurf (after Google poached the CEO for $2.4B) valued Cognition at $10.2 billion.

## Overview

| Metric | Value |
|--------|-------|
| **Website** | [devin.ai](https://devin.ai) |
| **Developer** | Cognition AI |
| **Valuation** | $10.2B (September 2025, two months after Windsurf acquisition) |
| **PR Merge Rate** | 67% (up from 34% year-over-year) |
| **Speed Improvement** | 4x faster at problem solving vs. prior year |
| **Efficiency** | 2x more efficient in resource consumption |
| **Collaboration** | Slack, Teams, Jira integration |

## Pricing

| Tier | Price | Included |
|------|-------|----------|
| **Individual** | $20/month minimum | $2.25 per Agent Compute Unit (ACU) |
| **Teams** | Higher tier | 250 ACUs/month, parallel sessions |
| **Enterprise** | Custom | Custom ACU allocation |

## Architecture

### Full VM Sandbox

Devin operates inside a fully sandboxed cloud environment containing:
- **Shell** — Full terminal access
- **Code Editor** — Built-in editor for file operations
- **Web Browser** — Browser for testing, research, and validation

This is fundamentally different from CLI agents (Claude Code, Codex, Gemini CLI) that run in the developer's local environment. Devin's VM isolation means it cannot damage the host system but also cannot access local-only resources.

### Autonomous Workflow

1. Receive task (via Slack, Teams, Jira, or web interface)
2. Spin up sandboxed VM
3. Clone repository, analyze codebase
4. Plan implementation
5. Write code, run tests, debug
6. Create feature branch, open PR
7. Respond to code review feedback

## Key Features

### Windsurf Acquisition

Cognition acquired Windsurf (the remaining team and technology after Google poached CEO Varun Mohan and ~40 engineers for $2.4B) in July 2025. The acquisition includes Windsurf's IP, product, trademark, brand, and team. Cognition's valuation jumped to $10.2B two months later.

### Playbooks

System prompts for repeated tasks. Create playbooks to standardize common workflows:
- The playbooks page shows session count, unique users, merged PRs per playbook
- Weekly activity charts for monitoring
- Customizable per team or project

### Scheduled Sessions

As of February 2026, Devin supports recurring scheduled sessions:
- One-time or recurring runs
- Agent selection
- Notification preferences
- Configurable frequency, prompt, and playbook

### Devin Desktop (v2.2)

Desktop application for direct interaction outside Slack/web, including code review capabilities.

### Performance Review Data

From Cognition's "Devin 2025 Performance Review":
- 67% PR merge rate (up from 34%)
- 4x faster problem solving
- 2x more efficient resource consumption
- Best suited for well-defined, repetitive tasks with clear success criteria

## Competitive Position

Devin occupies a unique position as the most autonomous agent — it requires the least human interaction but also has the least real-time developer control. The "assign and forget" model works for backlogs but struggles with ambiguous requirements.

| Aspect | Devin | Claude Code | Cursor |
|--------|-------|-------------|--------|
| **Autonomy** | Full (VM sandbox) | High (terminal) | Medium (IDE) |
| **Interaction** | Async (Slack/Jira/PR) | Real-time (terminal) | Real-time (IDE) |
| **Environment** | Cloud VM | Local machine | Local machine |
| **PR Automation** | Native (creates PRs) | Manual | Manual |
| **Best For** | Backlogs, repetitive tasks | Complex architecture | Quick edits, iteration |
| **Scheduling** | Yes (recurring sessions) | No | No |
| **Price** | $20/mo + ACU | API costs | $20/mo subscription |
