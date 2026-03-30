#!/usr/bin/env bash
# Ralph — Autonomous AI Agent Loop for zsh-highlighter
# Usage: ./scripts/ralph/ralph.sh [max_iterations]

set -euo pipefail

MAX_ITERATIONS="${1:-10}"
BRANCH="ralph/zsh-highlighter-v1"
PROJECT_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"

cd "$PROJECT_ROOT"

# Ensure git repo has at least one commit
if ! git rev-parse HEAD >/dev/null 2>&1; then
    echo "Creating initial commit..."
    git add -A
    git commit -m "chore: initial project setup with spec and ralph config"
fi

# Ensure we're on the right branch
CURRENT_BRANCH=$(git branch --show-current 2>/dev/null || echo "")
if [ "$CURRENT_BRANCH" != "$BRANCH" ]; then
    if git show-ref --verify --quiet "refs/heads/$BRANCH" 2>/dev/null; then
        git checkout "$BRANCH"
    else
        git checkout -b "$BRANCH"
    fi
fi

echo "=== Ralph Loop Starting ==="
echo "Branch: $BRANCH"
echo "Max iterations: $MAX_ITERATIONS"
echo ""

for i in $(seq 1 "$MAX_ITERATIONS"); do
    echo "=== Iteration $i/$MAX_ITERATIONS ==="

    # Check if all stories are complete
    INCOMPLETE=$(python3 -c "
import json
with open('prd.json') as f:
    data = json.load(f)
incomplete = [s for s in data['userStories'] if not s['passes']]
print(len(incomplete))
" 2>/dev/null || echo "unknown")

    if [ "$INCOMPLETE" = "0" ]; then
        echo ""
        echo "=== All stories complete! ==="
        exit 0
    fi

    echo "Incomplete stories: $INCOMPLETE"
    echo "Launching Claude Code..."
    echo ""

    # Launch Claude Code with the ralph instructions
    claude --print --dangerously-skip-permissions \
        "You are in a Ralph autonomous loop iteration $i. Follow the instructions in scripts/ralph/CLAUDE.md exactly. Read progress.txt first, then SPEC.md, then all existing source files, then find and implement the next incomplete story from prd.json." \
        2>&1 || true

    echo ""
    echo "=== Iteration $i complete ==="
    echo ""
done

echo "=== Ralph loop finished (max iterations reached) ==="
