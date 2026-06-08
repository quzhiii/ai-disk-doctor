param(
    [string]$ReportsDir = ".aidisk/reports",
    [string]$OutputDir = ".aidisk/governance",
    [string]$MinGrowth = "1GB",
    [double]$MinGrowthPercent = 30.0,
    [string]$NotifierAdapter = "local-file"
)

# Usage: .\scripts\governance\run-governance.ps1 -NotifierAdapter local-file

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$RepoRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$AidiskDir = Join-Path $RepoRoot "aidisk"
$ResolvedOutputDir = Join-Path $RepoRoot $OutputDir
$DefaultReportsDir = Join-Path $AidiskDir ".aidisk\reports"
$ResolvedReportsDir = Join-Path $RepoRoot $ReportsDir

New-Item -ItemType Directory -Force -Path $ResolvedOutputDir | Out-Null
New-Item -ItemType Directory -Force -Path $ResolvedReportsDir | Out-Null

Push-Location $AidiskDir
try {
    cargo run -- scan --json | Out-File -Encoding utf8 (Join-Path $ResolvedOutputDir "latest-scan.json")

    $LatestSnapshot = Get-ChildItem -LiteralPath $DefaultReportsDir -Filter "scan-*.json" |
        Sort-Object Name |
        Select-Object -Last 1
    if ($null -ne $LatestSnapshot -and ($ResolvedReportsDir -ne $DefaultReportsDir)) {
        Copy-Item -LiteralPath $LatestSnapshot.FullName -Destination (Join-Path $ResolvedReportsDir $LatestSnapshot.Name) -Force
    }

    try {
        cargo run -- anomaly --latest --reports-dir $ResolvedReportsDir --min-growth $MinGrowth --min-growth-percent $MinGrowthPercent --json |
            Out-File -Encoding utf8 (Join-Path $ResolvedOutputDir "latest-anomaly.json")

        cargo run -- anomaly --latest --reports-dir $ResolvedReportsDir --min-growth $MinGrowth --min-growth-percent $MinGrowthPercent --markdown |
            Out-File -Encoding utf8 (Join-Path $ResolvedOutputDir "latest-anomaly.md")
    }
    catch {
        if ($_.Exception.Message -like "*requires at least two scan snapshots*") {
            "Not enough history yet. anomaly --latest requires at least two scan snapshots." |
                Out-File -Encoding utf8 (Join-Path $ResolvedOutputDir "latest-anomaly-pending.txt")
        }
        else {
            throw
        }
    }
}
finally {
    Pop-Location
}

if ($NotifierAdapter -ne "local-file") {
    "Notifier adapter '$NotifierAdapter' is reserved for future webhook/IM delivery." |
        Out-File -Encoding utf8 (Join-Path $ResolvedOutputDir "notifier-placeholder.txt")
}
