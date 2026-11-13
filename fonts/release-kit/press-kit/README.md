# KMRP Original HD fonts 1.0 press kit

Release art and press text for the KMRP Original HD fonts on their own, without the Font
Converter (whose kit is [`font-converter/release/press-kit`](../../../font-converter/release/press-kit/README.md)).
Use the pages as they are; please do not crop a part out of a page without saying what it
shows.

Held to the [documentation standard](../../../docs/documentation-standard.md): every number
below and on the pages is read from the font files or from [`fonts/README.md`](../../README.md),
which says how it was measured.

## The pages

Six pages, 1920x1080, in the order to show them, and the comparison picture beside this
folder as a seventh.

| File | What |
| --- | --- |
| [`01-cover.png`](01-cover.png) | Cover: the name and version, "KOTOR's own fonts, redrawn in high resolution", 11 OpenType fonts, one .ttc collection, 218 characters, the licence, five sample tiles (Menu Large, Menu, Dialogue, Descriptions, Console) in the game's text colours |
| [`02-the-family.png`](02-the-family.png) | All 11 fonts, one line each, in the game's colour for that kind of text |
| [`03-drawn-not-traced.png`](03-drawn-not-traced.png) | Menu's R and Dialogue's g as outlines: straight edges blue, arcs yellow, points and handles; how many of each; the 0.62 to 1.61% measured against the HD atlases |
| [`04-characters.png`](04-characters.png) | Every character of Dialogue (216 drawn, plus the space and the no-break space), Windows-1252 |
| [`05-kerning.png`](05-kerning.png) | Two lines in Dialogue and Menu without kerning (as the game sets them) and with the fonts' own, each kerned pair's value in font units |
| [`06-which-font.png`](06-which-font.png) | Which font is which game font, and what the game uses it for |
| [`../KMRP-Original-HD-Fonts-comparison.png`](../KMRP-Original-HD-Fonts-comparison.png) | The game's own font textures enlarged beside the HD fonts at the same size (made earlier by `make_compare.py`, from the game's fonts) |

## Press text

**Short (one line):**

> KMRP Original HD fonts: KOTOR's own 11 fonts, redrawn in high resolution as OpenType, free
> under the SIL Open Font License 1.1.

**Long:**

> The KMRP Original HD fonts are the fonts of Star Wars: Knights of the Old Republic, redrawn
> in high resolution by RaymanGT for KMRP, the KOTOR Modern Restoration Patch. The game
> keeps its text in 17 small font textures; they hold 11 different sets of letters, and each
> one is now an OpenType font: Menu, Menu Small, Menu Small B, Menu Medium, Menu Large,
> Dialogue, Descriptions, Light, Console, Credits and Credits Bold.
>
> Every letter was measured from KMRP's HD atlases of the game's fonts and drawn again as
> straight lines, sharp corners and arcs, without tracing or smoothing. The letter widths,
> spacing and baseline are the game's, so text set in them lines up as it does in KOTOR.
> Each font has the game's 218 characters (Windows-1252: English, French, German, Italian,
> Spanish and similar; Console 96), and all but the monospaced Console carry letter-pair
> kerning for programs that use it.
>
> The download holds the 11 fonts as .otf files and as one .ttc collection. They are free
> to use, change and share in any game, mod, app, video or website under the SIL Open Font
> License 1.1; please credit "KMRP Original HD fonts by RaymanGT".

**Facts:**

| | |
| --- | --- |
| Name | KMRP Original HD fonts |
| Version | 1.0 (the files' own version string, "Version 1.000"; 1.600 until 2026-11-13) |
| By | RaymanGT, for KMRP |
| Files | 11 OpenType (CFF) fonts; the same 11 in one TrueType collection, `KMRP-OriginalHD.ttc` |
| Characters | 218 a font, Windows-1252; Console 96 |
| Kerning | 1,657 to 2,258 letter pairs a font, tightening only; Console none |
| Licence | SIL Open Font License 1.1, no Reserved Font Name |
| Credit | "KMRP Original HD fonts by RaymanGT" |
| Download | `KMRP-Original-HD-Fonts.zip` beside this folder |
| Home | https://github.com/RayesDiyab/KOTOR-KMRP-Font-Converter |

## How they were made

- **Everything is drawn by [`make_press_kit.py`](../make_press_kit.py)** from this
  repository alone: the files in `fonts/otf`, KMRP's logo (`font-converter/kmrp-logo.png`)
  and Pillow and fontTools. Run `python fonts/release-kit/make_press_kit.py`.
- **The look is the fonts' own**, the one the preview and the comparison pictures beside
  this folder already have: the game's GUI text colours (vanilla .gui TEXT COLOR: blue,
  yellow, white), dark panels with a blue rule, a faint grid. Deliberately apart from the
  Font Converter's navy window style.
- **The outlines on page 3 are the files' own**, read with fontTools and drawn point for
  point; the counts under each glyph are counted from them by the script.
- **The kerning on page 5 is the files' own**, read from their GPOS tables by the script
  and applied by it (the Pillow here has no Raqm, so it sets no kerning itself). The
  pair counts are counted the same way.
- **No picture of the game's own letters is made here.** The comparison picture is the
  only one in this kit that shows them, made before by `make_compare.py`.
- **Not claimed:** how the fonts install or look on macOS or Linux. Every page was drawn
  on Windows with Pillow; the fonts were not opened on a Mac or under Linux for this kit.

Seen while making page 4 (2026-11-13): three of Dialogue's characters, `]`, the left
single quote and the single left guillemet, have an advance of 1684 units, about two and a
half times the ink they draw (the other wide advances in the family are wide letters).
`fonts/README.md` says the advances are the game's to a unit, so this would be the game's
own `fnt_d16x16` width table; not checked against the game's files for this kit. Page 4
centres each character by its ink, so it does not show.
