# KOTOR-KMRP Font Converter: rules for agents

This repository holds the KMRP Font Converter and the KMRP Original HD fonts. It came out of
KMRP's repository on 2026-11-08, and KMRP's `AGENTS.md` is the fuller contract these rules
are drawn from: https://github.com/RayesDiyab/KMRP-KOTOR-Modern-Restoration-Patch/blob/master/AGENTS.md

1. **No commit or push without the maintainer's explicit permission, each time.**
   Permission to commit is not permission to push, and neither carries over to the next.
2. **No AI attribution** in commit messages: no `Co-Authored-By`, no "Generated with".
   Author identity: `Rayes Diyab <114070898+RayesDiyab@users.noreply.github.com>`.
3. **Never commit game files**: no atlas, texture or save of the game's, and no picture made
   from the game's own letters without asking. The fonts in `fonts/` are KMRP's own.
4. **Measure, then claim.** Docs follow [`docs/documentation-standard.md`](docs/documentation-standard.md),
   imported from KMRP's: measured claims only, corrections kept visible with their date,
   anything untested said to be untested. Run `python .github/scripts/check_links.py`
   before committing a document.
5. **Test before saying it works:** `python tests\Test-FontConverter.py` after any change to
   `font-converter/` (it builds the Rust program and converts every font type for both
   versions); for the fonts, KMRP's `tools/build_hd_original_atlases.py --check`. A claim
   about macOS or Linux names where it was seen (CI, a Mac, WSL), or says it was not.
6. **Plain ASCII in sources**; non-ASCII characters in Rust strings as `\u{XXXX}` (in the
   C# program, until 2026-11-09, `\uXXXX`). Write files with backslashes (Windows paths,
   regexes) with a file-writing tool, never a shell heredoc.
7. **Plain words**: no em dash in docs, commits or replies.
