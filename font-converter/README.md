# KMRP Font Converter

Turns any font file into Star Wars: Knights of the Old Republic's game fonts, for KMRP
(1.6.0 or later, which sizes them for the screen and offers them as Modded Font) or for
the original game, at the game's own font sizes. One Windows program, .NET Framework 4,
no other files; it is meant to run under Wine and Proton too (not yet tried).

- How to use it, where the fonts go and how to undo it: [release/README.txt](release/README.txt),
  the readme in the release zip.
- How it works and what it writes: [docs/font-converter.md](docs/font-converter.md).
- Its release audit: [docs/release-audit-2026-11-08.md](docs/release-audit-2026-11-08.md).
- The standard these documents are held to: [docs/documentation-standard.md](../docs/documentation-standard.md).

## Building

This folder builds on its own; nothing outside it is needed but a C# compiler.
`powershell -ExecutionPolicy Bypass -File build.ps1` makes `dist\KMRP Font Converter.exe`
with Visual Studio's C# compiler (deterministic: the same sources give the same exe), or
the one that ships with Windows. `-Release` also makes `dist\KMRP-Font-Converter-<version>.zip`
with the readme, the licences and the notices. The version is `Program.Version` in
`Program.cs`, the only place it is written.

The examples on its tiles and the font metrics it keeps are in `data/`, made by
`make_data.py` from KMRP's `assets/hd-original-fonts`. After that font changes, run
`build.ps1 -Refresh` with a KMRP checkout beside this repository or named by `KMRP_ROOT`
(`tools/kmrp_checkout.py`; needs Python with Pillow) to make them again.

`python tests/Test-FontConverter.py`, from this repository's root, builds it and
checks what it writes for every kind of font file and both versions.

GPL-3.0-or-later, like KMRP. Created by RaymanGT.
