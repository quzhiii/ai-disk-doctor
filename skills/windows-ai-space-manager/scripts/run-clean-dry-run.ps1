param(
    [switch]$SafeOnly,
    [string]$Category,
    [string]$RulesRepo,
    [string]$QuarantineRoot,
    [switch]$Json,
    [switch]$Markdown
)

Set-StrictMode -Version Latest
$InvokeAidisk = Join-Path $PSScriptRoot "invoke-aidisk.ps1"

$AidiskArgs = @("clean", "--dry-run")

if ($Json) { $AidiskArgs += "--json" }
elseif ($Markdown) { $AidiskArgs += "--markdown" }

if ($SafeOnly) { $AidiskArgs += "--safe-only" }
if ($Category) { $AidiskArgs += @("--category", $Category) }
if ($RulesRepo) { $AidiskArgs += @("--rules-repo", $RulesRepo) }
if ($QuarantineRoot) { $AidiskArgs += @("--quarantine-root", $QuarantineRoot) }

& $InvokeAidisk @AidiskArgs
