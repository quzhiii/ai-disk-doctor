param(
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$AidiskArgs
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..")).Path
$AidiskDir = Join-Path $RepoRoot "aidisk"
$CargoManifest = Join-Path $AidiskDir "Cargo.toml"
$Cargo = Get-Command "cargo" -CommandType Application -ErrorAction SilentlyContinue
$ExeName = if ([System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform([System.Runtime.InteropServices.OSPlatform]::Windows)) { "aidisk.exe" } else { "aidisk" }
$ReleaseBinary = Join-Path (Join-Path (Join-Path $AidiskDir "target") "release") $ExeName
$DebugBinary = Join-Path (Join-Path (Join-Path $AidiskDir "target") "debug") $ExeName

function Get-LocalSourceNewestUtc {
    if (-not (Test-Path -LiteralPath $CargoManifest)) {
        return [datetime]::MinValue
    }

    $Newest = (Get-Item -LiteralPath $CargoManifest).LastWriteTimeUtc
    $CargoLock = Join-Path $AidiskDir "Cargo.lock"
    if (Test-Path -LiteralPath $CargoLock) {
        $CargoLockTime = (Get-Item -LiteralPath $CargoLock).LastWriteTimeUtc
        if ($CargoLockTime -gt $Newest) { $Newest = $CargoLockTime }
    }

    foreach ($RelativePath in @("src", "rules", "config")) {
        $Path = Join-Path $AidiskDir $RelativePath
        if (Test-Path -LiteralPath $Path) {
            foreach ($File in Get-ChildItem -LiteralPath $Path -Recurse -File) {
                if ($File.LastWriteTimeUtc -gt $Newest) { $Newest = $File.LastWriteTimeUtc }
            }
        }
    }

    return $Newest
}

function Test-AIDiskBinaryCurrent {
    param([Parameter(Mandatory = $true)][string]$Path)

    if (-not (Test-Path -LiteralPath $Path)) {
        return $false
    }
    if (-not (Test-Path -LiteralPath $CargoManifest) -or $null -eq $Cargo) {
        return $true
    }

    return (Get-Item -LiteralPath $Path).LastWriteTimeUtc -ge (Get-LocalSourceNewestUtc)
}

function Build-AIDiskDebug {
    if ($null -eq $Cargo -or -not (Test-Path -LiteralPath $CargoManifest)) {
        return $null
    }

    Push-Location $RepoRoot
    try {
        & $Cargo.Source build --manifest-path $CargoManifest
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    }
    finally {
        Pop-Location
    }

    if (Test-Path -LiteralPath $DebugBinary) {
        return $DebugBinary
    }

    return $null
}

function Invoke-AIDiskBinary {
    param([Parameter(Mandatory = $true)][string]$Path)

    Push-Location $RepoRoot
    try {
        & $Path @AidiskArgs
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    }
    finally {
        Pop-Location
    }
}

if ($env:AIDISK_EXE -and (Test-Path -LiteralPath $env:AIDISK_EXE)) {
    Invoke-AIDiskBinary -Path $env:AIDISK_EXE
    return
}

$PathCommand = Get-Command "aidisk" -CommandType Application -ErrorAction SilentlyContinue
if ($null -ne $PathCommand) {
    Invoke-AIDiskBinary -Path $PathCommand.Source
    return
}

if (Test-AIDiskBinaryCurrent -Path $ReleaseBinary) {
    Invoke-AIDiskBinary -Path $ReleaseBinary
    return
}

if (Test-AIDiskBinaryCurrent -Path $DebugBinary) {
    Invoke-AIDiskBinary -Path $DebugBinary
    return
}

$BuiltBinary = Build-AIDiskDebug
if ($null -ne $BuiltBinary) {
    Invoke-AIDiskBinary -Path $BuiltBinary
    return
}

throw "aidisk binary was not found. Install aidisk on PATH, set AIDISK_EXE, or run Start-AIDiskDoctor.ps1 from the repository root."
