#!/bin/sh
# Builds the KMRP Font Converter for macOS into dist/ beside this file:
#   dist/KOTOR-Font-Converter-<version>-macOS.app     the program (Universal 2: Apple silicon and Intel)
#   dist/KOTOR-Font-Converter-<version>-macOS.dmg     the app with a link to Applications, to drag across
#   dist/KOTOR-Font-Converter-<version>-macOS.zip     the app with README.txt, the licences and the notices
#
# The files are named KOTOR-Font-Converter-<version>-<system> since 2026-11-09 (the
# maintainer); the name the menu bar shows is still "KMRP Font Converter".
#
#   sh build-macos.sh                 tests, then both architectures
#   sh build-macos.sh --arm64-only    Apple silicon only (no Intel half)
#   sh build-macos.sh --no-tests      skips the tests (the release build is the same)
#
# Made after KMSE's build-macos.sh (2026-11-09). Needs Rust (rustup, stable) with the
# aarch64-apple-darwin target, and x86_64-apple-darwin for the Intel half
# (`rustup target add x86_64-apple-darwin`); Xcode's command line tools (clang, lipo,
# codesign, iconutil, sips, hdiutil, plutil, ditto).
#
# Signing: with KMRP_SIGN_IDENTITY set to a "Developer ID Application: ..." identity in the
# keychain, the app is signed with it, the hardened runtime and a time stamp; otherwise it is
# signed ad hoc ("-"), which runs on the Mac that built it and, once the player allows it, on
# others. With KMRP_NOTARY_PROFILE also set (a profile stored with `xcrun notarytool
# store-credentials`), the DMG is sent to Apple's notary service and the ticket stapled.
# Nothing is uploaded or published otherwise.
set -eu

here=$(cd "$(dirname "$0")" && pwd)
cd "$here"

universal=1
tests=1
for arg in "$@"; do
    case "$arg" in
        --arm64-only) universal=0 ;;
        --no-tests) tests=0 ;;
        *) echo "unknown option: $arg" >&2; exit 2 ;;
    esac
done

fail() { echo "build-macos.sh: $*" >&2; exit 1; }

[ "$(uname)" = "Darwin" ] || fail "this script builds the macOS app; on Linux use build-linux.sh, on Windows build.ps1"
for tool in cargo rustup lipo codesign iconutil sips hdiutil plutil ditto tiffutil osascript; do
    command -v "$tool" >/dev/null 2>&1 || fail "$tool was not found"
done
targets="aarch64-apple-darwin"
[ "$universal" = 1 ] && targets="$targets x86_64-apple-darwin"
installed=$(rustup target list --installed)
for t in $targets; do
    echo "$installed" | grep -qx "$t" || fail "the Rust target $t is not installed: rustup target add $t"
done

version=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)
[ -n "$version" ] || fail "no version in Cargo.toml"
name="KMRP Font Converter"
exe="KOTOR-Font-Converter"
bundle_id="com.raymangt.kmrp-font-converter"
bin="kmrp-font-converter"
package="KOTOR-Font-Converter-$version-macOS"

# The program carries no path of this Mac: the sources' folders become fixed names in its
# panic messages. CARGO_ENCODED_RUSTFLAGS, its flags apart by the 0x1f byte: RUSTFLAGS splits
# at spaces, and a checkout's folder may have them.
cargo_home=${CARGO_HOME:-$HOME/.cargo}
us=$(printf '\037')
export CARGO_ENCODED_RUSTFLAGS="--remap-path-prefix=$here=/kmrp-font-converter$us--remap-path-prefix=$cargo_home=/cargo$us--remap-path-prefix=$HOME=/home"
unset RUSTFLAGS
# Apple silicon needs macOS 11; the Intel half is built for the same, so the app has one
# minimum. Set before the build: it is part of each binary's header.
export MACOSX_DEPLOYMENT_TARGET=11.0

if [ "$tests" = 1 ]; then
    echo "== tests"
    CARGO_TARGET_DIR="$here/build/target-tests" cargo test --profile tests
fi

for t in $targets; do
    echo "== release build, $t"
    cargo build --release --locked --target "$t"
done

dist="$here/dist"
app="$dist/$package.app"
mkdir -p "$dist"
rm -rf "$app"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"

echo "== bundle"
if [ "$universal" = 1 ]; then
    lipo -create \
        "$here/build/target/aarch64-apple-darwin/release/$bin" \
        "$here/build/target/x86_64-apple-darwin/release/$bin" \
        -output "$app/Contents/MacOS/$exe"
