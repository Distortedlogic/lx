-- Memory: diagnostic register. Honest assessment of what works and what's broken.
-- Rewrite when the assessment changes. Keep it short and honest.

# Design Health

Updated after Session 62 (2026-03-18).

## What Works

**Pipes + `^` + `??` compose beautifully.** `analyzer ~>? {task: "review"} ^ | (.findings) | filter (.critical)` — five operations, zero boilerplate, left-to-right. This composability is the language's strongest design choice.

**Boundary validation covers both directions.** `Protocol` validates agent-to-agent. `MCP` declarations validate agent-to-tool. `Trait` declarations validate agent behavioral contracts at definition time. No unvalidated boundary.

**`std/store` enables Rust→lx migration.** The generic concurrent k/v store extracts the DashMap+handle boilerplate that 14+ modules reimplement. First 5 conversions (knowledge, circuit, prompt, tasks, trace) each reduced to 30-50% of the Rust line count while passing all existing tests. The lx packages are readable, modifiable, and composable — exactly what lx is for.

## What's Still Wrong

**`lx check` is noisy on files with imports.** The type checker doesn't resolve `use` statements — it only sees the parsed AST of a single file. Any file that imports and uses external names produces false "undefined variable" diagnostics. `lx check` on the workspace reports 122 errors, almost all false positives from unresolved imports. Single-file `lx check tests/01_literals.lx` (no imports) works correctly. Fix requires the checker to either resolve imports or suppress diagnostics for imported names.

See `agent/PRIORITIES.md` for the full ordered work queue.

## What's Still Wrong (continued)

**Export names shadow builtins in lx packages.** `+filter` as an export name shadows the builtin `filter` HOF inside the module. Discovered when converting trace — had to rename `trace.filter` → `trace.query`. Any lx package exporting a name that matches a builtin (filter, map, fold, etc.) will hit this. Not a parser bug — it's environment scoping. Workaround: avoid builtin names in exports, or capture the builtin before the export (`keep = filter`).

**`? { }` is always parsed as match block.** `cond ? { ... }` after `?` starts a match block, not a regular block. Record spreads like `{..a, b: c}` inside `? { ... }` fail with "unexpected DotDot in pattern." Workaround: use parens `? ({..a, b: c})` or extract to a function.

## Bottom Line

Session 62: feature consolidation + `std/store` + 9 module→package conversions. 44 Rust stdlib → 35 Rust + 9 lx packages. Net ~-2770 lines Rust. Conversion wave hit its natural limit — budget/profile/pipeline stay Rust because lx lacks dynamic record field access, randomness, and hashing. Identified a real language gap: lx needs a generic `Class` keyword (Agent minus messaging) for plain stateful objects. 79/79 tests pass.
