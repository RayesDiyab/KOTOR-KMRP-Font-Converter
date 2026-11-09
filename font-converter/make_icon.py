#!/usr/bin/env python3
"""Make the converter's icons from font-logo.png: KMRP's sigil above the word FONT.

The maintainer, 2026-11-09: "make the app icon be the sigil with FONT below it like in the
readme". Until then favicon.ico was a copy of the KMRP installer's icon.

font-logo.png (677x539, made by KMRP's tools/build_brand_image.py --text FONT --tracking 0.12
--whole-crest) is centred on a transparent square as wide as it is, then each icon size is
made from that square with Lanczos. build.rs compiles favicon.ico into the Windows exe.

Since the Rust program (2026-11-09) it also writes icon.png, the same square at 512 pixels:
build-macos.sh makes the app's AppIcon.icns from it and build-linux.sh puts it in the
AppImage. The window draws its own icon from font-logo.png.

    python font-converter/make_icon.py
"""
from pathlib import Path

from PIL import Image

HERE = Path(__file__).resolve().parent
SIZES = [16, 24, 32, 48, 64, 128, 256]


def main():
    logo = Image.open(HERE / "font-logo.png").convert("RGBA")
    side = max(logo.size)
    square = Image.new("RGBA", (side, side), (0, 0, 0, 0))
    square.alpha_composite(logo, ((side - logo.width) // 2, (side - logo.height) // 2))
    out = HERE / "favicon.ico"
    square.save(out, format="ICO", sizes=[(s, s) for s in SIZES])
    print(f"{out}  {out.stat().st_size} bytes  sizes {SIZES}  from a {side}x{side} square")
    png = HERE / "icon.png"
    square.resize((512, 512), Image.LANCZOS).save(png, optimize=True)
    print(f"{png}  {png.stat().st_size} bytes  512x512")


if __name__ == "__main__":
    main()
