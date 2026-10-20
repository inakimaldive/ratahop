#!/usr/bin/env python3
"""Render captured terminal frames to an animated GIF.

Reads ANSI frame files from a directory (as produced by record.sh with
`tmux capture-pane -e`) and writes an animated GIF using Pillow. Colors
approximate RataHop's default theme.

Usage: render.py <frames-dir> <output.gif>
"""
import sys
from pathlib import Path

import pyte
from PIL import Image, ImageDraw, ImageFont

FONT = ImageFont.truetype(
    "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf", 14
)
FONT_BOLD = ImageFont.truetype(
    "/usr/share/fonts/truetype/dejavu/DejaVuSansMono-Bold.ttf", 14
)
CELL_W, CELL_H = 10, 20
BG = (15, 21, 31)
FG = (222, 231, 241)
# Base palette for named attributes; indexed colors resolve to hex through
# pyte and are used directly.
PALETTE = {
    "black": (15, 21, 31), "red": (255, 133, 144), "green": (120, 220, 160),
    "brown": (94, 218, 196), "yellow": (94, 218, 196), "blue": (133, 190, 255),
    "magenta": (200, 150, 235), "cyan": (94, 218, 196), "white": (222, 231, 241),
}


def color(value):
    if value in (None, "default"):
        return None
    if isinstance(value, (tuple, list)):
        return tuple(value[:3])
    if isinstance(value, str) and len(value) == 6 and all(
        c in "0123456789abcdef" for c in value.lower()
    ):
        v = value.lower()
        return (int(v[0:2], 16), int(v[2:4], 16), int(v[4:6], 16))
    return PALETTE.get(value)


def render(screen, width, height):
    image = Image.new("RGB", (width * CELL_W, height * CELL_H), BG)
    draw = ImageDraw.Draw(image)
    for y in range(height):
        for x in range(width):
            cell = screen.buffer[y][x]
            bg = color(cell.bg) or BG
            if bg != BG:
                draw.rectangle([x * CELL_W, y * CELL_H,
                                (x + 1) * CELL_W - 1, (y + 1) * CELL_H - 1], fill=bg)
            if cell.data != " ":
                fg = color(cell.fg) or FG
                bold = cell.bold
                draw.text((x * CELL_W + 1, y * CELL_H + 2), cell.data,
                          fill=fg, font=FONT_BOLD if bold else FONT)
    return image


def main():
    frames_dir, output = Path(sys.argv[1]), sys.argv[2]
    paths = sorted(frames_dir.glob("*.ans"))
    if not paths:
        sys.exit(f"no frames in {frames_dir}")
    screens, size = [], None
    size_file = frames_dir / "size.txt"
    if size_file.is_file():
        size = tuple(int(v) for v in size_file.read_text().split())
    for path in paths:
        screen = pyte.Screen(*(size or (max(len(line) for line in path.read_bytes()
                                         .decode("utf-8", "replace").splitlines()),
                                        len(path.read_bytes().decode("utf-8", "replace").splitlines()))))
        stream = pyte.Stream(screen)
        # capture-pane emits bare newlines; pyte keeps the cursor column
        # across them, so restore carriage returns first.
        stream.feed(path.read_text(errors="replace").replace("\n", "\r\n"))
        if size is None:
            size = (screen.columns, screen.lines)
        screens.append((path, screen))
    delays = []
    if (frames_dir / "delays.txt").is_file():
        delays = [int(v) for v in (frames_dir / "delays.txt").read_text().split()]
    frames = [render(screen, *size) for _, screen in screens]
    frames[-1].save(
        output, save_all=True, append_images=frames[1:],
        duration=[delays[i] if i < len(delays) else 800 for i in range(len(frames))],
        loop=0, optimize=True,
    )
    print(f"wrote {output} with {len(frames)} frames")


if __name__ == "__main__":
    main()
