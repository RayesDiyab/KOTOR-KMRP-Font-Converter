# The documentation standard

> **This file _is_ the standard.** Every other document in this repository is held to
> it, and each one links back here. Changing the rules here changes what the rest of the
> documentation has to meet.

Imported on 2026-11-09 from KMRP's
[`docs/documentation-standard.md`](https://github.com/RayesDiyab/KMRP-KOTOR-Modern-Restoration-Patch/blob/master/docs/documentation-standard.md)
(the maintainer: "import the doc quality standard from there"). KMRP's is written for a
patch to the game's executable: its rules speak of addresses, file offsets and
disassembly. This repository writes font files and a Windows program, so those rules are
restated for fonts, atlases and code, and the examples are this repository's own. The
rules themselves, the two kinds of document and the checklist are KMRP's. When KMRP's
standard changes, compare the two and bring this one along.

The test a document has to pass: **someone who was not here, working only from this
repository, Windows and a copy of the game, can verify every claim and redo the work.**

## Two kinds of document, kept apart

| Kind | Holds | Here |
| --- | --- | --- |
| **Lab record** | What was tried, in the order it happened. Failures, dead ends, rejected candidates, disproved theories. Never edited to look tidy. | [`release-audit-2026-11-08.md`](../font-converter/docs/release-audit-2026-11-08.md); section 6 of [`fonts/README.md`](../fonts/README.md) |
| **Reference** | What the program and the files are now, to the byte where bytes matter. | [`font-converter.md`](../font-converter/docs/font-converter.md); sections 1 to 5 and 7 of [`fonts/README.md`](../fonts/README.md) |

Each says at the top which it is, and links to the other.

## The rules

**1. Every claim is measured, and says how.** "`Test-FontConverter.py` found them equal
to the game's own TXI values", "seen in a vanilla game at 1280x960", "counted from the
10.0.26100.0 SDK headers". A number taken from someone else's documentation, a design
note or memory is a different claim and is labelled as such: it may be wrong.

> Not hypothetical. On 2026-11-08 the converter's reference still opened with "Not yet
> seen in the game" after Georgia had been seen in it that morning (audit R8). The
> sentence had been true when written and nobody measured it again.

**2. State the build.** A claim about what the program does names the exe it was seen in:
its version, length and SHA-256 (1.0.0 is 840,192 bytes, `81D4BD89...`). Before it
was released, 1.0.0 was three different exes over 2026-11-08 and 09: `30643C4A...`, then
`89B63293...` after the taller bottom row, at the same length, so length alone would not
have told those two apart. A claim about
the fonts names the files' hashes ([`fonts/README.md`](../fonts/README.md) section 1). A
claim about the game names the game's executable: the GOG 1.03 exe (`9C10E045...`) and
Steam's (`34E6D971...`) are different programs. "It works" without a build is a claim
about nothing.

**3. State the units, every time.** Three scales meet in this repository and none is
ever left implicit:

| Unit | What it measures | Example |
| --- | --- | --- |
| Font units | Outlines in the font files, 1000 to the em | MenuSmall's underscore, 38 units thick |
| HD atlas pixels | KMRP's atlases, **six times** the original fonts | `dialogfont16x16`: 96 pixels high |
| Original pixels | The game's own atlases, and the original-game version | the same font: 16 pixels, a sixth |

The TXI files add a trap of their own: they write lengths in hundreds of pixels, so those
96 pixels are `fontheight 0.960000` in KMRP's `dialogfont16x16.txi`. A fourth scale, the
window's 1600x900 canvas against the screen's own pixels, applies to the program's
layout. Mixing these silently gives sizes off by exactly six: a KMRP before
1.6.0 draws fonts made for KMRP six times too large.

**4. Tables for data, prose for reasoning.** Every font name, file, setting or slot gets a
row: what it is, its value before, its value now, why. Never bury a value in a sentence.

**5. Quote the code that carries the argument, and name where it is.** The function and
file (`Conversion.cs`, `LoadedFace.Hinted`), and for a COM call the interface and vtable
slot with where the number came from. Line numbers drift; a function name does not. A
lab record may cite line numbers, with the commit they belong to.

**6. Give the rule, then a table of what it produces.** "Original-game sizes are the HD
sizes divided by six" is the rule; the table of 10, 12, 14, 16, 19 and 32 pixels, each
found equal to the game's own, is the evidence that the rule is what ships.

**7. Corroborate where you can, and name the source.** The game's own TXI files, a second
renderer (FreeType against DirectWrite), fontbakery, KMRP's own atlases. Mark it as
corroboration, not as the primary measurement.

**8. Close the loop numerically.** The strongest check in this repository draws KMRP's
committed atlases again from the font files and compares them: 0.01% to 0.04%, the
rounding of 8-bit pixels. Find the check that could prove a claim wrong, and run it.

**9. Record what was rejected, and why.** The alternative that was tried and failed is
what stops the next person retrying it. Say what was appealing about it and what killed
it ([`fonts/README.md`](../fonts/README.md) section 6 is the model).

**10. Mark untested explicitly.** "Seen in a vanilla game at 1280x960; the HUD's numbers,
the end credits and Menu Large not looked at." "Proton: not yet tried." Never imply
coverage you do not have. A reader who finds one overstated claim has to doubt all of
them.

**11. Corrections stay visible.** When a document was wrong, say what it said, what is
true, and why the mistake was made, with the date. Do not silently edit. The mistakes are
load-bearing: they are the reason the rules exist.

**12. Say what is deliberately *not* changed.** A reader needs to tell "left alone on
purpose, here is why" from "never looked at". The font files still carry KMRP's address
as their vendor URL: on purpose, since changing it changes every file's hash.

**13. State the count, and how you established it.** "17 font names, 11 tiles, 34
files", not "the fonts". Assume there is a second copy of anything, and a third: the
KMRP Original HD letters live as OTF files, a TrueType collection, two TrueType copies
compiled into the converter, a copy in KMSE, and KMRP's atlases drawn from them. Finding
one means the search is incomplete, not finished.

**14. End with how to verify by hand.** The commands, the files, and the checks that catch
the mistakes that subject invites. If a reader cannot re-derive your claims without your
machine, the document has failed.

## Skeleton

```markdown
# <Subject>: what it writes, exactly

<One paragraph: what this covers, what it does not, and which document has the rest.>
<A sentence on provenance: measured how, and where untested is marked.>

## The build this describes           <- exe version, length, SHA-256; font hashes; game exe
## 1. What the game reads              <- the files and fields the game uses, measured from its own
## 2. The rule                         <- formulas, then the table they produce
## 3. Every file it writes             <- tables: names, formats, sizes, before and after
## 4. How it is done                   <- the code that carries it, by function and file
## 5. Limits                           <- sizes refused, character sets, platforms untried
## 6. What is deliberately not changed
## 7. Verifying by hand
```

Not every document needs all of it. Every reference needs 1, 3, 6 and 7, or says why a
section does not apply.

## Anti-patterns, all of them real

| Anti-pattern | What happened here |
| --- | --- |
| A sentence true when written, never measured again | "Not yet seen in the game" at the top of the converter's reference after Georgia was seen in it (audit R8, 2026-11-08). |
| A move that left the documents behind | After the converter and the fonts moved here on 2026-11-08, the release zip's `README.txt` still said the source is "in the KMRP repository, in the font-converter folder", and the notices linked to KMRP's `fonts/` folder, which no longer exists. Found and corrected 2026-11-09. |
| A size not checked | The first original-game build looked right at the larger sizes; at 10 pixels ClearType's padding stood Georgia's m apart ("Com bat"), 2026-11-08. Check the smallest size the game uses, not a comfortable one. |
| A stale number in a changelog | KMRP's changelog gave the converter as 727,552 bytes while a 1,047,552-byte build was current (audit R5). |
| A check that could not fail | A shell heredoc turned `·` into a different character in a Python edit, twice while the press kit was made; a search that matches nothing reports "clean". Test a search on a known hit before believing it finds none. |
| Implying coverage | "Works in the game" when one font was seen, in one version, at one resolution. |

## Before you commit a document

- [ ] Every number traced to a measurement, a formula, or a labelled outside source
- [ ] The build stated: exe version and SHA-256, font hashes, the game's exe where it matters
- [ ] Units stated: font units, HD atlas pixels or original pixels
- [ ] Files, names and settings in tables, with before and after
- [ ] The rule, plus the table it produces
- [ ] Anything untested labelled untested
- [ ] Rejected alternatives recorded with their reasoning
- [ ] Corrections visible and dated, not silent
- [ ] "Not changed" section present
- [ ] A reader with Windows and the game can verify it without asking you
- [ ] Plain ASCII where the file is read outside a browser; no em dash anywhere
- [ ] `python .github/scripts/check_links.py` passes
