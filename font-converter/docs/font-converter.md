# KMRP Font Converter

> **Documentation standard.** This document follows
> [`docs/documentation-standard.md`](../../docs/documentation-standard.md) (KMRP's, imported
> 2026-11-09). Read it before editing
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
below). Georgia made for the original game (1.0.0 as built then, `30643C4A...`; the build
since differs only in the window's bottom row, section 3, and `Test-FontConverter.py` finds
what it writes unchanged) was seen in a vanilla game
the same day: a copy in `build/vanilla-font-check/game` with the untouched GOG 1.03 exe
(`9C10E045...`), the original `binkw32.dll`, no KMRP or KPM, the Steam game's data linked in,
1280x960 windowed. The main menu, Load Game, the inventory with its tutorial pop-up and an
item's description, the journal, an object's name over it, a conversation line wrapped on
two lines and the reply list (the highlighted reply in the game's yellow) all drew in
Georgia, at the game's own size and place, against a run of the same copy without it. Not
looked at there: the HUD's numbers, the end credits, Menu Large.

**Rewritten in Rust on 2026-11-09** (the maintainer: "rewrite in rust, create windows, macos
and linux (intel and silicon) appimage version"). One source now builds the Windows exe, a
macOS app for Apple silicon and Intel, and Linux AppImages for x86_64 and aarch64. Fonts are
read and the original game's letters hinted by the program itself instead of Windows'
DirectWrite. The build this document describes from here on: 1.0.0, Windows exe 6,514,176
bytes, SHA-256 `6D12031699CC1DDEE5CEA50337A5A7F42791B21207D8F1C3D666107FF19940CB`. What
changed in what it writes is measured in section 9. **Not yet seen in the game:** any font
the Rust program wrote; the game sightings above are of the C# program's fonts. **Not yet
run:** the macOS app (no Mac here), the Linux window and the aarch64 AppImage.

## 1. Files

Split by job on 2026-11-08 (audit: the window was one 1,848-line file).

| File | What it is |
| --- | --- |
| `Cargo.toml`, `Cargo.lock` | The Rust program: its one version number (`version`, which the footer, the exe's properties, the macOS app's and every package's name read), its crates, pinned |
| `src/main.rs` | The start, the command line, `--convert --exit` without a window, embedded files |
| `src/app.rs` | The window: header, tiles, the bottom row with the Version box, the pop-ups (a message, Replace, Settings, the face picker, the missing-letters pop-up), input, fitting the window to the screen, starting a conversion |
| `src/theme.rs` | Colours and the window's fonts |
| `src/tiles.rs` | The eleven tiles and their example lines |
| `src/places.rs` | `settings.txt`; finding the game, its Override folder and whether KMRP is installed |
| `src/conversion.rs` | Writing a set of fonts into a folder, all or nothing (section 2) |
| `src/atlas.rs` | Glyph rasteriser, the atlas and TXI writer, for both versions; its unit tests |
| `src/font.rs` | Reading font files (skrifa), WOFF read here, WOFF2 through wuff; the hinted letters of the original game's version |
| `build.rs`, `.cargo/config.toml` | Lists the example pictures for the program; puts the icon and version in the Windows exe; build output in `build/`, `/Brepro` for a deterministic exe |
| `build.ps1`, `build-macos.sh`, `build-linux.sh` | The Windows exe and zip, the macOS app, DMG and zip, the Linux AppImages (section 6) |
| `icon.png` | The macOS and Linux icon, 512x512, made by `make_icon.py` with `favicon.ico` |
| `kmrp-logo.png`, `kotor-logo.png` | The logo, `tools/build_brand_image.py --text KMRP --tracking 0.12 --whole-crest`, 714x535 made 400x300; the same with `--text KOTOR`, at the same scale, 486x302 (2026-11-08). The KMRP one remade this way is pixel for pixel the committed file |
| `font-logo.png` | The repository README's logo, not built into the program: the same tool with `--text FONT`, at full size, 677x539 (2026-11-09) |
| `release/make_press_kit.py`, `release/press-kit/` | The press kit's five pages, in the converter's own style, from screenshots kept out of the repository (its README says where each came from) |
| `make_data.py`, `data/` | Makes `data/`: `references.txt`, `examples.txt` and the example PNGs, from KMRP's `assets/hd-original-fonts`; and since 2026-11-09 `originals.txt`, the original game's 17 fonts' atlas widths and TXI headers, from a built KMRP checkout's `build/native-runtime/original-fonts` (numbers only, no atlas or letter of the game's). Committed, so the converter builds without the rest of KMRP. Run again on 2026-11-09, it gave the other files byte for byte |
| `favicon.ico`, `make_icon.py` | The program's icon, compiled into the exe (`/win32icon`); the window takes it from there. Since 2026-11-09 the sigil above FONT, made by `make_icon.py` from `font-logo.png` centred on a transparent square, at 16, 24, 32, 48, 64, 128 and 256 pixels (the maintainer: "make the app icon be the sigil with FONT below it like in the readme"). Until then a copy of KMRP's installer icon, `src/patcher/favicon.ico` (370,070 bytes; the new one is 97,129, which made the exe 272,896 bytes smaller). FONT reads from 24 pixels up; at 16 it is the shape of the word |
| `settings.png` | The Settings button's gear: the installer's `src/patcher/icons/Settings.png` cropped to its ink and made 512 px, drawn in one colour from its alpha |
| `fonts/` | The window's text: KMRP Original HD Menu (titles) and Descriptions (text), TrueType copies from `fonts/KMRP-OriginalHD.ttc`'s build (GDI+ cannot load the OTF files' CFF outlines), SIL OFL 1.1, with `OFL.txt`; compiled in and loaded from memory |
| `release/README.txt`, `release/THIRD-PARTY-NOTICES.txt` | The player's readme and the notices, put in the release zip with `LICENSE.txt` (this repository's `LICENSE`, GPL-3.0) and `OFL.txt` |
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
| Size | The Original HD Font's: its TXI header (fontheight, baselineheight, spacing, numchars) unchanged, six times the original fonts | The original game's own fonts': each TXI header is the game's line for line (`data/originals.txt`, since 2026-11-09), only `texturewidth` set to the atlas, with the game's CR LF line ends. The heights are 10, 12, 14, 16, 19 and 32 pixels, the HD sizes divided by six. Until 2026-11-09 the HD header with fontheight, baselineheight, spacingR and spacingB divided by six, which differed from the game's in one value: `fnt_console`'s spacingR, 0.01 against the game's 0.02 |
| Letters | Drawn here from the outlines, exact-area coverage | Hinted by the font's own instructions (`LoadedFace::hinter` in `src/font.rs`: skrifa's TrueType and PostScript hinting, its automatic hinter for a TrueType font with none, FreeType's normal target), made 0.3 pixels wider sideways (`embolden_x` in `src/atlas.rs`, section 9), then filled by the converter's own rasteriser, each letter standing on a whole pixel, so text of 10 to 19 pixels is crisp. Where a font's hints cannot be run, the outlines unhinted, every letter alike. Until 2026-11-09 drawn by Windows (DirectWrite's GDI-classic greyscale); section 9 measures the difference |
| Atlas | The size of KMRP 1.6's own HD atlas for that name (1024, 2048 or 4096): the letters in slot order where they fit, else packed tighter (`pack_tight`: the blank slots share one empty cell, the letters widest first); a larger atlas only where neither fits (none of the nine fonts the test converts, 2026-11-09). Until 2026-11-09 it doubled as soon as slot order did not fit | The size of the game's own atlas for that name (128, 256 or 512): slot order, else tight, else (as the game's own HUD font) tight with no gap between cells; a larger atlas only where the letters cannot fit even so (one font in the test: KMRP's own HD Console face's HUD letters, 1,128 pixels of cells where 128 square holds 1,024). Until 2026-11-09 from 64, doubled until the letters fit in slot order, which gave the game's size for 16 of the 17 and 256 for `fnt_console` |
| In the game | KMRP fits each to its drawn size and offers them as **Modded Font** ([ingame-text-options.md](https://github.com/RayesDiyab/KMRP-KOTOR-Modern-Restoration-Patch/blob/master/docs/ingame-text-options.md) section 3) | The game draws them one texel to one pixel, as its own: the same size as the game's text, which is small on a modern screen |

*Corrected 2026-11-08, the same day:* the first original-game build took ClearType's
texture; its filter pads each glyph a pixel each side, and at 10 pixels Georgia's m stood
apart ("Com bat"). Greyscale has no padding. (The C# program; the Rust program has no
ClearType path.)

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

**The bottom row, taller** (2026-11-09, the maintainer from a screenshot: "the version
looks cramped increase the height of the bottom buttons", then "make Version grösser").
On the 1600x900 canvas (`ActionsY`, `ActionsH`, `PaintVersion` in `MainForm.cs` then;
`paint_actions` and `paint_version` in `src/app.rs` since the Rust rewrite, the same numbers):

| | Until 2026-11-09 | Since |
| --- | --- | --- |
| The row (the folder, Clear, the Version box, Convert) | y 770, 68 high | y 760, 86 high: 18 below the groups, which end at 742, as the 14 between them |
| The footer's middle | y 872 | y 876, clear of the taller row |
| "Version" | `Body(13)`, a 16-high line | `Body(15)`, a 19-high line |
| The radio rings | radius up to a row's half less 2 | less 3.5: at Large text the two rings had touched |

Measured on PrintWindow captures at 1600x900 (2026-11-09): at Medium the box is 85 pixels
high, "Version" 11 below its top edge and the lower ring 8 above its bottom edge; the two
rings, each 20 across, 5 pixels clear of each other at Medium and Large and 8 at Small
(each ring's ink measured, not by eye).

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
At Small the introduction's second line holds one word. Since 2026-11-09 egui draws the
text, laid out at its size on the screen and each line put on a whole pixel (`App::draw`,
`draw_at`, `draw_wrapped` in `src/app.rs`); the sizes in the table are unchanged. Checked by
screenshots at 1600x900 (Medium dark with four fonts, Large light with the picker open,
the green Converted button) and 1200x675 (Small, Settings open), each beside the press
kit's capture of the C# window.

**Window size: fitted to the screen, as KMSE** (2026-11-09, the maintainer: "make it
dynamic (like kmse smaller on smaller screens)"). Everything is laid out on a 1600x900
canvas scaled evenly to the window, text, pictures and spacing together; a window of another
shape centres the canvas, and the header's glow reaches the window's edges. The window opens
at 1600x900 at the screen's scale where that fits 94% of the screen's width and 88% of its
height, else as large as does (`App::fit_window` in `src/app.rs`); it can be made as small
as half the canvas (800x450 at 100%), KMSE's floor (its `fit_to_window`), never with a
smallest size larger than the screen. Moved to another screen, or the screen's scale
changed, the smallest size follows and a window larger than the new screen is made to fit it
(`App::follow_monitor`). Pictures are made again at the screen's own pixels whenever the
size changes. Checked by screenshots on 2026-11-09: `--size=1600x900` and `--size=1200x675`
lay out as the C# program's window. **Not checked:** the opening size on a small screen,
and moving the window between screens.

*Superseded 2026-11-09:* the C# window kept its inside 16:9 while an edge was dragged
(`WM_SIZING`, Windows only), opened at 94% of the work area's width and 92% of its height,
and was never smaller than three quarters of the canvas (1200x675 at 100%); measured on
2026-10-21: every edge and corner gave a 16:9 inside (1484x835, 2064x1161, 1984x1116,
1384x778). The Rust window has no 16:9 drag (winit offers none on every system); the canvas
is centred instead.

**Missing letters: a badge and a pop-up** (2026-11-09, the maintainer: the warning "isn't
really shown fully"; "number glyphs missing, and if you hover over it ... a pop-up with more
info"). A picked font that lacks letters of the game's text (they are left blank in the
game) shows a badge at its chip's right, beside the cross: a warning sign drawn as a
triangle, and how many. Pointing at it opens a pop-up: "8 letters missing", the letters
themselves, and that the game's text leaves them blank. Until then "N missing" followed the
font's name and cut it short: with Arial lacking 8 letters, the Dialogue tile showed "A...".
Checked by a screenshot at 1600x900 with the cursor on the badge: Arial with 8 of its
letters taken out by a fontTools subset (the Windows-1252 bytes 0x80, 0x8A, 0x8C, 0x8E,
0x9A, 0x9C, 0x9E, 0x9F), its name shown in full and the pop-up listing the 8.

**Settings** (2026-10-22). The gear at the header's top right opens a pop-up with Text Size
and Appearance (Dark, Light). Both apply at once and are kept in `settings.txt` in the
system's settings folder (`textsize=`, `theme=`, with `output=` and `version=`):
`%APPDATA%\KMRP Font Converter` on Windows, as before, `~/Library/Application Support/KMRP
Font Converter` on macOS and `~/.config/KMRP Font Converter` on Linux. Light mode
uses the Save Editor's light colours; the example boxes keep their dark backing in both,
because they show the game's own coloured text. Every edge was raised to at least 1.8:1
against the fill it sits on (WCAG contrast, measured 2026-10-22).

**Font files** (since 2026-11-09, `src/font.rs`). skrifa, the fontations project's reader
and scaler, reads TrueType and OpenType (both outline kinds, CFF and CFF2), collections
(`.ttc`, `.otc`) and variable fonts, one face per named instance (its `fvar` table, as
before); WOFF2 is unpacked in memory by wuff; WOFF 1.0 by the converter's own code, the C#
program's ported with its limits. Nothing is written to a temporary folder any more. The
metrics are DirectWrite's: the em, and the ascent and descent from OS/2's `usWinAscent` and
`usWinDescent`, or its typo values where the font sets `USE_TYPO_METRICS`, else `hhea`'s.
A face's name is chosen from its `name` table as before (Windows US English first). A font
of bitmaps only has no outlines to draw and is refused, as DirectWrite's was. Other types
(Type 1, `.fon`, `.dfont`, `.eot`, SVG fonts) are refused: "This font type can't be used".

*Superseded 2026-11-09:* until then Windows' DirectWrite read every file: TrueType,
OpenType and collections through `IDWriteFactory`; WOFF2 through
`IDWriteFactory5::UnpackFontFile` (Windows 10 1703 and later); named instances through
`IDWriteFactory6::CreateFontResource` (Windows 10 1809 and later); WOFF 1.0 inflated by the
converter into `%TEMP%\KMRP Font Converter`, emptied when the program started and ended.
Slot numbers, counted from the 10.0.26100.0 SDK headers and checked again by the audit:
`IDWriteFactory` 3 to 23 (`CreateFontFileReference` 7, `CreateFontFace` 9,
`CreateGlyphRunAnalysis` 23); `IDWriteFactory2::CreateGlyphRunAnalysis` 30;
`UnpackFontFile` 47; `CreateFontResource` 49; `IDWriteFontFace::GetMetrics` 8,
`GetGlyphIndices` 11, `TryGetFontTable` 12, `GetGlyphRunOutline` 14;
`IDWriteFontResource::CreateFontFace` 13; `IDWriteGlyphRunAnalysis::GetAlphaTextureBounds`
3, `CreateAlphaTexture` 4.

**macOS and Linux** (2026-11-09). The window is egui's (eframe, OpenGL through glow), the
same on every system; its text is set in the two KMRP fonts, with egui's own fonts for the
signs they lack and for the folder's path (Hack). The file pickers are the system's own (rfd:
Windows' dialogs, macOS's panels, and on Linux the desktop's portal). The tick, the clear
cross and the warning sign are drawn as strokes, as no font the window carries has them
(seen 2026-11-09: the cross and the warning sign came out as empty boxes when drawn as
letters). On Linux the AppImage opens OpenGL (or EGL), X11 or Wayland and xkbcommon from the
system. Under WSL's Ubuntu 20.04 (2026-11-09), which has neither xkbcommon nor Mesa, the
window did not open: without xkbcommon the program stopped in winit (`XKBNotFound`); with
xkbcommon unpacked beside it, it said "The window could not be opened" (no OpenGL
configuration). `--convert --exit` converted there. So the Linux window is **not yet seen**. Where the game is found on macOS and Linux: Steam's library
folders (`~/Library/Application Support/Steam`; `~/.steam/steam`, `~/.local/share/Steam`
and Flatpak's), where Proton keeps the Windows game with its `swkotor.exe`; and the
folder beside the AppImage or the macOS app.

*Superseded 2026-11-09:* the C# program was meant to run under Wine or Proton (never tried),
with .NET Framework from wine-mono and DirectWrite from Wine's `dwrite.dll`.

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
| `key=path[#face name]` | A tile's font (keys as in `src/tiles.rs`), as if chosen: the same checks and messages |
| `"--pick key=path"` | The same through the file's own path, opening the picker for a collection (one argument, quoted) |
| `--out=folder` | Write there |
| `--version=kmrp`, `--version=original` | Make the fonts for that version |
| `--textsize=small|medium|large`, `--theme=dark|light` | The look |
| `--convert` | Convert when the window shows; no Replace question with `--exit` |
| `--exit` | With `--convert`: no window at all (since 2026-11-09; the C# program opened it and closed it when done); the messages go to the standard error; exit code 0, or 1 when nothing was converted or it failed |
| `--size=WxH` | The window's inside at that size, in the screen's pixels (ignored when it is not two numbers) |
| `--still` | Ignore the mouse (screenshots) |
| `--settings` | Open Settings |

## 6. Building and the release

Since 2026-11-09 three scripts in `font-converter/`, each running the unit tests first
(`cargo test --profile tests`, in its own `build/target-tests`), building with `--locked`,
and checking what it made. Every package holds the program, `README.txt`, `LICENSE.txt`
(this repository's GPL-3.0), `OFL.txt` and `THIRD-PARTY-NOTICES.txt` (audit B3). The macOS
and Linux scripts follow KMSE's own.

| Script | Where it runs | Makes, in `dist/` | Its checks |
| --- | --- | --- | --- |
| `build.ps1 [-Release]` | Windows, Rust with the MSVC toolchain | `KMRP Font Converter.exe`; `KMRP-Font-Converter-<v>-Windows-x64.zip`, the files at its root | the exe's SHA-256 printed |
| `build-macos.sh [--arm64-only]` | macOS, Xcode's command line tools | `KMRP Font Converter.app` (Universal 2: `lipo` of the aarch64 and x86_64 builds, macOS 11 or later, ad hoc signed unless `KMRP_SIGN_IDENTITY`, notarized with `KMRP_NOTARY_PROFILE`); `KMRP-Font-Converter-<v>-macOS.dmg` (with a link to Applications) and `.zip` | both architectures in the app, only the system's libraries loaded, no path of the Mac in it, a headless conversion for both versions |
| `build-linux.sh [--arch x86_64\|aarch64] [--zig] [--pack-only]` | Linux or WSL | `KMRP-Font-Converter-<v>-Linux-<arch>.AppImage`, packed by appimagetool 1.9.1 (its SHA-256 checked against GitHub's: `ed4ce84f...` x86_64, `f0837e74...` aarch64) | only glibc and libgcc linked, the glibc it needs, no home path, a headless conversion for both versions where the architecture is the machine's |

The Windows exe is deterministic: the linker writes no time stamp (`/Brepro`,
`.cargo/config.toml`) and the sources' folders become fixed names
(`--remap-path-prefix`), so the same sources give the same bytes (audit R2; checked
2026-11-09: a rebuild after `cargo clean -p kmrp-font-converter` gave the same SHA-256).
`.github/workflows/build.yml` runs the unit and regression tests on Windows, macOS and
Ubuntu 22.04 on every push to master, and, run by hand or by a `v*` tag, the four packages:
Windows x64, macOS Universal 2, and the AppImages on Ubuntu 22.04 for x86_64 and on its ARM
runner for aarch64. **Not yet run:** the workflow (nothing has been pushed).

The packages made on 2026-11-09 on the maintainer's Windows PC. The Linux programs were
built there with `cargo zigbuild` for glibc 2.31 (as `build-linux.sh --zig` does, with the
same path mapping) and packed with `build-linux.sh --pack-only` under WSL's Ubuntu 20.04;
appimagetool's AppImage runtime for aarch64 was put in `build/tools` from the type2-runtime
release's `continuous` tag (SHA-256 `9a72a744...`, GitHub's digest), as GitHub answered 504
to the download from WSL. The macOS app was not made: that needs a Mac.

| Package (1.0.0, Rust) | Bytes | SHA-256 |
| --- | ---: | --- |
| `KMRP Font Converter.exe` | 6,514,176 | `6D12031699CC1DDEE5CEA50337A5A7F42791B21207D8F1C3D666107FF19940CB` |
| `KMRP-Font-Converter-1.0.0-Windows-x64.zip` | 3,616,040 | `852EDCB903A1A5BB86BB76C9A4174F547811B23DCB2D28F5A37192F2DD257F86` |
| `KMRP-Font-Converter-1.0.0-Linux-x86_64.AppImage` | 5,085,688 | `03274539E374BA5EDBA3517ADF6AC8CBBD495F1FABFFC71A38E887CBCC548C69` |
| `KMRP-Font-Converter-1.0.0-Linux-aarch64.AppImage` | 4,766,216 | `2F723C456B7FE88009D24117133AC7ADC8D41E882B4FE087D9BA5D4938E3FA2A` |

The zip and the AppImages hold file times, so they differ at every build; the Windows exe
inside does not. The exe is 7.7 times the C# one's 840,192 bytes: the window's toolkit, the
font reader and the two window fonts are now inside it rather than in Windows.

*Superseded 2026-11-09:* until the Rust rewrite, `build.ps1` compiled the C# sources with
Roslyn's `csc` from Visual Studio or its Build Tools (found with `vswhere`) with
`/deterministic` (checked 2026-11-08: two builds, one SHA-256), else the .NET Framework's,
which warned that the hash would differ, and `-Release` wrote
`dist\KMRP-Font-Converter-<version>.zip`. Those builds:

| Build | Bytes | SHA-256 |
| --- | ---: | --- |
| 1.0.0 exe, 2026-11-08, sources uncommitted | 1,113,088 | `30643C4AA241B1B19B14052E21D6D32E99A34E2FDB7EF4BF9858131F15032526` |
| its zip, `KMRP-Font-Converter-1.0.0.zip` | 791,587 | `279E88BCE48157900BB304B313AB9C7D5D4BC8350BA7AA5E1D9F5166E8D3965F` |
| 1.0.0 exe, 2026-11-09, built from commit `a0cf97f` in this repository | 1,113,088 | `30643C4AA241B1B19B14052E21D6D32E99A34E2FDB7EF4BF9858131F15032526`, the same |
| its zip, 2026-11-09, with the corrected `README.txt` and notices below | 791,603 | `686E47908B0CE63E609FC44468B34B62EC874E92F622D2AD25EE333F32CE8F45` |
| 1.0.0 exe, 2026-11-09, the taller bottom row (section 3) | 1,113,088 | `89B63293648D713F653EEC78B45498F7A4CB5659F87531E70B1ED61D1A214CEE` |
| its zip, the files at its root | 791,343 | `71C366D5B0FD3CBF92BCB5D4A7CE38BB5F4DECE36FD2778B3EA810E136212569` |
| **1.0.0 exe, 2026-11-09, the sigil-and-FONT icon (section 1)** | 840,192 | `81D4BD89357D8377A893559BC8FF94D301C8307C71480FA4A08508C00E6F5722` |
| its zip, the notices naming the icon, the copy in the root `dist\` | 722,680 | `5851FBB6AB10E841646C955C84A3C502E12212D6020EBD70E1D0321F41735D9B` |

The zip holds file times, so it differs at every build; the exe inside does not. The
last C# exe row was to be the 1.0.0 release's until the Rust rewrite replaced it.

*Superseded 2026-11-09, twice:* until the bottom row was made taller, this section said the
`30643C4A...` exe, which commit `a0cf97f` gave byte for byte, was the release's; then
`89B63293...` until the new icon. 1.0.0 was not released yet, so the version stayed 1.0.0
and its exe changed.

*Corrected 2026-11-09:* the zip of 2026-11-08 carried a `README.txt` saying the source is
"in the KMRP repository above, in the font-converter folder" and notices linking to KMRP's
`fonts/` folder; both were written before the move to this repository and not looked at
after it. Both now name this repository. The exe does not carry either text and is
unchanged.

## 7. Verifying

`python tests/Test-FontConverter.py` builds the program and runs it, on Windows, macOS or
Linux (2026-11-09, the Rust exe `6D120316...` on Windows 11: all checks passed; 2026-11-08,
the C# exe: all passed). Nine inputs (Arial TTF, KMRP's CFF OTF, KMRP's collection, KMRP's
TTF, Cambria's collection face, Bahnschrift's variable font, Georgia, and Arial as WOFF and
WOFF2; KMRP's three added 2026-11-09 so that a machine without Windows' fonts has inputs,
and there the WOFF files are made from KMRP's TTF) for both versions, each set checked (all 34 files; TGA type, depth,
length and size; `texturewidth`; every cell inside the atlas, none zero wide; no comma; the
heights against the HD font's, or a sixth of them and the game's own for the original game;
a power-of-two atlas there; ink in the dialogue font's A; no work folder left); random
bytes, an empty file, a font with digits only and a WOFF claiming 2 GB refused with nothing
written; a `.tpc` set aside earlier kept and the next one set aside as `-2`; a read-only file
stopping the conversion with nothing changed; and the window closed while converting,
leaving nothing (Windows only: the check sends `WM_CLOSE`). Each set took 0.3 to 1.2 s
(2026-11-09, Rust: 0.7 to 1.2 s for KMRP, 0.3 to 0.6 s for the original game; the C#
program took 1.0 to 1.1 s and 0.5 to 0.7 s).

*Corrected 2026-11-09:* the close check looked for the program's visible window. Under the
Rust program it found none and failed: eframe shows its window only after the first frame,
in which the conversion starts, and winit keeps a second visible window of its own. It now
finds the window by its title and process. Traced by hand: `WM_CLOSE` during a conversion
of all eleven tiles, the program exited 1 s later with nothing in the folder.

`cargo test` runs the unit tests in `src/atlas.rs`: .NET's half-to-even rounding, the TXI's
numbers rounded as .NET Framework wrote them, the Windows-1252 slots, the rasteriser's
coverage of a square, and the original game's widening (a square of either winding 0.3
wider from the same left edge, no taller).

On Linux and macOS (2026-11-09): the x86_64 Linux program, and the same inside its
AppImage, converted for both versions under WSL's Ubuntu 20.04 (the dialogue tile, 4
files each). Not run there: `Test-FontConverter.py`, the window (section 3), the aarch64
AppImage, anything on a Mac. The macOS code was type-checked for both architectures
(`cargo check --target aarch64-apple-darwin` and `x86_64-apple-darwin`, no errors), not
linked.

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
- The arithmetic of the KMRP version: ported line for line, `Math.Round`'s half-to-even and
  .NET Framework's number formatting with it (section 9).
- The layout's numbers, the colours, the texts and the settings file's keys: the C#
  program's, ported unchanged (2026-11-09).

*Superseded 2026-11-09:* "Per-monitor DPI: the window is system-DPI aware only, so on a
second screen of another scale it is stretched by Windows." winit, under the Rust window,
is per-monitor DPI aware, and the window follows a new screen's scale (section 3); not
checked on two screens.

## 9. The Rust rewrite, measured (2026-11-09)

The Rust program against the C# one (`81D4BD89...`), each converting the same three fonts
for all eleven tiles, both versions, into empty folders; compared file by file with a
script in the scratch folder (TXI text byte for byte, TGA alpha pixel by pixel) and glyph by
glyph (each cell cropped by its TXI coordinates, top-left aligned). Rust build
`6D120316...`, unless a table names another.

**KMRP version: the same arithmetic, in KMRP 1.6's own file sizes.** The TXI files of the
14 fonts outside the Journal tile are the C# program's byte for byte, for all three fonts.
The Journal tile's three (`fnt_creditsa`, `fnt_d16x16a`, `fnt_dialog16x16`) are laid out
otherwise since 2026-11-09 (below): the same header and cell widths, other places, in an
atlas half as wide and high. One cell width differs there, Arial's K, 50 pixels against 51:
its ink reaches past its advance and the cell is the ink's width rounded up (section 2), and
the outline's right edge, read by skrifa rather than DirectWrite, falls on the other side of
a whole pixel. The 14 atlases of the same size differ slightly in their pixels, from the
same outlines read by another reader:

| Font (KMRP version) | TXI files the same (of the 14) | Pixels that differ | Mean difference there (of 255) | Largest |
| --- | ---: | ---: | ---: | ---: |
| KMRP Original HD Dialogue (CFF OTF) | 14 | 463,419 | 1.2 | 13 |
| Georgia | 14 | 707,217 | 3.1 | 45 |
| Arial | 14 | 626,506 | 1.5 | 9 |

*Corrected 2026-11-09:* this table first compared all 17, the Journal's three included, and
for Arial gave 14 of 17 the same and a largest difference of 252, from the cells after K
moving a pixel.

**The file sizes are KMRP 1.6's** (2026-11-09, the maintainer: "the font converter needs to
give the exact size font files for kmrp that kmrp 1.6 currently uses"). KMRP 1.6 packs
`assets/hd-original-fonts` as its HD Font (`kmrfho00..17`, `tools/build_native_assets.py`),
and `data/references.txt` was found to hold those 17 TXI headers and atlas widths unchanged
(compared field by field that day). The converter matched them in 14 of 17: the Journal
tile's three are 1024 square in KMRP, at 84 pixels a letter 11 rows of 88, and KMRP's own
narrow letters fill them (9,516 pixels of cells). Arial's, Georgia's and KMRP's Dialogue
OTF's are 10,763 to 11,453, 1,824 of them the 38 blank slots (control codes, the five
undefined bytes), and did not fit in slot order: the converter doubled them to 2048, a 16 MB
file where KMRP's is 4 MB. Now, where slot order does not fit KMRP's size, the blank slots
share one empty cell, each keeping its own width (the TXI gives each slot its own place, and
places may coincide), and the letters go widest first into the first row with room. The
regression test checks that every KMRP-version atlas is KMRP's size and every header field
KMRP's: so for all nine fonts it converts, every `.tga` is the same size in bytes as KMRP's
(18 bytes of header and 4 a texel). Of the `.txi` files, 14 of 17 are the same size in bytes
as KMRP's for Arial, Georgia and KMRP's Dialogue OTF (measured); the Journal's three are 523
bytes smaller because KMRP's own copies of those three end their lines with CR LF and its
other 14, like the converter's, with LF alone. Text read back from the new
1024 Journal atlases by their TXI coordinates draws as from the 2048 ones (Georgia, Arial and
KMRP's Dialogue OTF, "Find Bastila, then a way off Taris."). A font wide enough not to fit
even so still gets a larger atlas; none of the nine needed one. **Not yet seen in the game.**

**Original-game version: the hinting is the font's, no longer Windows'.** DirectWrite's
GDI-classic drawing is Windows' own and not available elsewhere; the Rust program hints each
letter with the font's own instructions through skrifa and fills it with the converter's
rasteriser. Four of skrifa's targets were tried against the C# program's letters, over the
3,658 cells from slot 33 up in each set:

| Target | Arial: cells of another width, mean difference | Georgia | KMRP Dialogue |
| --- | --- | --- | --- |
| **Normal (FreeType's default), chosen** | 449, 38.5 | 700, 31.4 | 15, 26.6 |
| Light (the same as Normal since FreeType 2.7) | 449, 38.5 | 700, 31.4 | 15, 26.6 |
| Mono (strong hinting for black-and-white) | 530, 53.9 | 995, 66.4 | 15, 26.6 |
| None (the outlines unhinted) | 502, 53.7 | 846, 55.6 | 6, 32.4 |

(The mean difference is over the pixels inked in either, of 255.) *Rejected:* Mono, whose
Georgia was visibly jagged at 10 pixels in a side-by-side picture of "Combat Movement
Georgia"; unhinted, which is soft; keeping DirectWrite on Windows alone, which would have
made the fonts differ by the system they were made on. With Normal alone the letters were
a little narrower and lighter than Windows' (GDI-classic widens stems to whole pixels
across); their heights, baselines and the TXI's header fields are the same, as the
regression test checks against the game's own. (This table was measured on the builds of
2026-11-09 before the widening below.)

**Widened, 2026-11-09** (the maintainer: "I would prefer if it was like slightly wider
instead of slightly narrower"). Each hinted letter of the original-game version is made
`ORIGINAL_WIDEN` (0.3 of the game's pixels) wider: `embolden_x` in `src/atlas.rs` moves
every edge out sideways by half of it along the miter of its two neighbours, as FreeType's
`FT_Outline_EmboldenXY` with no vertical strength, then the letter right by the other half,
so its left side bearing stays; its advance is the design advance plus 0.3, rounded. The
KMRP version is not widened (its figures above are unchanged). Measured over A to Z, a to z
and 0 to 9 in six of the fonts (dialogfont10x10, 12x16, 16x16, 32x32, fnt_d16x16,
fnt_d16x16b), against the C# program's, the mean of each:

| Version | Arial: cell, ink width, ink amount | Georgia | KMRP Dialogue |
| --- | --- | --- | --- |
| Normal alone | 0.997, 1.032, 0.982 | 0.985, 0.985, 0.995 | 1.000, 1.006, 0.979 |
| Advances rounded up, no widening | 1.033, 1.032, 0.982 | 1.008, 0.985, 0.995 | 1.046, 1.006, 0.979 |
| Widened 0.25 | 1.036, 1.063, 1.079 | 1.013, 1.013, 1.089 | 1.021, 1.034, 1.050 |
| **Widened 0.3, chosen** (`AB195B7A...`) | 1.042, 1.069, 1.099 | 1.022, 1.021, 1.108 | 1.026, 1.040, 1.064 |
| Widened 0.35 | 1.047, 1.076, 1.118 | 1.027, 1.030, 1.126 | 1.031, 1.049, 1.078 |
| Widened 0.4 | 1.055, 1.087, 1.138 | 1.032, 1.035, 1.145 | 1.033, 1.051, 1.092 |
| Widened 0.6 | 1.092, 1.119, 1.215 | 1.055, 1.055, 1.220 | 1.073, 1.078, 1.148 |

*Rejected:* rounding the advances up, which widened only the spacing and opened gaps
("Com bat", wide word spaces) in a side-by-side picture of Georgia at 10 pixels; 0.4 and
more, visibly bold there; 0.25, which left Georgia barely wider (1.3%). 0.3 makes every
font tried wider than Windows' drew it, by 2 to 4% in spacing and 2 to 7% in the letters,
and 6 to 11% fuller. With it, three of the 17 atlases of KMRP's Dialogue OTF grew from one
power of two to the next (since the change below, they are the game's size again). **Not
yet seen in the game.**

**The game's own sizes, headers and spacing** (2026-11-09, the maintainer: "our original
game font output should match size, etc., and spacing of the original game", the same way
as KMRP's). The game's 17 fonts, as KMRP's Original Font has them (`kmrfog00..16`, which
KMRP's `tools/build_original_fonts.py` copies out of the game's GUI textures), against the
converter's for Arial, Georgia and KMRP's Dialogue OTF:

| | Before | After |
| --- | --- | --- |
| Atlas sizes the game's | 16 of 17: `fnt_console` 256, the game's 128 | 17 of 17 for all three; every `.tga` the same size in bytes as the game's |
| TXI header | the HD header divided by six: the same values but `fnt_console`'s spacingR, 0.01 against the game's 0.02; the game's misspelt `compress texture` (`dialogfont10x10b`) and `compresstexure` (`fnt_creditsb`, which also has no `filter` line) not there | the game's, line for line, misspelt keys and all: 17 of 17 the same text for all three |
| Line ends | LF | CR LF, as the game's |

The game's HUD font is 127 characters in 8 by 16 cells side by side, no gap, 128 square; its
31 control codes share one empty cell. The converter's HUD letters, drawn with their black
ring, are wider (983 pixels of cells for Georgia's 95), and fit 128 only with no gap between
cells, which is the step added before a larger atlas: tight packing with no gap, as the game
packs its own. The game's other fonts leave gaps (a 16-column grid) and are not changed by
this: slot order with 4 pixels between cells still fits them. HUD text drawn back from
Georgia's 128 atlas reads as from its 256 one, no letter reaching into the next. KMRP's own
HD Console face cannot fit: its HUD letters need 1,128 pixels where 128 square holds 1,024,
so its `fnt_console` is 256, and the test allows a larger atlas only where it computes the
letters cannot fit. Where the game's fonts' `.txi` text came from: KMRP's copies end their
lines CR CR LF, as its script writes the game's CR LF text through Python's text mode on
Windows; the converter writes CR LF. **Not yet seen in the game.**

**What else changed with the rewrite**, all in sections 1, 3, 5 and 6: the window is egui's
(the same layout; the 16:9 drag gone; fitted to the screen as KMSE); `--convert --exit`
opens no window; WOFF files are unpacked in memory; the program is 6.5 MB instead of 0.8 MB;
macOS and Linux builds.