else
    cp "$here/build/target/aarch64-apple-darwin/release/$bin" "$app/Contents/MacOS/$exe"
fi
chmod 755 "$app/Contents/MacOS/$exe"

# The icon, from icon-macos.png (make_icon_macos.py: the sigil above FONT on a starry rounded
# square, 1024 px; since 2026-11-13, as macOS 26 sets a transparent icon on a white plate),
# at every size macOS asks for.
iconset="$here/build/AppIcon.iconset"
rm -rf "$iconset"
mkdir -p "$iconset"
master="$here/icon-macos.png"
for s in 16 32 128 256 512; do
    sips -z "$s" "$s" "$master" --out "$iconset/icon_${s}x${s}.png" >/dev/null
    d=$((s * 2))
    sips -z "$d" "$d" "$master" --out "$iconset/icon_${s}x${s}@2x.png" >/dev/null
done
iconutil -c icns "$iconset" -o "$app/Contents/Resources/AppIcon.icns"
rm -rf "$iconset"

printf 'APPL????' > "$app/Contents/PkgInfo"
cat > "$app/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key><string>en</string>
    <key>CFBundleName</key><string>$name</string>
    <key>CFBundleDisplayName</key><string>$name</string>
    <key>CFBundleIdentifier</key><string>$bundle_id</string>
    <key>CFBundleExecutable</key><string>$exe</string>
    <key>CFBundleIconFile</key><string>AppIcon</string>
    <key>CFBundlePackageType</key><string>APPL</string>
    <key>CFBundleShortVersionString</key><string>$version</string>
    <key>CFBundleVersion</key><string>$version</string>
    <key>CFBundleInfoDictionaryVersion</key><string>6.0</string>
    <key>LSMinimumSystemVersion</key><string>11.0</string>
    <key>LSApplicationCategoryType</key><string>public.app-category.utilities</string>
    <key>NSHighResolutionCapable</key><true/>
    <key>NSSupportsAutomaticGraphicsSwitching</key><true/>
    <key>NSHumanReadableCopyright</key><string>Copyright (c) 2026 RaymanGT. GPL-3.0-or-later.</string>
</dict>
</plist>
PLIST
plutil -lint "$app/Contents/Info.plist" >/dev/null

echo "== signing"
identity=${KMRP_SIGN_IDENTITY:-}
if [ -n "$identity" ]; then
    codesign --force --options runtime --timestamp --sign "$identity" "$app"
else
    echo "no KMRP_SIGN_IDENTITY: signed ad hoc (not for distribution without notarization)"
    codesign --force --sign - "$app"
fi

echo "== checks"
codesign --verify --strict --verbose=1 "$app"
archs=$(lipo -archs "$app/Contents/MacOS/$exe")
echo "architectures: $archs"
echo "$archs" | grep -q arm64 || fail "the program has no arm64 code"
if [ "$universal" = 1 ]; then
    echo "$archs" | grep -q x86_64 || fail "the program has no x86_64 code"
