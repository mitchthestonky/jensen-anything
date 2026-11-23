#!/usr/bin/env python3
"""Renders docs/assets/demo.gif from the real output of the binary.

Usage: make_demo.py <path-to-jensen-anything> <output.gif>
The GIF contains terminal text only, so replacing the logo never affects it.
"""
import subprocess
import sys
import tempfile
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

W, LINE, PAD, TOP = 820, 19, 22, 54
BG, FG, DIM, GREEN = (8, 14, 10), (226, 240, 222), (128, 158, 124), (122, 224, 40)
FONT = ImageFont.truetype("/System/Library/Fonts/Menlo.ttc", 14)
BOLD = ImageFont.truetype("/System/Library/Fonts/Menlo.ttc", 14, index=1)


def capture(binary):
    company = Path("fixtures/soggy-toast-logistics")
    with tempfile.TemporaryDirectory() as tmp:
        out = subprocess.run(
            [binary, "run", str(company / "pitch.md"), "-f", str(company / "financials.csv"),
             "-o", f"{tmp}/rescue"],
            check=True, capture_output=True, text=True).stdout
    return out.replace(tmp + "/", "").splitlines()


def frame(lines, height):
    im = Image.new("RGB", (W, height), BG)
    d = ImageDraw.Draw(im)
    d.rectangle((0, 0, W, 32), fill=GREEN)
    d.text((W / 2, 16), "SATIRE, NOT FINANCIAL ADVICE", font=BOLD, fill=BG, anchor="mm")
    for i, text in enumerate(lines):
        color = GREEN if text.startswith("$") or " pass " in text else FG
        if text.startswith(("record", "pivot", "lift")):
            color = DIM
        d.text((PAD, TOP + i * LINE), text, font=BOLD if text.startswith("$") else FONT, fill=color)
    return im.quantize(colors=32, dither=Image.Dither.NONE)


def main(binary, target):
    command = "$ jensen-anything run pitch.md -f financials.csv -o rescue"
    output = capture(binary)
    height = TOP + (len(output) + 2) * LINE + 16
    frames, times = [], []
    for n in range(4, len(command) + 1, 5):
        frames.append(frame([command[:n]], height))
        times.append(45)
    frames.append(frame([command], height))
    times.append(450)
    for n in range(1, len(output) + 1):
        frames.append(frame([command] + output[:n], height))
        times.append(120)
    times[-1] = 5000
    frames[0].save(target, save_all=True, append_images=frames[1:], duration=times, loop=0,
                   optimize=True)


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
