# Compatibility policy

## Minimum supported Rust version (MSRV)

- **MSRV: 1.88.0.** This is the lowest toolchain on which the workspace
  (including dependencies at their locked versions) compiles and passes the
  full test suite. It was verified locally with
  `cargo +1.88.0 test --workspace --all-targets`,
  `cargo +1.88.0 test -p ratahop-ratatui --no-default-features`, and
  `cargo +1.88.0 clippy --workspace --all-targets`. The workspace declares
  `rust-version = "1.88"` in `Cargo.toml`.
- The floor is set by dependencies, not by RataHop's own code: Ratatui 0.30.2
  and its dependency tree (e.g., `darling 0.24.x`, `instability 0.3.14`)
  require rustc 1.88.0. Rust 1.85.0 (the Edition 2024 floor) is rejected by
  the dependency tree.
- Current development and testing use Rust 1.99.0 on Linux x86_64. The MSRV
  is intended to be enforced by CI (msrv job in `.github/workflows/ci.yml`);
  the CI workflow has been configured but has so far run only locally on
  Linux.
- MSRV bumps will be recorded in the changelog and avoided within a 0.x
  release series unless required by a dependency.

## Supported Ratatui and event-crate versions

- `ratahop-ratatui` targets **Ratatui 0.30** and **Crossterm 0.29**.
  Documented and tested against Ratatui 0.30.2 and Crossterm 0.29.
- Version coupling is isolated in the adapter. The core `ratahop` crate has
  no dependencies and supports any host that can supply `Rect`, `Target`,
  and normalized `Input` values.
- Crossterm conversion is optional (`default-features = false`); hosts using
  another event backend supply normalized `Input` values and drive
  `next_page` / `previous_page` themselves.
- Semver-compatible updates within the 0.30 (Ratatui) and 0.29 (Crossterm)
  lines are expected to remain compatible; breaking upgrades will be handled
  in adapter major/minor releases with a changelog entry.
