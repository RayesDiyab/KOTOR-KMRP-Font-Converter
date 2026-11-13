#!/usr/bin/env python3
"""Make the macOS app's icon: the sigil above FONT on a starry rounded square.

The maintainer, 2026-11-13: "the icon looks bad it has a white box remove that, make the icon
look good have a starry background". From macOS 26 Finder sets an icon that is not the
system's rounded square (icon.png is the sigil on a transparent ground) on a pale plate of
its own: the white box. This icon fills the square itself, on Apple's grid (an 824-pixel
rounded square in a 1024 canvas, 100 pixels in from each edge, with a soft shadow), so
Finder shows it as drawn.

The ground is the converter window's navy deepening to black, a blue glow behind the sigil
in the colour of its blade, and stars, fixed by a seed so every run draws the same sky.

Writes icon-macos.png (1024 x 1024) beside itself; build-macos.sh makes AppIcon.icns from
it. Windows and Linux keep icon.png and favicon.ico from make_icon.py.

    python font-converter/make_icon_macos.py
"""
import math
import random
from pathlib import Path

from PIL import Image, ImageChops, ImageDraw, ImageFilter

HERE = Path(__file__).resolve().parent
SIDE = 1024
INSET = 100              # Apple's grid: the square is 824 pixels, 100 in from each edge
SQUARE = SIDE - 2 * INSET
SS = 4                   # the shape is drawn this many times larger, then reduced

WINDOW = (10, 18, 32)    # src/theme.rs, dark mode
DEEP = (3, 6, 12)
BLADE = (40, 120, 255)   # the sigil's blue
ACCENT = (42, 198, 239)


def squircle_mask(size, inset, side, n=4.0):
    """A superellipse, close to the system's rounded square, drawn smooth."""
    big = side * SS
    half = (side - 2 * inset) * SS / 2
    c = big / 2
    points = []
    for i in range(1440):
        t = 2 * math.pi * i / 1440
        ct, st = math.cos(t), math.sin(t)
        x = c + half * math.copysign(abs(ct) ** (2 / n), ct)
        y = c + half * math.copysign(abs(st) ** (2 / n), st)
        points.append((x, y))
    mask = Image.new("L", (big, big), 0)
    ImageDraw.Draw(mask).polygon(points, fill=255)
    return mask.resize((size, size), Image.LANCZOS)


def radial(size, centre, radius, colour, strength):
    """A soft round glow of one colour, as an RGBA layer."""
    layer = Image.new("L", (size, size), 0)
    cx, cy = centre
    ImageDraw.Draw(layer).ellipse([cx - radius, cy - radius, cx + radius, cy + radius], fill=strength)
    layer = layer.filter(ImageFilter.GaussianBlur(radius / 2.2))
    out = Image.new("RGBA", (size, size), colour + (0,))
    out.putalpha(layer)
    return out


def sky(size):
    image = Image.new("RGB", (size, size))
    draw = ImageDraw.Draw(image)
    for y in range(size):
        t = y / (size - 1)
        draw.line([(0, y), (size, y)], fill=tuple(round(a + (b - a) * t) for a, b in zip(WINDOW, DEEP)))
    image = image.convert("RGBA")
    # Nebulae: the sigil's blue behind it, a cyan wisp high on the right, a violet one low left.
    image.alpha_composite(radial(size, (size * 0.5, size * 0.44), size * 0.30, BLADE, 120))
    image.alpha_composite(radial(size, (size * 0.76, size * 0.24), size * 0.16, ACCENT, 60))
    image.alpha_composite(radial(size, (size * 0.24, size * 0.80), size * 0.18, (120, 70, 200), 55))

    rng = random.Random(1003)   # the same sky every run
    stars = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    sd = ImageDraw.Draw(stars)
    for _ in range(420):
        x, y = rng.uniform(0, size), rng.uniform(0, size)
        r = rng.choice((0.7, 0.9, 1.1, 1.3, 1.6, 2.0))
        tint = rng.choice(((255, 255, 255), (200, 225, 255), (170, 210, 255), (255, 240, 220)))
        alpha = rng.randint(90, 255)
        sd.ellipse([x - r, y - r, x + r, y + r], fill=tint + (alpha,))
    image.alpha_composite(stars.filter(ImageFilter.GaussianBlur(0.4)))
    image.alpha_composite(stars)

    # A few bright stars with a glow and four points of light.
    glints = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    gd = ImageDraw.Draw(glints)
    for x, y, length in ((0.20, 0.22, 34), (0.83, 0.58, 26), (0.12, 0.50, 18), (0.70, 0.14, 20), (0.88, 0.86, 16)):
        x, y = x * size, y * size
        for w, a in ((5, 50), (2, 160), (1, 255)):
            gd.line([(x - length, y), (x + length, y)], fill=(220, 236, 255, a), width=w)
            gd.line([(x, y - length), (x, y + length)], fill=(220, 236, 255, a), width=w)
        gd.ellipse([x - 3, y - 3, x + 3, y + 3], fill=(255, 255, 255, 255))
    image.alpha_composite(glints.filter(ImageFilter.GaussianBlur(1.2)))
    return image


