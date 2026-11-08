"""Pack the upload zip from fonts/ and this folder's README.txt (staged in build/), then check it."""
import hashlib, shutil, zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]   # this repository
KIT = Path(__file__).resolve().parent
SRC = ROOT / "fonts"
STAGE = ROOT / "build" / "fonts-upload" / "stage" / "KMRP Original HD Fonts"
ZIP = KIT / "KMRP-Original-HD-Fonts-1.600.zip"

(STAGE / "otf").mkdir(parents=True, exist_ok=True)
files = sorted((SRC / "otf").glob("*.otf"))
assert len(files) == 11, files
for f in files:
    shutil.copy2(f, STAGE / "otf" / f.name)
shutil.copy2(SRC / "KMRP-OriginalHD.ttc", STAGE / "KMRP-OriginalHD.ttc")
shutil.copy2(SRC / "OFL.txt", STAGE / "OFL.txt")
shutil.copy2(KIT / "README.txt", STAGE / "README.txt")

readme = (STAGE / "README.txt").read_bytes()
assert all(b < 128 for b in readme), "README.txt has non-ASCII bytes"
# Windows line endings for Notepad users
(STAGE / "README.txt").write_bytes(readme.replace(b"\r\n", b"\n").replace(b"\n", b"\r\n"))

ZIP.unlink(missing_ok=True)
with zipfile.ZipFile(ZIP, "w", zipfile.ZIP_DEFLATED, compresslevel=9) as z:
    for p in sorted(STAGE.rglob("*")):
        if p.is_file():
            z.write(p, p.relative_to(STAGE.parent).as_posix())

sha = lambda b: hashlib.sha256(b).hexdigest()
with zipfile.ZipFile(ZIP) as z:
    assert z.testzip() is None
    names = z.namelist()
    for n in names:
        rel = n.split("/", 1)[1]
        src = SRC / rel if rel != "README.txt" else STAGE / rel
        same = sha(z.read(n)) == sha(src.read_bytes())
        print(f"{'ok ' if same else 'BAD'} {z.getinfo(n).file_size:>8}  {n}")
        assert same
print(len(names), "files;", ZIP.stat().st_size, "bytes;", sha(ZIP.read_bytes()), ZIP.name)
