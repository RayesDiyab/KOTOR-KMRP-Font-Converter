#!/usr/bin/env python3
"""The KMRP Original HD fonts' press kit: the fonts alone, without the converter.

Made 2026-11-13 (the maintainer: "please create a font only press release kit"). Its look
is the fonts' own, the one the kit's preview and comparison pictures already have, not the
Font Converter's window:

| Element | From |
| --- | --- |
| Colours | the game's GUI text colours (vanilla .gui TEXT COLOR): blue, yellow, white |
| Ground | near black with a faint em grid, as on a type designer's sheet |
| Panels | the dark panel and blue rule of the preview picture, with corner marks |
| Text | the 11 KMRP Original HD fonts themselves, nothing else |

Every page is drawn from the files in fonts/otf and the numbers in fonts/README.md. No
picture of the game's own letters is made here (AGENTS.md rule 3); the before and after
picture stays the committed KMRP-Original-HD-Fonts-comparison.png.

    python fonts/release-kit/make_press_kit.py
"""
from pathlib import Path

from fontTools.pens.boundsPen import BoundsPen
from fontTools.pens.recordingPen import RecordingPen
from fontTools.ttLib import TTFont
from PIL import Image, ImageChops, ImageDraw, ImageFilter, ImageFont

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[1]
OTF = REPO / "fonts" / "otf"
OUT = HERE / "press-kit"
VERSION = "1.600"

# The game's GUI text colours, as make_preview.py and make_compare.py use them.
BLUE = (0, 168, 250)
YELLOW = (250, 255, 0)
WHITE = (248, 255, 248)
BG = (4, 8, 13)
DEEP = (2, 4, 8)
PANEL = (8, 16, 26)
RULE = (0, 60, 92)
DIM = (0, 118, 176)
MUTED = (150, 178, 196)

W, H = 1920, 1080
SS = 3   # shapes drawn this many times larger, then reduced: smooth edges

FACES = (   # label, file stem, game font names, used in the game for
    ("Menu", "Menu", "dialogfont16x16, dialogfont16x16b", "screen titles, menu buttons"),
    ("Menu Small", "MenuSmall", "dialogfont10x10, 10x10a, 16x16a", "option lists, small labels"),
    ("Menu Small B", "MenuSmallB", "dialogfont10x10b", "tabs, small bold labels"),
    ("Menu Medium", "MenuMedium", "dialogfont12x16", "save and load lists"),
    ("Menu Large", "MenuLarge", "dialogfont32x32", "the largest headings"),
    ("Dialogue", "Dialogue", "fnt_d16x16, fnt_d10x10b", "conversations, subtitles"),
    ("Descriptions", "Descriptions", "fnt_d16x16b", "reply choices, descriptions"),
    ("Light", "Light", "fnt_d16x16a, fnt_creditsa, fnt_dialog16x16", "the journal, longer reading"),
    ("Console", "Console", "fnt_console", "the HUD's numbers"),
    ("Credits", "Credits", "fnt_credits", "job titles in the end credits"),
    ("Credits Bold", "CreditsBold", "fnt_creditsb", "names in the end credits"),
)


def face(stem, px):
    return ImageFont.truetype(str(OTF / f"KMRP-OriginalHD-{stem}.otf"), round(px))


# ---- drawing helpers ----

