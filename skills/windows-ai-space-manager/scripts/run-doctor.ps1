param(
    [switch]$Docker,
    [switch]$Wsl,
    [switch]$Ollama,
    [switch]$Playwright,
    [switch]$HuggingFace,
    [switch]$Agents,
    [switch]$ProbeTools,
    [switch]$Latest,
    [string]$ReportsDir,
    [string]$RulesRepo,
    [switch]$Json,
    [switch]$Markdown
)

Set-StrictMode -Version Latest
$InvokeAidisk = Join-Path $PSScriptRoot "invoke-aidisk.ps1"

$AidiskArgs = @("doctor")

if ($Json) { $AidiskArgs += "--json" }
elseif ($Markdown) { $AidiskArgs += "--markdown" }

if ($Docker) { $AidiskArgs += "--docker" }
if ($Wsl) { $AidiskArgs += "--wsl" }
if ($Ollama) { $AidiskArgs += "--ollama" }
if ($Playwright) { $AidiskArgs += "--playwright" }
if ($HuggingFace) { $AidiskArgs += "--huggingface" }
if ($Agents) { $AidiskArgs += "--agents" }
if ($ProbeTools) { $AidiskArgs += "--probe-tools" }
if ($Latest) { $AidiskArgs += "--latest" }
if ($ReportsDir) { $AidiskArgs += @("--reports-dir", $ReportsDir) }
if ($RulesRepo) { $AidiskArgs += @("--rules-repo", $RulesRepo) }

& $InvokeAidisk @AidiskArgs