fi
# nothing outside the system's own libraries is loaded, in either half
otool -arch all -L "$app/Contents/MacOS/$exe" | grep -v ':$' | awk '{print $1}' | while read -r lib; do
    case "$lib" in
        /usr/lib/*|/System/Library/*) ;;
        *) fail "the program loads $lib, which a Mac without it lacks" ;;
    esac
done
if strings -a "$app/Contents/MacOS/$exe" | grep -F -q "$HOME"; then
    fail "the program holds a path under $HOME"
fi
[ -f "$app/Contents/Resources/AppIcon.icns" ] || fail "no icon"
# The program converts without a window, as the test does, for both versions.
check="$here/build/check-macos"
for v in kmrp original; do
    rm -rf "$check"
    "$app/Contents/MacOS/$exe" --convert --exit "--out=$check" "--version=$v" "dialogue=$here/../fonts/otf/KMRP-OriginalHD-Dialogue.otf" \
        || fail "it did not convert for $v"
    [ "$(ls "$check" | wc -l | tr -d ' ')" = 4 ] || fail "it did not write the 4 files of the dialogue tile for $v"
done
rm -rf "$check"

echo "== packages"
dmg="$dist/$package.dmg"
zip="$dist/$package.zip"
rm -f "$dmg" "$zip"
stage="$here/build/dmg"
rm -rf "$stage"
mkdir -p "$stage"
ditto "$app" "$stage/$package.app"
ln -s /Applications "$stage/Applications"
cp "$here/release/README.txt" "$here/release/THIRD-PARTY-NOTICES.txt" "$here/fonts/OFL.txt" "$stage/"
cp "$here/../LICENSE" "$stage/LICENSE.txt"
# The DMG's window (since 2026-11-13): the converter's own background behind the app and
# the Applications link, drawn by release/make_dmg_background.py, whose docstring gives the
# icons' places used here. The image is made a writable disk first, Finder lays the window
# out (it asks once to be controlled by the terminal), and the disk is then compressed.
volume="$name $version"
rw="$here/build/dmg-rw.dmg"
mnt="$here/build/dmg-mount"
rm -f "$rw"
[ -d "$mnt" ] && { hdiutil detach -quiet -force "$mnt" 2>/dev/null || true; }
rm -rf "$mnt"
# Finder names the disk by its volume name: an earlier DMG of this version still open would
# take the layout instead (seen 2026-11-13), so it is ejected first.
if [ -d "/Volumes/$volume" ]; then
    hdiutil detach -quiet "/Volumes/$volume" || fail "an older \"$volume\" is open; eject it and build again"
fi
hdiutil create -quiet -volname "$volume" -srcfolder "$stage" -fs HFS+ -format UDRW -size 80m -ov "$rw"
mkdir -p "$mnt"
hdiutil attach -quiet -readwrite -noverify -noautoopen -mountpoint "$mnt" "$rw"
detach() { hdiutil detach -quiet "$mnt" 2>/dev/null || hdiutil detach -quiet -force "$mnt"; }
trap detach EXIT
mkdir "$mnt/.background"
tiffutil -cathidpicheck "$here/release/dmg-background.png" "$here/release/dmg-background@2x.png" \
    -out "$mnt/.background/background.tiff" >/dev/null 2>&1
layout="$here/build/dmg-layout.applescript"
cat > "$layout" <<'APPLESCRIPT'
on run argv
    set appName to item 1 of argv
    set mountFolder to (POSIX file (item 2 of argv)) as alias
    tell application "Finder"
        set f to mountFolder
        open f
        delay 2
        set w to container window of f
        set current view of w to icon view
        set toolbar visible of w to false
        set statusbar visible of w to false
        set the bounds of w to {200, 120, 920, 652}
        set o to icon view options of w
        set arrangement of o to not arranged
        set icon size of o to 88
        set text size of o to 12
        set background picture of o to file ".background:background.tiff" of f
        set position of item appName of f to {200, 190}
        set position of item "Applications" of f to {520, 190}
        set position of item "README.txt" of f to {120, 400}
        set position of item "LICENSE.txt" of f to {280, 400}
        set position of item "OFL.txt" of f to {440, 400}
        set position of item "THIRD-PARTY-NOTICES.txt" of f to {600, 400}
        update f without registering applications
        delay 1
        close w
    end tell
end run
APPLESCRIPT
# Finder sometimes refuses a window it has not finished opening (seen 2026-11-13): up to
# three tries before giving up.
tries=0
until osascript "$layout" "$package.app" "$mnt"; do
    tries=$((tries + 1))
    [ "$tries" -lt 3 ] || fail "Finder could not lay out the DMG's window (allow the terminal to control Finder in System Settings, Privacy and Security, Automation)"
    echo "Finder was not ready; trying again"
    sleep 2
done
rm -f "$layout"
[ -f "$mnt/.DS_Store" ] || fail "Finder wrote no layout for the DMG's window"
chflags hidden "$mnt/.background"
sync
detach
trap - EXIT
hdiutil convert -quiet "$rw" -format UDZO -imagekey zlib-level=9 -o "$dmg"
rm -f "$rw"
rm -rf "$mnt"
if [ -n "$identity" ]; then
    codesign --force --timestamp --sign "$identity" "$dmg"
    if [ -n "${KMRP_NOTARY_PROFILE:-}" ]; then
        xcrun notarytool submit "$dmg" --keychain-profile "$KMRP_NOTARY_PROFILE" --wait
        xcrun stapler staple "$dmg"
        xcrun stapler staple "$app"
        rm -rf "$stage/$package.app"
        ditto "$app" "$stage/$package.app"
    fi
fi
# The zip: the same files as the DMG without the Applications link, at its root.
rm "$stage/Applications"
(cd "$stage" && ditto -c -k --sequesterRsrc . "$zip")
rm -rf "$stage"

echo
for f in "$app/Contents/MacOS/$exe" "$dmg" "$zip"; do
    printf '%s  %s bytes  sha256 %s\n' "$f" "$(stat -f %z "$f")" "$(shasum -a 256 "$f" | cut -d' ' -f1)"
done
