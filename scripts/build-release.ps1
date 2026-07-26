#requires -Version 5.1
<#
.SYNOPSIS
  One-shot release build for Palbox Studio: runs the pre-bundle gate, compiles the
  Tauri app, and produces every distribution artifact in dist/.

.DESCRIPTION
  Outputs (dist/):
    * PalboxStudio-<version>-setup.exe        NSIS installer (Start-menu shortcut,
                                              handles the WebView2 runtime).
    * PalboxStudio-<version>-portable.zip      no-install, run-in-place build; also the
                                              folder a Vortex "tool" tile points at.
    * PalboxStudio-<version>-portable/         the unzipped portable folder.

  Both share one writable user database in %APPDATA% (tags / groups / passive presets),
  so that data survives switching between the installer and the portable build, and
  survives upgrades. The read-only reference DB rides along inside each build.

.PARAMETER SkipGate
  Skip the check / fmt / test / reference-DB gate (faster iteration; not for a real release).
.PARAMETER InstallerOnly
  Build only the NSIS installer.
.PARAMETER PortableOnly
  Build only the portable zip (uses --no-bundle, so it skips the installer compile).
#>
[CmdletBinding()]
param(
  [switch]$SkipGate,
  [switch]$InstallerOnly,
  [switch]$PortableOnly
)

$ErrorActionPreference = 'Stop'
$repo = Split-Path -Parent $PSScriptRoot
Set-Location $repo

function Assert-LastExit($what) {
  if ($LASTEXITCODE -ne 0) { throw "$what failed (exit code $LASTEXITCODE)" }
}
function Section($text) { Write-Host "`n=== $text ===" -ForegroundColor Cyan }

$doInstaller = -not $PortableOnly
$doPortable  = -not $InstallerOnly

# --- version + paths ---------------------------------------------------------
$conf    = Get-Content (Join-Path $repo 'src-tauri/tauri.conf.json') -Raw | ConvertFrom-Json
$version = $conf.version
$dist    = Join-Path $repo 'dist'
# Cargo workspace: the binary + bundle land under the workspace-root target/, not src-tauri/target/.
$exe     = Join-Path $repo 'target/release/palbox.exe'
$refDb   = Join-Path $repo 'data/palbox-reference.db'
New-Item -ItemType Directory -Force -Path $dist | Out-Null

Write-Host "Palbox Studio release build  v$version" -ForegroundColor Green

# --- pre-bundle gate ---------------------------------------------------------
if (-not $SkipGate) {
  Section 'Gate: frontend type-check'
  npm.cmd run check;                        Assert-LastExit 'npm run check'
  Section 'Gate: rustfmt --check'
  cargo fmt --all -- --check;               Assert-LastExit 'cargo fmt --check'
  Section 'Gate: core tests'
  cargo test;                               Assert-LastExit 'cargo test'
  Section 'Gate: reference DB integrity'
  python scripts/build_reference_db.py --check; Assert-LastExit 'reference DB --check'
} else {
  Write-Warning 'Gate skipped (-SkipGate) - do not ship an unverified build.'
}

# --- compile -----------------------------------------------------------------
# tauri build runs `npm run build` (beforeBuildCommand) itself.
if ($doInstaller) {
  Section 'Build: app + NSIS installer'
  npm.cmd run tauri build -- --bundles nsis; Assert-LastExit 'tauri build --bundles nsis'
} elseif ($doPortable) {
  Section 'Build: app exe (no installer bundle)'
  npm.cmd run tauri build -- --no-bundle;    Assert-LastExit 'tauri build --no-bundle'
}
if (-not (Test-Path $exe)) { throw "expected built exe not found: $exe" }

# --- collect the installer ---------------------------------------------------
if ($doInstaller) {
  Section 'Collect: NSIS installer'
  $setup = Get-ChildItem (Join-Path $repo 'target/release/bundle/nsis') -Filter '*-setup.exe' -ErrorAction SilentlyContinue |
             Sort-Object LastWriteTime | Select-Object -Last 1
  if ($null -eq $setup) { throw 'NSIS setup.exe was not produced' }
  $installerOut = Join-Path $dist "PalboxStudio-$version-setup.exe"
  Copy-Item $setup.FullName $installerOut -Force
  Write-Host "  installer  -> $installerOut"
}

