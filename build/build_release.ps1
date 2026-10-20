<#
.SYNOPSIS
    Builds the Windows release packages of Easy MIC Booster.

.DESCRIPTION
    Reads the version from version.txt (and fails if Cargo.toml disagrees),
    runs `cargo build --release --locked` for each Windows target, stages
    EasyMICBooster.exe + LICENSE-MIT + LICENSE-APACHE + THIRD-PARTY-NOTICES.txt +
    README.md into
    build\bin\<arch>\, zips it into build\zip\ and, when Inno Setup 6 is
    installed, builds the installer from installer\EasyMICBooster.iss.

    A target whose rustup target is not installed is skipped with a warning
    (add it with `rustup target add <triple>`).

.PARAMETER Arch
    Architectures to build: x64, x86, arm64 (default: all three).

.PARAMETER SkipInstaller
    Only build the portable zips.

.PARAMETER RequireInstaller
    Fail when ISCC.exe is not found instead of skipping the installers (CI).

.EXAMPLE
    powershell -NoProfile -ExecutionPolicy Bypass -File build\build_release.ps1
    powershell -NoProfile -ExecutionPolicy Bypass -File build\build_release.ps1 -Arch x64
#>
[CmdletBinding()]
param(
    [ValidateSet('x64', 'x86', 'arm64')]
    [string[]] $Arch = @('x64', 'x86', 'arm64'),
    [switch] $SkipInstaller,
    [switch] $RequireInstaller
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$Root = Split-Path -Parent $PSScriptRoot
$BuildDir = $PSScriptRoot
$BinDir = Join-Path $BuildDir 'bin'
$ZipDir = Join-Path $BuildDir 'zip'

$Triples = @{
    'x64'   = 'x86_64-pc-windows-msvc'
    'x86'   = 'i686-pc-windows-msvc'
    'arm64' = 'aarch64-pc-windows-msvc'
}
$ExeName = 'EasyMICBooster.exe'
$ShipFiles = @('LICENSE-MIT', 'LICENSE-APACHE', 'THIRD-PARTY-NOTICES.txt', 'README.md')

function Write-Step([string] $Message) {
    Write-Host ''
    Write-Host "==> $Message" -ForegroundColor Cyan
}

# ---------------------------------------------------------------------------
# Version: version.txt is the source of truth; Cargo.toml must match.
# ---------------------------------------------------------------------------
$Version = (Get-Content -Raw (Join-Path $Root 'version.txt')).Trim()
if ($Version -notmatch '^\d+\.\d+\.\d+$') {
    throw "version.txt does not contain a version (x.y.z): '$Version'"
}

$CargoVersion = $null
$InPackage = $false
foreach ($Line in Get-Content (Join-Path $Root 'Cargo.toml')) {
    if ($Line -match '^\s*\[(.+)\]\s*$') { $InPackage = ($Matches[1] -eq 'package'); continue }
    if ($InPackage -and $Line -match '^\s*version\s*=\s*"([^"]+)"') { $CargoVersion = $Matches[1]; break }
}
if ($CargoVersion -ne $Version) {
    throw "Version mismatch: version.txt is '$Version' but Cargo.toml [package] version is '$CargoVersion'. Update Cargo.toml (and Cargo.lock) to match."
}

Write-Host '=========================================='
Write-Host "Easy MIC Booster release build v$Version"
Write-Host '=========================================='

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    throw 'cargo not found. Install Rust from https://rustup.rs/ (with the MSVC build tools).'
}

$Installed = $null
if (Get-Command rustup -ErrorAction SilentlyContinue) {
    $Installed = @(& rustup target list --installed)
}

