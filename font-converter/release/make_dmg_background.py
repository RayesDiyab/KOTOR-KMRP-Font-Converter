#!/usr/bin/env python3
"""The background of the macOS DMG's window, in the converter's own style.

The maintainer, 2026-11-13: "create a custom font converter background for the dmg ... in the
font converter art style". It is drawn the way make_press_kit.py draws the press kit: the
window's navy with faint baselines and a glow behind the title, the title in KMRP Original HD
Menu with the version in gold, the orange, blue and green rule under it, and rounded tiles
like the window's for where the app and the Applications link stand.

The window is 720 x 500 points. build-macos.sh puts the icons (88 points) at these places,
so the two must change together:

| Icon | Centre (points) |
| --- | --- |
| The app | 200, 190 |
| Applications | 520, 190 |
| README.txt, LICENSE.txt, OFL.txt, THIRD-PARTY-NOTICES.txt | 120, 280, 440, 600 at 400 |

Finder writes the icons' names in black on a background picture, in Dark Mode too (seen on
macOS 27, 2026-11-13), so each name stands on a pale strip, as a font's name stands on the
strip at the foot of the window's tiles. A name's line is centred 59.5 points below its
icon's centre, the second line 16 points under that (measured from a screenshot).

It writes dmg-background.png (720 x 500) and dmg-background@2x.png (1440 x 1000) beside
itself; build-macos.sh joins them into one TIFF with tiffutil so Retina screens get the
sharp one.

    python font-converter/release/make_dmg_background.py
"""
import re
from pathlib import Path

from PIL import Image, ImageDraw, ImageFilter, ImageFont

HERE = Path(__file__).resolve().parent
CONVERTER = HERE.parent
FONTS = CONVERTER / "fonts"

W, H = 720, 500
APP = (200, 190)
APPLICATIONS = (520, 190)
EXTRAS_Y = 400
LABEL = 59.5   # a name's first line, below its icon's centre
LINE = 16

# The window's colours (src/theme.rs, dark mode), as in make_press_kit.py.
WINDOW = (10, 18, 32)
DEEP = (6, 11, 20)
TILE = (15, 26, 44)
TILE_EDGE = (62, 88, 126)
ACCENT = (42, 198, 239)
GOLD = (226, 195, 92)
PICKED = (255, 154, 60)
DONE = (126, 224, 171)
TEXT = (236, 243, 248)
MUTED = (177, 195, 208)
SOFT = (158, 177, 194)
DASHED = (74, 108, 156)
# Under Finder's black names: a light grey, partly see-through (the maintainer, 2026-11-13:
# "make the white boxes behind names grey and a bit transparent").
NAME_STRIP = (176, 186, 198, 200)


def version():
    text = (CONVERTER / "Cargo.toml").read_text(encoding="utf-8")
    full = re.search(r'^version = "(\d+)\.(\d+)\.(\d+)"', text, re.M)
    return f"{full.group(1)}.{full.group(2)}"   # the press kit's "1.0"


def face(name, px):
    return ImageFont.truetype(str(FONTS / f"KMRP-OriginalHD-{name}.ttf"), round(px))


