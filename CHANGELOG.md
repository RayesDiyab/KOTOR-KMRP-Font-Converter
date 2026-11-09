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
maintainer, 2026-11-08). The Windows exe is 11,238,912 bytes, SHA-256
`DF4B6178C7D7A507BAFE6CAE094A098D5516643487AA9DF20C1F34971582A348`, built deterministically
from this repository; it and the other packages, and how they were checked, are in
[font-converter.md](font-converter/docs/font-converter.md) sections 6 and 7.

**Changed, 2026-11-09: rewritten in Rust, for Windows, macOS and Linux**

- One Rust program instead of the C# one (.NET Framework 4, Windows only): a Windows x64
  exe and zip, a macOS app for Apple silicon and Intel (Universal 2, DMG and zip), and
  Linux AppImages for x86_64 and aarch64. `build.ps1`, `build-macos.sh` and
  `build-linux.sh` make them, after KMSE's scripts; `.github/workflows/build.yml` runs the
  tests on all three systems and builds no release (the maintainer: "dont build the actual
  release just make the ci run tests"; until then it could make the packages when run by
  hand or by a tag).
- Fonts read by the program itself (skrifa; wuff for WOFF2) instead of Windows'
  DirectWrite. The KMRP version's TXI files outside the Journal tile are the C# program's
  byte for byte for Arial, Georgia and KMRP's Dialogue OTF.
- The KMRP version's atlases are always the size of KMRP 1.6's own HD Font's (the
  maintainer: "the exact size font files for kmrp that kmrp 1.6 currently uses"). The
  Journal tile's three (`fnt_creditsa`, `fnt_d16x16a`, `fnt_dialog16x16`) had come out
  2048 square, 16 MB each, where KMRP's are 1024, 4 MB: where the letters do not fit in
  slot order, the blank slots now share one empty cell and the letters are packed widest
  first. All nine fonts the test converts fit KMRP's sizes; the test now checks it.
- The window drawn through wgpu (DirectX 12, with Windows' software renderer where there is
  no graphics driver; Metal; Vulkan), with OpenGL 2 as the fallback, and a message box where
  neither works. The first CI run found the window did not open on a Windows machine
  without a graphics driver, where only OpenGL 1.1 exists.
- Microsoft's C runtime linked into the Windows exe, so it starts without the Visual C++
  Redistributable; the exe is 11.2 MB for both changes (6.5 MB before).
- The test's KMRP-size check allows a larger atlas only where a font's letters cannot fit
  KMRP's size (KMRP's own Menu face's Journal letters, which the CI run on macOS and Linux
  converts); until then it demanded KMRP's size for any font.
- The original-game version the same way, after the game's own fonts (the maintainer:
  "should match size, etc., and spacing of the original game"): each atlas the game's size
  (the HUD font 128 square, as the game's, where it was 256), each TXI header the game's line
  for line (`fnt_console`'s spacingR 0.02, as the game's, where it was 0.01), CR LF line
  ends. The game's numbers are carried in `data/originals.txt`, made by `make_data.py`
  from a built KMRP checkout. A HUD font too wide for 128 even with no gap between letters
  still gets 256; since the HUD letters hold their ring (below), none of the nine test
  fonts is.
- The HUD font's letters drawn one ring width smaller on every side, so each letter and its
  black ring are the size the plain letter would be, as the game's own HUD letters hold
  their outline (the maintainer: "make it 2 pixels smaller to account for the black
  border"). Until then each cell was 2 pixels wider than the letter, and the ring was cut
  at the top and bottom. KMRP's own HD Console face now fits the game's 128 atlas (940
  pixels of letters where it needed 1,128).
- The original game's letters hinted by the font's own instructions (FreeType's normal
  target) instead of Windows' GDI-classic drawing, then widened by 0.3 of the game's
  pixels (the maintainer: "slightly wider instead of slightly narrower"): the same heights
  and baselines, letters 2 to 7% wider and spacing 2 to 4% wider than Windows drew them.
  Measured in font-converter.md section 9; **not yet seen in the game**.
- The window fits itself to the screen as KMSE does (the maintainer: "make it dynamic (like
  kmse smaller on smaller screens)"): it opens as large as the screen allows, shrinks evenly
  down to half the canvas, and follows a move to another screen. The 16:9 edge drag of the
  Windows window is gone.
- A font missing some of the game's letters shows a badge with the warning sign and how
  many, beside the cross; pointing at it lists the letters (the maintainer: the warning
  "isn't really shown fully"). Until then "N missing" cut the font's name short.
- `--convert --exit` opens no window.
- Every package's files named `KOTOR-Font-Converter-<version>-<system>` (the maintainer:
  "call exes from now on KOTOR-Font-Converter-1.0.0-Windows-x64.exe and same for rest"):
  the Windows exe and zip (`-Windows-x64`), the macOS app, DMG and zip (`-macOS`), the
  AppImages (`-Linux-x86_64`, `-Linux-aarch64`). Until then `KMRP Font Converter.exe`
  and `KMRP-Font-Converter-<version>-*`. The window's title and the macOS menu bar still
  say "KMRP Font Converter".
- The tests run on macOS and Linux too, with KMRP's own fonts as inputs where Windows' are
  missing; their close-while-converting check finds the window by its title, and passes on
  nothing written or the whole set, failing on any part of it (until then it demanded
  nothing, and failed on the CI's Windows machine, where the fonts were done before the
  window read the close).
- **Not yet run:** the macOS app (type-checked for both architectures, not built), the Linux
  window, the aarch64 AppImage. The x86_64 AppImage converted fonts under WSL's Ubuntu 20.04.

**Added**

- Two versions, in a Version box between Clear and Convert: **KMRP** (KMRP's HD size, for
  KMRP 1.6.0 or later, which offers the fonts as Modded Font) and **Original Game** (the
  game's own sizes, the HD sizes divided by six, with Windows' hinted drawing; the font's
  own hinting since the Rust rewrite). Seen working in an unmodified GOG 1.03 game at
  1280x960 with Georgia, made by the C# program.
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
- The window split from one 1,848-line file into nine by job (C#; since the Rust rewrite,
  eight Rust files by the same jobs).
- The bottom row 86 pixels high instead of 68, and "Version" larger (2026-11-09); until then
  1.0.0's exe was `30643C4A...`.
- The release zip holds its five files at its root, without a folder (2026-11-09).
- The program's icon is the sigil above FONT, as in the README, instead of KMRP's installer
  icon (2026-11-09); the exe is 272,896 bytes smaller for it. Until then 1.0.0's exe was
  `89B63293...`.

**Not yet:** signed (the macOS app is signed ad hoc); scanned on VirusTotal. (Until the
Rust rewrite this also said "run under Proton or Wine": the C# program was never tried
there, and the Rust one runs natively instead.) In the game, not yet
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
