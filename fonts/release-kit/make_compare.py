"""Side-by-side picture: the game's original font textures, enlarged so their pixels show,
against the KMRP Original HD fonts at the same size, in the game's own text colours."""
import sys
from pathlib import Path

import numpy as np
from fontTools.ttLib import TTFont
from PIL import Image, ImageDraw, ImageFont

REPO = Path(__file__).resolve().parents[2]   # this repository
sys.path.insert(0, str(REPO / "tools"))
from kmrp_checkout import kmrp_root  # noqa: E402
ROOT = kmrp_root()                          # the KMRP checkout: its font tools and the game's fonts
KIT = Path(__file__).resolve().parent
sys.path.insert(0, str(ROOT / "tools"))
from build_hd_original_fonts import Stock  # noqa: E402

ORIGINALS = ROOT / "build" / "native-runtime" / "original-fonts"
OTF = REPO / "fonts" / "otf"
OUT = KIT / "KMRP-Original-HD-Fonts-comparison.png"

# The game's GUI colours (vanilla .gui TEXT COLOR): buttons and labels, titles, dialogue.
BLUE = (0, 168, 250)
YELLOW = (250, 255, 0)
WHITE = (248, 255, 248)
BG = (4, 8, 13)
PANEL = (8, 16, 26)
RULE = (0, 60, 92)
DIM = (0, 118, 176)

W, H = 1920, 1080
HALF = 860

ROWS = [  # label, OTF stem, game resref, text, colour
    ("Menu Large", "MenuLarge", "dialogfont32x32", "OPTIONS", YELLOW),
    ("Menu", "Menu", "dialogfont16x16", "LOAD GAME", BLUE),
    ("Dialogue", "Dialogue", "fnt_d16x16", "Ebon Hawk", WHITE),
    ("Descriptions", "Descriptions", "fnt_d16x16b", "1. Tell me more.", BLUE),
    ("Console", "Console", "fnt_console", "HP 125/125", WHITE),
]


def original_line(stock, text):
    """The text set from the original texture's cells (advance = cell width), as the
    letters' coverage (brightness times opacity, 0 to 1). Coverage rather than the raw
    texture: dialogfont32x32 carries a green tint of its own and the console font a dark
    ring, so both sides differ only in resolution."""
    codes = text.encode("cp1252")
    widths = [stock.width(c) for c in codes]
    out = np.zeros((stock.height, sum(widths)))
    x = 0
    for c, w in zip(codes, widths):
        if w > 0:
            cov = stock.coverage(c)
            out[:cov.shape[0], x:x + cov.shape[1]] = cov
        x += w
    return out


def tint(cov, colour):
    """Coverage as the text colour's opacity."""
    a = np.zeros(cov.shape + (4,))
    a[..., :3] = np.array(colour) / 255
    a[..., 3] = np.clip(cov, 0, 1)
    return Image.fromarray((a * 255 + 0.5).astype(np.uint8), "RGBA")


def advance_units(path, text):
    f = TTFont(path)
    cmap, hmtx = f.getBestCmap(), f["hmtx"]
    return sum(hmtx[cmap[ord(ch)]][0] for ch in text), f["head"].unitsPerEm


def cap_units(path):
    from fontTools.pens.boundsPen import BoundsPen
    gs = TTFont(path).getGlyphSet()
    pen = BoundsPen(gs)
    gs["H"].draw(pen)
    return pen.bounds[3]


def label_font(size):
    return ImageFont.truetype(str(OTF / "KMRP-OriginalHD-MenuSmall.otf"), size)


def main():
    img = Image.new("RGBA", (W, H), BG + (255,))
    d = ImageDraw.Draw(img)
    d.text((W / 2, 70), "KMRP ORIGINAL HD FONTS", font=ImageFont.truetype(str(OTF / "KMRP-OriginalHD-MenuLarge.otf"), 72),
           fill=YELLOW, anchor="mm")
    head = label_font(34)
    lx, rx = 80 + HALF / 2, W - 80 - HALF / 2
    d.text((lx, 150), "THE GAME'S ORIGINAL FONT", font=head, fill=DIM, anchor="mm")
    d.text((rx, 150), "KMRP ORIGINAL HD", font=head, fill=BLUE, anchor="mm")

    top, bottom = 185, H - 70
    d.rectangle((60, top, W - 60, bottom), fill=PANEL)
    d.line((W / 2, top + 16, W / 2, bottom - 16), fill=RULE, width=2)
    row_h = (bottom - top) / len(ROWS)
    small = label_font(24)

    for i, (label, stem, res, text, colour) in enumerate(ROWS):
        y0 = top + i * row_h
        if i:
            d.line((80, y0, W - 80, y0), fill=RULE, width=1)
        d.text((84, y0 + 14), label.upper(), font=small, fill=DIM, anchor="lt")

        stock = Stock(ORIGINALS, res)
        cov = original_line(stock, text)
        line = tint(cov, colour)
        # Largest whole-pixel enlargement that fits the half and the row.
        k = int(min((HALF - 40) / line.width, (row_h - 44) / stock.height))
        big = line.resize((line.width * k, line.height * k), Image.NEAREST)
        base_y = y0 + 34 + (row_h - 34 - big.height) / 2 + stock.baseline * k
        ox = round(lx - big.width / 2)
        img.alpha_composite(big, (ox, round(base_y - stock.baseline * k)))

        # The HD font at the same scale: its capital H as tall as the original's.
        path = OTF / f"KMRP-OriginalHD-{stem}.otf"
        units, upem = advance_units(path, text)
        size = k * stock.cap_height * upem / cap_units(path)
        hd = ImageFont.truetype(str(path), round(size))
        d.text((rx - units * size / upem / 2, base_y), text, font=hd, fill=colour, anchor="ls")
        print(f"{label:13} x{k}  original {line.width}x{line.height} px  HD {size:.1f} px/em")

    foot = label_font(24)
    d.text((80, H - 36), "Same text, same size, same game colours. Left: the game's font texture enlarged. Right: the HD font.",
           font=ImageFont.truetype(str(OTF / "KMRP-OriginalHD-Light.otf"), 28), fill=DIM, anchor="lm")
    d.text((W - 80, H - 36), "BY RAYMANGT FOR KMRP", font=foot, fill=DIM, anchor="rm")

    img.convert("RGB").save(OUT, optimize=True)
    print(OUT)


main()
