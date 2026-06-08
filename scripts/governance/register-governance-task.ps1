param(
    [string]$TaskName = "AI Disk Doctor Governance",
    [string]$DailyAt = "09:00",
    [string]$NotifierAdapter = "local-file",
    [string]$WebhookUrl = ""
)

# Usage: .\scripts\governance\register-governance-task.ps1 -TaskName "AI Disk Doctor Governance" -DailyAt "09:00"

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$RepoRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$GovernanceScript = Join-Path $PSScriptRoot "run-governance.ps1"

$Arguments = @(
    "-NoProfile",
    "-File",
    ('"{0}"' -f $GovernanceScript),
    "-NotifierAdapter",
    $NotifierAdapter
)

if (-not [string]::IsNullOrWhiteSpace($WebhookUrl)) {
    $Arguments += @("-WebhookUrl", $WebhookUrl)
}

$Action = New-ScheduledTaskAction -Execute "pwsh.exe" -Argument ($Arguments -join " ")
$Trigger = New-ScheduledTaskTrigger -Daily -At $DailyAt
$Settings = New-ScheduledTaskSettingsSet -StartWhenAvailable

Register-ScheduledTask -TaskName $TaskName -Action $Action -Trigger $Trigger -Settings $Settings -Description "Run AI Disk Doctor local governance workflow"
