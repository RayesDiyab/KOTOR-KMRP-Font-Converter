"""Where a KMRP checkout is, for the few tools here that read KMRP's files.

This repository holds the KMRP Font Converter and the KMRP Original HD fonts; it moved out
of KMRP on 2026-11-08. Some of its tools still read files that are KMRP's: the HD atlases the
fonts were redrawn over (assets/hd-original-fonts), KMRP's font tools (tools/), and the
game's own font atlases a KMRP build copies (build/native-runtime/original-fonts). They find
KMRP here, in this order:

  1. the folder named by the KMRP_ROOT environment variable;
  2. a folder next to this repository named "KMRP - KOTOR Modern Restoration Patch" or
     "KMRP-KOTOR-Modern-Restoration-Patch" (the maintainer's and GitHub's names).

None of this is needed to build the converter or to use the fonts.
"""
import os
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
NAMES = ("KMRP - KOTOR Modern Restoration Patch", "KMRP-KOTOR-Modern-Restoration-Patch")


def kmrp_root(required=True):
    """The KMRP checkout, or None (or an error when `required`) when there is none."""
    named = os.environ.get("KMRP_ROOT")
    candidates = [Path(named)] if named else []
    candidates += [REPO.parent / name for name in NAMES]
    for candidate in candidates:
        if (candidate / "tools").is_dir() and (candidate / "assets").is_dir():
            return candidate
    if required:
        raise SystemExit("A KMRP checkout is needed for this: set KMRP_ROOT to it, or put it next to this "
                         "repository as \"KMRP - KOTOR Modern Restoration Patch\".")
    return None
