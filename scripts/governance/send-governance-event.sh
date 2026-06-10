#!/usr/bin/env bash
set -euo pipefail

# Usage: ./scripts/governance/send-governance-event.sh --adapter local-file --event-path .aidisk/governance/governance-event.json --output-dir .aidisk/governance
# Usage: ./scripts/governance/send-governance-event.sh --adapter webhook --event-path .aidisk/governance/governance-event.json --output-dir .aidisk/governance --webhook-url https://example.test/webhook
# Usage: FEISHU_WEBHOOK_URL=https://example.test/feishu ./scripts/governance/send-governance-event.sh --adapter feishu --event-path .aidisk/governance/governance-event.json --output-dir .aidisk/governance

ADAPTER="${ADAPTER:-local-file}"
EVENT_PATH="${EVENT_PATH:-}"
OUTPUT_DIR="${OUTPUT_DIR:-}"
WEBHOOK_URL="${WEBHOOK_URL:-}"
WEBHOOK_TIMEOUT_SECONDS="${WEBHOOK_TIMEOUT_SECONDS:-15}"

while [[ $# -gt 0 ]]; do
    case "$1" in
        --adapter)
            ADAPTER="$2"
            shift 2
            ;;
        --event-path)
            EVENT_PATH="$2"
            shift 2
            ;;
        --output-dir)
            OUTPUT_DIR="$2"
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

require_event_inputs() {
    if [[ -z "$EVENT_PATH" ]]; then
        echo "--event-path is required" >&2
        exit 1
    fi
    if [[ -z "$OUTPUT_DIR" ]]; then
        echo "--output-dir is required" >&2
        exit 1
    fi
    if [[ ! -f "$EVENT_PATH" ]]; then
        echo "governance-event.json not found at $EVENT_PATH" >&2
        exit 1
    fi
    mkdir -p "$OUTPUT_DIR"
}

mark_delivery_status() {
    local status="$1"
    jq --arg delivery_status "$status" --arg notifier_adapter "$ADAPTER" \
        'del(.webhook_failure_path, .feishu_failure_path) + {delivery_status: $delivery_status, notifier_adapter: $notifier_adapter}' \
        "$EVENT_PATH" > "$EVENT_PATH.tmp"
    mv "$EVENT_PATH.tmp" "$EVENT_PATH"
}

mark_delivery_failure() {
    local failure_path_key="$1"
    local failure_path_value="$2"
    jq --arg notifier_adapter "$ADAPTER" --arg failure_path_key "$failure_path_key" --arg failure_path_value "$failure_path_value" \
        'del(.webhook_failure_path, .feishu_failure_path) + {delivery_status: "failed", notifier_adapter: $notifier_adapter} + {($failure_path_key): $failure_path_value}' \
        "$EVENT_PATH" > "$EVENT_PATH.tmp"
    mv "$EVENT_PATH.tmp" "$EVENT_PATH"
}

send_webhook_event() {
    require_tool curl
    require_tool jq
    if [[ -z "$WEBHOOK_URL" ]]; then
        echo "webhook adapter requires --webhook-url" >&2
        exit 1
    fi

    local failure_artifact_path="$OUTPUT_DIR/webhook-failure.json"
    if curl \
        --request POST \
        --header "Content-Type: application/json" \
        --data-binary "@$EVENT_PATH" \
        --max-time "$WEBHOOK_TIMEOUT_SECONDS" \
        --fail \
        --silent \
        --show-error \
        "$WEBHOOK_URL" >/dev/null; then
        mark_delivery_status "delivered"
    else
        jq -n \
            --arg delivery_status "failed" \
            --arg failed_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
            --arg notifier_adapter "$ADAPTER" \
            --arg webhook_url "$WEBHOOK_URL" \
            --argjson webhook_timeout_seconds "$WEBHOOK_TIMEOUT_SECONDS" \
            --arg error_message "Webhook delivery failed" \
            --arg governance_event_path "$EVENT_PATH" \
            '{
                delivery_status: $delivery_status,
                failed_at: $failed_at,
                notifier_adapter: $notifier_adapter,
                webhook_url: $webhook_url,
                webhook_timeout_seconds: $webhook_timeout_seconds,
                error_message: $error_message,
                governance_event_path: $governance_event_path
            }' > "$failure_artifact_path"
        mark_delivery_failure "webhook_failure_path" "$failure_artifact_path"
    fi
}

send_feishu_event() {
    local script_dir
    script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    "$script_dir/notifiers/feishu.sh" \
        --event-path "$EVENT_PATH" \
        --output-dir "$OUTPUT_DIR" \
        --webhook-timeout-seconds "$WEBHOOK_TIMEOUT_SECONDS"
}

require_event_inputs
require_tool jq

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
dedup_result=$("$script_dir/dedup-governance-event.sh" \
    --event-path "$EVENT_PATH" \
    --dedup-dir "$OUTPUT_DIR/dedup" \
    --output-dir "$OUTPUT_DIR") || true
if [[ "$dedup_result" == "SKIPPED" ]]; then
    exit 0
fi

case "$ADAPTER" in
    local-file)
        mark_delivery_status "delivered"
        ;;
    webhook)
        send_webhook_event
        ;;
    feishu)
        send_feishu_event
        ;;
    *)
        echo "Unknown notifier adapter: $ADAPTER" >&2
        exit 1
        ;;
esac
