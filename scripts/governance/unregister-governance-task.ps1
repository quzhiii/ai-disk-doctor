param(
    [string]$TaskName = "AI Disk Doctor Governance"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

Unregister-ScheduledTask -TaskName $TaskName -Confirm:$false
