# KMRP Font Converter: release audit, 2026-11-08

> **Documentation standard.** This document follows
> [`docs/documentation-standard.md`](https://github.com/RayesDiyab/KMRP-KOTOR-Modern-Restoration-Patch/blob/master/docs/documentation-standard.md): measured claims
> only, every site tabulated, and anything untested labelled as untested.

Audit of `font-converter/` for a public release, at master `c5ae24d` plus the uncommitted
Override-font work of the same day. Nothing was changed by the audit. Three read-only code
reviews (font reading and atlas code; the window and file handling; release, licences and
docs), a run of a freshly built copy over 19 inputs, and a side-by-side look at the KOTOR
Modern Gamesave Editor (KMSE) for the window's text, which the maintainer asked to match.
Each finding says how it is known: **run** (seen when the program ran), **read** (followed
through the code end to end), or **suspected**.

## Verdict

Not ready to release. Four blockers, all about what ships rather than how it converts:
it needs a KMRP that is not released yet, the exe in `dist\` is stale, no licence text
travels with it, and the Deadly Stream mark has no recorded permission. The conversion
itself is sound: every real font tried made all 34 files, well formed. The defects that
matter are what happens when something goes wrong half way (files left in Override) and
input that is hostile or odd.

## How it was tested

A copy built from today's sources (`build.ps1`, 1,047,552 bytes, SHA-256
`EA1C9D1176FC4A67F92485B44237439D9BA2C988CA1F6880E960AB023BB9C8C5`) converted every tile
from one file with `--no-intro --no-done --convert --out=<empty folder>`. Each result was
checked: all 17 `.tga`/`.txi` pairs, TGA type 2 at 32 bits and full length, `texturewidth`
equal to the atlas width, every cell inside the atlas, no cell zero wide from slot 32 up,
no comma in any number, and ink in the dialogue font's `A`.

| Input | Result |
| --- | --- |
| Arial (TTF); KMRP Original HD Dialogue (OTF, CFF); Cambria and MS Gothic (TTC); Bahnschrift (variable); Gabriola (very tall); Segoe Script; Symbol; Wingdings; Segoe UI Emoji; Arial as WOFF and WOFF2 | all 34 files in 2.4 to 2.7 s, every check passed, largest atlas 4096 (`dialogfont32x32`) |
| An output folder named `Ünïcödé 日本 out` | passed |
| Random bytes, a 4 KB cut of Arial, an empty file, a text file named `.otf`, a missing file, a WOFF whose table claims 2 GB | nothing written; on the command line the error is swallowed (`catch (Exception) { }`, `FontConverter.cs:588`) and Convert then reports "Done: 0 game fonts made from 0 fonts". The window's own Choose path shows "This font can't be read" with the raw .NET message (`FontConverter.cs:1296`, read) |

Not tested: a run under Proton or Wine, a run in the game with each font (only Georgia was
seen in the game, 2026-11-08), a font with no Latin letters, a font that makes the atlas
grow past 4096.

## Blockers

*Settled the same day by the maintainer:* KMRP 1.6.0, the Font Converter and KMSE are
released on the same day, so B1 needs only the pop-up and docs to say "KMRP 1.6.0 or
later"; B2 is a fresh build for the release; B3's licence texts go with it.

| # | Finding | Where | How known |
| --- | --- | --- | --- |
| B1 | **Needs a KMRP that is not released.** Override fonts sized to KMRP's (issue 63) and fitted, with Modded Font, are all under `[Unreleased]`; released 1.5.1 draws these atlases six times too large. The start pop-up only says "need KMRP" and links to pages that serve 1.5.1. Release with or after KMRP 1.6.0, and say "KMRP 1.6.0 or later" in the pop-up, the readme and the docs | `FontConverter.cs:1690`; `CHANGELOG.md` | read |
| B2 | **`dist\` is stale.** The exe (1,035,264 bytes, 2026-10-22 18:26) embeds the window fonts from before the kerning commit `8a6fa87`; a build of today's sources is 1,047,552 bytes | `font-converter/dist` | run |
| B3 | **No licence text travels with the exe**: no GPL text or source offer, no `OFL.txt` for the two embedded fonts (OFL clause 2), no MIT notice for the GitHub mark. Ship a zip with `LICENSE`, `OFL.txt`, a notices file and a readme | `build.ps1`, `fonts/` | read |
| B4 | **The Deadly Stream mark has no recorded provenance or permission.** It is Darth Nihilus' mask, from a picture the maintainer supplied; `THIRD_PARTY_NOTICES.md` has no entry. Get Deadly Stream's permission or use a plain text button | `logo-deadlystream.png`; `docs/font-converter.md` section 1 | read |

## Should fix: conversion and files

| # | Finding | Where | How known |
| --- | --- | --- | --- |
| F1 | **A failure half way leaves a mixed set in Override.** Files are written straight to their final names, `.tga` then `.txi`; if font 5 of 17 fails (a read-only file of another mod, a full disk), fonts 1 to 4 are in place, their `.tpc` files renamed, and the failed name can hold a new `.tga` beside an old `.txi`, which KMRP's Modded Font then pairs. Write into a temporary folder and move everything at the end | `FontConverter.cs:1406`; `Atlas.cs:204` | read |
| F2 | **A second conversion stops at a `.tpc` already set aside.** `File.Move` throws when `*.tpc.before-font-converter` exists, e.g. after a mod put its `.tpc` back | `FontConverter.cs:1409` | read |
| F3 | **Closing the window during a conversion.** The worker's `BeginInvoke` is outside its `try`; after the window is gone it throws on the worker thread and the process ends with an error, or the thread is killed mid-write and a cut `.tga` stays in Override. No `FormClosing` guard | `FontConverter.cs:1416` | read |
| F4 | **The atlas can grow without limit.** `while (Pack(...) == null) size *= 2` has no cap, and a glyph's width is taken from its outline; one odd glyph can make a 16384 atlas (a 1 GB `.tga` in Override), and past 65536 `size * size` overflows. KMRP would then load it into the 32-bit game. Refuse above 4096, KMRP's own largest | `Atlas.cs:182` | read |
| F5 | **A font with no Latin letters makes invisible text.** Missing slots keep the reference width and no ink; only a "N missing" label shows, and Convert goes ahead: a CJK-only or Arabic-only font writes 17 blank atlases and the game's text disappears. Refuse or ask when A to Z or a to z are missing | `Atlas.cs:153`; `FontConverter.cs:834` | read |
| F6 | **WOFF is trusted.** `origLength` is allocated before inflating with no limit or check against `totalSfntSize`; the table directory is not bounds-checked; `offset + compressed` can wrap negative. Ends in a caught exception with a raw message, but a small file can make the program commit gigabytes first | `FontSource.cs:229` | read |
| F7 | `fvar` read without bounds checks; a damaged table refuses a font DirectWrite could read, with a meaningless message | `FontSource.cs:323` | read |
| F8 | Change stays clickable while converting: tiles turn green and "Converted" for a folder that got nothing. `Convert()` has no busy guard of its own | `FontConverter.cs:1093`, `1366` | read |
| F9 | "Open Output Folder" throws a WinForms error box on a bad or removed folder (`CreateDirectory` with no `try`); a malformed `--size=` crashes at start | `FontConverter.cs:1160`, `553` | read |
| F10 | Every WOFF or WOFF2 opened leaves a full font in `%TEMP%\KMRP Font Converter`, never removed | `FontSource.cs:189` | read |

## Should fix: what the window says

| # | Finding | Where |
| --- | --- | --- |
| T1 | The intro says "Anything you don't pick stays as it is". Since Modded Font, a name the player did not convert shows KMRP's HD Font while Modded Font is chosen, so a player on Original Font sees those switch to HD | `FontConverter.cs:701` |
| T2 | The Done note says "Start the game with KMRP to see them" even when the folder is not an Override (the Documents fallback when no game is found, a disc install, likely every Proton prefix). Say to copy them into Override there; name **Modded Font** on KMRP's Text Size and Font page and that the player can switch back there | `FontConverter.cs:1429` (and its copy at `567`) |
| T3 | The Replace prompt says files are replaced, but `.tpc` files are renamed; nothing in the window removes converted fonts again, and no doc says how | `FontConverter.cs:1383` |
| T4 | The unsupported-type message names TTF, OTF and WOFF only, not TTC or WOFF2; read errors show raw .NET text | `FontConverter.cs:1289`, `1298` |
| T5 | Escape on the Done pop-up opens Explorer (it picks the first non-primary button, Open Folder) | `FontConverter.cs:1570` |

## The window's text: large and crisp like KMSE

The maintainer, 2026-11-08: "like kmse has crisp fonts in the program, make the Font
Converter have the same crispy and large fonts". Both windows captured at 1616x939
(`PrintWindow`, the same screen):

| | Font Converter | KMSE |
| --- | --- | --- |
| Text Size steps (Small, Medium, Large) | 1.0, **1.15**, 1.3 | 1.0, **1.2**, 1.32 |
| Ordinary text, Medium | 12.5 to 15 px before the step: a tile's line 12.5 x 1.15 = 14.4 px, the path 13 | 15 to 16 px: a save's area line 15 x 1.2 = 18 px |
| Small labels | 11 to 11.5 px, in the faint colour | 13 to 14 px, near white |
| How text is drawn | GDI+, `AntiAliasGridFit`, on a 1600x900 canvas under `ScaleTransform` to the window | Slint, at the window's own pixels |

So the converter's text is about a fifth smaller at the same setting, much of it in a dim
colour, and it is fitted to the pixel grid at canvas size and then scaled, which softens it
whenever the window is not exactly 1600x900. Matching KMSE means: KMSE's steps (1.0, 1.2,
1.32) and its sizes per kind of text, its text colours, and text drawn at device pixels
(the transform's scale applied to the font size, not to the drawing). The layout must then
be checked at every window size, as section 3 of `docs/font-converter.md` records for the
current one.

## Should fix: release

| # | Finding |
| --- | --- |
| R1 | **No version to speak of**: `AssemblyVersion("1.0.0.0")` and `"1.0"` in two places, never bumped, empty copyright in the file properties. One version, bumped per release, its SHA-256 recorded |
| R2 | **The build is not repeatable**: the Framework's old `csc` has no `/deterministic`, so a rebuilt exe never matches a published hash. Use Roslyn with `/deterministic`, or say that hashes differ |
| R3 | **No CI and no test.** Nothing builds or runs it. The run above (`--convert --out=` and the file checks) can become `tests/Test-FontConverter.py` and a Windows CI job |
| R4 | **No user readme.** `font-converter/README.md` covers building only. Missing: requirements (Windows with .NET Framework 4, KMRP 1.6.0 or later), where files go, Modded Font, how to undo (delete the pairs, rename `*.tpc.before-font-converter` back), that `.tga`/`.txi` already there are replaced without a backup |
| R5 | **No release channel decided**: not in the installer, the main `README.md` or `docs/README.md`. `CHANGELOG.md`'s entry still gives `dist\KMRP Font Converter.exe (727,552 bytes)`; a stale 824,832-byte copy sits in the root `dist\` |
| R6 | **Notices**: `THIRD_PARTY_NOTICES.md:483` says no font file is redistributed, untrue now; add a Font Converter section (two OFL fonts, the Octicons mark under MIT, the Deadly Stream mark, `kmrp-logo.png`, `favicon.ico`, the trademarked names in the example lines) |
| R7 | **Antivirus**: unsigned, empty version fields, writes into `Program Files (x86)\Steam`, renames files, raw COM calls. Sign it (for example SignPath's free OSS programme), fill the version fields, scan on VirusTotal before release and record it as for the AppImage |
| R8 | `docs/font-converter.md`: "Not yet seen in the game" at the top against the Georgia seen on 2026-11-08; no build identity; section 5 lacks `--size=` and `--still`; no "not changed" or "verifying by hand" section as the standard asks |

## Minor

| Finding | Where |
| --- | --- |
| `FontConverter.cs` is 1,848 lines. Seams: `Theme` (77 to 259), tile definitions (261 to 343), `Places` (settings and game detection, 345 to 468), image helpers, `Convert`/`Run` as a UI-free class (where F1 to F3 belong and can be tested), the three pop-ups (1452 to 1846) | `FontConverter.cs` |
| Repaint work: the installed font families are listed on every paint (`Theme.Mono` through `Family`); the face picker keeps a bitmap per pixel height while an edge is dragged | `FontConverter.cs:190`, `1775` |
| COM objects not released on some error paths; a large collection read on the window's thread | `FontSource.cs:82`, `118`, `165` |
| `fnt_console`'s black ring is cut at the top and bottom of the cell | `Atlas.cs` (`Spread`) |
| Fonts with tall declared line metrics come out smaller than others (the box holds ascent and descent), a design choice | `Atlas.cs:118` |
| The dark title bar uses attribute 20 only (missing on Windows 10 1809 to 1909); system DPI awareness only, so a second screen at another scale is blurred | `FontConverter.cs` |
| The `--pick` and face-name command line forms differ from the doc; `g.Clip` region not disposed | `FontConverter.cs:572`, `1754` |

## Decisions for the maintainer

1. The Deadly Stream button: ask Deadly Stream for the mark, or a text-only button.
2. Where it is released: its own GitHub release asset, a Deadly Stream page, beside KMRP's installer, or all three; and whether it waits for KMRP 1.6.0 (B1 says it must, or the pop-up must say 1.6.0).
3. The example boxes: `FontConverter.cs:810` draws a rounded box with an edge round each example, against "no boxes around examples"; section 3 of the doc records them on purpose on 2026-10-22. Keep?
4. Signing: a certificate or SignPath, or unsigned with a VirusTotal record.

## Checked and fine

- The DirectWrite vtable slots, GUIDs, structure layouts and HRESULT checks; COM reference counts on success paths.
- Slot selection (Windows-1252, control codes and undefined bytes blank), no zero-wide cell, the rasterizer's bounds and winding.
- TGA type 2, 32-bit; TXI order, six decimals, culture-invariant numbers, `numchars` equal to the rows.
- The `name` table parse; symbol fonts and WOFF2 converted (run).
- The maintainer's UI rules except decision 3: orange picked, green converted, a pop-up at every start with both marks, no advanced mode, no tagline, the output folder chosen in the window.
- Settings in `%APPDATA%`, non-ASCII paths, Steam and GOG detection, guarded P/Invoke for Wine.
- `dist\` is ignored by git; `data\` is committed and remade byte for byte by `make_data.py`.

## Status after the fixes, 2026-11-08

Fixed the same day (sources uncommitted at the time of writing), with the maintainer's
decisions on the rest. `tests/Test-FontConverter.py` checks the fixes marked
"tested"; the others were checked by reading and screenshots.

| Finding | Now |
| --- | --- |
| B1 an unreleased KMRP | Released with KMRP 1.6.0 the same day (the maintainer); the readme says "KMRP 1.6.0 or later"; the start pop-up is gone |
| B2 stale `dist` | Rebuilt; `build.ps1` is deterministic with Visual Studio's compiler (two builds, one SHA-256) |
| B3 no licence text | `build.ps1 -Release` zips the exe with `README.txt`, `LICENSE.txt`, `OFL.txt` and `THIRD-PARTY-NOTICES.txt` |
| B4 the Deadly Stream mark | Removed with the pop-up (the maintainer); `logos.html` and both marks deleted |
| F1, F2, F3 partial writes, the `.tpc` name, closing | `Conversion.cs`: made in a work folder, moved only when all are made and none to replace is read-only; `-2`, `-3` names; closing stops with nothing written. Tested |
| F4 unbounded atlas | Refused above 4096. Read |
| F5 no Latin letters | Refused with a message naming them. Tested (a digits-only font) |
| F6, F7 WOFF and `fvar` | Bounds, sizes and inflated length checked; a damaged `fvar` offers the default instance. Tested (the 2 GB WOFF) |
| F8 Change while converting | Change, Clear, Settings and the Version box do nothing while converting |
| F9 Open Output Folder, `--size=` | A message instead of an error box; `--size=` ignored when not two numbers |
| F10 temp files | Emptied at start and at exit |
| T1 to T5 wording | New introduction; no Done pop-up (Convert turns green); the Replace question says `.tpc` files are renamed; every type named; the converter's own messages, not raw .NET text; Escape has no Done pop-up to open Explorer from |
| KMSE text | KMSE's steps and sizes, raised quiet colours, text at device pixels. Screenshots |
| R1 version | `Program.Version` 1.0.0, read by the footer, the file properties and the zip's name; copyright filled |
| R2 repeatable build | As B2 |
| R3 CI and tests | `Test-FontConverter.py` added; **not yet in CI** (it needs Windows fonts and a GUI session) |
| R4 user readme | `release/README.txt` |
| R5 release channel | With KMRP 1.6.0 and KMSE the same day; **where it is uploaded is the maintainer's to decide** |
| R6 notices | `release/THIRD-PARTY-NOTICES.txt`; the root file's font line corrected and a section added |
| R7 antivirus | **Open**: unsigned; scan on VirusTotal before release |
| R8 docs | `docs/font-converter.md` rewritten: files, both versions, window, command line, build identity, verifying, not changed |
| Monolith | Split into nine files by job |
| Repaint work, picker samples, `g.Clip`, COM on error paths, dark title bar 1809 to 1909 | Fixed (families listed once; one sample a face; the region disposed; tables, unpacked streams and faces released on errors; attribute 19 tried after 20) |
| Not changed | The examples' boxes (decision 3: kept); per-monitor DPI; the console ring cut at the cell's top and bottom; tall line metrics making smaller letters; a large collection read on the window's thread |
