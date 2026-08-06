param(
    [string]$Category,
    [string]$RulesRepo,
    [switch]$Json,
    [switch]$Markdown
)

Set-StrictMode -Version Latest
$InvokeAidisk = Join-Path $PSScriptRoot "invoke-aidisk.ps1"

$AidiskArgs = @("scan")

if ($Json) { $AidiskArgs += "--json" }
elseif ($Markdown) { $AidiskArgs += "--markdown" }

if ($Category) {
    $AidiskArgs += @("--category", $Category)
}
if ($RulesRepo) {
    $AidiskArgs += @("--rules-repo", $RulesRepo)
}

& $InvokeAidisk @AidiskArgs
