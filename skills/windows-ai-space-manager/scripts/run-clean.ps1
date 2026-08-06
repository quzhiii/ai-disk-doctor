param(
    [switch]$SafeOnly,
    [string]$Category,
    [string]$RulesRepo,
    [Parameter(Mandatory = $true)][string]$QuarantineRoot,
    [switch]$Yes,
    [switch]$Json,
    [switch]$Markdown
)

Set-StrictMode -Version Latest
$InvokeAidisk = Join-Path $PSScriptRoot "invoke-aidisk.ps1"

$AidiskArgs = @("clean")

if ($Json) { $AidiskArgs += "--json" }
elseif ($Markdown) { $AidiskArgs += "--markdown" }

if ($SafeOnly) { $AidiskArgs += "--safe-only" }
if ($Category) { $AidiskArgs += @("--category", $Category) }
if ($RulesRepo) { $AidiskArgs += @("--rules-repo", $RulesRepo) }
if ($Yes) { $AidiskArgs += "--yes" }
$AidiskArgs += @("--quarantine-root", $QuarantineRoot)

& $InvokeAidisk @AidiskArgs
