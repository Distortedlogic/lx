#!/usr/bin/env bash
set -euo pipefail

payload=$(cat)
file_path=$(printf '%s' "$payload" | jq -r '.tool_input.file_path // empty')

[[ -n "$file_path" && "$file_path" == *.mmd ]] || exit 0

project_root="${CLAUDE_PROJECT_DIR:-}"
if [[ -z "$project_root" ]]; then
  project_root=$(git -C "$(dirname "$file_path")" rev-parse --show-toplevel 2>/dev/null || true)
fi
if [[ -z "$project_root" || ! -f "$project_root/mermaid.config.json" ]]; then
  echo "mermaid-render: could not locate mermaid.config.json" >&2
  exit 0
fi

if ! command -v mmdc >/dev/null 2>&1; then
  echo "mermaid-render: mmdc not found in PATH" >&2
  exit 0
fi

cd "$project_root"
rel="${file_path#$project_root/}"
out="${rel%.mmd}.png"

if output=$(mmdc -i "$rel" -o "$out" -c mermaid.config.json -b "#000000" -s 8 2>&1); then
  echo "mermaid-render: $out"
  exit 0
else
  printf 'mermaid-render failed for %s:\n%s\n' "$rel" "$output" >&2
  exit 2
fi
