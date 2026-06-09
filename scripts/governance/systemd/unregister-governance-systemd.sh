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

SYSTEMD_USER_DIR="$HOME/.config/systemd/user"
SERVICE_FILE="$SYSTEMD_USER_DIR/$TASK_NAME.service"
TIMER_FILE="$SYSTEMD_USER_DIR/$TASK_NAME.timer"

if [[ ! -f "$TIMER_FILE" ]]; then
    echo "Error: Timer '$TASK_NAME' not found at $TIMER_FILE" >&2
    exit 1
fi

# Stop and disable timer
systemctl --user stop "$TASK_NAME.timer" || true
systemctl --user disable "$TASK_NAME.timer" || true

# Remove unit files
rm -f "$SERVICE_FILE" "$TIMER_FILE"

# Reload systemd
systemctl --user daemon-reload

echo "Successfully unregistered systemd timer '$TASK_NAME'"
