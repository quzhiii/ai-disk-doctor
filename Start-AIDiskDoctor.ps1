param(
    [string]$OutputDir = ".aidisk\quickstart",
    [string]$RulesRepo,
    [switch]$NoOpen,
    [switch]$IncludeDoctor,
    [switch]$IncludePlan
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$ScriptRoot = if ($PSScriptRoot) { $PSScriptRoot } else { (Get-Location).Path }
$ExeName = if ([System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform([System.Runtime.InteropServices.OSPlatform]::Windows)) { "aidisk.exe" } else { "aidisk" }
$AidiskDir = Join-Path $ScriptRoot "aidisk"
$CargoManifest = Join-Path $AidiskDir "Cargo.toml"
$Cargo = Get-Command "cargo" -CommandType Application -ErrorAction SilentlyContinue
$ReleaseBinary = Join-Path (Join-Path (Join-Path $AidiskDir "target") "release") $ExeName
$DebugBinary = Join-Path (Join-Path (Join-Path $AidiskDir "target") "debug") $ExeName

function Resolve-OutputPath {
    param([Parameter(Mandatory = $true)][string]$Path)

    if ([System.IO.Path]::IsPathRooted($Path)) {
        return $Path
    }

    return Join-Path $ScriptRoot $Path
}

function Resolve-AIDiskBinary {
    if ($env:AIDISK_EXE -and (Test-Path -LiteralPath $env:AIDISK_EXE)) {
        return $env:AIDISK_EXE
    }

    $SameDirBinary = Join-Path $ScriptRoot $ExeName
    if (Test-Path -LiteralPath $SameDirBinary) {
        return $SameDirBinary
    }

    $PathCommand = Get-Command "aidisk" -CommandType Application -ErrorAction SilentlyContinue
    if ($null -ne $PathCommand) {
        return $PathCommand.Source
    }

    if (Test-AIDiskBinaryCurrent -Path $ReleaseBinary) {
        return $ReleaseBinary
    }

    if (Test-AIDiskBinaryCurrent -Path $DebugBinary) {
        return $DebugBinary
    }

    $BuiltBinary = Build-AIDiskDebug
    if ($null -ne $BuiltBinary) {
        return $BuiltBinary
    }

    throw "aidisk was not found. Install a release package, put aidisk on PATH, set AIDISK_EXE, or install Rust so this script can build it."
}

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

    & $Cargo.Source build --manifest-path $CargoManifest
    if ($LASTEXITCODE -ne 0) {
        throw "cargo build failed while preparing aidisk."
    }
    if (Test-Path -LiteralPath $DebugBinary) {
        return $DebugBinary
    }

    return $null
}

function Invoke-AIDiskReport {
    param(
        [Parameter(Mandatory = $true)][string[]]$CommandArgs,
        [Parameter(Mandatory = $true)][string]$OutputPath
    )

    $Output = & $AidiskExe @CommandArgs 2>&1
    $ExitCode = $LASTEXITCODE
    $Output | Set-Content -Encoding utf8 -LiteralPath $OutputPath

    if ($ExitCode -ne 0) {
        throw "aidisk $($CommandArgs -join ' ') failed. See $OutputPath for details."
    }
}

$AidiskExe = Resolve-AIDiskBinary
$ResolvedOutputDir = Resolve-OutputPath -Path $OutputDir
$ReportsDir = Join-Path $ScriptRoot ".aidisk\reports"
$DashboardPath = Join-Path $ResolvedOutputDir "aidisk-dashboard.html"

New-Item -ItemType Directory -Force -Path $ResolvedOutputDir | Out-Null
New-Item -ItemType Directory -Force -Path $ReportsDir | Out-Null

$CommonArgs = @()
if ($RulesRepo) {
    $CommonArgs += @("--rules-repo", $RulesRepo)
}

Push-Location $ScriptRoot
try {
    Write-Host "Running read-only scan..."
    Invoke-AIDiskReport -CommandArgs (@("scan", "--markdown") + $CommonArgs) -OutputPath (Join-Path $ResolvedOutputDir "scan.md")

    if ($IncludeDoctor) {
        Write-Host "Running optional AI footprint diagnosis..."
        Invoke-AIDiskReport -CommandArgs (@("doctor", "--ai-footprint", "--markdown") + $CommonArgs) -OutputPath (Join-Path $ResolvedOutputDir "doctor-ai-footprint.md")
    }

    if ($IncludePlan) {
        Write-Host "Running optional safe-only cleanup preview..."
        Invoke-AIDiskReport -CommandArgs (@("plan", "--safe-only", "--markdown") + $CommonArgs) -OutputPath (Join-Path $ResolvedOutputDir "safe-cleanup-plan.md")
    }

    Write-Host "Generating local dashboard..."
    Invoke-AIDiskReport -CommandArgs @("visualize", "--html", "--reports-dir", $ReportsDir, "--output", $DashboardPath) -OutputPath (Join-Path $ResolvedOutputDir "dashboard-generation.txt")
}
finally {
    Pop-Location
}

if (-not $NoOpen -and (Test-Path -LiteralPath $DashboardPath)) {
    if ([System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform([System.Runtime.InteropServices.OSPlatform]::Windows)) {
        Start-Process -FilePath $DashboardPath | Out-Null
    }
    elseif (Get-Command "open" -CommandType Application -ErrorAction SilentlyContinue) {
        & open $DashboardPath | Out-Null
    }
    elseif (Get-Command "xdg-open" -CommandType Application -ErrorAction SilentlyContinue) {
        & xdg-open $DashboardPath | Out-Null
    }
}

Write-Host "AI Disk Doctor quickstart complete."
Write-Host "Reports: $ResolvedOutputDir"
Write-Host "Dashboard: $DashboardPath"
Write-Host "No files were cleaned. Real cleanup still requires an explicit aidisk clean --yes command."
