# KMRP Original HD fonts

Reference, 2026-10-22, following the [documentation standard](../docs/documentation-standard.md)
(KMRP's, imported 2026-11-09).
The lab record of how the redraw was reached is section 6 of this document. In KMRP's
repository until 2026-11-08, then in the KOTOR-KMRP Font Converter's with the converter:
`tools/`, `assets/` and `build/native-runtime` paths below are KMRP's, which the scripts in
`fonts/tools` find beside this repository or through `KMRP_ROOT` (`tools/kmrp_checkout.py`).

KOTOR's own fonts in high resolution, as 11 OpenType (CFF) files and one TrueType
collection holding all 11 (`KMRP-OriginalHD.ttc`): the letters of KMRP's (New) Original
HD Font, drawn as straight lines, sharp corners and arcs. **Free to use,
change and share, with credit, under the [SIL Open Font License 1.1](OFL.txt)**:
anyone may bundle them in a game, a mod, an app or a website without asking. No
Reserved Font Name, so a changed version may keep the name.

**Version 1.0, released** 2026-11-13 as GitHub release
[`fonts-v1.0`](https://github.com/RayesDiyab/KOTOR-KMRP-Font-Converter/releases/tag/fonts-v1.0):
`release-kit/KMRP-Original-HD-Fonts.zip`, 683,910 bytes, SHA-256
`8fb2f62945494ab46d615f8ba8ddcd136f4b603d68aaa2a86a6d6f2b3f3e433a`. Checked that day before the upload: the
zip's CRC passed, its 14 files are this repository's byte for byte (`README.txt` with CR LF
line ends), and the 12 font files match section 1's sizes and hashes.

*Corrected 2026-11-13:* the fonts were version 1.600 (`head.fontRevision` 1.6, name ID 5
"Version 1.600", name ID 3 "1.600;KMRP;..."), and the same day released as `fonts-v1.600`;
the maintainer: "the orignal hd font is 1.0!". That release and its tag were deleted and
made again as `fonts-v1.0`. `tools/set_version.py` set those three to 1.0, "Version 1.000"
and "1.000;KMRP;..." in the 11 OTF files and the 11 fonts of the collection, keeping
`head.modified`; every other table is the 1.600 files' byte for byte (compared table by
table against the old files), so the outlines, hints, kerning and metrics, and KMRP's
atlases drawn from them, are unchanged. Every hash in section 1 changed with it; the
collection is 1,992 bytes smaller (653,552 before), its tables the same, laid out by
fontTools' `shareTables` again. `finish.py` and `run.py` now write 1.0 too. Not changed:
the converter's two TrueType copies in `font-converter/fonts`, which are inside the
released converter 1.0.0 exe and still say 1.600.

KMRP draws its game atlases from these files (`tools/build_hd_original_atlases.py`
writes `assets/hd-original-fonts`), and the KOTOR Modern Gamesave Editor sets its window in them.

## 1. The files

`otf/`, 11 fonts, CFF outlines. One file per different font the game uses; the game
keeps the same letters under several names, so 11 files serve its 17 text fonts. 1000
units per em; advance widths, ascent and descent are the game's (to a unit), so text
sets as it does in KOTOR. `KMRP-OriginalHD.ttc` holds the same 11 with TrueType outlines
(the cubic curves made quadratic within half a unit, `tools/collection.py`) and
ttfautohint's hints: one file that installs all 11 fonts.

| File | Bytes | Units per em | Glyphs | SHA-256 |
| --- | ---: | ---: | ---: | --- |
| `KMRP-OriginalHD-Console.otf` | 16,068 | 1000 | 96 | `fd3d78a32bbecfd252c8936082d2469e852cba3334e7b361aa8994d571f2feee` |
| `KMRP-OriginalHD-Credits.otf` | 45,380 | 1000 | 218 | `d7a7cedd038286e8838445df1d657867e51372ec1a5b82659364be69c46160ad` |
| `KMRP-OriginalHD-CreditsBold.otf` | 45,420 | 1000 | 218 | `cc396509a0123e36031ae8aeb844d6f7d61e8f9a09c86047449a011bf5029954` |
| `KMRP-OriginalHD-Descriptions.otf` | 45,456 | 1000 | 218 | `1abf6e72534180d7ae9200c1d6abf441b4c31269efb2895c9d75bf68c2719a58` |
| `KMRP-OriginalHD-Dialogue.otf` | 45,384 | 1000 | 218 | `e267b4693563156559a4b9ff58fea2ea292aaf72bdebd65368117bd13cb6849d` |
| `KMRP-OriginalHD-Light.otf` | 38,028 | 1000 | 218 | `1a0dd3455d6831e42288e7c530e907d2863488ef173d1f9bd265432dbf6becf5` |
| `KMRP-OriginalHD-Menu.otf` | 40,244 | 1000 | 218 | `541880c0f8e3cfe3ca517096562d4c3bc49615b3a3e2cc1d39fbe59ed524f6b6` |
| `KMRP-OriginalHD-MenuLarge.otf` | 41,428 | 1000 | 218 | `1743791616d76c8bafc86035218e2baf286eadb789c57020be2de736060b596a` |
| `KMRP-OriginalHD-MenuMedium.otf` | 40,452 | 1000 | 218 | `f2c70ea7b6cb4e58caf49affa9cee3a3463bbd499474edb964287caa767f11b6` |
| `KMRP-OriginalHD-MenuSmall.otf` | 38,500 | 1000 | 218 | `867c8552f50e895d90836f22d7988969c0e171c7dffc8b6458d526d7c6de9e6a` |
| `KMRP-OriginalHD-MenuSmallB.otf` | 40,188 | 1000 | 218 | `da1466f65a0ac04fff03b537633f7427cda944b2b4ea285480af36bfe0b5f75b` |
| `KMRP-OriginalHD.ttc` | 651,560 | 1000 | 11 fonts | `c318758ebb70bbff438a32213b7caed6bacb84764641acd49192ec6a73f42086` |

Which file is which game font:

| File | In the game | Text |
| --- | --- | --- |
| Menu | dialogfont16x16, dialogfont16x16b | screen titles, menu buttons |
| MenuSmall | dialogfont10x10, 10x10a, 16x16a | option lists, small labels |
| MenuSmallB | dialogfont10x10b | tabs, small bold labels |
| MenuMedium | dialogfont12x16 | save and load lists |
| MenuLarge | dialogfont32x32 | the largest headings |
| Dialogue | fnt_d16x16, fnt_d10x10b | conversations, subtitles |
| Descriptions | fnt_d16x16b | reply choices, descriptions |
| Light | fnt_d16x16a, fnt_creditsa, fnt_dialog16x16 | the journal, longer reading |
| Console | fnt_console | the HUD's numbers (the game adds a black ring; not in the file) |
| Credits | fnt_credits | job titles in the end credits |
| CreditsBold | fnt_creditsb | names in the end credits |

Built to the OpenType recommendations for CFF fonts ([Microsoft](https://learn.microsoft.com/en-us/typography/opentype/spec/recom),
[SIL's Font Development Best Practices](https://silnrsi.github.io/FDBP/en-US/OpenType.html),
[Adobe's MakeOTF guide](http://adobe-type-tools.github.io/afdko/MakeOTFUserGuide.html)):

| Rule | In these files |
| --- | --- |
| Units per em | 1000, the CFF FontMatrix exactly 1/1000 |
| Outlines | outer contours counter-clockwise (PostScript), overlaps removed (skia-pathops), points on whole units |
| Glyph names | production names from the Adobe Glyph List (`A`, `eacute`), `.notdef` a visible box |
| Names | Windows records only; PostScript name (ID 6) as the CFF's font name; licence (IDs 0, 13, 14), designer, vendor URL |
| OS/2 | version 4, Regular, USE_TYPO_METRICS, `fsType` 0 (no embedding restriction), Unicode and code page ranges (Windows-1252), Console declared monospaced |
| Vertical metrics | hhea = typo; win ascent and descent at least the font's highest and lowest points (Windows clips beyond them) |
| Hints | OTF: Adobe `otfautohint` 3.0.0, alignment zones and stem widths measured from each font's own H, x, O, o, p and q; TTF: ttfautohint, the whole-pixel flag set |
| Size | outlines subroutinised (`cffsubr`); the collection shares the tables that are the same |
| Kerning | a GPOS `kern` feature, letter pairs measured from the outlines (section 2); Console none, as it is monospaced |

Two glyphs are made rather than measured: MenuSmall's underscore, which sits on the
very bottom of its atlas cell and is cut to an 11-unit sliver there (TrueType hinting
rounded it away at 16 px), is a bar 38 units thick like the Menu font's; and ÿ in Light
and MenuLarge, whose game fonts have no ÿ, is their y with the two dots of their ä
centred over it. U+00A0 (no-break space) is the space.

## 2. How they were made

The reference is the HD atlas each font came from (`assets/hd-original-fonts` as it
stood before this redraw, at commit `87547b0`), 6 times the original's size. Nothing
else was opened: not the old OTFs' outlines, not any other foundry's font.

| Step | What it does | Code (`tools/`) |
| --- | --- | --- |
| Edge | Where coverage crosses one half, between pixel centres (marching squares) | `measure.py` |
| Area reading | Each crossing moved to its edge's exact position, read from the coverage as area: across a row the partial pixels of an edge sum to its position (`k + sum(1 - c)` rising, `k + sum(c)` falling). Exact for a straight edge; rows for steep edges, columns for flat ones | `refine.py` |
| Straight edges | Runs whose points lie on one least-squares line: within 2.6 units scatter, no bend beyond 0.6 units (or 0.6% of the length). A short straight edge counts only between sharp corners: in a curve it is a chord | `redraw2.py` |
| Corners | Two straight edges meet where their lines cross, if the outline between follows that corner within 3 units / sin(half the angle) (the anti-aliasing fills acute corners more), at most 8; a stroke's cut end (a short horizontal or vertical edge against a long stroke) up to 10 | `redraw2.py` |
| Curves | Cubic arcs through the measured points, the points at the extremes with horizontal or vertical handles, accepted at 0.6 units mean and 1.5 units most | `redraw2.py`, `redraw.py` |
| Rings | The rings of © and ® are exact: a circle or an axis-aligned oval, whichever the HD ring is closer to (the oval only if it fits at least 30% better and is 1% from round), both edges about one centre, four quarter arcs each | `rings.py` |
| Grid | Points on 1/80 of an atlas pixel while drawing, each arc's handles fitted again to the rounded ends; 1000 units per em in the finished files | `redraw.py` (`realize`), `finish.py` |
| Finish | The rules of section 1, hinting, subroutinising | `finish.py` |
| Kerning | Pair kerning for a to z and A to Z, each with its accented forms: at every height both letters have ink, the gap between them from the outlines; the target is straight and round against each other (H H, O O, H O, O H; for small caps n n, o o, n o, o n); a pair looser than that is drawn in by 70% of the difference, its narrowest point never closer than half of H H's gap; only tightening, values in steps of 5 units, none under 15. About 1,650 to 2,260 pairs a font. Console has none (monospaced). Advances, outlines and hints are unchanged | `kern.py` (run by `finish.py`) |
| Collection | TrueType outlines, ttfautohint, one `.ttc` | `collection.py` |

`run.py` redraws all 11 and writes `build/fonts/redrawn`; `finish.py` writes `otf/`;
`collection.py` writes `KMRP-OriginalHD.ttc`; `check.py` compares the fonts with the
atlases (another set with `KMRP_ATLASES`); `sheet2.py` draws comparison sheets.

## 3. Measured

Against the HD atlases the fonts were drawn from (`assets/hd-original-fonts` at commit
`87547b0`; exact area coverage of every pixel of every glyph's cell, mean difference on
the ink; `KMRP_ATLASES=<those atlases> python check.py`), the traced fonts KMRP had
before (`tools/build_hd_original_otf.py`, potrace) and these:

| Font | Traced fonts | These fonts |
| --- | ---: | ---: |
| MenuLarge | 1.38% | 0.62% |
| Console | 2.85% | 0.76% |
| MenuSmallB | 2.49% | 0.84% |
| Menu | 2.39% | 0.85% |
| Dialogue, Credits, CreditsBold | 2.20% | 0.87% |
| Descriptions | 2.21% | 0.88% |
| MenuMedium | 2.97% | 1.01% |
| MenuSmall | 3.90% | 1.60% |
| Light | 3.98% | 1.61% |

Before the move to 1000 units per em the same fonts measured 0.33% to 1.41%: at 1000 a
point can sit half a unit off (a twentieth of an atlas pixel in most fonts, a tenth in
MenuLarge, whose em is 192 atlas pixels). Glyphs further from their atlas than the
traced ones were: in MenuLarge `>`, `<`, `=`, `^`, Y, Ý and ® (by 0.1 to 0.4 points; the
atlas's own anti-aliasing along long diagonals; the edge measurement was checked on
strokes rendered with exact coverage at eight angles, where it is within 0.002 units),
MenuSmall's © and ® (8.6% against 5.2%: the HD ring is 2.5 pixels thick and uneven, the
exact ring is even) and MenuSmallB's k (1.5% against 1.4%). Single pixels differ most at
corners, which the atlas softens and these draw sharp (up to about 75%), at ¤, whose
arms run off the atlas cell (96%), and at MenuSmall's underscore (made, section 1).

| Check | Result |
| --- | --- |
| fontbakery 1.1.0 `check-universal`, each file on its own | no FAIL in any of the 11 OTF and 11 TTF files; one ERROR of fontbakery's own (`opentype/monospace` reads `glyf`, which a CFF font has not) on Console's OTF |
| Windows GDI (`AddFontResourceExW`, `TextOutW`, 16 and 72 px, anti-aliased) | every character of every font draws, from the OTF files and from the collection (MenuSmall's underscore at 16 px only since it is a bar) |
| FreeType 2.12 (unhinted) against the atlases drawn from these fonts | 0.11% (MenuLarge) to 0.65% (Light) |
| `tools/build_hd_original_atlases.py` round trip | the committed atlases against these fonts: 0.01% to 0.04% |

*Corrected 2026-10-22, three times the same day.* (1) The first finished files had no
PostScript name (name ID 6) and no Regular bit, as the traced fonts before them;
Windows' font viewer refused them all ("keine gültige Schriftartendatei"), the old
traced ones too. (2) With those added, the viewer opened them but drew no letters:
Windows' CFF rasteriser left W, w and @ blank at 7680 units per em (and the viewer drops
every sample line holding one); at 1000 every character draws. The fonts were then
rebuilt to the recommendations of section 1 rather than patched. (3) MenuSmall's
underscore vanished at 16 px in the TrueType collection; it is now a bar.

## 4. Limits

- Windows-1252 only, as the game's fonts (218 glyphs; Console 96).
- Kerning only where a program applies it (GPOS: Slint, browsers, Word, HarfBuzz and
  DirectWrite do; GDI and GDI+ do not, so the Font Converter's window sets text without
  it). The game has none and its atlases are drawn glyph by glyph, so they are unchanged
  by it (`tools/build_hd_original_atlases.py --check`: 0.01% to 0.04%, as before).
- The redraw follows the HD atlases, which were rendered from Bank Gothic Medium and
  Helvetica fitted to the game's letters (`tools/build_hd_original_fonts.py`); a
  typeface's design is not protected by copyright in the US and its protection in the
  EU has long run out for these designs, and no outline data of those fonts is in
  these files (each point was placed by the redraw over the atlas's pixels). This is
  the reasoning, not legal advice.

## 5. Not changed

- The letters' widths, spacing and baseline: the game's. The kerning (2026-10-24) is an
  adjustment on top of them, for programs that set text from these files.
- The old traced fonts' tool, `tools/build_hd_original_otf.py`, and the atlases' first
  builder, `tools/build_hd_original_fonts.py`, stay in the repository as the record of
  how the atlases were first made; neither is part of any build now.
- `fnt_galahad14`: game art, made at build time, never a font file.
- The address in the files: their copyright (name ID 0 and the CFF Notice), vendor and
  designer URLs (name IDs 11 and 12), all from `COPYRIGHT` and `VENDOR_URL` in
  `tools/finish.py`, and `OFL.txt`'s first line name KMRP's repository, where the fonts
  were made. Left as they are on the move here (2026-11-08): KMRP's page is still the
  project's, and changing them changes every file and hash in section 1. The letters, and
  so KMRP's atlases drawn from them, would not change.

## 6. Lab record: rejected along the way (2026-10-22)

| Tried | Why not |
| --- | --- |
| Matching the traced fonts point for point | Their diagonals are potrace's stair steps from the atlas's anti-aliasing (the y in Light: 2.7 units waves every 21 units); copying them copied the defect. The maintainer: "it shouldn't be jaggy diagonals. Diagonals should be straight." |
| Smoothing the measurements (Gaussian, 0.5 to 1 pixel) | Rejected by the maintainer ("Don't do smoothing. Just redraw... Just draw one diagonal"); it also rounds small details |
| The 50% crossing alone as the edge | It ripples once per pixel along a slanted edge (about 1.5 units), as much as a gentle curve bends over 40 units, so no rule could tell a chord from a straight edge; the area reading fixed it |
| A bend allowance for every run length | Under 80 units the ripple and a curve look alike; the rule that a short straight edge sits between corners replaced it |
| Corners at a fixed 3 or 6 units | Acute corners (V's notch) are filled more by the anti-aliasing than square ones; the allowance now depends on the angle |

## 7. Verifying by hand

```bash
python fonts/tools/check.py                 # every glyph against its atlas (needs the traced fonts in build/fonts)
python tools/build_hd_original_atlases.py --check   # the atlases drawn from these fonts against the committed ones
python -m fontbakery check-universal fonts/otf/KMRP-OriginalHD-Menu.otf   # one file at a time
```

The second one is a round trip: the committed atlases are drawn from these files, so it
reports 0.01 to 0.04% (the rounding of 8-bit pixels). The tools need numpy, fontTools,
Pillow and scipy; numba makes the coverage check fast; `finish.py` needs skia-pathops,
cffsubr and Adobe's `otfautohint` (`pip install afdko`, in a Python AFDKO has wheels
for: 3.12 on 2026-10-22, not 3.14; `OTFAUTOHINT` names the program); `collection.py`
needs ttfautohint-py.
