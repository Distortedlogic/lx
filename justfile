import '../common.justfile'
set shell := ["bash", "-uc"]
set dotenv-load := true

_default:
    #!/usr/bin/env bash
    set -euo pipefail
    just --choose

clear:
    #!/usr/bin/env bash
    set -euo pipefail
    cargo clean
    rm *.lock

test:
    #!/usr/bin/env bash
    set -euo pipefail
    cargo test --workspace --exclude inference-server --exclude lx-desktop --all-targets --all-features -q 2>&1
    cargo run -p lx-cli -- test

# run lx-tui with a .lx file
tui:
    #!/usr/bin/env bash
    set -euo pipefail
    cargo run -p lx-tui

# run lx-desktop app
desktop:
    #!/usr/bin/env bash
    set -euo pipefail
    dx serve -p lx-desktop

# run nodeflow app (n8n-style workflow editor)
nodeflow:
    #!/usr/bin/env bash
    set -euo pipefail
    dx serve -p nodeflow

# run lx-mobile app
mobile:
    #!/usr/bin/env bash
    set -euo pipefail
    dx serve -p lx-mobile --platform android

# render all .mmd diagrams to .png recursively
diagrams:
    #!/usr/bin/env bash
    set -euo pipefail
    shopt -s globstar nullglob
    for f in **/*.mmd; do
      mmdc -i "$f" -o "${f%.mmd}.png" -c mermaid.config.json -b "#000000" -s 8
    done
    echo "diagrams: ok"

# build vscode extension vsix (install via VS Code UI: Ctrl+Shift+P > Install from VSIX)
package-vscode:
    #!/usr/bin/env bash
    set -euo pipefail
    cd editors/vscode && pnpm install --frozen-lockfile && pnpm package

