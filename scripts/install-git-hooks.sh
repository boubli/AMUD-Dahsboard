#!/usr/bin/env bash
# Install a pre-push hook that runs scripts/ci-check.sh (same checks as GitHub CI).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
HOOK="$ROOT/.git/hooks/pre-push"

cat > "$HOOK" << 'EOF'
#!/usr/bin/env bash
set -euo pipefail
ROOT="$(git rev-parse --show-toplevel)"
if [[ -f "$ROOT/scripts/ci-check.sh" ]]; then
  bash "$ROOT/scripts/ci-check.sh"
else
  echo "scripts/ci-check.sh missing — run cargo fmt --all before push"
  exit 1
fi
EOF

chmod +x "$HOOK"
chmod +x "$ROOT/scripts/ci-check.sh"
echo "Installed pre-push hook -> $HOOK"
