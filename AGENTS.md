# AGENTS.md — tripwires

Repo-specific operational notes. General agent/OODA doctrine, bd/beads
conventions, bash hygiene, and the Rust no-`//`-comments rule live in the
global `~/.config/opencode/AGENTS.md` (auto-loaded) — not repeated here.

## Repository purpose

Canonical repository housing shared fleet structural tripwires and CI gates,
starting with `non-exhaustive-check`:
- `non-exhaustive-check` — CI hard-gate enforcing closed public error enums
  (C4.5/C4.6 / RST-0006:R1).
- Exit codes adhere strictly to the fleet tri-state taxonomy:
  - `0`: clean pass / compliant / all assertions verified.
  - `1`: domain defect / violation / finding flagged.
  - `2`: unknown / environmental error / missing permission / indeterminate.
- Stream separation: structured machine-readable findings (TSV, JSON Lines)
  stream to `stdout`; operational telemetry, diagnostic logs, and error traces
  route to `stderr`.

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
- `cargo deny check` and `cargo audit` are supply-chain gates; run
  before publishing or bumping dependencies.

## Rustdoc budget gate

Run the same native check from the repository root locally and in CI:

```sh
comment-free --check-doc-budget --doc-advisory-words 80 --doc-max-words 120 --max-warning-files 0 .
```

Requires comment-free 0.2.0 at the canonical revision below:

```sh
cargo +1.98.0 install --git https://github.com/acje/comment-free --rev e45de7ef3b0fcd9a1ec299b9026b14fb5b0cf534 --locked comment-free
```

The read-only native gate recursively scans Rust sources under `.` with the
tool's build/hidden pruning: 80 prose words is advisory; 120 is enforced.
Fenced code is excluded by the tool. Summary-only output retains full totals
while suppressing finding details; diagnostics remain visible.
Native gate exits are 0 for pass, 1 for enforced breach, and 2 for
unknown/error, including undecided payloads or empty scope. Policy and its
implementation/tests/proofs belong upstream; repository checks establish
integration only. No rewrite mode runs.
Macro-generated docs without spelled `doc` tokens remain outside detection;
this is not proof of semantic documentation coverage or process-memory bounds.

## Beads configuration

- Issue prefix: `tw` (e.g. `tw-1`, `tw-a3f2dd`).
- Database: repo-local store at `.beads/embeddeddolt`.
- Pinned discovery: use `bd -C <repo-root>` to target this workspace directly.
- Preserve `.beads` scaffold (`config.yaml`, `metadata.json`, `hooks/`); database
  files and runtime exports remain untracked per `.gitignore`.

## Non-Interactive Shell Execution & Bash Hygiene

Subagents run non-interactively. Any command that could trigger an interactive
y/n prompt stalls execution indefinitely.
- Use explicit non-interactive flags: `cp -f`, `rm -f`, `rm -rf`.
- Git operations: use non-interactive commands; no interactive rebase (`git rebase -i`).
- Tooling CLI options: accept batch flags (`--batch`, `-y`, `--quiet`).
