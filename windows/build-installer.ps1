#Requires -Version 5.1
<#
.SYNOPSIS
    Compile the Inno Setup installer for Invisible Deck Helper.

.DESCRIPTION
    Wraps ISCC.exe, taking the version from Cargo.toml and compiling against a
    portable bundle produced by build-portable-zip.ps1 / .sh. The bundle must
    already exist: the installer packages the exe *plus* python\ and voices\,
    because the app cannot speak without them.

.PARAMETER BundleDir
    The portable bundle folder. Defaults to dist\InvisibleDeckHelper-<version>-windows-x86_64.

.PARAMETER IsccPath
    Path to ISCC.exe. Auto-detected from the usual Inno Setup 6 locations.

.PARAMETER SignTool
    Optional Inno Setup SignTool definition, e.g.
    'signtool sign /fd sha256 /tr http://timestamp.digicert.com /td sha256 $f'
    Requires [Setup] SignTool= to be configured too; see windows\installer.iss.

.EXAMPLE
    .\windows\build-installer.ps1
#>
[CmdletBinding()]
param(
    [string]$BundleDir = '',
    [string]$IsccPath = '',
    [string]$SignTool = ''
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$root = Split-Path -Parent (Split-Path -Parent $PSCommandPath)
Set-Location $root

$match = Select-String -Path (Join-Path $root 'Cargo.toml') -Pattern '^version\s*=\s*"([^"]+)"' | Select-Object -First 1
if (-not $match) { throw 'could not read the version from Cargo.toml' }
$version = $match.Matches[0].Groups[1].Value

if (-not $BundleDir) {
    $BundleDir = Join-Path $root "dist\InvisibleDeckHelper-$version-windows-x86_64"
}
if (-not (Test-Path $BundleDir)) {
    throw "bundle not found: $BundleDir`nRun windows\build-portable-zip.ps1 first."
}

# A bare exe would install a silently mute app — refuse rather than ship that.
foreach ($rel in @('invisible-deck-helper.exe', 'python\python.exe', 'voices\en_GB-alan-medium.onnx')) {
    if (-not (Test-Path (Join-Path $BundleDir $rel))) {
        throw "bundle incomplete, '$rel' is missing from $BundleDir"
    }
}

if (-not $IsccPath) {
    $candidates = @(
        (Join-Path $env:LOCALAPPDATA 'Programs\Inno Setup 6\ISCC.exe'),
        "${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe",
        "$env:ProgramFiles\Inno Setup 6\ISCC.exe"
    )
    $IsccPath = $candidates | Where-Object { Test-Path $_ } | Select-Object -First 1
    if (-not $IsccPath) {
        throw 'ISCC.exe not found. Install Inno Setup 6:  winget install --id JRSoftware.InnoSetup --exact'
    }
}

Write-Host ">> Inno Setup : $IsccPath"
Write-Host ">> version    : $version"
Write-Host ">> bundle     : $BundleDir"

$isccArgs = @("/DMyAppVersion=$version", "/DBundleDir=$BundleDir")
if ($SignTool) { $isccArgs += "/Ssigntool=$SignTool" }
$isccArgs += (Join-Path $root 'windows\installer.iss')

& $IsccPath @isccArgs
if ($LASTEXITCODE -ne 0) { throw "ISCC failed with exit code $LASTEXITCODE" }

$setup = Join-Path $root "dist\InvisibleDeckHelper-$version-windows-x86_64-setup.exe"
if (-not (Test-Path $setup)) { throw "ISCC reported success but $setup is missing" }

$size = [math]::Round((Get-Item $setup).Length / 1MB, 1)
Write-Host ">> wrote $setup ($size MB)"
Write-Host ">> sha256 $((Get-FileHash -Path $setup -Algorithm SHA256).Hash)"
