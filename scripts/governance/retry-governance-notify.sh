#!/usr/bin/env bash
set -euo pipefail

MAX_RETRIES="${MAX_RETRIES:-3}"
RETRY_DELAY="${RETRY_DELAY:-60}"
EVENT_PATH="${EVENT_PATH:-}"
ADAPTER="${ADAPTER:-local-file}"
OUTPUT_DIR="${OUTPUT_DIR:-}"
WEBHOOK_URL="${WEBHOOK_URL:-}"
WEBHOOK_TIMEOUT_SECONDS="${WEBHOOK_TIMEOUT_SECONDS:-15}"

while [[ $# -gt 0 ]]; do
    case "$1" in
        --max-retries)
            MAX_RETRIES="$2"
            shift 2
            ;;
        --retry-delay)
            RETRY_DELAY="$2"
            shift 2
            ;;
        --event-path)
            EVENT_PATH="$2"
            shift 2
            ;;
        --adapter)
            ADAPTER="$2"
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

if [[ -z "$EVENT_PATH" ]]; then
    echo "--event-path is required" >&2
    exit 1
fi
if [[ ! -f "$EVENT_PATH" ]]; then
    echo "Governance event not found at $EVENT_PATH" >&2
    exit 1
fi
if [[ -z "$OUTPUT_DIR" ]]; then
    echo "--output-dir is required" >&2
    exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DISPATCHER="$SCRIPT_DIR/send-governance-event.sh"

attempt=0
while [[ "$attempt" -lt "$MAX_RETRIES" ]]; do
    attempt=$((attempt + 1))
    dispatcher_args=(
        --adapter "$ADAPTER"
        --event-path "$EVENT_PATH"
        --output-dir "$OUTPUT_DIR"
        --webhook-timeout-seconds "$WEBHOOK_TIMEOUT_SECONDS"
    )
    if [[ "$ADAPTER" == "webhook" && -n "$WEBHOOK_URL" ]]; then
        dispatcher_args+=(--webhook-url "$WEBHOOK_URL")
    fi

    if "$DISPATCHER" "${dispatcher_args[@]}"; then
        exit 0
    fi

    if [[ "$attempt" -lt "$MAX_RETRIES" ]]; then
        sleep "$RETRY_DELAY"
    fi
done

jq -n \
    --argjson max_retries "$MAX_RETRIES" \
    --arg retry_delay "$RETRY_DELAY" \
    --arg adapter "$ADAPTER" \
    --arg governance_event_path "$EVENT_PATH" \
    --arg failed_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
    '{
        reason: "all notify retries exhausted",
        max_retries: $max_retries,
        retry_delay: $retry_delay,
        adapter: $adapter,
        governance_event_path: $governance_event_path,
        failed_at: $failed_at
    }' > "$OUTPUT_DIR/retry-failure.json"

exit 1
