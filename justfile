import '../common.justfile'
set shell := ["bash", "-uc"]
set dotenv-load := true
set allow-duplicate-recipes := true

_default:
    #!/usr/bin/env bash
    set -euo pipefail
    just --choose

test:
    #!/usr/bin/env bash
    set -euo pipefail
    cargo test --workspace --exclude lx-desktop --all-targets --all-features -q 2>&1
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

# build vscode extension vsix (install via VS Code UI: Ctrl+Shift+P > Install from VSIX)
package-vscode:
    #!/usr/bin/env bash
    set -euo pipefail
    cd editors/vscode && pnpm install --frozen-lockfile && pnpm package

