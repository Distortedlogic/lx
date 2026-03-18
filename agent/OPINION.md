# Design Opinion

Written by the language designer (Claude). Updated after Session 52 (2026-03-18).

## What Works

**Pipes + `^` + `??` compose beautifully.** `analyzer ~>? {task: "review"} ^ | (.findings) | filter (.critical)` — five operations, zero boilerplate, left-to-right. This composability is the language's strongest design choice.

**Boundary validation covers both directions.** `Protocol` validates agent-to-agent. `MCP` declarations validate agent-to-tool. `Trait` declarations validate agent behavioral contracts at definition time. No unvalidated boundary.

**Error messages are self-teaching.** Writing `if x then y` produces `undefined variable 'if' — lx uses 'cond ? then_expr : else_expr'`. Every type mismatch shows the actual value and type received. Agents learn lx syntax from the errors themselves.

## What's Still Wrong

**Record field value parsing is too restrictive on single lines.** `{key: f x y  other: z}` on one line doesn't work — the parser terminates the value too early. Multiline records work fine. Every flow author learns to use multiline or extract to temp bindings.

**No pipeline checkpoint/resume.** Multi-stage workflows restart from scratch when a late stage fails. This is the top priority in `agent/PRIORITIES.md`.

See `agent/PRIORITIES.md` for the full ordered work queue.

## Bottom Line

The core agent architecture is solid — spawn, message, validate, supervise, reconcile, and resource-scope all work end-to-end. Error handling is now uniform (field miss → None, Protocol fail → Err, tagged errors for pattern matching). The main remaining pain point is pipeline checkpoint/resume.
