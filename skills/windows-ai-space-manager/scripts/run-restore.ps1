param(
    [Parameter(Mandatory = $true)][string]$Index,
    [switch]$DryRun,
    [switch]$Yes,
    [switch]$Json,
    [switch]$Markdown
)

Set-StrictMode -Version Latest
$AidiskDir = Join-Path $PSScriptRoot "..\..\..\aidisk"

$args = @("run", "--", "restore", "--index", $Index)

if ($Json) { $args += "--json" }
elseif ($Markdown) { $args += "--markdown" }

if ($DryRun) { $args += "--dry-run" }
if ($Yes) { $args += "--yes" }

Push-Location $AidiskDir
try {
    & cargo @args
}
finally {
    Pop-Location
}
