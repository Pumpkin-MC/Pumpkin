#!/bin/bash
# start-session.sh <agent> <task-description>
# Prepares the environment for a Claude Code agent session

set -e

AGENT=$1
shift
TASK="$*"
TODAY=$(date +%Y-%m-%d)

VALID_AGENTS="architect protocol world entity redstone storage items core plugin"

if [ -z "$AGENT" ] || [ -z "$TASK" ]; then
    echo "Usage: ./start-session.sh <agent> <task-description>"
    echo "Agents: $VALID_AGENTS"
    exit 1
fi

# Validate agent name
if ! echo "$VALID_AGENTS" | grep -qw "$AGENT"; then
    echo "ERROR: Unknown agent '$AGENT'"
    echo "Valid agents: $VALID_AGENTS"
    exit 1
fi

# Set current agent marker
echo "$AGENT" > .current-agent

# Ensure today's log directory exists
mkdir -p "logs/$TODAY"

# Determine next sequence number
LAST_SEQ=$(ls "logs/$TODAY/" 2>/dev/null | grep -oP '^\d+' | sort -n | tail -1)
NEXT_SEQ=$(printf "%03d" $(( ${LAST_SEQ:-0} + 1 )))

# Calculate yesterday
YESTERDAY=$(date -d "yesterday" +%Y-%m-%d 2>/dev/null || date -v-1d +%Y-%m-%d 2>/dev/null || echo "none")

# Count context files
CONTEXT_COUNT=0

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "  MINECRAFT-SERVER-RUST — Agent Session"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "  Agent:      $AGENT"
echo "  Task:       $TASK"
echo "  Date:       $TODAY"
echo "  Log file:   logs/$TODAY/${NEXT_SEQ}_${AGENT}_<description>.md"
echo "  Contract:   contracts/${AGENT}.toml"
echo ""
echo "  Context to load:"
echo "  ─────────────────"

# List today's logs
if ls "logs/$TODAY/"*.md 1>/dev/null 2>&1; then
    echo "  Today's logs:"
    for f in "logs/$TODAY/"*.md; do
        echo "    ✓ $(basename "$f")"
        CONTEXT_COUNT=$((CONTEXT_COUNT + 1))
    done
else
    echo "  Today's logs: (none yet — you're first today)"
fi

# List yesterday's logs
if [ "$YESTERDAY" != "none" ] && [ -d "logs/$YESTERDAY" ]; then
    echo "  Yesterday's last 5:"
    for f in $(ls "logs/$YESTERDAY/"*.md 2>/dev/null | tail -5); do
        echo "    ✓ $(basename "$f")"
        CONTEXT_COUNT=$((CONTEXT_COUNT + 1))
    done
else
    echo "  Yesterday's logs: (none available)"
fi

echo "  Decision logs:"
echo "    ✓ logs/decisions/${AGENT}.md"
echo "    ✓ logs/decisions/architect.md"
CONTEXT_COUNT=$((CONTEXT_COUNT + 2))

echo ""
echo "  Total context files: $CONTEXT_COUNT"
echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "  REMEMBER: Read ALL context before writing ANY code."
echo "  REMEMBER: Write your session log before committing."
echo "═══════════════════════════════════════════════════════════════"
echo ""

# Export environment for Claude Code
export MC_AGENT="$AGENT"
export MC_TASK="$TASK"
export MC_LOG_SEQ="$NEXT_SEQ"
export MC_TODAY="$TODAY"
export MC_YESTERDAY="$YESTERDAY"
