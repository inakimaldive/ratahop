# RataHop development plan

## Implementation status (2026-10-20)

Implemented the core and Ratatui adapter, with generated label-invariant tests,
state-machine tests, rendering/event tests, a headless example, and a showcase
covering forms, scrolling rows, split panes, and modal text entry. Integrated and
terminal-tested Wire at `../tui-news-feed`. See [validation](VALIDATION.md).

Community-beta preparation completed 2026-10-20: minimum supported Rust
version evaluated and pinned at 1.88.0 ([compatibility policy](COMPATIBILITY.md)),
crate names verified as unclaimed on crates.io, integration guide
([GUIDE](GUIDE.md)), recorded animated demonstration ([demo](demo/demo.gif)),
changelog, and an MSRV CI job added.

Remaining community-beta work: feedback from independently maintained projects,
a second external integration, and release preparation. CI is configured for
Linux, Windows, and macOS but has only been run locally on Linux. The optional
hash-based duplicate-ID validation strategy stays deferred until integration
feedback shows larger registries need it; any such strategy would require a
stronger trait bound on target IDs (for example `Hash`), so it should not be
adopted lightly.

## Purpose

A small, community-focused Rust library for Vimium-style hint navigation inside
TUIs. Adoption goal: integrate an existing screen in roughly 20 lines without
replacing its widgets or event loop. This is a goal to validate, not a current API.

## Architecture

Start with a framework-independent `ratahop` core and a separate
`ratahop-ratatui` adapter. Add other adapters in response to real integrations.
Keep framework and terminal event dependencies out of the core.

The core accepts stable generic target IDs, terminal-cell rectangles, scopes,
and normalized input. It returns selection results and explicit event
consumption. Return IDs rather than storing application callbacks.

The adapter converts geometry and input, and supplies an overlay widget and
themes. The host owns terminal setup, shortcut activation, actions, and focus.

## Integration contract

1. Register enabled, visible targets during layout/rendering. Lists register
   visible rows only. The application supplies modal scope and occlusion rules.
2. Start a session from the latest completed layout snapshot.
3. Route input to the session before ordinary application shortcuts. Return
   whether each event was consumed; ignore key releases and define repeat policy.
4. Render labels after the normal widgets using current frame geometry.
5. Revalidate the selected ID before dispatching an application-owned action.

Avoid assumptions about Elm, component, synchronous, or async architectures.

## Initial behavior

- Configurable alphabet with a home-row default; validate duplicates and reject
  symbols the initial renderer cannot safely display in one terminal cell.
- Unique prefix-free labels. `a` and `as` cannot select different targets in the
  same session. Prefer short labels while preserving this invariant.
- Deterministic ordering and stable labels throughout a session.
- Prefix filtering, Backspace, Escape, and immediate completion.
- Invalid typed characters are consumed without triggering application actions;
  keep the existing prefix and provide visible feedback.
- Explicit empty-target and invalid-configuration outcomes.
- Host chooses focus versus activation. Activation shortcut is host-owned.
- Modal scopes restrict which controls are selectable.
- Freeze target membership and labels during a session. Cancel on layout or
  membership changes in the initial release; never silently remap a typed label.

## Terminal rendering

- Clip to the viewport, support explicit badge anchors, and distinguish typed
  and remaining characters without relying on color alone.
- Account for wide glyphs in surrounding content; prevent partial wide-cell
  overwrites and define supported alphabets explicitly.
- Place labels deterministically. Define collision behavior and an overflow
  picker when labels cannot fit. Never silently hide selectable targets.
- Cover dense tables, scrolling lists, split panes, modals, small viewports,
  nonzero viewport origins, and terminal resizing.

## Milestones

### 1. Core prototype

Design target, geometry, session, input, and outcome types. Implement label
allocation and the session state machine. Add an executable headless example.
Property-test uniqueness, prefix freedom, determinism, and selection round trips.
Test cancellation, invalid input, empty targets, and alphabet boundary cases.

Exit: API examples compile and invariants hold across generated target sets.

### 2. Ratatui integration

Add the adapter, overlay, event conversion, accessible themes, and runnable
examples: a simple form, a scrolling table, and a split view with a modal.
Test input consumption, stale IDs, resize cancellation, clipping, and collisions.
Use buffer snapshots for visual behavior and manually exercise examples.

Exit: an existing demo gains hint navigation without architectural changes.

### 3. Community beta

Validate the API in two independently structured TUIs. Publish a short guide,
animated demo, compatibility and minimum Rust version policies, release notes,
and contribution instructions. Benchmark allocation and rendering for 10, 100,
and 1,000 targets; publish measurements with environment details.

Exit: external integration feedback is addressed and names are checked before
publishing crates. Remove `publish = false` only as part of release preparation.

## Community decisions

Use MIT OR Apache-2.0. Keep the dependency footprint small. Establish the minimum
supported Rust version after the adapter dependencies are selected and test it
in CI. Document supported Ratatui and terminal event crate versions; isolate
version coupling in adapters. Introduce CI once substantive implementation lands.
Keep public errors actionable and examples copyable. Record API decisions and
maintain a changelog before the first release.

## Scope

The initial release handles hint selection across application-declared controls.
Full Vim editing, desktop automation, fuzzy search, and widget replacement remain
outside initial scope. Use Vimium and keyhop as references; implement the TUI
contract directly rather than depending on desktop automation infrastructure.
