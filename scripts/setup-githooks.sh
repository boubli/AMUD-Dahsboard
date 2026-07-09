#!/usr/bin/env bash
# Enable repo git hooks (run once after clone).
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

git config core.hooksPath .githooks
chmod +x .githooks/pre-commit .githooks/pre-push 2>/dev/null || true
chmod +x scripts/ci-check.sh 2>/dev/null || true

echo "Git hooks enabled: core.hooksPath=.githooks"
echo "  pre-commit  — cargo fmt --all (auto-stage *.rs)"
echo "  pre-push    — scripts/ci-check.sh (fmt + clippy + test, mirrors CI)"
echo ""
echo "Full unix agent code is only compiled on Linux CI; use WSL or wait for GitHub Actions on Windows."
