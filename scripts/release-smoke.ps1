Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$RepoRoot = Split-Path -Parent $PSScriptRoot
$AidiskDir = Join-Path $RepoRoot "aidisk"

Push-Location $AidiskDir
try {
    & cargo test
    & cargo run -- scan --rules-repo "tests/fixtures/community-rules" --json
    & cargo run -- plan --safe-only --json
    & cargo run -- clean --dry-run --safe-only --markdown
    & cargo run -- doctor --markdown
    & cargo run -- diff --before "..\examples\diff-before.example.json" --after "..\examples\diff-after.example.json" --markdown
}
finally {
    Pop-Location
}
