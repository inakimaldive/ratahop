# Validation — 2026-10-20

Tested locally on Linux x86_64 with Rust/Cargo 1.99.0, Ratatui 0.30.2, and
Crossterm 0.29. Windows and macOS CI jobs are configured but have not run here.

## Library

- Core: five tests, including generated invariant checks across four alphabets
  and target counts around label-length boundaries through 1,000 targets.
  Checks cover uniqueness, prefix freedom, determinism, selection round trips,
  invalid configuration, clipping, scopes, stale snapshots, and cancellation.
- Adapter: eight tests for badge positions, nonzero origins, wide glyphs,
  collision alternatives, crowded picker paging, offscreen selection, prefix
  styling, invalid feedback, small terminals, stale render geometry, resizing,
  and modified/repeated/released key events.
- **13 library tests and one documentation test passed.** The headless example
  also ran successfully.
- No-default-features adapter tests exercise the framework adapter without
  enabling its optional Crossterm event-conversion API.
- Workspace formatting, Clippy with warnings denied, and documentation checks.

The showcase was exercised in real tmux PTYs at 80×24 in form, table, and split
modes. Table hints selected row ID 0; a split-pane hint opened the text-entry
dialog. Captures: [form](qa/showcase-form.txt), [table](qa/showcase-table.txt),
[split](qa/showcase-split.txt).

## Wire integration

Ran the complete application's test suite: **29 tests passed** (four library,
six feed UI, and nineteen curator integration tests), plus Clippy with warnings
denied. Three feed UI tests were added for RataHop.

Real terminal checks, using bundled samples and isolated local state:

1. Started Wire at 150×54 and activated hints using `f`.
2. Typed the visible bookmark label; the story showed Saved and the isolated
   persisted JSON contained its URL.
3. Opened the composer; an ordinary `f` appeared in its text field.
4. Sent `Alt+f`; only close and publish controls received hints.
5. Selected the close hint and returned to the feed.
6. Started hints and resized to 90×30; the session cancelled.
7. Quit normally. No live curator service or external browser was used.

Captures: [feed hints](qa/wire-hints.txt), [dialog hints](qa/wire-dialog-hints.txt),
[resized feed](qa/wire-resized.txt).

Original app files are backed up outside this repository. The source patch and
integration instructions are in [integrations/wire-feed](../integrations/wire-feed/README.md).

## Timing probe

Command: `cargo run -p ratahop-ratatui --release --example measure`.
Intel Core i7-4510U @ 2.00 GHz, Linux x86_64. Arithmetic mean of 100 iterations;
these are local smoke measurements, not a statistical performance guarantee.
The render probe deliberately overlaps all targets, exercising the crowded picker
at 150×54 with a reused in-memory buffer; terminal I/O is excluded.

| Targets | Label allocation | Session start | Crowded render |
| --- | --- | --- | --- |
| 10 | 1.308 µs | 1.738 µs | 131.424 µs |
| 100 | 17.517 µs | 27.973 µs | 238.035 µs |
| 1,000 | 96.785 µs | 482.602 µs | 257.306 µs |

Duplicate-ID validation currently uses equality comparisons and has quadratic
worst-case cost. The core only requires `Clone + Eq`; larger-scale registries may
justify an optional hash-based strategy after integration feedback.

## Release limits

The crates are unpublished and marked `publish = false`. Only one external app
has been integrated. Lower Rust version compatibility, external maintainer
feedback, an animated demo, and registry name availability remain community-beta
work, as recorded in the development plan.

## Community-beta preparation addendum (2026-10-20, later)

- MSRV: the workspace compiles and passes `cargo test --workspace
  --all-targets`, the no-default-features adapter tests, and Clippy on
  Rust 1.93.0 and 1.88.0. Rust 1.85.0 is rejected by the dependency tree
  (`ratatui 0.30.2` and transitive dependencies require rustc 1.88.0).
  `rust-version = "1.88"` is now declared, with an MSRV job in CI.
- Registry names: `ratahop` and `ratahop-ratatui` returned 404 from the
  crates.io API, i.e., both names are currently unclaimed.
- Animated demonstration: recorded from the showcase with tmux and rendered
  to [demo/demo.gif](demo/demo.gif) via `docs/demo/record.sh` (tmux + python3
  with pyte and Pillow). Frames verified visually.
- Integration guide added at [GUIDE.md](GUIDE.md); changelog at
  `CHANGELOG.md`. The second external integration remains open.
