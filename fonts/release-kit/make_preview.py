"""Draw the font sample picture for the mod pages (1920x1080, PNG), in the game's own
GUI text colours (vanilla .gui TEXT COLOR values)."""
from pathlib import Path
from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parents[2]   # this repository
KIT = Path(__file__).resolve().parent
OTF = ROOT / "fonts" / "otf"
OUT = KIT / "KMRP-Original-HD-Fonts-preview.png"

BLUE = (0, 168, 250)      # buttons and labels
YELLOW = (250, 255, 0)    # screen titles
WHITE = (248, 255, 248)   # conversation text, HUD numbers
BG = (4, 8, 13)
PANEL = (8, 16, 26)
RULE = (0, 60, 92)
DIM = (0, 118, 176)

W, H = 1920, 1080

ROWS = [
    ("Menu Large", "MenuLarge", 64, "GALACTIC HEADINGS 1234", YELLOW),
    ("Menu", "Menu", 48, "NEW GAME   LOAD GAME   OPTIONS", BLUE),
    ("Menu Medium", "MenuMedium", 44, "Saved Game 042   Dantooine   12:47", BLUE),
    ("Menu Small", "MenuSmall", 44, "Graphics  Sound  Controls  Gameplay", BLUE),
    ("Menu Small B", "MenuSmallB", 44, "EQUIPMENT  INVENTORY  CHARACTER", BLUE),
    ("Dialogue", "Dialogue", 46, "The quick brown fox jumps over the lazy dog.", WHITE),
    ("Descriptions", "Descriptions", 46, "1. Tell me more about the Jedi Order.", BLUE),
    ("Light", "Light", 46, "Journal: Find the missing star map. \u00c4\u00e9\u00f1\u00f8\u00df", WHITE),
    ("Console", "Console", 46, "0123456789  HP 125/125  FP 48", WHITE),
    ("Credits", "Credits", 44, "Lead Designer   Sound Effects   Music", WHITE),
    ("Credits Bold", "CreditsBold", 44, "ABCDEFGHIJKLM abcdefghijklm", WHITE),
]


def font(name, size):
    return ImageFont.truetype(str(OTF / f"KMRP-OriginalHD-{name}.otf"), size)


def main():
    img = Image.new("RGB", (W, H), BG)
    d = ImageDraw.Draw(img)

    d.text((80, 52), "KMRP ORIGINAL HD FONTS", font=font("MenuLarge", 78), fill=YELLOW)
    d.text((82, 150), "KOTOR's own fonts, redrawn in high resolution.  11 OpenType fonts.  Free under the SIL Open Font License 1.1.",
           font=font("Light", 34), fill=BLUE)

    top, bottom = 214, H - 74
    d.rectangle((60, top, W - 60, bottom), fill=PANEL)
    row_h = (bottom - top - 16) / len(ROWS)
    label_font = font("MenuSmall", 30)
    for i, (label, file, size, sample, colour) in enumerate(ROWS):
        y0 = top + 8 + i * row_h
        cy = y0 + row_h / 2
        if i:
            d.line((80, y0, W - 80, y0), fill=RULE, width=1)
        d.text((96, cy), label.upper(), font=label_font, fill=DIM, anchor="lm")
        d.text((470, cy), sample, font=font(file, size), fill=colour, anchor="lm")

    foot = font("Light", 28)
    d.text((80, H - 38), "by RaymanGT for KMRP, the KOTOR Modern Restoration Patch", font=foot, fill=DIM, anchor="lm")
    d.text((W - 80, H - 38), "github.com/RayesDiyab/KMRP-KOTOR-Modern-Restoration-Patch", font=foot, fill=DIM, anchor="rm")

    img.save(OUT, optimize=True)
    print(OUT, img.size)


main()
