#!/usr/bin/env python3
"""Draws the project logo from hand-placed shapes. Uses no source image."""
import math
import sys

import numpy as np
from PIL import Image, ImageChops, ImageDraw, ImageFilter

SIZE, SCALE = 640, 4
LIME = (122, 204, 18)
LASER = (120, 255, 40)
INK = (9, 13, 10)


def pts(values):
    return [(x * SCALE, y * SCALE) for x, y in values]


def curve(anchors, steps=18):
    """Closed Catmull-Rom curve through the anchors."""
    out, n = [], len(anchors)
    for i in range(n):
        p0, p1, p2, p3 = (anchors[(i + k - 1) % n] for k in range(4))
        for s in range(steps):
            t = s / steps
            out.append(tuple(
                0.5 * (2 * p1[a] + (p2[a] - p0[a]) * t
                       + (2 * p0[a] - 5 * p1[a] + 4 * p2[a] - p3[a]) * t * t
                       + (3 * p1[a] - p0[a] - 3 * p2[a] + p3[a]) * t ** 3)
                for a in (0, 1)))
    return out


def figure():
    im = Image.new("RGB", (SIZE * SCALE, SIZE * SCALE), (0, 0, 0))
    mask = Image.new("L", im.size, 0)
    d, m = ImageDraw.Draw(im), ImageDraw.Draw(mask)

    def shape(values, color, smooth=False):
        poly = pts(curve(values) if smooth else values)
        d.polygon(poly, fill=color)
        m.polygon(poly, fill=255)

    def line(values, color, width):
        d.line(pts(values), fill=color, width=width * SCALE, joint="curve")

    # Jacket, tee and neck.
    shape([(18, 640), (40, 560), (92, 506), (196, 466), (262, 440), (378, 440),
           (446, 466), (548, 506), (600, 560), (622, 640)], (40, 44, 42))
    shape([(258, 446), (382, 446), (362, 640), (278, 640)], (17, 20, 18))
    shape([(268, 372), (372, 372), (380, 462), (322, 512), (260, 462)], (150, 112, 86))
    shape([(262, 452), (322, 512), (380, 452), (372, 492), (322, 536), (268, 492)], (24, 27, 25))
    # Collar and lapels.
    shape([(250, 424), (196, 462), (214, 540), (276, 590), (300, 524)], (74, 79, 76))
    shape([(390, 424), (444, 462), (426, 540), (364, 590), (340, 524)], (66, 71, 68))
    line([(250, 428), (206, 464), (222, 534), (274, 580)], (132, 138, 134), 3)
    line([(390, 428), (434, 464), (418, 534), (366, 580)], (118, 124, 120), 3)
    # Leather creases and shoulder light.
    for crease in ([(70, 590), (118, 540), (178, 520)], [(112, 640), (150, 582), (204, 566)],
                   [(572, 590), (524, 540), (462, 520)], [(530, 640), (492, 584), (438, 566)]):
        line(crease, (104, 110, 106), 4)
    line([(94, 520), (150, 494), (204, 480)], (150, 156, 152), 5)
    line([(546, 520), (492, 494), (438, 480)], (126, 132, 128), 5)
    line([(310, 536), (306, 640)], (120, 124, 120), 3)
    line([(332, 536), (336, 640)], (120, 124, 120), 3)

    # Ears and head.
    shape([(226, 248), (206, 268), (212, 318), (236, 336), (246, 290)], (196, 150, 118), True)
    shape([(414, 248), (434, 268), (428, 318), (404, 336), (394, 290)], (176, 132, 102), True)
    shape([(232, 190), (320, 150), (408, 190), (420, 280), (398, 362), (352, 414),
           (320, 424), (288, 414), (242, 362), (220, 280)], (224, 176, 140), True)
    # Cheek shade, nose and a level, unbothered mouth.
    shape([(372, 300), (414, 270), (398, 362), (352, 412), (330, 420), (372, 360)], (182, 136, 106))
    line([(322, 292), (312, 344), (334, 350)], (140, 98, 74), 4)
    line([(288, 382), (322, 378), (356, 382)], (92, 58, 46), 6)

    # Swept grey hair, then strands.
    shape([(214, 276), (200, 206), (226, 138), (292, 96), (372, 92), (438, 126), (452, 196),
           (428, 268), (412, 214), (372, 184), (318, 196), (264, 190), (238, 226)],
          (170, 174, 176), True)
    for strand, tone in (
        ([(226, 214), (250, 150), (312, 116), (390, 114)], (232, 234, 236)),
        ([(240, 206), (276, 158), (336, 138), (416, 150)], (120, 124, 128)),
        ([(262, 182), (310, 158), (372, 158), (430, 190)], (226, 228, 230)),
        ([(214, 244), (222, 186), (262, 136), (324, 108)], (118, 122, 126)),
        ([(300, 186), (348, 174), (398, 182), (428, 226)], (104, 108, 112)),
    ):
        line(strand, tone, 6)

    # Heavy brows and dark glasses.
    line([(244, 244), (276, 234), (306, 242)], (58, 56, 54), 8)
    line([(334, 242), (364, 234), (396, 244)], (58, 56, 54), 8)
    for box in ((232, 252, 310, 312), (330, 252, 408, 312)):
        d.rounded_rectangle(pts([box[:2], box[2:]]), radius=16 * SCALE, fill=(14, 22, 16),
                            outline=(4, 5, 4), width=5 * SCALE)
    line([(308, 268), (332, 268)], (4, 5, 4), 6)
    line([(232, 268), (214, 262)], (4, 5, 4), 5)
    line([(408, 268), (426, 262)], (4, 5, 4), 5)
    return im, mask


