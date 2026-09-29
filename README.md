# tripwires

Canonical repository housing shared fleet structural tripwires and CI gates.

## Member crates

### `non-exhaustive-check`

CI hard-gate enforcing closed public error enums (C4.5/C4.6 closed enumeration policy, RST-0006:R1).

Public error enums must not carry `#[non_exhaustive]`. Variant sets are complete within a major line; adding or changing variants is a breaking change requiring a major version bump, making unhandled error states unrepresentable at compile time.

#### Installation

Install from this canonical repository via Cargo:

```sh
cargo install --git https://github.com/org/tripwires --locked non-exhaustive-check
```

#### Usage

Pass one or more source directories to scan:

```sh
non-exhaustive-check <source_directories...>
```

Examples:

```sh
# Scan all crate sources in a workspace
non-exhaustive-check crates/

# Scan multiple specific source directories
non-exhaustive-check crates/core/src crates/client/src
```

#### Heuristic (FLAG-IFF)

An enum is flagged as a violation iff:
```
is_error_type && has_non_exhaustive
```
- `has_non_exhaustive`: any attribute path is exactly `non_exhaustive`
- `is_error_type`: any `#[derive(..)]` entry's last path segment is `Error`

#### Output & Exit Codes

- **Exit 0**: clean, no violations found. Prints summary:
  ```
  OK: 1 library crates scanned, 166 pub enums, 0 violations (syntax-only predicate; enum count includes private declarations)
  ```
- **Exit 1**: violations found. Prints one tab-separated line per violation, followed by a summary:
  ```
  VIOLATION	crates/foo/src/error.rs:12	FooError	forbidden #[non_exhaustive] on error enum (C4.5/C4.6 closed enumeration policy)
  SUMMARY: 1 library crates scanned, 166 pub enums, 1 violations
  ```
- **Exit 101**: invalid invocation or unreadable/unparseable Rust sources.

## License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.
