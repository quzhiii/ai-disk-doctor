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
PLIST_PATH="$HOME/Library/LaunchAgents/$LABEL.plist"

if [[ ! -f "$PLIST_PATH" ]]; then
    echo "Error: Task '$TASK_NAME' not found at $PLIST_PATH" >&2
    exit 1
fi

echo "Starting launchd task '$TASK_NAME' (label: $LABEL) now..."

# Start the job
launchctl start "$LABEL"

echo ""
echo "Task started. Check logs at:"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
echo "  stdout: $REPO_ROOT/.aidisk/governance/launchd-stdout.log"
echo "  stderr: $REPO_ROOT/.aidisk/governance/launchd-stderr.log"
