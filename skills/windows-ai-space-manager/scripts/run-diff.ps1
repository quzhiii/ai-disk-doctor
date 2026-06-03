param(
    [string]$Before,
    [string]$After,
    [switch]$Latest,
    [string]$ReportsDir,
    [switch]$Json,
    [switch]$Markdown
)

Set-StrictMode -Version Latest
$AidiskDir = Join-Path $PSScriptRoot "..\..\..\aidisk"

$args = @("run", "--", "diff")

if ($Json) { $args += "--json" }
elseif ($Markdown) { $args += "--markdown" }

if ($Latest) {
    $args += "--latest"
    if ($ReportsDir) { $args += @("--reports-dir", $ReportsDir) }
}
else {
    if (-not $Before -or -not $After) {
        throw "run-diff.ps1 requires -Before and -After unless -Latest is used"
    }
    $args += @("--before", $Before, "--after", $After)
}

Push-Location $AidiskDir
try {
    & cargo @args
}
finally {
    Pop-Location
}
