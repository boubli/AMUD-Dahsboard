#!/usr/bin/env bash
# Enable repo git hooks (run once after clone).
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

git config core.hooksPath .githooks
chmod +x .githooks/pre-commit 2>/dev/null || true

echo "Git hooks enabled: core.hooksPath=.githooks"
echo "Pre-commit will run 'cargo fmt --all' automatically before each commit."
