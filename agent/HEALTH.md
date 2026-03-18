-- Memory: diagnostic register. Honest assessment of what works and what's broken.
-- Rewrite when the assessment changes. Keep it short and honest.

# Design Health

Updated after Session 55 (2026-03-18).

## What Works

**Pipes + `^` + `??` compose beautifully.** `analyzer ~>? {task: "review"} ^ | (.findings) | filter (.critical)` — five operations, zero boilerplate, left-to-right. This composability is the language's strongest design choice.

**Boundary validation covers both directions.** `Protocol` validates agent-to-agent. `MCP` declarations validate agent-to-tool. `Trait` declarations validate agent behavioral contracts at definition time. No unvalidated boundary.

**Error messages are self-teaching.** Writing `if x then y` produces `undefined variable 'if' — lx uses 'cond ? then_expr : else_expr'`. Every type mismatch shows the actual value and type received. Agents learn lx syntax from the errors themselves.

## What's Still Wrong

**Record field value parsing is too restrictive on single lines.** `{key: f x y  other: z}` on one line doesn't work — the parser terminates the value too early. Multiline records work fine. Every flow author learns to use multiline or extract to temp bindings.

**`lx check` is noisy on files with imports.** The type checker doesn't resolve `use` statements — it only sees the parsed AST of a single file. Any file that imports and uses external names produces false "undefined variable" diagnostics. `lx check` on the workspace reports 122 errors, almost all false positives from unresolved imports. Single-file `lx check tests/01_literals.lx` (no imports) works correctly. Fix requires the checker to either resolve imports or suppress diagnostics for imported names.

See `agent/PRIORITIES.md` for the full ordered work queue.

## Bottom Line

The core agent architecture is solid — spawn, message, validate, supervise, reconcile, and resource-scope all work end-to-end. Error handling is now uniform (field miss → None, Protocol fail → Err, tagged errors for pattern matching). Workspace fully operational — manifests, cross-member module resolution (`use brain/protocols`), `lx run/test/check/list` all workspace-aware. `std/pipeline` now provides checkpoint/resume for multi-stage workflows — no more re-executing completed stages on failure. Next priority: `AgentErr` structured errors.