def ground(s):
    """The window's navy, deeper at the foot, a glow at the top left, baselines every 27 points."""
    w, h = W * s, H * s
    image = Image.new("RGB", (w, h), WINDOW)
    draw = ImageDraw.Draw(image)
    for y in range(h):
        t = y / (h - 1)
        draw.line([(0, y), (w, y)], fill=tuple(round(a + (b - a) * t) for a, b in zip(WINDOW, DEEP)))
    glow = Image.new("L", (w, h), 0)
    ImageDraw.Draw(glow).ellipse([-w * 0.3, -h * 0.7, w * 0.8, h * 0.55], fill=70)
    glow = glow.filter(ImageFilter.GaussianBlur(w // 10))
    image = Image.composite(Image.new("RGB", (w, h), (22, 36, 58)), image, glow).convert("RGBA")
    lines = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    ld = ImageDraw.Draw(lines)
    for y in range(27 * s, h, 27 * s):
        ld.line([(0, y), (w, y)], fill=ACCENT + (14,), width=1)
    return Image.alpha_composite(image, lines)


def rounded(canvas, box, radius, fill=None, outline=None, width=1, ss=3):
    """A rounded box drawn larger, then reduced, so its corners are smooth."""
    x0, y0, x1, y1 = [round(v) for v in box]
    pad = width + 2
    w, h = x1 - x0 + 2 * pad, y1 - y0 + 2 * pad
    big = Image.new("RGBA", (w * ss, h * ss), (0, 0, 0, 0))
    ImageDraw.Draw(big).rounded_rectangle([pad * ss, pad * ss, (w - pad) * ss - 1, (h - pad) * ss - 1],
                                          radius=radius * ss, fill=fill, outline=outline, width=width * ss)
    canvas.alpha_composite(big.resize((w, h), Image.LANCZOS), (x0 - pad, y0 - pad))


def dashed_box(canvas, box, radius, colour, s):
    """The window's empty tile: a rounded box edged in dashes, as "Choose a font..." is."""
    layer = Image.new("RGBA", canvas.size, (0, 0, 0, 0))
    rounded(layer, box, radius, outline=colour + (255,), width=max(1, round(1.5 * s)))
    x0, y0, x1, y1 = [round(v) for v in box]
    mask = Image.new("L", canvas.size, 255)
    md = ImageDraw.Draw(mask)
    dash, gap = 7 * s, 5 * s
    for x in range(x0 + radius + dash, x1 - radius, dash + gap):
        md.rectangle([x, y0 - 4 * s, x + gap - 1, y0 + 4 * s], fill=0)
        md.rectangle([x, y1 - 4 * s, x + gap - 1, y1 + 4 * s], fill=0)
    for y in range(y0 + radius + dash, y1 - radius, dash + gap):
        md.rectangle([x0 - 4 * s, y, x0 + 4 * s, y + gap - 1], fill=0)
        md.rectangle([x1 - 4 * s, y, x1 + 4 * s, y + gap - 1], fill=0)
    layer.putalpha(Image.composite(layer.getchannel("A"), Image.new("L", canvas.size, 0), mask))
    canvas.alpha_composite(layer)


def spaced(draw, x, y, text, font, fill, tracking, centre=False, right=False):
    widths = [draw.textlength(c, font=font) for c in text]
    total = sum(widths) + tracking * (len(text) - 1)
    if centre:
        x -= total / 2
    if right:
        x -= total
    for c, w in zip(text, widths):
        draw.text((x, y), c, font=font, fill=fill, anchor="ls")
        x += w + tracking
    return total


def chevron(canvas, cx, cy, size, colour, alpha, s):
    layer = Image.new("RGBA", canvas.size, (0, 0, 0, 0))
    d = ImageDraw.Draw(layer)
    half = size / 2
    d.line([(cx - half * 0.5, cy - half), (cx + half * 0.5, cy), (cx - half * 0.5, cy + half)],
           fill=colour + (alpha,), width=round(4 * s), joint="curve")
    canvas.alpha_composite(layer)


def make(s):
    canvas = ground(s)
    draw = ImageDraw.Draw(canvas)
    p = lambda v: v * s   # points to pixels

    # The header: the sigil, the title, the version in gold, the three-colour rule.
    sigil = Image.open(CONVERTER / "font-logo.png").convert("RGBA")
    sigil = sigil.resize((round(sigil.width * p(46) / sigil.height), p(46)), Image.LANCZOS)
    canvas.alpha_composite(sigil, (p(30), p(20)))
    title = face("Menu", p(30))
    tx = p(30) + sigil.width + p(14)
    draw.text((tx, p(56)), "Font Converter", font=title, fill=TEXT, anchor="ls")
    vx = tx + draw.textlength("Font Converter", font=title) + p(10)
    draw.text((vx, p(56)), version(), font=face("Menu", p(21)), fill=GOLD, anchor="ls")
    spaced(draw, p(W - 30), p(53), "FOR MACOS", face("Menu", p(15)), ACCENT, p(2), right=True)
    draw.line([(p(30), p(80)), (p(W - 30), p(80))], fill=TILE_EDGE, width=max(1, s))
    for i, colour in enumerate((PICKED, ACCENT, DONE)):
        draw.line([(p(30 + i * 40), p(80)), (p(30 + (i + 1) * 40), p(80))], fill=colour, width=p(2))

    # The two tiles: the app, edged orange like a picked font; Applications, dashed like an
    # empty tile waiting for it. At the foot of each, a pale strip for Finder's black name:
    # two lines for the app's long name, one for Applications.
    tw = 176
    top, foot = 64, 104
    for (cx, cy), kind in ((APP, "app"), (APPLICATIONS, "dest")):
        box = (p(cx - tw / 2), p(cy - top), p(cx + tw / 2), p(cy + foot))
        lines = 2 if kind == "app" else 1
        strip = (p(cx - tw / 2 + 8), p(cy + LABEL - 10), p(cx + tw / 2 - 8), p(cy + LABEL + 10 + LINE * (lines - 1)))
        if kind == "app":
            rounded(canvas, box, p(10), fill=PICKED + (26,), outline=PICKED + (255,), width=p(2))
            rounded(canvas, strip, p(6), fill=NAME_STRIP, outline=PICKED + (255,), width=max(1, s))
        else:
            rounded(canvas, box, p(10), fill=TILE + (200,))
            dashed_box(canvas, box, p(10), DASHED, s)
            rounded(canvas, strip, p(6), fill=NAME_STRIP, outline=ACCENT + (255,), width=max(1, s))
    draw = ImageDraw.Draw(canvas)
    small = face("Menu", p(11))
    spaced(draw, p(APP[0]), p(APP[1] - top - 8), "1  PICK UP", small, PICKED, p(1.5), centre=True)
    spaced(draw, p(APPLICATIONS[0]), p(APPLICATIONS[1] - top - 8), "2  DROP HERE", small, ACCENT, p(1.5), centre=True)

    # The arrow between them: three chevrons, fading in, in the accent blue.
    for i, alpha in enumerate((90, 170, 255)):
        chevron(canvas, p(326 + i * 34), p(APP[1]), p(30), ACCENT, alpha, s)

    draw = ImageDraw.Draw(canvas)
    draw.text((p(W / 2), p(APP[1] + foot + 24)), "Drag the converter into Applications, then open it from there.",
              font=face("Descriptions", p(15)), fill=MUTED, anchor="ms")

    # The papers: a faint rule and a label above the readme, the licences and the notices,
    # and one pale strip under their four names.
    draw.line([(p(30), p(326)), (p(W - 30), p(326))], fill=TILE_EDGE + (120,), width=max(1, s))
    label = face("Menu", p(11))
    spaced(draw, p(30), p(342), "README AND LICENCES", label, SOFT, p(1.5))
    spaced(draw, p(W - 30), p(342), "ANY FONT INTO KOTOR'S GAME FONTS", label, GOLD, p(1.5), right=True)
    strip = (p(30), p(EXTRAS_Y + LABEL - 10), p(W - 30), p(EXTRAS_Y + LABEL + 10))
    rounded(canvas, strip, p(6), fill=NAME_STRIP, outline=TILE_EDGE + (255,), width=max(1, s))
    return canvas.convert("RGB")


def main():
    for s, name in ((1, "dmg-background.png"), (2, "dmg-background@2x.png")):
        out = HERE / name
        image = make(s)
        image.save(out, optimize=True, dpi=(72 * s, 72 * s))
        print(f"{out}  {out.stat().st_size} bytes  {image.width}x{image.height}")


if __name__ == "__main__":
    main()
