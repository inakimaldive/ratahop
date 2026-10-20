# Contributing to RataHop

Start with [the development plan](docs/PLAN.md). The core and Ratatui adapter are implemented. Current priorities are integration
feedback, layout edge cases, and API polish. Keep changes small and discuss public API changes before
building framework-specific abstractions.

Run from the repository root:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
cargo test --workspace --doc
cargo test -p ratahop-ratatui --no-default-features
cargo doc --workspace --no-deps
```

Add behavioral tests with implementation changes. For the allocator, test
invariants rather than duplicating its algorithm. For adapters, cover real
rendering and input routing failures. New runtime dependencies should have a
clear purpose and stay within the appropriate crate.

Contributions are accepted under the project's MIT OR Apache-2.0 license.
