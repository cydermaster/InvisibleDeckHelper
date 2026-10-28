#Requires -Version 5.1
<#
.SYNOPSIS
    Build the portable Windows ZIP for Invisible Deck Helper.

.DESCRIPTION
    Produces dist\InvisibleDeckHelper-<version>-windows-x86_64.zip containing:

        invisible-deck-helper.exe       the app, built natively with MSVC
        python\                         embedded CPython + piper-tts and its deps
        voices\en_GB-alan-medium.onnx   the voice, same file the Flatpak uses
        README.txt                      how to run it

    Nothing is installed on the target machine: the folder is self-contained.
    The voice model is fetched from Hugging Face and checked against a pinned
    sha256 — it is deliberately not stored in the repository.

.PARAMETER NumpyPin
    Optional numpy requirement, e.g. 'numpy==1.26.4', to fall back to numpy 1.x
    if a future numpy release fails to load on the target machine.

.EXAMPLE
    .\windows\build-portable-zip.ps1
    .\windows\build-portable-zip.ps1 -NumpyPin 'numpy==1.26.4'
#>
[CmdletBinding()]
param(
    [string]$PythonVersion = '3.11.9',
    [string]$PythonTag = '311',
    [string]$NumpyPin = ''
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

# The voice is downloaded, so pin it: a silent substitution would ship a broken app.
$VoiceFile     = 'en_GB-alan-medium.onnx'
$VoiceSha256   = '0A309668932205E762801F1EFC2736CD4B0120329622ADF62BE09E56339D3330'
$VoiceJsonSha256 = 'C0F0D124E5895C00E7C03B35DCC8287F319A6998A365B182DEB5C8E752EE8C1E'
$VoiceBaseUrl  = 'https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_GB/alan/medium'
$PiperSpec     = 'piper-tts==1.8.0'

$root = Split-Path -Parent (Split-Path -Parent $PSCommandPath)
Set-Location $root

$cargoToml = Join-Path $root 'Cargo.toml'
$match = Select-String -Path $cargoToml -Pattern '^version\s*=\s*"([^"]+)"' | Select-Object -First 1
if (-not $match) { throw "could not read the version from $cargoToml" }
$version = $match.Matches[0].Groups[1].Value

$name = "InvisibleDeckHelper-$version-windows-x86_64"
$work = Join-Path $root '.win-build'
$dist = Join-Path $root 'dist'
$out  = Join-Path $dist $name
$zip  = Join-Path $dist "$name.zip"

Write-Host ">> Invisible Deck Helper $version -> $out"

foreach ($dir in @($work, $out)) {
    if (Test-Path $dir) { Remove-Item -Recurse -Force $dir }
}
New-Item -ItemType Directory -Force -Path $work, (Join-Path $out 'voices') | Out-Null

# ---- 1. build the exe -------------------------------------------------------
Write-Host '>> cargo build --release'
cargo build --release --locked
if ($LASTEXITCODE -ne 0) { throw 'cargo build failed' }
Copy-Item (Join-Path $root 'target\release\invisible-deck-helper.exe') $out -Force

# ---- 2. embedded python -----------------------------------------------------
$embedZip  = Join-Path $work 'python-embed.zip'
$pyembed   = Join-Path $work 'pyembed'
$embedUrl  = "https://www.python.org/ftp/python/$PythonVersion/python-$PythonVersion-embed-amd64.zip"
Write-Host ">> embedded python $PythonVersion"
Invoke-WebRequest -UseBasicParsing -Uri $embedUrl -OutFile $embedZip
Expand-Archive -Path $embedZip -DestinationPath $pyembed -Force

# ---- 3. piper + its wheel closure, targeted at the embedded interpreter ------
# The runner's own Python differs from the embedded 3.11, so the platform/abi
# flags are mandatory: without them pip fetches wheels for the wrong ABI.
$wheels       = Join-Path $work 'wheels'
$sitePackages = Join-Path $pyembed 'Lib\site-packages'
New-Item -ItemType Directory -Force -Path $wheels | Out-Null

$targetArgs = @(
    '--platform', 'win_amd64',
    '--python-version', $PythonTag,
    '--implementation', 'cp',
    '--abi', "cp$PythonTag",
    '--only-binary=:all:'
)
$pin = @()
if ($NumpyPin) { $pin = @($NumpyPin) }

Write-Host ">> fetching $PiperSpec and its dependencies"
python -m pip download --quiet --only-binary=:all: @targetArgs $PiperSpec @pin -d $wheels
if ($LASTEXITCODE -ne 0) { throw 'pip download failed' }

Write-Host '>> installing into the embedded interpreter'
python -m pip install --quiet --no-index --find-links=$wheels --target=$sitePackages @targetArgs $PiperSpec @pin --no-compile
if ($LASTEXITCODE -ne 0) { throw 'pip install failed' }

# --no-compile stops pip writing .pyc for *its own* interpreter (the runner's,
# not the embedded 3.11). Those files are useless here and only bloat the
# bundle, so purge any that slipped in from an earlier run.
Get-ChildItem -Path $sitePackages -Recurse -Directory -Filter '__pycache__' -ErrorAction SilentlyContinue |
    Remove-Item -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item -Recurse -Force (Join-Path $sitePackages 'bin') -ErrorAction SilentlyContinue

# The embedded distribution ignores site-packages until its ._pth lists it.
$pth = Join-Path $pyembed "python$PythonTag._pth"
Set-Content -Path $pth -Encoding ascii -Value @(
    "python$PythonTag.zip",
    '.',
    'Lib\site-packages',
    'import site'
)
Copy-Item -Recurse -Force $pyembed (Join-Path $out 'python')

# ---- 4. voice model + README ------------------------------------------------
Write-Host '>> voice model'
$onnx = Join-Path $out "voices\$VoiceFile"
Invoke-WebRequest -UseBasicParsing -Uri "$VoiceBaseUrl/$VoiceFile" -OutFile $onnx
$onnxHash = (Get-FileHash -Path $onnx -Algorithm SHA256).Hash
if ($onnxHash -ne $VoiceSha256) {
    throw "voice model checksum mismatch:`n  expected $VoiceSha256`n  got      $onnxHash"
}
$json = "$onnx.json"
Invoke-WebRequest -UseBasicParsing -Uri "$VoiceBaseUrl/$VoiceFile.json" -OutFile $json
$jsonHash = (Get-FileHash -Path $json -Algorithm SHA256).Hash
if ($jsonHash -ne $VoiceJsonSha256) {
    throw "voice config checksum mismatch:`n  expected $VoiceJsonSha256`n  got      $jsonHash"
}

$readme = (Get-Content (Join-Path $root 'windows\README-portable.txt') -Raw) -replace '__VERSION__', $version
Set-Content -Path (Join-Path $out 'README.txt') -Value $readme -Encoding utf8

# Licence texts travel with the binaries that need them.
foreach ($doc in @('LICENSE', 'NOTICE')) {
    $src = Join-Path $root $doc
    if (Test-Path $src) { Copy-Item $src $out -Force }
}

# ---- 5. zip -----------------------------------------------------------------
Write-Host '>> zipping'
if (Test-Path $zip) { Remove-Item -Force $zip }
Compress-Archive -Path $out -DestinationPath $zip -CompressionLevel Optimal

$size = [math]::Round((Get-Item $zip).Length / 1MB, 1)
Write-Host ">> wrote $zip ($size MB)"
Write-Host ">> sha256 $((Get-FileHash -Path $zip -Algorithm SHA256).Hash)"
