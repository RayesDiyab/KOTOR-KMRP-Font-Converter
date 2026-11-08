# KOTOR-KMRP Font Converter

Two things for Star Wars: Knights of the Old Republic, made for KMRP (the KOTOR Modern
Restoration Patch) and usable without it:

| | What it is | Licence |
| --- | --- | --- |
| [`font-converter/`](font-converter/) | **KMRP Font Converter**: turns any font file into the game's 17 text fonts, for KMRP 1.6.0 or later or for the original game. One Windows program | GPL-3.0-or-later ([LICENSE](LICENSE)) |
| [`fonts/`](fonts/) | **KMRP Original HD fonts**: KOTOR's own fonts redrawn in high resolution, as 11 OpenType files and one collection | SIL Open Font License 1.1 ([fonts/OFL.txt](fonts/OFL.txt)) |

KMRP ships its in-game HD Font as atlases drawn from these fonts; they stay in KMRP's own
repository (`assets/hd-original-fonts`). This repository holds the fonts themselves and the
converter. Both were in KMRP's repository until 2026-11-08, and their history came with
them.

- The converter: how to use it, [font-converter/release/README.txt](font-converter/release/README.txt);
  how it works, [font-converter/docs/font-converter.md](font-converter/docs/font-converter.md);
  building, [font-converter/README.md](font-converter/README.md).
- The fonts: [fonts/README.md](fonts/README.md).
- KMRP: https://github.com/RayesDiyab/KMRP-KOTOR-Modern-Restoration-Patch

## Layout

| Path | What |
| --- | --- |
| `font-converter/` | The converter's C# sources, `build.ps1`, its data, docs and release files |
| `fonts/otf/`, `fonts/KMRP-OriginalHD.ttc` | The fonts |
| `fonts/tools/` | How the fonts were redrawn (Python); some read KMRP's files |
| `fonts/release-kit/` | The fonts' own download: a zip and two pictures, and the scripts that make them |
| `tests/Test-FontConverter.py` | Builds the converter and checks what it writes |
| `tools/kmrp_checkout.py` | Finds a KMRP checkout (`KMRP_ROOT`, or beside this repository) for the tools that need one |

## Checking

```
powershell -ExecutionPolicy Bypass -File font-converter\build.ps1
python tests\Test-FontConverter.py
```

The test needs Windows' own fonts (Arial, Cambria, Bahnschrift, Georgia) and, for WOFF
files, Python with fontTools and brotli. With a built KMRP checkout beside this repository
it also compares the original-game fonts with the game's own.

Created by RaymanGT.
