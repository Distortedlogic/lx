#!/usr/bin/env bash
# Inject project justfile recipes into Claude's context at session start.
# Mirrors @pi-mermaid/pi-ext-justfile (pi's before_agent_start) but fires once per session.

out=$(timeout 4 just --list --list-submodules --unsorted --list-heading '' --list-prefix '' 2>/dev/null) || exit 0
[ -n "$out" ] || exit 0

cat <<'HEADER'
## Project Justfile Recipes

This project has a justfile. The following recipe list comes directly from:
`just --list --list-submodules --unsorted --list-heading '' --list-prefix ''`

```text
HEADER
printf '%s\n' "$out"
cat <<'FOOTER'
```

Instructions:
- Prefer using an existing just recipe over hand-rolled bash when it matches the task.
- For top-level recipes, invoke them with `just <recipe>`.
- For recipes listed under a module section like `infra:`, prefer invoking them as `just infra::<recipe>`.
- If you need details before running a recipe, use bash with `just --usage <recipe>` or `just --show <recipe>`.
- For logs, watch, dev-server, restart, setup, build, test, deploy, and maintenance tasks, prefer the just recipe instead of reconstructing the underlying shell commands.
- Only hand-roll bash when no suitable just recipe exists, or when the user explicitly asks for raw shell commands instead of the project recipe.
FOOTER
