param(
    [switch]$SafeOnly,
    [string]$Category,
    [switch]$Json,
    [switch]$Markdown,
    [int]$SkipModifiedWithinMinutes = 30
)

Set-StrictMode -Version Latest
$AidiskDir = Join-Path $PSScriptRoot "..\..\..\aidisk"

$args = @("run", "--", "plan")

if ($Json) { $args += "--json" }
elseif ($Markdown) { $args += "--markdown" }

if ($SafeOnly) { $args += "--safe-only" }
if ($Category) { $args += @("--category", $Category) }
if ($SkipModifiedWithinMinutes -ge 0) {
    $args += @("--skip-modified-within-minutes", "$SkipModifiedWithinMinutes")
}

Push-Location $AidiskDir
try {
    & cargo @args
}
finally {
    Pop-Location
}
