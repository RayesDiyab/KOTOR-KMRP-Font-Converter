#!/usr/bin/env python3
"""Set the version inside the finished fonts, and nothing else.

Written 2026-11-13, when the fonts' version became 1.0 (the maintainer: "the orignal hd
font is 1.0!"); until then finish.py and collection.py wrote 1.600. Rebuilding with
finish.py needs a KMRP checkout and AFDKO; this changes the three places the version is
in the files that are already here, so their outlines, hints and kerning stay byte for
byte:

| Where | Before | After |
| --- | --- | --- |
| head.fontRevision | 1.6 | 1.0 |
| name ID 5 | Version 1.600 (the collection: "; ttfautohint (...)" after it) | Version 1.000 (the same after it) |
| name ID 3 | 1.600;KMRP;<PostScript name> | 1.000;KMRP;<PostScript name> |

head.modified is kept. Every other table is compared before and after and must be the same.

    python fonts/tools/set_version.py
"""
import io
from pathlib import Path

from fontTools.ttLib import TTCollection, TTFont

FONTS = Path(__file__).resolve().parents[1]
OLD, NEW = "1.600", "1.000"
REVISION = 1.0


def original(font):
    """Each table's bytes as the file on disk holds them."""
    return {tag: font.reader[tag] for tag in font.reader.keys()}


def tables(font):
    """Each table's bytes as they would be written now."""
    buf = io.BytesIO()
    font.save(buf)
    buf.seek(0)
    again = TTFont(buf, lazy=True)
    return {tag: again.reader[tag] for tag in again.reader.keys()}


def retag(font):
    font["head"].fontRevision = REVISION
    changed = 0
    for rec in font["name"].names:
        if rec.nameID in (3, 5):
            text = rec.toUnicode()
            assert text.startswith(OLD) or text.startswith("Version " + OLD), text
            rec.string = text.replace(OLD, NEW, 1)
            changed += 1
    assert changed >= 2, changed


def check(before, after):
    for tag in before:
        if tag not in ("head", "name"):
            assert before[tag] == after[tag], tag
    assert set(before) == set(after)


def main():
    for path in sorted((FONTS / "otf").glob("*.otf")):
        font = TTFont(path, recalcTimestamp=False)
        before = original(font)
        modified = font["head"].modified
        retag(font)
        font["head"].modified = modified
        after = tables(font)
        check(before, after)
        font.save(path)
        print(f"  {path.name}  {path.stat().st_size} bytes")

    path = FONTS / "KMRP-OriginalHD.ttc"
    coll = TTCollection(path, recalcTimestamp=False)
    befores = []
    for font in coll.fonts:
        font.recalcTimestamp = False
        befores.append(original(font))
        modified = font["head"].modified
        retag(font)
        font["head"].modified = modified
    for font, before in zip(coll.fonts, befores):
        check(before, tables(font))
    coll.save(path, shareTables=True)
    print(f"  {path.name}  {path.stat().st_size} bytes")


if __name__ == "__main__":
    main()
