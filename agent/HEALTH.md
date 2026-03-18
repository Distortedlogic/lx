-- Memory: diagnostic register. Honest assessment of what works and what's broken.
-- Rewrite when the assessment changes. Keep it short and honest.

# Design Health

Updated after Session 61 (2026-03-18).

## What Works

**Pipes + `^` + `??` compose beautifully.** `analyzer ~>? {task: "review"} ^ | (.findings) | filter (.critical)` — five operations, zero boilerplate, left-to-right. This composability is the language's strongest design choice.

**Boundary validation covers both directions.** `Protocol` validates agent-to-agent. `MCP` declarations validate agent-to-tool. `Trait` declarations validate agent behavioral contracts at definition time. No unvalidated boundary.

**Error messages are self-teaching.** Writing `if x then y` produces `undefined variable 'if' — lx uses 'cond ? then_expr : else_expr'`. Every type mismatch shows the actual value and type received. Agents learn lx syntax from the errors themselves.

## What's Still Wrong

**`lx check` is noisy on files with imports.** The type checker doesn't resolve `use` statements — it only sees the parsed AST of a single file. Any file that imports and uses external names produces false "undefined variable" diagnostics. `lx check` on the workspace reports 122 errors, almost all false positives from unresolved imports. Single-file `lx check tests/01_literals.lx` (no imports) works correctly. Fix requires the checker to either resolve imports or suppress diagnostics for imported names.

See `agent/PRIORITIES.md` for the full ordered work queue.

## Bottom Line

Session 61 fixed the parser foundation: list spread bp, module multi-level `../..`, Agent body uppercase tokens, dot access for uppercase fields. All 7 over-300-line files split. Remaining parser bugs (named-arg ternary ambiguity, assert greedy parsing) are inherent to juxtaposition syntax — workarounds documented, not fixable without parser architecture change. 78/78 tests pass. Ready for feature consolidation audit (Tier 0 Session 62).
