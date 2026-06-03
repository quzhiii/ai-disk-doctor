param(
    [switch]$SafeOnly,
    [string]$Category,
    [string]$RulesRepo,
    [string]$QuarantineRoot,
    [switch]$Json,
    [switch]$Markdown
)

Set-StrictMode -Version Latest
$AidiskDir = Join-Path $PSScriptRoot "..\..\..\aidisk"

$args = @("run", "--", "clean", "--dry-run")

if ($Json) { $args += "--json" }
elseif ($Markdown) { $args += "--markdown" }

if ($SafeOnly) { $args += "--safe-only" }
if ($Category) { $args += @("--category", $Category) }
if ($RulesRepo) { $args += @("--rules-repo", $RulesRepo) }
if ($QuarantineRoot) { $args += @("--quarantine-root", $QuarantineRoot) }

Push-Location $AidiskDir
try {
    & cargo @args
}
finally {
    Pop-Location
}
