#!/usr/bin/env python3
"""The KMRP Font Converter's press kit, in its own style.

Since 2026-11-09 the kit has a look of its own, taken from the converter's window rather
than from KMRP's release art (the maintainer: "make it distinct for font converter"):

| Element | From |
| --- | --- |
| Titles | KMRP Original HD Menu, the window's title face: the game's small capitals |
| Text | KMRP Original HD Descriptions, the window's text face |
| Colours | the window's Theme: its blue accent, gold for the version, the tiles' orange (a font picked) and green (converted) |
| Ground | the window's navy with faint baselines, as on a type specimen sheet |
| Frames | rounded like the window's tiles, edged in the colour of what they show: blue for the converter, green for fonts it made, silver for the game's own |
| Cover | the title, the three steps as chips in the window's colours, and sample tiles in four fonts |

Until that day the pages were composed from KMRP's own release tools (the starfield, the
crest-and-KOTOR lockup, silver and gold Georgia lettering); this script needed a KMRP
checkout then and does not now.

The screenshots are read from build/press-kit-shots (not committed; README.md beside the
pages says where each came from). Nothing in a frame is retouched or cropped: each is the
whole screen or window, scaled down.

    python font-converter/release/make_press_kit.py
"""
from pathlib import Path

from PIL import Image, ImageDraw, ImageFilter, ImageFont

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[1]
FONTS = REPO / "font-converter" / "fonts"
SYSTEM = Path(r"C:\Windows\Fonts")
SHOTS = REPO / "build" / "press-kit-shots"
OUT = HERE / "press-kit"
VERSION = "1.0"

# The window's colours (font-converter/Theme.cs, dark mode).
WINDOW = (10, 18, 32)
DEEP = (6, 11, 20)
TILE = (15, 26, 44)
TILE_EDGE = (62, 88, 126)
EXAMPLE = (5, 10, 18)
ACCENT = (42, 198, 239)
GOLD = (226, 195, 92)
PICKED = (255, 154, 60)
DONE = (126, 224, 171)
TEXT = (236, 243, 248)
MUTED = (177, 195, 208)
SOFT = (158, 177, 194)
SILVER = (198, 206, 218)
# The game's own text colours, as the tiles' examples show them.
GAME_BLUE = (0, 168, 250)
GAME_PALE = (200, 226, 250)

W = 1920
SS = 3   # shapes drawn this many times larger, then reduced: smooth curves


def face(name, px):
    path = FONTS / f"KMRP-OriginalHD-{name}.ttf"
    return ImageFont.truetype(str(path), round(px))


def system(file, px):
    return ImageFont.truetype(str(SYSTEM / file), round(px))


