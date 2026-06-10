#!/usr/bin/env bash
set -euo pipefail

# Usage: ./scripts/governance/dedup-governance-event.sh --event-path .aidisk/governance/governance-event.json --dedup-dir .aidisk/governance/dedup --output-dir .aidisk/governance

EVENT_PATH="${EVENT_PATH:-}"
DEDUP_DIR="${DEDUP_DIR:-}"
OUTPUT_DIR="${OUTPUT_DIR:-}"

while [[ $# -gt 0 ]]; do
    case "$1" in
        --event-path)
            EVENT_PATH="$2"
            shift 2
            ;;
        --dedup-dir)
            DEDUP_DIR="$2"
            shift 2
            ;;
        --output-dir)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1" >&2
            exit 1
            ;;
    esac
done

require_tool() {
    local tool_name="$1"
    if ! command -v "$tool_name" >/dev/null 2>&1; then
        echo "Required tool '$tool_name' was not found in PATH" >&2
        exit 1
    fi
}

if [[ -z "$EVENT_PATH" ]]; then
    echo "--event-path is required" >&2
    exit 1
fi
if [[ -z "$DEDUP_DIR" ]]; then
    echo "--dedup-dir is required" >&2
    exit 1
fi
if [[ -z "$OUTPUT_DIR" ]]; then
    echo "--output-dir is required" >&2
    exit 1
fi
if [[ ! -f "$EVENT_PATH" ]]; then
    echo "Governance event not found at $EVENT_PATH" >&2
    exit 1
fi

require_tool jq
mkdir -p "$DEDUP_DIR"
mkdir -p "$OUTPUT_DIR"

event_hash=$(jq -c '{event_type, generated_at, anomaly_summary, top_anomaly_path, top_anomaly_growth_bytes, anomaly_report_path}' "$EVENT_PATH" | jq -r 'tostring | gsub("[^a-zA-Z0-9]"; "")')
hash_file="$DEDUP_DIR/$event_hash"

if [[ -f "$hash_file" ]]; then
    jq -n \
        --arg reason "duplicate event detected" \
        --arg generated_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
        --arg event_hash "$event_hash" \
        --arg governance_event_path "$EVENT_PATH" \
        '{reason: $reason, generated_at: $generated_at, event_hash: $event_hash, governance_event_path: $governance_event_path}' \
        > "$OUTPUT_DIR/dedup-skipped.json"
    echo "SKIPPED"
    exit 0
fi

echo "$(date -u +%Y-%m-%dT%H:%M:%SZ)" > "$hash_file"
echo "FRESH"
exit 0
