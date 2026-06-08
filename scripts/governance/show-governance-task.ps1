param(
    [string]$TaskName = "AI Disk Doctor Governance"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$Task = Get-ScheduledTask -TaskName $TaskName
$Info = Get-ScheduledTaskInfo -TaskName $TaskName

[pscustomobject]@{
    TaskName = $Task.TaskName
    State = $Task.State
    LastRunTime = $Info.LastRunTime
    NextRunTime = $Info.NextRunTime
}
