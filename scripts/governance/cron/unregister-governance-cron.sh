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

# Check if task exists
if ! crontab -l 2>/dev/null | grep -q "# $TASK_NAME"; then
    echo "Error: Task '$TASK_NAME' not found in crontab" >&2
    exit 1
fi

# Remove from crontab
crontab -l 2>/dev/null | grep -v "# $TASK_NAME" | crontab -

echo "Successfully unregistered cron task '$TASK_NAME'"
