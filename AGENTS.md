# AGENTS.md — tripwires

Repo-specific operational notes. General agent/OODA doctrine, bd/beads
conventions, bash hygiene, and the Rust no-`//`-comments rule live in the
global `~/.config/opencode/AGENTS.md` (auto-loaded) — not repeated here.

## Section 1: Canonical Fleet Doctrine

### OODA Loop Roles
- **Copernicus** (Observe): Raw evidence gathering from environment, code, and external specs. Pure sensor; produces no hypotheses.
- **Feynman** (Orient): Produces ranked hypotheses with falsifiers; stress-tests against concrete examples.
- **Moltke** (Decide): Standing mission commander. Emits executable mission contracts, sets intent, boundaries, and abort criteria.
- **Hopper** (Act): Executes missions using Kent Beck TDD (red-green-refactor) with verify-before-claim discipline.
- **Linus** (Review): Mandatory pre-merge Rust reviewer for idiom conformance, type safety, unsafe soundness, and supply chain.
- **Hamilton** (Assurance): Architectural alignment and assurance reviewer running during CI wait windows.
- **Gardener** (GC): Post-mission cleanup specialist; reclaims transient scaffolding and closes completed mission beads.

### Priority Hierarchy
Tradeoffs strictly resolve in this five-tier priority order:
1. **Maintainability**: Pure trunk development, small deployable increments, minimal cognitive overhead, low complexity.
2. **Correctness by design**: Make illegal states unrepresentable via types, explicit state machines, and private invariant constructors.
3. **Response times**: Latency-sensitive read paths and prompt fact propagation across boundaries.
4. **Energy efficiency in code**: Minimize redundant polling, hot loops, unnecessary serialization, and idle CPU/memory burn.
5. **Features**: New functionality ranks last and must never compromise the higher tiers.

### Non-Interactive Shell Commands & Bash Hygiene
Subagents execute non-interactively. Commands that prompt for user confirmation stall execution indefinitely.
- Always use non-interactive and force flags: `cp -f`, `rm -f`, `rm -rf`.
- Streaming and batch mode: use `--batch`, `-y`, or `--quiet` where available.
- Stream separation: machine-readable findings route to `stdout`; diagnostics and logs route to `stderr`.

### Zero Plain Comments
In Rust source (`*.rs`), plain comments (`//` or `/* */`) are forbidden.
- Rationale belongs in commit messages, ADRs, or bead descriptions.
- Use `///` or `//!` contract doc-comments only when defining public API documentation (with required `# Errors`, `# Panics`, `# Safety` sections).
- Suppress lints with `#[expect(lint, reason = "...")]` rather than plain comments.

### Doctrine: "Make tools fast to iterate fast"
Developer and verification tooling must be compiled, ultra-fast Rust binaries operating directly on ASTs and files rather than slow interpreted wrappers or token-heavy in-context simulation. Fast tools enable high-frequency local feedback loops (INNER cadence) without friction.

### Doctrine: "Zero compliance theatre"
High-assurance testing techniques—such as property-based testing (proptest), fuzzing (cargo-fuzz), formal model checking, or fault injection—must be applied purposefully at critical serialization, concurrency, and storage boundaries (high-risk seams), not sprayed ubiquitously as box-ticking ceremony. Where type invariants and deterministic unit tests suffice, do not add compliance overhead.

## Section 2: Target-Specific Profile

### Target Classification & Entrypoint
- Target class: `attended-app` (as mapped in `sf-sdlc.toml`).
- Real entrypoint: `crates/non-exhaustive-check/src/main.rs`.
- Canonical verification entrypoint: `scripts/verify.sh`
- Repository purpose: Canonical repository housing shared fleet structural tripwires
  and CI gates, starting with `non-exhaustive-check`:
  - `non-exhaustive-check` — CI hard-gate enforcing closed public error enums
    (C4.5/C4.6 / RST-0006:R1).
