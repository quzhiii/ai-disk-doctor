param(
    [string]$ReportsDir = ".aidisk/reports",
    [string]$OutputDir = ".aidisk/governance",
    [string]$MinGrowth = "1GB",
    [double]$MinGrowthPercent = 30.0,
    [string]$NotifierAdapter = "local-file",
    [string]$WebhookUrl = ""
)

# Usage: .\scripts\governance\run-governance.ps1 -NotifierAdapter local-file
# Usage: .\scripts\governance\run-governance.ps1 -NotifierAdapter webhook -WebhookUrl https://example.test/webhook

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$RepoRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$AidiskDir = Join-Path $RepoRoot "aidisk"
$ResolvedOutputDir = Join-Path $RepoRoot $OutputDir
$DefaultReportsDir = Join-Path $AidiskDir ".aidisk\reports"
$ResolvedReportsDir = Join-Path $RepoRoot $ReportsDir
$GovernanceEventPath = Join-Path $ResolvedOutputDir "governance-event.json"

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

        $AnomalyJsonPath = Join-Path $ResolvedOutputDir "latest-anomaly.json"
        $AnomalyReport = Get-Content -LiteralPath $AnomalyJsonPath -Raw | ConvertFrom-Json
        $EventType = if ($AnomalyReport.summary.anomalies -gt 0) { "anomaly_found" } else { "no_anomaly" }
        @{
            event_type = $EventType
            generated_at = (Get-Date).ToString("o")
            notifier_adapter = $NotifierAdapter
            reports_dir = $ResolvedReportsDir
            output_dir = $ResolvedOutputDir
            min_growth = $MinGrowth
            min_growth_percent = $MinGrowthPercent
            anomaly_summary = $AnomalyReport.summary
            anomaly_report_path = $AnomalyJsonPath
            markdown_report_path = (Join-Path $ResolvedOutputDir "latest-anomaly.md")
        } | ConvertTo-Json -Depth 6 | Out-File -Encoding utf8 $GovernanceEventPath
    }
    catch {
        if ($_.Exception.Message -like "*requires at least two scan snapshots*") {
            "Not enough history yet. anomaly --latest requires at least two scan snapshots." |
                Out-File -Encoding utf8 (Join-Path $ResolvedOutputDir "latest-anomaly-pending.txt")

            @{
                event_type = "pending_history"
                generated_at = (Get-Date).ToString("o")
                notifier_adapter = $NotifierAdapter
                reports_dir = $ResolvedReportsDir
                output_dir = $ResolvedOutputDir
                min_growth = $MinGrowth
                min_growth_percent = $MinGrowthPercent
                message = "anomaly --latest requires at least two scan snapshots"
                pending_note_path = (Join-Path $ResolvedOutputDir "latest-anomaly-pending.txt")
            } | ConvertTo-Json -Depth 6 | Out-File -Encoding utf8 $GovernanceEventPath
        }
        else {
            throw
        }
    }
}
finally {
    Pop-Location
}

if ($NotifierAdapter -eq "webhook") {
    if ([string]::IsNullOrWhiteSpace($WebhookUrl)) {
        throw "Webhook notifier requires -WebhookUrl"
    }

    if (Test-Path -LiteralPath $GovernanceEventPath) {
        $Payload = Get-Content -LiteralPath $GovernanceEventPath -Raw
        Invoke-RestMethod -Method Post -Uri $WebhookUrl -Body $Payload -ContentType "application/json"
    }
    else {
        "Webhook delivery skipped because no governance event artifact exists yet." |
            Out-File -Encoding utf8 (Join-Path $ResolvedOutputDir "webhook-pending.txt")
    }
}
elseif ($NotifierAdapter -ne "local-file") {
    "Notifier adapter '$NotifierAdapter' is reserved for future webhook/IM delivery." |
        Out-File -Encoding utf8 (Join-Path $ResolvedOutputDir "notifier-placeholder.txt")
}
