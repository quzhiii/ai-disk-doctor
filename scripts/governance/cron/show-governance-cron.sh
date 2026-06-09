#!/usr/bin/env bash
set -euo pipefail

TASK_NAME="${TASK_NAME:-aidisk-governance}"

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --task-name)
            TASK_NAME="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1" >&2
            exit 1
            ;;
    esac
done

# Find task in crontab
CRON_LINE=$(crontab -l 2>/dev/null | grep "# $TASK_NAME" || echo "")

if [[ -z "$CRON_LINE" ]]; then
    echo "Error: Task '$TASK_NAME' not found in crontab" >&2
    exit 1
fi

echo "Task Name: $TASK_NAME"
echo "Status: Registered"
echo "Schedule: $CRON_LINE"
echo ""
echo "Note: cron does not track last/next run times. Use system logs to check execution history."
