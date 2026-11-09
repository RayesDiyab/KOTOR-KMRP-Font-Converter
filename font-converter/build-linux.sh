#!/bin/sh
# Builds the KMRP Font Converter for Linux as an AppImage, into dist/ beside this file:
#   dist/KOTOR-Font-Converter-<version>-Linux-x86_64.AppImage    Intel and AMD
# (named so since 2026-11-09, the maintainer; until then KMRP-Font-Converter-...)
# x86_64 only since 2026-11-09 (the maintainer: "make linux one build for this repo the
# x86_64 AppImage"); until then --arch aarch64 also made an ARM AppImage, never run.
#
#   sh build-linux.sh                    tests, then the release build, then the AppImage
#   sh build-linux.sh --zig              build with cargo-zigbuild for glibc 2.31, so the
#                                        program runs on older systems and can be built on
#                                        an ARM machine too
#   sh build-linux.sh --no-tests         skips the tests (the release build is the same)
#   sh build-linux.sh --pack-only        packs the program already in build/target/<target>/release
#
# Packed the way KMSE packs its own (its build-linux.sh, after KMRP's Linux installer): an
# AppDir with the program, a launcher (AppRun), a desktop entry and the icon (icon.png), made
# into one file by appimagetool 1.9.1 from the AppImage project's GitHub release, downloaded
# once into build/tools and checked against the SHA-256 GitHub publishes for it.
# appimagetool fetches the AppImage runtime for the architecture it packs for from GitHub.
#
# Needs curl, sha256sum and file (appimagetool stops without it); to build, Rust (rustup, stable) with the target installed, and
# with --zig cargo-zigbuild and zig. The AppImage carries the program alone: the libraries it
# opens (OpenGL or EGL; X11 with Xcursor, Xi and xkbcommon, or Wayland and xkbcommon) are a
# Linux desktop's own and are taken from the system, as AppImages do. File pickers are the
# desktop's own, through its portal (xdg-desktop-portal). Without --zig it runs on Linux with
# the glibc of the machine that built it or newer: build it on Ubuntu 22.04 (glibc 2.35).
set -eu

here=$(cd "$(dirname "$0")" && pwd)
cd "$here"

tests=1
build=1
zig=0
arch=x86_64
while [ $# -gt 0 ]; do
    case "$1" in
        --no-tests) tests=0 ;;
        --pack-only) tests=0; build=0 ;;
        --zig) zig=1 ;;
        *) echo "unknown option: $1" >&2; exit 2 ;;
    esac
    shift
done

fail() { echo "build-linux.sh: $*" >&2; exit 1; }

[ "$(uname)" = "Linux" ] || fail "this script builds the Linux AppImage; on macOS use build-macos.sh, on Windows build.ps1"
[ "$(uname -m)" = "$arch" ] || [ "$zig" = 1 ] || [ "$build" = 0 ] || fail "on $(uname -m) the x86_64 program is built with --zig"
target="$arch-unknown-linux-gnu"
for tool in curl sha256sum file; do
    command -v "$tool" >/dev/null 2>&1 || fail "$tool was not found"
done
if [ "$build" = 1 ]; then
    command -v cargo >/dev/null 2>&1 || fail "cargo was not found"
    [ "$zig" = 0 ] || command -v cargo-zigbuild >/dev/null 2>&1 || fail "cargo-zigbuild was not found (pip install cargo-zigbuild ziglang)"
fi

# (without the carriage return a checkout made on Windows ends its lines with, as under WSL)
version=$(tr -d '\r' < Cargo.toml | sed -n 's/^version = "\(.*\)"/\1/p' | head -n 1)
[ -n "$version" ] || fail "no version in Cargo.toml"
bin="kmrp-font-converter"

# The program carries no path of this machine: the sources' folders become fixed names in
# its panic messages. CARGO_ENCODED_RUSTFLAGS, its flags apart by the 0x1f byte.
cargo_home=${CARGO_HOME:-$HOME/.cargo}
us=$(printf '\037')
export CARGO_ENCODED_RUSTFLAGS="--remap-path-prefix=$here=/kmrp-font-converter$us--remap-path-prefix=$cargo_home=/cargo$us--remap-path-prefix=$HOME=/home"
unset RUSTFLAGS

if [ "$tests" = 1 ]; then
    echo "== tests"
    CARGO_TARGET_DIR="$here/build/target-tests" cargo test --profile tests
fi

if [ "$build" = 1 ]; then
    echo "== release build, $target"
    if [ "$zig" = 1 ]; then
        cargo zigbuild --release --locked --target "$target.2.31"
    else
        cargo build --release --locked --target "$target"
    fi
fi
program="$here/build/target/$target/release/$bin"
[ -f "$program" ] || fail "no program at $program"

echo "== checks"
# nothing outside glibc and libgcc is linked; the window's libraries are opened at run time
if command -v readelf >/dev/null 2>&1; then
    readelf -d "$program" | sed -n 's/.*(NEEDED).*\[\(.*\)\]/\1/p' | while read -r lib; do
        case "$lib" in
            libc.so.*|libm.so.*|libgcc_s.so.*|libpthread.so.*|libdl.so.*|librt.so.*|ld-linux-*) ;;
            *) fail "the program links $lib, which the AppImage does not carry" ;;
        esac
    done