def halftone(im, mask, pitch=6.4):
    small = im.resize((SIZE, SIZE), Image.LANCZOS)
    guide = np.asarray(mask.resize((SIZE, SIZE), Image.BILINEAR))
    out = Image.new("RGB", im.size, INK)
    d = ImageDraw.Draw(out)
    row_h = pitch * math.sqrt(3) / 2
    for row in range(int(SIZE / row_h) + 2):
        y = row * row_h
        for col in range(int(SIZE / pitch) + 2):
            x = col * pitch + (pitch / 2 if row % 2 else 0)
            ix, iy = min(int(x), SIZE - 1), min(int(y), SIZE - 1)
            if guide[iy, ix] < 128:
                continue
            color = small.getpixel((ix, iy))
            lum = (0.3 * color[0] + 0.59 * color[1] + 0.11 * color[2]) / 255
            light = 1.08 - 0.3 * (x / SIZE) - 0.1 * (y / SIZE)
            shaded = tuple(min(255, int(c * light)) for c in color)
            r = pitch * (0.17 + 0.33 * lum ** 0.6) * SCALE
            cx, cy = x * SCALE, y * SCALE
            d.ellipse((cx - r, cy - r, cx + r, cy + r), fill=shaded)
    return out


def lasers(size):
    glow = Image.new("RGB", size, (0, 0, 0))
    d = ImageDraw.Draw(glow)
    for cx, cy in ((271, 282), (369, 282)):
        for radius, level in ((34, 0.35), (21, 0.7), (11, 1.0)):
            c = tuple(int(v * level) for v in LASER)
            d.ellipse(pts([(cx - radius, cy - radius), (cx + radius, cy + radius)]), fill=c)
        d.polygon(pts([(cx - 92, cy), (cx, cy - 4), (cx + 92, cy), (cx, cy + 4)]), fill=LASER)
        d.polygon(pts([(cx, cy - 46), (cx + 3, cy), (cx, cy + 46), (cx - 3, cy)]), fill=LASER)
    wide = glow.filter(ImageFilter.GaussianBlur(11 * SCALE))
    tight = glow.filter(ImageFilter.GaussianBlur(2 * SCALE))
    core = Image.new("RGB", size, (0, 0, 0))
    c = ImageDraw.Draw(core)
    for cx, cy in ((271, 282), (369, 282)):
        c.ellipse(pts([(cx - 6, cy - 6), (cx + 6, cy + 6)]), fill=(240, 255, 225))
    return ImageChops.add(ImageChops.add(wide, tight), core.filter(ImageFilter.GaussianBlur(SCALE)))


def main(path):
    im, mask = figure()
    dotted = halftone(im, mask)
    tile = Image.new("RGB", im.size, LIME)
    sticker = mask.filter(ImageFilter.MaxFilter(9)).resize((SIZE, SIZE), Image.BILINEAR)
    sticker = sticker.filter(ImageFilter.MaxFilter(15)).resize(im.size, Image.BICUBIC)
    sticker = sticker.point(lambda v: 255 if v > 127 else 0)
    tile.paste((255, 255, 255), mask=sticker)
    tile.paste(dotted, mask=mask)
    final = ImageChops.add(tile, lasers(im.size))
    final.resize((SIZE, SIZE), Image.LANCZOS).save(path, optimize=True)


if __name__ == "__main__":
    main(sys.argv[1] if len(sys.argv) > 1 else "docs/assets/logo-illustrated.png")
