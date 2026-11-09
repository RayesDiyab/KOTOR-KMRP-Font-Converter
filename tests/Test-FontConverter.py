#!/usr/bin/env python3
"""The KMRP Font Converter, run: every font type into both versions, and what it must refuse.

Builds the converter (unless --no-build): font-converter/build.ps1 -NoTests on Windows, else
cargo build --release. Then runs the program with --convert --exit --out=<an empty folder>,
which converts without a window, and checks what it wrote, as docs
(font-converter/docs/font-converter.md section 7) describe:

  * every input below, for KMRP (--version=kmrp) and the original game (--version=original):
    all 17 .tga/.txi pairs; TGA type 2 at 32 bits, full length, at most 4096 square; the
    TXI's texturewidth the atlas's width, every cell inside the atlas, none zero wide from
    slot 32 up, no comma in a number; the dialogue font's A has ink. For KMRP the atlas is
    the size of KMRP 1.6's own HD Original font's and the TXI header its header (since
    2026-11-09); for the original game the heights a sixth of them, the atlas a power of two
    and the game's own font's size, the TXI header the game's line for line, CR LF (since
    2026-11-09),
    and (when build/native-runtime/original-fonts holds the game's own) fontheight and
    baselineheight equal to the game's;
  * refused, nothing written, exit code 1: random bytes, an empty file, a font with no Latin
    letters, a WOFF whose table claims 2 GB;
  * a .tpc already set aside is not lost: the next one is set aside as -2;
  * a read-only file in the folder stops the conversion before anything changes;
  * the window closed while converting: nothing written, no work folder left (Windows only).

Since 2026-11-09 the converter is a Rust program and this test runs on Windows, macOS and
Linux. Windows' own fonts (Arial, Cambria, Bahnschrift, Georgia) are used where they are;
elsewhere the inputs are made from KMRP's own fonts in this repository, which every machine
has.

No game, Override or setting of the player's is touched: every run writes into a folder
under build/font-converter-check, with --version and --out, which are not remembered.
"""
import argparse
import ctypes
import os
import shutil
import stat
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CONVERTER = ROOT / "font-converter"
WINDOWS = sys.platform == "win32"
EXE = CONVERTER / "dist" / "KMRP Font Converter.exe" if WINDOWS else CONVERTER / "build" / "target" / "release" / "kmrp-font-converter"
WORK = ROOT / "build" / "font-converter-check"
FONTS = Path(os.environ.get("WINDIR", r"C:\Windows")) / "Fonts"
# A TrueType font with every letter the game uses, for the inputs made here: Arial where it is,
# else KMRP's own Menu font, which the converter carries for its window.
SOURCE = FONTS / "arial.ttf" if (FONTS / "arial.ttf").exists() else CONVERTER / "fonts" / "KMRP-OriginalHD-Menu.ttf"
sys.path.insert(0, str(ROOT / "tools"))
from kmrp_checkout import kmrp_root  # noqa: E402
# The game's own font atlases a KMRP build copies: compared against when a KMRP checkout is
# beside this repository and has been built.
_KMRP = kmrp_root(required=False)
ORIGINALS = (_KMRP / "build" / "native-runtime" / "original-fonts") if _KMRP else ROOT / "build" / "no-kmrp"
TILES = ["menu", "menusmall", "menusmallb", "menumedium", "menularge", "dialogue", "descriptions",
         "light", "console", "credits", "creditsb"]
NAMES = ["dialogfont10x10", "dialogfont10x10a", "dialogfont10x10b", "dialogfont12x16", "dialogfont16x16",
         "dialogfont16x16a", "dialogfont16x16b", "dialogfont32x32", "fnt_console", "fnt_credits", "fnt_creditsa",
         "fnt_creditsb", "fnt_d10x10b", "fnt_d16x16", "fnt_d16x16a", "fnt_d16x16b", "fnt_dialog16x16"]
