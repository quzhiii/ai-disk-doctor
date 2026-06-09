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

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
GOVERNANCE_SCRIPT="$REPO_ROOT/scripts/governance/run-governance.sh"

echo "Starting governance task '$TASK_NAME' now..."

# Execute the governance script directly
bash "$GOVERNANCE_SCRIPT"

echo ""
echo "Task execution completed."
