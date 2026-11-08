# KMRP Font Converter

> **Documentation standard.** This document follows
> [`docs/documentation-standard.md`](https://github.com/RayesDiyab/KMRP-KOTOR-Modern-Restoration-Patch/blob/master/docs/documentation-standard.md). Read it before editing
> this file, and check the result still meets it: measured claims only, every
> site tabulated, rejected alternatives and corrections kept visible, and
> anything untested labelled as untested.

Reference, 2026-10-21, revised for release on 2026-11-08 after its audit. In KMRP's
repository until 2026-11-08, then in its own with the KMRP Original HD fonts (KOTOR-KMRP
Font Converter); `tools/` and `assets/` paths here are KMRP's unless they name this
repository's `tools/kmrp_checkout.py`. The audit is
[release-audit-2026-11-08.md](release-audit-2026-11-08.md). A separate program that turns
any font file into KOTOR's game fonts, for KMRP (1.6.0 or later) or for the original game.
It shares no code with KMRP's patch or installer and carries KMRP's branding. Version
1.0.0, released with KMRP 1.6.0 and KMSE on the same day (the maintainer, 2026-11-08).
**Run under Proton: not yet.** In the game: Georgia made for KMRP was seen on KMRP's Text
Size and Font page on 2026-11-08 (the maintainer's screenshot, before the release changes
below). Georgia made for the original game (1.0.0, `30643C4A...`) was seen in a vanilla game
the same day: a copy in `build/vanilla-font-check/game` with the untouched GOG 1.03 exe
(`9C10E045...`), the original `binkw32.dll`, no KMRP or KPM, the Steam game's data linked in,
1280x960 windowed. The main menu, Load Game, the inventory with its tutorial pop-up and an
item's description, the journal, an object's name over it, a conversation line wrapped on
two lines and the reply list (the highlighted reply in the game's yellow) all drew in
Georgia, at the game's own size and place, against a run of the same copy without it. Not
looked at there: the HUD's numbers, the end credits, Menu Large.

## 1. Files

Split by job on 2026-11-08 (audit: the window was one 1,848-line file).

| File | What it is |
| --- | --- |
| `Program.cs` | The start, the one version number (`Program.Version`, which the footer, the file's properties and `build.ps1 -Release` read), embedded files |
| `MainForm.cs` | The window: header, tiles, the bottom row with the Version box, input, what the tiles do, starting a conversion |
| `Dialogs.cs` | The pop-ups drawn in the window: a message, Replace, Settings, the face picker |
| `Theme.cs` | Colours, fonts, and text drawn at the device's own pixels (section 3) |
| `Tiles.cs` | The eleven tiles and their example lines |
| `Places.cs` | `settings.txt`; finding the game, its Override folder and whether KMRP is installed |
| `Conversion.cs` | Writing a set of fonts into a folder, all or nothing (section 2) |
| `Atlas.cs` | Glyph rasteriser, the atlas and TXI writer, for both versions |
| `FontSource.cs` | Reading font files through DirectWrite, by vtable; WOFF read here; Windows' own hinted drawing of small letters |
| `build.ps1` | Builds `dist\KMRP Font Converter.exe` from this folder alone; `-Refresh` remakes `data/`; `-Release` the release zip (section 6) |
| `kmrp-logo.png`, `kotor-logo.png` | The logo, `tools/build_brand_image.py --text KMRP --tracking 0.12 --whole-crest`, 714x535 made 400x300; the same with `--text KOTOR`, at the same scale, 486x302 (2026-11-08). The KMRP one remade this way is pixel for pixel the committed file |
| `make_data.py`, `data/` | Makes `data/`: `references.txt`, `examples.txt` and the example PNGs, from KMRP's `assets/hd-original-fonts`; committed, so the converter builds without the rest of KMRP |
| `favicon.ico` | The window icon, a copy of `src/patcher/favicon.ico` |
| `settings.png` | The Settings button's gear: the installer's `src/patcher/icons/Settings.png` cropped to its ink and made 512 px, drawn in one colour from its alpha |
| `fonts/` | The window's text: KMRP Original HD Menu (titles) and Descriptions (text), TrueType copies from `fonts/KMRP-OriginalHD.ttc`'s build (GDI+ cannot load the OTF files' CFF outlines), SIL OFL 1.1, with `OFL.txt`; compiled in and loaded from memory |
| `release/README.txt`, `release/THIRD-PARTY-NOTICES.txt` | The player's readme and the notices, put in the release zip with `LICENSE.txt` (KMRP's GPL) and `OFL.txt` |
| `tests/Test-FontConverter.py` (this repository's root) | Builds and runs it (section 7) |

*Removed 2026-11-08:* `FontConverter.cs` (split into the files above), and `logos.html`,
`logo-github.png`, `logo-deadlystream.png`, the start pop-up's two marks, with the pop-up
(section 4).

## 2. What it writes

Eleven tiles, one for each distinct letter set of the game's 17 text fonts (the sets
`tools/build_hd_original_otf.py` finds by comparing every glyph; `fnt_galahad14` is
left out, it is decorative art KMRP only enlarges). A tile's font is written to every font
name of its set:

| Tile | Group | Font names |
| --- | --- | --- |
| Menu | Menus | dialogfont16x16, dialogfont16x16b |
| Menu Small | Menus | dialogfont10x10, dialogfont10x10a, dialogfont16x16a |
| Menu Small Bold | Menus | dialogfont10x10b |
| Menu Medium | Menus | dialogfont12x16 |
| Menu Large | Menus | dialogfont32x32 |
| Dialogue | Reading | fnt_d16x16, fnt_d10x10b |
| Replies | Reading | fnt_d16x16b |
| Journal | Reading | fnt_d16x16a, fnt_creditsa, fnt_dialog16x16 |
| HUD | HUD | fnt_console |
| Credits | Credits | fnt_credits |
| Credits Bold | Credits | fnt_creditsb |

**Two versions** (2026-11-08, the maintainer: "an option to create override files for the
original KOTOR and one for KMRP"), chosen in the Version box beside Convert:

| | KMRP | Original Game |
| --- | --- | --- |
| Size | The Original HD Font's: its TXI header (fontheight, baselineheight, spacing, numchars) unchanged, six times the original fonts | The original fonts': fontheight, baselineheight, spacingR and spacingB divided by six. Each divides exactly (10, 12, 14, 16, 19 and 32 pixels), and `Test-FontConverter.py` finds them equal to the game's own TXI values |
| Letters | Drawn here from the outlines, exact-area coverage | Drawn by Windows with the font's own hinting (`LoadedFace.Hinted`: DirectWrite's GDI-classic rendering, greyscale through `IDWriteFactory2::CreateGlyphRunAnalysis`, slot 30), each letter standing on a whole pixel, so text of 10 to 19 pixels is crisp. Where that is missing, ClearType samples averaged (`IDWriteFactory`, slot 23); where neither works, the outlines unhinted, every letter alike |
| Atlas | Starts at the HD atlas's width, doubles until the letters fit | Starts at 64, doubles: a power of two like the game's own |
| In the game | KMRP fits each to its drawn size and offers them as **Modded Font** ([ingame-text-options.md](https://github.com/RayesDiyab/KMRP-KOTOR-Modern-Restoration-Patch/blob/master/docs/ingame-text-options.md) section 3) | The game draws them one texel to one pixel, as its own: the same size as the game's text, which is small on a modern screen |

*Corrected 2026-11-08, the same day:* the first original-game build took ClearType's
texture; its filter pads each glyph a pixel each side, and at 10 pixels Georgia's m stood
apart ("Com bat"). Greyscale has no padding.

The rules are `tools/build_font_from_ttf.py`'s, measured from the game's atlases:
Windows-1252 slots, no zero-wide glyph (a slot the font lacks keeps the HD Original glyph's
width, divided for the original game, and stays blank), ink moved inside its cell, 4 pixels
between cells. The glyph box holds the font's ascent and descent or, where a letter of the
game's text reaches further, that letter (Bahnschrift's caron over S and Z was cut off at
the ascent). `fnt_console` gets the game's black ring, one original pixel wide.

**Refused** (2026-11-08, audit F4 to F6): a font without every letter A to Z and a to z
(its text would be invisible in the game), with a message naming the missing letters; a
letter or a set that needs an atlas over 4096, KMRP's own largest; a font file over
128 MB, and a WOFF whose tables claim more than that or more than its own header says.

**All or nothing** (`Conversion.cs`, 2026-11-08, audit F1 to F3). Every font is made in a
work folder inside the output folder first. Only when all are made, and no file to be
replaced is read-only, do they move into place, one rename each; a `.tpc` of the same name
(which the game would prefer to the new `.tga`) is renamed `*.tpc.before-font-converter`,
or `-2`, `-3` when an earlier one is there. The work folder is deleted however it ends.
Closing the window while converting stops it, with nothing written, then closes. Until then
each font was written straight to its name, so a failure half way left a mixed set.

Measured on the maintainer's PC (2026-11-08, `Test-FontConverter.py`): all 34 files in 1.0
to 1.1 s for KMRP and 0.5 to 0.7 s for the original game. Atlases are uncompressed 32-bit
TGA: KMRP's 16 MB at 2048 square (273 MB for the 17 with Georgia), the original game's
256 or 512 square.

## 3. The window

**No pop-up at the start, Done as a green button** (2026-11-08, the maintainer: "remove the
pop-up, needs KMRP, and remove the download button", "keep two", "done should be like a
green button"). The program opens on its window. Convert turns green, "Converted" with a
tick drawn as two strokes (KMRP's fonts have only the game's letters, and under Wine no
other font may lend a tick), until a font, a tile, the folder or the version changes. The
pop-ups left: a message (a font that can't be read or used, fonts that could not be
written, a folder that can't be opened) and "Replace fonts already there?".

**The Version box** (2026-11-08, the maintainer's design over five mock-ups): between Clear
and Convert, its own rounded box like the folder's; "Version" in the window's yellow
(`Theme.Gold`, as MENUS and the count), then KMRP and Original Game, each with a round radio
ring in KOTOR's own GUI blue (0, 168, 250, the vanilla `.gui` TEXT COLOR) as KMRP's Text
Size and Font page draws them in the game. The choice is kept in `settings.txt`
(`version=`); with none kept, KMRP when the output folder's game has KMRP installed
(`patches\kmrp.dll` or the `KMRP` folder) or, with no game there, when any game found has
it; else the original game. With Original Game chosen the logo's word is KOTOR, and the
title moves right by its extra width (33 pixels).

**Text: KMSE's sizes, at the device's pixels** (2026-11-08, the maintainer: "like kmse has
crisp fonts in the program, make the Font Converter have the same crispy and large fonts").
Measured beside KMSE at 1616x939: its ordinary text was 14.4 pixels against KMSE's 18. Now:

| | Until 2026-11-08 | Since |
| --- | --- | --- |
| Text Size steps (Small, Medium, Large) | 1, 1.15, 1.3 | KMSE's: 1, 1.2, 1.32 (Medium the default) |
| A tile's use line, a group's line | 12.5, in the dim colours | 14, Muted and Soft |
| The folder's path, SAVE TO | 13, 11 | 14, 12.5 |
| Pop-up text | 15, 13.5 | 16, 15 |
| Faint, Soft (dark mode) | 122,138,154; 143,163,181 | 138,155,172; 158,177,194 |
| How text is drawn | GDI+ through the canvas's scale transform, its lines wherever the layout put them | The transform taken off: the font made at its size on the screen, each line on a whole pixel (`Theme.Draw`, `DrawAt`, `DrawWrapped`) |

Checked by screenshots at 1600x900: Medium dark with nothing picked and with three fonts
(Original Game, KOTOR logo), Large dark after converting (the green button), Small light.
At Small the introduction's second line holds one word.

**Window size.** Everything is laid out on a 1600x900 canvas scaled evenly to the window.
Dragging an edge keeps the inside 16:9 (`WM_SIZING`), so the canvas always fills it;
maximized on a screen of another shape it is centred and the header's glow reaches the
window's edges. The window opens at the screen's scale as large as fits 94% of the work
area's width and 92% of its height; the smallest is three quarters of the canvas at the
screen's scale (1200x675 at 100%). Each image is kept at one size and made again when the
size changes, stretched while an edge is dragged and sharp again when it is let go.
Measured on 2026-10-21 by sending the window `WM_SIZING`: every edge and corner gave a 16:9
inside (1484x835, 2064x1161, 1984x1116, 1384x778), and `--size=1000x560` opened at the
minimum, 1200x675.

**Settings** (2026-10-22). The gear at the header's top right opens a pop-up with Text Size
and Appearance (Dark, Light). Both apply at once and are kept in `%APPDATA%\KMRP Font
Converter\settings.txt` (`textsize=`, `theme=`, with `output=` and `version=`). Light mode
uses the Save Editor's light colours; the example boxes keep their dark backing in both,
because they show the game's own coloured text. Every edge was raised to at least 1.8:1
against the fill it sits on (WCAG contrast, measured 2026-10-22).

**Font files.** DirectWrite reads TrueType and OpenType (both outline kinds) and
collections; WOFF2 is unpacked by `IDWriteFactory5::UnpackFontFile` (Windows 10 1703 and
later), a variable font's named instances come from its `fvar` table and are made with
`IDWriteFactory6::CreateFontResource` (Windows 10 1809 and later). WOFF 1.0 is inflated by
the converter itself into `%TEMP%\KMRP Font Converter`, emptied when the program starts and
ends (until 2026-11-08 every one stayed). Other types (Type 1, `.fon`, `.dfont`, `.eot`,
SVG fonts) are refused: "This font type can't be used".

Slot numbers, counted from the 10.0.26100.0 SDK headers and checked again by the audit:
`IDWriteFactory` 3 to 23 (`CreateFontFileReference` 7, `CreateFontFace` 9,
`CreateGlyphRunAnalysis` 23); `IDWriteFactory2::CreateGlyphRunAnalysis` 30;
`UnpackFontFile` 47; `CreateFontResource` 49; `IDWriteFontFace::GetMetrics` 8,
`GetGlyphIndices` 11, `TryGetFontTable` 12, `GetGlyphRunOutline` 14;
`IDWriteFontResource::CreateFontFace` 13; `IDWriteGlyphRunAnalysis::GetAlphaTextureBounds`
3, `CreateAlphaTexture` 4.

**Linux with Proton (untested).** The program is meant to run under Wine/Proton: .NET
Framework comes from wine-mono, DirectWrite from Wine's `dwrite.dll`. Where Bahnschrift,
Segoe UI or Consolas are missing it falls back to Tahoma, Arial or the system sans; the dark
title bar is skipped where DWM is missing.

## 4. KMRP and the original game

KMRP sizes Override fonts for the resolution and text size in use, and since 2026-11-08
fits them to their drawn size and offers them as **Modded Font** on its Text Size and Font
page, chosen by itself when new fonts appear in Override
([ingame-text-options.md](https://github.com/RayesDiyab/KMRP-KOTOR-Modern-Restoration-Patch/blob/master/docs/ingame-text-options.md) section 3). A KMRP before 1.6.0
draws fonts made for KMRP six times too large; the readme says 1.6.0 or later.

Until 2026-11-08 a pop-up at every start said the fonts need KMRP, with buttons to KMRP's
Deadly Stream page and GitHub releases, carrying their marks. With the original game as a
version of its own, KMRP is no longer needed, and the maintainer had the pop-up and both
buttons removed. That also ended the audit's question of the Deadly Stream mark (Darth
Nihilus' mask), which had no recorded permission.

*Audited 2026-11-08* against what KMRP and the game read: the 17 names are every text font
the game has; a Georgia conversion matched KMRP's own HD fonts in every TXI header field,
character count and cell height, with no cell outside its atlas and none zero wide; and
`make_data.py` run again gave `data/` byte for byte.

## 5. Command line

For screenshots, checks and the regression test; none of these is remembered:

| Argument | What it does |
| --- | --- |
| `key=path[#face name]` | A tile's font (keys as in `Tiles.All`), as if chosen: the same checks and messages |
| `"--pick key=path"` | The same through the file's own path, opening the picker for a collection (one argument, quoted) |
| `--out=folder` | Write there |
| `--version=kmrp`, `--version=original` | Make the fonts for that version |
| `--textsize=small|medium|large`, `--theme=dark|light` | The look |
| `--convert` | Convert when the window shows; no Replace question with `--exit` |
| `--exit` | With `--convert`: close when done; exit code 0, or 1 when nothing was converted or it failed |
| `--size=WxH` | The window's inside at that size (ignored when it is not two numbers) |
| `--still` | Ignore the mouse (screenshots) |
| `--settings` | Open Settings |

## 6. Building and the release

`powershell -ExecutionPolicy Bypass -File font-converter\build.ps1 -Release` builds the exe
and `dist\KMRP-Font-Converter-<version>.zip` holding it, `README.txt`, `LICENSE.txt` (KMRP's
GPL-3.0), `OFL.txt` and `THIRD-PARTY-NOTICES.txt` (audit B3). The compiler is Roslyn's
`csc` from Visual Studio or its Build Tools (found with `vswhere`) with `/deterministic`, so
a rebuild gives the same exe (audit R2; checked 2026-11-08: two builds, one SHA-256), else
the .NET Framework's, which warns that the hash will differ.

| Build | Bytes | SHA-256 |
| --- | ---: | --- |
| 1.0.0 exe, 2026-11-08, sources uncommitted | 1,113,088 | `30643C4AA241B1B19B14052E21D6D32E99A34E2FDB7EF4BF9858131F15032526` |
| its zip, `KMRP-Font-Converter-1.0.0.zip` | 791,587 | `279E88BCE48157900BB304B313AB9C7D5D4BC8350BA7AA5E1D9F5166E8D3965F` |

The zip holds file times, so it differs at every build; the exe inside does not. The
release build's identity is recorded here when it is made from committed sources.

## 7. Verifying

`python tests/Test-FontConverter.py` builds the exe and runs it (2026-11-08:
all checks passed): seven inputs (TTF, CFF OTF, a collection's face, a variable font,
Georgia, WOFF, WOFF2) for both versions, each set checked (all 34 files; TGA type, depth,
length and size; `texturewidth`; every cell inside the atlas, none zero wide; no comma; the
heights against the HD font's, or a sixth of them and the game's own for the original game;
a power-of-two atlas there; ink in the dialogue font's A; no work folder left); random
bytes, an empty file, a font with digits only and a WOFF claiming 2 GB refused with nothing
written; a `.tpc` set aside earlier kept and the next one set aside as `-2`; a read-only file
stopping the conversion with nothing changed; and the window closed while converting,
leaving nothing.

By hand, before a release: open it on a PC where KMRP is installed and on one without (the
Version box starts on KMRP and on Original Game); choose a font for a tile, switch the
version (the logo reads KOTOR and back), convert into the game's Override and see the green
button; in the game, KMRP's Text Size and Font page shows Modded Font chosen, and without
KMRP the text is in the new font at the game's size.

## 8. Deliberately not changed

- The canvas layout and the tiles: the approved mock-up's.
- The boxes round the examples: the audit asked whether they break "no boxes around
  examples"; they show the game's dark text backing and stay (recorded 2026-10-22).
- The rasteriser for KMRP's version: exact-area coverage at six times, which KMRP fits down.
- Per-monitor DPI: the window is system-DPI aware only, so on a second screen of another
  scale it is stretched by Windows.
