# Wire integration

The local app at `/home/ignat/Documents/tui-news-feed` has the integration applied.
The app uses Ratatui 0.30.2 and Crossterm 0.29; its recorded mouse-hit rectangles
and actions also drive RataHop. Repeated actions are deduplicated in registration
order. Dialogs already clear background hit targets, providing natural scope.

- `f`: start hints outside text entry and dialogs.
- `Alt+f`: start hints inside dialogs without capturing ordinary text.
- `Esc`: exit hints before normal dialog shortcuts run.
- Pointer input and resizing cancel hints.
- Live feed updates wait for the session to end, keeping story indices stable.
- `apply_feed` and action dispatch explicitly cancel sessions.

Run:

```sh
cd /home/ignat/Documents/tui-news-feed
cargo run -- --samples
```

Use sample mode for testing without remote services. Integration tests use
isolated temporary persistence. Live terminal QA used a separate `XDG_DATA_HOME`.
No existing user bookmark data was changed by QA.

`ratahop.patch` records the source changes relative to the app before integration;
Cargo updates the lockfile when applying it to a matching checkout. The local
path dependency assumes the two repositories are siblings. The patch is already
applied here; do not apply it again. Original files are backed up outside this
repository at `/home/ignat/.local/state/ratahop/wire-feed-before-integration-20261020`.
