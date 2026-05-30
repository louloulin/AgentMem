#!/bin/bash
# agentmem-hook.sh - Claude Code Hook for AgentMem Integration
#
# This hook integrates with Claude Code to capture command context
# and store relevant information in AgentMem memory.
#
# Installation:
#   1. Copy this file to ~/.claude/hooks/
#   2. Make it executable: chmod +x ~/.claude/hooks/agentmem-hook.sh
#   3. Add to your project's CLAUDE.md or system CLAUDE.md
#
# Features:
#   - Captures command context before execution
#   - Stores session information for memory retrieval
#   - Tracks important file changes and decisions

AGENTMEM_HOOK_VERSION="1.0.0"

# Colors for output (only if terminal supports it)
if [ -t 1 ]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    BLUE='\033[0;34m'
    NC='\033[0m'
else
    RED=''
    GREEN=''
    BLUE=''
    NC=''
fi

# Log function
log() {
    echo -e "${BLUE}[AgentMem Hook]${NC} $1"
}

# Get current context
get_context() {
    # Current directory
    local dir=$(pwd)

    # Git branch (if in git repo)
    local branch=$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "N/A")

    # Recent commits (last 3)
    local recent_commits=$(git log --oneline -3 2>/dev/null | head -3 | sed 's/^/    /' || echo "    Not a git repo")

    # Current time
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')

    echo "{\"timestamp\":\"$timestamp\",\"dir\":\"$dir\",\"branch\":\"$branch\",\"recent_commits\":$(echo "$recent_commits" | head -1)}"
}

# Main hook logic
main() {
    local action="${1:-unknown}"

    case "$action" in
        "pre-command")
            log "Pre-command context captured"
            # Could capture command context here
            ;;
        "post-command")
            log "Post-command context captured"
            # Could store results in AgentMem here
            ;;
        "session-start")
            log "Session started at $(date)"
            log "Context: $(get_context)"
            ;;
        "session-end")
            log "Session ended"
            ;;
        *)
            log "Unknown action: $action"
            ;;
    esac
}

# Run main function
main "$@"
