#!/usr/bin/env bash
set -euo pipefail

# Usage: FEISHU_WEBHOOK_URL=https://example.test/feishu ./scripts/governance/notifiers/feishu.sh --event-path .aidisk/governance/governance-event.json --output-dir .aidisk/governance

EVENT_PATH="${EVENT_PATH:-}"
OUTPUT_DIR="${OUTPUT_DIR:-}"
WEBHOOK_TIMEOUT_SECONDS="${WEBHOOK_TIMEOUT_SECONDS:-15}"
TEMPLATE_PATH="${TEMPLATE_PATH:-}"

while [[ $# -gt 0 ]]; do
    case "$1" in
        --event-path)
            EVENT_PATH="$2"
            shift 2
            ;;
        --output-dir)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        --webhook-timeout-seconds)
            WEBHOOK_TIMEOUT_SECONDS="$2"
            shift 2
            ;;
        --template)
            TEMPLATE_PATH="$2"
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
if [[ -z "$OUTPUT_DIR" ]]; then
    echo "--output-dir is required" >&2
    exit 1
fi
if [[ ! -f "$EVENT_PATH" ]]; then
    echo "governance-event.json not found at $EVENT_PATH" >&2
    exit 1
fi
if [[ -z "${FEISHU_WEBHOOK_URL:-}" ]]; then
    echo "FEISHU_WEBHOOK_URL is required for Feishu delivery" >&2
    exit 1
fi

require_tool curl
require_tool jq
mkdir -p "$OUTPUT_DIR"

failure_artifact_path="$OUTPUT_DIR/feishu-failure.json"
payload_path="$OUTPUT_DIR/feishu-payload.json"
response_path="$OUTPUT_DIR/feishu-response.json"

: > "$response_path"

if [[ -n "${TEMPLATE_PATH:-}" ]]; then
    if [[ ! -f "$TEMPLATE_PATH" ]]; then
        echo "template file not found at $TEMPLATE_PATH" >&2
        exit 1
    fi
    headline_val="$(jq -r '.headline' "$EVENT_PATH")"
    summary_val="$(jq -r '.summary_markdown' "$EVENT_PATH")"
    template_content="$(cat "$TEMPLATE_PATH")"
    template_content="${template_content//\$\{headline\}/$headline_val}"
    template_content="${template_content//\$\{summary_markdown\}/$summary_val}"
    jq -n \
        --arg text "$template_content" \
        '{msg_type: "text", content: {text: $text}}' \
        > "$payload_path"
else
    jq -n \
        --arg text "$(jq -r '.headline + "\n\n" + .summary_markdown' "$EVENT_PATH")" \
        '{msg_type: "text", content: {text: $text}}' \
        > "$payload_path"
fi

if curl \
    --request POST \
    --header "Content-Type: application/json" \
    --data-binary "@$payload_path" \
    --max-time "$WEBHOOK_TIMEOUT_SECONDS" \
    --fail \
    --silent \
    --show-error \
    --output "$response_path" \
    "$FEISHU_WEBHOOK_URL"; then
    if jq -e '((has("code") or has("StatusCode")) and ((has("code") | not) or .code == 0) and ((has("StatusCode") | not) or .StatusCode == 0))' "$response_path" >/dev/null 2>&1; then
        jq 'del(.webhook_failure_path, .feishu_failure_path) + {delivery_status: "delivered", notifier_adapter: "feishu"}' "$EVENT_PATH" > "$EVENT_PATH.tmp"
        mv "$EVENT_PATH.tmp" "$EVENT_PATH"
    else
        jq -n \
            --arg delivery_status "failed" \
            --arg failed_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
            --arg notifier_adapter "feishu" \
            --argjson webhook_timeout_seconds "$WEBHOOK_TIMEOUT_SECONDS" \
            --arg error_message "Feishu delivery returned a non-zero response code" \
            --arg governance_event_path "$EVENT_PATH" \
            --arg feishu_response_path "$response_path" \
            '{
                delivery_status: $delivery_status,
                failed_at: $failed_at,
                notifier_adapter: $notifier_adapter,
                webhook_timeout_seconds: $webhook_timeout_seconds,
                error_message: $error_message,
                governance_event_path: $governance_event_path,
                feishu_response_path: $feishu_response_path
            }' > "$failure_artifact_path"
        jq --arg feishu_failure_path "$failure_artifact_path" \
            'del(.webhook_failure_path, .feishu_failure_path) + {delivery_status: "failed", notifier_adapter: "feishu", feishu_failure_path: $feishu_failure_path}' \
            "$EVENT_PATH" > "$EVENT_PATH.tmp"
        mv "$EVENT_PATH.tmp" "$EVENT_PATH"
    fi
else
    jq -n \
        --arg delivery_status "failed" \
        --arg failed_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
        --arg notifier_adapter "feishu" \
        --argjson webhook_timeout_seconds "$WEBHOOK_TIMEOUT_SECONDS" \
        --arg error_message "Feishu delivery failed" \
        --arg governance_event_path "$EVENT_PATH" \
        --arg feishu_response_path "$response_path" \
        '{
            delivery_status: $delivery_status,
            failed_at: $failed_at,
            notifier_adapter: $notifier_adapter,
            webhook_timeout_seconds: $webhook_timeout_seconds,
            error_message: $error_message,
            governance_event_path: $governance_event_path,
            feishu_response_path: $feishu_response_path
        }' > "$failure_artifact_path"
    jq --arg feishu_failure_path "$failure_artifact_path" \
        'del(.webhook_failure_path, .feishu_failure_path) + {delivery_status: "failed", notifier_adapter: "feishu", feishu_failure_path: $feishu_failure_path}' \
        "$EVENT_PATH" > "$EVENT_PATH.tmp"
    mv "$EVENT_PATH.tmp" "$EVENT_PATH"
fi
