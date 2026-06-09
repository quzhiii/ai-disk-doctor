Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$RepoRoot = Split-Path -Parent $PSScriptRoot
$AidiskDir = Join-Path $RepoRoot "aidisk"
$FixtureUserRoot = Join-Path $AidiskDir "tests\fixtures\windows-user"
$FixtureLocalAppData = Join-Path $FixtureUserRoot "AppData\Local"
$FixtureAppData = Join-Path $FixtureUserRoot "AppData\Roaming"
$PreviousUserProfile = $env:USERPROFILE
$PreviousLocalAppData = $env:LOCALAPPDATA
$PreviousAppData = $env:APPDATA
$PreviousHome = $env:HOME
$AidiskExe = Join-Path $AidiskDir "target\debug\aidisk.exe"

Push-Location $AidiskDir
try {
    & cargo test
    & cargo build

    $env:USERPROFILE = $FixtureUserRoot
    $env:LOCALAPPDATA = $FixtureLocalAppData
    $env:APPDATA = $FixtureAppData
    $env:HOME = $FixtureUserRoot

    & $AidiskExe scan --rules-repo "tests/fixtures/community-rules" --json
    & $AidiskExe scan --large-files --min-size 500MB --root $FixtureUserRoot --json
    & $AidiskExe plan --safe-only --json
    & $AidiskExe clean --dry-run --safe-only --markdown
    & $AidiskExe doctor --markdown
    & $AidiskExe diff --before "..\examples\diff-before.example.json" --after "..\examples\diff-after.example.json" --markdown
    & $AidiskExe anomaly --before "..\examples\diff-before.example.json" --after "..\examples\diff-after.example.json" --min-growth 100MB --min-growth-percent 30 --markdown
}
finally {
    $env:USERPROFILE = $PreviousUserProfile
    $env:LOCALAPPDATA = $PreviousLocalAppData
    $env:APPDATA = $PreviousAppData
    $env:HOME = $PreviousHome
    Pop-Location
}