def main():
    face = sky(SQUARE)

    # The sigil above FONT, as on the other systems, fitted to the square with room around it.
    logo = Image.open(HERE / "font-logo.png").convert("RGBA")
    logo = logo.crop(logo.getbbox())
    width = round(SQUARE * 0.80)
    logo = logo.resize((width, round(logo.height * width / logo.width)), Image.LANCZOS)
    # Its grey haze reads as fog on the night sky: keep only what is clearly drawn.
    alpha = logo.getchannel("A").point(lambda a: 0 if a < 40 else min(255, round((a - 40) * 255 / 215)))
    logo.putalpha(alpha)
    x = (SQUARE - logo.width) // 2
    y = (SQUARE - logo.height) // 2 + round(SQUARE * 0.02)
    shade = Image.new("RGBA", logo.size, (0, 0, 0, 0))
    shade.putalpha(logo.getchannel("A").point(lambda a: a * 0.6))
    face.alpha_composite(shade.filter(ImageFilter.GaussianBlur(10)), (x, y + 8))
    face.alpha_composite(logo, (x, y))

    # A thin light rim at the top edge, as the system's icons have.
    rim = Image.new("RGBA", face.size, (0, 0, 0, 0))
    shape = squircle_mask(SQUARE, 0, SQUARE)
    inner = shape.filter(ImageFilter.MinFilter(5))
    edge = ImageChops.subtract(shape, inner)
    fade = Image.linear_gradient("L").resize(face.size).point(lambda v: 255 - v)
    rim.putalpha(ImageChops.multiply(edge, fade.point(lambda v: v * 0.45)))
    rim_rgb = Image.new("RGBA", face.size, (190, 220, 255, 0))
    rim_rgb.putalpha(rim.getchannel("A"))
    face.alpha_composite(rim_rgb)

    icon = Image.new("RGBA", (SIDE, SIDE), (0, 0, 0, 0))
    mask = squircle_mask(SIDE, INSET, SIDE)
    # Apple's grid: a shadow under the square, 12 pixels down, blurred 28.
    shadow = Image.new("RGBA", (SIDE, SIDE), (0, 0, 0, 0))
    shadow.putalpha(mask.point(lambda a: a * 0.5))
    shadow = shadow.transform(shadow.size, Image.AFFINE, (1, 0, 0, 0, 1, -12)).filter(ImageFilter.GaussianBlur(28))
    icon.alpha_composite(shadow)
    placed = Image.new("RGBA", (SIDE, SIDE), (0, 0, 0, 0))
    placed.paste(face, (INSET, INSET))
    placed.putalpha(ImageChops.multiply(placed.getchannel("A"), mask))
    icon.alpha_composite(placed)

    out = HERE / "icon-macos.png"
    icon.save(out, optimize=True)
    print(f"{out}  {out.stat().st_size} bytes  {SIDE}x{SIDE}")


if __name__ == "__main__":
    main()