def ground():
    """Near black, deeper at the foot, a faint em grid, a soft blue glow at the top left."""
    image = Image.new("RGB", (W, H), BG)
    draw = ImageDraw.Draw(image)
    for y in range(H):
        t = y / (H - 1)
        draw.line([(0, y), (W, y)], fill=tuple(round(a + (b - a) * t) for a, b in zip(BG, DEEP)))
    image = image.convert("RGBA")
    glow = Image.new("L", image.size, 0)
    ImageDraw.Draw(glow).ellipse([-W * 0.25, -H * 0.7, W * 0.7, H * 0.6], fill=70)
    glow = glow.filter(ImageFilter.GaussianBlur(W // 10))
    image = Image.composite(Image.new("RGBA", image.size, (6, 22, 40, 255)), image, glow)
    grid = Image.new("RGBA", image.size, (0, 0, 0, 0))
    gd = ImageDraw.Draw(grid)
    for x in range(0, W, 60):
        gd.line([(x, 0), (x, H)], fill=BLUE + (9,), width=1)
    for y in range(0, H, 60):
        gd.line([(0, y), (W, y)], fill=BLUE + (9,), width=1)
    return Image.alpha_composite(image, grid)


def smooth(canvas, shape, box, **kw):
    """A rectangle or rounded rectangle drawn larger, then reduced onto canvas."""
    x0, y0, x1, y1 = [round(v) for v in box]
    width = kw.get("width", 1)
    pad = width + 2
    w, h = x1 - x0 + 2 * pad, y1 - y0 + 2 * pad
    big = Image.new("RGBA", (w * SS, h * SS), (0, 0, 0, 0))
    d = ImageDraw.Draw(big)
    rect = [pad * SS, pad * SS, (w - pad) * SS - 1, (h - pad) * SS - 1]
    if shape == "round":
        d.rounded_rectangle(rect, radius=kw.get("radius", 8) * SS, fill=kw.get("fill"),
                            outline=kw.get("outline"), width=width * SS)
    else:
        d.rectangle(rect, fill=kw.get("fill"), outline=kw.get("outline"), width=width * SS)
    canvas.alpha_composite(big.resize((w, h), Image.LANCZOS), (x0 - pad, y0 - pad))


def panel(canvas, box, edge=RULE, marks=BLUE):
    """The preview picture's dark panel, a thin rule round it, short marks at its corners."""
    x0, y0, x1, y1 = box
    layer = Image.new("RGBA", canvas.size, (0, 0, 0, 0))
    ImageDraw.Draw(layer).rectangle([x0, y0 + 12, x1, y1 + 16], fill=(0, 0, 0, 150))
    canvas.alpha_composite(layer.filter(ImageFilter.GaussianBlur(20)))
    smooth(canvas, "rect", box, fill=PANEL + (235,), outline=edge + (255,), width=1)
    draw = ImageDraw.Draw(canvas)
    m = 22
    for (cx, cy, dx, dy) in ((x0, y0, 1, 1), (x1, y0, -1, 1), (x0, y1, 1, -1), (x1, y1, -1, -1)):
        draw.line([(cx, cy), (cx + dx * m, cy)], fill=marks, width=3)
        draw.line([(cx, cy), (cx, cy + dy * m)], fill=marks, width=3)


def ink_box(draw, text, font):
    return draw.textbbox((0, 0), text, font=font, anchor="ls")


def middle(draw, x, cy, text, font, fill, centre=False, right=False):
    """Text whose drawn ink is centred on cy (not its line box); returns its width."""
    left, top, rgt, bottom = ink_box(draw, text, font)
    width = draw.textlength(text, font=font)
    if centre:
        x -= width / 2
    elif right:
        x -= width
    draw.text((x, cy - (top + bottom) / 2), text, font=font, fill=fill, anchor="ls")
    return width


def spaced(draw, x, y, text, font, fill, tracking, right=False):
    widths = [draw.textlength(c, font=font) for c in text]
    total = sum(widths) + tracking * (len(text) - 1)
    if right:
        x -= total
    for c, w in zip(text, widths):
        draw.text((x, y), c, font=font, fill=fill, anchor="ls")
        x += w + tracking
    return total


def header(canvas, subject):
    draw = ImageDraw.Draw(canvas)
    title = face("MenuLarge", 46)
    draw.text((60, 92), "KMRP Original HD Fonts", font=title, fill=YELLOW, anchor="ls")
    x = 60 + draw.textlength("KMRP Original HD Fonts", font=title) + 20
    draw.text((x, 92), VERSION, font=face("Menu", 30), fill=DIM, anchor="ls")
    spaced(draw, W - 60, 88, subject, face("MenuSmallB", 30), BLUE, 3, right=True)
    draw.line([(60, 122), (W - 60, 122)], fill=RULE, width=2)
    draw.line([(60, 122), (60 + 180, 122)], fill=YELLOW, width=4)
    draw.line([(240, 122), (240 + 180, 122)], fill=BLUE, width=4)


def footer(canvas, line):
    draw = ImageDraw.Draw(canvas)
    font = face("Light", 28)
    draw.text((60, H - 40), line, font=font, fill=DIM, anchor="ls")
    spaced(draw, W - 60, H - 40, "BY RAYMANGT FOR KMRP", face("MenuSmallB", 24), DIM, 3, right=True)


# ---- kerning, read from the files' GPOS (Pillow without raqm sets no kerning) ----

class Kerning:
    def __init__(self, stem):
        self.font = TTFont(OTF / f"KMRP-OriginalHD-{stem}.otf")
        self.cmap = self.font.getBestCmap()
        self.hmtx = self.font["hmtx"].metrics
        self.upm = self.font["head"].unitsPerEm
        self.subtables = []
        if "GPOS" in self.font:
            for lookup in self.font["GPOS"].table.LookupList.Lookup:
                if lookup.LookupType == 2:
                    self.subtables.extend(lookup.SubTable)

    def glyph(self, ch):
        return self.cmap[ord(ch)]

    def advance(self, ch):
        return self.hmtx[self.glyph(ch)][0]

    def pair(self, a, b):
        left, right = self.glyph(a), self.glyph(b)
        for st in self.subtables:
            if left not in st.Coverage.glyphs:
                continue
            if st.Format == 1:
                ps = st.PairSet[st.Coverage.glyphs.index(left)]
                for rec in ps.PairValueRecord:
                    if rec.SecondGlyph == right:
                        return getattr(rec.Value1, "XAdvance", 0) or 0
            else:
                c1 = st.ClassDef1.classDefs.get(left, 0)
                c2 = st.ClassDef2.classDefs.get(right, 0)
                value = st.Class1Record[c1].Class2Record[c2].Value1
                v = getattr(value, "XAdvance", 0) if value else 0
                if v:
                    return v
        return 0

    def layout(self, text, px, kern=True):
        """x positions in pixels of each character, and the kerning applied before each."""
        scale = px / self.upm
        xs, kerns, x = [], [], 0.0
        for i, ch in enumerate(text):
            k = self.pair(text[i - 1], ch) if kern and i else 0
            x += k * scale
            xs.append(x)
            kerns.append(k)
            x += self.advance(ch) * scale
        return xs, kerns, x


def set_line(draw, x, y, text, stem, px, fill, kern=True):
    k = Kerning(stem)
    font = face(stem, px)
    xs, kerns, width = k.layout(text, px, kern)
    for ch, cx in zip(text, xs):
        draw.text((x + cx, y), ch, font=font, fill=fill, anchor="ls")
    return xs, kerns, width


# ---- the pages ----

COVER_TILES = (   # face label, stem, size, line, colour (the game's colour for that text)
    ("Menu Large", "MenuLarge", 64, "OPTIONS", YELLOW),
    ("Menu", "Menu", 52, "LOAD GAME", BLUE),
    ("Dialogue", "Dialogue", 54, "Ebon Hawk", WHITE),
    ("Descriptions", "Descriptions", 50, "1. Tell me more.", BLUE),
    ("Console", "Console", 52, "HP 125/125", WHITE),
)


def make_cover():
    canvas = ground()
    draw = ImageDraw.Draw(canvas)
    logo = Image.open(REPO / "font-converter" / "kmrp-logo.png").convert("RGBA")
    logo = logo.resize((round(logo.width * 120 / logo.height), 120), Image.LANCZOS)
    canvas.alpha_composite(logo, (104, 72))
    lx = 104 + logo.width + 28
    spaced(draw, lx, 72 + 50, "BY RAYMANGT FOR KMRP", face("MenuSmallB", 26), BLUE, 4)
    spaced(draw, lx, 72 + 86, "THE KOTOR MODERN RESTORATION PATCH", face("MenuSmall", 22), DIM, 4)
    spaced(draw, lx, 72 + 116, "FOR STAR WARS: KNIGHTS OF THE OLD REPUBLIC", face("MenuSmall", 22), DIM, 4)

    title = face("MenuLarge", 132)
    draw.text((100, 420), "KMRP", font=title, fill=YELLOW, anchor="ls")
    draw.text((100, 560), "Original HD", font=title, fill=YELLOW, anchor="ls")
    draw.text((100, 700), "Fonts", font=title, fill=YELLOW, anchor="ls")
    x = 100 + draw.textlength("Fonts", font=title) + 30
    draw.text((x, 700), VERSION, font=face("Menu", 60), fill=DIM, anchor="ls")
    draw.line([(106, 738), (106 + 150, 738)], fill=YELLOW, width=5)
    draw.line([(256, 738), (256 + 150, 738)], fill=BLUE, width=5)
    draw.text((106, 806), "KOTOR's own fonts, redrawn in high resolution.", font=face("Dialogue", 42),
              fill=WHITE, anchor="ls")

    facts = ("11 OpenType fonts", "One .ttc collection", "218 characters")
    cx = 106
    chip_font = face("MenuSmallB", 26)
    for words in facts:
        w = draw.textlength(words, font=chip_font) + 48
        smooth(canvas, "round", (cx, 852, cx + w, 912), radius=6, fill=BLUE + (28,), outline=BLUE + (255,), width=2)
        draw = ImageDraw.Draw(canvas)
        middle(draw, cx + 24, 882, words, chip_font, BLUE)
        cx += w + 16
    draw.text((106, 980), "Free under the SIL Open Font License 1.1", font=face("Descriptions", 32),
              fill=YELLOW, anchor="ls")
    draw.text((106, 1024), "Use them in any game, mod, app, video or website, with credit.",
              font=face("Light", 30), fill=MUTED, anchor="ls")

    tx, tw, th, gap = 1250, 590, 150, 24
    ty = (H - (len(COVER_TILES) * th + (len(COVER_TILES) - 1) * gap)) // 2
    label_font = face("MenuSmallB", 22)
    for i, (label, stem, px, line, colour) in enumerate(COVER_TILES):
        y = ty + i * (th + gap)
        panel(canvas, (tx, y, tx + tw, y + th))
        draw = ImageDraw.Draw(canvas)
        spaced(draw, tx + 26, y + 40, label.upper(), label_font, DIM, 3)
        middle(draw, tx + 26, y + 96, line, face(stem, px), colour)
    return canvas.convert("RGB")


FAMILY = {   # stem: the line, its size, its colour (make_preview.py's lines)
    "MenuLarge": ("GALACTIC HEADINGS 1234", 60, YELLOW),
    "Menu": ("NEW GAME   LOAD GAME   OPTIONS", 46, BLUE),
    "MenuMedium": ("Saved Game 042   Dantooine   12:47", 42, BLUE),
    "MenuSmall": ("Graphics  Sound  Controls  Gameplay", 42, BLUE),
    "MenuSmallB": ("EQUIPMENT  INVENTORY  CHARACTER", 42, BLUE),
    "Dialogue": ("The quick brown fox jumps over the lazy dog.", 44, WHITE),
    "Descriptions": ("1. Tell me more about the Jedi Order.", 44, BLUE),
    "Light": ("Journal: Find the missing star map. \u00c4\u00e9\u00f1\u00f8\u00df", 44, WHITE),
    "Console": ("0123456789  HP 125/125  FP 48", 44, WHITE),
    "Credits": ("Lead Designer   Sound Effects   Music", 42, WHITE),
    "CreditsBold": ("ABCDEFGHIJKLM abcdefghijklm", 42, WHITE),
}
FAMILY_ORDER = ("MenuLarge", "Menu", "MenuMedium", "MenuSmall", "MenuSmallB", "Dialogue",
                "Descriptions", "Light", "Console", "Credits", "CreditsBold")


def make_family():
    canvas = ground()
    top, bottom = 160, H - 86
    panel(canvas, (60, top, W - 60, bottom))
    draw = ImageDraw.Draw(canvas)
    labels = {stem: label for label, stem, _, _ in FACES}
    row = (bottom - top - 20) / len(FAMILY_ORDER)
    label_font = face("MenuSmallB", 26)
    for i, stem in enumerate(FAMILY_ORDER):
        y0 = top + 10 + i * row
        cy = y0 + row / 2
        if i:
            draw.line([(84, y0), (W - 84, y0)], fill=RULE, width=1)
        spaced(draw, 100, cy + 10, labels[stem].upper(), label_font, DIM, 3)
        line, px, colour = FAMILY[stem]
        middle(draw, 470, cy, line, face(stem, px), colour)
    header(canvas, "THE FAMILY")
    footer(canvas, "Eleven fonts for the game's seventeen: the game keeps the same letters under several names.")
    return canvas.convert("RGB")


def outline(canvas, stem, ch, origin, px):
    """One glyph's outline from the file: straight segments blue, arcs yellow, on-curve
    points as squares, handles as small rings on thin lines."""
    font = TTFont(OTF / f"KMRP-OriginalHD-{stem}.otf")
    gs = font.getGlyphSet()
    name = font.getBestCmap()[ord(ch)]
    pen = RecordingPen()
    gs[name].draw(pen)
    scale = px / font["head"].unitsPerEm
    ox, oy = origin

    def at(p):
        return ((ox + p[0] * scale) * SS, (oy - p[1] * scale) * SS)

    big = Image.new("RGBA", (canvas.width * SS, canvas.height * SS), (0, 0, 0, 0))
    fill = Image.new("RGBA", big.size, (0, 0, 0, 0))
    fd = ImageDraw.Draw(fill)
    d = ImageDraw.Draw(big)
    # The filled glyph, faint, for the shape; even-odd fill by drawing each contour with XOR.
    mask = Image.new("1", big.size, 0)
    contours, current = [], []
    for op, args in pen.value:
        if op == "moveTo":
            current = [args[0]]
        elif op == "lineTo":
            current.append(args[0])
        elif op == "curveTo":
            p0 = current[-1]
            p1, p2, p3 = args
            for i in range(1, 25):
                t = i / 24
                u = 1 - t
                current.append(tuple(u ** 3 * a + 3 * u * u * t * b + 3 * u * t * t * c + t ** 3 * e
                                     for a, b, c, e in zip(p0, p1, p2, p3)))
        elif op in ("closePath", "endPath"):
            contours.append(current)
    for c in contours:
        layer = Image.new("1", big.size, 0)
        ImageDraw.Draw(layer).polygon([at(p) for p in c], fill=1)
        mask = ImageChops.logical_xor(mask, layer)
    fd.bitmap((0, 0), mask, fill=BLUE + (34,))
    big.alpha_composite(fill)

    lines = curves = points = handles = 0
    w = 3 * SS
    start = prev = None
    for op, args in pen.value:
        if op == "moveTo":
            start = prev = args[0]
        elif op == "lineTo":
            d.line([at(prev), at(args[0])], fill=BLUE + (255,), width=w)
            prev = args[0]
            lines += 1
        elif op == "curveTo":
            p1, p2, p3 = args
            d.line([at(prev), at(p1)], fill=MUTED + (150,), width=SS)
            d.line([at(p3), at(p2)], fill=MUTED + (150,), width=SS)
            pts = []
            for i in range(0, 41):
                t = i / 40
                u = 1 - t
                pts.append(at(tuple(u ** 3 * a + 3 * u * u * t * b + 3 * u * t * t * c + t ** 3 * e
                                    for a, b, c, e in zip(prev, p1, p2, p3))))
            d.line(pts, fill=YELLOW + (255,), width=w, joint="curve")
            for h in (p1, p2):
                x, y = at(h)
                r = 5 * SS
                d.ellipse([x - r, y - r, x + r, y + r], fill=PANEL + (255,), outline=MUTED + (255,), width=SS)
                handles += 1
            prev = p3
            curves += 1
        elif op == "closePath":
            if prev != start:
                d.line([at(prev), at(start)], fill=BLUE + (255,), width=w)
                lines += 1
    for op, args in pen.value:
        if op in ("moveTo", "lineTo", "curveTo"):
            x, y = at(args[-1])
            r = 6 * SS
            d.rectangle([x - r, y - r, x + r, y + r], fill=WHITE + (255,), outline=BG + (255,), width=SS)
            points += 1
    canvas.alpha_composite(big.resize(canvas.size, Image.LANCZOS))
    return lines, curves, points


def make_drawn():
    canvas = ground()
    header(canvas, "DRAWN, NOT TRACED")
    left = (60, 160, 1290, H - 86)
    panel(canvas, left)
    draw = ImageDraw.Draw(canvas)
    # Baseline and em lines for the two glyphs.
    base = 716
    px = 660
    for stem, ch, x in (("Menu", "R", 150), ("Dialogue", "g", 760)):
        font = TTFont(OTF / f"KMRP-OriginalHD-{stem}.otf")
        adv = font["hmtx"].metrics[font.getBestCmap()[ord(ch)]][0] * px / 1000
        draw.line([(x - 30, base), (x + adv + 30, base)], fill=DIM, width=1)
        xh = font["OS/2"].sCapHeight if ch.isupper() else font["OS/2"].sxHeight
        draw.line([(x - 30, base - xh * px / 1000), (x + adv + 30, base - xh * px / 1000)], fill=RULE, width=1)
        n_lines, n_curves, n_points = outline(canvas, stem, ch, (x, base), px)
        draw = ImageDraw.Draw(canvas)
        spaced(draw, x, H - 166, f"{stem.upper()}   {ch}", face("MenuSmallB", 26), BLUE, 3)
        draw.text((x, H - 126), f"{n_lines} straight edges, {n_curves} arcs, {n_points} points",
                  font=face("Light", 28), fill=MUTED, anchor="ls")

    right = (1330, 160, W - 60, H - 86)
    panel(canvas, right)
    draw = ImageDraw.Draw(canvas)
    x = 1370
    y = 226
    key = (("Straight edges", BLUE, "line"), ("Arcs", YELLOW, "line"), ("Points on the outline", WHITE, "square"),
           ("Arc handles", MUTED, "ring"))
    for words, colour, kind in key:
        if kind == "line":
            draw.line([(x, y - 12), (x + 46, y - 12)], fill=colour, width=4)
        elif kind == "square":
            draw.rectangle([x + 16, y - 19, x + 30, y - 5], fill=colour)
        else:
            draw.ellipse([x + 17, y - 18, x + 29, y - 6], outline=colour, width=2)
        draw.text((x + 66, y), words, font=face("Descriptions", 30), fill=WHITE, anchor="ls")
        y += 52
    y += 28
    body = face("Light", 30)
    lines = (
        "Each letter was measured from KMRP's",
        "HD atlas of the game's fonts, then drawn",
        "again as straight lines, sharp corners",
        "and arcs: no stair-stepped diagonals,",
        "no smoothing, no tracing.",
        "",
        "Letter widths, spacing and baseline are",
        "the game's to a unit, so text sets as it",
        "does in KOTOR.",
    )
    for line in lines:
        draw.text((x, y), line, font=body, fill=WHITE, anchor="ls")
        y += 42
    stat = face("MenuLarge", 60)
    draw.text((x, y + 30), "0.62-1.61%", font=stat, fill=YELLOW, anchor="ls")
    draw.text((x, y + 74), "mean difference on the ink from the", font=face("Light", 26), fill=MUTED, anchor="ls")
    draw.text((x, y + 106), "HD atlases they were drawn from, per font", font=face("Light", 26), fill=MUTED,
              anchor="ls")
    footer(canvas, "Both outlines exactly as they are in the files, 1000 units to the em.")
    return canvas.convert("RGB")


def make_characters():
    canvas = ground()
    header(canvas, "218 CHARACTERS")
    box = (60, 160, W - 60, H - 86)
    panel(canvas, box)
    draw = ImageDraw.Draw(canvas)
    k = Kerning("Dialogue")
    codes = [c for c in sorted(k.cmap) if c not in (0x20, 0xA0)]
    cols = 26
    rows = -(-len(codes) // cols)
    cw = (box[2] - box[0] - 60) / cols
    ch = (box[3] - box[1] - 140) / rows
    font = face("Dialogue", 46)
    glyphs = k.font.getGlyphSet()
    for i, c in enumerate(codes):
        r, q = divmod(i, cols)
        cx = box[0] + 30 + q * cw + cw / 2
        cy = box[1] + 30 + r * ch + ch / 2
        draw.rectangle([cx - cw / 2 + 3, cy - ch / 2 + 3, cx + cw / 2 - 3, cy + ch / 2 - 3], outline=RULE, width=1)
        colour = YELLOW if chr(c).isalpha() and c > 0x7F else WHITE
        # Centred by the ink, on the baseline: a few of Dialogue's advances are the game's
        # own wide cells (], the left single quote and guillemet, 1684 units), so centring
        # by advance would push their ink aside.
        # The ink's extent from the outline itself: Pillow's text box reaches to the advance.
        bounds = BoundsPen(glyphs)
        glyphs[k.glyph(chr(c))].draw(bounds)
        left, _, right, _ = (v * font.size / k.upm for v in bounds.bounds)
        draw.text((cx - (left + right) / 2, cy + 16), chr(c), font=font, fill=colour, anchor="ls")
    y = box[3] - 50
    draw.text((box[0] + 34, y), f"Dialogue, all {len(codes)} drawn characters (and the two spaces).",
              font=face("Light", 30), fill=WHITE, anchor="ls")
    middle(draw, box[2] - 34, y - 10, "Letters beyond A to Z in yellow", face("Light", 30), YELLOW, right=True)
    footer(canvas, "Windows-1252, the game's own set: English, French, German, Italian, Spanish. Console: 96.")
    return canvas.convert("RGB")


KERN_LINES = (   # stem, size, text, colour
    ("Dialogue", 88, "Yavin Tatooine AVATAR", WHITE),
    ("Menu", 76, "WAVE TRAVEL LYRA", BLUE),
)


def make_kerning():
    canvas = ground()
    header(canvas, "KERNING")
    box = (60, 160, W - 60, H - 86)
    panel(canvas, box)
    draw = ImageDraw.Draw(canvas)
    y = 300
    label = face("MenuSmallB", 22)
    numbers = face("Console", 20)
    for stem, px, text, colour in KERN_LINES:
        spaced(draw, 110, y - 92, f"{stem.upper()}   WITHOUT KERNING, AS IN THE GAME", label, DIM, 3)
        set_line(draw, 110, y, text, stem, px, tuple(round(c * 0.55) for c in colour), kern=False)
        y += 160
        spaced(draw, 110, y - 92, f"{stem.upper()}   WITH THE FONT'S KERNING", label, BLUE, 3)
        xs, kerns, _ = set_line(draw, 110, y, text, stem, px, colour, kern=True)
        n = 0
        for cx, kv in zip(xs, kerns):
            if kv:
                drop = 28 * (n % 2)   # every other value a line lower, so neighbours never touch
                draw.line([(110 + cx, y + 16), (110 + cx, y + 30 + drop)], fill=YELLOW, width=2)
                draw.text((110 + cx, y + 54 + drop), str(kv), font=numbers, fill=YELLOW, anchor="ms")
                n += 1
        y += 250
    counts = []
    for _, stem, _, _ in FACES:
        f = Kerning(stem)
        n = 0
        for st in f.subtables:
            if st.Format == 1:
                n += sum(len(ps.PairValueRecord) for ps in st.PairSet)
            else:
                n += sum(1 for c1 in st.Class1Record for c2 in c1.Class2Record if c2.Value1 and c2.Value1.XAdvance)
        if n:
            counts.append(n)
    nx, ny = 1420, 300
    draw.line([(nx - 40, box[1] + 60), (nx - 40, box[3] - 60)], fill=RULE, width=1)
    draw.text((nx, ny), f"{min(counts):,}-{max(counts):,}", font=face("MenuLarge", 60), fill=YELLOW, anchor="ls")
    note = face("Light", 28)
    for i, line in enumerate(("letter pairs kerned in each font,", "measured from the outlines:",
                              "a to z and A to Z with their accented", "forms. Tightening only.", "",
                              "Console has none: it is monospaced.", "", "Yellow: each pair's kerning",
                              "in font units, 1000 to the em.")):
        draw.text((nx, ny + 56 + i * 40), line, font=note, fill=WHITE if i < 4 else MUTED, anchor="ls")
    footer(canvas, "For programs that kern: browsers, Word, design tools. KOTOR itself does not.")
    return canvas.convert("RGB")


def make_which():
    canvas = ground()
    header(canvas, "WHICH FONT IS WHICH")
    box = (60, 160, W - 60, H - 86)
    panel(canvas, box)
    draw = ImageDraw.Draw(canvas)
    cols = (100, 560, 1260)
    head = face("MenuSmallB", 24)
    hy = box[1] + 50
    for x, words in zip(cols, ("FONT", "GAME FONT NAMES", "USED IN THE GAME FOR")):
        spaced(draw, x, hy, words, head, DIM, 3)
    top = hy + 22
    row = (box[3] - top - 14) / len(FACES)
    names = face("Light", 28)
    for i, (label, stem, game, use) in enumerate(FACES):
        y0 = top + i * row
        cy = y0 + row / 2
        draw.line([(84, y0), (W - 84, y0)], fill=RULE, width=1)
        colour = YELLOW if stem == "MenuLarge" else BLUE if stem.startswith("Menu") or stem == "Descriptions" else WHITE
        middle(draw, cols[0], cy, label, face(stem, 38), colour)
        middle(draw, cols[1], cy, game, names, MUTED)
        middle(draw, cols[2], cy, use, names, WHITE)
    footer(canvas, "Installed, each appears as \"KMRP Original HD\" and its name: KMRP Original HD Dialogue.")
    return canvas.convert("RGB")


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    made = {
        "01-cover": make_cover(),
        "02-the-family": make_family(),
        "03-drawn-not-traced": make_drawn(),
        "04-characters": make_characters(),
        "05-kerning": make_kerning(),
        "06-which-font": make_which(),
    }
    for name, image in made.items():
        path = OUT / (name + ".png")
        image.save(path, optimize=True)
        print(f"  {path.relative_to(REPO)}  {image.width}x{image.height}  {path.stat().st_size} bytes")


if __name__ == "__main__":
    main()
