# RataHop integration guide

Add keyboard hint navigation to an existing Ratatui screen in about twenty lines
without replacing its widgets or event loop. This guide walks through the
contract used by the working integrations: the [showcase
example](../crates/ratahop-ratatui/examples/showcase.rs) and Wire
([notes](../integrations/wire-feed/README.md)).

## 1. Add the dependency

```toml
[dependencies]
ratahop-ratatui = { path = "../ratahop/crates/ratahop-ratatui" }
```

The adapter targets Ratatui 0.30 and Crossterm 0.29. The optional Crossterm
event conversion can be disabled (`default-features = false`) if your
application uses another event backend; use normalized `Input` and
`next_page` / `previous_page` in that case. The core `ratahop` crate has no
dependencies and is re-exported through the adapter.

## 2. Keep a `Hop` in your state

Your control ID type must be `Clone + Eq`. Application enums work well.

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
enum Action { Save, Close }

struct App {
    hop: ratahop_ratatui::Hop<Action>,
    // ...
}
```

## 3. Register targets during rendering

After your normal widgets render — and before the frame ends — register every
enabled, visible control with the rectangle a pointer would hit:

```rust
app.hop.update(
    vec![
        Target::new(Action::Save, rect(save_area), "Save profile"),
        Target::new(Action::Close, rect(close_area), "Close dialog"),
    ],
    frame.area(), // viewport
    None,         // optional scope
);
frame.render_widget(&app.hop, frame.area()); // always render last
```

Notes:

- Reuse the rectangles your mouse handling already uses. Controls that are
  offscreen or zero-sized are clipped out automatically.
- Duplicate IDs in one `update` are an error; deduplicate repeated actions
  yourself, or give each placement a distinct ID.
- Lists should register visible rows only.
- The same `update` call also synchronizes the active session: if geometry,
  membership, order, titles, anchors, scope, or the viewport changed since the
  last frame, the session is cancelled rather than silently remapping typed
  labels. Register targets on *every* frame.

## 4. Route input before ordinary shortcuts

In your event handler, let an active session consume keys first:

```rust
let outcome = app.hop.handle_key(key);
if let Outcome::Selected(action) = &outcome {
    app.dispatch(action.clone());
}
if outcome.consumed() {
    return; // hint characters must not trigger normal shortcuts
}
```

Rules encoded by the adapter:

- Key releases are ignored; key repeats are consumed but can never select a
  second control.
- Modified characters (Ctrl/Alt) are not interpreted as hint letters.
- Invalid characters keep the prefix and show visible feedback; the event is
  consumed. Backspace edits the prefix, Escape cancels.

## 5. Choose an activation shortcut

The host owns activation. Call `start()` from the latest completed layout
snapshot:

```rust
if key.code == KeyCode::Char('f') && key.modifiers.is_empty() && !app.editing {
    if let Err(error) = app.hop.start() {
        app.status = format!("Hints: {error}"); // NoTargets or DuplicateId
    }
    return;
}
```

If your screen has text entry, pick a modified variant (for example `Alt+f`)
that starts hints without capturing ordinary typing. See Wire: `f` outside
dialogs, `Alt+f` inside them.

## 6. Cancel at the right moments

Cancel explicitly when application state changes in ways rendering does not
cover:

```rust
app.hop.cancel(); // after dispatching an action, applying data updates
```

and in the event loop:

- `Event::Resize(..)` → `cancel()`
- mouse input → `cancel()` before normal pointer handling

Content refreshes that would alter target membership can either be deferred
until the session ends (Wire's approach for live feed updates) or allowed to
proceed; the next `update` call cancels the session either way.

## 7. Modals and scopes

Modal hosts should provide only modal targets. Either filter your target list,
or tag targets with a scope and pass the active one:

```rust
let scope = if app.modal.is_some() { Some(1) } else { None };
// Target::new(...).scope(1) for dialog controls
app.hop.update(targets, frame.area(), scope);
```

## 8. Revalidate before dispatch

IDs are host-owned. If your data can change between session start and
selection, verify the selected ID still refers to the same content before
dispatching. Including a generation in the ID (for example
`("link", page_gen, index)`) makes stale selections fail naturally.

## Crowded layouts

When labels cannot fit inside their controls, the adapter automatically
switches to a full-viewport paged picker showing target titles. Page with
Page Up / Page Down (or `handle` with normalized `Input::Other` plus explicit
`next_page` / `previous_page`). Selection works across all pages. Badges
overwrite wide Unicode glyphs safely; ASCII alphabets guarantee one-cell
labels.

## Theming

Set `hop.theme` (badge, typed, invalid, picker styles) for monochrome
terminals or to match your application palette. Custom
`Alphabet::new("hjklasdf")` layouts are supported; alphabets must be at least
two distinct printable ASCII characters.

## Checklist

- [ ] Targets registered on every frame, after layout, before input routing
- [ ] Overlay rendered last
- [ ] Active-session keys consumed before normal shortcuts
- [ ] Activation shortcut chosen; modified variant for text entry
- [ ] Cancel on dispatch, data updates, resize, and mouse input
- [ ] Modal scopes filter targets
- [ ] Selected IDs revalidated when data can change

## Troubleshooting

| Symptom | Cause |
| --- | --- |
| Hints never appear | `update` not called each frame, or all targets clipped out |
| Session drops immediately | Targets or viewport change every frame (animations, clocks) — exclude volatile values from targets or titles |
| Typing `f` inserts into a text field | `f` is not routed before the text-entry handler; use a modified activation or check `hop.is_active()` first |
| `DuplicateId` error | Two placements share an ID; deduplicate or split IDs |
| Badges cover content | That is the overlay contract; render order controls it |
