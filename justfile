default:
  @just --list

# run cargo check + clippy
diagnose:
    #!/usr/bin/env python3
    import subprocess, sys
    sys.path.insert(0, "scripts")
    from diagnose_parser import parse_cargo_json, group_and_format
    proc = subprocess.run(
        ["cargo", "clippy", "--all-targets", "--message-format=json"],
        capture_output=True, text=True)
    deduped = parse_cargo_json(proc.stdout.splitlines())
    output, total_e = group_and_format(deduped)
    for line in output:
        print(line)
    if total_e > 0:
        sys.exit(1)
    print("diagnose: ok")

# run all .lx suite tests
test:
  cargo run -p lx-cli -- test tests/

# run a single .lx file
run file:
  cargo run -p lx-cli -- run {{file}}

# format all rust code
fmt:
  cargo fmt

# check formatting without writing
fmt-check:
  cargo fmt -- --check

# build in release mode
build:
  cargo build --release

# compile release and install lx binary system-wide
install:
  cargo install --path crates/lx-cli

# run work-item generation against an audit checklist (interactive chooser)
audit:
  #!/usr/bin/env bash
  shopt -s nullglob
  files=(rules/*audit* rules/*-audit*)
  unique=($(printf '%s\n' "${files[@]}" | sort -u))
  if [[ ${#unique[@]} -eq 0 ]]; then
    echo "No audit files found in rules/"
    exit 1
  fi
  if command -v fzf &>/dev/null; then
    file=$(printf '%s\n' "${unique[@]}" | fzf --prompt="Select audit: ")
  else
    echo "Select audit file:"
    select file in "${unique[@]}"; do break; done
  fi
  if [[ -n "$file" ]]; then
    AUDIT_FILE="$file" cargo run -p lx-cli -- run workgen/run.lx
  else
    echo "No file selected"
    exit 1
  fi

# run work-item generation for a specific audit file
audit-file file:
  AUDIT_FILE={{file}} cargo run -p lx-cli -- run workgen/run.lx

# run workgen satisfaction tests (optional: TEST_TAG=smoke for filtered)
audit-test tag="":
  TEST_TAG={{tag}} cargo run -p lx-cli -- run workgen/tests/run.lx
