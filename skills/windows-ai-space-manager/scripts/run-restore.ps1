param(
    [Parameter(Mandatory = $true)][string]$Index,
    [switch]$DryRun,
    [switch]$Yes,
    [switch]$Json,
    [switch]$Markdown
)

Set-StrictMode -Version Latest
$InvokeAidisk = Join-Path $PSScriptRoot "invoke-aidisk.ps1"

$AidiskArgs = @("restore", "--index", $Index)

if ($Json) { $AidiskArgs += "--json" }
elseif ($Markdown) { $AidiskArgs += "--markdown" }

if ($DryRun) { $AidiskArgs += "--dry-run" }
if ($Yes) { $AidiskArgs += "--yes" }

& $InvokeAidisk @AidiskArgs
