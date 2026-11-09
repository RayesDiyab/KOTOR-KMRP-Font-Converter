# KMRP Font Converter

Turns any font file into Star Wars: Knights of the Old Republic's game fonts, for KMRP
(1.6.0 or later, which sizes them for the screen and offers them as Modded Font) or for
the original game, at the game's own font sizes. One Rust program for Windows, macOS (Apple
silicon and Intel) and Linux (x86_64 and aarch64), nothing installed. Until 2026-11-09 it
was a C# program for Windows alone (.NET Framework 4); the fonts it writes for KMRP are made
by the same arithmetic, measured in
[docs/font-converter.md](docs/font-converter.md) section 9.

- How to use it, where the fonts go and how to undo it: [release/README.txt](release/README.txt),
  the readme in each release package.
- How it works and what it writes: [docs/font-converter.md](docs/font-converter.md).
- Its release audit (of the C# program): [docs/release-audit-2026-11-08.md](docs/release-audit-2026-11-08.md).
- The standard these documents are held to: [docs/documentation-standard.md](../docs/documentation-standard.md).

## Building

This folder builds on its own; nothing outside it is needed but Rust (rustup, stable) and,
for the packages, the tools each script names. The version is `version` in `Cargo.toml`,
the only place it is written.

| System | Command | Makes, in `dist/` |
| --- | --- | --- |
| Windows | `powershell -ExecutionPolicy Bypass -File build.ps1 [-Release]` | `KOTOR-Font-Converter-<version>-Windows-x64.exe`; with `-Release` the `.zip` of the same name |
| macOS | `sh build-macos.sh` | `KOTOR-Font-Converter-<version>-macOS.app` (Universal 2), `.dmg` and `.zip` |
| Linux | `sh build-linux.sh [--arch x86_64\|aarch64] [--zig]` | `KOTOR-Font-Converter-<version>-Linux-<arch>.AppImage` |

Each script runs the unit tests first (`--no-tests` / `-NoTests` skips them), builds with
`--locked`, and checks what it made. The Windows exe is deterministic: the same sources give
the same bytes (`/Brepro`, `.cargo/config.toml`, and the sources' folders mapped to fixed
names). The macOS and Linux scripts were made after KMSE's own (2026-11-09). The workflow
[`.github/workflows/build.yml`](../.github/workflows/build.yml) runs the tests on all three
systems and, run by hand or by a `v*` tag, makes all four packages.

The examples on its tiles and the font metrics it keeps are in `data/`, made by
`make_data.py` from KMRP's `assets/hd-original-fonts`. After that font changes, run
`build.ps1 -Refresh` with a KMRP checkout beside this repository or named by `KMRP_ROOT`
(`tools/kmrp_checkout.py`; needs Python with Pillow) to make them again. `make_icon.py`
makes `favicon.ico` (Windows) and `icon.png` (macOS, Linux) from `font-logo.png`.

`python tests/Test-FontConverter.py`, from this repository's root, builds it and checks
what it writes for every kind of font file and both versions.

GPL-3.0-or-later, like KMRP. Created by RaymanGT.