else
    echo "no readelf: the linked libraries are not checked"
fi
if command -v objdump >/dev/null 2>&1; then
    glibc=$(objdump -T "$program" 2>/dev/null | grep -o 'GLIBC_[0-9][0-9.]*' | sed 's/GLIBC_//' | sort -t. -k1,1n -k2,2n -k3,3n | tail -n 1)
    echo "needs glibc ${glibc:-?} or newer"
fi
if strings -a "$program" 2>/dev/null | grep -F -q "$HOME"; then
    fail "the program holds a path under $HOME"
fi
# Where it can run here, it converts without a window, as the test does, for both versions.
if [ "$(uname -m)" = "$arch" ]; then
    check="$here/build/check-linux"
    for v in kmrp original; do
        rm -rf "$check"
        "$program" --convert --exit "--out=$check" "--version=$v" "dialogue=$here/../fonts/otf/KMRP-OriginalHD-Dialogue.otf" \
            || fail "it did not convert for $v"
        [ "$(ls "$check" | wc -l)" = 4 ] || fail "it did not write the 4 files of the dialogue tile for $v"
    done
    rm -rf "$check"
    echo "converts for KMRP and the original game"
else
    echo "built for $arch on $(uname -m): not run here"
fi

echo "== appimagetool"
# The tool for this machine (it packs for either architecture), with GitHub's published SHA-256.
host=$(uname -m)
case "$host" in
    x86_64) host_sha256="ed4ce84f0d9caff66f50bcca6ff6f35aae54ce8135408b3fa33abfc3cb384eb0" ;;
    aarch64) host_sha256="f0837e7448a0c1e4e650a93bb3e85802546e60654ef287576f46c71c126a9158" ;;
    *) fail "appimagetool has no build for $host" ;;
esac
tool="$here/build/tools/appimagetool-$host.AppImage"
tool_url="https://github.com/AppImage/appimagetool/releases/download/1.9.1/appimagetool-$host.AppImage"
if [ ! -f "$tool" ] || ! echo "$host_sha256  $tool" | sha256sum -c --status -; then
    mkdir -p "$(dirname "$tool")"
    curl -fsSL -o "$tool" "$tool_url"
    echo "$host_sha256  $tool" | sha256sum -c - || { rm -f "$tool"; fail "appimagetool's SHA-256 is not the published one"; }
fi
chmod +x "$tool"

echo "== AppDir"
appdir="$here/build/AppDir-$arch"
rm -rf "$appdir"
mkdir -p "$appdir/usr/bin" "$appdir/usr/share/applications" "$appdir/usr/share/icons/hicolor/512x512/apps" "$appdir/usr/share/doc/$bin"
cp "$program" "$appdir/usr/bin/$bin"
chmod 755 "$appdir/usr/bin/$bin"
cp "$here/icon.png" "$appdir/$bin.png"
cp "$here/icon.png" "$appdir/usr/share/icons/hicolor/512x512/apps/$bin.png"
cp "$here/release/README.txt" "$here/release/THIRD-PARTY-NOTICES.txt" "$here/fonts/OFL.txt" "$appdir/usr/share/doc/$bin/"
cp "$here/../LICENSE" "$appdir/usr/share/doc/$bin/LICENSE.txt"
cat > "$appdir/AppRun" <<'EOF'
#!/bin/sh
HERE="$(dirname "$(readlink -f "$0")")"
exec "$HERE/usr/bin/kmrp-font-converter" "$@"
EOF
chmod 755 "$appdir/AppRun"
cat > "$appdir/$bin.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=KMRP Font Converter
GenericName=Font Converter
Comment=Turns any font file into KOTOR's game fonts, for KMRP or the original game
Exec=$bin
Icon=$bin
Categories=Game;Utility;
Terminal=false
EOF
cp "$appdir/$bin.desktop" "$appdir/usr/share/applications/$bin.desktop"

echo "== AppImage"
dist="$here/dist"
out="$dist/KOTOR-Font-Converter-$version-Linux-$arch.AppImage"
mkdir -p "$dist"
rm -f "$out"
# The tool runs from its own extraction, so it needs no FUSE (WSL and CI have none). It
# downloads the AppImage runtime for $arch from the AppImage project's GitHub release; where
# that download fails (it did from WSL on 2026-11-09, GitHub answering 504), a runtime put in
# build/tools/runtime-$arch beforehand (the same release's file) is used instead.
runtime="$here/build/tools/runtime-$arch"
if [ -f "$runtime" ]; then
    ARCH=$arch VERSION="$version" APPIMAGE_EXTRACT_AND_RUN=1 "$tool" --no-appstream --runtime-file "$runtime" "$appdir" "$out"
else
    ARCH=$arch VERSION="$version" APPIMAGE_EXTRACT_AND_RUN=1 "$tool" --no-appstream "$appdir" "$out"
fi
chmod 755 "$out"

echo
printf '%s  %s bytes  sha256 %s\n' "$out" "$(stat -c %s "$out")" "$(sha256sum "$out" | cut -d' ' -f1)"
