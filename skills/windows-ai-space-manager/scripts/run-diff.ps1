param(
    [Parameter(Mandatory = $true)][string]$Before,
    [Parameter(Mandatory = $true)][string]$After,
    [switch]$Json,
    [switch]$Markdown
)

Set-StrictMode -Version Latest
$AidiskDir = Join-Path $PSScriptRoot "..\..\..\aidisk"

$args = @("run", "--", "diff", "--before", $Before, "--after", $After)

if ($Json) { $args += "--json" }
elseif ($Markdown) { $args += "--markdown" }

Push-Location $AidiskDir
try {
    & cargo @args
}
finally {
    Pop-Location
}