# The order tools/build_scaled_fonts.py FONT_RESREFS gives the game's fonts (kmrfogNN).
GAME_ORDER = ["dialogfont10x10", "dialogfont10x10a", "dialogfont10x10b", "dialogfont12x16", "dialogfont16x16",
              "dialogfont16x16a", "dialogfont16x16b", "dialogfont32x32", "fnt_console", "fnt_credits", "fnt_creditsa",
              "fnt_creditsb", "fnt_d10x10b", "fnt_d16x16", "fnt_d16x16a", "fnt_d16x16b", "fnt_dialog16x16", "fnt_galahad14"]

failures = []


def check(condition, message):
    if not condition:
        failures.append(message)
        print("FAIL", message)
    return condition


def references():
    """The HD Original font's TXI headers, as the converter carries them (data/references.txt)."""
    refs, cur = {}, None
    for line in (CONVERTER / "data" / "references.txt").read_text(encoding="ascii").splitlines():
        p = line.split()
        if line.startswith("font "):
            cur = refs[p[1]] = {"atlas": int(p[2])}
        elif len(p) == 2:
            cur[p[0]] = float(p[1])
    return refs


def originals():
    """The original game's own fonts' atlas widths and TXI header lines (data/originals.txt)."""
    games, cur = {}, None
    for line in (CONVERTER / "data" / "originals.txt").read_text(encoding="ascii").splitlines():
        if line.startswith("font "):
            p = line.split()
            cur = games[p[1]] = {"atlas": int(p[2]), "header": []}
        elif line.startswith("  "):
            cur["header"].append(line.strip())
    return games


GAMES = originals()


def txi(path):
    lines = [l.split() for l in path.read_text(encoding="latin-1").splitlines() if l.strip()]
    head = {l[0]: l[1] for l in lines if len(l) == 2}
    up = next(i for i, l in enumerate(lines) if l[0] == "upperleftcoords")
    lo = next(i for i, l in enumerate(lines) if l[0] == "lowerrightcoords")
    n = int(float(head["numchars"]))
    cells = [tuple(map(float, lines[up + 1 + c][:2])) + tuple(map(float, lines[lo + 1 + c][:2])) for c in range(n)]
    return head, cells, path.read_text(encoding="latin-1")


def game_txi(name):
    path = ORIGINALS / f"kmrfog{GAME_ORDER.index(name):02d}.txi"
    if not path.exists():
        return None
    head = {}
    for line in path.read_text(encoding="latin-1").replace("\r", "\n").splitlines():
        p = line.split()
        if len(p) == 2:
            try:
                head[p[0]] = float(p[1])
            except ValueError:
                pass
    return head


def run(folder, args, timeout=90):
    """The converter with these arguments into `folder`; its exit code."""
    if folder.exists():
        shutil.rmtree(folder, onerror=lambda f, p, e: (os.chmod(p, stat.S_IWRITE), f(p)))
    folder.mkdir(parents=True)
    p = subprocess.run([str(EXE), "--convert", "--exit", f"--out={folder}"] + args, timeout=timeout)
    return p.returncode


