#!/usr/bin/env sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$ROOT"

echo "==> Running closed-error gate check (non-exhaustive-check / RST-0006)..."
# non-exhaustive-check enforces closed public error enums (C4.5/C4.6 / RST-0006:R1)
cargo run --locked -p non-exhaustive-check -- "$ROOT/crates"

echo "==> Running unsafe-code gate check (forbid-unsafe-total / RST-0005)..."
# All crate roots must enforce #![forbid(unsafe_code)]
for root_file in "$ROOT"/crates/*/src/main.rs "$ROOT"/crates/*/src/lib.rs; do
  if [ -f "$root_file" ]; then
    grep -q 'forbid(unsafe_code)' "$root_file" || {
      echo "::error::forbid-unsafe-total: $root_file lacks #![forbid(unsafe_code)] (RST-0005)" >&2
      exit 1
    }
  fi
done

echo "==> Running cargo test..."
cargo test --workspace --all-features --locked

echo "==> Running cargo clippy..."
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings

echo "==> Running cargo fmt check..."
cargo fmt --all -- --check

echo "==> All tripwires verification checks passed."
