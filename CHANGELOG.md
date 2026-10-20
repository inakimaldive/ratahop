# Changelog

All notable changes to RataHop are documented in this file. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project
adheres to Semantic Versioning.

## [Unreleased]

### Added

- `ratahop` core (0.1.0): `Target`, `Rect`, `Alphabet`, `Input`, `Outcome`,
  `Session`, and `HintError`. Deterministic prefix-free label allocation over
  a configurable ASCII alphabet; session state machine with explicit event
  consumption and cancellation on snapshot changes.
- `ratahop-ratatui` adapter (0.1.0): `Hop` stateful overlay widget with badge
  placement, anchors, collision fallback to a paged crowded-layout picker,
  wide-glyph-safe badge clearing, optional Crossterm key conversion
  (`handle_key`), and a customizable `Theme`.
- Examples: headless core demo (`ratahop`), interactive form/table/split
  showcase and a timing probe (`ratahop-ratatui`).
- Generated invariant tests (uniqueness, prefix freedom, determinism,
  round trips), adapter rendering/event tests, and documentation examples.
- Wire (tui-news-feed) integration with a recorded patch and integration notes.
- CI covering Linux, Windows, and macOS; MSRV (1.88.0) job.
- Integration guide, compatibility policy, validation record, and a recorded
  animated demonstration.

### Deferred

- Second external integration (community-beta milestone item); awaiting an
  independently structured TUI integration and external maintainer feedback.
