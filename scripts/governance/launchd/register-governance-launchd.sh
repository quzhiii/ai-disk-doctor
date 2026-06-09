#!/usr/bin/env bash
set -euo pipefail

TASK_NAME="${TASK_NAME:-aidisk-governance}"
SCHEDULE_HOUR="${SCHEDULE_HOUR:-9}"
SCHEDULE_MINUTE="${SCHEDULE_MINUTE:-0}"
NOTIFIER_ADAPTER="${NOTIFIER_ADAPTER:-local-file}"
WEBHOOK_URL="${WEBHOOK_URL:-}"

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --task-name)
            TASK_NAME="$2"
            shift 2
            ;;
        --schedule-hour)
            SCHEDULE_HOUR="$2"
            shift 2
            ;;
        --schedule-minute)
            SCHEDULE_MINUTE="$2"
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
PLIST_DIR="$HOME/Library/LaunchAgents"
PLIST_PATH="$PLIST_DIR/com.aidisk.$TASK_NAME.plist"

mkdir -p "$PLIST_DIR"

# Check if already exists
if [[ -f "$PLIST_PATH" ]]; then
    echo "Error: Task '$TASK_NAME' already exists at $PLIST_PATH" >&2
    exit 1
fi

# Build program arguments
PROGRAM_ARGS=(
    "<string>bash</string>"
    "<string>$GOVERNANCE_SCRIPT</string>"
    "<string>--notifier-adapter</string>"
    "<string>$NOTIFIER_ADAPTER</string>"
)

if [[ -n "$WEBHOOK_URL" ]]; then
    PROGRAM_ARGS+=(
        "<string>--webhook-url</string>"
        "<string>$WEBHOOK_URL</string>"
    )
fi

# Generate plist
cat > "$PLIST_PATH" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.aidisk.$TASK_NAME</string>
    <key>ProgramArguments</key>
    <array>
        ${PROGRAM_ARGS[@]}
    </array>
    <key>WorkingDirectory</key>
    <string>$REPO_ROOT</string>
    <key>StartCalendarInterval</key>
    <dict>
        <key>Hour</key>
        <integer>$SCHEDULE_HOUR</integer>
        <key>Minute</key>
        <integer>$SCHEDULE_MINUTE</integer>
    </dict>
    <key>StandardOutPath</key>
    <string>$REPO_ROOT/.aidisk/governance/launchd-stdout.log</string>
    <key>StandardErrorPath</key>
    <string>$REPO_ROOT/.aidisk/governance/launchd-stderr.log</string>
</dict>
</plist>
EOF

# Load the agent
launchctl load "$PLIST_PATH"

echo "Successfully registered launchd task '$TASK_NAME'"
echo "Plist file: $PLIST_PATH"
echo "Schedule: Daily at $SCHEDULE_HOUR:$(printf '%02d' $SCHEDULE_MINUTE)"
