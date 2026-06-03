# Architecture

## Runtime Flow

```text
User / AI Agent
       |
       v
  aidisk CLI (clap v4)
       |
       +-- Config Loader .............. policy.yaml, CLI args
       +-- Rules Engine ............... YAML rules + glob expansion
       +-- Scanner .................... WalkDir with depth limits
       +-- Planner .................... Risk filter + sensitive path blocking
       +-- Cleaner .................... Quarantine / restore with cross-disk fallback
       +-- Doctor ..................... Topic-specific analyzers
       +-- Diff Engine ................ Snapshot comparison
       +-- Reporter ................... JSON / Markdown output
```

## Components

### Config Loader
- Loads `policy.yaml` for default thresholds (e.g., `min_size_bytes`, `max_scan_depth`)
- Merges CLI arguments to override policy settings
- Validates configuration before execution

### Rules Engine
- Parses YAML rule definitions from built-in rules + `--rules-repo`
- Expands glob patterns (e.g., `%USERPROFILE%\projects\**\.playwright-browsers`)
- Resolves environment variables in paths
- Caches remote rules repositories in `.aidisk/rules-repos/`

### Scanner
- Uses `WalkDir` with configurable `max_scan_depth` (default: 20)
- Checks path existence before reporting
- Calculates directory sizes recursively
- Groups findings by category and risk level
- Automatically saves snapshot to `.aidisk/reports/scan-*.json`

### Planner
- Filters findings by risk level (`safe`, `careful`, `dangerous`)
- Applies `skip-modified-within-minutes` threshold
- Blocks sensitive paths (system directories, active user data)
- Generates action groups with destination paths for quarantine

### Cleaner
- **Quarantine**: Moves files to `--quarantine-root` with index tracking
  - Cross-disk fallback: `rename` fails → `copy + delete`
  - Generates quarantine index (JSON) for restore
- **Restore**: Reads quarantine index, validates structure
  - Skips conflicts (destination exists) → `skipped-conflict`
  - Validates root/results/status paths before execution
- All mutations require explicit `--yes`; default is `--dry-run`

### Doctor
- Analyzes specific topics: Docker, WSL, Ollama, Playwright, Hugging Face
- Generates policy-aware recommendations
- Explains empty results and missing paths
- Outputs actionable cleanup suggestions per topic

### Diff Engine
- Compares two scan snapshots
- Classifies changes: `grew`, `shrunk`, `appeared`, `disappeared`
- Uses `exists` field (not `size_bytes`) to avoid false positives
- Auto-discovers latest two snapshots via `latest_scan_pair`

### Reporter
- JSON output: Machine-parseable, schema-stable
- Markdown output: Human-readable with tables and sections
- Both formats include: summary, findings, top items, recommendations

## Data Flow

```text
Rules YAML
    |
    v
Rules Engine --> Expanded Paths
                    |
                    v
Scanner --> Findings (path, size, risk, category)
              |
              +---> Planner --> Cleanup Plan
              |                     |
              |                     v
              |                 Cleaner (dry-run / quarantine / restore)
              |
              +---> Doctor --> Topic Analysis
              |
              +---> Diff Engine --> Change Report
              |
              v
           Reporter --> JSON / Markdown
```

## Security Model

1. **Read-First** — All discovery commands are read-only
2. **Explicit Consent** — Mutation requires `--yes` flag
3. **Quarantine Pattern** — Files are moved, not deleted; recoverable via index
4. **Path Validation** — Restore validates index structure before touching filesystem
5. **Conflict Avoidance** — Never overwrites existing files during restore
6. **URL Safety** — `--rules-repo` rejects `http://`, `file://`, `localhost`, and private IP ranges

## Extension Points

- **Custom Rules** — `--rules-repo` accepts local directory or HTTPS git URL
- **Policy Override** — `policy.yaml` allows tuning thresholds without code changes
- **New Topics** — Doctor topics are modular; add new YAML rule categories
- **Output Formats** — Reporter trait allows adding new output serializers

## Design Constraints

- Unknown paths are reported, never processed
- All mutation commands default to dry-run
- No hardcoded paths — everything is rule-driven
- Agent-friendly structured output
- Cross-platform path handling (Windows primary, extensible)
