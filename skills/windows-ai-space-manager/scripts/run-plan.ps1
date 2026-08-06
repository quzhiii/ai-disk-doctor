param(
    [switch]$SafeOnly,
    [string]$Category,
    [string]$RulesRepo,
    [switch]$Json,
    [switch]$Markdown,
    [int]$SkipModifiedWithinMinutes = 30
)

Set-StrictMode -Version Latest
$InvokeAidisk = Join-Path $PSScriptRoot "invoke-aidisk.ps1"

$AidiskArgs = @("plan")

if ($Json) { $AidiskArgs += "--json" }
elseif ($Markdown) { $AidiskArgs += "--markdown" }

if ($SafeOnly) { $AidiskArgs += "--safe-only" }
if ($Category) { $AidiskArgs += @("--category", $Category) }
if ($RulesRepo) { $AidiskArgs += @("--rules-repo", $RulesRepo) }
if ($SkipModifiedWithinMinutes -ge 0) {
    $AidiskArgs += @("--skip-modified-within-minutes", "$SkipModifiedWithinMinutes")
}

& $InvokeAidisk @AidiskArgs
