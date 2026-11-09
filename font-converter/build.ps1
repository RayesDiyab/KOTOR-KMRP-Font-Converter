# Builds dist\KMRP Font Converter.exe beside this file: one file, .NET Framework 4 (in every
# Windows since 8, and in Wine's wine-mono under Proton), no other files.
#
#   powershell -ExecutionPolicy Bypass -File font-converter\build.ps1 [-Refresh] [-Release] [-Python python]
#
# Everything it needs is in this folder, so it builds on its own, outside KMRP. The
# examples and font metrics it carries are in data\, made from KMRP's Original HD Font by
# make_data.py (needs Pillow); -Refresh makes them again, from the KMRP checkout this
# folder sits in (assets\hd-original-fonts), before compiling them in as resources.
#
# The compiler: Roslyn's csc from Visual Studio or its Build Tools when vswhere finds one,
# with /deterministic, so the same sources give the same exe and a published SHA-256 can be
# checked by building again (audit R2, 2026-11-08); else the .NET Framework's own csc, whose
# exe differs at every build (it has no /deterministic) and says so.
#
# -Release also writes dist\KMRP-Font-Converter-<version>.zip: the exe, its README.txt, the
# GPL (LICENSE.txt), the SIL Open Font License of the two fonts the window is set in
# (OFL.txt) and THIRD-PARTY-NOTICES.txt (audit B3: until then no licence travelled with it).
param([switch]$Refresh, [switch]$Release, [string]$Python = "python")
$ErrorActionPreference = "Stop"

$here = $PSScriptRoot
$data = Join-Path $here "data"
$dist = Join-Path $here "dist"
$exe = Join-Path $dist "KMRP Font Converter.exe"

$compiler = $null
$deterministic = $false
$vswhere = Join-Path ${env:ProgramFiles(x86)} "Microsoft Visual Studio\Installer\vswhere.exe"
if (Test-Path -LiteralPath $vswhere) {
    $found = & $vswhere -latest -products * -find "MSBuild\**\Roslyn\csc.exe" | Select-Object -First 1
    if ($found -and (Test-Path -LiteralPath $found)) { $compiler = $found; $deterministic = $true }
}
if (-not $compiler) {
    $compiler = "C:\Windows\Microsoft.NET\Framework\v4.0.30319\csc.exe"
    if (-not (Test-Path -LiteralPath $compiler)) { throw "No C# compiler: neither Visual Studio's Roslyn nor the .NET Framework's csc was found" }
    Write-Warning "Visual Studio's C# compiler was not found; built with the .NET Framework's, so this exe's hash is not repeatable."
}

if ($Refresh) {
    & $Python (Join-Path $here "make_data.py") $data
    if ($LASTEXITCODE -ne 0) { throw "make_data.py failed" }
}
New-Item -ItemType Directory -Force -Path $dist | Out-Null

$resources = @(
    "/resource:$data\references.txt,references.txt",
    "/resource:$data\examples.txt,examples.txt",
    "/resource:$here\kmrp-logo.png,kmrp-logo.png",
    # The same lockup saying KOTOR, for fonts made for the original game (2026-11-08):
    # tools/build_brand_image.py --text KOTOR --tracking 0.12 --whole-crest, made at the KMRP
    # logo's scale (its 714 pixel width to 400).
    "/resource:$here\kotor-logo.png,kotor-logo.png",
    # The installer's settings gear (src/patcher/icons/Settings.png, cropped to its ink and
    # made 512 px), line art drawn tinted.
    "/resource:$here\settings.png,settings.png"
)
# KMRP's own fonts for the window's text (SIL OFL 1.1, fonts\OFL.txt), TrueType: GDI+
# cannot load the CFF outlines of the OTF files.
foreach ($ttf in Get-ChildItem -LiteralPath (Join-Path $here "fonts") -Filter *.ttf | Sort-Object Name) {
    $resources += "/resource:$($ttf.FullName),fonts.$($ttf.Name)"
}
foreach ($png in Get-ChildItem -LiteralPath (Join-Path $data "examples") -Filter *.png | Sort-Object Name) {
    $resources += "/resource:$($png.FullName),examples.$($png.Name)"
}

$framework = "C:\Windows\Microsoft.NET\Framework\v4.0.30319"
$references = "System.dll", "System.Drawing.dll", "System.Windows.Forms.dll", "System.Core.dll" | ForEach-Object { "/reference:$framework\$_" }
$sources = Get-ChildItem -LiteralPath $here -Filter *.cs | Sort-Object Name | ForEach-Object { $_.FullName }
$options = @("/nologo", "/optimize+", "/target:winexe", "/platform:anycpu", "/out:$exe", "/win32icon:$here\favicon.ico")
if ($deterministic) { $options += "/deterministic", "/debug-", "/pathmap:$here=font-converter" }
& $compiler @options @references $resources $sources
if ($LASTEXITCODE -ne 0) { throw "The converter did not compile" }

$item = Get-Item -LiteralPath $exe
$hash = (Get-FileHash -LiteralPath $exe -Algorithm SHA256).Hash
Write-Host ("{0}  {1:n0} bytes  SHA-256 {2}{3}" -f $item.FullName, $item.Length, $hash, $(if ($deterministic) { "  (deterministic)" } else { "" }))

if ($Release) {
    $version = [regex]::Match((Get-Content -LiteralPath (Join-Path $here "Program.cs") -Raw), 'const string Version = "([^"]+)"').Groups[1].Value
    if (-not $version) { throw "The version was not found in Program.cs" }
    $stage = Join-Path $dist "KMRP Font Converter $version"
    if (Test-Path -LiteralPath $stage) { Remove-Item -LiteralPath $stage -Recurse -Force }
    New-Item -ItemType Directory -Path $stage | Out-Null
    $root = Split-Path $here -Parent      # this repository
    Copy-Item -LiteralPath $exe -Destination $stage
    Copy-Item -LiteralPath (Join-Path $here "release\README.txt") -Destination $stage
    Copy-Item -LiteralPath (Join-Path $here "release\THIRD-PARTY-NOTICES.txt") -Destination $stage
    Copy-Item -LiteralPath (Join-Path $here "fonts\OFL.txt") -Destination $stage
    Copy-Item -LiteralPath (Join-Path $root "LICENSE") -Destination (Join-Path $stage "LICENSE.txt")
    $zip = Join-Path $dist "KMRP-Font-Converter-$version.zip"
    if (Test-Path -LiteralPath $zip) { Remove-Item -LiteralPath $zip -Force }
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    # The files at the zip's root, no folder (the maintainer, 2026-11-09: "put everything in
    # the root"); until then they were inside "KMRP Font Converter <version>\".
    [System.IO.Compression.ZipFile]::CreateFromDirectory($stage, $zip, [System.IO.Compression.CompressionLevel]::Optimal, $false)
    $zipHash = (Get-FileHash -LiteralPath $zip -Algorithm SHA256).Hash
    Write-Host ("{0}  {1:n0} bytes  SHA-256 {2}" -f $zip, (Get-Item -LiteralPath $zip).Length, $zipHash)
}
