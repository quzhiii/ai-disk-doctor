param(
    [string]$Before,
    [string]$After,
    [switch]$Latest,
    [string]$ReportsDir,
    [switch]$Json,
    [switch]$Markdown
)

Set-StrictMode -Version Latest
$InvokeAidisk = Join-Path $PSScriptRoot "invoke-aidisk.ps1"

$AidiskArgs = @("diff")

if ($Json) { $AidiskArgs += "--json" }
elseif ($Markdown) { $AidiskArgs += "--markdown" }

if ($Latest) {
    $AidiskArgs += "--latest"
    if ($ReportsDir) { $AidiskArgs += @("--reports-dir", $ReportsDir) }
}
else {
    if (-not $Before -or -not $After) {
        throw "run-diff.ps1 requires -Before and -After unless -Latest is used"
    }
    $AidiskArgs += @("--before", $Before, "--after", $After)
}

& $InvokeAidisk @AidiskArgs
