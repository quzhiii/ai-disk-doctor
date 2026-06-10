#!/usr/bin/env bash
set -euo pipefail

# Usage: ./scripts/governance/run-governance.sh --notifier-adapter local-file
# Usage: ./scripts/governance/run-governance.sh --notifier-adapter webhook --webhook-url https://example.test/webhook --webhook-timeout-seconds 15

REPORTS_DIR="${REPORTS_DIR:-.aidisk/reports}"
OUTPUT_DIR="${OUTPUT_DIR:-.aidisk/governance}"
MIN_GROWTH="${MIN_GROWTH:-1GB}"
MIN_GROWTH_PERCENT="${MIN_GROWTH_PERCENT:-30.0}"
NOTIFIER_ADAPTER="${NOTIFIER_ADAPTER:-local-file}"
WEBHOOK_URL="${WEBHOOK_URL:-}"
WEBHOOK_TIMEOUT_SECONDS="${WEBHOOK_TIMEOUT_SECONDS:-15}"

while [[ $# -gt 0 ]]; do
    case "$1" in
        --reports-dir)
            REPORTS_DIR="$2"
            shift 2
            ;;
        --output-dir)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        --min-growth)
            MIN_GROWTH="$2"
            shift 2
            ;;
        --min-growth-percent)
            MIN_GROWTH_PERCENT="$2"
            shift 2
            ;;
        --notifier-adapter)
            NOTIFIER_ADAPTER="$2"
            shift 2
            ;;
        --webhook-url)
            WEBHOOK_URL="$2"
            shift 2
            ;;
        --webhook-timeout-seconds)
            WEBHOOK_TIMEOUT_SECONDS="$2"
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

json_string_or_null() {
    local value="$1"
    if [[ -z "$value" || "$value" == "null" ]]; then
        printf 'null'
    else
        jq -Rn --arg value "$value" '$value'
    fi
}

new_governance_event() {
    local event_type="$1"
    local headline="$2"
    local summary_markdown="$3"
    local anomaly_summary_json="$4"
    local anomaly_report_path="$5"
    local markdown_report_path="$6"
    local top_anomaly_path_json="$7"
    local top_anomaly_growth_bytes_json="$8"
    local message="$9"
    local pending_note_path="${10}"

    jq -n \
        --arg event_type "$event_type" \
        --arg headline "$headline" \
        --arg generated_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
        --arg notifier_adapter "$NOTIFIER_ADAPTER" \
        --arg reports_dir "$RESOLVED_REPORTS_DIR" \
        --arg output_dir "$RESOLVED_OUTPUT_DIR" \
        --arg min_growth "$MIN_GROWTH" \
        --argjson min_growth_percent "$MIN_GROWTH_PERCENT" \
        --arg summary_markdown "$summary_markdown" \
        --argjson anomaly_summary "$anomaly_summary_json" \
        --arg anomaly_report_path "$anomaly_report_path" \
        --arg markdown_report_path "$markdown_report_path" \
        --argjson top_anomaly_path "$top_anomaly_path_json" \
        --argjson top_anomaly_growth_bytes "$top_anomaly_growth_bytes_json" \
        --arg message "$message" \
        --arg pending_note_path "$pending_note_path" \
        '{
            event_type: $event_type,
            headline: $headline,
            generated_at: $generated_at,
            notifier_adapter: $notifier_adapter,
            reports_dir: $reports_dir,
            output_dir: $output_dir,
            min_growth: $min_growth,
            min_growth_percent: $min_growth_percent,
            summary_markdown: $summary_markdown,
            anomaly_summary: $anomaly_summary,
            anomaly_report_path: $anomaly_report_path,
            markdown_report_path: $markdown_report_path,
            top_anomaly_path: $top_anomaly_path,
            top_anomaly_growth_bytes: $top_anomaly_growth_bytes
        }
        | if $message != "" then . + {message: $message} else . end
        | if $pending_note_path != "" then . + {pending_note_path: $pending_note_path} else . end' \
        > "$GOVERNANCE_EVENT_PATH"
}

send_notifier_event() {
    if [[ "$NOTIFIER_ADAPTER" == "webhook" ]]; then
        if [[ -z "$WEBHOOK_URL" ]]; then
            echo "Webhook notifier requires --webhook-url" >&2
            exit 1
        fi

        if [[ -f "$GOVERNANCE_EVENT_PATH" ]]; then
            local failure_artifact_path="$RESOLVED_OUTPUT_DIR/webhook-failure.json"
            if curl \
                --request POST \
                --header "Content-Type: application/json" \
                --data-binary "@$GOVERNANCE_EVENT_PATH" \
                --max-time "$WEBHOOK_TIMEOUT_SECONDS" \
                --fail \
                --silent \
                --show-error \
                "$WEBHOOK_URL" >/dev/null; then
                jq '. + {delivery_status: "delivered"}' "$GOVERNANCE_EVENT_PATH" > "$GOVERNANCE_EVENT_PATH.tmp"
                mv "$GOVERNANCE_EVENT_PATH.tmp" "$GOVERNANCE_EVENT_PATH"
            else
                jq -n \
                    --arg delivery_status "failed" \
                    --arg failed_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
                    --arg notifier_adapter "$NOTIFIER_ADAPTER" \
                    --arg webhook_url "$WEBHOOK_URL" \
                    --argjson webhook_timeout_seconds "$WEBHOOK_TIMEOUT_SECONDS" \
                    --arg error_message "Webhook delivery failed" \
                    --arg governance_event_path "$GOVERNANCE_EVENT_PATH" \
                    '{
                        delivery_status: $delivery_status,
                        failed_at: $failed_at,
                        notifier_adapter: $notifier_adapter,
                        webhook_url: $webhook_url,
                        webhook_timeout_seconds: $webhook_timeout_seconds,
                        error_message: $error_message,
                        governance_event_path: $governance_event_path
                    }' > "$failure_artifact_path"

                jq --arg webhook_failure_path "$failure_artifact_path" \
                    '. + {delivery_status: "failed", webhook_failure_path: $webhook_failure_path}' \
                    "$GOVERNANCE_EVENT_PATH" > "$GOVERNANCE_EVENT_PATH.tmp"
                mv "$GOVERNANCE_EVENT_PATH.tmp" "$GOVERNANCE_EVENT_PATH"
            fi
        else
            echo "Webhook delivery skipped because no governance event artifact exists yet." \
                > "$RESOLVED_OUTPUT_DIR/webhook-pending.txt"
        fi
    elif [[ "$NOTIFIER_ADAPTER" != "local-file" ]]; then
        echo "Notifier adapter '$NOTIFIER_ADAPTER' is reserved for future webhook/IM delivery." \
            > "$RESOLVED_OUTPUT_DIR/notifier-placeholder.txt"
    fi
}