def ground(height):
    """The window's navy, deeper at the foot, with faint baselines every 54 pixels."""
    top, bottom = WINDOW, DEEP
    image = Image.new("RGB", (W, height), top)
    draw = ImageDraw.Draw(image)
    for y in range(height):
        t = y / max(1, height - 1)
        draw.line([(0, y), (W, y)], fill=tuple(round(a + (b - a) * t) for a, b in zip(top, bottom)))
    image = image.convert("RGBA")
    lines = Image.new("RGBA", image.size, (0, 0, 0, 0))
    ld = ImageDraw.Draw(lines)
    for y in range(54, height, 54):
        ld.line([(0, y), (W, y)], fill=ACCENT + (14,), width=1)
    # A glow from the top left, where the title stands.
    glow = Image.new("L", image.size, 0)
    ImageDraw.Draw(glow).ellipse([-W * 0.3, -height * 0.6, W * 0.75, height * 0.7], fill=60)
    glow = glow.filter(ImageFilter.GaussianBlur(W // 10))
    tint = Image.new("RGBA", image.size, (22, 36, 58, 255))
    image = Image.composite(tint, image, glow)
    return Image.alpha_composite(image, lines)


def rounded(canvas, box, radius, fill=None, outline=None, width=1):
    """A rounded box drawn smooth (larger, then reduced) onto `canvas`."""
    x0, y0, x1, y1 = [round(v) for v in box]
    pad = width + 2
    w, h = x1 - x0 + 2 * pad, y1 - y0 + 2 * pad
    big = Image.new("RGBA", (w * SS, h * SS), (0, 0, 0, 0))
    ImageDraw.Draw(big).rounded_rectangle([pad * SS, pad * SS, (w - pad) * SS - 1, (h - pad) * SS - 1], radius=radius * SS,
                                          fill=fill, outline=outline, width=width * SS)
    canvas.alpha_composite(big.resize((w, h), Image.LANCZOS), (x0 - pad, y0 - pad))


def shadow(canvas, box, radius, strength=150):
    x0, y0, x1, y1 = box
    layer = Image.new("RGBA", canvas.size, (0, 0, 0, 0))
    ImageDraw.Draw(layer).rounded_rectangle([x0, y0 + 14, x1, y1 + 18], radius=radius, fill=(0, 0, 0, strength))
    canvas.alpha_composite(layer.filter(ImageFilter.GaussianBlur(22)))


def spaced(draw, x, y, text, font, fill, tracking, anchor="ls", centre=False, right=False):
    """Letter-spaced text on a baseline; returns its width."""
    widths = [draw.textlength(c, font=font) for c in text]
    total = sum(widths) + tracking * (len(text) - 1)
    if centre:
        x -= total / 2
    if right:
        x -= total
    for c, w in zip(text, widths):
        draw.text((x, y), c, font=font, fill=fill, anchor=anchor)
        x += w + tracking
    return total


# ---- the cover ----

SAMPLES = (   # a tile: the font's name, its file, the line, the game's colour for it
    ("Georgia", system("georgia.ttf", 46), "Carth: We need a ship", GAME_PALE),
    ("Arial", system("arial.ttf", 44), "1. Who are you?", GAME_BLUE),
    ("Consolas", system("consola.ttf", 44), "XP 4,500", GAME_BLUE),
    ("KMRP Original HD", face("Menu", 52), "Character", GOLD),
)


def sample_tile(canvas, x, y, w, h, name, font, line, colour):
    """A tile as the window draws a converted one: green edge, the example on its dark box,
    the font's name in the chip at its foot."""
    shadow(canvas, (x, y, x + w, y + h), 14, 110)
    rounded(canvas, (x, y, x + w, y + h), 12, fill=TILE + (255,), outline=DONE + (255,), width=2)
    rounded(canvas, (x + 16, y + 16, x + w - 16, y + h - 64), 7, fill=EXAMPLE + (255,), outline=(46, 66, 96, 255), width=1)
    draw = ImageDraw.Draw(canvas)
    draw.text((x + 34, y + 16 + (h - 80) / 2), line, font=font, fill=colour, anchor="lm")
    rounded(canvas, (x + 16, y + h - 52, x + w - 16, y + h - 14), 6, fill=DONE + (36,), outline=DONE + (150,), width=1)
    label = face("Menu", 26)
    _, top, _, bottom = draw.textbbox((0, 0), "H", font=label, anchor="ls")
    draw.text((x + 30, y + h - 33 - (top + bottom) / 2), name, font=label, fill=DONE, anchor="ls")


def chip(canvas, x, y, number, words, colour, filled=False):
    """A step: its number in a disc, its words, edged (or filled) in the window's colour."""
    draw = ImageDraw.Draw(canvas)
    font = face("Menu", 25)
    w = 84 + draw.textlength(words, font=font) + 28
    h = 64
    if filled:
        rounded(canvas, (x, y, x + w, y + h), 9, fill=colour + (255,))
    else:
        rounded(canvas, (x, y, x + w, y + h), 9, fill=colour + (30,), outline=colour + (255,), width=2)
    ink = WINDOW if filled else colour
    rounded(canvas, (x + 16, y + 12, x + 56, y + 52), 20, fill=(ink if filled else colour) + (255,))
    centred(draw, (x + 36, y + 32), str(number), face("Menu", 26), colour if filled else WINDOW)
    _, top, _, bottom = draw.textbbox((0, 0), "H", font=font, anchor="ls")
    draw.text((x + 72, y + 32 - (top + bottom) / 2), words, font=font, fill=ink if filled else TEXT, anchor="ls")
    return w


def centred(draw, centre, text, font, fill):
    """Draws text with the middle of its drawn ink on centre (anchor "mm" centres the line box)."""
    left, top, right, bottom = draw.textbbox((0, 0), text, font=font, anchor="ls")
    draw.text((centre[0] - (left + right) / 2, centre[1] - (top + bottom) / 2), text, font=font, fill=fill, anchor="ls")


def make_cover():
    canvas = ground(1080)
    draw = ImageDraw.Draw(canvas)
    logo = Image.open(REPO / "font-converter" / "kmrp-logo.png").convert("RGBA")
    logo = logo.resize((round(logo.width * 128 / logo.height), 128), Image.LANCZOS)
    canvas.alpha_composite(logo, (110, 70))
    spaced(draw, 110 + logo.width + 26, 70 + 74, "BY KMRP, THE KOTOR MODERN RESTORATION PATCH", face("Descriptions", 24), MUTED, 5)
    spaced(draw, 110 + logo.width + 26, 70 + 112, "FOR STAR WARS: KNIGHTS OF THE OLD REPUBLIC", face("Descriptions", 20), SOFT, 4)

    title = face("Menu", 148)
    draw.text((104, 440), "Font", font=title, fill=TEXT, anchor="ls")
    draw.text((104, 596), "Converter", font=title, fill=TEXT, anchor="ls")
    x = 104 + draw.textlength("Converter", font=title) + 28
    draw.text((x, 596), VERSION, font=face("Menu", 92), fill=GOLD, anchor="ls")
    draw.line([(110, 634), (110 + 110, 634)], fill=PICKED, width=5)
    draw.line([(220, 634), (330, 634)], fill=ACCENT, width=5)
    draw.line([(330, 634), (440, 634)], fill=DONE, width=5)
    draw.text((110, 696), "Any font into KOTOR's game fonts.", font=face("Descriptions", 40), fill=ACCENT, anchor="ls")

    cx = 110
    for n, (words, colour, filled) in enumerate((("Pick a font", PICKED, False), ("KMRP or Original Game", ACCENT, False),
                                                 ("Convert", DONE, True)), 1):
        cx += chip(canvas, cx, 752, n, words, colour, filled) + 18

    draw = ImageDraw.Draw(canvas)
    draw.text((110, 900), "TTF \u00b7 OTF \u00b7 TTC \u00b7 WOFF \u00b7 WOFF2", font=face("Menu", 32), fill=GOLD, anchor="ls")
    # What it works with and where it runs (2026-11-13, the maintainer: "say it works with kmrp
    # and vanilla kotor and also it has linux, windows and macos support"); until then one line,
    # "For KMRP 1.6 or the original game, Windows, Free, GPL".
    draw.text((110, 952), "Works with KMRP 1.6 and vanilla KOTOR", font=face("Descriptions", 30), fill=TEXT, anchor="ls")
    draw.text((110, 996), "Windows  \u00b7  macOS  \u00b7  Linux  \u00b7  Free, GPL", font=face("Descriptions", 28),
              fill=MUTED, anchor="ls")

    tx, tw, th, gap = 1250, 580, 176, 26
    ty = (1080 - (4 * th + 3 * gap)) // 2
    for i, (name, font, line, colour) in enumerate(SAMPLES):
        sample_tile(canvas, tx, ty + i * (th + gap), tw, th, name, font, line, colour)
    return canvas.convert("RGB")


# ---- the pages ----

def header(canvas, subject):
    draw = ImageDraw.Draw(canvas)
    font = face("Menu", 58)
    draw.text((56, 96), "Font Converter", font=font, fill=TEXT, anchor="ls")
    x = 56 + draw.textlength("Font Converter", font=font) + 18
    draw.text((x, 96), VERSION, font=face("Menu", 40), fill=GOLD, anchor="ls")
    spaced(draw, W - 56, 90, subject, face("Menu", 34), ACCENT, 3, right=True)
    draw.line([(56, 124), (W - 56, 124)], fill=TILE_EDGE, width=2)
    for i, colour in enumerate((PICKED, ACCENT, DONE)):
        draw.line([(56 + i * 70, 124), (56 + (i + 1) * 70, 124)], fill=colour, width=4)


def footer(canvas, line):
    draw = ImageDraw.Draw(canvas)
    font = face("Descriptions", 32)
    while draw.textlength(line, font=font) > W - 160 and font.size > 18:
        font = face("Descriptions", font.size - 1)
    draw.text((W / 2, canvas.height - 46), line, font=font, fill=MUTED, anchor="ms")


def frame(canvas, shot, x, y, width, label, edge):
    """One screenshot, whole, scaled to `width`, in a rounded frame of `edge`, labelled under."""
    height = round(shot.height * width / shot.width)
    small = shot.resize((width, height), Image.LANCZOS).convert("RGBA")
    shadow(canvas, (x, y, x + width, y + height), 12)
    mask = Image.new("L", (width * SS, height * SS), 0)
    ImageDraw.Draw(mask).rounded_rectangle([0, 0, width * SS - 1, height * SS - 1], radius=10 * SS, fill=255)
    canvas.paste(small, (x, y), mask.resize((width, height), Image.LANCZOS))
    rounded(canvas, (x - 1, y - 1, x + width + 1, y + height + 1), 11, outline=edge + (255,), width=3)
    if label:
        spaced(ImageDraw.Draw(canvas), x + 4, y + height + 44, label, face("Menu", 26), edge, 2)
    return height


def load(name):
    path = SHOTS / (name + ".png")
    if not path.exists():
        raise SystemExit(f"Not found: {path} (README.md beside the pages says where it came from)")
    return Image.open(path).convert("RGB")


def solo(subject, line, shot, label, edge, width=1500):
    picture = load(shot)
    width = min(width, picture.width)
    height = round(picture.height * width / picture.width)
    canvas = ground(160 + height + 170)
    frame(canvas, picture, (W - width) // 2, 160, width, label, edge)
    header(canvas, subject)
    footer(canvas, line)
    return canvas.convert("RGB")


def pair(subject, line, before, before_label, after, after_label):
    """The game's own small at the left, the converted large at the right, an arrow between."""
    pictures = [load(before), load(after)]
    canvas = ground(1080)
    before_w = 560
    after_w = min(1196, round(720 * pictures[1].width / pictures[1].height))
    before_h = round(pictures[0].height * before_w / pictures[0].width)
    after_h = round(pictures[1].height * after_w / pictures[1].width)
    top = 160 + (780 - after_h) // 2
    frame(canvas, pictures[0], 56, top + (after_h - before_h) // 2, before_w, before_label, SILVER)
    frame(canvas, pictures[1], W - 56 - after_w, top, after_w, after_label, DONE)
    mid = top + after_h // 2
    ax = 56 + before_w + (W - 112 - before_w - after_w) // 2
    rounded(canvas, (ax - 40, mid - 40, ax + 40, mid + 40), 40, fill=DONE + (40,), outline=DONE + (255,), width=2)
    ImageDraw.Draw(canvas).polygon([(ax - 10, mid - 17), (ax + 16, mid), (ax - 10, mid + 17)], fill=DONE)
    header(canvas, subject)
    footer(canvas, line)
    return canvas.convert("RGB")


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    made = {
        "01-cover": make_cover(),
        "02-the-converter": solo("THE CONVERTER", "Pick a font for any kind of text. Every size the game needs is made from it.",
                                 "converter-window", "KMRP FONT CONVERTER  1600x900", ACCENT),
        "03-original-game": pair("ORIGINAL GAME", "No KMRP needed: made at the game's own size, sharp, straight into Override.",
                                 "vanilla-menu-game-font", "ORIGINAL GAME  GAME FONT",
                                 "vanilla-menu-georgia", "ORIGINAL GAME  GEORGIA  1280x960"),
        "04-conversations": solo("CONVERSATIONS", "Conversations, replies, the journal and every menu, in the font you picked.",
                                 "vanilla-replies-georgia", "ORIGINAL GAME  GEORGIA  1280x960", DONE, width=1180),
        "05-with-kmrp": solo("WITH KMRP", "With KMRP the fonts are sized for any screen and text size: here at 3440x1440.",
                             "kmrp-text-page-georgia", "KMRP  GEORGIA  3440x1440", DONE, width=1808),
    }
    for name, image in made.items():
        path = OUT / (name + ".png")
        image.save(path, optimize=True)
        print(f"  {path.relative_to(REPO)}  {image.width}x{image.height}  {path.stat().st_size} bytes")


if __name__ == "__main__":
    main()
