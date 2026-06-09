#!/usr/bin/env bash
set -euo pipefail

TASK_NAME="${TASK_NAME:-aidisk-governance}"
SCHEDULE="${SCHEDULE:-0 9 * * *}"  # Daily at 09:00
NOTIFIER_ADAPTER="${NOTIFIER_ADAPTER:-local-file}"
WEBHOOK_URL="${WEBHOOK_URL:-}"

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --task-name)
            TASK_NAME="$2"
            shift 2
            ;;
        --schedule)
            SCHEDULE="$2"
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
        *)
            echo "Unknown option: $1" >&2
            exit 1
            ;;
    esac
done

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
GOVERNANCE_SCRIPT="$REPO_ROOT/scripts/governance/run-governance.sh"

# Build cron command
CRON_CMD="$SCHEDULE cd $REPO_ROOT && bash $GOVERNANCE_SCRIPT --notifier-adapter $NOTIFIER_ADAPTER"
if [[ -n "$WEBHOOK_URL" ]]; then
    CRON_CMD="$CRON_CMD --webhook-url $WEBHOOK_URL"
fi
CRON_CMD="$CRON_CMD # $TASK_NAME"

# Check if task already exists
if crontab -l 2>/dev/null | grep -q "# $TASK_NAME"; then
    echo "Error: Task '$TASK_NAME' already exists in crontab" >&2
    exit 1
fi

# Add to crontab
(crontab -l 2>/dev/null || echo "") | { cat; echo "$CRON_CMD"; } | crontab -

echo "Successfully registered cron task '$TASK_NAME' with schedule: $SCHEDULE"