def check_set(folder, version, label, refs):
    from PIL import Image
    ok = True
    for name in NAMES:
        tga_path, txi_path = folder / f"{name}.tga", folder / f"{name}.txi"
        if not check(tga_path.exists() and txi_path.exists(), f"{label}: {name} missing"):
            ok = False
            continue
        tga = tga_path.read_bytes()
        w, h = tga[12] | tga[13] << 8, tga[14] | tga[15] << 8
        ok &= check(tga[2] == 2 and tga[16] == 32, f"{label}: {name} TGA type {tga[2]} at {tga[16]} bits")
        ok &= check(len(tga) >= 18 + w * h * 4, f"{label}: {name} TGA short")
        ok &= check(w == h and w <= 4096, f"{label}: {name} atlas {w}x{h}")
        head, cells, text = txi(txi_path)
        ok &= check("," not in text, f"{label}: {name} has a comma in a number")
        ok &= check(abs(float(head["texturewidth"]) * 100 - w) < 0.6, f"{label}: {name} texturewidth {head['texturewidth']} for {w}")
        outside = [c for c, (x0, y0, x1, y1) in enumerate(cells) if not (0 <= x0 <= x1 <= 1.0001 and 0 <= y1 <= y0 <= 1.0001)]
        ok &= check(not outside, f"{label}: {name} cells outside the atlas: {outside[:5]}")
        zero = [c for c, (x0, _, x1, _) in enumerate(cells) if c >= 32 and round((x1 - x0) * w) <= 0]
        ok &= check(not zero, f"{label}: {name} zero-wide cells: {zero[:5]}")
        ref = refs[name]
        divisor = 6 if version == "original" else 1
        for key in ("fontheight", "baselineheight"):
            ok &= check(abs(float(head[key]) - ref[key] / divisor) < 1e-6, f"{label}: {name} {key} {head[key]}, wanted {ref[key] / divisor:.6f}")
        if version == "kmrp":
            # The atlas KMRP 1.6's own HD font has (2026-11-09: the maintainer wants the
            # exact size font files KMRP 1.6 uses), and every header line but texturewidth
            # as its TXI has it.
            ok &= check(w == ref["atlas"], f"{label}: {name} atlas {w}, KMRP's is {ref['atlas']}")
            for key, value in ref.items():
                if key not in ("atlas", "texturewidth"):
                    ok &= check(abs(float(head[key]) - value) < 1e-6, f"{label}: {name} {key} {head.get(key)}, KMRP's is {value}")
        if version == "original":
            ok &= check(w & (w - 1) == 0, f"{label}: {name} atlas {w} not a power of two")
            # The game's own font's atlas size, and its TXI header line for line (only
            # texturewidth is the atlas's), CR LF as the game's (since 2026-11-09).
            game = GAMES[name]
            # Larger only where the letters cannot fit the game's size even side by side
            # with no gap: their cells' total width more than its rows hold (KMRP's own HD
            # Console face's HUD letters, 1,128 pixels against 1,024 in 128 square).
            height = cells[ord("A")][1] - cells[ord("A")][3]
            inked = {c for i, c in enumerate(cells) if i > 32 and round((c[2] - c[0]) * w) > 0}
            need = sum(round((x1 - x0) * w) for x0, _, x1, _ in inked)
            room = (game["atlas"] // max(1, round(height * w))) * game["atlas"]
            wanted = game["atlas"] if need <= room else game["atlas"] * 2
            ok &= check(w == wanted, f"{label}: {name} atlas {w}, the game's is {game['atlas']} (letters {need} px, room {room})")
            raw = txi_path.read_bytes()
            ok &= check(raw.count(b"\r\n") == raw.count(b"\n"), f"{label}: {name} has lines not ended CR LF")
            mine = [l.strip() for l in text.replace("\r", "").splitlines()]
            mine = mine[:next(i for i, l in enumerate(mine) if l.lower().startswith("upperleftcoords"))]
            theirs = [l for l in game["header"] if not l.lower().startswith("texturewidth")]
            ok &= check([l for l in mine if not l.lower().startswith("texturewidth")] == theirs,
                        f"{label}: {name} header differs from the game's")
            game = game_txi(name)
            if game:
                for key in ("fontheight", "baselineheight"):
                    ok &= check(abs(float(head[key]) - game[key]) < 1e-6, f"{label}: {name} {key} {head[key]} against the game's {game[key]}")
        if name == "fnt_d16x16":
            x0, y0, x1, y1 = cells[ord("A")]
            img = Image.open(tga_path)
            box = (round(x0 * w), round((1 - y0) * h), round(x1 * w), round((1 - y1) * h))
            ok &= check(img.crop(box).getchannel("A").getextrema()[1] > 200, f"{label}: the dialogue font's A has no ink")
    leftovers = [p.name for p in folder.iterdir() if p.name.startswith(".kmrp-font-converter-")]
    ok &= check(not leftovers, f"{label}: work folder left: {leftovers}")
    return ok


def inputs():
    """Test fonts made from the system's: WOFF and WOFF2 of Arial, a font with no letters, a
    WOFF lying about its size, random bytes and an empty file."""
    from fontTools.ttLib import TTFont
    from fontTools import subset
    folder = WORK / "inputs"
    folder.mkdir(parents=True, exist_ok=True)
    made = {}
    for flavor in ("woff", "woff2"):
        out = folder / f"arial.{flavor}"
        if not out.exists():
            try:
                f = TTFont(SOURCE)
                f.flavor = flavor
                f.save(out)
            except Exception as e:      # WOFF2 needs the brotli module
                print(f"skip {flavor}: {e}")
                continue
        made[flavor] = out
    digits = folder / "digits-only.ttf"
    if not digits.exists():
        options = subset.Options()
        font = subset.load_font(str(SOURCE), options)
        s = subset.Subsetter(options)
        s.populate(text="0123456789")
        s.subset(font)
        subset.save_font(font, str(digits), options)
    made["digits"] = digits
    if "woff" in made:
        w = bytearray(made["woff"].read_bytes())
        w[44 + 12:44 + 16] = (0x7FFFFFFF).to_bytes(4, "big")
        (folder / "huge.woff").write_bytes(bytes(w))
        made["huge"] = folder / "huge.woff"
    (folder / "random.ttf").write_bytes(os.urandom(65536))
    (folder / "empty.ttf").write_bytes(b"")
    made["random"], made["empty"] = folder / "random.ttf", folder / "empty.ttf"
    return made


def close_while_converting(spec):
    """WM_CLOSE to the window while it converts: it must stop, write nothing and exit."""
    import ctypes.wintypes as wt
    user32 = ctypes.WinDLL("user32")
    folder = WORK / "close"
    if folder.exists():
        shutil.rmtree(folder)
    folder.mkdir(parents=True)
    p = subprocess.Popen([str(EXE), "--convert", f"--out={folder}", "--version=kmrp"] + [f"{t}={spec}" for t in TILES])
    found = []

    @ctypes.WINFUNCTYPE(wt.BOOL, wt.HWND, wt.LPARAM)
    def enum(hwnd, _):
        pid = wt.DWORD()
        user32.GetWindowThreadProcessId(hwnd, ctypes.byref(pid))
        # The window by its title, shown or not: winit keeps a second, visible window of its
        # own ("Winit Thread Event Target"), which WM_CLOSE does nothing to, and eframe shows
        # the program's window only after its first frame, in which the conversion starts
        # (2026-11-09: the visible-window test of the C# program's check found none).
        title = ctypes.create_unicode_buffer(256)
        user32.GetWindowTextW(hwnd, title, 256)
        if pid.value == p.pid and title.value == "KMRP Font Converter":
            found.append(hwnd)
        return True
    deadline = time.time() + 30
    # Close once the conversion has started: the work folder is there.
    while time.time() < deadline and not any(x.name.startswith(".kmrp-font-converter-") for x in folder.iterdir()):
        time.sleep(0.02)
    user32.EnumWindows(enum, 0)
    if found:
        user32.PostMessageW(found[0], 0x0010, 0, 0)   # WM_CLOSE
    try:
        p.wait(timeout=60)
    except subprocess.TimeoutExpired:
        p.kill()
        check(False, "close while converting: the program did not exit")
        return
    left = sorted(x.name for x in folder.iterdir())
    check(bool(found), "close while converting: no window found")
    check(not left, f"close while converting: left in the folder: {left[:6]}")


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--no-build", action="store_true")
    args = parser.parse_args()
    if not args.no_build:
        if WINDOWS:
            subprocess.run(["powershell", "-NoProfile", "-ExecutionPolicy", "Bypass", "-File", str(CONVERTER / "build.ps1"), "-NoTests"], check=True)
        else:
            subprocess.run(["cargo", "build", "--release", "--locked"], cwd=CONVERTER, check=True)
    WORK.mkdir(parents=True, exist_ok=True)
    refs = references()
    made = inputs()
    good = [
        ("arial", str(FONTS / "arial.ttf")),
        ("kmrp-otf-cff", str(ROOT / "fonts" / "otf" / "KMRP-OriginalHD-Dialogue.otf")),
        ("kmrp-ttc", str(ROOT / "fonts" / "KMRP-OriginalHD.ttc")),
        ("kmrp-ttf", str(CONVERTER / "fonts" / "KMRP-OriginalHD-Descriptions.ttf")),
        ("cambria-ttc", str(FONTS / "cambria.ttc") + "#Cambria"),
        ("bahnschrift-variable", str(FONTS / "bahnschrift.ttf")),
        ("georgia", str(FONTS / "georgia.ttf")),
    ] + [(k, str(v)) for k, v in made.items() if k in ("woff", "woff2")]
    for name, spec in good:
        if not Path(spec.split("#")[0]).exists():
            print(f"skip {name}: not on this machine")
            continue
        for version in ("kmrp", "original"):
            label = f"{name} for {version}"
            folder = WORK / f"{name}-{version}"
            t0 = time.time()
            code = run(folder, [f"--version={version}"] + [f"{t}={spec}" for t in TILES])
            if check(code == 0, f"{label}: exit code {code}") and check_set(folder, version, label, refs):
                print(f"ok    {label}: 34 files in {time.time() - t0:.1f} s")
            shutil.rmtree(folder, ignore_errors=True)

    for name in ("random", "empty", "digits", "huge"):
        if name not in made:
            continue
        folder = WORK / f"refuse-{name}"
        code = run(folder, ["--version=kmrp", f"dialogue={made[name]}"], timeout=60)
        left = list(folder.iterdir())
        if check(code == 1 and not left, f"refuse {name}: exit {code}, {len(left)} files written"):
            print(f"ok    refused {name}: nothing written")
        shutil.rmtree(folder, ignore_errors=True)

    # A .tpc set aside earlier is kept; this one is set aside as -2.
    folder = WORK / "tpc"
    shutil.rmtree(folder, ignore_errors=True)
    folder.mkdir(parents=True)
    (folder / "fnt_d16x16.tpc").write_bytes(b"new mod")
    (folder / "fnt_d16x16.tpc.before-font-converter").write_bytes(b"earlier mod")
    code = subprocess.run([str(EXE), "--convert", "--exit", f"--out={folder}", "--version=kmrp", f"dialogue={SOURCE}"], timeout=90).returncode
    if check(code == 0, f"tpc: exit {code}") and \
       check((folder / "fnt_d16x16.tpc.before-font-converter").read_bytes() == b"earlier mod", "tpc: the earlier copy was changed") and \
       check((folder / "fnt_d16x16.tpc.before-font-converter-2").read_bytes() == b"new mod", "tpc: the new one was not set aside as -2") and \
       check(not (folder / "fnt_d16x16.tpc").exists() and (folder / "fnt_d16x16.tga").exists(), "tpc: not replaced"):
        print("ok    a .tpc set aside before is kept; the next one is set aside as -2")
    shutil.rmtree(folder, ignore_errors=True)

    # A read-only file stops the conversion before anything moves.
    folder = WORK / "readonly"
    shutil.rmtree(folder, ignore_errors=True)
    folder.mkdir(parents=True)
    old = folder / "fnt_d16x16.txi"
    old.write_text("old\n")
    os.chmod(old, stat.S_IREAD)
    (folder / "dialogfont16x16.tpc").write_bytes(b"keep")
    code = subprocess.run([str(EXE), "--convert", "--exit", f"--out={folder}", "--version=kmrp",
                           f"menu={SOURCE}", f"dialogue={SOURCE}"], timeout=90).returncode
    names = sorted(p.name for p in folder.iterdir())
    if check(code == 1, f"read-only: exit {code}") and \
       check(names == ["dialogfont16x16.tpc", "fnt_d16x16.txi"], f"read-only: the folder changed: {names}"):
        print("ok    a read-only file stops the conversion with nothing changed")
    os.chmod(old, stat.S_IWRITE)
    shutil.rmtree(folder, ignore_errors=True)

    if WINDOWS:
        close_while_converting(str(SOURCE))
        if not any(f.startswith("close") for f in failures):
            print("ok    closed while converting: stopped, nothing written")
    else:
        print("skip  closed while converting: the check sends WM_CLOSE, Windows only")

    print()
    if failures:
        print(f"{len(failures)} failed")
        return 1
    print("all checks passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
