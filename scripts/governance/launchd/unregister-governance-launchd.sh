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

PLIST_DIR="$HOME/Library/LaunchAgents"
PLIST_PATH="$PLIST_DIR/com.aidisk.$TASK_NAME.plist"

if [[ ! -f "$PLIST_PATH" ]]; then
    echo "Error: Task '$TASK_NAME' not found at $PLIST_PATH" >&2
    exit 1
fi

# Unload the agent
launchctl unload "$PLIST_PATH"

# Remove plist file
rm "$PLIST_PATH"

echo "Successfully unregistered launchd task '$TASK_NAME'"
