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

fmt:
    #!/usr/bin/env bash
    set -euo pipefail
    for member in $(cargo metadata --no-deps --format-version 1 | jq -r '.packages[].name'); do
      dx fmt -p "$member" > /dev/null
    done
    cargo fmt --all > /dev/null
    eclint -exclude "reference" .
    echo "fmt: ok"

test:
    #!/usr/bin/env bash
    set -euo pipefail
    cargo test --workspace --exclude inference-server --exclude lx-desktop --all-targets --all-features -q 2>&1
    cargo run -p lx-cli -- test

rust-diagnose:
    #!/usr/bin/env bash
    set -euo pipefail
    devdiag clippy

py-diagnose:
    #!/usr/bin/env bash
    set -euo pipefail
    devdiag ruff
    devdiag ty

py-fix:
    #!/usr/bin/env bash
    set -euo pipefail
    devdiag ruff

ts-diagnose:
    #!/usr/bin/env bash
    set -euo pipefail
    devdiag tsc

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
