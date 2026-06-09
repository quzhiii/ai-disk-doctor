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

if [[ ! -f "$SERVICE_FILE" ]]; then
    echo "Error: Service '$TASK_NAME' not found at $SERVICE_FILE" >&2
    exit 1
fi

echo "Starting systemd service '$TASK_NAME' now..."

# Start the service immediately
systemctl --user start "$TASK_NAME.service"

echo ""
echo "Service started. Check status with:"
echo "  systemctl --user status $TASK_NAME.service"
echo ""
echo "View logs with:"
echo "  journalctl --user -u $TASK_NAME.service"
