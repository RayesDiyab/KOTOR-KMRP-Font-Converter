# Changelog

> **Documentation standard.** This document follows
> [`docs/documentation-standard.md`](docs/documentation-standard.md). Read it before editing
> this file, and check the result still meets it: measured claims only, corrections kept
> visible, and anything untested labelled as untested.

All notable changes to the KMRP Font Converter and the KMRP Original HD fonts are recorded
here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

Both were developed in KMRP's repository until 2026-11-08 and came here with their
history (`git log` reaches back to their first commits). Their entries until then are in
[KMRP's changelog](https://github.com/RayesDiyab/KMRP-KOTOR-Modern-Restoration-Patch/blob/master/CHANGELOG.md).
Nothing has been released from this repository yet: no tag, no GitHub release
(checked 2026-11-09).

## [Unreleased]

### KMRP Font Converter 1.0.0

To be released on the same day as KMRP 1.6.0 and the KOTOR Modern Gamesave Editor (the
maintainer, 2026-11-08). The exe is 1,113,088 bytes, SHA-256
`30643C4AA241B1B19B14052E21D6D32E99A34E2FDB7EF4BF9858131F15032526`, built deterministically
from this repository; how it was checked is in
[font-converter.md](font-converter/docs/font-converter.md) section 6.

**Added**

- Two versions, in a Version box between Clear and Convert: **KMRP** (KMRP's HD size, for
  KMRP 1.6.0 or later, which offers the fonts as Modded Font) and **Original Game** (the
  game's own sizes, the HD sizes divided by six, with Windows' hinted drawing). Seen
  working in an unmodified GOG 1.03 game at 1280x960 with Georgia.
- The logo reads KOTOR when Original Game is chosen.
- All-or-nothing conversion: every font made in a work folder and moved into place only
  when all are made; a `.tpc` set aside earlier is kept and the next named `-2`, `-3`.
- Refusals with a message: a font missing letters A to Z or a to z, an atlas over 4096
  pixels, a font file over 128 MB, a WOFF whose tables claim more than that.
- `build.ps1 -Release`: a zip with the exe, `README.txt`, `LICENSE.txt`, `OFL.txt` and
  `THIRD-PARTY-NOTICES.txt`. `tests/Test-FontConverter.py`: the regression test.

**Changed**

- No pop-up at the start and no Deadly Stream or GitHub buttons; Done is the Convert
  button turning green ("Converted").
- The window's text at KMSE's sizes and drawn at the screen's own pixels.
- The window split from one 1,848-line file into nine by job.

**Not yet:** run under Proton or Wine; signed; scanned on VirusTotal. In the game, not yet
looked at: the HUD's numbers, the end credits and Menu Large.

### KMRP Original HD fonts 1.600

The 11 OpenType files and the collection listed with their hashes in
[fonts/README.md](fonts/README.md) section 1; the download is
`fonts/release-kit/KMRP-Original-HD-Fonts.zip`.

### Repository

- 2026-11-08: the converter and the fonts moved here from KMRP's repository, history
  kept. KMRP keeps its HD Font atlases, drawn from these fonts.
- 2026-11-09: the press kit, in the converter's own style; the README's logo; the
  documentation standard imported from KMRP's, with KMRP's link checker; this changelog.

**Corrected 2026-11-09:** the release zip's `README.txt` and notices, and the fonts'
download readme, still named KMRP's repository as the home of the source and the fonts
after the move. They now name this one; the fonts' zip was made again with the new
readme (its 14 files otherwise the repository's, byte for byte). `make_zip.py` wrote
`KMRP-Original-HD-Fonts-1.600.zip` while the committed zip is named without the
version; it now writes the committed name.