require_tool cargo
require_tool jq
if [[ "$NOTIFIER_ADAPTER" == "webhook" ]]; then
    require_tool curl
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
AIDISK_DIR="$REPO_ROOT/aidisk"
RESOLVED_OUTPUT_DIR="$REPO_ROOT/$OUTPUT_DIR"
DEFAULT_REPORTS_DIR="$AIDISK_DIR/.aidisk/reports"
RESOLVED_REPORTS_DIR="$REPO_ROOT/$REPORTS_DIR"
GOVERNANCE_EVENT_PATH="$RESOLVED_OUTPUT_DIR/governance-event.json"

mkdir -p "$RESOLVED_OUTPUT_DIR"
mkdir -p "$RESOLVED_REPORTS_DIR"

cd "$AIDISK_DIR"
cargo run -- scan --json > "$RESOLVED_OUTPUT_DIR/latest-scan.json"

latest_snapshot=""
if compgen -G "$DEFAULT_REPORTS_DIR/scan-*.json" >/dev/null; then
    latest_snapshot="$(find "$DEFAULT_REPORTS_DIR" -maxdepth 1 -name 'scan-*.json' -type f | sort | tail -n 1)"
fi
if [[ -n "$latest_snapshot" && "$RESOLVED_REPORTS_DIR" != "$DEFAULT_REPORTS_DIR" ]]; then
    cp "$latest_snapshot" "$RESOLVED_REPORTS_DIR/$(basename "$latest_snapshot")"
fi

anomaly_error_path="$RESOLVED_OUTPUT_DIR/latest-anomaly-error.txt"
if cargo run -- anomaly --latest --reports-dir "$RESOLVED_REPORTS_DIR" --min-growth "$MIN_GROWTH" --min-growth-percent "$MIN_GROWTH_PERCENT" --json \
    > "$RESOLVED_OUTPUT_DIR/latest-anomaly.json" 2> "$anomaly_error_path"; then
    cargo run -- anomaly --latest --reports-dir "$RESOLVED_REPORTS_DIR" --min-growth "$MIN_GROWTH" --min-growth-percent "$MIN_GROWTH_PERCENT" --markdown \
        > "$RESOLVED_OUTPUT_DIR/latest-anomaly.md"

    anomaly_count="$(jq -r '.summary.anomalies' "$RESOLVED_OUTPUT_DIR/latest-anomaly.json")"
    summary_markdown="$(cat "$RESOLVED_OUTPUT_DIR/latest-anomaly.md")"
    anomaly_summary_json="$(jq -c '.summary' "$RESOLVED_OUTPUT_DIR/latest-anomaly.json")"

    if [[ "$anomaly_count" -gt 0 ]]; then
        event_type="anomaly_found"
        headline="AI Disk governance detected $anomaly_count growth anomalies"
    else
        event_type="no_anomaly"
        headline="AI Disk governance found no growth anomalies"
    fi

    top_anomaly_path_json="$(json_string_or_null "$(jq -r '.anomalies[0].path // empty' "$RESOLVED_OUTPUT_DIR/latest-anomaly.json")")"
    top_anomaly_growth_bytes_json="$(jq -c '.anomalies[0].delta_bytes // null' "$RESOLVED_OUTPUT_DIR/latest-anomaly.json")"

    new_governance_event \
        "$event_type" \
        "$headline" \
        "$summary_markdown" \
        "$anomaly_summary_json" \
        "$RESOLVED_OUTPUT_DIR/latest-anomaly.json" \
        "$RESOLVED_OUTPUT_DIR/latest-anomaly.md" \
        "$top_anomaly_path_json" \
        "$top_anomaly_growth_bytes_json" \
        "" \
        ""
else
    anomaly_error="$(cat "$anomaly_error_path" 2>/dev/null || true)"
    if [[ "$anomaly_error" == *"requires at least two scan snapshots"* ]]; then
        pending_message="Not enough history yet. anomaly --latest requires at least two scan snapshots."
        pending_note_path="$RESOLVED_OUTPUT_DIR/latest-anomaly-pending.txt"
        echo "$pending_message" > "$pending_note_path"

        new_governance_event \
            "pending_history" \
            "AI Disk governance needs more snapshot history" \
            "$pending_message" \
            "null" \
            "" \
            "" \
            "null" \
            "null" \
            "anomaly --latest requires at least two scan snapshots" \
            "$pending_note_path"
    else
        cat "$anomaly_error_path" >&2
        exit 1
    fi
fi

send_notifier_event
