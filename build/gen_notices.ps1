<#
.SYNOPSIS
    Regenerates THIRD-PARTY-NOTICES.txt from the Rust dependency tree.

.DESCRIPTION
    Runs cargo-about with build\about.toml and the build\about.hbs template.
    The template also carries the hand-written notices that cargo-about cannot
    discover (the RNNoise algorithm/model and the Phosphor icon font).
    Re-run it whenever Cargo.lock changes.

    Requires cargo-about: cargo install --locked cargo-about

.PARAMETER Offline
    Do not access the network (no clearlydefined.io lookups).
#>
[CmdletBinding()]
param(
    [switch] $Offline
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$Root = Split-Path -Parent $PSScriptRoot
$Out = Join-Path $Root 'THIRD-PARTY-NOTICES.txt'

if (-not (Get-Command cargo-about -ErrorAction SilentlyContinue)) {
    throw 'cargo-about not found. Install it with: cargo install --locked cargo-about'
}

$AboutArgs = @(
    'about', 'generate',
    '--manifest-path', (Join-Path $Root 'Cargo.toml'),
    '-c', (Join-Path $PSScriptRoot 'about.toml'),
    '--locked', '--fail',
    '-o', $Out
)
if ($Offline) { $AboutArgs += '--offline' }
$AboutArgs += (Join-Path $PSScriptRoot 'about.hbs')

& cargo @AboutArgs
if ($LASTEXITCODE -ne 0) { throw "cargo about failed (exit $LASTEXITCODE)" }

# Normalize to CRLF so the file reads well in Notepad on Windows.
$Text = [IO.File]::ReadAllText($Out) -replace "`r?`n", "`r`n"
[IO.File]::WriteAllText($Out, $Text, (New-Object Text.UTF8Encoding($false)))

Write-Host "Wrote $Out"
