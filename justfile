default:
  @just --list

# run cargo check + clippy
diagnose:
  cargo check
  cargo clippy -- -D warnings

# run all .lx suite tests
test:
  cargo run -p lx-cli -- test asl/suite/

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
