param(
    [string]$Category,
    [string]$RulesRepo,
    [switch]$Json,
    [switch]$Markdown
)

Set-StrictMode -Version Latest
$AidiskDir = Join-Path $PSScriptRoot "..\..\..\aidisk"

$args = @("run", "--", "scan")

if ($Json) { $args += "--json" }
elseif ($Markdown) { $args += "--markdown" }

if ($Category) {
    $args += @("--category", $Category)
}
if ($RulesRepo) {
    $args += @("--rules-repo", $RulesRepo)
}

Push-Location $AidiskDir
try {
    & cargo @args
}
finally {
    Pop-Location
}
