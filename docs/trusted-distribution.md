# Trusted Distribution

AI Disk Doctor release artifacts are built by `.github/workflows/release-artifacts.yml` when a `v*.*.*` tag is pushed or the workflow is run manually.

The v1.8.0 artifact line is prepared as a release candidate for Owner Acceptance. Do not tag or publish v1.8.0 until approval; the existing published v1.7.0 artifact remains the older release baseline and does not contain the complete Agent Alpha runtime contract.

## Artifact Matrix

| Platform | Target | Package |
|---|---|---|
| Windows x86_64 | `x86_64-pc-windows-msvc` | `aidisk-v<VERSION>-x86_64-pc-windows-msvc.zip` |
| Windows ARM64 | `aarch64-pc-windows-msvc` | `aidisk-v<VERSION>-aarch64-pc-windows-msvc.zip` |
| Linux x86_64 | `x86_64-unknown-linux-gnu` | `aidisk-v<VERSION>-x86_64-unknown-linux-gnu.tar.gz` |
| Linux ARM64 | `aarch64-unknown-linux-gnu` | `aidisk-v<VERSION>-aarch64-unknown-linux-gnu.tar.gz` |
| macOS Intel | `x86_64-apple-darwin` | `aidisk-v<VERSION>-x86_64-apple-darwin.tar.gz` |
| macOS Apple Silicon | `aarch64-apple-darwin` | `aidisk-v<VERSION>-aarch64-apple-darwin.tar.gz` |

Each package includes:

- `aidisk` or `aidisk.exe`
- `Start-AIDiskDoctor.ps1`
- built-in `rules/`
- default `config/`
- `README.md`
- `CHANGELOG.md`
- `LICENSE-MIT`
- `LICENSE-APACHE`
- `report-schema.md`

## Verification

Every package is uploaded with checksum, SBOM, and provenance sidecars:

- `<package>.sha256`
- `<package>.sbom.cargo-metadata.json`
- `<package>.provenance.json`

Verify SHA-256 before placing `aidisk` on PATH.

Windows PowerShell:

```powershell
Get-FileHash .\aidisk-v1.8.0-x86_64-pc-windows-msvc.zip -Algorithm SHA256
Get-Content .\aidisk-v1.8.0-x86_64-pc-windows-msvc.sha256
```

Linux / macOS:

```bash
sha256sum -c aidisk-v1.8.0-x86_64-unknown-linux-gnu.sha256
shasum -a 256 -c aidisk-v1.8.0-aarch64-apple-darwin.sha256
```

Then run:

```bash
aidisk --help
aidisk scan --help
aidisk capabilities --json
aidisk explain --json --snapshot skip --category dev-artifact
```

For v1.8.0, `aidisk capabilities --json` must advertise `agent-capabilities-v1` and `explainability-v1`; `aidisk explain --json --snapshot skip` must return `agent-diagnostic-cli-v1` with embedded `explainability-v1` and `snapshot.persisted = false`.

## Install

Windows PowerShell:

```powershell
Expand-Archive .\aidisk-v1.8.0-x86_64-pc-windows-msvc.zip -DestinationPath "$env:LOCALAPPDATA\aidisk" -Force
$env:Path = "$env:LOCALAPPDATA\aidisk\aidisk-v1.8.0-x86_64-pc-windows-msvc;$env:Path"
aidisk --help
```

Linux / macOS:

```bash
tar -xzf aidisk-v1.8.0-x86_64-unknown-linux-gnu.tar.gz
sudo install -m 0755 aidisk-v1.8.0-x86_64-unknown-linux-gnu/aidisk /usr/local/bin/aidisk
aidisk --help
```

## Upgrade

Download the new package, verify the checksum, replace the old `aidisk` binary, and rerun:

```bash
aidisk --help
aidisk scan --help
aidisk capabilities --json
aidisk explain --json --snapshot skip --category dev-artifact
```

## Uninstall

Remove the installed binary and optional local data.

Windows PowerShell:

```powershell
Remove-Item "$env:LOCALAPPDATA\aidisk" -Recurse -Force
```

Linux / macOS:

```bash
sudo rm -f /usr/local/bin/aidisk
```

Optional project-local data is stored under `.aidisk/` in directories where you run the tool.

## Package Manager Drafts

Homebrew tap and winget publishing are not live yet. Draft templates are tracked in `packaging/homebrew/aidisk.rb` and `packaging/winget/AI-Disk-Doctor.yaml` so release metadata can be filled in after GitHub artifacts are published.

crates.io publishing is deferred until the CLI name, package description, release signing expectations, and support policy are stable.
