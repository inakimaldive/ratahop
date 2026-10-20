# Adapter ecosystem plan

RataHop's core (`ratahop`) is framework-independent: hosts supply generic
`Rect` coordinates, `Target` IDs, and normalized `Input` values. Only the
adapter layer couples to a specific stack. This plan sequences adapter work
for the rest of the Rust TUI ecosystem, following the same integration-driven
approach that produced the Ratatui adapter: build an adapter when a real
application needs it, validate the integration contract in that application,
and only then extract shared abstractions.

## Landscape

| Stack | Status | Assessment |
| --- | --- | --- |
| Ratatui | Adapter exists (`ratahop-ratatui`) | Dominant current stack |
| Crossterm only (manual drawing, no widget crate) | Planned first | Common in small tools; key conversion already framework-free |
| Cursive | Planned second | Second-most-used framework; owns its own event loop and backends |
| Termion | Planned third | Niche; usually paired with legacy tui-rs |
| tui-rs (legacy) | Not planned | Unmaintained; covered by migrating to Ratatui |
| ncurses / pancurses | On demand | Rare in new Rust applications; evaluate only with a real integration |
| Manual ANSI / custom backends | Already supported | Core works directly; document the pattern instead of shipping a crate |

## Milestone 4: Second adapter (Crossterm-only hosts)

Applications that draw with Crossterm but skip Ratatui cannot use a widget
overlay. The core's normalized `Input` already works for them; what is
missing is badge painting onto a manually managed screen.

Scope:

- A small `ratahop-crossterm` adapter (or a documented core-only pattern,
  if painting glue proves trivial) that:
  - converts Crossterm events to normalized `Input` (the existing
    `handle_key` logic lives behind a feature flag and depends on Crossterm
    only — move it to the new home rather than duplicating it);
  - paints badges through a host-supplied cell callback
    (`fn(x, y, ch, style)`), so the adapter never owns a screen buffer;
  - reuses the badge placement and crowded-picker decision logic (see
    extraction note below).
- A runnable example on a hand-drawn screen (no widget crate).
- Tests for event mapping and a buffer-diff style paint test.

Extraction note: badge layout, collision fallback, and the picker currently
live in `ratahop-ratatui`. Extract them only after this second adapter exists
and the shared shape is known from two implementations, per the contributing
guidance against speculative abstractions. The likely home is a
framework-neutral `ratahop-overlay` crate parameterized by a cell setter and
a style type.

Exit: an existing hand-drawn Crossterm application gains hint navigation
without architectural changes; the Ratatui adapter shrinks or stays put
without duplicating logic.

## Milestone 5: Cursive adapter

Cursive inverts the Ratatui flow: it owns the event loop and dispatches
through callbacks, and drawing happens inside view callbacks on its own
backend abstraction.

Scope:

- A `ratahop-cursive` crate providing a wrapper view:
  - `draw` renders the overlay after the wrapped view's content, using the
    Cursive backend's cell primitives through the shared painting shape from
    milestone 4;
  - `on_event` routes active-session input before the wrapped view's
    callbacks, with the host's activation shortcut mapped from
    `Event::Char`/key variants;
  - target registration bridges Cursive view identification to host-declared
    IDs (Cursive has no public per-widget hit rectangles, so the host
    declares them, as everywhere else).
- One real integration in an existing Cursive application.
- Manual PTY QA (tmux captures) following the Wire pattern; Cursive's test
  backends (puppet) if practical.

Exit: the integration contract (register, start, route, render, revalidate)
is demonstrable in a Cursive app; event-loop ownership questions are settled
and recorded in the guide.

## Milestone 6: Termion and beyond

- `ratahop-termion`: key-event mapping only; painting follows the
  milestone-4 cell-callback shape. Build only with a real integration in
  hand.
- tui-rs: decline and document a migration path to Ratatui instead.
- ncurses/pancurses: evaluate on demand; `getch` mapping is easy but the
  audience is small.
- Non-Rust TUIs remain out of scope.

## Standing constraints (all adapters)

- Core stays dependency-free; each adapter isolates its stack and version
  line, with its own semver and changelog entries.
- Every adapter is validated by at least one real application before
  publication, and `publish = false` is removed only as part of release
  preparation.
- Normalized `Input` semantics never drift between adapters: releases
  ignored, repeats consumed without reselection, modified characters not
  interpreted as hint letters.
- Keep host-owned responsibilities (terminal setup, activation shortcut,
  focus, dispatch, async scheduling) out of every adapter.
