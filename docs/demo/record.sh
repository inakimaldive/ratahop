#!/usr/bin/env bash
# Record the RataHop showcase into terminal frames and assemble an animated GIF.
#
# Requirements: tmux, cargo, python3 with pyte and Pillow.
# Usage: docs/demo/record.sh [output.gif]
set -euo pipefail
cd "$(dirname "$0")/../.."

OUTPUT="${1:-docs/demo/demo.gif}"
SESSION=ratahop-demo
FRAMES="${FRAMES:-$(mktemp -d)}"
mkdir -p "$FRAMES"
trap '{ tmux kill-session -t $SESSION 2>/dev/null || true; } && { [ -n "${KEEP_FRAMES:-}" ] || rm -rf "$FRAMES"; }' EXIT

cargo build -p ratahop-ratatui --release --example showcase
tmux new-session -d -x 100 -y 30 -s "$SESSION" \
    -e TERM=xterm-256color \
    "./target/release/examples/showcase form"
tmux set-window-option -t "$SESSION" window-size manual
tmux resize-window -t "$SESSION" -x 100 -y 30
sleep 1
echo "100 30" > "$FRAMES/size.txt"

# capture <delay-ms> <keys...>: send keys, pause, snapshot the pane.
FRAME=0
capture() {
    local delay="$1"; shift
    if [ "$#" -gt 0 ]; then
        tmux send-keys -t "$SESSION" -- "$@"
        sleep 0.7
    fi
    printf '%s\n' "$delay" >> "$FRAMES/delays.txt"
    tmux capture-pane -t "$SESSION" -p -e \
        > "$FRAMES/$(printf '%03d' "$FRAME").ans"
    FRAME=$((FRAME + 1))
}

capture 1200   # reader view, settled
capture 1000 f # badges appear: a s d
capture 900 s  # type "s": selects Save profile
capture 1000 f # badges again
capture 900 a  # type "a": opens the edit dialog
capture 1200 M-f # Alt+f: hints inside the dialog
capture 900 d  # close the dialog through its hint
tmux kill-session -t "$SESSION"

# Second pass: crowded table mode shows the paged picker.
tmux new-session -d -x 100 -y 30 -s "$SESSION" \
    -e TERM=xterm-256color \
    "./target/release/examples/showcase table"
tmux set-window-option -t "$SESSION" window-size manual
tmux resize-window -t "$SESSION" -x 100 -y 30
sleep 1
capture 1200     # table view, visible rows only
capture 1000 f   # badges on every visible row
capture 1000 a   # type "a": prefix filters the matching badges
capture 1100 k   # complete "ak": selects Item 015
tmux kill-session -t "$SESSION"

python3 docs/demo/render.py "$FRAMES" "$OUTPUT"
