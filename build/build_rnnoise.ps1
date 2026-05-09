#requires -Version 5.1
<#
.SYNOPSIS
  Builds rnnoise.dll for win-x64 and win-x86 from xiph/rnnoise source.

.DESCRIPTION
  Clones xiph/rnnoise (or reuses a cached clone), downloads the model weights
  blob from media.xiph.org, and compiles a single-DLL build with MSVC for both
  64-bit and 32-bit architectures. The resulting DLLs are placed under
  native/runtimes/{win-x64,win-x86}/native/ where EasyMICBooster.csproj picks
  them up at publish time.

  Requirements:
    - Visual Studio 2022 (Community / Professional / Enterprise) with the
      "Desktop development with C++" workload (provides cl.exe and the SDKs).
    - git on PATH.
    - PowerShell 5.1+ (Windows-built-in is fine).

  Re-runs are cheap: skips clone/download when the cached working tree exists.
  Pass -Clean to force a fresh build.
#>

[CmdletBinding()]
param(
    [switch]$Clean
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$RepoRoot     = Resolve-Path (Join-Path $PSScriptRoot '..')
$WorkRoot     = Join-Path $RepoRoot 'build\rnnoise-src'
$NativeOutX64 = Join-Path $RepoRoot 'native\runtimes\win-x64\native'
$NativeOutX86 = Join-Path $RepoRoot 'native\runtimes\win-x86\native'

$RnnoiseRepo  = 'https://github.com/xiph/rnnoise.git'
$ModelBaseUrl = 'https://media.xiph.org/rnnoise/models'

# Files that compose the library (mirrors RNNOISE_SOURCES in upstream Makefile.am).
$LibSources = @(
    'src\denoise.c'
    'src\rnn.c'
    'src\pitch.c'
    'src\kiss_fft.c'
    'src\celt_lpc.c'
    'src\nnet.c'
    'src\nnet_default.c'
    'src\parse_lpcnet_weights.c'
    'src\rnnoise_data.c'
    'src\rnnoise_tables.c'
)

function Find-VcVarsAll {
    $candidates = @(
        'C:\Program Files\Microsoft Visual Studio\2022\Enterprise\VC\Auxiliary\Build\vcvarsall.bat'
        'C:\Program Files\Microsoft Visual Studio\2022\Professional\VC\Auxiliary\Build\vcvarsall.bat'
        'C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvarsall.bat'
        'C:\Program Files\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvarsall.bat'
        'C:\Program Files (x86)\Microsoft Visual Studio\2019\Enterprise\VC\Auxiliary\Build\vcvarsall.bat'
        'C:\Program Files (x86)\Microsoft Visual Studio\2019\Professional\VC\Auxiliary\Build\vcvarsall.bat'
        'C:\Program Files (x86)\Microsoft Visual Studio\2019\Community\VC\Auxiliary\Build\vcvarsall.bat'
        'C:\Program Files (x86)\Microsoft Visual Studio\2019\BuildTools\VC\Auxiliary\Build\vcvarsall.bat'
    )
    foreach ($path in $candidates) {
        if (Test-Path -LiteralPath $path) { return $path }
    }
    throw "vcvarsall.bat not found. Install Visual Studio with the 'Desktop development with C++' workload."
}

function Ensure-Source {
    if ($Clean -and (Test-Path -LiteralPath $WorkRoot)) {
        Write-Host "[clean] removing $WorkRoot"
        Remove-Item -Recurse -Force -LiteralPath $WorkRoot
    }

    if (-not (Test-Path -LiteralPath $WorkRoot)) {
        Write-Host "[clone] $RnnoiseRepo -> $WorkRoot"
        & git clone --depth 1 $RnnoiseRepo $WorkRoot
        if ($LASTEXITCODE -ne 0) { throw "git clone failed" }
    } else {
        Write-Host "[reuse] $WorkRoot"
    }

    $hashFile = Join-Path $WorkRoot 'model_version'
    if (-not (Test-Path -LiteralPath $hashFile)) {
        throw "model_version not found in cloned repo at $hashFile"
    }
    $modelHash = (Get-Content -Raw -LiteralPath $hashFile).Trim()
    $tarball   = Join-Path $WorkRoot ("rnnoise_data-{0}.tar.gz" -f $modelHash)
    $weightsC  = Join-Path $WorkRoot 'src\rnnoise_data.c'

    if (-not (Test-Path -LiteralPath $weightsC)) {
        if (-not (Test-Path -LiteralPath $tarball)) {
            $url = "$ModelBaseUrl/rnnoise_data-$modelHash.tar.gz"
            Write-Host "[download] $url"
            Invoke-WebRequest -Uri $url -OutFile $tarball -UseBasicParsing
        }
        Write-Host "[extract] $tarball"
        Push-Location $WorkRoot
        try {
            & tar -xzf $tarball
            if ($LASTEXITCODE -ne 0) { throw "tar extract failed" }
        } finally {
            Pop-Location
        }
    }

    if (-not (Test-Path -LiteralPath $weightsC)) {
        throw "rnnoise_data.c missing after model extraction"
    }
}

function Build-Arch {
    param(
        [Parameter(Mandatory)] [ValidateSet('x64', 'x86')] [string] $Arch,
        [Parameter(Mandatory)] [string] $VcVarsAll,
        [Parameter(Mandatory)] [string] $OutDir
    )

    if (-not (Test-Path -LiteralPath $OutDir)) {
        New-Item -ItemType Directory -Path $OutDir -Force | Out-Null
    }

    $sources    = ($LibSources -join ' ')
    $outputDll  = Join-Path $WorkRoot ("rnnoise-{0}.dll" -f $Arch)
    if (Test-Path -LiteralPath $outputDll) { Remove-Item -Force -LiteralPath $outputDll }

    # Run in a single cmd /c invocation so vcvars env vars survive into cl.exe.
    # /MT links the CRT statically so the DLL has no extra runtime dependency.
    $clLine = ("cl.exe /nologo /LD /MT /O2 /W2 /DWIN32 /DRNNOISE_BUILD /DDLL_EXPORT " +
               "/I include /I src $sources /Fe:""$outputDll"" /link /OUT:""$outputDll""")

    $cmd = "call `"$VcVarsAll`" $Arch && $clLine"
    Write-Host "[build $Arch] $clLine"

    Push-Location $WorkRoot
    try {
        & cmd.exe /c $cmd
        if ($LASTEXITCODE -ne 0) { throw "cl.exe failed for $Arch (exit $LASTEXITCODE)" }
    } finally {
        Pop-Location
    }

    if (-not (Test-Path -LiteralPath $outputDll)) {
        throw "Expected $outputDll after build, but it is missing"
    }

    $finalDll = Join-Path $OutDir 'rnnoise.dll'
    Copy-Item -LiteralPath $outputDll -Destination $finalDll -Force
    Write-Host "[place] $finalDll ($([math]::Round((Get-Item $finalDll).Length / 1MB, 1)) MB)"
}

$vcvars = Find-VcVarsAll
Write-Host "[vcvars] $vcvars"

Ensure-Source

Build-Arch -Arch 'x64' -VcVarsAll $vcvars -OutDir $NativeOutX64
Build-Arch -Arch 'x86' -VcVarsAll $vcvars -OutDir $NativeOutX86

Write-Host ""
Write-Host "rnnoise.dll built and placed:"
Write-Host "  $NativeOutX64\rnnoise.dll"
Write-Host "  $NativeOutX86\rnnoise.dll"
