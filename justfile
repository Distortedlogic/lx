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
