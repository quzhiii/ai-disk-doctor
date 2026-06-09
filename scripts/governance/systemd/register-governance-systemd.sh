#!/usr/bin/env bash
set -euo pipefail

TASK_NAME="${TASK_NAME:-aidisk-governance}"
SCHEDULE="${SCHEDULE:-*-*-* 09:00:00}"  # Daily at 09:00
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
SYSTEMD_USER_DIR="$HOME/.config/systemd/user"
SERVICE_FILE="$SYSTEMD_USER_DIR/$TASK_NAME.service"
TIMER_FILE="$SYSTEMD_USER_DIR/$TASK_NAME.timer"

mkdir -p "$SYSTEMD_USER_DIR"

# Check if already exists
if [[ -f "$SERVICE_FILE" ]] || [[ -f "$TIMER_FILE" ]]; then
    echo "Error: Task '$TASK_NAME' already exists in $SYSTEMD_USER_DIR" >&2
    exit 1
fi

# Build ExecStart command
EXEC_START="bash $GOVERNANCE_SCRIPT --notifier-adapter $NOTIFIER_ADAPTER"
if [[ -n "$WEBHOOK_URL" ]]; then
    EXEC_START="$EXEC_START --webhook-url $WEBHOOK_URL"
fi

# Generate .service file
cat > "$SERVICE_FILE" << EOF
[Unit]
Description=AI Disk Doctor Governance ($TASK_NAME)

[Service]
Type=oneshot
WorkingDirectory=$REPO_ROOT
ExecStart=$EXEC_START
EOF

# Generate .timer file
cat > "$TIMER_FILE" << EOF
[Unit]
Description=Timer for AI Disk Doctor Governance ($TASK_NAME)

[Timer]
OnCalendar=$SCHEDULE
Persistent=true

[Install]
WantedBy=timers.target
EOF

# Reload systemd, enable and start timer
systemctl --user daemon-reload
systemctl --user enable "$TASK_NAME.timer"
systemctl --user start "$TASK_NAME.timer"

echo "Successfully registered systemd timer '$TASK_NAME'"
echo "Service file: $SERVICE_FILE"
echo "Timer file: $TIMER_FILE"
echo "Schedule: $SCHEDULE"
