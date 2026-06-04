param(
    [switch]$Docker,
    [switch]$Wsl,
    [switch]$Ollama,
    [switch]$Playwright,
    [switch]$HuggingFace,
    [switch]$Agents,
    [switch]$ProbeTools,
    [string]$RulesRepo,
    [switch]$Json,
    [switch]$Markdown
)

Set-StrictMode -Version Latest
$AidiskDir = Join-Path $PSScriptRoot "..\..\..\aidisk"

$args = @("run", "--", "doctor")

if ($Json) { $args += "--json" }
elseif ($Markdown) { $args += "--markdown" }

if ($Docker) { $args += "--docker" }
if ($Wsl) { $args += "--wsl" }
if ($Ollama) { $args += "--ollama" }
if ($Playwright) { $args += "--playwright" }
if ($HuggingFace) { $args += "--huggingface" }
if ($Agents) { $args += "--agents" }
if ($ProbeTools) { $args += "--probe-tools" }
if ($RulesRepo) { $args += @("--rules-repo", $RulesRepo) }

Push-Location $AidiskDir
try {
    & cargo @args
}
finally {
    Pop-Location
}