# ---------------------------------------------------------------------------
# Inno Setup 6 compiler (optional: without it only the zips are built).
# Install with: winget install JRSoftware.InnoSetup
# ---------------------------------------------------------------------------
$Iscc = $null
if (-not $SkipInstaller) {
    $Candidates = @()
    if ($env:LOCALAPPDATA) { $Candidates += Join-Path $env:LOCALAPPDATA 'Programs\Inno Setup 6\ISCC.exe' }
    if (${env:ProgramFiles(x86)}) { $Candidates += Join-Path ${env:ProgramFiles(x86)} 'Inno Setup 6\ISCC.exe' }
    if ($env:ProgramFiles) { $Candidates += Join-Path $env:ProgramFiles 'Inno Setup 6\ISCC.exe' }
    foreach ($C in $Candidates) { if (Test-Path -LiteralPath $C) { $Iscc = $C; break } }
    if (-not $Iscc) {
        $Cmd = Get-Command ISCC.exe -ErrorAction SilentlyContinue
        if ($Cmd) { $Iscc = $Cmd.Source }
    }
    if ($Iscc) {
        Write-Host "Inno Setup: $Iscc"
    } elseif ($RequireInstaller) {
        throw 'Inno Setup 6 (ISCC.exe) not found.'
    } else {
        Write-Warning 'Inno Setup 6 (ISCC.exe) not found; skipping the installers.'
    }
}

New-Item -ItemType Directory -Force -Path $BinDir, $ZipDir | Out-Null

$Outputs = @()
$Skipped = @()
foreach ($A in $Arch) {
    $Triple = $Triples[$A]
    if ($null -ne $Installed -and $Installed -notcontains $Triple) {
        Write-Warning "Rust target $Triple is not installed; skipping $A. Run: rustup target add $Triple"
        $Skipped += $A
        continue
    }

    Write-Step "Building $A ($Triple)"
    Push-Location $Root
    try {
        & cargo build --release --locked --target $Triple
        if ($LASTEXITCODE -ne 0) { throw "cargo build failed for $Triple (exit $LASTEXITCODE)" }
    } finally {
        Pop-Location
    }

    $Stage = Join-Path $BinDir $A
    if (Test-Path -LiteralPath $Stage) { Remove-Item -Recurse -Force -LiteralPath $Stage }
    New-Item -ItemType Directory -Force -Path $Stage | Out-Null
    Copy-Item -LiteralPath (Join-Path $Root "target\$Triple\release\$ExeName") -Destination $Stage
    foreach ($F in $ShipFiles) { Copy-Item -LiteralPath (Join-Path $Root $F) -Destination $Stage }

    Write-Step "Zipping $A"
    $Zip = Join-Path $ZipDir "EasyMICBooster-$A-v$Version-portable.zip"
    if (Test-Path -LiteralPath $Zip) { Remove-Item -Force -LiteralPath $Zip }
    Compress-Archive -Path (Join-Path $Stage '*') -DestinationPath $Zip
    $Outputs += $Zip

    if ($Iscc) {
        Write-Step "Building $A installer"
        # The license page shows one file: both licenses, the choice stated first.
        $LicensePage = Join-Path $BuildDir 'obj\LICENSE-installer.txt'
        New-Item -ItemType Directory -Force -Path (Split-Path $LicensePage) | Out-Null
        $Rule = '=' * 78
        $Page = @(
            'Easy MIC Booster is dual-licensed under the MIT License or the Apache',
            'License, Version 2.0, at your option. Both texts follow.',
            '', $Rule, 'MIT License', $Rule, '',
            (Get-Content -Raw (Join-Path $Root 'LICENSE-MIT')),
            '', $Rule, 'Apache License, Version 2.0', $Rule, '',
            (Get-Content -Raw (Join-Path $Root 'LICENSE-APACHE'))
        ) -join "`r`n"
        [IO.File]::WriteAllText($LicensePage, $Page)
        & $Iscc /Q "/DAppVersion=$Version" "/DArch=$A" "/DPkgDir=$Stage" "/DOutputDir=$ZipDir" "/DLicensePage=$LicensePage" (Join-Path $Root 'installer\EasyMICBooster.iss')
        if ($LASTEXITCODE -ne 0) { throw "Inno Setup failed for $A (exit $LASTEXITCODE)" }
        $Outputs += Join-Path $ZipDir "EasyMICBooster-$A-v$Version-setup.exe"
    }
}

if ($Outputs.Count -eq 0) {
    throw 'Nothing was built (all requested targets were skipped).'
}

Write-Host ''
Write-Host '=========================================='
Write-Host 'Build Success!'
Write-Host '=========================================='
Write-Host 'Output:'
foreach ($O in $Outputs) { Write-Host "  $O" }
if ($Skipped.Count -gt 0) { Write-Warning ("Skipped: " + ($Skipped -join ', ')) }