- Exit codes adhere strictly to the fleet tri-state taxonomy:
  - `0`: clean pass / compliant / all assertions verified.
  - `1`: domain defect / violation / finding flagged.
  - `2`: unknown / environmental error / missing permission / indeterminate.
- Stream separation: structured machine-readable findings (TSV, JSON Lines)
  stream to `stdout`; operational telemetry, diagnostic logs, and error traces
  route to `stderr`.

### Verification Cadences (Three-Tier Cadence)
Verification is strictly tier-scoped. A claim is backed by the tier whose scope
matches the claim: sub-missions are backed by MID; epics and releases are backed
by BOUNDARY.

- **INNER** (every TDD increment, changed crate only):
  ```sh
  CARGO_TERM_PROGRESS_WHEN=never cargo test -p <crate> --message-format=short
  CARGO_TERM_PROGRESS_WHEN=never cargo clippy -p <crate> --all-targets --message-format=short -- -D warnings
  ```
  `--all-targets` is mandatory on clippy to catch test/bench/example lints.
  `--workspace` and `--all-features` are forbidden at this tier.

- **MID** (once at sub-mission completion, changed crates plus reverse dependents):
  ```sh
  cargo test --all-targets --locked
  cargo clippy --all-targets --locked -- -D warnings
  cargo fmt --all -- --check
  ```
  `--workspace` is forbidden at this tier; verify stays scoped to affected crates.

- **BOUNDARY** (once per epic, full workspace):
  ```sh
  cargo build --workspace --all-features --locked
  timeout 900 cargo test --workspace --all-features --locked --no-fail-fast
  cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
  cargo fmt --all -- --check
  sh scripts/verify.sh
  ```
  - `timeout 900` is mandatory on the test line. Exit 124 is `Outcome::Surprise`,
    NEVER a test failure. Investigate the stall; do not fold it into a failure count.
  - `--no-fail-fast` is mandatory on the test line to ensure full blast-radius
    visibility in a single pass.

### Rust Policy and Toolchain
- Toolchain: pinned channel **1.98.0** (`rust-toolchain.toml`), MSRV **1.98**,
  edition **2024**, resolver **3**.
- Members inherit root dependencies and `[lints] workspace = true`. The
  workspace manifest defines `pedantic = warn` and the centralized clippy warning roster,
  enforced with `-D warnings`.
- Format: stable-default rustfmt. No custom `rustfmt.toml`.
- Crate roots forbid unsafe code (`#![forbid(unsafe_code)]`).

### Supply Chain Gates
`cargo deny check` and `cargo audit` are supply-chain gates; run
before publishing or bumping dependencies.

### Rustdoc Budget Gate
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

### TigerStyle Construction-Path Inventory
Invariant-bearing domain types must enforce "illegal states unrepresentable"
by design. For each changed constrained type, review all construction routes:
1. Public fields / struct literals (reject if fields allow inconsistent mutation).
2. Constructors & builders (`new()`, `builder()`).
3. `Default::default()` (must yield a valid domain state or be omitted).
4. Conversions (`From`, `TryFrom`).
5. Serde deserialization (custom validation if raw wire data could bypass invariants).
6. Mutation routes (setters, `DerefMut`).

Independent booleans remain valid booleans; genuine optionality remains `Option`.
Do not invent artificial domain restrictions where none exist.

### Closed Error Enum Policy (C4.5/C4.6)
Public error enums MUST NOT carry `#[non_exhaustive]`. Variant sets are complete
within a major semver line, making unhandled error states unrepresentable at
compile time. Enforced mechanically via `non-exhaustive-check`.

### Beads Configuration & Issue Tracking
- Issue prefix: `tw` (e.g. `tw-1`, `tw-a3f2dd`).
- Database: repo-local store at `.beads/embeddeddolt`.
- Pinned discovery: use `bd -C <repo-root>` to target this workspace directly.
- Preserve `.beads` scaffold (`config.yaml`, `metadata.json`, `hooks/`); database
  files and runtime exports remain untracked per `.gitignore`.
- Autonomous commits follow `~/.config/opencode/AGENTS.md § Commits — agent-driven by default`.