# --- assemble the portable / Vortex-tool folder ------------------------------
if ($doPortable) {
  Section 'Assemble: portable + Vortex-tool folder'
  $pdir = Join-Path $dist "PalboxStudio-$version-portable"
  if (Test-Path $pdir) { Remove-Item $pdir -Recurse -Force }
  New-Item -ItemType Directory -Force -Path $pdir | Out-Null
  New-Item -ItemType Directory -Force -Path (Join-Path $pdir 'data') | Out-Null

  # The app resolves BaseDirectory::Resource to its own folder, so the read-only
  # reference DB must sit at <exe folder>/data/palbox-reference.db.
  Copy-Item $exe   (Join-Path $pdir 'Palbox Studio.exe')            -Force
  Copy-Item $refDb (Join-Path $pdir 'data/palbox-reference.db')     -Force
  Copy-Item (Join-Path $repo 'LICENSE') (Join-Path $pdir 'LICENSE.txt') -Force

  # Built as a line array (no here-string) so it is immune to LF/CRLF quirks in
  # Windows PowerShell 5.1.
  $readme = @(
    'Palbox Studio - portable build',
    '==============================',
    '',
    'A standalone Palworld 1.0 Global Palbox editor. This is a normal desktop app, not a',
    'game mod: nothing is installed into Palworld, and nothing is deployed into the game',
    'folder. You just run it, open your Global Palbox save, edit, and save.',
    '',
    'RUN IT',
    '  Double-click "Palbox Studio.exe" in this folder. Keep the whole folder together;',
    '  the app reads its game-reference database from the "data" subfolder next to the exe.',
    '',
    '  (Windows 11 already includes the WebView2 runtime this app uses. On older Windows,',
    '   if it will not start, install "Microsoft Edge WebView2 Runtime" from Microsoft,',
    '   then try again.)',
    '',
    'ADD IT TO VORTEX (optional, ~30 seconds)',
    '  1. Open Vortex and select Palworld so its dashboard is showing.',
    '  2. On the Dashboard tools strip, click the "+ Add Tool" tile, then "New...".',
    '  3. Name:      Palbox Studio',
    '     Target:    the "Palbox Studio.exe" in THIS folder',
    '     Start in:  THIS folder',
    '  4. Save. Palbox Studio now launches from its tile on the Palworld dashboard.',
    '',
    'YOUR SAVE + YOUR DATA',
    '  - Global Palbox save (typical location):',
    '      %LOCALAPPDATA%\Pal\Saved\SaveGames\<your-id>\GlobalPalStorage.sav',
    '  - Close Palworld completely before saving edits.',
    '  - Every save first writes a byte-verified backup into a "PalboxStudio-backups"',
    '    folder next to your save; "Open backup" reveals it.',
    '  - Your tags, groups, and passive presets are stored per-user under %APPDATA%, so',
    '    they persist across updates and between the portable and installer builds.'
  ) -join "`r`n"
  Set-Content -Path (Join-Path $pdir 'READ ME FIRST.txt') -Value $readme -Encoding UTF8

  $zip = Join-Path $dist "PalboxStudio-$version-portable.zip"
  if (Test-Path $zip) { Remove-Item $zip -Force }
  Compress-Archive -Path (Join-Path $pdir '*') -DestinationPath $zip -CompressionLevel Optimal
  Write-Host "  portable   -> $zip"
  Write-Host "  (folder)   -> $pdir"
}

# --- summary -----------------------------------------------------------------
Section 'Done - artifacts in dist/'
Get-ChildItem $dist | Where-Object { $_.Name -like "PalboxStudio-$version*" } |
  ForEach-Object {
    $size = if ($_.PSIsContainer) { '<dir>' } else { '{0:N1} MB' -f ($_.Length / 1MB) }
    Write-Host ('  {0,-46} {1}' -f $_.Name, $size)
  }
