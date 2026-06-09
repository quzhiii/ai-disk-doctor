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

LABEL="com.aidisk.$TASK_NAME"

# Check if task is loaded
if launchctl list | grep -q "$LABEL"; then
    echo "Task Name: $TASK_NAME"
    echo "Label: $LABEL"
    echo "Status: Loaded"
    echo ""
    launchctl list | grep "$LABEL"
else
    echo "Error: Task '$TASK_NAME' (label: $LABEL) not found" >&2
    exit 1
fi
