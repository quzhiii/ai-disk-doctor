param(
    [string]$TaskName = "AI Disk Doctor Governance"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

Get-ScheduledTask -TaskName $TaskName | Out-Null

Write-Host "Starting governance task '$TaskName' now."
Start-ScheduledTask -TaskName $TaskName

$ShowScript = Join-Path $PSScriptRoot "show-governance-task.ps1"
& $ShowScript -TaskName $TaskName
