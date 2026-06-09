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

echo "=== Timer Status ==="
systemctl --user status "$TASK_NAME.timer" || {
    echo "Error: Timer '$TASK_NAME.timer' not found" >&2
    exit 1
}

echo ""
echo "=== Service Status ==="
systemctl --user status "$TASK_NAME.service" || echo "(Service has not run yet)"

echo ""
echo "=== Next Scheduled Run ==="
systemctl --user list-timers "$TASK_NAME.timer"
