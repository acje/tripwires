# AGENTS.md — tripwires

Repo-specific operational notes. General agent/OODA doctrine, bd/beads
conventions, bash hygiene, and the Rust no-`//`-comments rule live in the
global `~/.config/opencode/AGENTS.md` (auto-loaded) — not repeated here.

## Repository purpose

Canonical repository housing shared fleet structural tripwires and CI gates,
starting with `non-exhaustive-check`:
- `non-exhaustive-check` — CI hard-gate enforcing closed public error enums
  (C4.5/C4.6 / RST-0006:R1).

## Rust policy and toolchain

- Toolchain: pinned channel **1.98.0** (`rust-toolchain.toml`), MSRV **1.98**,
  edition **2024**, resolver **3**.
- Members inherit root dependencies and `[lints] workspace = true`. The
  workspace manifest defines `pedantic = warn` and the centralized clippy warning roster,
  enforced with `-D warnings`.
- Format: stable-default rustfmt. No custom `rustfmt.toml`.
- Crate roots forbid unsafe code (`#![forbid(unsafe_code)]`).
- House style: zero non-doc comments (`//` or `/* */`). Doc comments (`///`, `//!`)
  are written only for public API contracts and required sections. Suppress lints with
  `#[expect(lint, reason = "…")]` (or `#[allow(lint, reason = "…")]` when unfulfilled).

## Verification tiers

- **INNER** (every TDD increment, changed crate only):
  ```sh
  CARGO_TERM_PROGRESS_WHEN=never cargo test -p <crate> --message-format=short
  CARGO_TERM_PROGRESS_WHEN=never cargo clippy -p <crate> --all-targets --message-format=short -- -D warnings
  ```
- **MID** (once at sub-mission completion, changed crates plus reverse dependents):
  ```sh
  cargo test --all-targets --locked
  cargo clippy --all-targets --locked -- -D warnings
  cargo fmt --all -- --check
  ```
- **BOUNDARY** (once per epic, full workspace):
  ```sh
  cargo build --workspace --all-features --locked
  timeout 900 cargo test --workspace --all-features --locked --no-fail-fast
  cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
  cargo fmt --all -- --check
  ```

## Beads configuration

- Issue prefix: `tw` (e.g. `tw-1`, `tw-a3f2dd`).
- Database: repo-local store at `.beads/embeddeddolt`.
- Pinned discovery: use `bd -C <repo-root>` to target this workspace directly.
- Preserve `.beads` scaffold (`config.yaml`, `metadata.json`, `hooks/`); database
  files and runtime exports remain untracked per `.gitignore`.
