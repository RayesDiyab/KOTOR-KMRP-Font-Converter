# Builds the KMRP Font Converter for Windows into dist\ beside this file:
#   dist\KOTOR-Font-Converter-<version>-Windows-x64.exe   the program, one file, nothing installed
#   dist\KOTOR-Font-Converter-<version>-Windows-x64.zip   with -Release: the exe, README.txt,
#                                                          LICENSE.txt, OFL.txt and the notices
#
#   powershell -ExecutionPolicy Bypass -File font-converter\build.ps1 [-NoTests] [-Release] [-Refresh] [-Python python]
#
# Since 2026-11-09 the converter is a Rust program (until then C#, .NET Framework 4, built by
# this script with csc); build-macos.sh and build-linux.sh make the macOS app and the Linux
# AppImages from the same sources. Needs Rust (rustup, stable, the MSVC toolchain) and Visual
# Studio's C++ build tools or Build Tools (the linker; rc.exe from the Windows SDK puts the icon
# in the exe, and without it the exe has none and the build says so).
#
# Deterministic: the linker writes no time stamp (/Brepro, .cargo\config.toml) and the
# sources' folders become fixed names, so the same sources give the same exe and a published
# SHA-256 can be checked by building again (audit R2).
#
# -Refresh makes data\ again from a KMRP checkout beside this repository or named by KMRP_ROOT
# (make_data.py; needs Pillow) before building.
param([switch]$NoTests, [switch]$Release, [switch]$Refresh, [string]$Python = "python")
$ErrorActionPreference = "Stop"

$here = $PSScriptRoot
$dist = Join-Path $here "dist"
Push-Location -LiteralPath $here
try {
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) { throw "cargo was not found: install Rust with rustup" }
    $version = [regex]::Match((Get-Content -LiteralPath (Join-Path $here "Cargo.toml") -Raw), '(?m)^version = "([^"]+)"').Groups[1].Value
    if (-not $version) { throw "No version in Cargo.toml" }
    # Every package's files are named KOTOR-Font-Converter-<version>-<system> (2026-11-09, the
    # maintainer: "call exes from now on KOTOR-Font-Converter-1.0.0-Windows-x64.exe and same
    # for rest"); until then "KMRP Font Converter.exe" and KMRP-Font-Converter-<version>-*.
    $exe = Join-Path $dist "KOTOR-Font-Converter-$version-Windows-x64.exe"

    if ($Refresh) {
        & $Python (Join-Path $here "make_data.py") (Join-Path $here "data")
        if ($LASTEXITCODE -ne 0) { throw "make_data.py failed" }
    }

    # No path of this machine in the program: the sources' folders become fixed names in its
    # panic messages. CARGO_ENCODED_RUSTFLAGS, its flags apart by the 0x1f byte, as a folder
    # may have spaces; the /Brepro of .cargo\config.toml goes with them, as these replace it.
    $cargoHome = if ($env:CARGO_HOME) { $env:CARGO_HOME } else { Join-Path $env:USERPROFILE ".cargo" }
    $us = [char]0x1f
    $env:CARGO_ENCODED_RUSTFLAGS = "--remap-path-prefix=$here=/kmrp-font-converter$us--remap-path-prefix=$cargoHome=/cargo$us-C${us}link-arg=/Brepro"
    Remove-Item Env:RUSTFLAGS -ErrorAction SilentlyContinue

    if (-not $NoTests) {
        Write-Host "== tests"
        $env:CARGO_TARGET_DIR = Join-Path $here "build\target-tests"
        cargo test --profile tests
        if ($LASTEXITCODE -ne 0) { throw "The tests failed" }
        Remove-Item Env:CARGO_TARGET_DIR
    }

    Write-Host "== release build"
    cargo build --release --locked
    if ($LASTEXITCODE -ne 0) { throw "The converter did not compile" }

    New-Item -ItemType Directory -Force -Path $dist | Out-Null
    Copy-Item -LiteralPath (Join-Path $here "build\target\release\kmrp-font-converter.exe") -Destination $exe -Force
    $item = Get-Item -LiteralPath $exe
    $hash = (Get-FileHash -LiteralPath $exe -Algorithm SHA256).Hash
    Write-Host ("{0}  {1:n0} bytes  SHA-256 {2}" -f $item.FullName, $item.Length, $hash)

    if ($Release) {
        $stage = Join-Path $here "build\stage-windows"
        if (Test-Path -LiteralPath $stage) { Remove-Item -LiteralPath $stage -Recurse -Force }
        New-Item -ItemType Directory -Path $stage | Out-Null
        $root = Split-Path $here -Parent      # this repository
        Copy-Item -LiteralPath $exe -Destination $stage
        Copy-Item -LiteralPath (Join-Path $here "release\README.txt") -Destination $stage
        Copy-Item -LiteralPath (Join-Path $here "release\THIRD-PARTY-NOTICES.txt") -Destination $stage
        Copy-Item -LiteralPath (Join-Path $here "fonts\OFL.txt") -Destination $stage
        Copy-Item -LiteralPath (Join-Path $root "LICENSE") -Destination (Join-Path $stage "LICENSE.txt")
        $zip = Join-Path $dist "KOTOR-Font-Converter-$version-Windows-x64.zip"
        if (Test-Path -LiteralPath $zip) { Remove-Item -LiteralPath $zip -Force }
        Add-Type -AssemblyName System.IO.Compression.FileSystem
        # The files at the zip's root, no folder (the maintainer, 2026-11-09: "put everything in
        # the root").
        [System.IO.Compression.ZipFile]::CreateFromDirectory($stage, $zip, [System.IO.Compression.CompressionLevel]::Optimal, $false)
        Remove-Item -LiteralPath $stage -Recurse -Force
        $zipHash = (Get-FileHash -LiteralPath $zip -Algorithm SHA256).Hash
        Write-Host ("{0}  {1:n0} bytes  SHA-256 {2}" -f $zip, (Get-Item -LiteralPath $zip).Length, $zipHash)
    }
}
finally {
    Pop-Location
}
