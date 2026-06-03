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
$AidiskDir = Join-Path $PSScriptRoot "..\..\..\aidisk"

$args = @("run", "--", "clean")

if ($Json) { $args += "--json" }
elseif ($Markdown) { $args += "--markdown" }

if ($SafeOnly) { $args += "--safe-only" }
if ($Category) { $args += @("--category", $Category) }
if ($RulesRepo) { $args += @("--rules-repo", $RulesRepo) }
if ($Yes) { $args += "--yes" }
$args += @("--quarantine-root", $QuarantineRoot)

Push-Location $AidiskDir
try {
    & cargo @args
}
finally {
    Pop-Location
}
